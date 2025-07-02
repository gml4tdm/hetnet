use std::collections::HashMap;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use crate::graph::{HeteroDiGraph, HeteroDiGraphBuilder, NodeRef};
use crate::meta_path::{MetaPath, PathComponent};

pub mod graph;
mod errors;
pub mod meta_path;


#[pyclass(name = "MetaPath")]
#[derive(Clone)]
struct PyMetaPath(MetaPath<String>);

#[pymethods]
impl PyMetaPath {
    #[new]
    fn new(pattern: String) -> PyResult<Self> {
        let inner = MetaPath::new(pattern)?;
        Ok(PyMetaPath(inner))
    }
    
    fn __repr__(&self) -> String {
        let mut parts = vec![_format_mp_node(&self.0.start)];
        for (edge, node) in self.0.steps.iter() {
            parts.push(_format_mp_edge(edge));
            parts.push(_format_mp_node(node));
        }
        format!("MetaPath(\"{}\")", parts.join(" "))
    }
}

fn _format_mp_node(x: &PathComponent<String>) -> String {
    match x {
        PathComponent::Typed(inner) => format!("[{inner}]"),
        PathComponent::Wildcard => "[*]".to_string(),
    }
}

fn _format_mp_edge(e: &PathComponent<String>) -> String {
    match e {
        PathComponent::Typed(inner) => format!("-{{{inner}}}->"),
        PathComponent::Wildcard => "->".to_string(),
    }
}


#[pyclass(name = "Graph")]
struct PyHeteroDiGraph(HeteroDiGraph);

#[pymethods]
impl PyHeteroDiGraph {
    fn node_list(&self) -> Vec<(usize, String)> {
        self.0.node_list()
    }
    
    fn edge_list(&self) -> Vec<(usize, usize, String, usize)> {
        self.0.edge_list()
    }
    
    fn node_properties(&self, node: usize) -> PyResult<&HashMap<String, String>> {
        Ok(self.0.node_properties(node)?)
    }
    
    fn edge_properties(&self, 
                       source: usize, 
                       destination: usize,
                       r#type: String) -> PyResult<&HashMap<String, String>> {
        Ok(self.0.edge_properties(source, destination, r#type)?)
    }
    
    fn meta_path_subgraph(&self, 
                          meta_paths: HashMap<String, PyMetaPath>) -> PyResult<Self>
    {
        let meta_paths = meta_paths.into_iter()
            .map(|(name, meta_path)| (name, meta_path.0))
            .collect();
        Ok(PyHeteroDiGraph(self.0.meta_path_subgraph(meta_paths)?))
    }
    
    fn _debug(&self) -> String {
        self.0.debug()
    }
}

#[pyclass(name = "GraphBuilder")]
struct PyHeteroDiGraphBuilder(HeteroDiGraphBuilder, bool);

#[pymethods]
impl PyHeteroDiGraphBuilder {
    
    #[new]
    fn new() -> Self {
        Self(HeteroDiGraphBuilder::new(), false)
    }
    
    #[pyo3(signature = (r#type, *, properties = None))]
    fn add_node(&mut self, r#type: String, properties: Option<HashMap<String, String>>) -> PyResult<PyNodeRef> {
        if self.1 {
            return Err(PyErr::new::<PyException, _>("Graph already built"));
        }
        Ok(PyNodeRef(self.0.add_node(r#type, properties)))
    }
    
    #[pyo3(signature = (source, destination, r#type, *, properties = None))]
    fn add_edge(&mut self,
                source: PyNodeRef,
                destination: PyNodeRef,
                r#type: String,
                properties: Option<HashMap<String, String>>) -> PyResult<()> {
        if self.1 {
            return Err(PyErr::new::<PyException, _>("Graph already built"));
        }
        self.0.add_edge(
            source.0,
            destination.0,
            r#type,
            properties
        );
        Ok(())
    }
    
    fn build(&mut self) -> PyResult<PyHeteroDiGraph> {
        if self.1 {
            return Err(PyErr::new::<PyException, _>("Graph already built"));
        }
        self.1 = true;
        Ok(PyHeteroDiGraph(self.0.clone().build()))
    }
}

#[pyclass(name = "NodeRef")]
#[derive(Copy, Clone)]
struct PyNodeRef(NodeRef);

#[pymethods]
impl PyNodeRef {
    fn __repr__(&self) -> String {
        format!("NodeRef<{}>", self.0.0)
    }
}


/// A Python module implemented in Rust.
#[pymodule]
fn _hetnet(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyMetaPath>()?;
    m.add_class::<PyHeteroDiGraph>()?;
    m.add_class::<PyHeteroDiGraphBuilder>()?;
    m.add_class::<PyNodeRef>()?;
    Ok(())
}
