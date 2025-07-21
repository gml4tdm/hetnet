//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Imports
//////////////////////////////////////////////////////////////////////////////////////////////////

use std::collections::{BTreeSet, HashMap, HashSet};

use crate::errors::{GraphQueryingError, HetNetError};
use crate::meta_path::MetaPath;
use crate::shared_types::{Edge, EdgeDescriptor, EdgeRef, Node, NodeDescriptor, NodeRef};
use crate::walker;

//////////////////////////////////////////////////////////////////////////////////////////////////
// Types
//////////////////////////////////////////////////////////////////////////////////////////////////

pub struct HeteroDiGraph {
    metadata: GraphMetaData,
    nodes: Vec<Node>
}

struct GraphMetaData {
    // Node types
    node_types: Vec<String>,
    node_types_reverse: HashMap<String, usize>,

    // Edge types
    edge_types: Vec<String>,
    edge_types_reverse: HashMap<String, usize>,

    // Properties
    node_properties: Vec<HashMap<String, String>>,
    edge_properties: Vec<HashMap<String, String>>
}

pub(crate) struct Neighbours<'a> {
    graph: &'a HeteroDiGraph,
}

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Builder
//////////////////////////////////////////////////////////////////////////////////////////////////


#[derive(Debug, Default, Clone)]
pub struct HeteroDiGraphBuilder {
    nodes: Vec<Node>,
    node_types: HashMap<String, usize>,
    edge_types: HashMap<String, usize>,
    node_properties: Vec<HashMap<String, String>>,
    edge_properties: Vec<HashMap<String, String>>,
    next_edge_id: usize
}

impl HeteroDiGraphBuilder {
    pub fn new() -> Self {
        let mut s = Self::default();
        s.next_edge_id = 0;
        s
    }

