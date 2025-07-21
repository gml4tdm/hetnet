#[derive(Debug, Clone)]
pub struct Node {
    pub(crate) uid: usize,
    pub(crate) r#type: usize,
    pub(crate) connections: Vec<Edge>
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Edge {
    pub(crate) r#type: usize,
    pub(crate) to: usize,
    pub(crate) weight: f64,
    pub(crate) uid: usize
}


#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct NodeRef(pub(crate) usize);


#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct EdgeRef(pub(crate) usize);


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
