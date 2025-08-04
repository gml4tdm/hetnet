use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

use crate::builder::next_graph_id;
use crate::graph::GraphMetadata;
use crate::HeteroDiGraph;

impl HeteroDiGraph {
    pub fn with_distance_matrix(&self) -> Self {
        let v = self.nodes.len() as u64;
        let e = self.graph_metadata.next_edge_id as u64;
        let matrix = if v.pow(3) < v * (v + e) {
            self.floyd_warshall_distance_matrix()
        } else {
            self.bfs_distance_matrix()
        };
        Self {
            node_metadata: self.node_metadata.clone(),
            edge_metadata: self.edge_metadata.clone(),
            graph_metadata: Arc::new(
                GraphMetadata {
                    weighted_distance_matrix: self.graph_metadata.weighted_distance_matrix.clone(),
                    next_edge_id: self.graph_metadata.next_edge_id,
                    is_markov: self.graph_metadata.is_markov,
                    distance_matrix: Some(
                        Arc::new(
                            matrix.into_iter().map(|x| x as usize).collect()
                        )
                    ),
                }
            ),
            uid: next_graph_id(),
            nodes: self.nodes.clone()
        }
    }

    fn init_distance_matrix(&self) -> Vec<i64> {
        let v = self.nodes.len();
        let mut matrix = vec![-1i64; v*v];
        for i in 0..v {
            matrix[i + v*i] = 0;
        }
        matrix
    }

    fn bfs_distance_matrix(&self) -> Vec<i64> {
        let v = self.nodes.len();
        let mut matrix = self.init_distance_matrix();

        for i in 0..v {
            let mut seen = HashSet::new();
            let mut queue = VecDeque::new();
            queue.push_back((&self.nodes[i], 0));
            while let Some((current, dist)) = queue.pop_front() {
                if seen.contains(&current.uid) {
                    continue;
                }
                matrix[i + v*current.uid] = dist;
                seen.insert(current.uid);
                for edge in current.connections.iter() {
                    queue.push_back((&self.nodes[edge.to.0], dist + 1));
                }
            }
        }
        
        matrix
    }

    fn floyd_warshall_distance_matrix(&self) -> Vec<i64> {
        let v = self.nodes.len();
        let mut matrix = self.init_distance_matrix();

        // Initialise matrix using matrix connections
        for (i, node) in self.nodes.iter().enumerate() {
            for edge in node.connections.iter() {
                matrix[i + v*edge.to.0] = 1;
            }
        }

        for i in 0..v {
            for j in 0..v {
                for k in 0..v {
                    let i_j_idx = i + v*j;
                    let i_k_idx = i + v*k;
                    let k_j_idx = k + v*j;
                    let d_i_j = matrix[i_j_idx];
                    let d_i_k = matrix[i_k_idx];
                    let d_k_j = matrix[k_j_idx];
                    if (d_i_k != -1 && d_k_j != -1) && (d_i_j == -1 || d_i_j < d_i_k + d_k_j) {
                        matrix[i_j_idx] = d_i_k + d_k_j;
                    }
                }
            }
        }
        matrix
    }
}