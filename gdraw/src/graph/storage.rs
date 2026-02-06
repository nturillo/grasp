use eframe::egui::Vec2;
use std::collections::HashMap;

pub struct Vertex {
    pub center: Vec2,
    pub id: usize,
}

pub struct Edge {
    pub start_vertex: usize,
    pub end_vertex: usize,
}

#[derive(Default)]
pub struct Graph {
    pub vertex_list: HashMap<usize, Vertex>,
    pub selected_list: Vec<usize>,
    pub edge_list: Vec<Edge>,
    pub directed: bool,

    index: usize,
}

impl Graph {
    pub fn create_vertex(&mut self, center: Vec2) {
        self.vertex_list.insert(
            self.index,
            Vertex {
                center: center,
                id: self.index,
            },
        );

        self.index += 1;
    }
}
