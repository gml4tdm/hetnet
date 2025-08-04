use std::sync::Arc;
use crate::builder::next_graph_id;
use crate::graph::GraphMetadata;
use crate::HeteroDiGraph;

impl HeteroDiGraph {
    pub fn to_markov_graph(&self) -> Self {
        Self {
            uid: next_graph_id(),
            graph_metadata: Arc::new(GraphMetadata {
                next_edge_id: self.graph_metadata.next_edge_id,
                is_markov: true,
                distance_matrix: self.graph_metadata.distance_matrix.clone(),
                weighted_distance_matrix: None
            }),
            node_metadata: self.node_metadata.clone(),
            edge_metadata: self.edge_metadata.clone(),
            nodes: self.nodes.iter()
                .map(|node| {
                    let mut new = node.clone();
                    let w_total = new.connections.iter()
                        .map(|edge| edge.weight)
                        .sum::<f64>();
                    new.connections.iter_mut().for_each(|e| e.weight /= w_total);
                    new
                })
                .collect()
        }
    }
}