from __future__ import annotations

import collections

import graphviz

try:
    from . import _hetnet
except ImportError as e:
    message = ('Could not import _hetnet extension module. '
               'Make sure the module has been built')
    raise RuntimeError(message) from e


MetaPath = _hetnet.MetaPath
NodeRef = _hetnet.NodeRef


class GraphBuilder:

    def __init__(self):
        self._builder = _hetnet.GraphBuilder()

    def add_node(self, kind: str, *, properties: dict[str, str] | None = None) -> NodeRef:
        return self._builder.add_node(kind, properties=properties)

    def add_edge(self,
                 source: NodeRef,
                 destination: NodeRef,
                 kind: str, *,
                 properties: dict[str, str] | None = None):
        self._builder.add_edge(source, destination, kind, properties=properties)

    def build(self) -> Graph:
        return Graph(self._builder.build())


class Graph:

    def __init__(self, base_graph: _hetnet.Graph):
        self._graph = base_graph
        self._cache = {}

    def node_list(self) -> list[tuple[int, str]]:
        return self._graph.node_list()

    def edge_list(self) -> list[tuple[int, int, str, int]]:
        return self._graph.edge_list()

    def node_properties(self, node: int) -> dict[str, str]:
        return self._graph.node_properties(node)

    def edge_properties(self, source: int, destination: int, kind: str) -> dict[str, str]:
        return self._graph.edge_properties(source, destination, kind)

    def meta_path_subgraph(self, metapaths: dict[str, MetaPath]) -> Graph:
        return Graph(self._graph.meta_path_subgraph(metapaths))

    def is_undirected(self) -> bool:
        if 'is_undirected' not in self._cache:
            counts = {}
            for fr, to, kind, count in self.edge_list():
                if (to, fr, kind) in counts:
                    counts[(to, fr, kind)] -= count
                else:
                    counts[(fr, to, kind)] = count
            self._cache['is_undirected'] = all(
                x == 0 for x in counts.values()
            )
        return self._cache['is_undirected']

    def to_dot_graph(self, merge_bidirectional: bool = False):
        if self.is_undirected() and merge_bidirectional:
            seen = set()
            graph = graphviz.Graph()
            for uid, kind in self.node_list():
                graph.node(str(uid), label=kind)
            for fr, to, kind, count in self.edge_list():
                if (to, fr, kind) in seen:
                    continue
                seen.add((fr, to, kind))
                graph.edge(str(fr), str(to), label=f'{kind} ({count})')
            return graph
        else:
            graph = graphviz.Digraph()
            for uid, kind in self.node_list():
                graph.node(str(uid), label=kind)
            for fr, to, kind, count in self.edge_list():
                graph.edge(str(fr), str(to), label=f'{kind} ({count})')
            return graph
