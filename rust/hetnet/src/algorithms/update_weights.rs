use std::collections::HashMap;
use crate::{EdgeRef, HetNetError, HetNetResult, HeteroDiGraph};
use crate::builder::next_graph_id;

impl HeteroDiGraph {
    pub fn update_weights(&self, weights: HashMap<EdgeRef, f64>) -> HetNetResult<Self> {
        let mapping = weights.into_iter()
            .map(|(k, v)| {
                if k.graph_uid != self.uid {
                    Err(HetNetError::InvalidReference)
                } else {
                    Ok((k.edge_uid, v))
                }
            })
            .collect::<Result<HashMap<_, _>, _>>()?;
        
        let new = Self {
            uid: next_graph_id(),
            node_metadata: self.node_metadata.clone(),
            edge_metadata: self.edge_metadata.clone(),
            graph_metadata: self.graph_metadata.clone(),
            nodes: self.nodes.iter()
                .map(|node| {
                    let mut new = node.clone();
                    for edge in new.connections.iter_mut() {
                        if let Some(w) = mapping.get(&edge.uid) {
                            edge.weight = *w;
                        }
                    }
                    new 
                })
                .collect()
        };
        Ok(new)
    }
}