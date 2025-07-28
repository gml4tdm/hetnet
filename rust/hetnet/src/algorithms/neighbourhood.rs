//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Imports
//////////////////////////////////////////////////////////////////////////////////////////////////

use std::collections::{HashMap, HashSet};

use crate::{HetNetError, HeteroDiGraph};
use crate::errors::GraphQueryingError;
use crate::graph::RawNodeRef;

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Type -- Node2Vec Args
//////////////////////////////////////////////////////////////////////////////////////////////////


pub struct Node2VecArgs {
    pub(super) p: f64,
    pub(super) q: f64
}

impl Default for Node2VecArgs {
    fn default() -> Self {
        Self::new(1.0, 1.0)
    }
}

impl Node2VecArgs {
    pub fn new(p: f64, q: f64) -> Self {
        Self { p, q }
    }

    pub fn p(mut self, p: f64) -> Self {
        self.p = p;
        self
    }

    pub fn q(mut self, q: f64) -> Self {
        self.q = q;
        self
    }
}


//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Type -- Neighbours
//////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Neighbours<'a> {
    graph: &'a HeteroDiGraph,
}


impl HeteroDiGraph {
    pub fn neighbours(&self) -> Neighbours {
        Neighbours { graph: self }
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Neighbours -- Implementation
//////////////////////////////////////////////////////////////////////////////////////////////////


impl<'a> super::walkers::GraphExplorer for Neighbours<'a> {
    type State = Option<RawNodeRef>;
    type Config = Node2VecArgs;

    fn graph_uid(&self) -> usize {
        self.graph.uid
    }

    fn is_markov_graph(&self) -> bool {
        self.graph.graph_metadata.is_markov
    }

    fn neighbours(
        &self,
        RawNodeRef(uid): RawNodeRef,
        state: &mut Self::State,
        config: &Self::Config
    ) -> Result<HashMap<RawNodeRef, f64>, HetNetError>
    {
        let node = self.graph.nodes.get(uid)
            .ok_or(GraphQueryingError::InvalidNodeId{uid})?;
        let stream = node.connections.iter()
            .map(|edge| (edge.to, edge.weight));
        let mut result = HashMap::new();
        match state {
            None => {
                for (item, w) in stream {
                    result.entry(item).and_modify(|x| *x += w).or_insert(w);
                }
            }
            Some(prev) => {
                let reachable = self.graph.nodes.get(prev.0)
                    .expect("Invalid state")
                    .connections
                    .iter()
                    .map(|edge| edge.to)
                    .collect::<HashSet<_>>();
                for (to, w) in stream {
                    let p = if *prev == to {
                        1.0/config.p * w
                    } else if reachable.contains(&to) {
                        w
                    } else {
                        1.0/config.q * w
                    };
                    result.entry(to).and_modify(|x| *x += p).or_insert(p);
                }
            }
        }
        Ok(result)
    }
}
