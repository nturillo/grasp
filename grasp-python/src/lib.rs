use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::exceptions::PyValueError;

use grasp::graph::prelude::*;
use grasp::algorithms::algo_traits::{AlgoTrait};

#[pyclass(name = "SparseGraph")]
#[derive(Debug)]
pub struct PySparseGraph {
    inner: SparseSimpleGraph,
}

#[pymethods]
impl PySparseGraph {
    #[new]
    fn new() -> Self {
        Self {
            inner: SparseSimpleGraph::default(),
        }
    }

    fn create_vertex(&mut self) -> usize {
        self.inner.create_vertex()
    }

    fn add_vertex(&mut self, v: usize) {
        self.inner.add_vertex(v);
    }

    fn add_edge(&mut self, v1: usize, v2: usize) {
        self.inner.add_edge((v1, v2));
    }

    fn add_neighbors(&mut self, v: usize, neighbors: Vec<usize>) {
        self.inner.add_neighbors(v, neighbors.into_iter());
    }

    fn delete_vertex(&mut self, v: usize) -> Vec<(usize, usize)> {
        self.inner.delete_vertex(v).collect()
    }

    fn delete_edge(&mut self, v1: usize, v2: usize) {
        self.inner.delete_edge((v1, v2));
    }

    fn contains(&self, v: usize) -> bool {
        self.inner.contains(v)
    }

    fn has_edge(&self, v1: usize, v2: usize) -> bool {
        self.inner.has_edge((v1, v2))
    }

    fn num_vertices(&self) -> usize {
        self.inner.vertex_count()
    }

    fn num_edges(&self) -> usize {
        self.inner.edge_count()
    }

    fn vertices(&self) -> Vec<usize> {
        self.inner.vertices().collect()
    }

    fn edges(&self) -> Vec<(usize, usize)> {
        self.inner.edges().collect()
    }

    fn vertex_set(&self) -> Vec<usize> {
        self.inner.vertex_set().into_iter().collect()
    }

    fn neighbors(&self, v: usize) -> PyResult<Vec<usize>> {
        match self.inner.neighbors(v) {
            Some(s) => Ok(s.into_owned().into_iter().collect()),
            None => Err(PyValueError::new_err(format!("Vertex {} not in graph", v))),
        }
    }

    fn bfs(&self, source: usize) -> PyResult<Vec<usize>> {
        self.inner
            .bfs_iter(source)
            .map(|it| it.collect::<Vec<VertexID>>())
            .map_err(graph_error_to_py)
    }

    fn dfs(&self, source: usize) -> PyResult<Vec<usize>> {
        self.inner
            .dfs_iter(source)
            .map(|it| it.collect::<Vec<VertexID>>())
            .map_err(graph_error_to_py)
    }

    fn dijkstra(&self, source: usize, weights: std::collections::HashMap<(usize, usize), f64>) -> PyResult<(std::collections::HashMap<usize, f64>, std::collections::HashMap<usize, usize>)> {
        let weight_fn = move |_g: &SparseSimpleGraph, edge: EdgeID| -> Option<f64> {
            let (u, v) = edge;
            weights
                .get(&(u, v))
                .copied()
                .or_else(|| weights.get(&(v, u)).copied())
        };

        let mut iter = self
            .inner
            .dijkstra_iter(source, weight_fn)
            .map_err(graph_error_to_py)?;

        let mut distances = std::collections::HashMap::new();

        while let Some(result) = iter.next() {
            let (v, d) = result.map_err(graph_error_to_py)?;
            distances.insert(v, d);
        }

        let mut predecessors = std::collections::HashMap::new();
        for (k, v) in iter.predecessors().iter() {
            predecessors.insert(*k, *v);
        }

        Ok((distances, predecessors))
    }

    fn subgraph_vertex(&self, vertices: Vec<usize>, py: Python) -> PyResult<PyObject> {
        let sub = self.inner.subgraph_vertex(vertices.into_iter(), SparseSimpleGraph::default);
        let obj = Py::new(py, PySparseGraph { inner: sub })?;
        Ok(obj.into_py(py))
    }

    fn subgraph_edges(&self, edges: Vec<(usize, usize)>, py: Python) -> PyResult<PyObject> {
        let sub = self.inner.subgraph_edges(edges.into_iter(), SparseSimpleGraph::default);
        let obj = Py::new(py, PySparseGraph { inner: sub })?;
        Ok(obj.into_py(py))
    }

