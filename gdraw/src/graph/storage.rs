use crate::graph::layout::{PartialLayout};
use eframe::egui::{Color32, Vec2};
use grasp::graph::{EdgeID, GraphTrait, VertexID, adjacency_list::SparseDiGraph};
use std::{
    collections::HashMap,
};

use crate::graph::layout::LayoutConfig;

#[derive(Default, Clone)]
pub struct Vertex {
    pub center: Vec2,
    pub id: usize,

    pub color: Option<Color32>,
}

impl Vertex {
    pub fn assign_color(&mut self, color: Color32) {
        self.color = Some(color);
    }

    pub fn clear_color(&mut self) {
        self.color = None;
    }
}

#[derive(Default, Clone)]
pub struct Edge {
    pub vertex_pair: EdgeID,
}

pub struct Graph {
    pub base: SparseDiGraph,
    pub vertex_labels: HashMap<VertexID, Vertex>,
    pub edge_labels: HashMap<EdgeID, Edge>,
    pub selected_list: Vec<VertexID>,
    pub directed: bool,
    pub layout_config: LayoutConfig,
    pub vertex_id: VertexID,
}
impl Default for Graph {
    fn default() -> Self {
        Self{base: SparseDiGraph::default(), vertex_labels: HashMap::default(), edge_labels: HashMap::default(), selected_list: Vec::default(), directed: false, layout_config: LayoutConfig::default(), vertex_id: 0}
    }
}
impl Clone for Graph {
    fn clone(&self) -> Self {
        Self { base: clone_graph(&self.base), vertex_labels: self.vertex_labels.clone(), edge_labels: self.edge_labels.clone(), selected_list: self.selected_list.clone(), directed: self.directed.clone(), layout_config: self.layout_config.clone(), vertex_id: 0 }
    }
}

impl Graph {
    pub fn create_vertex(&mut self, center: Vec2) {
        self.reset_partial_data();
        while self.base.vertices().any(|f| f == self.vertex_id) {
            self.vertex_id += 1;
        }

        self.base.add_vertex(self.vertex_id);
        self.vertex_labels.insert(self.vertex_id, Vertex {
                center: center,
                id: self.vertex_id,
                color: Default::default(),
            });
    }

    pub fn insert_vertex(&mut self, vertex: Vertex) -> bool {
        if self.vertex_labels.contains_key(&vertex.id) {
            return false;
        }
        self.base.add_vertex(vertex.id);
        self.vertex_labels.insert(vertex.id, vertex);
        self.reset_partial_data();

        return true;
    }

    pub fn create_edge(&mut self, pair: EdgeID) {
        self.reset_partial_data();
        self.base.add_edge(pair);
        self.edge_labels.insert(pair, Edge { vertex_pair: pair });
    }

    pub fn has_edge(&self, pair: EdgeID) -> bool {
        self.base.has_edge(pair) || (!self.directed && self.base.has_edge((pair.1, pair.0)))
    }

    pub fn remove_vertex(&mut self, vertex_id: VertexID) -> Option<Vertex> {
        self.reset_partial_data();
        match self.vertex_labels.remove(&vertex_id) {
            None => None,
            Some(vertex) => {
                let _ = self.base.delete_vertex(vertex.id);
                self.edge_labels.retain(|&(source, target), _| source != vertex_id && target != vertex_id);
                self.selected_list.retain(|&v| v != vertex.id);

                Some(vertex)
            }
        }
    }

    pub fn remove_selected(&mut self) {
        self.reset_partial_data();
        self.selected_list.drain(..).for_each(|vert| {
            self.edge_labels.retain(|&(source, target), _| source != vert && target != vert);
            self.vertex_labels.remove(&vert);
            let _ = self.base.delete_vertex(vert);} );
    }

    pub fn remove_edge(&mut self, pair: EdgeID) {
        self.reset_partial_data();

        self.edge_labels.remove(&pair);
        self.base.delete_edge(pair);

        if !self.directed {
            let pair = (pair.1, pair.0);
            self.edge_labels.remove(&pair);
            self.base.delete_edge(pair);
        }
    }

    fn reset_partial_data(&mut self) {
        self.layout_config.partial_data = PartialLayout::None;
    }
}

impl<G: GraphTrait + Default> From<&G> for Graph {
    fn from(provider: &G) -> Self {
        let mut graph = Graph::default();

        for vertex_id in provider.vertices() {
            graph.insert_vertex(Vertex {
                id: vertex_id,
                center: Default::default(),
                color: Default::default(),
            });
        }

        for edge in provider.edges() {
            graph.create_edge((edge.0, edge.1));
        }

        graph
    }
}

pub fn clone_graph(in_graph: &SparseDiGraph) -> SparseDiGraph {
    let mut graph = SparseDiGraph::default();

    for vertex in in_graph.vertices() {
        graph.add_vertex(vertex);
    }

    for edge in in_graph.edges() {
        graph.add_edge(edge);
    }

    graph
}