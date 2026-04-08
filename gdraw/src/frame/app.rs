use crate::{
    frame::{
        function_window::FunctionWindow, graph_interaction::{handle_vertex_response, vertex_context}, header, sandbox::{self, Sandbox}, style::Style, windows
    },
    graph::{
        layout::{self, LayoutConfig},
        storage::Graph,
    },
};
use eframe::{egui::{
    self, CentralPanel, Color32, Context, Id, MenuBar, Popup, Rect, Sense, Stroke, TopBottomPanel, Vec2, Window
}};
use grasp::graph::{GraphTrait, VertexID, prelude::{SparseDiGraph}, set::Set};

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
                    self.style.clone(),
                )))
            }),
        )
    }

    /// Loads a graph from anything that implements [`grasp::graph::graph_traits::GraphTrait`]
    pub fn load<T: GraphTrait + Default>(&mut self, graph: &T) {
        self.graph = Graph::from(graph);
        layout::apply(&mut self.graph);
    }

    /// Create a new [`crate::frame::app::GraspApp`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the visualizer's graph layout.
    ///
    /// Layouts are located at [`crate::graph::layout::LayoutType`]
    pub fn set_layout_config(&mut self, config: LayoutConfig) {
        self.graph.layout_config = config;
    }

    /// Highlight a set of vertices.
    pub fn highlight_set<S: Set<Item = VertexID>>(&mut self, set: &S, color: Color32) {
        self.graph.highlight_set(set, color);
    }

    /// Returns a copy of the [`grasp::graph::adjacency_list::SparseDiGraph`] underlying the visualizer.
    pub fn as_sparse_digraph(&mut self) -> SparseDiGraph {
        self.graph.clone().base
    }
}

pub(crate) struct GraspAppHandler<'a> {
    pub sandbox: sandbox::Sandbox,
    pub graph: &'a mut Graph,
    pub style: Style,

    pub show_settings: bool,
    pub show_metrics: bool,
    pub func_window: FunctionWindow,
    pub vertex_focused: Option<VertexID>,
}

impl<'a> GraspAppHandler<'a> {
    fn new(_cc: &eframe::CreationContext<'_>, graph: &'a mut Graph, style: Style) -> Self {
        let sandbox = Sandbox::default();

        Self {
            sandbox: sandbox,
            graph: graph,
            style: style,

            show_settings: false,
            show_metrics: false,
            func_window: Default::default(),

            vertex_focused: None,
        }
    }
}

impl<'a> eframe::App for GraspAppHandler<'a> {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top(Id::new("menu_header")).show(ctx, |ui| {
            MenuBar::new().ui(ui, |ui| {
                header::file_menu(self, ui);
                header::edit_menu(self, ui);
                header::view_menu(self, ui);
                header::tool_menu(self, ui);
            });
        });

        if self.show_settings {
            Window::new("Settings")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| windows::settings_window(self, ui));
        }

        if self.show_metrics {
            Window::new("Metrics")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| windows::metrics_window(self, ui));
        }

        if self.func_window.visible {
            self.func_window.show(self.graph, &self.style, ctx);
        }

        CentralPanel::default().show(ctx, |ui| {
            self.sandbox.update_screen_rect(ui.max_rect());

            let response = ui.interact(
                ui.max_rect(),
                Id::new("sandbox_clickable"),
                Sense::click_and_drag(),
            );

            self.sandbox.draw_graph(ui, &self.graph, &self.style);

            if !Popup::is_any_open(ui.ctx()) {
                let mapped: Vec<(usize, Vec2)> = self.graph.vertex_labels.iter().map(|(id, v)| (*id, v.center)).collect();
                if response.dragged() && self.vertex_focused.is_some() {
                    handle_vertex_response(self.graph, &self.sandbox, ui, self.vertex_focused.unwrap(), &response);
                }
                if !(response.dragged() && self.vertex_focused.is_none()) && let Some((id, _)) = mapped.iter().rev().find(|(_, data)| if let Some(pos) = ctx.input(|input| input.pointer.hover_pos()) && (pos.to_vec2() - self.sandbox.sandbox_to_screen(*data)).length() <= (self.style.vertex_radius + self.style.outline_thickness) {true} else {false}) {
                    handle_vertex_response(self.graph, &self.sandbox, ui, *id, &response);
                    self.vertex_focused = Some(*id);
                } else if !(response.dragged() && self.vertex_focused.is_some()) {
                    self.vertex_focused = None;

                    if response.clicked()
                        && let Some(coords) = response.interact_pointer_pos()
                    {
                        self.sandbox
                            .create_vertex(coords.to_vec2(), &mut self.graph);
                        self.graph.selected_list = vec![self.graph.vertex_id];
                    }
                    else if !ui.input(|input| input.modifiers.shift) && response.dragged() {
                        self.sandbox.center -= self
                            .sandbox
                            .screen_dist_to_sandbox_dist(response.drag_delta());
                    }
                    else if ui.input(|input| input.modifiers.shift) && response.dragged() {
                        let origin = response.interact_pointer_pos().unwrap();
                        let rect = Rect::from_two_pos((origin.to_vec2() - response.total_drag_delta().unwrap()).to_pos2(), origin);
                        ui.painter().rect(rect, 0.0, Color32::from_rgba_unmultiplied(0xAD, 0xD8, 0xE6, 110), Stroke::new(1.0, Color32::from_rgb(80, 150, 255)), egui::StrokeKind::Inside);

                        let ex_rect = rect.expand((self.style.vertex_radius + self.style.outline_thickness) / self.sandbox.scale);
                        for (id, data) in &self.graph.vertex_labels {
                            let pos = self.graph.selected_list.iter().position(|v| v == id);
                            let contained = ex_rect.contains(self.sandbox.sandbox_to_screen(data.center).to_pos2());
                            if contained && pos.is_none() {
                                self.graph.selected_list.push(*id);
                            } else if !contained && pos.is_some() {
                                self.graph.selected_list.remove(pos.unwrap());
                            }
                        }
                    }
                }
            }

            self.sandbox.scale *= (1.0 + self.style.scroll_sensitivity).powf(
                            -1.0 * ui.ctx()
                                .input(|input| input.smooth_scroll_delta)
                                .y
                                .clamp(-10.0, 10.0),
                        );

            if let Some(id) = self.vertex_focused {
                response.context_menu(|ui| vertex_context(self.graph, ui, &id));
                response.on_hover_text_at_pointer(format!("id: {}", id));
            } else {
                response.context_menu(|ui| {
                    self.sandbox
                        .context_menu(ui, response.interact_pointer_pos(), &mut self.graph)
                });
            }

            if self.graph.layout_config.run_per_update {
                layout::reapply(&mut self.graph);
                ctx.request_repaint();
            }
        });
    }
}
