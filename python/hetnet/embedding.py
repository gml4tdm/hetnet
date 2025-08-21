import typing

import torch
#import torch_geometric.nn

from .core import Graph, NodeRef
from . import _node2vec


def node2vec(g: Graph, *,
             # Random Walk & Sample Generation Settings
             weighted: bool = True,
             embedding_size: int = 128,
             walk_length: int = 20,
             context_size: int = 10,
             walks_per_node: int = 1,
             p: float = 1.0,
             q: float = 1.0,
             num_negative_samples: int = 1,
             negative_sampling_strategy: typing.Literal['unigram', 'uniform'] = 'uniform',
             unigram_walks_per_node: int = 5,
             # Training Settings
             learning_rate: float = 0.01,
             batch_size: int = 32,
             sparse: bool = False,
             epochs: int = 5,
             num_workers: int = 1,
             device_hint: str | None = None,
             fast_walker: bool = False,
             n_workers: int = 1) -> tuple[torch.Tensor, dict[NodeRef, int]]:
    if device_hint is None:
        device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
    else:
        device = torch.device(device_hint)

    #mapping, edge_index = _to_torch_info(g)
    # model = torch_geometric.nn.Node2Vec(
    #     edge_index,
    #     embedding_dim=embedding_size,
    #     walk_length=walk_length,
    #     context_size=context_size,
    #     walks_per_node=walks_per_node,
    #     p=p,
    #     q=q,
    #     num_negative_samples=num_negative_samples,
    #     sparse=sparse,
    # )
    model = _node2vec.Node2Vec(
        g,
        weighted=weighted,
        embedding_dim=embedding_size,
        walk_length=walk_length,
        context_size=context_size,
        walks_per_node=walks_per_node,
        p=p,
        q=q,
        num_negative_samples=num_negative_samples,
        negative_sampling_strategy=negative_sampling_strategy,
        unigram_walks_per_node=unigram_walks_per_node,
        sparse=sparse,
        fast_walker=fast_walker,
        n_workers=n_workers
    )
    model = model.to(device)

    loader = model.loader(batch_size=batch_size, shuffle=True, num_workers=num_workers)

    if sparse:
        optimiser = torch.optim.SparseAdam(model.parameters(), lr=learning_rate)
    else:
        optimiser = torch.optim.Adam(model.parameters(), lr=learning_rate)

    for epoch in range(epochs):
        total_loss = 0
        for pos_rw, neg_rw in loader:
            optimiser.zero_grad()
            loss = model.loss(pos_rw.to(device), neg_rw.to(device))
            loss.backward()
            optimiser.step()
            total_loss += loss.item()
        print(f'Epoch {epoch}: {total_loss / len(loader):.4f}')

    embeddings = model.embedding.weight.detach().cpu()
    return embeddings, model.node_to_index_mapping


# def _to_torch_info(g: Graph):
#     mapping = {}
#     edge_index_from = []
#     edge_index_to = []

#     for node, edges in g.edges_by_node().items():
#         fr_uid = mapping.setdefault(node, len(mapping))
#         for edge in edges:
#             to_uid = mapping.setdefault(edge.destination, len(mapping))
#             edge_index_from.append(fr_uid)
#             edge_index_to.append(to_uid)

#     return mapping, torch.tensor([edge_index_from, edge_index_to])
