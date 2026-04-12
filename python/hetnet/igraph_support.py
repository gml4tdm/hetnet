import igraph

from . import core


class _IdMapper(dict):
    def __missing__(self, key):
        n = len(self)
        self[key] = n
        return n

    def to_strict_dict(self):
        return {k: v for k, v in self.items()}


def to_igraph_components(g: core.Graph):
    return _to_igraph_components(g)[0]


def to_igraph_components_and_mapping(g: core.Graph):
    return _to_igraph_components(g)


def from_igraph_components(components):
    edge_list, weights = components
    ig = igraph.Graph(edge_list)
    ig.es['weight'] = weights
    return ig


def _to_igraph_components(g: core.Graph):
    mapper = _IdMapper()
    edges = g.edge_list()
    edge_list = [
        (mapper[edge.source], mapper[edge.destination])
        for edge in edges
    ]
    weights = [edge.weight for edge in edges]
    return (edge_list, weights), mapper.to_strict_dict()


def to_igraph_graph(g: core.Graph) -> igraph.Graph:
    ig, _ = _to_igraph_graph(g)
    return ig


def _to_igraph_graph(g: core.Graph):
    components, mapper = _to_igraph_components(g)
    return from_igraph_components(components), mapper


def personalised_pagerank(g: igraph.Graph | core.Graph,
                          alpha: float,
                          nodes: list[core.NodeRef] | list[int]) -> dict[int, float] | dict[core.NodeRef, float]:
    if isinstance(g, core.Graph):
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
    else:
        scores = g.personalized_pagerank(
            vertices=None,
            reset_vertices=nodes,
            weights='weight',
            damping=alpha,
            directed=True,
        )
        return {node: score for node, score in enumerate(scores)}
