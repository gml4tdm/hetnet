import torch_geometric.datasets

from .core import GraphBuilder


def karate_club():
    dataset = torch_geometric.datasets.KarateClub()
    builder = GraphBuilder()
    refs = {}
    for x, y in zip(dataset.edge_index[0], dataset.edge_index[1]):
        x = x.item()
        y = y.item()
        if x not in refs:
            refs[x] = builder.add_node(kind='Node')
        if y not in refs:
            refs[y] = builder.add_node(kind='Node')
        builder.add_edge(refs[x], refs[y], kind='Edge')
    return builder.build()