    fn merge(&self, other: &PySparseGraph, py: Python) -> PyResult<(PyObject, PyObject, PyObject)> {
        let (merged, self_map, other_map) = self
            .inner
            .merge(&other.inner, SparseSimpleGraph::default);

        let merged_obj = Py::new(py, PySparseGraph { inner: merged })?.into_py(py);

        let dict_self = PyDict::new_bound(py);
        for (k, v) in self_map {
            dict_self.set_item(k, v)?;
        }

        let dict_other = PyDict::new_bound(py);
        for (k, v) in other_map {
            dict_other.set_item(k, v)?;
        }

        Ok((merged_obj, dict_self.into_py(py), dict_other.into_py(py)))
    }

    fn complement(&self, py: Python) -> PyResult<PyObject> {
        let comp = self.inner.complement(SparseSimpleGraph::default);
        let obj = Py::new(py, PySparseGraph { inner: comp })?;
        Ok(obj.into_py(py))
    }

    fn join(&self, other: &PySparseGraph, py: Python) -> PyResult<(PyObject, PyObject, PyObject)> {
        let (joined, self_map, other_map) = self.inner.join(&other.inner, || SparseSimpleGraph::default());
        let joined_obj = Py::new(py, PySparseGraph { inner: joined })?.into_py(py);
        let dict_self = PyDict::new_bound(py);
        for (k, v) in self_map {
            dict_self.set_item(k, v)?;
        }
        let dict_other = PyDict::new_bound(py);
        for (k, v) in other_map {
            dict_other.set_item(k, v)?;
        }
        Ok((joined_obj, dict_self.into_py(py), dict_other.into_py(py)))
    }

    fn product(&self, other: &PySparseGraph, py: Python) -> PyResult<(PyObject, PyObject)> {
        let (product, map) = self.inner.product(&other.inner, || SparseSimpleGraph::default());
        let product_obj = Py::new(py, PySparseGraph { inner: product })?.into_py(py);
        let dict_map = PyDict::new_bound(py);
        for ((a,b), v) in map {
            dict_map.set_item((a,b), v)?;
        }
        Ok((product_obj, dict_map.into_py(py)))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("PySparseGraph(vertices={}, edges={})", self.inner.vertex_count(), self.inner.edge_count()))
    }
}

#[pyclass(name = "SparseDiGraph")]
#[derive(Debug)]
pub struct PySparseDiGraph {
    inner: SparseDiGraph,
}

#[pymethods]
impl PySparseDiGraph {
    #[new]
    fn new() -> Self {
        Self { inner: SparseDiGraph::default() }
    }

    fn create_vertex(&mut self) -> usize {
        self.inner.create_vertex()
    }

    fn add_vertex(&mut self, v: usize) {
        self.inner.add_vertex(v);
    }

    fn add_edge(&mut self, v1: usize, v2: usize) {
        self.inner.add_edge((v1, v2));
    }

    fn num_vertices(&self) -> usize {
        self.inner.vertex_count()
    }

    fn num_edges(&self) -> usize {
        self.inner.edge_count()
    }

    fn vertices(&self) -> Vec<usize> {
        self.inner.vertices().collect()
    }

    fn edges(&self) -> Vec<(usize, usize)> {
        self.inner.edges().collect()
    }

    fn neighbors(&self, v: usize) -> PyResult<Vec<usize>> {
        match self.inner.neighbors(v) {
            Some(s) => Ok(s.into_owned().into_iter().collect()),
            None => Err(PyValueError::new_err(format!("Vertex {} not in graph", v))),
        }
    }

    fn out_neighbors(&self, v: usize) -> PyResult<Vec<usize>> {
        match self.inner.out_neighbors(v) {
            Some(s) => Ok(s.into_owned().into_iter().collect()),
            None => Err(PyValueError::new_err(format!("Vertex {} not in graph", v))),
        }
    }

    fn in_neighbors(&self, v: usize) -> PyResult<Vec<usize>> {
        match self.inner.in_neighbors(v) {
            Some(s) => Ok(s.into_owned().into_iter().collect()),
            None => Err(PyValueError::new_err(format!("Vertex {} not in graph", v))),
        }
    }

    fn to_simple(&self, py: Python) -> PyResult<PyObject> {
        let simple = self.inner.underlying_graph();
        let obj = Py::new(py, PySparseGraph { inner: simple })?;
        Ok(obj.into_py(py))
    }
}



fn graph_error_to_py(err: GraphError) -> PyErr {
    pyo3::exceptions::PyValueError::new_err(format!("{err:?}"))
}

#[pymodule]
#[pyo3(name="grasp")]
fn grasp_lib(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySparseGraph>()?;
    m.add_class::<PySparseDiGraph>()?;
    Ok(())
}