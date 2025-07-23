mod graph;
mod errors;
mod builder;
mod meta_path;
mod descriptors;
mod algorithms;

// Public exports
pub use graph::{
    HeteroDiGraph,
    NodeRef,
    EdgeRef,
};
pub use builder::HeteroDiGraphBuilder;
pub use meta_path::MetaPath;
pub use descriptors::{NodeDescriptor, EdgeDescriptor};
pub use errors::{HetNetError, HetNetResult};

pub mod deduplication {
    pub use crate::algorithms::deduplicate::DataHandling;
    pub use crate::algorithms::deduplicate::WeightHandling;
}

pub mod walkers {
    pub use crate::algorithms::neighbourhood::Neighbours;
    pub use crate::algorithms::neighbourhood::Node2VecArgs;
    pub use crate::algorithms::walkers::{
        RandomWalkConfig,
        UnweightedNeighbourSelector,
        WeightedNeighbourSelector,
        GraphExplorer,
        NeighbourSelector,
        RandomWalker
    };
}
