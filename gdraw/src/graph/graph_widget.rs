use eframe::egui::{Rect, Response, Sense, Stroke, Ui, Vec2, Widget};

use crate::{
    frame::style::Style,
    graph::storage::{Edge, Graph, Vertex},
};

pub struct VertexWidget<'a> {
    pub vertex: &'a Vertex,
    pub graph: &'a Graph,
    pub style: &'a Style,
    pub screen_center: Vec2,
    pub scale: f32,
}

impl<'a> Widget for VertexWidget<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let screen_rect = Rect::from_center_size(
            self.screen_center.to_pos2(),
            Vec2::splat(2.0 * self.style.vertex_radius / self.scale),
        );

        let color = if let Some(color) = self.vertex.color {
            color
        } else {
            self.style.vertex_color
        }
        .lerp_to_gamma(
            self.style.select_color,
            if self.graph.selected_list.contains(&self.vertex.id) {
                self.style.select_color_strength
            } else {
                0.0
            },
        );

        let response = ui.allocate_rect(screen_rect, Sense::click_and_drag()).on_hover_text(format!("id: {}", self.vertex.id));

        ui.painter().circle(
            screen_rect.center(),
            self.style.vertex_radius / self.scale,
            color,
            Stroke::new(self.style.outline_thickness / self.scale, self.style.outline_color),
        );

        response
    }
}

pub struct EdgeWidget<'a> {
    pub edge: &'a Edge,
    pub graph: &'a Graph,
    pub style: &'a Style,
    pub start_vertex_center: Vec2,
    pub end_vertex_center: Vec2,
    pub scale: f32,
}

impl<'a> Widget for EdgeWidget<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut screen_rect = Rect::NOTHING;
        screen_rect.extend_with(self.start_vertex_center.to_pos2());
        screen_rect.extend_with(self.end_vertex_center.to_pos2());

        let response = ui.allocate_rect(screen_rect, Sense::click());

        let radius = if self.style.show_vertices {
            (self.style.vertex_radius + self.style.outline_thickness) / self.scale
        } else {
            0.0
        };

        let color = if let Some(color) = self.edge.color {
            color
        } else {
            self.style.edge_color
        };

        let dir_vector = (self.end_vertex_center - self.start_vertex_center).normalized();
        let new_start = self.start_vertex_center + radius * dir_vector;
        let new_end = self.end_vertex_center - radius * dir_vector;
        let thickness = self.style.edge_thickness / self.scale;

        if self.graph.directed {
            let line_cutoff = new_end - f32::min(self.style.arrow_size / self.scale, (self.end_vertex_center - self.start_vertex_center).length()) * dir_vector;

            ui.painter().line(
                vec![line_cutoff.to_pos2(), new_start.to_pos2()],
                Stroke::new(thickness, color),
            );

            ui.painter().arrow(
                line_cutoff.to_pos2(),
                new_end - line_cutoff,
                Stroke::new(thickness, color),
            );
        } else {
            ui.painter().line(
                vec![new_end.to_pos2(), new_start.to_pos2()],
                Stroke::new(thickness, color),
            );
        }

        response
    }
}
