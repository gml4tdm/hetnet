use crate::{HetNetError, HetNetResult, HeteroDiGraph, MetaPath};

impl HeteroDiGraph {
    pub(super) fn convert_edge_types(&self, types: Vec<String>) -> HetNetResult<Vec<usize>> {
        let metadata = &*self.edge_metadata;
        let converted = types.into_iter()
            .map(|tp|
                metadata.edge_types_reverse.get(&tp).copied()
                    .ok_or_else(|| HetNetError::UnknownType {kind: "edge".to_string(), name: tp})
            )
            .collect::<Result<Vec<_>, _>>()?;
        Ok(converted)
    }

    pub(super) fn resolve_meta_path(&self, mp: MetaPath<String>) -> HetNetResult<MetaPath<usize>> {
        let resolved = mp.resolve_types(
            &self.node_metadata.node_types_reverse,
            &self.edge_metadata.edge_types_reverse
        )?;
        Ok(resolved)
    }
}