    pub fn add_node(&mut self,
                    r#type: String,
                    properties: Option<HashMap<String, String>>) -> NodeRef {
        let uid = self.nodes.len();
        let next_type_id = self.node_types.len();
        let type_id = *self.node_types
            .entry(r#type.clone())
            .or_insert(next_type_id);
        self.nodes.push(Node {
            uid,
            r#type: type_id,
            connections: Vec::new(),
        });
        self.node_properties.push(properties.unwrap_or_default());
        NodeRef(uid)
    }

    pub fn add_edge(&mut self,
                    from: NodeRef,
                    to: NodeRef,
                    r#type: String,
                    weight: Option<f64>,
                    properties: Option<HashMap<String, String>>) {
        let next_type_id = self.edge_types.len();
        let type_id = *self.edge_types
            .entry(r#type.clone())
            .or_insert(next_type_id);
        let edge = Edge{
            r#type: type_id, 
            to: to.0, 
            weight: weight.unwrap_or(1.0),
            uid: self.next_edge_id
        };
        self.next_edge_id += 1; 
        self.nodes.get_mut(from.0)
            .expect("Invalid node reference")
            .connections
            .push(edge);
        self.edge_properties.push(properties.unwrap_or_default());
    }

    pub fn build(self) -> HeteroDiGraph {
        let edge_types = Self::convert_mapping(self.edge_types);
        let edge_types_reverse = edge_types.iter()
            .enumerate()
            .map(|(i, x)| (x.to_string(), i))
            .collect::<HashMap<_, _>>();

        let node_types = Self::convert_mapping(self.node_types);
        let node_types_reverse = node_types.iter()
            .enumerate()
            .map(|(i, x)| (x.to_string(), i))
            .collect::<HashMap<_, _>>();

        let metadata = GraphMetaData {
            node_types,
            node_types_reverse,
            edge_types,
            edge_types_reverse,
            node_properties: self.node_properties,
            edge_properties: self.edge_properties,
        };
        HeteroDiGraph {
            metadata,
            nodes: self.nodes,
        }
    }

    fn convert_mapping(m: HashMap<String, usize>) -> Vec<String> {
        let mut as_vec = m.into_iter().collect::<Vec<_>>();
        as_vec.sort_by_key(|x| x.1);
        as_vec.into_iter().map(|x| x.0).collect()
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Path Traversal
//////////////////////////////////////////////////////////////////////////////////////////////////


pub struct Node2VecArgs {
    pub(super) p: f64,
    pub(super) q: f64
}

impl Default for Node2VecArgs {
    fn default() -> Self {
        Self {p: 1.0, q: 1.0}
    }
}

impl<'a> walker::GraphExplorer for Neighbours<'a> {
    type State = Option<NodeRef>;
    type Config = Node2VecArgs;

    fn neighbours(
        &self,
        NodeRef(uid): NodeRef,
        state: &mut Self::State,
        config: &Self::Config
    ) -> Result<HashMap<NodeRef, f64>, HetNetError>
    {
        let node = self.graph.nodes.get(uid)
            .ok_or(GraphQueryingError::InvalidNodeId{uid})?;
        let stream = node.connections.iter()
            .map(|edge| (NodeRef(edge.to), edge.weight));
        let mut result = HashMap::new();
        match state {
            None => {
                for (item, w) in stream {
                    result.entry(item).and_modify(|x| *x += w).or_insert(w);
                }
            }
            Some(prev) => {
                let NodeRef(prev_id) = *prev;
                let reachable = self.graph.nodes.get(prev_id)
                    .expect("Invalid state")
                    .connections
                    .iter()
                    .map(|edge| edge.to)
                    .collect::<HashSet<_>>();
                for (NodeRef(item), w) in stream {
                    let p = if prev_id == item {
                        1.0/config.p * w
                    } else if reachable.contains(&item) {
                        w
                    } else {
                        1.0/config.q * w
                    };
                    result.entry(NodeRef(item)).and_modify(|x| *x += p).or_insert(p);
                }
            }
        }
        Ok(result)
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Implementation
//////////////////////////////////////////////////////////////////////////////////////////////////


impl HeteroDiGraph {

    pub fn node_list(&self) -> Vec<NodeDescriptor> {
        let mut result = Vec::with_capacity(self.nodes.len());
        for node in self.nodes.iter() {
            let type_name = self.metadata.node_types[node.r#type].clone();
            result.push(NodeDescriptor { 
                uid: NodeRef(node.uid),
                r#type: type_name,
            });
        }
        result
    }

    pub fn edge_list(&self) -> Vec<EdgeDescriptor> {
        let mut result = Vec::new();
        for node in self.nodes.iter() {
            for edge in node.connections.iter() {
                let type_name = self.metadata.edge_types[edge.r#type].clone();
                result.push(EdgeDescriptor {
                    uid: EdgeRef(edge.uid),
                    from: NodeRef(node.uid),
                    to: NodeRef(edge.to),
                    r#type: type_name,
                    weight: edge.weight
                })
            }
        }
        result 
    }

    pub fn node_properties(&self, NodeRef(uid): NodeRef) -> Result<&HashMap<String, String>, GraphQueryingError> {
        self.metadata.node_properties.get(uid)
            .ok_or(GraphQueryingError::InvalidNodeId{uid})
    }

    pub fn edge_properties(&self, EdgeRef(uid): EdgeRef) -> Result<&HashMap<String, String>, GraphQueryingError> {
        self.metadata.edge_properties.get(uid)
            .ok_or(GraphQueryingError::InvalidEdgeId{uid})
    }

    pub fn deduplicate_edges(&self, types: Vec<String>) -> Result<Self, HetNetError> {
        // Build the new graph
        let mut nodes = HashMap::new();
        let mut builder = HeteroDiGraphBuilder::new();
        for node in self.nodes.iter() {
            let uid = builder.add_node(
                self.metadata.node_types[node.r#type].clone(),
                self.metadata.node_properties.get(node.uid).cloned()
            );
            nodes.insert(node.uid, uid);
        }
        let dedup = types.into_iter()
            .map(|tp|
                self.metadata.edge_types_reverse.get(&tp).copied()
                    .ok_or_else(|| HetNetError::UnknownType {kind: "edge".to_string(), name: tp})
            )
            .collect::<Result<HashSet<_>, _>>()?;
        let mut seen_edges = HashSet::new();
        for node in self.nodes.iter() {
            for edge in node.connections.iter() {
                let key = (node.uid, edge.to, edge.r#type);
                if dedup.contains(&edge.r#type) {
                    if seen_edges.contains(&key) {
                        continue;
                    }
                    seen_edges.insert(key);
                }
                builder.add_edge(
                    nodes.get(&node.uid).copied().expect("Missing Node"),
                    nodes.get(&edge.to).copied().expect("Missing Node"),
                    self.metadata.edge_types[edge.r#type].clone(),
                    None,
                    self.metadata.edge_properties.get(edge.uid).cloned()
                );
            }
        }
        Ok(builder.build())
    }

    pub(crate) fn neighbours(&self) -> Neighbours {
        Neighbours { graph: self }
    }

    pub fn meta_path_subgraph(
        &self,
        meta_paths: Vec<(String, MetaPath<String>)>,
        unique_nodes: bool) -> Result<Self, HetNetError>
    {
        // Convert meta-paths to numerical types
        let meta_paths = meta_paths.into_iter()
            .map(
                |(name, mp)|
                    Ok((name, self.resolve_meta_path(mp)?))
            )
            .collect::<Result<Vec<(String, MetaPath<usize>)>, HetNetError>>()?;

        // Build new graph
        let mut builder = HeteroDiGraphBuilder::new();
        let mut nodes = HashMap::new();
        for node in self.nodes.iter() {
            for (mp_name, meta_path) in meta_paths.iter() {
                if !meta_path.start.matches(&node.r#type) {
                    continue;
                }
                for target in self.walk_meta_path(node, meta_path, unique_nodes) {
                    let src_uid = *nodes.entry(node.uid)
                        .or_insert_with(|| self.copy_node_to_builder(node, &mut builder));
                    let tgt_uid = *nodes.entry(target.uid)
                        .or_insert_with(|| self.copy_node_to_builder(target, &mut builder));
                    builder.add_edge(src_uid, tgt_uid, mp_name.clone(), None, None);
                }
            }
        }

        Ok(builder.build())
    }

    fn resolve_meta_path(&self, mp: MetaPath<String>) -> Result<MetaPath<usize>, HetNetError> {
        let resolved = mp.resolve_types(
            &self.metadata.node_types_reverse,
            &self.metadata.edge_types_reverse
        )?;
        Ok(resolved)
    }

    fn copy_node_to_builder(&self, node: &Node, builder: &mut HeteroDiGraphBuilder) -> NodeRef {
        let node_type = self.metadata.node_types[node.r#type].clone();
        let node_data = self.metadata.node_properties
            .get(node.uid)
            .cloned();
        builder.add_node(node_type, node_data)
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
                    let new_node = &self.nodes[edge.to];
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
