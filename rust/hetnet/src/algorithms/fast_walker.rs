//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Imports
//////////////////////////////////////////////////////////////////////////////////////////////////

use std::collections::{BTreeMap, BTreeSet};

use crate::graph::RawNodeRef;
use crate::{HetNetError, HetNetResult, NodeRef};
use crate::utils::rng::AliasSampler;
use crate::walkers::{GraphExplorer, Neighbours, Node2VecArgs};

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Cached Random Walker
//////////////////////////////////////////////////////////////////////////////////////////////////


pub struct CachedNode2VecWalker {
    graph_uid: usize,
    base_matrix: Vec<AliasSampler<(usize, usize)>>,
    transition_matrix: Vec<AliasSampler<(usize, usize)>>
}

impl CachedNode2VecWalker {
    pub fn estimate_size(g: &crate::graph::HeteroDiGraph) -> usize {
        Self::estimate_size_helper(g, 0)
    }

    pub fn estimate_size_as_undirected(g: &crate::graph::HeteroDiGraph) -> usize {
        Self::estimate_size_helper(g, 1)
    }

    #[inline]
    fn estimate_size_helper(g: &crate::graph::HeteroDiGraph, bi_multiplier: usize) -> usize {
        let incoming = Self::collect_incoming_nodes(g);

        let mut size = 0;
        size += size_of::<usize>();  // graph_uid
        for node in g.nodes.iter() {
            let n_out = node.connections.len();
            let n_in = incoming[node.uid].len();
            let mut sampler_size = 0;
            sampler_size += size_of::<f64>();
            sampler_size += size_of::<rand::distr::Uniform<f64>>();
            sampler_size += size_of::<Vec<(f64, (usize, usize))>>();
            sampler_size += size_of::<Vec<(usize, usize)>>();
            sampler_size += (n_out + bi_multiplier*n_in) * size_of::<(f64, (usize, usize))>();
            sampler_size += (n_out + bi_multiplier*n_in) * size_of::<(usize, usize)>();
            size += (n_in + 1 + bi_multiplier*n_out) * sampler_size;
        }

        size
    }

    pub fn new<G>(explorer: G, config: Node2VecArgs) -> HetNetResult<Self>
    where
        G: GraphExplorer<Config=Node2VecArgs>
    {
        // Collect incoming nodes
        let g = explorer.graph();
        let incoming = Self::collect_incoming_nodes(&g);

        // Prepare index table
        let mut global_index_map = vec![0; g.nodes.len()];
        let mut local_index_map = vec![BTreeMap::new(); g.nodes.len()];
        let mut offset = 0;
        for v in 0..g.nodes.len() {
            global_index_map[v] = offset;
            offset += incoming[v].len();
            for (i, u) in incoming[v].iter().copied().enumerate() {
                local_index_map[v].insert(u, i);
            }
        }

        // First order initial transitions
        let neighbours = g.neighbours();
        let mut base_matrix = Vec::with_capacity(g.nodes.len());
        for v in 0..g.nodes.len() {
            let dist = Self::build_dist(
                v, &config, &neighbours, None, &global_index_map, &local_index_map
            )?;
            base_matrix.push(dist);
        }

        // Second order transitions
        let mut transition_matrix = Vec::with_capacity(offset);
        for v in 0..g.nodes.len() {
            for u in incoming[v].iter().copied() {
                let dist = Self::build_dist(
                    v, &config, &neighbours, Some(RawNodeRef(u)), &global_index_map, &local_index_map
                )?;
                transition_matrix.push(dist);
            }
        }

        Ok(Self { graph_uid: g.uid, base_matrix, transition_matrix })
    }

    fn collect_incoming_nodes(g: &crate::graph::HeteroDiGraph) -> Vec<BTreeSet<usize>> {
        let mut incoming = vec![BTreeSet::new(); g.nodes.len()];
        for node in g.nodes.iter() {
            for edge in node.connections.iter() {
                incoming[edge.to.0].insert(node.uid);
            }
        }
        incoming
    }

    #[inline]
    fn build_dist(current_node: usize,
                  config: &Node2VecArgs,
                  neighbours: &Neighbours,
                  mut state: Option<RawNodeRef>,
                  global_index_map: &Vec<usize>,
                  local_index_map: &Vec<BTreeMap<usize, usize>>) -> HetNetResult<AliasSampler<(usize, usize)>>
    {
        let hist = neighbours.neighbours(
            RawNodeRef(current_node), &mut state, &config
        )?;
        let dist = hist.into_iter()
            .map(|(RawNodeRef(w), p)| {
                let offset = local_index_map[w].get(&current_node)
                    .expect("Failed to get local offset for incoming transition");
                let idx = global_index_map[w] + offset;
                ((w, idx), p)
            })
            .collect();
        Ok(AliasSampler::new(dist))
    }


    pub fn walk_from(&self, start: NodeRef, path_length: usize) -> HetNetResult<Vec<NodeRef>> {
        if self.graph_uid != start.graph_uid {
            return Err(HetNetError::InvalidReference);
        }

        let mut rng = rand::rng();

        let mut path = Vec::with_capacity(path_length);
        path.push(start);
        let mut current = start.downgrade().0;
        let mut node;

        // Perform the first fully random jump
        //#current = Self::jump(&self.base_matrix[current], &mut rng);
        (node, current) = self.base_matrix[current].sample(&mut rng);
        path.push(RawNodeRef(node).upgrade(self.graph_uid));

        // For the remainder of the walk, perform 2nd order jumps
        for _ in 0..path_length - 1 {
            (node, current) = self.transition_matrix[current].sample(&mut rng);
            path.push(RawNodeRef(node).upgrade(self.graph_uid));
        }

        Ok(path)
    }
}
