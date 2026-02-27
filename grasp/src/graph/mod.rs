pub mod error;
pub mod labeled_graph;
pub mod adjacency_list;
pub mod graph_ops;
pub mod util;
pub mod set;

pub mod prelude{
    pub use super::{
        VertexID, EdgeID, VertexMap, GraphTrait, SimpleGraph, DiGraph, UnderlyingGraph,
        labeled_graph::{LabeledGraph, HashMapLabeledGraph}, 
        adjacency_list::{SparseSimpleGraph, SparseDiGraph}, 
        error::GraphError,
        graph_ops::*,
        util::*, set::{Set, IntoSet}
    };
}

use std::collections::HashMap;
use set::Set;


pub type VertexID = usize;
pub type EdgeID = (VertexID, VertexID);
pub type VertexMap = HashMap<VertexID, VertexID>;

/// Core Graph functionality. Enables edge and vertex manipulation
pub trait GraphTrait{
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
    fn neighbors(&self, v: VertexID) -> Option<impl Set<Item = VertexID>>;
    /// Returns a set of all vertices in the graph
    fn vertex_set(&self) -> impl Set<Item = VertexID>;

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
    fn add_neighbors(&mut self, v: VertexID, nbhrs: impl Iterator<Item=VertexID>){
        for v2 in nbhrs {self.add_edge((v, v2));}
    }
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
    fn in_neighbors(&self, v: VertexID) -> Option<impl Set<Item = VertexID>>;
    /// Set of vertices that have arcs coming from the specified vertex.
    fn out_neighbors(&self, v: VertexID) -> Option<impl Set<Item = VertexID>>;
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
    use std::collections::HashSet;

    use crate::graph::{UnderlyingGraph, prelude::*, set::VertexSet};

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
        let neighbors: VertexSet<_> = HashSet::from_iter([1, 2]).into();
        let out_neighbors: VertexSet<_> = HashSet::from_iter([1]).into();
        let in_neighbors: VertexSet<_> = HashSet::from_iter([2]).into();
        assert!(digraph.neighbors(0).is_some_and(|s| VertexSet::from(s)==neighbors));
        assert!(digraph.out_neighbors(0).is_some_and(|s| VertexSet::from(s)==out_neighbors));
        assert!(digraph.in_neighbors(0).is_some_and(|s| VertexSet::from(s)==in_neighbors));
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
