use pyo3::prelude::*;
use pyo3::types::PyAny;
use grasp::graph::{adjacency_list::SparseSimpleGraph, graph_ops::GraphOps, GraphTrait};
use gdraw::app::GraspApp;

#[pyfunction]
fn open_app_with_graph(py_graph: &PyAny) -> PyResult<()> {
    let py_edges = py_graph.call_method0("edges")?;
    let edges: Vec<(usize, usize)> = py_edges.extract()?;
    let mut graph = SparseSimpleGraph::default();
    for (u, v) in edges {
        graph.add_edge((u, v));
    }
    let mut app = GraspApp::default();
    app.load(&graph);
    let _ = app.start();
    Ok(())
}

#[pymodule]
#[pyo3(name="gdraw")]
fn gdraw_lib(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(open_app_with_graph, m)?)?;
    Ok(())
}