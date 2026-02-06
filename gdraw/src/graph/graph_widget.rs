use eframe::egui::{Color32, Rect, Response, Sense, Stroke, Ui, Vec2, Widget};

use crate::{
    app::GraspApp,
    frame::style::Style,
    graph::storage::{Edge, Graph, Vertex},
};

pub struct VertexWidget<'a> {
    pub vertex: &'a Vertex,
    pub graph: &'a Graph,
    pub style: &'a Style,
    pub screen_center: Vec2,
}

impl<'a> Widget for VertexWidget<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let screen_rect = Rect::from_center_size(
            self.screen_center.to_pos2(),
            Vec2::splat(2.0 * self.style.vertex_radius),
        );

        let response = ui.allocate_rect(screen_rect, Sense::click_and_drag());
        let color = match self.graph.selected_list.contains(&self.vertex.id) {
            true => Color32::from_rgb(196, 194, 167),
            false => Color32::from_rgb(200, 200, 200),
        };

        ui.painter().circle(
            screen_rect.center(),
            self.style.vertex_radius,
            color,
            Stroke::new(2.0, Color32::from_rgb(130, 130, 130)),
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
}

impl<'a> Widget for EdgeWidget<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut screen_rect = Rect::NOTHING;
        screen_rect.extend_with(self.start_vertex_center.to_pos2());
        screen_rect.extend_with(self.end_vertex_center.to_pos2());

        let response = ui.allocate_rect(screen_rect, Sense::click());

        let dir_vector = (self.end_vertex_center - self.start_vertex_center).normalized();
        let new_start = self.start_vertex_center
            + (self.style.vertex_radius + self.style.edge_thickness / 2.0) * dir_vector;
        let new_end = self.end_vertex_center
            - (self.style.vertex_radius + self.style.edge_thickness / 2.0) * dir_vector;

        if self.graph.directed {
            let line_cutoff = new_end - self.style.arrow_size * dir_vector;

            ui.painter().line(
                vec![line_cutoff.to_pos2(), new_start.to_pos2()],
                Stroke::new(self.style.edge_thickness, Color32::BLACK),
            );

            ui.painter().arrow(
                line_cutoff.to_pos2(),
                new_end - line_cutoff,
                Stroke::new(self.style.edge_thickness, Color32::BLACK),
            );
        } else {
            ui.painter().line(
                vec![new_end.to_pos2(), new_start.to_pos2()],
                Stroke::new(self.style.edge_thickness, Color32::BLACK),
            );
        }

        response
    }
}
