//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Imports
//////////////////////////////////////////////////////////////////////////////////////////////////

use std::collections::HashMap;
use std::sync::Arc;
use crate::graph::{
    HeteroDiGraph,
    Node, NodeRef, NodeTypeRef,
    Edge, EdgeRef, EdgeTypeRef,
    NodeMetadata,
    EdgeMetadata,
    GraphMetadata
};

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Types
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

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Implementation
//////////////////////////////////////////////////////////////////////////////////////////////////

impl HeteroDiGraphBuilder {
    pub fn new() -> Self {
        Self { next_edge_id: 0, ..Default::default() }
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
            r#type: NodeTypeRef(type_id),
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
                    properties: Option<HashMap<String, String>>) -> EdgeRef {
        let next_type_id = self.edge_types.len();
        let type_id = *self.edge_types
            .entry(r#type.clone())
            .or_insert(next_type_id);
        let edge = Edge {
            r#type: EdgeTypeRef(type_id),
            to: NodeRef(to.0),
            weight: weight.unwrap_or(1.0),
            uid: self.next_edge_id
        };
        self.next_edge_id += 1;
        self.nodes.get_mut(from.0)
            .expect("Invalid node reference")
            .connections
            .push(edge);
        self.edge_properties.push(properties.unwrap_or_default());
        EdgeRef(edge.uid)
    }

    pub fn build(self) -> HeteroDiGraph {
        let edge_types = Self::convert_mapping(self.edge_types);
        let edge_types_reverse = edge_types.iter()
            .enumerate()
            .map(|(i, x)| (x.to_string(), EdgeTypeRef(i)))
            .collect::<HashMap<_, _>>();

        let node_types = Self::convert_mapping(self.node_types);
        let node_types_reverse = node_types.iter()
            .enumerate()
            .map(|(i, x)| (x.to_string(), NodeTypeRef(i)))
            .collect::<HashMap<_, _>>();

        let node_metadata = NodeMetadata {
            node_types,
            node_types_reverse,
            node_properties: self.node_properties,
        };
        let edge_metadata = EdgeMetadata {
            edge_types,
            edge_types_reverse,
            edge_properties: self.edge_properties,
        };
        let graph_metadata = GraphMetadata {
            next_edge_id: self.next_edge_id
        };

        HeteroDiGraph {
            node_metadata: Arc::new(node_metadata),
            edge_metadata: Arc::new(edge_metadata),
            graph_metadata: Arc::new(graph_metadata),
            nodes: self.nodes,
        }
    }

    fn convert_mapping(m: HashMap<String, usize>) -> Vec<String> {
        let mut as_vec = m.into_iter().collect::<Vec<_>>();
        as_vec.sort_by_key(|x| x.1);
        as_vec.into_iter().map(|x| x.0).collect()
    }
}