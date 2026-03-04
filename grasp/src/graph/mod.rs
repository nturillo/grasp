pub mod error;
pub mod labeled_graph;
pub mod adjacency_list;
pub mod graph_ops;
pub mod util;
pub mod set;
pub mod directed;

pub mod prelude{
    pub use super::{
        GraphTrait, VertexMap, EdgeID, EdgeType, VertexID, GraphMut, ArbitraryIDGraph, BuildableGraph,
        directed::{SimpleGraph, DiGraph, DigraphProjection, SimpleView, UnderlyingView},
        labeled_graph::{LabeledGraph, HashMapLabeledGraph}, 
        adjacency_list::{SparseSimpleGraph, SparseDiGraph}, 
        error::GraphError,
        graph_ops::*,
        util::*, set::{Set, EmptyVertexSet}
    };
}

use std::collections::HashMap;

use set::Set;
use crate::graph::error::GraphError;

pub type VertexMap = HashMap<VertexID, VertexID>;
pub type VertexID = usize;
pub type EdgeID = (VertexID, VertexID);

pub trait EdgeType{
    /// Converts to a simple EdgeID
    fn to_simple(self) -> Self;
    /// Turns edge ab into ba
    fn inv(self) -> Self;
}
impl EdgeType for EdgeID{
    fn to_simple(self) -> Self {
        if self.0 <= self.1 {(self.0, self.1)} else {(self.1, self.0)}
    }
    fn inv(self) -> Self {
        (self.1, self.0)
    }
}

/// Immutable Graph Functionality.
pub trait GraphTrait{
    /// Number of vertices
    fn vertex_count(&self) -> usize;
    /// Number of edges
    fn edge_count(&self) -> usize;
    /// Whether the graph is empty
    fn is_empty(&self) -> bool {self.vertex_count() == 0}

    /// Whether the graph contains the vertex
    fn has_vertex(&self, v: VertexID) -> bool;
    /// Whether the graph contains the edge
    fn has_edge(&self, e: EdgeID) -> bool;

    /// Iterator over vertices
    fn vertices(&self) -> impl Iterator<Item=VertexID>;
    /// Iterator over edges
    fn edges(&self) -> impl Iterator<Item=EdgeID>;

    /// Returns a set of vertices adjacent to given vertex. Returns empty set if the vertex is not in the graph. \
    /// In the case of a digraph, should only be out connected vtcs.
    fn neighbors(&self, v: VertexID) -> impl Set<Item = VertexID>;
    /// Returns a set of all vertices in the graph
    fn vertex_set(&self) -> impl Set<Item = VertexID>;

    /// Gets the degree of a vertex, 0 if not in the graph
    fn degree(&self, v: VertexID) -> usize{
        self.neighbors(v).len()
    }
}
/// Graph Mutation for adding and removing vtcs/edges
pub trait GraphMut: GraphTrait{
    /// Creates a new vertex in the graph and returns its ID
    fn create_vertex(&mut self) -> VertexID;
    /// Removes a vertex from the graph, returns a list of edges it removed as a consequence
    fn remove_vertex(&mut self, v: VertexID) -> impl Iterator<Item = EdgeID>;

    /// Adds an edge to the graph. 
    fn try_add_edge(&mut self, edge: EdgeID) -> Result<(), GraphError>;
    /// Removes an edge from the graph, does not remove the vertices.
    fn remove_edge(&mut self, e: EdgeID) -> bool;

    /// Creates edges from v1 to the neighbors, returns a list of vtcs which failed to add their edges, and the error.
    fn try_add_neighbors(&mut self, v: VertexID, nbhrs: impl IntoIterator<Item=VertexID>) -> Vec<(VertexID, GraphError)>{
        let mut errors = Vec::new();
        for v2 in nbhrs {
            if let Err(e) = self.try_add_edge((v, v2)) {
                errors.push((v2, e));
            }
        }
        errors
    }
}
/// Methods for graphs with arbitrary VertexID functionality
pub trait ArbitraryIDGraph: GraphMut{
    /// Adds a vertex with the specified id into the graph
    fn add_vertex(&mut self, id: VertexID);
    /// Adds an edge and makes sure the two vertices are added as well.
    fn add_edge(&mut self, edge: EdgeID) {
        let (v1, v2) = edge;
        self.add_vertex(v1); self.add_vertex(v2);
        let _ = self.try_add_edge(edge);
    }
    /// Adds edges from v to the vertices in nbhrs, adding missing vtcs as needed.
    fn add_neighbors(&mut self, v: VertexID, nbhrs: impl IntoIterator<Item = VertexID>){
        self.add_vertex(v);
        for v2 in nbhrs.into_iter(){
            self.add_vertex(v2);
            let _ = self.try_add_edge((v, v2));
        }
    }
}
/// Graph types that can be build from scratch
pub trait BuildableGraph: GraphMut+Sized {
    fn empty() -> Self;
    fn with_capacity(_v: usize, _e: usize) -> Self {
        Self::empty()
    }
}
impl<G: Default+GraphMut> BuildableGraph for G{
    fn empty() -> Self {
        Self::default()
    }
}

/// Contains test templates for GraphOps and SimpleGraphOps, should be used as a seperate test for each graph implementation
#[cfg(test)]
mod test{
    use std::collections::HashSet;

    use crate::graph::prelude::*;

    /// Assures SimpleGraph and DiGraph traits work as intended.
    pub fn graph_vs_digraph_test<S: SimpleGraph+Default+ArbitraryIDGraph, D: DiGraph+Default+ArbitraryIDGraph>(){
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
    pub fn digraph_fn_test<G: DiGraph+Default+ArbitraryIDGraph>(){
        let mut digraph = G::default();
        digraph.add_edge((0, 1).into()); digraph.add_edge((2, 0).into());
        // neighborhoods
        let neighbors = HashSet::from_iter([1, 2]);
        let out_neighbors = HashSet::from_iter([1]);
        let in_neighbors = HashSet::from_iter([2]);
        assert!(digraph.neighbors(0).set_eq(&neighbors));
        assert!(digraph.out_neighbors(0).set_eq(&out_neighbors));
        assert!(digraph.in_neighbors(0).set_eq(&in_neighbors));
    }

    /// Assures Underlying Graphs are correctly calculated
    pub fn digraph_projection_test<G: DigraphProjection+Default+ArbitraryIDGraph>(){
        // underlying graph
        let mut digraph = G::default();
        digraph.add_edge((0, 1)); digraph.add_edge((2, 0));
        digraph.add_edge((0, 3)); digraph.add_edge((3, 0));
        let simple_graph = digraph.as_simple();
        let und_graph = digraph.as_underlying();

        assert!(simple_graph.has_edge((1, 0)));
        assert!(simple_graph.has_edge((0, 2)));
        assert!(simple_graph.has_edge((0, 3)));
        assert_eq!(simple_graph.edge_count(), 3);
        assert!(und_graph.has_edge((0, 3)));
        assert_eq!(und_graph.edge_count(), 1);
    }
}
