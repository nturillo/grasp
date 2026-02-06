use crate::{
    frame::{header, sandbox},
    graph::storage::{Graph, Vertex},
};
use eframe::egui::{
    self, CentralPanel, Context, Id, MenuBar, PointerButton, Popup, Response, Sense,
    TopBottomPanel, Ui, Vec2,
};

#[derive(Default)]
pub(crate) struct GraspApp {
    pub sandbox: sandbox::Sandbox,
    pub graph: Graph,
}

impl GraspApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
}

impl eframe::App for GraspApp {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        TopBottomPanel::top(Id::new("menu_header")).show(ctx, |ui| {
            MenuBar::new().ui(ui, |ui| {
                header::file_menu(ui);
                header::edit_menu(ui);
                header::view_menu(ui);
                header::tool_menu(ui);
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            self.sandbox.screen_rect = ui.max_rect();

            let response = ui.interact(
                ui.max_rect(),
                Id::new("sandbox_clickable"),
                Sense::click_and_drag(),
            );

            if !Popup::is_any_open(ui.ctx()) && response.clicked() {
                if let Some(coords) = response.interact_pointer_pos() {
                    self.sandbox
                        .create_vertex(coords.to_vec2(), &mut self.graph);
                }
            }

            response.context_menu(|ui| {
                self.sandbox
                    .context_menu(ui, response.interact_pointer_pos(), &mut self.graph)
            });

            let vertex_list = self.sandbox.draw_graph(ui, &self.graph);
            for (vertex_response, vertex_id) in vertex_list {
                if !Popup::is_any_open(ui.ctx()) {
                    self.handle_vertex_response(ui, vertex_id, vertex_response);
                }
            }
        });
    }
}

impl GraspApp {
    fn vertex_primary_click(&mut self, ui: &mut Ui, vertex_id: usize, response: Response) {
        if self.graph.selected_list.contains(&vertex_id) {
            self.graph.selected_list.retain(|&id| id != vertex_id)
        } else {
            self.graph.selected_list = vec![vertex_id];
        }
    }

    fn handle_vertex_response(&mut self, ui: &mut Ui, vertex_id: usize, response: Response) {
        if let Some(vertex) = self.graph.vertex_list.get(&vertex_id) {
            if response.clicked() {
                self.vertex_primary_click(ui, vertex_id, response);
            }
        }
    }
}

/// Opens the visualizer window.
///
/// To display a graph, use [`crate::graph::load`] to load the graph before calling this function.
pub fn start() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Grasp",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::light());
            Ok(Box::new(GraspApp::new(cc)))
        }),
    )
}
