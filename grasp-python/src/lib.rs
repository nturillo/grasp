use std::collections::HashMap;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::exceptions::PyValueError;

use grasp::graph::prelude::*;
use grasp::graph::labeled_graph::{HashMapLabeledSimpleGraph, HashMapLabeledDiGraph, LabeledGraph, LabeledGraphMut};
use grasp::algorithms::algo_traits::{AlgoTrait};
use grasp::algorithms::connectivity;
use grasp::graph::constructors::*;
use grasp::algorithms::matchings::maximum_matching;
use grasp::algorithms::trees::kruskal_mst;
use grasp::algorithms::coloring::*;
use grasp::algorithms::gonality::compute_gonality;

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

    fn remove_vertex(&mut self, v: usize) -> Vec<(usize, usize)> {
        self.inner.remove_vertex(v).collect()
    }

    fn remove_edge(&mut self, v1: usize, v2: usize) {
        self.inner.remove_edge((v1, v2));
    }

    fn has_vertex(&self, v: usize) -> bool {
        self.inner.has_vertex(v)
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
        self.inner.vertex_set().iter().map(|c| *c).collect()
    }

    fn neighbors(&self, v: usize) -> PyResult<Vec<usize>> {
        if !self.inner.has_vertex(v) {
            return Err(PyValueError::new_err(format!("Vertex {} not in graph", v)));
        }

        Ok(self.inner.neighbors(v).iter().map(|c| *c).collect())
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

    fn dijkstra(&self, source: usize) -> PyResult<(HashMap<usize, f64>, HashMap<usize, usize>)> {
        self.inner.dijkstra(source).map_err(graph_error_to_py)
    }

    fn astar(&self, source: usize, target: usize) -> PyResult<(Vec<usize>, f64)> {
        self.inner.astar(source, target).map_err(graph_error_to_py)
    }

    fn kruskal(&self, weights: HashMap<(usize, usize), f64>) -> PyResult<(Vec<(usize, usize)>, f64)> {
        let mst = kruskal_mst(&self.inner, |_, e| {
            weights
                .get(&e)
                .copied()
                .or_else(|| weights.get(&(e.1, e.0)).copied())
        })
        .map_err(graph_error_to_py)?;

        let total: f64 = mst.iter().map(|(_, _, w)| *w).sum();
        let edges = mst.into_iter().map(|(u, v, _)| (u, v)).collect();

        Ok((edges, total))
    }

    fn subgraph_vertex(&self, vertices: Vec<usize>, py: Python) -> PyResult<PyObject> {
        let sub = self.inner.subgraph_vertex(vertices.into_iter());
        let obj = Py::new(py, PySparseGraph { inner: sub })?;
        Ok(obj.into_py(py))
    }

    fn subgraph_edges(&self, edges: Vec<(usize, usize)>, py: Python) -> PyResult<PyObject> {
        let sub = self.inner.subgraph_edges(edges.into_iter());
        let obj = Py::new(py, PySparseGraph { inner: sub })?;
        Ok(obj.into_py(py))
    }

    fn merge(&self, other: &PySparseGraph, py: Python) -> PyResult<(PyObject, PyObject, PyObject)> {
        let (merged, self_map, other_map) = self
            .inner
            .merge(&other.inner);

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
        let comp = self.inner.complement();
        let obj = Py::new(py, PySparseGraph { inner: comp })?;
        Ok(obj.into_py(py))
    }

    fn join(&self, other: &PySparseGraph, py: Python) -> PyResult<(PyObject, PyObject, PyObject)> {
        let (joined, self_map, other_map) = self.inner.join(&other.inner);
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
        let (product, map) = self.inner.product(&other.inner);
        let product_obj = Py::new(py, PySparseGraph { inner: product })?.into_py(py);
        let dict_map = PyDict::new_bound(py);
        for ((a,b), v) in map {
            dict_map.set_item((a,b), v)?;
        }
        Ok((product_obj, dict_map.into_py(py)))
    }

    fn is_connected(&self) -> bool {
        connectivity::is_connected(&self.inner)
    }

    fn cut_vertices(&self) -> Vec<usize> {
        connectivity::cut_vertices(&self.inner).into_iter().collect()
    }

    fn bridges(&self) -> Vec<(usize, usize)> {
        connectivity::bridges(&self.inner).into_iter().collect()
    }

    fn edge_connectivity(&self) -> u32 {
        connectivity::edge_connectivity(&self.inner)
    }

    fn vertex_connectivity(&self) -> u32 {
        connectivity::vertex_connectivity(&self.inner)
    }

    fn is_complete(&self) -> bool {
        connectivity::simple_graph_is_complete(&self.inner)
    }

    fn maximum_matching(&self) -> Vec<(usize, usize)> {
        let matching = maximum_matching(&self.inner);
        matching.edges().collect()
    }

    fn dsatur_coloring(&self) -> Vec<Vec<usize>> {
        let coloring = dsatur(&self.inner);
        coloring.into_iter()
            .map(|set| set.iter().map(|v| v.into_owned()).collect::<Vec<usize>>())
            .collect()
    }

    /// Returns the chromatic number via backtracking with an upper bound.
    fn chromatic_number_upper(&self, upper_bound: usize) -> PyResult<Vec<Vec<usize>>> {
        chromatic_number_upper_bound(&self.inner, upper_bound)
            .map(|coloring| coloring.into_iter()
                .map(|set| set.iter().map(|v| v.into_owned()).collect::<Vec<usize>>())
                .collect())
            .map_err(|_| PyValueError::new_err("Could not find chromatic number within bounds"))
    }

    fn chromatic_number_lower_bound(&self, lower_bound: usize) -> Vec<Vec<usize>> {
        chromatic_number_lower_bound(&self.inner, lower_bound)
            .into_iter()
            .map(|set| set.iter().map(|v| v.into_owned()).collect::<Vec<usize>>())
            .collect::<Vec<Vec<usize>>>()
    }

    fn chromatic_number_bounded(&self, lower_bound: usize, upper_bound: usize) -> PyResult<Vec<Vec<usize>>> {
        chromatic_number_bounded(&self.inner, lower_bound, upper_bound)
            .map(|coloring| coloring.into_iter()
                .map(|set| set.iter().map(|v| v.into_owned()).collect::<Vec<usize>>())
                .collect())
            .map_err(|_| PyValueError::new_err("Could not find chromatic number within bounds"))
    }

    fn clique_number(&self) -> usize {
        clique_number(&self.inner)
    }

    fn gonality(&self) -> PyResult<usize> {
        compute_gonality(&self.inner).map_err(graph_error_to_py)
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
        if !self.inner.has_vertex(v) {
            return Err(PyValueError::new_err(format!("Vertex {} not in graph", v)));
        }

        Ok(self.inner.neighbors(v).iter().map(|c| *c).collect())
    }

    fn out_neighbors(&self, v: usize) -> PyResult<Vec<usize>> {
        if !self.inner.has_vertex(v) {
            return Err(PyValueError::new_err(format!("Vertex {} not in graph", v)));
        }

        Ok(self.inner.out_neighbors(v).iter().map(|c| *c).collect())
    }

    fn in_neighbors(&self, v: usize) -> PyResult<Vec<usize>> {
        if !self.inner.has_vertex(v) {
            return Err(PyValueError::new_err(format!("Vertex {} not in graph", v)));
        }

        Ok(self.inner.in_neighbors(v).iter().map(|c| *c).collect())
    }

    fn to_simple(&self, py: Python) -> PyResult<PyObject> {
        let view = self.inner.as_underlying();

        let mut simple = SparseSimpleGraph::default();
        for v in view.vertices() {
            simple.add_vertex(v);
        }
        for (u, v) in view.edges() {
            simple.add_edge((u, v));
        }

        let obj = Py::new(py, PySparseGraph { inner: simple })?;
        Ok(obj.into_py(py))
    }

    fn dijkstra(&self, source: usize) -> PyResult<(HashMap<usize, f64>, HashMap<usize, usize>)> {
        self.inner.dijkstra(source).map_err(graph_error_to_py)
    }

    fn astar(&self, source: usize, target: usize) -> PyResult<(Vec<usize>, f64)> {
        self.inner.astar(source, target).map_err(graph_error_to_py)
    }

    fn is_weakly_connected(&self) -> bool {
        connectivity::is_weakly_connected(&self.inner)
    }

    fn is_strongly_connected(&self) -> bool {
        connectivity::is_strongly_connected(&self.inner)
    }

    fn strongly_connected_components(&self) -> Vec<Vec<usize>> {
        connectivity::strongly_connected_components(&self.inner)
            .into_iter()
            .map(|scc| scc.into_iter().collect())
            .collect()
    }

    fn is_complete(&self) -> bool {
        connectivity::digraph_is_complete(&self.inner)
    }
}

