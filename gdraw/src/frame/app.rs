use crate::{
    frame::{
        graph_interaction, header,
        sandbox::{self, Sandbox},
        style::Style,
        windows,
    },
    graph::{
        layout::{self, LayoutConfig},
        storage::Graph,
    },
};
use eframe::egui::{
    self, CentralPanel, Context, Id, Key, MenuBar, PointerButton, Popup, Response, Sense,
    TopBottomPanel, Ui, Vec2, Window,
};
use grasp::graph::graph_traits::GraphTrait;

pub struct GraspApp {
    pub style: Style,
    graph: Graph,
    pub window_size: (f32, f32),
}

impl Default for GraspApp {
    fn default() -> Self {
        Self {
            style: Default::default(),
            graph: Default::default(),
            window_size: (800.0, 800.0),
        }
    }
}

impl GraspApp {
    /// Opens the visualizer window.
    ///
    /// To display a graph, call [`crate::frame::app::GraspApp::load`] to load the graph before calling this function.
    pub fn start(&mut self) -> Result<(), eframe::Error> {
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
                    &mut self.graph,
                    self.style,
                )))
            }),
        )
    }

    /// Loads a graph from anything that implements [`grasp::graph::graph_traits::GraphTrait`]
    pub fn load<T: GraphTrait>(&mut self, graph: &T) {
        self.graph = Graph::from(graph);
        layout::apply(&mut self.graph);
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_layout_config(&mut self, config: LayoutConfig) {
        self.graph.layout_config = config;
    }
}

pub(crate) struct GraspAppHandler<'a> {
    pub sandbox: sandbox::Sandbox,
    pub graph: &'a mut Graph,
    pub style: Style,

    pub show_settings: bool,
}

impl<'a> GraspAppHandler<'a> {
    fn new(cc: &eframe::CreationContext<'_>, graph: &'a mut Graph, style: Style) -> Self {
        let mut sandbox = Sandbox::default();
        sandbox.scale(3.0);

        Self {
            sandbox: sandbox,
            graph: graph,
            style: style,

            show_settings: false,
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

            if self.show_settings {
                Window::new("Settings")
                    .collapsible(false)
                    .resizable(false)
                    .show(ui.ctx(), |ui| windows::settings_window(self, ui));
            }
        });

        CentralPanel::default().show(ctx, |ui| {
            self.sandbox.update_screen_rect(ui.max_rect());

            let response = ui.interact(
                ui.max_rect(),
                Id::new("sandbox_clickable"),
                Sense::click_and_drag(),
            );

            if !Popup::is_any_open(ui.ctx()) {
                if response.clicked()
                    && let Some(coords) = response.interact_pointer_pos()
                {
                    self.sandbox
                        .create_vertex(coords.to_vec2(), &mut self.graph);
                }

                if response.dragged() {
                    self.sandbox.center -= self
                        .sandbox
                        .screen_dist_to_sandbox_dist(response.drag_delta());
                }

                self.sandbox.scale(
                    (1.0 + self.style.scroll_sensitivity).powf(
                        -ui.ctx()
                            .input(|input| input.smooth_scroll_delta)
                            .y
                            .clamp(-10.0, 10.0),
                    ),
                );
            }

            response.context_menu(|ui| {
                self.sandbox
                    .context_menu(ui, response.interact_pointer_pos(), &mut self.graph)
            });

            let graph_list = self.sandbox.draw_graph(ui, &self.graph, &self.style);
            for (vertex_response, vertex_id) in graph_list.0 {
                graph_interaction::handle_vertex_response(self, ui, vertex_id, vertex_response);
            }

            if self.graph.layout_config.run_per_update {
                layout::reapply(&mut self.graph);
                ctx.request_repaint();
            }
        });
    }
}
