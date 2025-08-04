use std::collections::BinaryHeap;
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
    }

    fn bfs_distance_matrix(&self) -> Vec<i64> {

    }

    fn floyd_warshall_distance_matrix(&self) -> Vec<i64> {
        let v = self.nodes.len();
        let mut matrix = vec![-1i64; v*v];

        // Self-connections are 0
        for i in 0..v {
            matrix[i + v*i] = 0;
        }

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