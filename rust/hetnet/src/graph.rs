//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Imports
//////////////////////////////////////////////////////////////////////////////////////////////////

use std::collections::HashMap;
use std::sync::Arc;

use crate::{NodeDescriptor, EdgeDescriptor};
use crate::errors::GraphQueryingError;

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Types
//////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct HeteroDiGraph {
    pub(crate) node_metadata: Arc<NodeMetadata>,
    pub(crate) edge_metadata: Arc<EdgeMetadata>,
    pub(crate) graph_metadata: Arc<GraphMetadata>,
    pub(crate) nodes: Vec<Node>
}

#[derive(Debug)]
pub(crate) struct GraphMetadata {
    pub(crate) next_edge_id: usize
}

#[derive(Debug)]
pub(crate) struct NodeMetadata {
    pub(crate) node_types: Vec<String>,
    pub(crate) node_types_reverse: HashMap<String, NodeTypeRef>,
    pub(crate) node_properties: Vec<HashMap<String, String>>,
}

#[derive(Debug)]
pub(crate) struct EdgeMetadata {
    pub(crate) edge_types: Vec<String>,
    pub(crate) edge_types_reverse: HashMap<String, EdgeTypeRef>,
    pub(crate) edge_properties: Vec<HashMap<String, String>>
}

#[derive(Debug, Clone)]
pub(crate) struct Node {
    pub(crate) uid: usize,
    pub(crate) r#type: NodeTypeRef,
    pub(crate) connections: Vec<Edge>
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct Edge {
    pub(crate) uid: usize,
    pub(crate) r#type: EdgeTypeRef,
    pub(crate) to: NodeRef,
    pub(crate) weight: f64
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct NodeRef(pub(crate) usize);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct EdgeRef(pub(crate) usize);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub(crate) struct NodeTypeRef(pub(crate) usize);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub(crate) struct EdgeTypeRef(pub(crate) usize);

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Basic Interface
//////////////////////////////////////////////////////////////////////////////////////////////////

impl HeteroDiGraph {
    pub fn node_list(&self) -> Vec<NodeDescriptor> {
        let mut result = Vec::with_capacity(self.nodes.len());
        let metadata = &*self.node_metadata;
        for node in self.nodes.iter() {
            let type_name = metadata.node_types[node.r#type.0].clone();
            result.push(NodeDescriptor {
                uid: NodeRef(node.uid),
                r#type: type_name,
            });
        }
        result
    }

    pub fn edge_list(&self) -> Vec<EdgeDescriptor> {
        let mut result = Vec::new();
        let metadata = &*self.edge_metadata;
        for node in self.nodes.iter() {
            for edge in node.connections.iter() {
                let type_name = metadata.edge_types[edge.r#type.0].clone();
                result.push(EdgeDescriptor {
                    uid: EdgeRef(edge.uid),
                    from: NodeRef(node.uid),
                    to: edge.to,
                    r#type: type_name,
                    weight: edge.weight
                })
            }
        }
        result
    }

    pub fn node_properties(&self, NodeRef(uid): NodeRef) -> Result<&HashMap<String, String>, GraphQueryingError> {
        self.node_metadata.node_properties.get(uid)
            .ok_or(GraphQueryingError::InvalidNodeId{uid})
    }

    pub fn edge_properties(&self, EdgeRef(uid): EdgeRef) -> Result<&HashMap<String, String>, GraphQueryingError> {
        self.edge_metadata.edge_properties.get(uid)
            .ok_or(GraphQueryingError::InvalidEdgeId{uid})
    }
}