error_set::error_set!{
    HetNetError = MetaPathDefinitionError || GraphQueryingError || InternalError || GraphProcessingError;

    InternalError = {
        #[display("Error in concurrency primitive: {detail}")]
        ConcurrencyError{detail: String}
    };

    MetaPathDefinitionError = {
        #[display("Invalid syntax in meta path definition: {detail}")]
        InvalidSyntax{detail: String},

        #[display("Unknown {kind} type: {name}")]
        UnknownType{kind: String, name: String},
    };

    GraphQueryingError = {
        #[display("Invalid Reference for Graph")]
        InvalidReference,
        
        #[display("Unknown {kind} type: {name}")]
        UnknownType{kind: String, name: String},

        #[display("Invalid Node ID: {uid}")]
        InvalidNodeId{uid: usize},

        #[display("Invalid Edge ID: {uid}")]
        InvalidEdgeId{uid: usize},

        #[display("No edge of type {kind} between nodes {src} and {tgt}")]
        NoSuchEdge{kind: String, src: usize, tgt: usize},
    };
    
    GraphProcessingError = {
        #[display("Not all edge weights equal while deduplicating")]
        NotAllEdgeWeightsEqual,
        
        #[display("Not all edge properties equal while deduplicating")]
        NotAllEdgePropertiesEqual,
    };
}

pub type HetNetResult<T> = Result<T, HetNetError>;
