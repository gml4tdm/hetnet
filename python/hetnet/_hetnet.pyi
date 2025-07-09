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
                 properties: dict[str, str] | None = None): ...
    def build(self) -> Graph: ...


class NodeRef:
    ...


class Graph:
    def node_list(self) -> list[tuple[NodeRef, str]]: ...
    def edge_list(self) -> list[tuple[NodeRef, NodeRef, str, int]]: ...
    def meta_path_subgraph(self,
                           metapaths: dict[str, MetaPath],
                           *,
                           unique_nodes=True) -> Graph: ...
    def node_properties(self, node: NodeRef) -> dict[str, str]: ...
    def edge_properties(self,
                        source: NodeRef,
                        destination: NodeRef,
                        type: str) -> dict[str, str]: ...
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


