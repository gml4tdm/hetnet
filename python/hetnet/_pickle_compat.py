from ._hetnet import Graph, GraphBuilder


def to_pickleble_data(g: Graph):
    nodes = []
    edges = []
    node_uid_mapping = {
        node.uid: i for i, node in enumerate(g.node_list())
    }
    for node in g.node_list():
        nodes.append(
            (
                node_uid_mapping[node.uid],
                node.type,
                g.node_properties(node.uid)
            )
        )
    for edge in g.edge_list():
        edges.append(
            (
                node_uid_mapping[edge.source],
                node_uid_mapping[edge.destination],
                edge.type,
                g.edge_properties(edge.uid)
            )
        )
    return nodes, edges


def from_pickleble_data(nodes, edges):
    builder = GraphBuilder()
    mapping = {}
    for uid, kind, props in nodes:
        mapping[uid] = builder.add_node(
            type=kind, properties=props
        )
    for fr, to, kind, props in edges:
        builder.add_edge(
            mapping[fr], mapping[to],
            type=kind, properties=props
        )
    return builder.build()
