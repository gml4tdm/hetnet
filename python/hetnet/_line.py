import typing
import warnings

import torch

from .core import Graph, NodeRef


class LINEModel(torch.nn.Module):
    """LINE Implementation.

    Based on the paper:
    LINE: Large-scale Information Network Embedding
    https://arxiv.org/abs/1503.03578
    """

    def __init__(self,
                 g: Graph,
                 *,
                 weighted: bool = True,
                 embedding_size: int = 128,
                 num_negative_samples: int = 5,
                 sparse: bool = False,
                 num_threads: int = 1):
        super().__init__()
        self._weighted = weighted
        self._embedding_size = embedding_size
        self._num_negative_samples = num_negative_samples
        self._num_threads = num_threads
        self._num_nodes = len(g.node_list())
        self._sparse = sparse
        self._node_embeddings = torch.nn.Embedding(
            self.num_nodes, self.embedding_dim, sparse=sparse
        )
        self._context_embeddings = torch.nn.Embedding(
            self.num_nodes, self.embedding_dim, sparse=sparse
        )

    @abc.abstractmethod
    def build(self, input_size: int) -> torch.nn.Module:
        pass

    def forward(self,
                batch: typing.Optional[torch.Tensor],
                *args,
                which: typing.Literal['first', 'second', 'both'],
                normalise: bool = False) -> torch.Tensor:
        if which == 'first':
            emb = self._node_embeddings(batch)
            if normalise:
                emb = torch.nn.functional.normalize(emb, p=2, dim=1)
            return emb
        elif which == 'second':
            emb = self._context_embeddings(batch)
            if normalise:
                emb = torch.nn.functional.normalize(emb, p=2, dim=1)
            return emb
        elif which == 'both':
            if not normalise:
                warnings.warn('Joint embedding without normalisation')]
            batch_1st = self._node_embeddings(batch)
            batch_2nd = self._context_embeddings(batch)
            if normalise:
                batch_1st = torch.nn.functional.normalize(batch_1st, p=2, dim=1)
                batch_2nd = torch.nn.functional.normalize(batch_2nd, p=2, dim=1)
            combined = torch.cat([batch_1st, batch_2nd], dim=1)
            if normalise:
                combined = torch.nn.functional.normalize(combined, p=2, dim=1)
            return combined
        else:
            raise ValueError(f'Invalid value for which: {which}')
