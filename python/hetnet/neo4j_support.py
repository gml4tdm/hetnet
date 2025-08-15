import datetime
import json
import pathlib

import neo4j

from .core import Graph, GraphBuilder


def load_json(filename: pathlib.Path | str, *,
              directed: bool = True,
              index=None) -> Graph:
    builder = GraphBuilder()
    nodes = {}
    with open(filename) as f:
        for record in map(json.loads, f):
            if record['type'] == 'node':
                if len(record['labels']) != 1:
                    raise ValueError('Only support single-label nodes')
                nodes[record['id']] = builder.add_node(
                    kind=record['labels'][0],
                    properties={
                        k: str(v) if not isinstance(v, str) else v
                        for k, v in record.get('properties', {}).items()
                    },
                )
            elif record['type'] == 'relationship':
                props = {
                    k: str(v) if not isinstance(v, str) else v
                    for k, v in record.get('properties', {}).items()
                }
                builder.add_edge(
                    source=nodes[record['start']['id']],
                    destination=nodes[record['end']['id']],
                    kind=record['label'],
                    properties=props
                )
                if not directed:
                    builder.add_edge(
                        source=nodes[record['end']['id']],
                        destination=nodes[record['start']['id']],
                        kind=record['label'],
                        properties=props
                    )
            else:
                raise ValueError(f'Unknown record type: {record["type"]}')
    return builder.build(index=index)


def load_graph(uri: str, *,
               auth: tuple[str, str] | None = None,
               directed: bool = True,
               index=None) -> Graph:
    with neo4j.GraphDatabase.driver(uri, auth=auth) as driver:
        records, _, _ = driver.execute_query(
            """MATCH (n) OPTIONAL MATCH (n)-[r]-(m)
            RETURN COLLECT(DISTINCT n) AS nodes, COLLECT(DISTINCT r) AS relationships"""
        )
        builder = GraphBuilder()
        nodes = {}
        for record in records:
            for node in record['nodes']:
                if len(node.labels) != 1:
                    raise ValueError('Only support single-label nodes')
                nodes[node.element_id] = builder.add_node(
                    next(iter(node.labels)),
                    properties=_convert_properties(node)
                )
            for rel in record['relationships']:
                assert len(rel.nodes) == 2
                builder.add_edge(
                    nodes[rel.nodes[0].element_id],
                    nodes[rel.nodes[1].element_id],
                    rel.type,
                    properties=_convert_properties(rel)
                )
                if not directed:
                    builder.add_edge(
                        nodes[rel.nodes[1].element_id],
                        nodes[rel.nodes[0].element_id],
                        rel.type,
                        properties=_convert_properties(rel)
                    )
    return builder.build(index=index)


def _convert_properties(x):
    return {
        k: _convert_property_value(v)
        for k, v in x._properties.items()
    }


def _convert_property_value(x):
    if isinstance(x, str):
        return x
    elif isinstance(x, int):
        return str(x)
    elif isinstance(x, float):
        return str(x)
    elif isinstance(x, bool):
        return str(x)
    elif isinstance(x, neo4j.time.DateTime):
        return x.isoformat()
    else:
        raise ValueError(f'Unknown property type: {type(x)}')
