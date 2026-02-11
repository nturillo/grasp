use super::{Graph, VertexID, VertexMap, EdgeID};
use std::collections::HashMap;

pub struct LabeledGraph<G, V=(), E=()> where G: Graph {
    pub graph: G,
    pub vertex_labels: HashMap<VertexID, V>,
    pub edge_labels: HashMap<EdgeID, E>
}
impl<G: Graph, V, E> Default for LabeledGraph<G, V, E>{
    fn default() -> Self {
        Self{graph: G::default(), vertex_labels: HashMap::default(), edge_labels: HashMap::default()}
    }
}
impl<G: Graph, V, E> AsRef<G> for LabeledGraph<G, V, E>{
    fn as_ref(&self) -> &G {
        &self.graph
    }
}
impl<G: Graph, V, E> AsMut<G> for LabeledGraph<G, V, E>{
    fn as_mut(&mut self) -> &mut G {
        &mut self.graph
    }
}
impl<G: Graph, V, E> LabeledGraph<G, V, E>{
    pub fn get_edge_label(&self, edge: EdgeID) -> Option<&E>{
        if self.graph.has_edge(edge) {
            Some(self.edge_labels.get(&edge)?)
        }else {None}
    }
    pub fn get_vertex_label(&self, vertex: VertexID) -> Option<&V>{
        if self.graph.contains(vertex) {
            Some(self.vertex_labels.get(&vertex)?)
        }else {None}
    }
    /// adds new vertex if needed. Same behaviour as Graph Storage
    pub fn set_vertex_label(&mut self, vertex: VertexID, label: V) -> Option<V>{
        if !self.graph.contains(vertex) {self.graph.add_vertex(vertex);}
        self.vertex_labels.insert(vertex, label)
    }
    /// adds two new vertices and an edge if needed. Same behaviour as Graph Storage
    pub fn set_edge_label(&mut self, edge: EdgeID, label: E) -> Option<E>{
        if !self.graph.has_edge(edge) {self.graph.add_edge(edge);}
        self.edge_labels.insert(edge, label)
    }
}
impl<G: Graph, V: Clone, E: Clone> Graph for LabeledGraph<G, V, E>{
    type VertexSet = G::VertexSet;

    fn vertex_count(&self) -> usize {self.graph.vertex_count()}
    fn edge_count(&self) -> usize {self.graph.edge_count()}
    fn vertices(&self) -> impl Iterator<Item=VertexID> {self.graph.vertices()}
    fn edges(&self) -> impl Iterator<Item=EdgeID> {self.graph.edges()}
    fn contains(&self, v: VertexID) -> bool {self.graph.contains(v)}
    fn has_edge(&self, e: EdgeID) -> bool {self.graph.has_edge(e)}
    fn neighbors(&self, v: VertexID) -> Option<&Self::VertexSet> {self.graph.neighbors(v)}
    fn vertex_set(&self) -> Self::VertexSet {self.graph.vertex_set()}
    fn create_vertex(&mut self) -> VertexID {self.graph.create_vertex()}
    fn add_vertex(&mut self, v: VertexID) {self.graph.add_vertex(v)}
    fn add_edge(&mut self, e: EdgeID) {self.graph.add_edge(e)}
    fn add_neighbors(&mut self, v: VertexID, nbhrs: impl Iterator<Item=VertexID>) {self.graph.add_neighbors(v, nbhrs)}

    // Manipulation Functions
    fn subgraph_vertex(&self, vertices: impl Iterator<Item=VertexID>) -> Self {
        let subgraph = self.graph.subgraph_vertex(vertices);
        let vertex_labels = subgraph.vertices().filter_map(|v| self.get_vertex_label(v).cloned().map(|d| (v, d))).collect();
        let edge_labels = subgraph.edges().filter_map(|e| self.get_edge_label(e).cloned().map(|d| (e, d))).collect();
        Self{graph: subgraph, vertex_labels, edge_labels}
    }

