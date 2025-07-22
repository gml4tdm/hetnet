use crate::{NodeRef, EdgeRef};


#[derive(Debug, Clone)]
pub struct NodeDescriptor {
    pub(crate) uid: NodeRef,
    pub(crate) r#type: String
}

#[derive(Debug, Clone)]
pub struct EdgeDescriptor {
    pub(crate) uid: EdgeRef,
    pub(crate) from: NodeRef,
    pub(crate) to: NodeRef,
    pub(crate) r#type: String,
    pub(crate) weight: f64,
}

impl NodeDescriptor {
    pub fn uid(&self) -> NodeRef {
        self.uid
    }

    pub fn r#type(&self) -> &str {
        self.r#type.as_str()
    }
}

impl EdgeDescriptor {
    pub fn uid(&self) -> EdgeRef {
        self.uid
    }

    pub fn from(&self) -> NodeRef {
        self.from
    }

    pub fn to(&self) -> NodeRef {
        self.to
    }

    pub fn r#type(&self) -> &str {
        self.r#type.as_str()
    }

    pub fn weight(&self) -> f64 {
        self.weight
    }
}
