pub mod error;
pub mod labeled_graph;
pub mod adjacency_list;

pub mod prelude{
    pub use super::{*, labeled_graph::LabeledGraph, adjacency_list::SparseSimpleGraph, error::GraphError};
}

use std::{borrow::Cow, collections::{HashMap, HashSet}};

pub trait Set<V>: Clone {
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
pub trait Graph: Default{
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
pub trait SimpleGraph: Graph{}
/// Trait Used to represent the promise that edge ab!~ba
pub trait DiGraph: Graph{
    type UnderlyingGraph: Graph+SimpleGraph;
    fn in_neighbors(&self, v: VertexID) -> Option<Cow<'_, Self::VertexSet>>;
    fn out_neighbors(&self, v: VertexID) -> Option<Cow<'_, Self::VertexSet>>;
    fn underlying_graph(&self) -> Self::UnderlyingGraph;
}

/// Graph operations that are agnostic to simple graphs and digraphs
pub trait GraphOps: Graph{
    fn subgraph_vertex(&self, vertices: impl Iterator<Item=VertexID>) -> Self {
        let mut new_graph = Self::default();
        for vertex in vertices{
            let Some(neighbors) = self.neighbors(vertex) else {continue;};
            new_graph.add_vertex(vertex);
            for neighbor in neighbors.as_ref().iter(){
                new_graph.add_edge((vertex, *neighbor));
            }
        }
        new_graph
    }

    fn subgraph_edges(&self, edges: impl Iterator<Item=EdgeID>) -> Self {
        let mut new_graph = Self::default();
        for edge in edges{
            if !self.has_edge(edge) {continue;}
            new_graph.add_edge(edge);
        }
        new_graph
    }

    fn merge(&self, other: &Self) -> (Self, VertexMap, VertexMap) {
        let mut self_map = HashMap::default();
        let mut other_map = HashMap::default();
        let mut new_graph = Self::default();
        // vertices
        for v in self.vertices() {
            let new_vertex = new_graph.create_vertex();
            self_map.insert(v, new_vertex);
        }
        for v in other.vertices() {
            let new_vertex = new_graph.create_vertex();
            other_map.insert(v, new_vertex);
        }
        // edges
        for (v1, v2) in self.edges() {
            let Some(v1) = self_map.get(&v1) else {continue;};
            let Some(v2) = self_map.get(&v2) else {continue;};
            new_graph.add_edge((*v1, *v2));
        }
        for (v1, v2) in other.edges() {
            let Some(v1) = other_map.get(&v1) else {continue;};
            let Some(v2) = other_map.get(&v2) else {continue;};
            new_graph.add_edge((*v1, *v2));
        }
        (new_graph, self_map, other_map)
    }
}

/// Graph operations that only work for simple graphs
pub trait SimpleGraphOps: GraphOps+SimpleGraph{
    fn join(&self, other: &Self) -> (Self, VertexMap, VertexMap) {
        let (mut merged, self_map, other_map) = self.merge(other);
        for v1 in self.vertices(){
            let Some(v1) = self_map.get(&v1) else {continue;};
            for v2 in other.vertices(){
                let Some(v2) = other_map.get(&v2) else {continue;};
                merged.add_edge((*v1, *v2));
            }
        }
        (merged, self_map, other_map)
    }

    fn product(&self, other: &Self) -> (Self, HashMap<(VertexID, VertexID), VertexID>) {
        let mut map = HashMap::default();
        let mut product = Self::default();
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

    fn complement(&self) -> Self {
        let mut complement = Self::default();
        let vertex_set = self.vertex_set();
        for v in self.vertices(){
            complement.add_vertex(v);
            let neighbors = self.neighbors(v).unwrap();
            complement.add_neighbors(v, vertex_set.difference(&neighbors).into_iter());
        }
        complement
    }
}

// TODO: Tests for graph manipulation functions
