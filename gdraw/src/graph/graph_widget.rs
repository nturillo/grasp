use eframe::egui::{Color32, Rect, Response, Sense, Stroke, Ui, Vec2, Widget};

use crate::{
    app::GraspApp,
    graph::storage::{Graph, Vertex},
};

pub(crate) struct VertexWidget<'a> {
    pub vertex: &'a Vertex,
    pub graph: &'a Graph,
    pub radius: f32,
    pub screen_center: Vec2,
}

impl<'a> Widget for VertexWidget<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let screen_rect =
            Rect::from_center_size(self.screen_center.to_pos2(), Vec2::splat(2.0 * self.radius));
        let response = ui.allocate_rect(screen_rect, Sense::click_and_drag());

        let color = match self.graph.selected_list.contains(&self.vertex.id) {
            true => Color32::from_rgb(196, 194, 167),
            false => Color32::from_rgb(200, 200, 200),
        };

        ui.painter().circle(
            screen_rect.center(),
            self.radius,
            color,
            Stroke::new(2.0, Color32::from_rgb(130, 130, 130)),
        );

        response
    }
}
