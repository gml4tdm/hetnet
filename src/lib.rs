//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Imports and modules
//////////////////////////////////////////////////////////////////////////////////////////////////

use std::collections::HashMap;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

use hetnet::{
    HeteroDiGraph,
    HeteroDiGraphBuilder,
    MetaPath,
    NodeDescriptor, NodeRef,
    EdgeDescriptor, EdgeRef,
    walkers::{
       UnweightedNeighbourSelector,
       WeightedNeighbourSelector,
       Node2VecArgs,
       RandomWalkConfig,
       GraphExplorer,
       NeighbourSelector,
       RandomWalker,
    },
    deduplication as dedup
};

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Error Handling
//////////////////////////////////////////////////////////////////////////////////////////////////

fn convert_result<T, E: std::fmt::Display>(r: Result<T, E>) -> PyResult<T> {
    r.map_err(|e| PyErr::new::<PyException, _>(format!("{}", e)))
}

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Meta-path
//////////////////////////////////////////////////////////////////////////////////////////////////

#[pyclass(name = "MetaPath")]
#[derive(Clone)]
struct PyMetaPath(MetaPath<String>);

#[pymethods]
impl PyMetaPath {
    #[new]
    fn new(pattern: String) -> PyResult<Self> {
        let inner = convert_result(MetaPath::new(pattern))?;
        Ok(PyMetaPath(inner))
    }

    fn reverse(&self) -> Self {
        PyMetaPath(self.0.reverse())
    }

