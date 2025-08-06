use std::collections::{HashMap, VecDeque};
use std::collections::hash_map::Entry;

use crate::HeteroDiGraph;


impl HeteroDiGraph {
    pub(super) fn sparse_distance_matrix(&self, max_hops: usize) -> Vec<HashMap<usize, usize>> {
        let mut result = Vec::new();
        for start in self.nodes.iter() {
            let mut distances = HashMap::new();
            let mut queue = VecDeque::new();
            queue.push_back((start, 0));
            while let Some((node, dist)) = queue.pop_front() {
                match distances.entry(node.uid) {
                    Entry::Occupied(_) => { continue; }
                    Entry::Vacant(e) => {
                        e.insert(dist);
                        if dist < max_hops {
                            for edge in node.connections.iter() {
                                queue.push_back((&self.nodes[edge.to.0], dist + 1));
                            }
                        }
                    }
                }
            }
            result.push(distances);
        }
        result
    }
}
