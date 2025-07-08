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

    def __init__(self,
                 base_graph: _hetnet.Graph, *,
                 index: str | list[str] | None = None):
        self._graph = base_graph
        self._cache = {}
        if index is None:
            self._index = _GraphIndex(mapping={}, key=())
        else:
            keys = [index] if isinstance(index, str) else index
            mapping = {}
            for ref, _ in self._graph.node_list():
                properties = self._graph.node_properties(ref)
                node_key = tuple(properties[k] for k in keys)
                if node_key in mapping:
                    raise ValueError(f'Key {keys} is not unique')
                mapping[node_key] = ref
            self._index = _GraphIndex(mapping=mapping, key=tuple(keys))

    @property
    def index(self) -> _GraphIndex:
        return self._index

    def node_list(self) -> list[tuple[NodeRef, str]]:
        return self._graph.node_list()

    def edge_list(self) -> list[tuple[NodeRef, NodeRef, str, int]]:
        return self._graph.edge_list()

    def node_properties(self, node: NodeRef) -> dict[str, str]:
        return self._graph.node_properties(node)

    def edge_properties(self, source: NodeRef, destination: NodeRef, kind: str) -> dict[str, str]:
        return self._graph.edge_properties(source, destination, kind)

    def meta_path_subgraph(self, metapaths: dict[str, MetaPath], *, unique_nodes=True) -> Graph:
        return Graph(self._graph.meta_path_subgraph(metapaths, unique_nodes=unique_nodes))

    def random_walk(self,
                    start: NodeRef, *,
                    weighted: bool = True,
                    path_length: int = 10) -> list[NodeRef]:
        return self._graph.random_walk(start, weighted=weighted, path_length=path_length)

    def meta_path_random_walk(self,
                              start: NodeRef,
                              meta_paths: list[MetaPath], *,
                              weighted: bool = True,
                              path_length: int = 10) -> list[NodeRef]:
        return self._graph.meta_path_random_walk(
            start, meta_paths, weighted=weighted, path_length=path_length
        )

    def random_walk_distribution(self,
                                 start: NodeRef, *,
                                 weighted: bool = True,
                                 path_length: int = 10,
                                 n_iter: int = 100) -> dict[NodeRef, int]:
        return self._graph.random_walk_distribution(
            start, weighted=weighted, path_length=path_length, n_iter=n_iter
        )

    def meta_path_random_walk_distribution(self,
                                           start: NodeRef,
                                           meta_paths: list[MetaPath], *,
                                           weighted: bool = True,
                                           path_length: int = 10,
                                           n_iter: int = 100) -> dict[NodeRef, int]:
        return self._graph.meta_path_random_walk_distribution(
            start, meta_paths, weighted=weighted, path_length=path_length, n_iter=n_iter
        )

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


class _GraphIndex:

    def __init__(self, *, mapping: dict[tuple[str, ...], NodeRef], key: tuple[str, ...]):
        self._mapping = mapping
        self._key = key

    def __getitem__(self, key: str | list[str] | dict[str, str]):
        if isinstance(key, str):
            key = [key]
        if isinstance(key, dict):
            if set(key) != set(self._key):
                raise ValueError(f'Expected key fields {self._key}, got {tuple(key)}')
            key = [key[x] for x in self._key]
        return self._mapping[tuple(key)]