    fn __repr__(&self) -> String {
        format!("MetaPath(\"{}\")", self.0)
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Graph Object
//////////////////////////////////////////////////////////////////////////////////////////////////


#[pyclass(name = "Graph")]
struct PyHeteroDiGraph(HeteroDiGraph);

#[pymethods]
impl PyHeteroDiGraph {
    fn node_info(&self, node: PyNodeRef) -> PyResult<PyNodeDescriptor> {
        Ok(PyNodeDescriptor(convert_result(self.0.node_info(node.0))?))
    }

    fn node_list(&self) -> Vec<PyNodeDescriptor> {
        self.0.node_list().into_iter().map(PyNodeDescriptor).collect()
    }

    fn edge_list(&self) -> Vec<PyEdgeDescriptor> {
        self.0.edge_list().into_iter().map(PyEdgeDescriptor).collect()
    }

    fn node_properties(&self, node: PyNodeRef) -> PyResult<&HashMap<String, String>> {
        Ok(convert_result(self.0.node_properties(node.0))?)
    }

    fn edge_properties(&self, uid: PyEdgeRef) -> PyResult<&HashMap<String, String>> {
        Ok(convert_result(self.0.edge_properties(uid.0))?)
    }

    fn update_weights(&self, weights: HashMap<PyEdgeRef, f64>) -> PyResult<Self> {
        let conv = weights.into_iter()
            .map(|(PyEdgeRef(k), v)| (k, v))
            .collect();
        Ok(PyHeteroDiGraph(convert_result(self.0.update_weights(conv))?))
    }

    fn to_markov_graph(&self) -> Self {
        PyHeteroDiGraph(self.0.to_markov_graph())
    }

    #[pyo3(signature = (types, *, data_handling, weight_handling))]
    fn deduplicate_edges(&self,
                         types: Vec<String>,
                         data_handling: String,
                         weight_handling: String) -> PyResult<Self> {
        let dh = match data_handling.as_str() {
            "discard" => dedup::DataHandling::Discard,
            "enforce_identical" => dedup::DataHandling::EnforceIdentical,
            x => {
                return Err(PyErr::new::<PyException, _>(
                    format!(
                        "Invalid data handling '{x}', expected 'discard' or 'enforce_identical'"
                    )
                ));
            }
        };
        let wh = match weight_handling.as_str() {
            "set_to_one" => dedup::WeightHandling::SetToOne,
            "enforce_identical" => dedup::WeightHandling::EnforceIdentical,
            "sum_aggregate" => dedup::WeightHandling::SumAggregate,
            x => {
                return Err(PyErr::new::<PyException, _>(
                    format!(
                        "Invalid weight handling '{x}', expected 'set_to_one', 'enforce_identical', or 'sum_aggregate'"
                    )
                ));
            }
        };
        Ok(PyHeteroDiGraph(convert_result(self.0.deduplicate_edges(types, dh, wh))?))
    }

    #[pyo3(signature = (meta_paths, *, unique_nodes = true))]
    fn meta_path_subgraph(&self,
                          meta_paths: HashMap<String, PyMetaPath>, unique_nodes: bool) -> PyResult<Self>
    {
        let meta_paths = meta_paths.into_iter()
            .map(|(name, meta_path)| (name, meta_path.0))
            .collect();
        Ok(PyHeteroDiGraph(
            convert_result(self.0.meta_path_subgraph(meta_paths, unique_nodes))?
        ))
    }

    #[pyo3(signature = (start, *, weighted = true, path_length = 10, p = 1.0, q = 1.0))]
    fn random_walk(&mut self, start: PyNodeRef, weighted: bool, path_length: usize, p: f64, q: f64) -> PyResult<Vec<PyNodeRef>> {
        let args = Node2VecArgs::new(p, q);
        self.random_walk_helper(start, weighted, path_length, self.0.neighbours(), args)
    }

    #[pyo3(signature = (start, *, weighted = true, path_length = 10, p = 1.0, q = 1.0, n_iter = 100))]
    fn random_walk_distribution(&mut self,
                                start: PyNodeRef,
                                weighted: bool,
                                path_length: usize,
                                p: f64,
                                q: f64,
                                n_iter: usize) -> PyResult<HashMap<PyNodeRef, usize>> {
        let args = Node2VecArgs::new(p, q);
        self.random_walk_dist_helper(start, weighted, path_length, self.0.neighbours(), args, n_iter)
    }
}

impl PyHeteroDiGraph {
    fn random_walk_helper<T>(&self,
                             start: PyNodeRef,
                             weighted: bool,
                             path_length: usize,
                             explorer: T,
                             args: T::Config) -> PyResult<Vec<PyNodeRef>>
    where
        T: GraphExplorer,
    {
        if weighted {
            self.random_walk_helper_2(
                start, path_length, explorer, args, UnweightedNeighbourSelector::default()
            )
        } else {
            self.random_walk_helper_2(
                start, path_length, explorer, args, WeightedNeighbourSelector::default()
            )
        }
    }

    fn random_walk_helper_2<T, R>(&self,
                                  start: PyNodeRef,
                                  path_length: usize,
                                  explorer: T,
                                  args: T::Config,
                                  selector: R) -> PyResult<Vec<PyNodeRef>>
    where
        T: GraphExplorer,
        R: NeighbourSelector
    {
        let mut walker = RandomWalker::new(
            explorer,
            selector,
            RandomWalkConfig::<T>::default()
                .with_path_length(path_length)
                .with_selector_config(args)
        );
        let path = convert_result(walker.walk_from(start.0))?;
        Ok(path.into_iter().map(PyNodeRef).collect())
    }

    fn random_walk_dist_helper<T>(&self,
                                  start: PyNodeRef,
                                  weighted: bool,
                                  path_length: usize,
                                  explorer: T,
                                  args: T::Config,
                                  n_iter: usize) -> PyResult<HashMap<PyNodeRef, usize>>
    where
        T: GraphExplorer,
    {
        if weighted {
            self.random_walk_dist_helper_2(
                start, path_length, explorer, args, UnweightedNeighbourSelector::default(), n_iter
            )
        } else {
            self.random_walk_dist_helper_2(
                start, path_length, explorer, args, WeightedNeighbourSelector::default(), n_iter
            )
        }
    }

    fn random_walk_dist_helper_2<T, R>(&self,
                                       start: PyNodeRef,
                                       path_length: usize,
                                       explorer: T,
                                       args: T::Config,
                                       selector: R,
                                       n_iter: usize) -> PyResult<HashMap<PyNodeRef, usize>>
    where
        T: GraphExplorer,
        R: NeighbourSelector
    {
        let mut walker = RandomWalker::new(
            explorer,
            selector,
            RandomWalkConfig::default()
                .with_path_length(path_length)
                .with_selector_config(args)
        );
        let dist = convert_result(walker.distribution(start.0, n_iter))?;
        Ok(
            dist.into_iter().map(|(k, v)| (PyNodeRef(k), v)).collect()
        )
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Graph Builder
//////////////////////////////////////////////////////////////////////////////////////////////////

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

    #[pyo3(signature = (source, destination, r#type, *, weight = 1.0, properties = None))]
    fn add_edge(&mut self,
                source: PyNodeRef,
                destination: PyNodeRef,
                r#type: String,
                weight: f64,
                properties: Option<HashMap<String, String>>) -> PyResult<PyEdgeRef> {
        if self.1 {
            return Err(PyErr::new::<PyException, _>("Graph already built"));
        }
        let result = self.0.add_edge(
            source.0,
            destination.0,
            r#type,
            Some(weight),
            properties
        );
        Ok(PyEdgeRef(convert_result(result)?))
    }

    fn build(&mut self) -> PyResult<PyHeteroDiGraph> {
        if self.1 {
            return Err(PyErr::new::<PyException, _>("Graph already built"));
        }
        self.1 = true;
        Ok(PyHeteroDiGraph(self.0.clone().build()))
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Descriptors
//////////////////////////////////////////////////////////////////////////////////////////////////

#[pyclass(name = "NodeDescriptor")]
struct PyNodeDescriptor(NodeDescriptor);

#[pymethods]
impl PyNodeDescriptor {
    #[getter]
    fn r#type(&self) -> String {
        self.0.r#type().to_string()
    }

    #[getter]
    fn uid(&self) -> PyNodeRef {
        PyNodeRef(self.0.uid())
    }
}


#[pyclass(name = "EdgeDescriptor")]
struct PyEdgeDescriptor(EdgeDescriptor);

#[pymethods]
impl PyEdgeDescriptor {
    #[getter]
    fn source(&self) -> PyNodeRef {
        PyNodeRef(self.0.from())
    }

    #[getter]
    fn destination(&self) -> PyNodeRef {
        PyNodeRef(self.0.to())
    }

    #[getter]
    fn r#type(&self) -> String {
        self.0.r#type().to_string()
    }

    #[getter]
    fn uid(&self) -> PyEdgeRef {
        PyEdgeRef(self.0.uid())
    }

    #[getter]
    fn weight(&self) -> f64 {
        self.0.weight()
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Ref Objects
//////////////////////////////////////////////////////////////////////////////////////////////////

#[pyclass(name = "NodeRef", eq, hash, frozen)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct PyNodeRef(NodeRef);

#[pymethods]
impl PyNodeRef {
    fn __repr__(&self) -> String {
        format!("{}", self.0)
    }
}


#[pyclass(name = "EdgeRef", eq, hash, frozen)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct PyEdgeRef(EdgeRef);

#[pymethods]
impl PyEdgeRef {
    fn __repr__(&self) -> String {
        format!("{}", self.0)
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Module entry point
//////////////////////////////////////////////////////////////////////////////////////////////////

/// A Python module implemented in Rust.
#[pymodule]
fn _hetnet(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyMetaPath>()?;
    m.add_class::<PyHeteroDiGraph>()?;
    m.add_class::<PyHeteroDiGraphBuilder>()?;
    m.add_class::<PyNodeDescriptor>()?;
    m.add_class::<PyEdgeDescriptor>()?;
    m.add_class::<PyNodeRef>()?;
    m.add_class::<PyEdgeRef>()?;
    Ok(())
}
