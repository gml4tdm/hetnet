//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Imports
//////////////////////////////////////////////////////////////////////////////////////////////////

use std::collections::{HashMap, HashSet};

use crate::errors::HetNetError;
use crate::shared_types::NodeRef;

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Traits
//////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) trait GraphExplorer {
    fn neighbours(&self, node: NodeRef) -> Result<HashMap<NodeRef, usize>, HetNetError>;
}

pub trait NeighbourSelector {
    fn select(&mut self, from: &[(NodeRef, usize)]) -> NodeRef;
}

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Neighbour Selectors
//////////////////////////////////////////////////////////////////////////////////////////////////

pub struct UnweightedNeighbourSelector<T: rand::Rng> {
    rng: T
}

impl Default for UnweightedNeighbourSelector<rand::rngs::ThreadRng> {
    fn default() -> Self {
        Self::with_rng(rand::rng())
    }
}

impl<T: rand::Rng> UnweightedNeighbourSelector<T> {
    fn with_rng(rng: T) -> Self {
        UnweightedNeighbourSelector { rng }
    }
}

impl<T: rand::Rng> NeighbourSelector for UnweightedNeighbourSelector<T> {
    fn select(&mut self, from: &[(NodeRef, usize)]) -> NodeRef {
        let index = self.rng.random_range(0..from.len());
        from[index].0
    }
}


pub struct WeightedNeighbourSelector<T: rand::Rng> {
    rng: T
}

impl Default for WeightedNeighbourSelector<rand::rngs::ThreadRng> {
    fn default() -> Self {
        Self::with_rng(rand::rng())
    }
}

impl<T: rand::Rng> WeightedNeighbourSelector<T> {
    fn with_rng(rng: T) -> Self {
        WeightedNeighbourSelector { rng }
    }
}

impl<T: rand::Rng> NeighbourSelector for WeightedNeighbourSelector<T> {
    fn select(&mut self, from: &[(NodeRef, usize)]) -> NodeRef {
        let mut hist = Vec::new();
        let mut total = 0;
        for (_, count) in from.iter().copied() {
            total += count;
            hist.push(total);
        }

        let selected = self.rng.random_range(0..total);
        let index = hist.partition_point(|&x| x <= selected);
        from[index].0
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Random Walker
//////////////////////////////////////////////////////////////////////////////////////////////////


pub struct RandomWalkConfig {
    path_length: usize,
    p: f64,
    q: f64
}

impl Default for RandomWalkConfig {
    fn default() -> Self {
        Self {
            path_length: 10,
            p: 1.0,
            q: 1.0
        }
    }
}

impl RandomWalkConfig {
    pub fn with_path_length(mut self, path_length: usize) -> Self {
        self.path_length = path_length;
        self
    }

    pub fn with_p(mut self, p: f64) -> Self {
        self.p = p;
        self
    }

    pub fn with_q(mut self, q: f64) -> Self {
        self.q = q;
        self
    }
}


pub struct RandomWalker<G: GraphExplorer, N: NeighbourSelector> {
    explorer: G,
    selector: N,
    config: RandomWalkConfig,
}

impl<G: GraphExplorer, N: NeighbourSelector> RandomWalker<G, N> {
    pub fn new(explorer: G, selector: N, config: RandomWalkConfig) -> Self {
        RandomWalker { explorer, selector, config }
    }

    pub fn walk_from(&mut self, start: NodeRef) -> Result<Vec<NodeRef>, HetNetError> {
        let mut path = vec![start];
        let mut current = start;

        for _ in 0..self.config.path_length {
            let neighbours = self.explorer.neighbours(current)?;
            let histogram = neighbours.into_iter().collect::<Vec<_>>();
            current = self.selector.select(&histogram);
            path.push(current);
        }

        Ok(path)
    }

    pub fn distribution(&mut self,
                        start: NodeRef,
                        n_iter: usize) -> Result<HashMap<NodeRef, usize>, HetNetError>
    {
        let mut dist = HashMap::new();

        for _ in 0..n_iter {
            let path = self.walk_from(start)?;
            let unique = path.iter().copied().collect::<HashSet<_>>();
            for node in unique {
                dist.entry(node).and_modify(|x| *x += 1).or_insert(1);
            }
        }

        Ok(dist)
    }
}
