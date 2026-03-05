use crate::{HetNetError, HetNetResult, HeteroDiGraph, MetaPath};

impl HeteroDiGraph {
    pub(super) fn convert_edge_types(
        &self,
        types: Vec<String>,
        allow_unknown_types: bool
    ) -> HetNetResult<Vec<usize>>
    {
        let metadata = &*self.edge_metadata;
        let converted = types.into_iter()
            .map(|tp|
                metadata.edge_types_reverse.get(&tp).copied()
                    .ok_or_else(|| HetNetError::UnknownType {kind: "edge".to_string(), name: tp})
            )
            .filter(|r| r.is_ok() || !allow_unknown_types)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(converted)
    }

    pub(super) fn resolve_meta_path(
        &self,
        mp: MetaPath<String>,
        allow_unknown_types: bool
    ) -> HetNetResult<Option<MetaPath<usize>>>
    {
        let result = mp.resolve_types(
            &self.node_metadata.node_types_reverse,
            &self.edge_metadata.edge_types_reverse
        );
        match result {
            Ok(mp) => Ok(Some(mp)),
            Err(_) if allow_unknown_types => Ok(None),
            Err(e) => Err(e.into())
        }
    }
}
