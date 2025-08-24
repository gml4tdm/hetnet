from __future__ import annotations

import collections
import typing

import graphviz

try:
    from . import _hetnet
except ImportError as e:
    message = ('Could not import _hetnet extension module. '
               'Make sure the module has been built')
    raise RuntimeError(message) from e


MetaPath = _hetnet.MetaPath
NodeRef = _hetnet.NodeRef
EdgeRef = _hetnet.EdgeRef
NodeDescriptor = _hetnet.NodeDescriptor
EdgeDescriptor = _hetnet.EdgeDescriptor

FastWalker = _hetnet.FastWalker

RandomWalkEvalResult = _hetnet.RandomWalkEvalResult


class GraphBuilder:

    def __init__(self):
        self._builder = _hetnet.GraphBuilder()

    def add_node(self, kind: str, *, properties: dict[str, str] | None = None) -> NodeRef:
        return self._builder.add_node(kind, properties=properties)

    def add_edge(self,
                 source: NodeRef,
                 destination: NodeRef,
                 kind: str, *,
                 weight: float = 1.0,
                 properties: dict[str, str] | None = None):
        self._builder.add_edge(
            source, destination, kind, weight=weight, properties=properties
        )

    def build(self, index: str | list[str] | tuple[str, ...] | None = None) -> Graph:
        return Graph(self._builder.build(), index=index)


class Graph:

    def __init__(self,
                 base_graph: _hetnet.Graph, *,
                 index: str | list[str] | tuple[str, ...] | None = None):
        self._graph = base_graph
        self._cache = {}
        if index is None:
            self._index = _GraphIndex(mapping={}, key=())
            self._raw_index = None
        else:
            keys = [index] if isinstance(index, str) else index
            self._raw_index = tuple(keys)
            mapping = {}
            for descriptor in self._graph.node_list():
                ref = descriptor.uid
                properties = self._graph.node_properties(ref)
                node_key = tuple(properties[k] for k in keys)
                if node_key in mapping:
                    raise ValueError(f'Key {keys} is not unique')
                mapping[node_key] = ref
            self._index = _GraphIndex(mapping=mapping, key=tuple(keys))

    def __repr__(self) -> str:
        return repr(self._graph)

    @property
    def index(self) -> _GraphIndex:
        return self._index

    def node_info(self, node: NodeRef, /) -> NodeDescriptor:
        return self._graph.node_info(node)

    def node_list(self) -> list[NodeDescriptor]:
        return self._graph.node_list()

    def edge_list(self) -> list[EdgeDescriptor]:
        return self._graph.edge_list()

    def edges_by_node(self) -> dict[NodeRef, list[EdgeDescriptor]]:
        result = {}
        for descriptor in self.edge_list():
            col = result.setdefault(descriptor.source, [])
            col.append(descriptor)
        return result

    def node_properties(self, node: NodeRef) -> dict[str, str]:
        return self._graph.node_properties(node)

    def edge_properties(self, edge: EdgeRef) -> dict[str, str]:
        return self._graph.edge_properties(edge)

    def update_weights(self, weigths: dict[EdgeRef, float]) -> Graph:
        return Graph(self._graph.update_weights(weigths), index=self._raw_index)

    def to_markov_graph(self) -> Graph:
        return Graph(self._graph.to_markov_graph(), index=self._raw_index)

    def deduplicate_edges(
        self,
        *types: str,
        data_handling: typing.Literal['discard', 'enforce_identical'],
        weight_handling: typing.Literal['set_to_one', 'enforce_identical', 'sum_aggregate']
    ) -> Graph:
        return Graph(
            self._graph.deduplicate_edges(
                list(types), data_handling=data_handling, weight_handling=weight_handling),
            index=self._raw_index
        )

    def meta_path_subgraph(self,
                           metapaths: dict[str, MetaPath], *,
                           unique_nodes=True,
                           index: str | list[str] | tuple[str, ...] | None = None) -> Graph:
        return Graph(
            self._graph.meta_path_subgraph(metapaths, unique_nodes=unique_nodes),
            index=index
        )

    def random_walk(self,
                    start: NodeRef, *,
                    weighted: bool = True,
                    path_length: int = 10,
                    p: float = 1.0,
                    q: float = 1.0) -> list[NodeRef]:
        return self._graph.random_walk(
            start, weighted=weighted, path_length=path_length, p=p, q=q
        )

    def random_walks(self,
                     starts: list[NodeRef], *,
                     weighted: bool = True,
                     path_length: int = 10,
                     p: float = 1.0,
                     q: float = 1.0) -> list[list[NodeRef]]:
        return self._graph.random_walks(
            starts, weighted=weighted, path_length=path_length, p=p, q=q
        )


    def random_walk_distribution(self,
                                 start: NodeRef, *,
                                 weighted: bool = True,
                                 path_length: int = 10,
                                 p: float = 1.0,
                                 q: float = 1.0,
                                 n_iter: int = 100) -> dict[NodeRef, int]:
        return self._graph.random_walk_distribution(
            start, weighted=weighted, path_length=path_length, n_iter=n_iter, p=p, q=q
        )

    def evaluate_random_walk_settings(self, *,
                                      on_nodes: set[NodeRef],
                                      weighted: bool = True,
                                      path_length: int = 10,
                                      p: float = 1.0,
                                      q: float = 1.0,
                                      n_iter: int = 100) -> RandomWalkEvalResult:
        return self._graph.evaluate_random_walk_settings(
            on_nodes=on_nodes, weighted=weighted, path_length=path_length, p=p, q=q, n_iter=n_iter
        )

    def fast_walker(self, p: float = 1.0, q: float = 1.0) -> FastWalker:
        return self._graph.fast_walker(p, q)

    def to_dot_graph(self, *,
                     aggregated_edges: bool = False):
        graph = graphviz.Digraph()
        for descriptor in self.node_list():
            uid = descriptor.uid
            kind = descriptor.type
            if self._raw_index is not None and len(self._raw_index) == 1:
                prop = self.node_properties(uid)
                label = f'{prop[self._raw_index[0]]} ({kind}))'
            else:
                label = kind
            graph.node(str(uid), label=label)
        if aggregated_edges:
            agg = collections.defaultdict(float)
            for descriptor in self.edge_list():
                fr = descriptor.source
                to = descriptor.destination
                kind = descriptor.type
                count = descriptor.weight
                agg[(fr, to, kind)] += count
            for fr, to, kind in agg:
                graph.edge(str(fr), str(to), label=f'{kind} ({agg[(fr, to, kind)]})')
        else:
            for descriptor in self.edge_list():
                fr = descriptor.source
                to = descriptor.destination
                kind = descriptor.type
                count = descriptor.weight
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
