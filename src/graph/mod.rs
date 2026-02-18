pub mod error;
pub mod labeled_graph;
pub mod adjacency_list;
pub mod graph_ops;
pub mod util;

pub mod prelude{
    pub use super::{
        Set, VertexID, EdgeID, VertexMap, GraphTrait, SimpleGraph, DiGraph, 
        labeled_graph::{LabeledGraph, HashMapLabeledGraph}, 
        adjacency_list::{SparseSimpleGraph, SparseDiGraph}, 
        error::GraphError,
        graph_ops::*,
        util::*
    };
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

/// Contains test templates for GraphOps and SimpleGraphOps, should be used as a seperate test for each graph implementation
#[cfg(test)]
mod test{
    use crate::graph::prelude::*;

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
}
