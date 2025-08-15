import re

from .core import GraphBuilder, Graph


def parse_script(script: str) -> Graph:
    builder = GraphBuilder()
    nodes, edges = _parse_script_components(script)
    node_map = {}
    for uid, kind, properties in nodes:
        handle = builder.add_node(kind, properties=properties)
        node_map[uid] = handle
    for fr, to, kind, properties in edges:
        builder.add_edge(
            node_map[fr], node_map[to], kind, properties=properties
        )
    return builder.build(index='name')


def _parse_script_components(script: str):
    nodes = []
    edges = []
    for line in map(str.strip, script.splitlines()):
        if not line:
            continue
        if line.startswith("node"):
            name, kind, properties = _parse_node(line)
            nodes.append((name, kind, properties))
        elif line.startswith("edge"):
             for fr, to, kind, properties in _parse_edge(line):
                 edges.append((fr, to, kind, properties))
        else:
            raise ValueError(f"Invalid line: {line}")
    return nodes, edges


def _parse_node(line: str):
    pattern = re.compile(
        r'^node\s+(?P<name>\w+)\[(?P<type>\w+)\]\s*'
    )
    m = pattern.match(line)
    if not m:
        raise ValueError(f"Invalid node line: {line}")
    name = m.group("name")
    kind = m.group("type")
    remainder = line[m.end():]
    properties = _parse_properties(remainder)
    properties['name'] = name
    return name, kind, properties


def _parse_edge(line: str):
    pattern = re.compile(
        r'^edge\[(?P<type>\w+)\]\s+(?P<fr>\w+)\s+(?P<arrow>-|\<-|-\>|\<-\>)\s+(?P<to>\w+)\s*'
    )
    m = pattern.match(line)
    if not m:
        raise ValueError(f"Invalid properties line: {line}")
    fr = m.group("fr")
    to = m.group("to")
    kind = m.group("type")
    arrow = m.group("arrow")
    properties = _parse_properties(line[m.end():])
    if arrow == "-":
        yield fr, to, kind, properties
        yield to, fr, kind, properties
    elif arrow == '<->':
        yield fr, to, kind, properties
        yield to, fr, f'{kind}_rev', properties
    elif arrow == '<-':
        yield to, fr, kind, properties
    elif arrow == '->':
        yield fr, to, kind, properties
    else:
        raise NotImplementedError(f"Invalid arrow: {arrow}")


def _parse_properties(line: str):
    if not line:
        return {}
    if not line.startswith("{"):
        raise ValueError(f"Invalid properties line: {line}")
    if not line.endswith("}"):
        raise ValueError(f"Invalid properties line: {line}")
    line = line[1:-1]
    parts = map(str.strip, line.split(","))
    properties = {}
    for part in parts:
        key, remainder = _parse_string(part)
        if not remainder.startswith(':'):
            raise ValueError(f"Invalid properties line: {line}")
        value, remainder = _parse_string(remainder[1:])
        if remainder:
            raise ValueError(f"Invalid properties line: {line}")
        properties[key.strip()] = value.strip()
    return properties


def _parse_string(line: str):
    if line.startswith('"'):
        return _parse_quoted_string(line)
    else:
        return _parse_unquoted_string(line)


def _parse_unquoted_string(line: str):
    pattern = re.compile(r'^\w+')
    m = pattern.match(line)
    if not m:
        raise ValueError(f"Invalid string: {line}")
    return m.group(0), line[m.end():]


def _parse_quoted_string(line: str):
    parts = []
    pos = 0
    escaped = False
    while pos < len(line):
        pos += 1
        if escaped:
            parts.append(line[pos])
            escaped = False
        elif line[pos] == '\\':
            escaped = True
        elif line[pos] == '"':
            break
        else:
            parts.append(line[pos])
    if pos == len(line):
        raise ValueError(f"Unterminated string: {line}")
    return "".join(parts), line[pos+1:]
