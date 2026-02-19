use pyo3::prelude::*;
use grasp_core::graph::prelude::{SparseSimpleGraph, GraphTrait, VertexID, GraphError};
use grasp_core::algorithms::algo_traits::AlgoTrait;

#[pyclass]
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

    fn neighbors(&self, v: usize) -> PyResult<Vec<usize>> {
        self.inner
            .neighbors(v)
            .map(|s| s.iter().copied().collect())
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Vertex {} not in graph", v)
            ))
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
}

fn graph_error_to_py(err: GraphError) -> PyErr {
    pyo3::exceptions::PyValueError::new_err(format!("{err:?}"))
}

#[pymodule]
fn grasp(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySparseGraph>()?;
    Ok(())
}