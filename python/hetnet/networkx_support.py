import matplotlib.pyplot as pyplot
import matplotlib.patches
import networkx

from . import core


def to_networkx_graph(graph: core.Graph) -> networkx.Graph:
    out = networkx.DiGraph()
    idx = graph._raw_index
    if idx is not None and len(idx) == 1:
        for node in graph.node_list():
            name = graph.node_properties(node.uid)[idx[0]]
            out.add_node(node.uid, label=name)
    for edge in graph.edge_list():
        out.add_edge(edge.source, edge.destination, weight=edge.weight)
    return out


def draw(graph: core.Graph | networkx.Graph):
    if isinstance(graph, core.Graph):
        graph = to_networkx_graph(graph)

    fig, ax = pyplot.subplots()

    pos = networkx.spring_layout(graph)
    weights = list(networkx.get_edge_attributes(graph, 'weight').values())
    max_weight = max(weights)

    networkx.draw_networkx_nodes(
        graph,
        pos,
        #with_labels=True,
        node_color='lightblue',
        node_size=5000,
        ax=ax
    )
    labels = networkx.get_node_attributes(graph, 'label')
    networkx.draw_networkx_labels(graph, pos, labels, ax=ax)

    for u, v, data in graph.edges(data=True):
        w = data['weight']
        color = pyplot.cm.viridis(w / max_weight)       # type: ignore
        patch = matplotlib.patches.FancyArrowPatch(
            posA=pos[u], posB=pos[v],                   # type: ignore
            connectionstyle='arc3,rad=0.15',
            color=color,
            arrowstyle='->',
            linewidth=10 * w / max_weight,
            alpha=0.8
        )
        ax.add_patch(patch)     # type: ignore

    return fig, ax
