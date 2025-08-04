use crate::{HetNetResult, HeteroDiGraph, NodeRef};
use crate::walkers::{
    UnweightedNeighbourSelector,
    WeightedNeighbourSelector,
    Node2VecArgs,
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

pub struct EvalResult {
    exploration_density: Vec<Vec<f64>>,
}


pub fn evaluate_random_walk_config<G, N>(
    graph: &HeteroDiGraph,
    on_nodes: Vec<NodeRef>,
    config: EvalArgs<G, N>) -> HetNetResult<EvalResult>
where
    G: GraphExplorer,
    N: NeighbourSelector
{
    Ok(EvalResult{ exploration_density: vec![] })
}