#[pyclass(name = "LabeledSimpleGraph")]
#[derive(Debug)]
pub struct PyLabeledSimpleGraph {
    inner: HashMapLabeledSimpleGraph<SparseSimpleGraph, PyObject, PyObject>,
}

#[pymethods]
impl PyLabeledSimpleGraph {
    #[new]
    fn new() -> Self {
        Self {
            inner: HashMapLabeledSimpleGraph::default(),
        }
    }

    fn add_vertex(&mut self, v: usize) {
        self.inner.add_vertex(v);
    }

    fn add_edge(&mut self, u: usize, v: usize) {
        self.inner.add_edge((u, v));
    }

    fn vertices(&self) -> Vec<usize> {
        self.inner.vertices().collect()
    }

    fn edges(&self) -> Vec<(usize, usize)> {
        self.inner.edges().collect()
    }

    fn set_vertex_label(&mut self, py: Python, v: usize, label: PyObject) -> PyResult<()> {
        if !self.inner.has_vertex(v) {
            return Err(PyValueError::new_err(format!("Vertex {} not in graph", v)));
        }
        self.inner.set_vertex_label(v, label.clone_ref(py));
        Ok(())
    }

    fn get_vertex_label(&self, py: Python, v: usize) -> PyResult<Option<PyObject>> {
        Ok(self.inner.get_vertex_label(v).map(|obj| obj.clone_ref(py)))
    }

