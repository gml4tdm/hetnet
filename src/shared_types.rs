#[derive(Debug, Clone)]
pub struct Node {
    pub(crate) uid: usize,
    pub(crate) r#type: usize,
    pub(crate) connections: Vec<Edge>
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Edge {
    pub(crate) r#type: usize,
    pub(crate) to: usize,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct NodeRef(pub(crate) usize);
