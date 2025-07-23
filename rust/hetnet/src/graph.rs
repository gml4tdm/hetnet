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
    pub(crate) uid: usize,
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
    pub(crate) node_types_reverse: HashMap<String, usize>,
    pub(crate) node_properties: Vec<HashMap<String, String>>,
}

#[derive(Debug)]
pub(crate) struct EdgeMetadata {
    pub(crate) edge_types: Vec<String>,
    pub(crate) edge_types_reverse: HashMap<String, usize>,
    pub(crate) edge_properties: Vec<HashMap<String, String>>
}

#[derive(Debug, Clone)]
pub(crate) struct Node {
    pub(crate) uid: usize,
    pub(crate) property_index: usize,
    pub(crate) r#type: usize,
    pub(crate) connections: Vec<Edge>
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct Edge {
    pub(crate) uid: usize,
    pub(crate) r#type: usize,
    pub(crate) to: RawNodeRef,
    pub(crate) weight: f64
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct NodeRef {
    pub(crate) graph_uid: usize,
    pub(crate) node_uid: usize
}

impl NodeRef {
    pub(crate) fn downgrade(&self) -> RawNodeRef {
        RawNodeRef(self.node_uid)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct EdgeRef {
    pub(crate) graph_uid: usize,
    pub(crate) edge_uid: usize
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct RawNodeRef(pub(crate) usize);

impl RawNodeRef {
    pub(crate) fn upgrade(&self, graph_uid: usize) -> NodeRef {
        NodeRef { graph_uid, node_uid: self.0 }
    }
}


//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Basic Interface
//////////////////////////////////////////////////////////////////////////////////////////////////

impl HeteroDiGraph {
    pub fn node_list(&self) -> Vec<NodeDescriptor> {
        let mut result = Vec::with_capacity(self.nodes.len());
        let metadata = &*self.node_metadata;
        for node in self.nodes.iter() {
            let type_name = metadata.node_types[node.r#type].clone();
            result.push(NodeDescriptor {
                uid: NodeRef { graph_uid: self.uid, node_uid: node.uid },
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
                let type_name = metadata.edge_types[edge.r#type].clone();
                result.push(EdgeDescriptor {
                    uid: EdgeRef { graph_uid: self.uid, edge_uid: edge.uid },
                    from: NodeRef { graph_uid: self.uid, node_uid: node.uid },
                    to: edge.to.upgrade(self.uid),
                    r#type: type_name,
                    weight: edge.weight
                })
            }
        }
        result
    }

    pub fn node_properties(&self, reference: NodeRef) -> Result<&HashMap<String, String>, GraphQueryingError> {
        if reference.graph_uid != self.uid {
            return Err(GraphQueryingError::InvalidReference);
        }
        let index = self.nodes.get(reference.node_uid)
            .ok_or(GraphQueryingError::InvalidNodeId{uid: reference.node_uid})?
            .property_index;
        self.node_metadata.node_properties.get(index)
            .ok_or_else(|| panic!("No metadata for index {index}"))
    }

    pub fn edge_properties(&self, reference: EdgeRef) -> Result<&HashMap<String, String>, GraphQueryingError> {
        if reference.graph_uid != self.uid {
            return Err(GraphQueryingError::InvalidReference);
        }
        self.edge_metadata.edge_properties.get(reference.edge_uid)
            .ok_or(GraphQueryingError::InvalidEdgeId{uid: reference.edge_uid})
    }
}