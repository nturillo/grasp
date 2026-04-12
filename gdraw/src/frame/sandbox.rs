use crate::{
    frame::style::Style,
    graph::{
        graph_widget::{draw_edge, draw_vertex},
        storage::Graph,
    },
};
use eframe::egui::{Pos2, Rect, Ui, Vec2};

pub(crate) struct Sandbox {
    pub center: Vec2,
    pub size: Vec2,
    pub screen_rect: Rect,
    pub scale: f32,

    last_context_location: Pos2,
}

impl Default for Sandbox {
    fn default() -> Self {
        Self {
            center: Vec2::new(0.0, 0.0),
            size: Vec2::new(1.0, 1.0),
            scale: 1.0,
            screen_rect: Rect::NOTHING,
            last_context_location: Pos2::ZERO,
        }
    }
}

impl Sandbox {
    pub fn sandbox_to_screen(&self, sandbox_coord: Vec2) -> Vec2 {
        self.screen_rect.center().to_vec2()
            + (sandbox_coord - self.center) * self.screen_rect.size() / (self.size * self.scale)
    }

    pub fn screen_to_sandbox(&self, screen_coord: Vec2) -> Vec2 {
        self.center
            + (screen_coord - self.screen_rect.center().to_vec2()) * (self.size * self.scale)
                / self.screen_rect.size()
    }

    #[allow(dead_code)]
    pub fn sandbox_dist_to_screen_dist(&self, sandbox_dist: Vec2) -> Vec2 {
        sandbox_dist * self.screen_rect.size() / (self.size * self.scale)
    }

    pub fn screen_dist_to_sandbox_dist(&self, screen_dist: Vec2) -> Vec2 {
        screen_dist * (self.size * self.scale) / self.screen_rect.size()
    }

    pub fn create_vertex(&self, screen_coord: Vec2, graph: &mut Graph) {
        graph.create_vertex(self.screen_to_sandbox(screen_coord));
    }

    pub fn draw_graph(
        &self,
        ui: &mut Ui,
        graph: &Graph,
        style: &Style,
    ) {
        for ((start_index, end_index), edge) in &graph.edge_labels {
            if let (Some(start_vertex), Some(end_vertex)) = (
                graph.vertex_labels.get(start_index),
                graph.vertex_labels.get(end_index),
            ) {
                draw_edge(ui, edge, graph, style, self.sandbox_to_screen(start_vertex.center), self.sandbox_to_screen(end_vertex.center), self.scale);
            }
        }

        if style.show_vertices {
            for (_, vertex) in &graph.vertex_labels {
                draw_vertex(ui, vertex, graph, style, self.sandbox_to_screen(vertex.center), self.scale);
            }
        }
    }

    pub fn context_menu(&mut self, ui: &mut Ui, context_location: Option<Pos2>, graph: &mut Graph) {
        if let Some(coord) = context_location {
            self.last_context_location = coord
        }

        let create_vertex_response = ui.button("Create Vertex");
        if create_vertex_response.clicked() {
            self.create_vertex(self.last_context_location.to_vec2(), graph);
        }
    }

    pub fn update_screen_rect(&mut self, rect: Rect) {
        if self.screen_rect == Rect::NOTHING {
            self.screen_rect = rect;
            return;
        }

        self.size = self.size * rect.size() / self.screen_rect.size();
        self.screen_rect = rect;
    }

    pub fn reset(&mut self) {
        self.center = Vec2::new(0.0, 0.0);
        self.scale = 1.0;
    }
}
