import typing

import torch

from .core import Graph
from .utils.containers import ObjectIdMapping
from .utils.rng import AliasSampler


class LINEModel(torch.nn.Module):
    """LINE Implementation.

    Based on the paper:
    LINE: Large-scale Information Network Embedding
    https://arxiv.org/abs/1503.03578
    """

    def __init__(self,
                 g: Graph,
                 *,
                 order: int = 1,
                 weighted: bool = True,
                 embedding_size: int = 128,
                 num_negative_samples: int = 5,
                 sparse: bool = False,
                 device: str | None = None):
        super().__init__()
        if order not in (1, 2):
            raise ValueError('LINE only supports orders 1 and 2')
        self.order = order
        self.weighted = weighted
        self.embedding_size = embedding_size
        self.num_negative_samples = num_negative_samples
        self.num_nodes = len(g.node_list())
        self.num_edges = len(g.edge_list())
        self.sparse = sparse
        self._node_embeddings = torch.nn.Embedding(
            self.num_nodes, self.embedding_size, sparse=sparse
        )
        if self.order == 2:
            self._context_embeddings = torch.nn.Embedding(
                self.num_nodes, self.embedding_size, sparse=sparse
            )
        else:
            self._context_embeddings = None
        node_mapping = ObjectIdMapping()
        nodes = g.node_list()
        self._nodes = [node_mapping[node.uid] for node in nodes]
        self.node_to_index_mapping = {
            ref.uid: i for ref, i in zip(nodes, self._nodes)
        }
        edges = g.edges_by_node()
        self._node_sampler = AliasSampler(torch.tensor([
            pow(
                sum(
                    edge.weight if weighted else 1.0
                    for edge in edges[node.uid]
                ),
                3/4
            )
            for node in g.node_list()
        ]))

        edges = []
        edge_weights = []
        for edge in g.edge_list():
            edges.append(
                [node_mapping[edge.source], node_mapping[edge.destination]]
            )
            edge_weights.append(
                edge.weight if self.weighted else 1.0
            )
        self._edges = torch.tensor(edges, dtype=torch.long)
        self._edge_weights = torch.tensor(edge_weights)
        self._edge_sampler = AliasSampler(self._edge_weights)

    def to(self, *args, **kwargs) -> 'LINEModel':
        super().to(*args, **kwargs)

        if len(args) > 0 and isinstance(args[0], (torch.device, str)):
            self.device = args[0]
        elif "device" in kwargs and kwargs["device"] is not None:
            self.device = kwargs["device"]
        else:
            self.device = next(self.parameters()).device

        return self

    @property
    def embedding(self):
        emb = self._node_embeddings.weight.detach().cpu()
        return torch.nn.functional.normalize(emb, p=2, dim=1)

    def reset_parameters(self):
        self._node_embeddings.reset_parameters()
        if self._context_embeddings is not None:
            self._context_embeddings.reset_parameters()

    def forward(self, batch: typing.Optional[torch.Tensor]) -> torch.Tensor:
        raise NotImplementedError(
            '.forward(...) is not implemented. '
            'Call .loss(...) directly for training.'
        )

    def loss(self, pos, neg):
        if self.order == 1:
            embedding = self._node_embeddings
        else:
            embedding = self._context_embeddings
        pos_loss = self._partial_loss(pos, 1, embedding)
        neg_loss = self._partial_loss(neg, -1, embedding)
        return -(pos_loss + neg_loss)

    def _partial_loss(self, x, alpha, embedding):
        fr, to = x[:, 0], x[:, 1]
        h_fr = self._node_embeddings(fr)
        h_to = embedding(to)
        log_probs = torch.nn.functional.logsigmoid(
            alpha * torch.sum(h_fr * h_to, dim=1)
        )
        return log_probs.mean()

    def loader(self, **kwargs):
        return torch.utils.data.DataLoader(
            range(self.num_edges), collate_fn=self._sample, **kwargs    # type: ignore
        )

    def sample_one(self):
        edge_index = self._edge_sampler.sample(1)
        node_index = self._node_sampler.sample(self.num_negative_samples)
        positives = self._edges[edge_index]
        from_nodes = torch.full((self.num_negative_samples,), positives[0, 0])
        negatives = torch.column_stack([from_nodes, node_index])
        return positives, negatives

    def _sample(self,
                batch: list[int] | torch.Tensor):
        if not isinstance(batch, torch.Tensor):
            batch = torch.tensor(batch)
        n_positive = batch.size(0)
        edge_index = self._edge_sampler.sample(n_positive).to(self.device)
        n_negative = n_positive * self.num_negative_samples.to(self.device)
        node_index = self._node_sampler.sample(n_negative)
        positives = self._edges[edge_index]
        from_nodes = positives[:, 0].repeat(self.num_negative_samples)
        negatives = torch.column_stack([from_nodes, node_index])
        return positives, negatives
