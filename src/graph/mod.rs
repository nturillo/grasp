pub mod error;
pub mod labeled_graph;
pub mod adjacency_list;

pub mod prelude{
    pub use super::{*, labeled_graph::{LabeledGraph, HashMapLabeledGraph}, adjacency_list::{SparseSimpleGraph, SparseDiGraph}, error::GraphError};
}

use std::{borrow::Cow, collections::{HashMap, HashSet}};

pub trait Set<V>: Clone+FromIterator<V>+PartialEq {
    fn contains(&self, v: V) -> bool;
    fn count(&self) -> usize;
    fn union(&self, other: &Self) -> Self;
    fn intersection(&self, other: &Self) -> Self;
    fn difference(&self, other: &Self) -> Self;
    fn iter<'a>(&'a self) -> impl Iterator<Item=&'a V> where V: 'a;
    fn into_iter(self) -> impl Iterator<Item=V>;
}
impl<V: Clone+Eq+std::hash::Hash> Set<V> for HashSet<V>{
    fn contains(&self, v: V) -> bool {
        HashSet::contains(self, &v)
    }
    fn count(&self) -> usize {
        self.len()
    }
    fn union(&self, other: &Self) -> Self {
        self | other
    }
    fn intersection(&self, other: &Self) -> Self {
        self & other
    }
    fn difference(&self, other: &Self) -> Self {
        HashSet::difference(self, other).cloned().collect()
    }
    fn iter<'a>(&'a self) -> impl Iterator<Item=&'a V> where V: 'a {
        HashSet::iter(self)
    }
    fn into_iter(self) -> impl Iterator<Item=V> {
        IntoIterator::into_iter(self)
    }
}

pub type VertexID = usize;
pub type EdgeID = (VertexID, VertexID);
pub type VertexMap = HashMap<VertexID, VertexID>;

/// Core Graph functionality. Enables edge and vertex manipulation
pub trait GraphTrait{
    type VertexSet: Set<VertexID>;
    
    fn vertex_count(&self) -> usize;
    fn edge_count(&self) -> usize;
    fn vertices(&self) -> impl Iterator<Item=VertexID>;
    fn edges(&self) -> impl Iterator<Item=EdgeID>;
    fn contains(&self, v: VertexID) -> bool;
    fn has_edge(&self, e: EdgeID) -> bool;
    fn neighbors(&self, v: VertexID) -> Option<Cow<'_, Self::VertexSet>>;
    fn vertex_set(&self) -> Self::VertexSet;

    fn create_vertex(&mut self) -> VertexID;
    fn add_vertex(&mut self, v: VertexID);
    /// adds v1 and v2 if they don't exist
    /// Also adding (a,b) should also add (b,a) without double the edges. this is not a directed graph
    fn add_edge(&mut self, e: EdgeID);
    /// adds v and nbhrs if they don't exist
    fn add_neighbors(&mut self, v: VertexID, nbhrs: impl Iterator<Item=VertexID>);

    /// Returns a list of edges it removed as a consequence
    fn delete_vertex(&mut self, v: VertexID) -> impl Iterator<Item=EdgeID>;
    fn delete_edge(&mut self, e: EdgeID);
}

/// Tag Trait Used to represent the promise that edge ab~ba
pub trait SimpleGraph: GraphTrait{}
/// Trait Used to represent the promise that edge ab!~ba
pub trait DiGraph: GraphTrait{
    type UnderlyingGraph: GraphTrait+SimpleGraph;
    fn in_neighbors(&self, v: VertexID) -> Option<Cow<'_, Self::VertexSet>>;
    fn out_neighbors(&self, v: VertexID) -> Option<Cow<'_, Self::VertexSet>>;
    fn underlying_graph(&self) -> Self::UnderlyingGraph;
}

/// Graph operations that are agnostic to simple graphs and digraphs
pub trait GraphOps: GraphTrait+Sized{
    fn subgraph_vertex(&self, vertices: impl IntoIterator<Item=VertexID>, graph_builder: impl FnOnce() -> Self) -> Self {
        let mut subgraph = graph_builder();
        for vertex in vertices{
            subgraph.add_vertex(vertex);
        }
        for (v1, v2) in self.edges(){
            if subgraph.contains(v1) && subgraph.contains(v2) {
                subgraph.add_edge((v1, v2));
            }
        }
        subgraph
    }

    fn subgraph_edges(&self, edges: impl IntoIterator<Item=EdgeID>, graph_builder: impl FnOnce() -> Self) -> Self {
        let mut subgraph = graph_builder();
        for edge in edges{
            if !self.has_edge(edge) {continue;}
            subgraph.add_edge(edge);
        }
        subgraph
    }

