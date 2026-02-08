use eframe::egui::Color32;

pub struct Style {
    pub vertex_radius: f32,
    pub edge_thickness: f32,
    pub arrow_size: f32,
    pub scroll_sensitivity: f32,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            vertex_radius: 25.0,
            edge_thickness: 5.0,
            arrow_size: 50.0,
            scroll_sensitivity: 0.01,
        }
    }
}
