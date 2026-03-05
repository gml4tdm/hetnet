//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Imports
//////////////////////////////////////////////////////////////////////////////////////////////////

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::{HetNetError, HetNetResult, HeteroDiGraph};
use crate::builder::next_graph_id;
use crate::graph::{Edge, EdgeMetadata, Node};

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Types
//////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DataHandling {
    Discard,
    EnforceIdentical
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WeightHandling {
    SetToOne,
    EnforceIdentical,
    SumAggregate
}

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Implementation
//////////////////////////////////////////////////////////////////////////////////////////////////

impl HeteroDiGraph {
    pub fn deduplicate_edges(&self,
                             types: Vec<String>,
                             data_handling: DataHandling,
                             weight_handling: WeightHandling,
                             allow_unknown_types: bool) -> HetNetResult<Self>
    {
        // Convert edge types
        let dedup = self.convert_edge_types(types, allow_unknown_types)?.into_iter()
            .collect::<HashSet<_>>();

        // Discard edges as needed
        let mut next_edge_id = self.graph_metadata.next_edge_id;
        let metadata = &*self.edge_metadata;
        let nodes = self.nodes.iter()
            .map(|node| {
                let node = Node {
                    uid: node.uid,
                    property_index: node.property_index,
                    r#type: node.r#type,
                    connections: deduplicate_edges(
                        &node.connections,
                        &dedup,
                        data_handling,
                        weight_handling,
                        &mut next_edge_id,
                        metadata
                    )?
                };
                Ok(node)
            })
            .collect::<Result<Vec<_>, HetNetError>>()?;

        // Build new graph
        let graph = HeteroDiGraph {
            uid: next_graph_id(),
            node_metadata: self.node_metadata.clone(),
            graph_metadata: self.graph_metadata.clone(),
            edge_metadata: Arc::new(
                EdgeMetadata {
                    edge_types: self.edge_metadata.edge_types.clone(),
                    edge_types_reverse: self.edge_metadata.edge_types_reverse.clone(),
                    edge_properties: Vec::new()
                }
            ),
            nodes
        };
        Ok(graph)
    }
}

fn deduplicate_edges(edges: &[Edge],
                     dedup: &HashSet<usize>,
                     data_handling: DataHandling,
                     weight_handling: WeightHandling,
                     next_edge_id: &mut usize,
                     metadata: &EdgeMetadata) -> HetNetResult<Vec<Edge>>
{
    let mut result = Vec::new();
    let mut seen = HashMap::new();
    for edge in edges.iter().copied() {
        if !dedup.contains(&edge.r#type) {
            result.push(edge);
            continue;
        }
        let key = (edge.r#type, edge.to);
        match seen.entry(key) {
            Entry::Vacant(e) => {
                let edge_id = if data_handling == DataHandling::Discard {
                    let uid = *next_edge_id;
                    *next_edge_id += 1;
                    uid
                } else {
                    edge.uid
                };
                let w = if weight_handling == WeightHandling::SetToOne { 1.0 } else { edge.weight };
                e.insert((edge.uid, edge.weight, result.len()));
                result.push(Edge { uid: edge_id, r#type: edge.r#type, weight: w, to: edge.to});
            }
            Entry::Occupied(e) => {
                let (uid, weight, index) = e.get();
                if data_handling == DataHandling::EnforceIdentical {
                    if metadata.edge_properties.get(*uid) != metadata.edge_properties.get(edge.uid) {
                        return Err(HetNetError::NotAllEdgePropertiesEqual);
                    }
                }
                if weight_handling == WeightHandling::EnforceIdentical {
                    if (weight - edge.weight).abs() > 1e-5 {
                        return Err(HetNetError::NotAllEdgePropertiesEqual);
                    }
                } else if weight_handling == WeightHandling::SumAggregate {
                    result[*index].weight += edge.weight;
                }
            }
        }
    }
    Ok(result)
}
