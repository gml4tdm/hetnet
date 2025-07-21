class MetaPath:
    def __init__(self, pattern: str): ...


class GraphBuilder:
    def __init__(self): ...
    def add_node(self,
                 type: str,
                 *,
                 properties: dict[str, str] | None = None) -> NodeRef: ...
    def add_edge(self,
                 source: NodeRef,
                 destination: NodeRef,
                 type: str,
                 *,
                 weight: float = 1.0,
                 properties: dict[str, str] | None = None): ...
    def build(self) -> Graph: ...


class NodeDescriptor:
    @property
    def type(self) -> str: ...
    @property
    def uid(self) -> NodeRef: ...


class EdgeDescriptor:
    @property
    def source(self) -> NodeRef: ...
    @property
    def destination(self) -> NodeRef: ...
    @property
    def type(self) -> str: ...
    @property
    def uid(self) -> EdgeRef: ...
    @property
    def weight(self) -> float: ...


class NodeRef:
    ...

class EdgeRef:
    ...


class Graph:
    def node_list(self) -> list[NodeDescriptor]: ...
    def edge_list(self) -> list[EdgeDescriptor]: ...
    def meta_path_subgraph(self,
                           metapaths: dict[str, MetaPath],
                           *,
                           unique_nodes=True) -> Graph: ...
    def node_properties(self, node: NodeRef, /) -> dict[str, str]: ...
    def edge_properties(self, edge: EdgeRef, /) -> dict[str, str]: ...
    def deduplicate_edges(self, types: list[str]) -> Graph: ...
    def random_walk(self,
                    start: NodeRef, *,
                    weighted: bool = True,
                    path_length: int = 10) -> list[NodeRef]:
        ...
    def meta_path_random_walk(self,
                              start: NodeRef,
                              meta_paths: list[MetaPath], *,
                              weighted: bool = True,
                              path_length: int = 10,
                              unique_nodes: bool = True,) -> list[NodeRef]:
        ...
    def random_walk_distribution(self,
                                 start: NodeRef, *,
                                 weighted: bool = True,
                                 path_length: int = 10,
                                 n_iter: int = 100) -> dict[NodeRef, int]:
        ...
    def meta_path_random_walk_distribution(self,
                                           start: NodeRef,
                                           meta_paths: list[MetaPath], *,
                                           weighted: bool = True,
                                           path_length: int = 10,
                                           unique_nodes: bool = True,
                                           n_iter: int = 100) -> dict[NodeRef, int]:
        ...
