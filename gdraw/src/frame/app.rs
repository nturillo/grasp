use crate::{
    frame::{
        graph_interaction, header,
        sandbox::{self, Sandbox},
        style::Style,
    },
    graph::{
        layout::{self, LayoutConfig},
        storage::Graph,
    },
};
use eframe::egui::{
    self, CentralPanel, Context, Id, Key, MenuBar, PointerButton, Popup, Response, Sense,
    TopBottomPanel, Ui, Vec2,
};
use grasp::graph::graph_traits::GraphTrait;

pub struct GraspApp {
    pub style: Style,
    pub layout_config: LayoutConfig,
    pub graph: Graph,
    pub window_size: (f32, f32),
}

impl Default for GraspApp {
    fn default() -> Self {
        Self {
            style: Default::default(),
            layout_config: Default::default(),
            graph: Default::default(),
            window_size: (800.0, 800.0),
        }
    }
}

impl GraspApp {
    /// Opens the visualizer window.
    ///
    /// To display a graph, use [`crate::graph::load`] to load the graph before calling this function.
    pub fn start(&self) -> Result<(), eframe::Error> {
        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size(Vec2::from(self.window_size)),
            ..Default::default()
        };

        eframe::run_native(
            "Grasp",
            native_options,
            Box::new(|cc| {
                cc.egui_ctx.set_visuals(egui::Visuals::light());
                Ok(Box::new(GraspAppHandler::new(
                    cc,
                    self.graph.clone(),
                    &self.style,
                )))
            }),
        )
    }

    /// Loads a graph from anything that implements [`grasp::graph::graph_traits::GraphTrait`]
    pub fn load<T: GraphTrait>(&mut self, graph: &T) {
        self.graph = Graph::from(graph);
        layout::apply(&mut self.graph, &self.layout_config);
    }
}

pub(crate) struct GraspAppHandler<'a> {
    pub sandbox: sandbox::Sandbox,
    pub graph: Graph,
    pub style: &'a Style,
}

impl<'a> GraspAppHandler<'a> {
    fn new(cc: &eframe::CreationContext<'_>, graph: Graph, style: &'a Style) -> Self {
        let mut sandbox = Sandbox::default();
        sandbox.scale(3.0);

        Self {
            sandbox: sandbox,
            graph: graph,
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
