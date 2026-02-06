use pyo3::prelude::*;
use grasp_core::{SparseGraph, GraphTrait, AlgoTrait, GraphError};

#[pyclass]
pub struct PySparseGraph {
    inner: SparseGraph,
}

#[pymethods]
impl PySparseGraph {
    #[new]
    fn new() -> Self {
        Self {
            inner: SparseGraph::new(),
        }
    }

    fn add_vertex(&mut self, v: usize) {
        self.inner.add_vertex(v);
    }

    fn add_edge(&mut self, v1: usize, v2: usize) {
        self.inner.add_edge((v1, v2));
    }

    fn num_vertices(&self) -> usize {
        self.inner.num_vertices()
    }

    fn num_edges(&self) -> usize {
        self.inner.num_edges()
    }

    fn neighbors(&self, v: usize) -> PyResult<Vec<usize>> {
        self.inner
            .neighbors(v)
            .map(|s| s.iter().copied().collect())
            .map_err(graph_error_to_py)
    }

    fn bfs(&self, source: usize) -> PyResult<Vec<usize>> {
        self.inner
            .bfs_iter(source)
            .map(|it| it.collect())
            .map_err(graph_error_to_py)
    }

    fn dfs(&self, source: usize) -> PyResult<Vec<usize>> {
        self.inner
            .dfs_iter(source)
            .map(|it| it.collect())
            .map_err(graph_error_to_py)
    }
}

fn graph_error_to_py(err: GraphError) -> PyErr {
    pyo3::exceptions::PyValueError::new_err(format!("{err:?}"))
}

#[pymodule]
fn grasp(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PySparseGraph>()?;
    Ok(())
}