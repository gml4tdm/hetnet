//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Imports
//////////////////////////////////////////////////////////////////////////////////////////////////

use std::collections::{HashMap, HashSet};

use rand::Rng;

use crate::graph::RawNodeRef;
use crate::{HetNetError, HetNetResult, HeteroDiGraph, NodeRef};
use crate::walkers::{GraphExplorer, Node2VecArgs};

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Cached Random Walker
//////////////////////////////////////////////////////////////////////////////////////////////////

type EdgeTable = (Vec<f64>, Vec<usize>);

pub struct CachedNode2VecWalker {
    graph_uid: usize,
    base_matrix: Vec<EdgeTable>,
    transition_matrix: Vec<HashMap<usize, EdgeTable>>
}

impl CachedNode2VecWalker {
    /// Estimate the memory required to hold the pre-computed
    /// transition matrices.
    /// Does not attempt to account for over-allocation/load factors.
    ///
    /// Returns result in bytes
    pub fn estimate_required_memory(graph: &HeteroDiGraph) -> usize {
        let outer_vec_size = size_of::<Vec<HashMap<usize, EdgeTable>>>();
        let map_size = size_of::<HashMap<usize, EdgeTable>>();
        let inner_f64_vec_size = size_of::<Vec<f64>>();
        let inner_usize_vec_size = size_of::<Vec<usize>>();
        let f64_size = size_of::<f64>();
        let usize_size = size_of::<usize>();

        let mut total = 0;

        let mut incoming = vec![HashSet::new(); graph.nodes.len()];
        for node in graph.nodes.iter() {
            for edge in node.connections.iter() {
                incoming[edge.to.0].insert(node.uid);
            }
        }

        for node in graph.nodes.iter() {
            // Compute size for a single edge table
            let e = node.connections.len();
            let table_size =
                inner_f64_vec_size + inner_usize_vec_size + e*f64_size + e*usize_size;

            // Account for base matrix
            total += table_size;

            // Account for transition matrix
            let i = incoming[node.uid].len();
            total +=
                map_size + usize_size + i*table_size
        }

        // Account for outer vec
        total += outer_vec_size;

        total
    }

    pub fn new<G>(explorer: G, config: Node2VecArgs) -> HetNetResult<Self>
    where
        G: GraphExplorer<Config=Node2VecArgs>
    {
        let g = explorer.graph();
        let mut incoming = vec![HashSet::new(); g.nodes.len()];
        for node in g.nodes.iter() {
            for edge in node.connections.iter() {
                incoming[edge.to.0].insert(node.uid);
            }
        }
        let neighbours = g.neighbours();

        // First order initial transitions
        let mut base_matrix = Vec::with_capacity(g.nodes.len());
        for v in 0..g.nodes.len() {
            let mut state = Option::<RawNodeRef>::None;
            let hist = neighbours.neighbours(
                RawNodeRef(v), &mut state, &config
            )?;
            base_matrix.push(Self::hist_to_edge_table(hist));
        }

        // Second order transitions
        let mut transition_matrix = vec![HashMap::new(); g.nodes.len()];
        for v in 0..g.nodes.len() {
            for u in incoming[v].iter().copied() {
                let mut state = Some(RawNodeRef(u));
                let hist = neighbours.neighbours(
                    RawNodeRef(v), &mut state, &config
                )?;
                transition_matrix[v].insert(u, Self::hist_to_edge_table(hist));
            }
        }


        Ok(Self { graph_uid: g.uid, base_matrix, transition_matrix })
    }

    fn hist_to_edge_table(hist: HashMap<RawNodeRef, f64>) -> EdgeTable {
        let mut destinations = Vec::with_capacity(hist.len());
        let mut cum_sums = Vec::with_capacity(hist.len());
        let mut total = 0.0;
        for (to, prob) in hist {
            destinations.push(to.0);
            total += prob;
            cum_sums.push(total);
        }
        let probabilities = cum_sums.into_iter()
            .map(|p| p / total)
            .collect();
        (probabilities, destinations)
    }

    pub fn walk_from(&self, start: NodeRef, path_length: usize) -> HetNetResult<Vec<NodeRef>> {
        if self.graph_uid != start.graph_uid {
            return Err(HetNetError::InvalidReference);
        }

        let mut rng = rand::rng();

        let mut path = Vec::with_capacity(path_length);
        path.push(start);
        let mut current = start.downgrade().0;

        // Perform the first fully random jump
        let mut prev = current;
        current = Self::jump(&self.base_matrix[current], &mut rng);
        path.push(RawNodeRef(current).upgrade(self.graph_uid));

        // For the remainder of the walk, perform 2nd order jumps
        for _ in 0..path_length - 1 {
            let new = Self::jump(
                self.transition_matrix[current].get(&prev)
                    .expect("Failed to find transitions for 2nd order jump"),
                &mut rng
            );
            prev = current;
            current = new;
            path.push(RawNodeRef(current).upgrade(self.graph_uid));
        }

        Ok(path)
    }

    #[inline]
    fn jump(table: &EdgeTable, rng: &mut impl Rng) -> usize {
        let selected = rng.random();
        let index = table.0.partition_point(|&x| x <= selected);
        //println!("{selected} --- {index} --- {:?}", table.0);
        table.1[index]
    }
}
