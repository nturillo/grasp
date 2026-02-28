use eframe::egui::Color32;

#[derive(Clone, Copy)]
pub struct Style {
    pub vertex_radius: f32,
    pub vertex_color: Color32,
    pub outline_color: Color32,
    pub outline_thickness: f32,
    pub select_color: Color32,
    pub select_color_strength: f32,
    pub highlight_color: Color32,
    pub edge_color: Color32,
    pub edge_highlight_color: Color32,
    pub show_vertices: bool,
    pub edge_thickness: f32,
    pub arrow_size: f32,
    pub scroll_sensitivity: f32,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            vertex_radius: 25.0,
            vertex_color: Color32::from_rgb(200, 200, 200),
            outline_color: Color32::from_rgb(130, 130, 130),
            outline_thickness: 2.0,
            select_color: Color32::YELLOW,
            select_color_strength: 0.2,
            highlight_color: Color32::RED,
            edge_color: Color32::BLACK,
            edge_highlight_color: Color32::RED,
            show_vertices: true,
            edge_thickness: 5.0,
            arrow_size: 50.0,
            scroll_sensitivity: 0.01,
        }
    }
}
