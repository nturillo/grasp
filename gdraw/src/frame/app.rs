use crate::{
    frame::{graph_interaction, header, sandbox, style::Style},
    graph::storage::{Graph, Vertex},
};
use eframe::egui::{
    self, CentralPanel, Context, Id, Key, MenuBar, PointerButton, Popup, Response, Sense,
    TopBottomPanel, Ui, Vec2,
};

#[derive(Default)]
pub struct GraspApp {
    pub style: Style,
}

impl GraspApp {
    /// Opens the visualizer window.
    ///
    /// To display a graph, use [`crate::graph::load`] to load the graph before calling this function.
    pub fn start(&self) -> Result<(), eframe::Error> {
        let native_options = eframe::NativeOptions::default();
        eframe::run_native(
            "Grasp",
            native_options,
            Box::new(|cc| {
                cc.egui_ctx.set_visuals(egui::Visuals::light());
                Ok(Box::new(GraspAppHandler::new(cc, &self.style)))
            }),
        )
    }
}

pub(crate) struct GraspAppHandler<'a> {
    pub sandbox: sandbox::Sandbox,
    pub graph: Graph,
    pub style: &'a Style,
}

impl<'a> GraspAppHandler<'a> {
    fn new(cc: &eframe::CreationContext<'_>, style: &'a Style) -> Self {
        Self {
            sandbox: Default::default(),
            graph: Default::default(),
            style: style,
        }
    }
}

impl<'a> eframe::App for GraspAppHandler<'a> {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        TopBottomPanel::top(Id::new("menu_header")).show(ctx, |ui| {
            MenuBar::new().ui(ui, |ui| {
                header::file_menu(self, ui);
                header::edit_menu(self, ui);
                header::view_menu(self, ui);
                header::tool_menu(self, ui);
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            self.sandbox.update_screen_rect(ui.max_rect());

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

            let graph_list = self.sandbox.draw_graph(ui, &self.graph, &self.style);
            for (vertex_response, vertex_id) in graph_list.0 {
                graph_interaction::handle_vertex_response(self, ui, vertex_id, vertex_response);
            }
        });
    }
}