    fn merge(&self, other: &Self, graph_builder: impl FnOnce() -> Self) -> (Self, VertexMap, VertexMap) {
        let mut self_map = HashMap::default();
        let mut other_map = HashMap::default();
        let mut merged = graph_builder();
        // vertices
        for v in self.vertices() {
            let new_vertex = merged.create_vertex();
            self_map.insert(v, new_vertex);
        }
        for v in other.vertices() {
            let new_vertex = merged.create_vertex();
            other_map.insert(v, new_vertex);
        }
        // edges
        for (v1, v2) in self.edges() {
            let Some(v1) = self_map.get(&v1) else {continue;};
            let Some(v2) = self_map.get(&v2) else {continue;};
            merged.add_edge((*v1, *v2));
        }
        for (v1, v2) in other.edges() {
            let Some(v1) = other_map.get(&v1) else {continue;};
            let Some(v2) = other_map.get(&v2) else {continue;};
            merged.add_edge((*v1, *v2));
        }
        (merged, self_map, other_map)
    }

    fn complement(&self, graph_builder: impl FnOnce() -> Self) -> Self {
        let mut complement = graph_builder();
        for v1 in self.vertices(){
            complement.add_vertex(v1);
            for v2 in self.vertices(){
                if v1==v2 {continue;} // Complement ignores loops
                if self.has_edge((v1, v2)){continue;}
                complement.add_edge((v1, v2));
            }
        }
        complement
    }
}

/// Graph operations that only work for simple graphs
pub trait SimpleGraphOps: GraphOps+SimpleGraph{
    fn join(&self, other: &Self, graph_builder: impl FnOnce() -> Self) -> (Self, VertexMap, VertexMap) {
        let (mut joined, self_map, other_map) = self.merge(other, graph_builder);
        for v1 in self.vertices(){
            let Some(v1) = self_map.get(&v1) else {continue;};
            for v2 in other.vertices(){
                let Some(v2) = other_map.get(&v2) else {continue;};
                joined.add_edge((*v1, *v2));
            }
        }
        (joined, self_map, other_map)
    }

    fn product(&self, other: &Self, graph_builder: impl FnOnce() -> Self) -> (Self, HashMap<(VertexID, VertexID), VertexID>) {
        let mut map = HashMap::default();
        let mut product = graph_builder();
        // Vertices and map
        for v1 in self.vertices(){
            for v2 in other.vertices(){
                let v = product.create_vertex();
                map.insert((v1, v2), v);
            }
        }
        // edges part 1
        for (s1, s2) in self.edges(){
            for o in other.vertices(){
                let Some(v1) = map.get(&(s1, o)) else {continue;};
                let Some(v2) = map.get(&(s2, o)) else {continue;};
                product.add_edge((*v1, *v2));
            }
        }
        // edges part 2
        for (o1, o2) in other.edges(){
            for s in self.vertices(){
                let Some(v1) = map.get(&(s, o1)) else {continue;};
                let Some(v2) = map.get(&(s, o2)) else {continue;};
                product.add_edge((*v1, *v2));
            }
        }
        (product, map)
    }
}

/// Contains test templates for GraphOps and SimpleGraphOps, should be used as a seperate test for each graph implementation
#[cfg(test)]
mod test{
    use crate::graph::{DiGraph, GraphOps, GraphTrait, SimpleGraph, SimpleGraphOps};

    pub fn graphs_eq<G: GraphTrait>(graph_a: &G, graph_b: &G) -> bool{
        for vertex in graph_a.vertices(){if !graph_b.contains(vertex) {return false;}}
        for vertex in graph_b.vertices(){if !graph_a.contains(vertex) {return false;}}
        for edge in graph_a.edges(){if !graph_b.has_edge(edge) {return false;}}
        for edge in graph_b.edges(){if !graph_a.has_edge(edge) {return false;}}
        true
    }

    /// Assures SimpleGraph and DiGraph traits work as intended.
    pub fn graph_vs_digraph_test<S: SimpleGraph+Default, D: DiGraph+Default>(){
        let mut simple_graph = S::default();
        let mut digraph = D::default();
        simple_graph.add_edge((0, 1));
        digraph.add_edge((0, 1));
        assert!(simple_graph.has_edge((0, 1)));
        assert!(digraph.has_edge((0, 1)));
        assert!(simple_graph.has_edge((1, 0)));
        assert!(!digraph.has_edge((1, 0)));
    }

