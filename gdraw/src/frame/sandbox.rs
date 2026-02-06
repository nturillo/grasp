use crate::graph::{
    graph_widget::VertexWidget,
    storage::{Graph, Vertex},
};
use eframe::egui::{Pos2, Rect, Response, Ui, Vec2};

pub(crate) struct Sandbox {
    pub center: Vec2,
    pub size: Vec2,
    pub screen_rect: Rect,

    last_context_location: Pos2,
}

impl Default for Sandbox {
    fn default() -> Self {
        Self {
            center: Vec2::new(0.5, 0.5),
            size: Vec2::new(1.0, 1.0),
            screen_rect: Rect::NOTHING,
            last_context_location: Pos2::ZERO,
        }
    }
}

impl Sandbox {
    pub fn sandbox_to_screen(&self, sandbox_coord: Vec2) -> Vec2 {
        self.screen_rect.center().to_vec2()
            + (sandbox_coord - self.center) * self.screen_rect.size() / self.size
    }

    pub fn screen_to_sandbox(&self, screen_coord: Vec2) -> Vec2 {
        self.center
            + (screen_coord - self.screen_rect.center().to_vec2()) * self.size
                / self.screen_rect.size()
    }

    pub fn create_vertex(&self, screen_coord: Vec2, graph: &mut Graph) {
        graph.create_vertex(self.screen_to_sandbox(screen_coord));
    }

    pub fn draw_graph(&self, ui: &mut Ui, graph: &Graph) -> Vec<(Response, usize)> {
        let mut vec = Vec::new();

        for (&index, vertex) in &graph.vertex_list {
            vec.push((
                ui.add(VertexWidget {
                    radius: 25.0,
                    vertex: &vertex,
                    graph: graph,
                    screen_center: self.sandbox_to_screen(vertex.center),
                }),
                index,
            ));
        }

        vec
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
}
