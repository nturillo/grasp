use std::collections::{HashMap, HashSet};
use crate::graph::errors::*;

pub type VertexType = usize;
pub type EdgeType = (VertexType, VertexType);

pub trait Set<V>: Clone {
    fn contains(&self, v: V) -> bool;
    fn count(&self) -> usize;
    fn union(&self, other: &Self) -> Self;
    fn intersection(&self, other: &Self) -> Self;
    fn iter(&self) -> impl Iterator<Item=&V>;
    fn into_iter(self) -> impl Iterator<Item=V>;
}
impl<V: Clone> Set<V> for HashSet<V>{
    fn contains(&self, v: V) -> bool {
        self.contains(v)
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
    fn iter(&self) -> impl Iterator<Item=&V>{
        self.iter()
    }
    fn into_iter(self) -> impl Iterator<Item=V> {
        IntoIterator::into_iter(self)
    }
}

pub trait GraphTrait {
    type NeighborSet: Set<VertexID>;
    
    fn new() -> Self;

    fn num_vertices(&self) -> usize;
    fn num_edges(&self) -> usize;
    fn vertices(&self) -> impl Iterator<Item=VertexType>;
    fn edges(&self) -> impl Iterator<Item=(VertexType,VertexType)>;
    fn contains(&self, v: VertexType) -> bool;
    fn has_edge(&self, e: EdgeType) -> Result<bool, GraphError>;
    fn neighbors(&self, v: VertexType) -> Result<&Self::NeighborSet, GraphError>;
    
    fn add_vertex(&mut self, v: VertexType);
    fn add_edge(&mut self, e: EdgeType);
        //adds v1 and v2 if they don't exist
    fn add_neighbors(&mut self, v: VertexType, nbhrs: impl Iterator<Item=VertexType>);
        //adds v and nbhrs if they don't exist
}

pub type VertexID = usize;
pub type EdgeID = (VertexID, VertexID);
pub type VertexMap = HashMap<VertexID, VertexID>;

pub trait GraphStorage: Default{
    type VertexSet: Set<VertexID>;
    
    fn vertex_count(&self) -> usize;
    fn edge_count(&self) -> usize;
    fn vertices(&self) -> impl Iterator<Item=VertexID>;
    fn edges(&self) -> impl Iterator<Item=(VertexID,VertexID)>;
    fn contains(&self, v: VertexID) -> bool;
    fn has_edge(&self, v1: VertexID, v2: VertexID) -> Option<bool>;
    fn neighbors(&self, v: VertexID) -> Option<&Self::VertexSet>;

    fn add_vertex(&mut self, v: VertexID);
    /// adds v1 and v2 if they don't exist
    fn add_edge(&mut self, v1: VertexID, v2:VertexID);
    /// adds v and nbhrs if they don't exist
    fn add_neighbors(&mut self, v: VertexID, nbhrs: impl Iterator<Item=VertexID>);
}

pub trait Graph{
    /// Returns the subgraph induced by the set of vertices
    fn subgraph_vertex(&self, vertices: impl Iterator<Item=VertexID>) -> Self;
    /// Returns the subgraph induced by the set of edges
    fn subgraph_edges(&self, edges: impl Iterator<Item=EdgeID>) -> Self;
    /// Returns the two graphs combined into one as well as a map to the new vertexIDs
    /// Maps (in_self_graph, vertex) -> new_vertex
    fn merge(&self, other: &Self) -> (Self, VertexMap, VertexMap);
    /// Returns the join of self and other, as well as a map to new vertexIDs
    /// Maps (in_self_graph, vertex) -> new_vertex
    fn join(&self, other: &Self) -> (Self, HashMap<(bool, VertexID), VertexID>);
}

pub struct LabeledGraph<G, V=(), E=()> where G: Graph {
    pub graph: G,
    pub vertex_labels: V,
    pub edge_labels: E
}