    fn set_edge_label(&mut self, py: Python, u: usize, v: usize, label: PyObject) -> PyResult<()> {
        if !self.inner.has_edge((u, v)) {
            return Err(PyValueError::new_err(format!("Edge ({},{}) not in graph", u, v)));
        }
        self.inner.set_edge_label((u, v), label.clone_ref(py));
        Ok(())
    }

    fn get_edge_label(&self, py: Python, u: usize, v: usize) -> PyResult<Option<PyObject>> {
        Ok(self.inner.get_edge_label((u, v)).map(|obj| obj.clone_ref(py)))
    }

    fn vertex_labels(&self, py: Python) -> PyResult<PyObject> {
        let dict = PyDict::new_bound(py);
        for (v, label) in self.inner.vertex_labels() {
            dict.set_item(v, label.clone_ref(py))?;
        }
        Ok(dict.into_py(py))
    }

    fn edge_labels(&self, py: Python) -> PyResult<PyObject> {
        let dict = PyDict::new_bound(py);
        for ((u, v), label) in self.inner.edge_labels() {
            dict.set_item((u, v), label.clone_ref(py))?;
        }
        Ok(dict.into_py(py))
    }

    fn remove_vertex_label(&mut self, py: Python, v: usize) -> PyResult<Option<PyObject>> {
        Ok(self.inner.remove_vertex_label(v).map(|obj| obj.clone_ref(py)))
    }

    fn remove_edge_label(&mut self, py: Python, u: usize, v: usize) -> PyResult<Option<PyObject>> {
        Ok(self.inner.remove_edge_label((u, v)).map(|obj| obj.clone_ref(py)))
    }

    fn dijkstra(&self, source: usize) -> PyResult<(HashMap<usize, f64>, HashMap<usize, usize>)> {
        self.inner.dijkstra(source).map_err(graph_error_to_py)
    }

    fn astar(&self, source: usize, target: usize) -> PyResult<(Vec<usize>, f64)> {
        self.inner.astar(source, target).map_err(graph_error_to_py)
    }

    fn kruskal(&self, py: Python) -> PyResult<(Vec<(usize, usize)>, f64)> {
        let mst = kruskal_mst(&self.inner, |g, e| {
            g.get_edge_label(e)
                .and_then(|obj| obj.extract::<f64>(py).ok())
        })
        .map_err(graph_error_to_py)?;

        let total: f64 = mst.iter().map(|(_, _, w)| *w).sum();
        let edges = mst.into_iter().map(|(u, v, _)| (u, v)).collect();

        Ok((edges, total))
    }

    fn is_connected(&self) -> bool {
        connectivity::is_connected(&self.inner)
    }

    fn cut_vertices(&self) -> Vec<usize> {
        connectivity::cut_vertices(&self.inner).into_iter().collect()
    }

    fn bridges(&self) -> Vec<(usize, usize)> {
        connectivity::bridges(&self.inner).into_iter().collect()
    }

    fn edge_connectivity(&self) -> u32 {
        connectivity::edge_connectivity(&self.inner)
    }

    fn vertex_connectivity(&self) -> u32 {
        connectivity::vertex_connectivity(&self.inner)
    }

    fn is_complete(&self) -> bool {
        connectivity::simple_graph_is_complete(&self.inner)
    }

    fn maximum_matching(&self) -> Vec<(usize, usize)> {
        let matching = maximum_matching(&self.inner);
        matching.edges().collect()
    }

    fn dsatur_coloring(&self) -> Vec<Vec<usize>> {
        let coloring = dsatur(&self.inner);
        coloring.into_iter()
            .map(|set| set.iter().map(|v| v.into_owned()).collect::<Vec<usize>>())
            .collect()
    }

