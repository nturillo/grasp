pub mod error;
pub mod labeled_graph;
pub mod adjacency_list;
pub mod graph_ops;
pub mod util;

pub mod prelude{
    pub use super::{
        Set, VertexID, EdgeID, VertexMap, GraphTrait, SimpleGraph, DiGraph, UnderlyingGraph,
        labeled_graph::{LabeledGraph, HashMapLabeledGraph}, 
        adjacency_list::{SparseSimpleGraph, SparseDiGraph}, 
        error::GraphError,
        graph_ops::*,
        util::*
    };
}

use std::{borrow::Cow, collections::{HashMap, HashSet}};

/// Trait for types that can act as a set of 'V'.
pub trait Set<V>: Clone+FromIterator<V>+PartialEq+IntoIterator<Item = V>{
    fn contains(&self, v: V) -> bool;
    fn count(&self) -> usize;
    fn union(&self, other: &Self) -> Self;
    fn intersection(&self, other: &Self) -> Self;
    fn difference(&self, other: &Self) -> Self;
    fn iter<'a>(&'a self) -> impl Iterator<Item=&'a V> where V: 'a;
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
        HashSet::iter(&self)
    }
}

pub type VertexID = usize;
pub type EdgeID = (VertexID, VertexID);
pub type VertexMap = HashMap<VertexID, VertexID>;

/// Core Graph functionality. Enables edge and vertex manipulation
pub trait GraphTrait : Default{
    type VertexSet: Set<VertexID>;
    
    /// Number of vertices
    fn vertex_count(&self) -> usize;
    /// Number of edges
    fn edge_count(&self) -> usize;
    /// Iterator over vertices
    fn vertices(&self) -> impl Iterator<Item=VertexID>;
    /// Iterator over edges
    fn edges(&self) -> impl Iterator<Item=EdgeID>;
    /// Whether the graph contains the vertex
    fn contains(&self, v: VertexID) -> bool;
    /// Whether the graph contains the edge
    fn has_edge(&self, e: EdgeID) -> bool;
    /// Returns a set of vertices adjacent to given vertex. Returns None if the vertex is not in the graph
    fn neighbors(&self, v: VertexID) -> Option<Cow<'_, Self::VertexSet>>;
    /// Returns a set of all vertices in the graph
    fn vertex_set(&self) -> Self::VertexSet;

    /// Creates a new vertex in the graph and returns its ID
    fn create_vertex(&mut self) -> VertexID;
    /// Adds a vertex to the graph by ID if it is not already in it.
    fn add_vertex(&mut self, v: VertexID);
    /// Adds an edge to the graph \
    /// adds v1 and v2 if they don't exist \
    /// SimpleGraphs should also add edge (v2, v1)
    fn add_edge(&mut self, e: EdgeID);
    /// Creates edges from v1 to the neighbors. \
    /// adds v and nbhrs if they don't exist
    fn add_neighbors(&mut self, v: VertexID, nbhrs: impl Iterator<Item=VertexID>);
    /// Removes a vertex from the graph \
    /// Returns a list of edges it removed as a consequence
    fn delete_vertex(&mut self, v: VertexID) -> impl Iterator<Item=EdgeID>;
    /// Removes an edge from the graph, does not remove the vertices.
    fn delete_edge(&mut self, e: EdgeID);
}

/// Tag Trait Used to represent the promise that edge ab=ba
pub trait SimpleGraph: GraphTrait{}
/// Trait Used to represent the promise that edge ab!=ba
pub trait DiGraph: GraphTrait{
    /// Set of vertices that have arcs going to the specified vertex.
    fn in_neighbors(&self, v: VertexID) -> Option<Cow<'_, Self::VertexSet>>;
    /// Set of vertices that have arcs coming from the specified vertex.
    fn out_neighbors(&self, v: VertexID) -> Option<Cow<'_, Self::VertexSet>>;
}
/// Trait that allows DiGraphs to be converted into SimpleGraphs
pub trait UnderlyingGraph: DiGraph{
    /// SimpleGraph type that this graph can be converted into
    type UnderlyingGraph: GraphTrait+SimpleGraph;
    fn underlying_graph(&self) -> Self::UnderlyingGraph;
}

/// Contains test templates for GraphOps and SimpleGraphOps, should be used as a seperate test for each graph implementation
#[cfg(test)]
mod test{
    use crate::graph::{UnderlyingGraph, prelude::*};

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
    }

    /// Assures Underlying Graphs are correctly calculated
    pub fn underlying_graph_test<G: UnderlyingGraph+Default>(){
        // underlying graph
        let mut digraph = G::default();
        digraph.add_edge((0, 1)); digraph.add_edge((2, 0));
        let und_graph = digraph.underlying_graph();
        assert!(und_graph.has_edge((0, 1)));
        assert!(und_graph.has_edge((0, 2)));
        assert_eq!(und_graph.edge_count(), 2);
    }
}
