use eframe::egui::Vec2;
use std::collections::HashMap;

pub type VertexPair = [usize; 2];

pub struct Vertex {
    pub center: Vec2,
    pub id: usize,
}

pub struct Edge {
    pub vertex_pair: VertexPair,
}

#[derive(Default)]
pub struct Graph {
    pub vertex_list: HashMap<usize, Vertex>,
    pub edge_list: HashMap<VertexPair, Edge>,
    pub selected_list: Vec<usize>,
    pub directed: bool,

    vertex_id: usize,
}

impl Graph {
    pub fn create_vertex(&mut self, center: Vec2) {
        self.vertex_list.insert(
            self.vertex_id,
            Vertex {
                center: center,
                id: self.vertex_id,
            },
        );

        self.vertex_id += 1;
    }

    pub fn create_edge(&mut self, pair: VertexPair) {
        self.edge_list.insert(pair, Edge { vertex_pair: pair });
    }

    pub fn has_edge(&self, pair: VertexPair) -> bool {
        self.edge_list.contains_key(&pair)
            || (!self.directed && self.edge_list.contains_key(&[pair[1], pair[0]]))
    }

    pub fn remove_vertex(&mut self, vertex_id: &usize) -> Option<Vertex> {
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
        self.vertex_list
            .retain(|vert, _| !self.selected_list.contains(vert));
        self.selected_list = vec![];
    }

    pub fn remove_edge(&mut self, pair: VertexPair) -> Option<Edge> {
        self.edge_list.remove(&pair)
    }
}
