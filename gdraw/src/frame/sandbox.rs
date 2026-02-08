use crate::{
    frame::style::Style,
    graph::{
        graph_widget::{EdgeWidget, VertexWidget},
        storage::{Graph, Vertex, VertexPair},
    },
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
            center: Vec2::new(0.0, 0.0),
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

    pub fn draw_graph(
        &self,
        ui: &mut Ui,
        graph: &Graph,
        style: &Style,
    ) -> (Vec<(Response, usize)>, Vec<(Response, VertexPair)>) {
        let mut vertex_vec = Vec::new();
        let mut edge_vec = Vec::new();

        for ([start_index, end_index], edge) in &graph.edge_list {
            if let (Some(start_vertex), Some(end_vertex)) = (
                graph.vertex_list.get(start_index),
                graph.vertex_list.get(end_index),
            ) {
                edge_vec.push((
                    ui.add(EdgeWidget {
                        edge: &edge,
                        graph: graph,
                        style: style,
                        start_vertex_center: self.sandbox_to_screen(start_vertex.center),
                        end_vertex_center: self.sandbox_to_screen(end_vertex.center),
                    }),
                    [*start_index, *end_index],
                ));
            }
        }

        for (&index, vertex) in &graph.vertex_list {
            let widget = ui.add(VertexWidget {
                vertex: &vertex,
                graph: graph,
                style: style,
                screen_center: self.sandbox_to_screen(vertex.center),
            });

            vertex_vec.push((widget, index));
        }

        (vertex_vec, edge_vec)
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

    pub fn scale(&mut self, factor: f32) {
        self.size *= factor;
    }
}
