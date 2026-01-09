import abc
import typing
import warnings

import torch

from .core import Graph, NodeRef


class AbstractNode2Vec(abc.ABC, torch.nn.Module):
    """Node2Vec Implementation.

    Based on the implementation in PyTorch Geometric:
    https://pytorch-geometric.readthedocs.io/en/latest/
        _modules/torch_geometric/nn/models/node2vec.html#Node2Vec

    With modifications to support the following additional features:
        - Weighted graphs
        - Unigram based sampling of negative samples
        - Modifyable network architecture
    """

    EPS = 1e-15

    def __init__(self,
                 graph: Graph, *,
                 weighted: bool = False,
                 embedding_dim: int,
                 walk_length: int,
                 context_size: int,
                 walks_per_node: int,
                 p: float = 1.0,
                 q: float = 1.0,
                 num_negative_samples: int = 1,
                 negative_sampling_strategy: typing.Literal['unigram', 'uniform'] = 'uniform',
                 unigram_walks_per_node: int = 5,
                 sparse: bool = False,
                 fast_walker: bool = False,
                 n_workers: int = 1):
        super().__init__()
        assert walk_length >= context_size
        self.weighted = weighted
        self.graph = graph
        self.embedding_dim = embedding_dim
        self.walk_length = walk_length
        self.context_size = context_size
        self.walks_per_node = walks_per_node
        self.p = p
        self.q = q
        self.num_negative_samples = num_negative_samples
        self.num_nodes = len(self.graph.node_list())
        self.fast_walker = fast_walker
        self.n_workers = n_workers
        self.walker = None
        if self.fast_walker:
            self.walker = self.graph.fast_walker(
                p=self.p, q=self.q, n_workers=self.n_workers
            )
        if self.fast_walker and not self.weighted:
            raise ValueError('fast_walker can only be used if weighted is True')
        if self.weighted and not self.fast_walker:
            warnings.warn(
                'Using weighted graphs without a fast walker. '
                'Consider converting the graph to a Markov graph '
                '(.to_markov()) for better performance.'
            )
        if self.n_workers > 1 and not self.fast_walker:
            warnings.warn(
                'n_workers > 1 is ignored when not using '
                'the fast walker.'
            )
        self.node_to_index_mapping = {
            node.uid: i for i, node in enumerate(self.graph.node_list())
        }
        self.index_to_node_mapping: list[NodeRef] = [None] * self.num_nodes     # type: ignore
        for node, idx in self.node_to_index_mapping.items():
            self.index_to_node_mapping[idx] = node
        self.negative_sampling_strategy = negative_sampling_strategy
        if self.negative_sampling_strategy == 'uniform':
            self.unigram_walks_per_node = None
            self.negative_sampling_weights = torch.tensor(1.0 / self.num_nodes).repeat(self.num_nodes)
        else:
            self.unigram_walks_per_node = unigram_walks_per_node
            self.negative_sampling_weights = self._unigram_probabilities().pow(3/4)
        self.cumulative_negative_sampling_weights = self.negative_sampling_weights.cumsum(dim=0)
        self.embedding = torch.nn.Embedding(
            self.num_nodes, self.embedding_dim, sparse=sparse
        )
        self.model = self.build(self.embedding_dim)
        self.reset_parameters()

    @abc.abstractmethod
    def build(self, input_size: int) -> torch.nn.Module:
        pass

    def _unigram_probabilities(self) -> torch.Tensor:
        assert self.unigram_walks_per_node is not None
        all_nodes = [node.uid for node in self.graph.node_list()]
        hist = torch.zeros(self.num_nodes)
        for _ in range(self.unigram_walks_per_node):
            if self.walker is None:
                paths = self.graph.random_walks(
                    all_nodes,
                    weighted=self.weighted,
                    path_length=self.walk_length,
                    p=self.p,
                    q=self.q,
                    n_workers=self.n_workers
                )
            else:
                paths = self.walker.walks(all_nodes, self.walk_length)
            for path in paths:
                for node in path:
                    hist[self.node_to_index_mapping[node]] += 1
        return hist / (self.num_nodes * self.unigram_walks_per_node)

    def reset_parameters(self):
        self.embedding.reset_parameters()
        if hasattr(self.model, 'reset_parameters'):
            self.model.reset_parameters()

    def forward(self, batch: typing.Optional[torch.Tensor]) -> torch.Tensor:
        w = self.embedding.weight
        x = w if batch is None else w[batch]
        return self.model(x)

    def loader(self, **kwargs):
        return torch.utils.data.DataLoader(
            range(self.num_nodes), collate_fn=self._sample, **kwargs    # type: ignore
        )

    def _sample(self, batch: list[int] | torch.Tensor) -> tuple[torch.Tensor, torch.Tensor]:
        if not isinstance(batch, torch.Tensor):
            batch = torch.tensor(batch)
        return self._sample_positives(batch), self._sample_negatives(batch)

    def _sample_positives(self, batch: torch.Tensor):
        batch = batch.repeat(self.walks_per_node)
        starts = [self.index_to_node_mapping[idx] for idx in batch.tolist()]
        if self.walker is None:
            rw = self.graph.random_walks(
                starts,
                weighted=self.weighted,
                path_length=self.walk_length,
                p=self.p,
                q=self.q,
                n_workers=self.n_workers
            )
        else:
            rw = self.walker.walks(starts, self.walk_length)
        rw = torch.tensor([
            [self.node_to_index_mapping[node] for node in walk]
            for walk in rw
        ])
        return self._apply_context_window(rw)

    def _sample_negatives(self, batch: torch.Tensor):
        # For every item in the batch, we want `num_negative_samples`,
        # but also for every item (= node), we generate `walks_per_node` total pairs
        batch = batch.repeat(self.num_negative_samples * self.walks_per_node)

        # Use binary search to generate a matrix of random walks,
        # with a walk of length `walk_length` for each item in the batch
        walks = torch.searchsorted(
            self.cumulative_negative_sampling_weights,
            torch.rand(*(batch.size(0), self.walk_length), device=batch.device)
        )

        # Prefix each random walk (the context) with the "center" node
        samples = torch.cat([batch.view(-1, 1), walks], dim=-1)

        return self._apply_context_window(samples)

    def _apply_context_window(self, samples: torch.Tensor):
        all_walks = []
        number_of_slices_per_walk = self.walk_length - self.context_size + 2
        for i in range(number_of_slices_per_walk):
            all_walks.append(samples[:, i:i + self.context_size])
        return torch.cat(all_walks, dim=0)

    def loss(self, pos, neg):
        return self._loss(pos, 0, 1) + self._loss(neg, 1, -1)

    def _loss(self, x, alpha, beta):
        start, rest = x[:, 0], x[:, 1:].contiguous()
        h_start = self.embedding(start).view(x.size(0), -1, self.embedding_dim)
        h_rest = self.embedding(rest.view(-1)).view(x.size(0), -1, self.embedding_dim)
        out = (h_start * h_rest).sum(dim=-1).view(-1)
        return -torch.log(alpha + beta*torch.sigmoid(out) + self.EPS).mean()


class Node2Vec(AbstractNode2Vec):
    """Classic Node2Vec Implementation, with a word2vec-style neural network,
    i.e. only the embedding layer is trainable.
    """

    def build(self, input_size: int) -> torch.nn.Module:
        return _NullModule(input_size)


class _NullModule(torch.nn.Module):
    def __init__(self, input_size: int):
        super().__init__()
        self.input_size = input_size

    def forward(self, x: torch.Tensor):
        return x
