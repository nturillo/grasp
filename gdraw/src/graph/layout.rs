pub struct LayoutConfig {
    pub edge_length: f32,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self { edge_length: 25.0 }
    }
}

pub enum LayoutType {
    FruchtermanReingold,
}
