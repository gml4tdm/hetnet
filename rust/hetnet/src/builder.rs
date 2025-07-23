//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Imports
//////////////////////////////////////////////////////////////////////////////////////////////////

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::graph::{
    HeteroDiGraph,
    Node, NodeRef,
    Edge, EdgeRef,
    NodeMetadata, EdgeMetadata, GraphMetadata,
    RawNodeRef
};
use crate::{HetNetError, HetNetResult};

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Auxiliary functions
//////////////////////////////////////////////////////////////////////////////////////////////////

static NEXT_GRAPH_UID: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn next_graph_id() -> usize {
    NEXT_GRAPH_UID.fetch_add(1, Ordering::Relaxed)
}


//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Types
//////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Clone)]
pub struct HeteroDiGraphBuilder {
    graph_uid: usize,
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
        Self {
            graph_uid: next_graph_id(),
            next_edge_id: 0,
            ..Default::default()
        }
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
            property_index: uid,    // Equal to UID, unless node trimming is performed
            r#type: type_id,
            connections: Vec::new(),
        });
        self.node_properties.push(properties.unwrap_or_default());
        NodeRef { graph_uid: self.graph_uid, node_uid: uid }
    }

    pub fn add_edge(&mut self,
                    from: NodeRef,
                    to: NodeRef,
                    r#type: String,
                    weight: Option<f64>,
                    properties: Option<HashMap<String, String>>) -> HetNetResult<EdgeRef> {
        if from.graph_uid != self.graph_uid || to.graph_uid != self.graph_uid {
            return Err(HetNetError::InvalidReference);
        }
        let next_type_id = self.edge_types.len();
        let type_id = *self.edge_types
            .entry(r#type.clone())
            .or_insert(next_type_id);
        let edge = Edge {
            r#type: type_id,
            to: RawNodeRef(to.node_uid),
            weight: weight.unwrap_or(1.0),
            uid: self.next_edge_id
        };
        self.next_edge_id += 1;
        self.nodes.get_mut(from.node_uid)
            .expect("Invalid node reference")
            .connections
            .push(edge);
        self.edge_properties.push(properties.unwrap_or_default());
        Ok(EdgeRef { graph_uid: self.graph_uid, edge_uid: edge.uid })
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
            uid: self.graph_uid,
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
