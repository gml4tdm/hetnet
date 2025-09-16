import datetime
import json
import pathlib
import time

import neo4j

from .core import Graph, GraphBuilder



def _json_loader(x):
    try:
        return json.loads(x)
    except json.JSONDecodeError as e:
        raise ValueError(f'Failed to parse JSON: {x}') from e


def load_json(filename: pathlib.Path | str, *,
              directed: bool = True,
              index=None,
              encoding='utf8') -> Graph:
    builder = GraphBuilder()
    nodes = {}
    with open(filename, encoding=encoding) as f:
        stream = filter(lambda x: bool(x.strip()), f)
        for record in map(_json_loader, stream):
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



class ProgressReporter:

    def __init__(self, callback):
        self._n_nodes = 0
        self._n_edges = 0
        self._got_nodes = False
        self._got_edges = False
        self._prev_time = time.time()
        self._callback = callback

    def log_node(self):
        self._n_nodes += 1
        self._got_nodes = True

    def log_edge(self):
        self._n_edges += 1
        self._got_edges = True

    def end_epoch(self):
        t = time.time()
        dt = t - self._prev_time
        self._prev_time = t
        if self._got_nodes and self._got_edges:
            msg = f'{self._n_nodes} nodes, {self._n_edges} edges ({dt:.2f} seconds)'
        elif self._got_nodes:
            msg = f'{self._n_nodes} nodes ({dt:.2f} seconds)'
        elif self._got_edges:
            msg = f'{self._n_edges} edges ({dt:.2f} seconds)'
        else:
            msg = f'Empty batch ({dt:.2f} seconds)'
        self._got_nodes = False
        self._got_edges = False
        self._callback(msg)


default_reporter = ProgressReporter(print)


def load_graph_streaming(uri: str, *,
                         auth: tuple[str, str] | None = None,
                         directed: bool = True,
                         index=None,
                         reporter=None) -> Graph:
    with neo4j.GraphDatabase.driver(uri, auth=auth) as driver:
        with driver.session() as session:
            builder, mapping = session.execute_read(
                _process_nodes_factory(reporter)
            )
            builder = session.execute_read(
                _process_edges_factory(builder, mapping, directed, reporter)
            )
    return builder.build(index=index)


def _process_nodes_factory(reporter):
    def _process_nodes(tx):
        builder = GraphBuilder()
        mapping = {}
        result = tx.run('MATCH (n) RETURN DISTINCT n')
        for record in result:
            if reporter is not None:
                reporter.log_node()
            node = record['n']
            if len(node.labels) != 1:
                raise ValueError('Only support single-label nodes')
            mapping[node.element_id] = builder.add_node(
                next(iter(node.labels)),
                properties=_convert_properties(node)
            )
        if reporter is not None:
            reporter.end_epoch()
        return builder, mapping
    return _process_nodes


def _process_edges_factory(builder, mapping, is_directed: bool, reporter):
    def _process_edges(tx):
        result = tx.run('MATCH (n)-[r]-(m) RETURN DISTINCT r')
        for record in result:
            if reporter is not None:
                reporter.log_edge()
            edge = record['r']
            assert len(edge.nodes) == 2
            builder.add_edge(
                mapping[edge.nodes[0].element_id],
                mapping[edge.nodes[1].element_id],
                edge.type,
                properties=_convert_properties(edge)
            )
            if not is_directed:
                builder.add_edge(
                    mapping[edge.nodes[1].element_id],
                    mapping[edge.nodes[0].element_id],
                    edge.type,
                    properties=_convert_properties(edge)
                )
        if reporter is not None:
            reporter.end_epoch()
        return builder
    return _process_edges


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