    /// Returns the chromatic number via backtracking with an upper bound.
    fn chromatic_number_upper(&self, upper_bound: usize) -> PyResult<Vec<Vec<usize>>> {
        chromatic_number_upper_bound(&self.inner, upper_bound)
            .map(|coloring| coloring.into_iter()
                .map(|set| set.iter().map(|v| v.into_owned()).collect::<Vec<usize>>())
                .collect())
            .map_err(|_| PyValueError::new_err("Could not find chromatic number within bounds"))
    }

    fn chromatic_number_lower_bound(&self, lower_bound: usize) -> Vec<Vec<usize>> {
        chromatic_number_lower_bound(&self.inner, lower_bound)
            .into_iter()
            .map(|set| set.iter().map(|v| v.into_owned()).collect::<Vec<usize>>())
            .collect::<Vec<Vec<usize>>>()
    }

    fn chromatic_number_bounded(&self, lower_bound: usize, upper_bound: usize) -> PyResult<Vec<Vec<usize>>> {
        chromatic_number_bounded(&self.inner, lower_bound, upper_bound)
            .map(|coloring| coloring.into_iter()
                .map(|set| set.iter().map(|v| v.into_owned()).collect::<Vec<usize>>())
                .collect())
            .map_err(|_| PyValueError::new_err("Could not find chromatic number within bounds"))
    }

    fn clique_number(&self) -> usize {
        clique_number(&self.inner)
    }

    fn gonality(&self) -> PyResult<usize> {
        compute_gonality(&self.inner).map_err(graph_error_to_py)
    }
}

#[pyclass(name = "LabeledDiGraph")]
#[derive(Debug)]
pub struct PyLabeledDiGraph {
    inner: HashMapLabeledDiGraph<SparseDiGraph, PyObject, PyObject>,
}

#[pymethods]
impl PyLabeledDiGraph {
    #[new]
    fn new() -> Self {
        Self {
            inner: HashMapLabeledDiGraph::default(),
        }
    }

    fn add_vertex(&mut self, v: usize) {
        self.inner.add_vertex(v);
    }

    fn add_edge(&mut self, u: usize, v: usize) {
        self.inner.add_edge((u, v));
    }

    fn vertices(&self) -> Vec<usize> {
        self.inner.vertices().collect()
    }

    fn edges(&self) -> Vec<(usize, usize)> {
        self.inner.edges().collect()
    }

    fn set_vertex_label(&mut self, py: Python, v: usize, label: PyObject) -> PyResult<()> {
        if !self.inner.has_vertex(v) {
            return Err(PyValueError::new_err(format!("Vertex {} not in graph", v)));
        }
        self.inner.set_vertex_label(v, label.clone_ref(py));
        Ok(())
    }

    fn get_vertex_label(&self, py: Python, v: usize) -> PyResult<Option<PyObject>> {
        Ok(self.inner.get_vertex_label(v).map(|obj| obj.clone_ref(py)))
    }

    fn set_edge_label(&mut self, py: Python, u: usize, v: usize, label: PyObject) -> PyResult<()> {
        if !self.inner.has_edge((u, v)) {
            return Err(PyValueError::new_err(format!("Edge ({},{}) not in graph", u, v)));
        }
        self.inner.set_edge_label((u, v), label.clone_ref(py));
        Ok(())
    }

    fn get_edge_label(&self, py: Python, u: usize, v: usize) -> PyResult<Option<PyObject>> {
        Ok(self.inner.get_edge_label((u, v)).map(|obj| obj.clone_ref(py)))
    }

    fn vertex_labels(&self, py: Python) -> PyResult<PyObject> {
        let dict = PyDict::new_bound(py);
        for (v, label) in self.inner.vertex_labels() {
            dict.set_item(v, label.clone_ref(py))?;
        }
        Ok(dict.into_py(py))
    }

    fn edge_labels(&self, py: Python) -> PyResult<PyObject> {
        let dict = PyDict::new_bound(py);
        for ((u, v), label) in self.inner.edge_labels() {
            dict.set_item((u, v), label.clone_ref(py))?;
        }
        Ok(dict.into_py(py))
    }

    fn remove_vertex_label(&mut self, py: Python, v: usize) -> PyResult<Option<PyObject>> {
        Ok(self.inner.remove_vertex_label(v).map(|obj| obj.clone_ref(py)))
    }

    fn remove_edge_label(&mut self, py: Python, u: usize, v: usize) -> PyResult<Option<PyObject>> {
        Ok(self.inner.remove_edge_label((u, v)).map(|obj| obj.clone_ref(py)))
    }

