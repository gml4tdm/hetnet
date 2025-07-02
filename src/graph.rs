//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Imports
//////////////////////////////////////////////////////////////////////////////////////////////////

use std::collections::{BTreeSet, HashMap};

use crate::errors::{GraphQueryingError, HetNetError};
use crate::meta_path::MetaPath;
use crate::shared_types::{Edge, Node, NodeRef};

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Types
//////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct HeteroDiGraph {
    metadata: GraphMetaData,
    nodes: Vec<Node>
}

#[derive(Debug)]
struct GraphMetaData {
    node_types: Vec<String>,
    edge_types: Vec<String>,
    edge_types_reverse: HashMap<String, usize>,
    node_properties: Vec<HashMap<String, String>>,
    edge_properties: HashMap<(usize, Edge), HashMap<String, String>>
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
    edge_properties: HashMap<(usize, Edge), HashMap<String, String>>
}

impl HeteroDiGraphBuilder {
    pub fn new() -> Self {
        Self::default()
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
                    properties: Option<HashMap<String, String>>) {
        let next_type_id = self.edge_types.len();
        let type_id = *self.edge_types
            .entry(r#type.clone())
            .or_insert(next_type_id);
        let edge = Edge{r#type: type_id, to: to.0};
        self.nodes.get_mut(from.0)
            .expect("Invalid node reference")
            .connections
            .push(edge);
        let key = (from.0, edge);
        self.edge_properties.insert(key, properties.unwrap_or_default());
    }
    
    pub fn build(self) -> HeteroDiGraph {
        let edge_types = Self::convert_mapping(self.edge_types);
        let edge_types_reverse = edge_types.iter()
            .enumerate()
            .map(|(i, x)| (x.to_string(), i))
            .collect::<HashMap<_, _>>();
        let metadata = GraphMetaData {
            node_types: Self::convert_mapping(self.node_types),
            edge_types,
            edge_types_reverse,
            node_properties: self.node_properties,
            edge_properties: self.edge_properties
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
// Implementation
//////////////////////////////////////////////////////////////////////////////////////////////////


impl HeteroDiGraph {
    pub fn debug(&self) -> String {
        format!("{self:?}")
    }
    
    pub fn node_list(&self) -> Vec<(NodeRef, String)> {
        let mut result = Vec::with_capacity(self.nodes.len());
        for node in self.nodes.iter() {
            let type_name = self.metadata.node_types[node.r#type].clone();
            result.push((NodeRef(node.uid), type_name));
        }
        result
    }

    pub fn edge_list(&self) -> Vec<(NodeRef, NodeRef, String, usize)> {
        let mut counts = HashMap::new();
        for node in self.nodes.iter() {
            for edge in node.connections.iter() {
                let type_name = self.metadata.edge_types[edge.r#type].clone();
                let key = (node.uid, edge.to, type_name);
                counts.entry(key).
                    and_modify(|x| *x += 1).
                    or_insert(1);
            }
        }
        counts.into_iter()
            .map(
                |((fr, to, kind), count)| 
                    (NodeRef(fr), NodeRef(to), kind, count)
            )
            .collect()
    }
    
    pub fn node_properties(&self, NodeRef(uid): NodeRef) -> Result<&HashMap<String, String>, GraphQueryingError> {
        self.metadata.node_properties.get(uid)
            .ok_or(GraphQueryingError::InvalidNodeId{uid})
    }
    
    pub fn edge_properties(&self, 
                           NodeRef(from): NodeRef,
                           NodeRef(to): NodeRef, 
                           r#type: String) -> Result<&HashMap<String, String>, GraphQueryingError> {
        if from >= self.nodes.len() {
            return Err(GraphQueryingError::InvalidNodeId{uid: from});
        }
        if to >= self.nodes.len() {
            return Err(GraphQueryingError::InvalidNodeId{uid: to});
        }
        let type_id = self.metadata.edge_types_reverse.get(&r#type)
            .copied()
            .ok_or_else(|| GraphQueryingError::UnknownType {kind: "edge".to_string(), name: r#type.clone()})?;
        
        let key = (from, Edge { to, r#type: type_id });
        self.metadata.edge_properties.get(&key)
            .ok_or(GraphQueryingError::NoSuchEdge {kind: r#type, src: from, tgt: to})
    }

    pub fn meta_path_subgraph(
        &self,
        meta_paths: Vec<(String, MetaPath<String>)>,
        unique_nodes: bool) -> Result<Self, HetNetError>
    {
        // Convert meta paths to numerical types
        let node_types = self.metadata.node_types.iter()
            .enumerate()
            .map(|(i, x)| (x.to_string(), i))
            .collect::<HashMap<_, _>>();
        let meta_paths = meta_paths.into_iter()
            .map(
                |(name, mp)| 
                    Ok((name, mp.resolve_types(&node_types, &self.metadata.edge_types_reverse)?))
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
                    builder.add_edge(src_uid, tgt_uid, mp_name.clone(), None);
                }
            }
        }

        Ok(builder.build())
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

        while let Some((node, index, mut seen)) = stack.pop() {
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
                    if unique_nodes {
                        seen.insert(new_node.uid);
                    }
                    stack.push((new_node, index + 1, seen.clone()));
                }
            }
        }

        result
    }
}
