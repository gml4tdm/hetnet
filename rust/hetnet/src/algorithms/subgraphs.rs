//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Imports
//////////////////////////////////////////////////////////////////////////////////////////////////

use std::collections::{BTreeSet, HashMap};
use std::collections::hash_map::Entry;
use std::sync::Arc;
use crate::{HetNetError, HetNetResult, HeteroDiGraph, MetaPath};
use crate::builder::next_graph_id;
use crate::graph::{Edge, EdgeMetadata, GraphMetadata, Node, RawNodeRef};

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Types
//////////////////////////////////////////////////////////////////////////////////////////////////

struct SubgraphBuilder {
    nodes: Vec<Node>,
    node_mapping: HashMap<usize, usize>,
    edge_types: Vec<String>,
    edge_types_reversed: HashMap<String, usize>,
    next_edge_id: usize
}

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Type Implementation
//////////////////////////////////////////////////////////////////////////////////////////////////

impl SubgraphBuilder {
    fn new(edge_types: Vec<String>) -> Self {
        let edge_types_reversed = edge_types.iter().cloned()
            .enumerate()
            .map(|(i, v)| (v, i))
            .collect();
        Self {
            nodes: Vec::new(),
            node_mapping: HashMap::new(),
            edge_types,
            edge_types_reversed,
            next_edge_id: 0
        }
    }

    fn maybe_add_node(&mut self, node: &Node) -> RawNodeRef {
        let uid = match self.node_mapping.entry(node.uid) {
            Entry::Occupied(e) => {
                *e.get()
            }
            Entry::Vacant(e) => {
                let uid = *e.insert(self.nodes.len());
                let new_node = Node {
                    uid,
                    property_index: node.property_index,
                    r#type: node.r#type,
                    connections: Vec::new()
                };
                self.nodes.push(new_node);
                uid
            }
        };
        RawNodeRef(uid)
    }

    fn add_edge(&mut self,
                from: RawNodeRef,
                to: RawNodeRef,
                r#type: usize,
                weight: f64) {
        let uid = self.next_edge_id;
        self.next_edge_id += 1;
        self.nodes[from.0].connections.push(Edge {
            uid, r#type, to, weight
        });
    }

    fn build_from(self, graph: &HeteroDiGraph) -> HeteroDiGraph {
        HeteroDiGraph {
            uid: next_graph_id(),
            node_metadata: graph.node_metadata.clone(),
            graph_metadata: Arc::new(
                GraphMetadata {
                    next_edge_id: self.next_edge_id,
                    is_markov: false    // Markov state is lost 
                }
            ),
            edge_metadata: Arc::new(
                EdgeMetadata {
                    edge_types: self.edge_types,
                    edge_types_reverse: self.edge_types_reversed,
                    edge_properties: vec![HashMap::new(); self.next_edge_id]
                }
            ),
            nodes: self.nodes,
        }
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Main Implementation
//////////////////////////////////////////////////////////////////////////////////////////////////

impl HeteroDiGraph {
    pub fn meta_path_subgraph(
        &self,
        meta_paths: Vec<(String, MetaPath<String>)>,
        unique_nodes: bool) -> HetNetResult<Self>
    {
        // Convert meta-paths to numerical types
        let meta_paths = meta_paths.into_iter()
            .map(
                |(name, mp)|
                    Ok((name, self.resolve_meta_path(mp)?))
            )
            .collect::<Result<Vec<(String, MetaPath<usize>)>, HetNetError>>()?;

        let edge_types = meta_paths.iter()
            .map(|(name, _)| name)
            .cloned()
            .collect();
        let mut builder = SubgraphBuilder::new(edge_types);

        for source in self.nodes.iter() {
            for (meta_path_index, (_, meta_path)) in meta_paths.iter().enumerate() {
                if !meta_path.start.matches(&source.r#type) {
                    continue;
                }
                for target in self.walk_meta_path(source, meta_path, unique_nodes) {
                    let from = builder.maybe_add_node(source);
                    let to = builder.maybe_add_node(target);
                    builder.add_edge(from, to, meta_path_index, 1.0);
                }
            }
        }

        Ok(builder.build_from(self))
    }

    fn walk_meta_path<'a>(&'a self,
                          start: &'a Node,
                          meta_path: &MetaPath<usize>,
                          unique_nodes: bool) -> Vec<&'a Node> {
        let mut stack = vec![
            (start, 0usize, BTreeSet::from_iter(vec![start.uid]))
        ];
        let mut result = Vec::new();

        while let Some((node, index, seen)) = stack.pop() {
            if index >= meta_path.steps.len() {
                result.push(node);
            } else {
                let (edge_type, node_type) = meta_path.steps[index];
                for edge in node.connections.iter() {
                    if !edge_type.matches(&edge.r#type) {
                        continue;
                    }
                    let new_node = &self.nodes[edge.to.0];
                    if !node_type.matches(&new_node.r#type) || seen.contains(&new_node.uid) {
                        continue;
                    }
                    let mut seen_for_edge = seen.clone();
                    if unique_nodes {
                        seen_for_edge.insert(new_node.uid);
                    }
                    stack.push((new_node, index + 1, seen_for_edge));
                }
            }
        }

        result
    }
}
