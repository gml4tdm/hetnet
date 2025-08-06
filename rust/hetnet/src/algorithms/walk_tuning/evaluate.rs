use std::collections::{HashMap, HashSet};

use crate::{HetNetResult, HeteroDiGraph, NodeRef};
use crate::graph::RawNodeRef;
use crate::walkers::{
    RandomWalkConfig,
    GraphExplorer,
    NeighbourSelector,
    RandomWalker,
};


pub struct EvalArgs<G, N>
where
    G: GraphExplorer,
    N: NeighbourSelector,
{
    // Random walk config
    explorer: G,
    selector: N,
    config: RandomWalkConfig<G>,

    // Evaluation Config
    n_rounds: u32
}

impl<G: GraphExplorer, N: NeighbourSelector> EvalArgs<G, N> {
    pub fn new(explorer: G, selector: N, config: RandomWalkConfig<G>) -> Self {
        Self { explorer, selector, config, n_rounds: 1 }
    }

    pub fn with_n_rounds(mut self, n_rounds: u32) -> Self {
        self.n_rounds = n_rounds;
        self
    }
}


pub struct EvalResult {
    exploration_density: Vec<f64>,
    cumulative_exploration_density: Vec<f64>,
    max_dist: f64,
    max_dist_exploration_density: f64
}

impl EvalResult {
    pub fn exploration_density_at_distance(&self) -> &Vec<f64> {
        &self.exploration_density
    }

    pub fn cumulative_exploration_density_at_distance(&self) -> &Vec<f64> {
        &self.cumulative_exploration_density
    }

    pub fn max_dist(&self) -> f64 {
        self.max_dist
    }

    pub fn exploration_density_at_max_dist(&self) -> f64 {
        self.max_dist_exploration_density
    }
}


pub fn evaluate_random_walk_config<G, N>(
    graph: &HeteroDiGraph,
    on_nodes: HashSet<NodeRef>,
    config: EvalArgs<G, N>) -> HetNetResult<EvalResult>
where
    G: GraphExplorer,
    N: NeighbourSelector
{
    let mut summed_exploration_density = vec![0.0; config.config.path_length + 1];
    let mut summed_cumulative_exploration_density = vec![0.0; config.config.path_length + 1];
    let mut summed_max_dist = 0.0;
    let mut summed_max_dist_exploration_density = 0.0;
    let mut total_runs_by_dist = vec![0.0; config.config.path_length + 1];
    let mut total_runs = 0.0;
    let mut max_dist = 0usize;

    let distance_matrix = graph.sparse_distance_matrix(
        config.config.path_length
    );
    let mut nodes_by_distance_matrix = Vec::new();
    for distances in distance_matrix.iter() {
        let max_dist = distances.values().max().copied().expect("Empty distance vector");
        let mut hist = vec![HashSet::new(); max_dist + 1];
        for (node, dist) in distances.iter() {
            hist[*dist].insert(*node);
        }
        nodes_by_distance_matrix.push(hist);
    }

    let mut walker = RandomWalker::new(
        config.explorer,
        config.selector,
        config.config
    );

    for node in graph.nodes.iter() {
        let start = RawNodeRef(node.uid).upgrade(graph.uid);
        if !on_nodes.contains(&start) {
            continue;
        }
        for _ in 0..config.n_rounds {
            let path = walker.walk_from(start)?
                .into_iter()
                .map(|r| r.node_uid)
                .collect::<Vec<_>>();
            let out = evaluate_walk(
                node.uid, path, &distance_matrix, &nodes_by_distance_matrix
            );
            let (coverage_per_level, cumulative_coverage_per_level) = out;
            total_runs += 1.0;
            summed_max_dist += (coverage_per_level.len() - 1) as f64;
            summed_max_dist_exploration_density = cumulative_coverage_per_level.last()
                .copied()
                .expect("Empty cumulative coverage");
            max_dist = usize::max(max_dist, coverage_per_level.len());
            for (i, x) in coverage_per_level.into_iter().enumerate() {
                total_runs_by_dist[i] += 1.0;
                summed_exploration_density[i] += x;
            }
            for (i, x) in cumulative_coverage_per_level.into_iter().enumerate() {
                summed_cumulative_exploration_density[i] += x;
            }
        }
    }

    summed_exploration_density.truncate(max_dist + 1);
    summed_cumulative_exploration_density.truncate(max_dist + 1);
    total_runs_by_dist.truncate(max_dist + 1);

    Ok(
        EvalResult {
            exploration_density: summed_exploration_density.into_iter()
                .zip(total_runs_by_dist.iter())
                .map(|(x, y)| x / y)
                .collect(),
            cumulative_exploration_density: summed_cumulative_exploration_density.into_iter()
                .zip(total_runs_by_dist.iter())
                .map(|(x, y)| x / y)
                .collect(),
            max_dist: summed_max_dist / total_runs,
            max_dist_exploration_density: summed_max_dist_exploration_density / total_runs
        }
    )
}


fn evaluate_walk(start: usize,
                 path: Vec<usize>,
                 distance_matrix: &[HashMap<usize, usize>],
                 nodes_by_distance_matrix: &[Vec<HashSet<usize>>]) -> (Vec<f64>, Vec<f64>)
{
    let distance_vector = &distance_matrix[start];
    let nodes_by_distance = &nodes_by_distance_matrix[start];

    let max_dist = path.iter()
        .map(|n| distance_vector.get(n).copied().expect("No node distance"))
        .max()
        .expect("Empty Path");

    // TODO: is the start in the path by default?
    let mut nodes_in_path_by_distance = vec![HashSet::new(); max_dist + 1];
    nodes_in_path_by_distance[0].insert(start);
    for node in path {
        let dist = distance_vector.get(&node).copied().expect("No node distance");
        nodes_in_path_by_distance[dist].insert(node);
    }

    let coverage = nodes_in_path_by_distance.iter()
        .enumerate()
        .map(|(dist, nodes)| {
            let all_at_dist = &nodes_by_distance[dist];
            let visited = (nodes & all_at_dist).len() as f64;
            let total = all_at_dist.len() as f64;
            (visited, total)
        })
        .collect::<Vec<_>>();

    let coverage_per_level = coverage.iter()
        .map(|&(v, t)| v / t)
        .collect::<Vec<_>>();

    let mut v_acc = 0.0;
    let mut t_acc = 0.0;
    let cumulative_coverage_per_level = coverage.into_iter()
        .map(|(v, t)| {
            v_acc += v;
            t_acc += t;
            v_acc / t_acc
        })
        .collect::<Vec<_>>();

    (coverage_per_level, cumulative_coverage_per_level)
}