//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Types
//////////////////////////////////////////////////////////////////////////////////////////////////

use std::collections::HashMap;

#[derive(Debug)]
pub struct HeteroDiGraph {
    metadata: GraphMetaData,
    nodes: Vec<Node>
}

#[derive(Debug)]
struct GraphMetaData {
    node_types: Vec<String>,
    edge_types: Vec<String>,
}

#[derive(Debug)]
struct Node {
    uid: usize,
    r#type: usize,
    connections: Vec<Edge>
}

#[derive(Debug, Copy, Clone)]
struct Edge {
    r#type: usize,
    to: usize,
}

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Builder
//////////////////////////////////////////////////////////////////////////////////////////////////


#[derive(Debug, Default)]
pub struct HeteroDiGraphBuilder {
    nodes: Vec<Node>,
    node_types: HashMap<String, usize>,
    edge_types: HashMap<String, usize>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct NodeRef(usize);

impl HeteroDiGraphBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn add_node(&mut self, r#type: String) -> NodeRef {
        let uid = self.nodes.len();
        let next_type_id = self.node_types.len();
        let type_id = *self.node_types
            .entry(r#type.clone())
            .or_insert(next_type_id);
        self.nodes.push(Node {
            uid,
            r#type: type_id,
            connections: Vec::new(),
        });
        NodeRef(uid)
    }
    
    pub fn add_edge(&mut self, from: NodeRef, to: NodeRef, r#type: String) {
        let next_type_id = self.edge_types.len();
        let type_id = *self.edge_types
            .entry(r#type.clone())
            .or_insert(next_type_id);
        self.nodes.get_mut(from.0)
            .expect("Invalid node reference")
            .connections
            .push(Edge{r#type: type_id, to: to.0});
    }
    
    pub fn build(self) -> HeteroDiGraph {
        let metadata = GraphMetaData {
            node_types: Self::convert_mapping(self.node_types),
            edge_types: Self::convert_mapping(self.edge_types),
        };
        HeteroDiGraph {
            metadata,
            nodes: self.nodes,
        }
    }
    
    fn convert_mapping(m: HashMap<String, usize>) -> Vec<String> {
        let mut as_vec = m.into_iter().collect::<Vec<_>>();
        as_vec.sort_by_key(|x| x.1);
        as_vec.into_iter().map(|x| x.0).collect()
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Builder
//////////////////////////////////////////////////////////////////////////////////////////////////