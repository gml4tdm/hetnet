import igraph

from . import core


class _IdMapper(dict):
    def __missing__(self, key):
        n = len(self)
        self[key] = n
        return n

    def to_strict_dict(self):
        return {k: v for k, v in self.items()}


def to_igraph_graph(g: core.Graph) -> igraph.Graph:
    ig, _ = _to_igraph_graph(g)
    return ig


def _to_igraph_graph(g: core.Graph):
    mapper = _IdMapper()
    edges = g.edge_list()
    edge_list = [
        (mapper[edge.source], mapper[edge.destination])
        for edge in edges
    ]
    weights = [edge.weight for edge in edges]
    ig = igraph.Graph(edge_list)
    ig.es['weight'] = weights
    return ig, mapper.to_strict_dict()


def personalised_pagerank(g: core.Graph,
                          alpha: float,
                          nodes: list[core.NodeRef]) -> dict[core.NodeRef, float]:
    ig, mapping = _to_igraph_graph(g)
    scores = ig.personalized_pagerank(
        vertices=None,
        reset_vertices=[mapping[node] for node in nodes],
        weights='weight',
        damping=alpha,
        directed=True,
    )
    rev_mapping = {v: k for k, v in mapping.items()}
    return {rev_mapping[i]: score for i, score in enumerate(scores)}