    fn subgraph_edges(&self, edges: impl Iterator<Item=EdgeID>) -> Self {
        let subgraph = self.graph.subgraph_edges(edges);
        let vertex_labels = subgraph.vertices().filter_map(|v| self.get_vertex_label(v).cloned().map(|d| (v, d))).collect();
        let edge_labels = subgraph.edges().filter_map(|e| self.get_edge_label(e).cloned().map(|d| (e, d))).collect();
        Self{graph: subgraph, vertex_labels, edge_labels}
    }

    fn merge(&self, other: &Self) -> (Self, VertexMap, VertexMap) {
        let (merged, self_map, other_map) = self.graph.merge(&other.graph);
        let vertex_labels: HashMap<VertexID, V> = self.vertex_labels.iter()
            .filter_map(|(k, v)| Some((*self_map.get(k)?, v.clone())))
        .chain(other.vertex_labels.iter()
            .filter_map(|(k, v)| Some((*other_map.get(k)?, v.clone())))
        ).collect();
        let edge_labels: HashMap<EdgeID, E> = self.edge_labels.iter().filter_map(
            |((v1, v2), e)| Some(((*self_map.get(v1)?, *self_map.get(v2)?), e.clone()))
        ).chain(other.edge_labels.iter().filter_map(
            |((v1, v2), e)| Some(((*other_map.get(v1)?, *other_map.get(v2)?), e.clone()))
        )).collect();
        (Self{graph: merged, vertex_labels, edge_labels}, self_map, other_map)
    }

    fn join(&self, other: &Self) -> (Self, VertexMap, VertexMap) {
        let (join, self_map, other_map) = self.graph.merge(&other.graph);
        let vertex_labels: HashMap<VertexID, V> = self.vertex_labels.iter()
            .filter_map(|(k, v)| Some((*self_map.get(k)?, v.clone())))
        .chain(other.vertex_labels.iter()
            .filter_map(|(k, v)| Some((*other_map.get(k)?, v.clone())))
        ).collect();
        let edge_labels: HashMap<EdgeID, E> = self.edge_labels.iter().filter_map(
            |((v1, v2), e)| Some(((*self_map.get(v1)?, *self_map.get(v2)?), e.clone()))
        ).chain(other.edge_labels.iter().filter_map(
            |((v1, v2), e)| Some(((*other_map.get(v1)?, *other_map.get(v2)?), e.clone()))
        )).collect();
        (Self{graph: join, vertex_labels, edge_labels}, self_map, other_map)
    }

    fn product(&self, other: &Self) -> (Self, HashMap<(VertexID, VertexID), VertexID>) {
        let (product, map) = self.graph.product(&other.graph);
        // Since vertex and edge labels are ambiguous here we will leave them empty. 
        // Ideally, the return type of product would switch the vertex label into a tuple of (Option<V>, Option<V>), as that would allow the most correct behaviour.
        // Unfortunately it was not designed to support this and I dont care enough to change it now.
        (Self{graph: product, vertex_labels: HashMap::default(), edge_labels: HashMap::default()}, map)
    }

    fn complement(&self) -> Self {
        Self{graph: self.graph.complement(), vertex_labels: self.vertex_labels.clone(), edge_labels: HashMap::default()}
    }
    
}

/// Trait used to represent types that can be used as a number
pub trait Number: Clone+Copy+std::ops::Add<Output=Self>+std::ops::Sub<Output=Self>+std::ops::Mul<Output=Self>+std::ops::Div<Output=Self>+PartialOrd{}
impl<T> Number for T where T: 
    Clone+Copy+PartialOrd+
    std::ops::Add<Output=Self>+std::ops::Sub<Output=Self>+
    std::ops::Mul<Output=Self>+std::ops::Div<Output=Self>
{}

/// Trait implemented on graph types that have lengths attached to edges
pub trait DistanceGraph: Graph{
    type EdgeLengths: Number;
    /// Returns the length of an edge, converted to f32
    fn edge_length(&self, e: EdgeID) -> Option<Self::EdgeLengths>;
}
impl<G: Graph, V: Clone, E: Number> DistanceGraph for LabeledGraph<G, V, E>{
    type EdgeLengths = E;
    fn edge_length(&self, e: EdgeID) -> Option<Self::EdgeLengths> {
        self.get_edge_label(e).cloned()
    }
}

// TODO: Tests for Labeled Graphs
