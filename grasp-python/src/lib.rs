use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::exceptions::PyValueError;

use grasp::graph::prelude::*;
use grasp::graph::labeled_graph::{HashMapLabeledSimpleGraph, HashMapLabeledDiGraph, LabeledGraph, LabeledGraphMut};
use grasp::algorithms::algo_traits::{AlgoTrait};
use grasp::algorithms::search::{ShortestPath};
use grasp::algorithms::connectivity;

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
        self.inner.vertex_set().iter().cloned().collect()
    }

    fn neighbors(&self, v: usize) -> PyResult<Vec<usize>> {
        if !self.inner.has_vertex(v) {
            return Err(PyValueError::new_err(format!("Vertex {} not in graph", v)));
        }

        Ok(self.inner.neighbors(v).iter().cloned().collect())
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

    fn astar(&self, py: Python, source: usize, target: usize, weights: std::collections::HashMap<(usize, usize), f64>, heuristic: PyObject) -> PyResult<(Vec<usize>, f64)> {

        let weight_fn = move |_g: &SparseSimpleGraph, edge: EdgeID| -> Option<f64> {
            weights.get(&edge)
                .copied()
                .or_else(|| weights.get(&(edge.1, edge.0)).copied())
        };

        let heuristic_fn = move |v: usize| -> f64 {
            Python::with_gil(|py| {
                heuristic.call1(py, (v,))
                    .and_then(|res| res.extract::<f64>(py))
                    .unwrap_or(0.0)
            })
        };

        let mut iter = self.inner
            .astar_iter(source, target, weight_fn, heuristic_fn)
            .map_err(graph_error_to_py)?;

        let mut last = None;

        while let Some(res) = iter.next() {
            last = Some(res.map_err(graph_error_to_py)?);
        }

        let (end, cost) = last.ok_or_else(|| PyValueError::new_err("No path found"))?;

        let mut path = vec![end];
        let mut current = end;

        while let Some(&p) = iter.predecessors().get(&current) {
            path.push(p);
            current = p;
        }

        path.reverse();

        Ok((path, cost))
    }

    fn kruskal(&self, py: Python, weights: std::collections::HashMap<(usize, usize), f64>) -> PyResult<(Vec<(usize, usize)>, f64)> {
        let mut parent: std::collections::HashMap<usize, usize> =
            self.inner.vertices().map(|v| (v, v)).collect();

        fn find(parent: &mut std::collections::HashMap<usize, usize>, x: usize) -> usize {
            if parent[&x] != x {
                let root = find(parent, parent[&x]);
                parent.insert(x, root);
            }
            parent[&x]
        }

        fn union(parent: &mut std::collections::HashMap<usize, usize>, a: usize, b: usize) {
            let pa = find(parent, a);
            let pb = find(parent, b);
            if pa != pb {
                parent.insert(pa, pb);
            }
        }

        let mut edges: Vec<_> = self.inner.edges().collect();

        edges.sort_by(|&(u1,v1), &(u2,v2)| {
            let w1 = weights.get(&(u1,v1)).or(weights.get(&(v1,u1))).unwrap_or(&f64::INFINITY);
            let w2 = weights.get(&(u2,v2)).or(weights.get(&(v2,u2))).unwrap_or(&f64::INFINITY);
            w1.partial_cmp(w2).unwrap()
        });

        let mut mst = Vec::new();
        let mut total = 0.0;

        for (u, v) in edges {
            if find(&mut parent, u) != find(&mut parent, v) {
                union(&mut parent, u, v);

                let w = weights.get(&(u,v))
                    .or(weights.get(&(v,u)))
                    .copied()
                    .unwrap_or(0.0);

                mst.push((u,v));
                total += w;
            }
        }

        Ok((mst, total))
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

        Ok(self.inner.neighbors(v).iter().cloned().collect())
    }

    fn out_neighbors(&self, v: usize) -> PyResult<Vec<usize>> {
        if !self.inner.has_vertex(v) {
            return Err(PyValueError::new_err(format!("Vertex {} not in graph", v)));
        }

        Ok(self.inner.out_neighbors(v).iter().cloned().collect())
    }

    fn in_neighbors(&self, v: usize) -> PyResult<Vec<usize>> {
        if !self.inner.has_vertex(v) {
            return Err(PyValueError::new_err(format!("Vertex {} not in graph", v)));
        }

        Ok(self.inner.in_neighbors(v).iter().cloned().collect())
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

    fn dijkstra(&self, py: Python, source: usize) -> PyResult<(PyObject, PyObject)> {
        let weight_fn = |g: &HashMapLabeledSimpleGraph<SparseSimpleGraph, PyObject, PyObject>, e: EdgeID| {
            g.get_edge_label(e).and_then(|obj| obj.extract::<f64>(py).ok())
        };

        let mut iter = self.inner.dijkstra_iter(source, weight_fn).map_err(graph_error_to_py)?;

        let mut dist = std::collections::HashMap::new();
        while let Some(res) = iter.next() {
            let (v, d) = res.map_err(graph_error_to_py)?;
            dist.insert(v, d);
        }

        let mut prev = std::collections::HashMap::new();
        for (k, v) in iter.predecessors() {
            prev.insert(*k, *v);
        }

        let py_dist = PyDict::new_bound(py);
        for (k, v) in dist { py_dist.set_item(k, v)?; }

        let py_prev = PyDict::new_bound(py);
        for (k, v) in prev { py_prev.set_item(k, v)?; }

        Ok((py_dist.into_py(py), py_prev.into_py(py)))
    }

    fn astar(&self, py: Python, source: usize, target: usize, heuristic: PyObject) -> PyResult<(Vec<usize>, f64)> {
        let weight_fn = |g: &HashMapLabeledSimpleGraph<SparseSimpleGraph, PyObject, PyObject>, e: EdgeID| {
            g.get_edge_label(e).and_then(|obj| obj.extract::<f64>(py).ok())
        };

        let heuristic_fn = move |v: usize| -> f64 {
            Python::with_gil(|py| {
                heuristic.call1(py, (v,)).and_then(|r| r.extract::<f64>(py)).unwrap_or(0.0)
            })
        };

        let mut iter = self.inner.astar_iter(source, target, weight_fn, heuristic_fn)
            .map_err(graph_error_to_py)?;

        let mut last = None;
        while let Some(res) = iter.next() {
            last = Some(res.map_err(graph_error_to_py)?);
        }

        let (end, cost) = last.ok_or_else(|| PyValueError::new_err("No path found"))?;

        let mut path = vec![end];
        let mut current = end;

        while let Some(&p) = iter.predecessors().get(&current) {
            path.push(p);
            current = p;
        }

        path.reverse();
        Ok((path, cost))
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

    fn dijkstra(&self, py: Python, source: usize) -> PyResult<(PyObject, PyObject)> {
        let weight_fn = |g: &HashMapLabeledDiGraph<SparseDiGraph, PyObject, PyObject>, e: EdgeID| {
            g.get_edge_label(e).and_then(|obj| obj.extract::<f64>(py).ok())
        };

        let mut iter = self.inner.dijkstra_iter(source, weight_fn).map_err(graph_error_to_py)?;

        let mut dist = std::collections::HashMap::new();
        while let Some(res) = iter.next() {
            let (v, d) = res.map_err(graph_error_to_py)?;
            dist.insert(v, d);
        }

        let mut prev = std::collections::HashMap::new();
        for (k, v) in iter.predecessors() {
            prev.insert(*k, *v);
        }

        let py_dist = PyDict::new_bound(py);
        for (k, v) in dist { py_dist.set_item(k, v)?; }

        let py_prev = PyDict::new_bound(py);
        for (k, v) in prev { py_prev.set_item(k, v)?; }

        Ok((py_dist.into_py(py), py_prev.into_py(py)))
    }

    fn astar(&self, py: Python, source: usize, target: usize, heuristic: PyObject) -> PyResult<(Vec<usize>, f64)> {
        let weight_fn = |g: &HashMapLabeledDiGraph<SparseDiGraph, PyObject, PyObject>, e: EdgeID| {
            g.get_edge_label(e).and_then(|obj| obj.extract::<f64>(py).ok())
        };

        let heuristic_fn = move |v: usize| -> f64 {
            Python::with_gil(|py| {
                heuristic.call1(py, (v,)).and_then(|r| r.extract::<f64>(py)).unwrap_or(0.0)
            })
        };

        let mut iter = self.inner.astar_iter(source, target, weight_fn, heuristic_fn)
            .map_err(graph_error_to_py)?;

        let mut last = None;
        while let Some(res) = iter.next() {
            last = Some(res.map_err(graph_error_to_py)?);
        }

        let (end, cost) = last.ok_or_else(|| PyValueError::new_err("No path found"))?;

        let mut path = vec![end];
        let mut current = end;

        while let Some(&p) = iter.predecessors().get(&current) {
            path.push(p);
            current = p;
        }

        path.reverse();
        Ok((path, cost))
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
    
    Ok(())
}