use crate::graph::layout::{self, PartialLayout};
use eframe::egui::{Color32, Vec2};
use grasp::graph::{adjacency_list::SparseSimpleGraph, error::GraphError, graph_ops::GraphOps, GraphTrait};
use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use crate::graph::layout::LayoutConfig;

pub type VertexPair = [usize; 2];

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
    pub vertex_pair: VertexPair,
}

#[derive(Default)]
pub struct Graph {
    pub vertex_list: HashMap<usize, Vertex>,
    pub edge_list: HashMap<VertexPair, Edge>,
    pub selected_list: Vec<usize>,
    pub directed: bool,
    pub layout_config: LayoutConfig,
    pub base_graph: Option<SparseGraph>,

    vertex_id: usize,
}

impl Clone for Graph {
    fn clone(&self) -> Self {
        Self {
            vertex_list: self.vertex_list.clone(),
            edge_list: self.edge_list.clone(),
            selected_list: self.selected_list.clone(),
            directed: self.directed,
            layout_config: self.layout_config.clone(),
            base_graph: match &self.base_graph {
                Some(graph) => Some(clone_graph(&graph)),
                None => None,
            },

            vertex_id: self.vertex_id,
        }
    }
}

impl Graph {
    pub fn create_vertex(&mut self, center: Vec2) {
        self.reset_partial_data();
        while self.vertex_list.contains_key(&self.vertex_id) {
            self.vertex_id += 1;
        }

        self.vertex_list.insert(
            self.vertex_id,
            Vertex {
                center: center,
                id: self.vertex_id,
                color: Default::default(),
            },
        );
    }

    pub fn insert_vertex(&mut self, vertex: Vertex) -> bool {
        if self.vertex_list.contains_key(&vertex.id) {
            return false;
        }

        self.reset_partial_data();
        return match self.vertex_list.insert(vertex.id, vertex) {
            None => true,
            _ => false,
        };
    }

    pub fn create_edge(&mut self, pair: VertexPair) {
        self.reset_partial_data();
        self.edge_list.insert(pair, Edge { vertex_pair: pair });
    }

    pub fn has_edge(&self, pair: VertexPair) -> bool {
        self.edge_list.contains_key(&pair)
            || (!self.directed && self.edge_list.contains_key(&[pair[1], pair[0]]))
    }

    pub fn remove_vertex(&mut self, vertex_id: &usize) -> Option<Vertex> {
        self.reset_partial_data();
        match self.vertex_list.remove(vertex_id) {
            None => None,
            Some(vertex) => {
                self.edge_list.retain(|edge, _| !edge.contains(&vertex_id));
                self.selected_list.retain(|vert| vert != vertex_id);

                Some(vertex)
            }
        }
    }

    pub fn remove_selected(&mut self) {
        self.reset_partial_data();
        self.vertex_list
            .retain(|vert, _| !self.selected_list.contains(vert));
        self.selected_list = vec![];
    }

    pub fn remove_edge(&mut self, pair: VertexPair) {
        self.reset_partial_data();
        self.edge_list
            .retain(|edge, _| edge != &pair && (self.directed || edge != &[pair[1], pair[0]]));
    }

    pub fn vertices(&self) -> usize {
        self.vertex_list.len()
    }

    pub fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.vertex_list
            .values()
            .for_each(|val| val.id.hash(&mut hasher));
        self.edge_list.keys().for_each(|val| val.hash(&mut hasher));
        hasher.finish()
    }

    fn reset_partial_data(&mut self) {
        self.layout_config.partial_data = PartialLayout::None;
    }

    pub fn save_base_graph(&mut self) {
        let mut graph = SparseGraph::new();

        for &vertex in self.vertex_list.keys() {
            graph.add_vertex(vertex);
        }

        for &edge in self.edge_list.keys() {
            graph.add_edge((edge[0], edge[1]));
        }

        self.base_graph = Some(graph);
    }
}

impl<G: GraphOps> From<&G> for Graph {
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
            graph.create_edge([edge.0, edge.1]);
        }

        graph
    }
}

pub fn clone_graph(in_graph: &SparseGraph) -> SparseGraph {
    let mut graph = SparseGraph::new();

    for vertex in in_graph.vertices() {
        graph.add_vertex(vertex);
    }

    for edge in in_graph.edges() {
        graph.add_edge(edge);
    }

    graph
}