    /// Assures Digraph functionality.
    pub fn digraph_fn_test<G: DiGraph+Default>(){
        let mut digraph = G::default();
        digraph.add_edge((0, 1)); digraph.add_edge((2, 0));
        // neighborhoods
        let neighbors = G::VertexSet::from_iter([1, 2]);
        let out_neighbors = G::VertexSet::from_iter([1]);
        let in_neighbors = G::VertexSet::from_iter([2]);
        assert!(digraph.neighbors(0).is_some_and(|s| *s==neighbors));
        assert!(digraph.out_neighbors(0).is_some_and(|s| *s==out_neighbors));
        assert!(digraph.in_neighbors(0).is_some_and(|s| *s==in_neighbors));
        // underlying graph
        let und_graph = digraph.underlying_graph();
        assert!(und_graph.has_edge((0, 1)));
        assert!(und_graph.has_edge((0, 2)));
        assert_eq!(und_graph.edge_count(), 2);
    }

    /// Assures Graph Ops functionality
    pub fn graph_ops_test<G: GraphOps+Default>(){
        let mut graph_a =  G::default();
        graph_a.add_edge((0, 1)); graph_a.add_edge((1, 2)); graph_a.add_edge((2, 0));
        // ensure subgraphs work
        let subgraph_vertices_a = graph_a.subgraph_vertex([0, 1], G::default);
        let subgraph_edges_a = graph_a.subgraph_edges([(0, 1)], G::default);
        let mut test_subgraph = G::default(); test_subgraph.add_edge((0, 1));
        assert!(graphs_eq(&subgraph_edges_a, &test_subgraph));
        assert!(graphs_eq(&subgraph_vertices_a, &test_subgraph));
        // create merged graph manually and test to ensure equality
        let (merged, map_a, map_b) = graph_a.merge(&graph_a, G::default);
        let mut test_graph = G::default();
        test_graph.add_edge((*map_a.get(&0).unwrap(), *map_a.get(&1).unwrap()));
        test_graph.add_edge((*map_a.get(&1).unwrap(), *map_a.get(&2).unwrap()));
        test_graph.add_edge((*map_a.get(&2).unwrap(), *map_a.get(&0).unwrap()));
        test_graph.add_edge((*map_b.get(&0).unwrap(), *map_b.get(&1).unwrap()));
        test_graph.add_edge((*map_b.get(&1).unwrap(), *map_b.get(&2).unwrap()));
        test_graph.add_edge((*map_b.get(&2).unwrap(), *map_b.get(&0).unwrap()));
        assert!(graphs_eq(&merged, &test_graph));
    }

    pub fn simple_graph_complement_test<G: GraphOps+Default>(){
        // Complement
        let mut graph = G::default();
        graph.add_edge((0, 1)); graph.add_edge((1, 2));
        let complement = graph.complement(G::default);
        let mut test_complement = G::default();
        test_complement.add_edge((0, 2)); test_complement.add_vertex(1);
        assert!(graphs_eq(&complement, &test_complement));
    }

    pub fn digraph_complement_test<G: GraphOps+Default>(){
        // Complement
        let mut graph = G::default();
        graph.add_edge((0, 1)); graph.add_edge((1, 2));
        let complement = graph.complement(G::default);
        let mut test_complement = G::default();
        test_complement.add_edge((1, 0)); test_complement.add_edge((2, 1));
        test_complement.add_edge((0, 2)); test_complement.add_edge((2, 0));
        assert!(graphs_eq(&complement, &test_complement));
    }

    /// Assures SimpleGraphs Ops (Join, Product, Complement) work
    pub fn simple_graph_ops_test<G: SimpleGraphOps+Default>(){
        let mut line = G::default();
        line.add_vertex(0); line.add_vertex(1);
        // Join
        let (join, map_a, map_b) = line.join(&line, G::default);
        let mut test_join = G::default();
        test_join.add_edge((*map_a.get(&0).unwrap(), *map_b.get(&0).unwrap()));
        test_join.add_edge((*map_a.get(&0).unwrap(), *map_b.get(&1).unwrap()));
        test_join.add_edge((*map_a.get(&1).unwrap(), *map_b.get(&0).unwrap()));
        test_join.add_edge((*map_a.get(&1).unwrap(), *map_b.get(&1).unwrap()));
        assert!(graphs_eq(&join, &test_join));
        // Product
        line.add_edge((0, 1));
        let (square, map) = line.product(&line, G::default);
        let mut test_square = G::default();
        test_square.add_edge((*map.get(&(0, 0)).unwrap(), *map.get(&(0, 1)).unwrap()));
        test_square.add_edge((*map.get(&(1, 0)).unwrap(), *map.get(&(1, 1)).unwrap()));
        test_square.add_edge((*map.get(&(0, 0)).unwrap(), *map.get(&(1, 0)).unwrap()));
        test_square.add_edge((*map.get(&(0, 1)).unwrap(), *map.get(&(1, 1)).unwrap()));
        assert!(graphs_eq(&square, &test_square));
    }
}
