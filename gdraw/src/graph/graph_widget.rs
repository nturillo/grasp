use eframe::egui::{Align2, Color32, FontId, Stroke, Ui, Vec2};

use crate::{
    frame::style::Style,
    graph::storage::{Edge, Graph, Vertex},
};

pub fn draw_vertex(ui: &mut Ui, vertex: &Vertex, graph: &Graph, style: &Style, screen_center: Vec2, scale: f32) {
    let color = if let Some(color) = vertex.color {
        color
    } else {
        style.vertex_color
    }
    .lerp_to_gamma(
        style.select_color,
        if graph.selected_list.contains(&vertex.id) {
            style.select_color_strength
        } else {
            0.0
        },
    );

    ui.painter().circle(
        screen_center.to_pos2(),
        style.vertex_radius / scale,
        color,
        Stroke::new(style.outline_thickness / scale, style.outline_color),
    );

    if style.display_ids {
        ui.painter().text(screen_center.to_pos2(), Align2::CENTER_CENTER, vertex.id, FontId::monospace(style.vertex_radius / scale), Color32::BLACK);
    }
}

pub fn draw_edge(ui: &mut Ui, edge: &Edge, graph: &Graph, style: &Style, start_vertex_center: Vec2, end_vertex_center: Vec2, scale: f32) {
    let radius = if style.show_vertices {
        (style.vertex_radius + style.outline_thickness) / scale
    } else {
        0.0
    };

    let color = if let Some(color) = edge.color {
        color
    } else {
        style.edge_color
    };

    let dir_vector = (end_vertex_center - start_vertex_center).normalized();
    let new_start = start_vertex_center + radius * dir_vector;
    let new_end = end_vertex_center - radius * dir_vector;
    let thickness = style.edge_thickness / scale;

    if graph.directed {
        let line_cutoff = new_end - f32::min(style.arrow_size / scale, (end_vertex_center - start_vertex_center).length()) * dir_vector;

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
}