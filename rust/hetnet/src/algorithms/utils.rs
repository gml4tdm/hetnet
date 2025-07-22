use std::collections::HashSet;
use crate::{HetNetError, HetNetResult, HeteroDiGraph};
use crate::graph::EdgeTypeRef;

impl HeteroDiGraph {
    pub(super) fn convert_edge_types(&self, types: Vec<String>) -> HetNetResult<Vec<EdgeTypeRef>> {
        let metadata = &*self.edge_metadata;
        let converted = types.into_iter()
            .map(|tp|
                metadata.edge_types_reverse.get(&tp).copied()
                    .ok_or_else(|| HetNetError::UnknownType {kind: "edge".to_string(), name: tp})
            )
            .collect::<Result<Vec<_>, _>>()?;
        Ok(converted)
    }
}