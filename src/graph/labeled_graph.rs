use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use crate::graph::graph_traits::{EdgeType, GraphTrait, VertexType};

pub trait LabeledGraphTrait<V, E>: GraphTrait {
    fn get_vertex_label(&self, v: &VertexType) -> Option<&V>;
    fn get_edge_label(&self, e: &EdgeType) -> Option<&E>;
}

pub struct LabeledGraph<G, V, E> {
    graph: G,
    vertex_data: HashMap<VertexType, V>,
    edge_data: HashMap<EdgeType, E>,
}

impl<G: GraphTrait, V, E> LabeledGraph<G, V, E> {
    fn from_graph(g: G) -> Self {
        Self {
            graph: g,
            vertex_data: HashMap::new(),
            edge_data: HashMap::new(),
        }
    }
}

impl<G: GraphTrait, V, E> LabeledGraphTrait<V, E> for LabeledGraph<G, V, E> {
    fn get_vertex_label(&self, v: &VertexType) -> Option<&V> {
        self.vertex_data.get(v)
    }
    fn get_edge_label(&self, e: &EdgeType) -> Option<&E> {
        self.edge_data.get(e)
    }
}

impl<G: GraphTrait, V, E> GraphTrait for LabeledGraph<G, V, E> {
    type NeighborSet = G::NeighborSet;

    fn new() -> Self {
        Self {
            graph: G::new(),
            vertex_data: HashMap::new(),
            edge_data: HashMap::new(),
        }
    }
    fn num_vertices(&self) -> usize {
        self.graph.num_vertices()
    }
    fn num_edges(&self) -> usize {
        self.graph.num_edges()
    }
    fn vertices(&self) -> impl Iterator<Item = VertexType> {
        self.graph.vertices()
    }
    fn edges(&self) -> impl Iterator<Item = (VertexType, VertexType)> {
        self.graph.edges()
    }
    fn contains(&self, v: VertexType) -> bool {
        self.graph.contains(v)
    }
    fn has_edge(&self, e: EdgeType) -> Result<bool, super::errors::GraphError> {
        self.graph.has_edge(e)
    }
    fn neighbors(&self, v: VertexType) -> Result<&Self::NeighborSet, super::errors::GraphError> {
        self.graph.neighbors(v)
    }
    fn add_vertex(&mut self, v: VertexType) {
        self.graph.add_vertex(v);
    }
    fn add_edge(&mut self, e: EdgeType) {
        self.graph.add_edge(e);
    }
    fn add_neighbors(&mut self, v: VertexType, nbhrs: impl Iterator<Item = VertexType>) {
        self.graph.add_neighbors(v, nbhrs);
    }
}
