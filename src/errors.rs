use pyo3::exceptions::PyException;
use pyo3::PyErr;

error_set::error_set!{
    HetNetError = MetaPathDefinitionError || GraphQueryingError;
    
    MetaPathDefinitionError = {
        #[display("Invalid syntax in meta path definition: {detail}")]
        InvalidSyntax{detail: String},
        
        #[display("Unknown {kind} type: {name}")]
        UnknownType{kind: String, name: String},
    };
    
    GraphQueryingError = {
        #[display("Unknown {kind} type: {name}")]
        UnknownType{kind: String, name: String},
        
        #[display("Invalid Node ID: {uid}")]
        InvalidNodeId{uid: usize},
        
        #[display("No edge of type {kind} between nodes {src} and {tgt}")]
        NoSuchEdge{kind: String, src: usize, tgt: usize},
    };
}

impl From<HetNetError> for PyErr {
    fn from(value: HetNetError) -> Self {
        PyErr::new::<PyException, _>(value.to_string())
    }
}

impl From<MetaPathDefinitionError> for PyErr {
    fn from(value: MetaPathDefinitionError) -> Self {
        HetNetError::from(value).into()
    }
}

impl From<GraphQueryingError> for PyErr {
    fn from(value: GraphQueryingError) -> Self {
        HetNetError::from(value).into()
    }
}