    fn dijkstra(&self, source: usize) -> PyResult<(HashMap<usize, f64>, HashMap<usize, usize>)> {
        self.inner.dijkstra(source).map_err(graph_error_to_py)
    }

    fn astar(&self, source: usize, target: usize) -> PyResult<(Vec<usize>, f64)> {
        self.inner.astar(source, target).map_err(graph_error_to_py)
    }

    fn is_weakly_connected(&self) -> bool {
        connectivity::is_weakly_connected(&self.inner.graph)
    }

    fn is_strongly_connected(&self) -> bool {
        connectivity::is_strongly_connected(&self.inner)
    }

    fn strongly_connected_components(&self) -> Vec<Vec<usize>> {
        connectivity::strongly_connected_components(&self.inner)
            .into_iter()
            .map(|scc| scc.into_iter().collect())
            .collect()
    }

    fn is_complete(&self) -> bool {
        connectivity::digraph_is_complete(&self.inner)
    }
}

#[pyfunction]
fn simple_complete_graph(size: usize) -> PySparseGraph {
    PySparseGraph {
        inner: build_complete_graph::<SparseSimpleGraph>(size),
    }
}

#[pyfunction]
fn simple_cycle_graph(size: usize) -> PySparseGraph {
    PySparseGraph {
        inner: build_cycle::<SparseSimpleGraph>(size),
    }
}

#[pyfunction]
fn simple_path_graph(size: usize) -> PySparseGraph {
    PySparseGraph {
        inner: build_path::<SparseSimpleGraph>(size),
    }
}

#[pyfunction]
fn simple_binary_tree(layers: usize) -> PySparseGraph {
    PySparseGraph {
        inner: build_binary_tree::<SparseSimpleGraph>(layers),
    }
}

#[pyfunction]
fn simple_bowtie_graph() -> PySparseGraph {
    PySparseGraph {
        inner: build_bowtie::<SparseSimpleGraph>(),
    }
}

#[pyfunction]
fn simple_partite_graph(groups: Vec<usize>) -> PySparseGraph {
    PySparseGraph {
        inner: build_partite_graph::<SparseSimpleGraph>(groups),
    }
}

#[pyfunction]
fn directional_complete_graph(size: usize) -> PySparseDiGraph {
    PySparseDiGraph {
        inner: build_complete_graph::<SparseDiGraph>(size),
    }
}

#[pyfunction]
fn directional_cycle_graph(size: usize) -> PySparseDiGraph {
    PySparseDiGraph {
        inner: build_cycle::<SparseDiGraph>(size),
    }
}

#[pyfunction]
fn directional_path_graph(size: usize) -> PySparseDiGraph {
    PySparseDiGraph {
        inner: build_path::<SparseDiGraph>(size),
    }
}

#[pyfunction]
fn directional_binary_tree(layers: usize) -> PySparseDiGraph {
    PySparseDiGraph {
        inner: build_binary_tree::<SparseDiGraph>(layers),
    }
}

#[pyfunction]
fn directional_bowtie_graph() -> PySparseDiGraph {
    PySparseDiGraph {
        inner: build_bowtie::<SparseDiGraph>(),
    }
}

#[pyfunction]
fn directional_partite_graph(groups: Vec<usize>) -> PySparseDiGraph {
    PySparseDiGraph {
        inner: build_partite_graph::<SparseDiGraph>(groups),
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
    m.add_class::<PyLabeledSimpleGraph>()?;
    m.add_class::<PyLabeledDiGraph>()?;
    
    m.add_function(wrap_pyfunction!(simple_complete_graph, m)?)?;
    m.add_function(wrap_pyfunction!(simple_cycle_graph, m)?)?;
    m.add_function(wrap_pyfunction!(simple_path_graph, m)?)?;
    m.add_function(wrap_pyfunction!(simple_binary_tree, m)?)?;
    m.add_function(wrap_pyfunction!(simple_bowtie_graph, m)?)?;
    m.add_function(wrap_pyfunction!(simple_partite_graph, m)?)?;

    m.add_function(wrap_pyfunction!(directional_complete_graph, m)?)?;
    m.add_function(wrap_pyfunction!(directional_cycle_graph, m)?)?;
    m.add_function(wrap_pyfunction!(directional_path_graph, m)?)?;
    m.add_function(wrap_pyfunction!(directional_binary_tree, m)?)?;
    m.add_function(wrap_pyfunction!(directional_bowtie_graph, m)?)?;
    m.add_function(wrap_pyfunction!(directional_partite_graph, m)?)?;

    Ok(())
}