use std::collections::{HashSet};

use eframe::egui::{Align, Context, DragValue, Layout, Window};
use grasp::{algorithms::registry::{ArgType, FunctionData, ReturnType}, graph::{EdgeID, VertexID}};

use crate::{frame::style::Style, graph::storage::Graph};

#[derive(Default)]
pub struct FunctionWindow {
    pub visible: bool,
    pub metadata: Option<&'static FunctionData>,

    args: Vec<ArgType>,
    string_result: Option<String>,
}

impl FunctionWindow {
    pub fn open(&mut self, metadata: &'static FunctionData) {
        self.args = vec![];
        self.metadata = Some(metadata);
        self.visible = true;
        self.string_result = None;

        for [_, ty] in metadata.param_data {
            self.args.push(match *ty {
                "Integer" => ArgType::Integer(0),
                "Float" => ArgType::Float(0.0),
                "Unsigned" => ArgType::UnsignedInteger(0),
                "Boolean" => ArgType::Boolean(false),
                "Vertex" => ArgType::Vertex(0),
                "Edge" => ArgType::Edge((0, 0)),
                "VertexList" => ArgType::VertexList(vec![]),
                "EdgeList" => ArgType::EdgeList(vec![]),
                "String" => ArgType::String(String::new()),
                _ => panic!("Encountered broken function"),
            });
        }
    }

    pub fn show(&mut self, graph: &mut Graph, style: &Style, ctx: &Context) {
        let metadata = match &self.metadata {
            None => { return; },
            Some(m) => m,
        };

        Window::new(metadata.name)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                if !metadata.desc.is_empty() {
                    ui.label(metadata.desc);
                    ui.add_space(20.0);
                }

                for (i, [name, ty]) in metadata.param_data.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}: ", *name));
                        ui.label(*ty);

                        match self.args.get_mut(i).unwrap() {
                            ArgType::Integer(num) => { ui.add(DragValue::new(num)); },
                            ArgType::UnsignedInteger(num) => { ui.add(DragValue::new(num).range(0..=u64::MAX)); },
                            ArgType::Float(num) => { ui.add(DragValue::new(num)); },
                            ArgType::Boolean(n) => { ui.checkbox(n, ""); },
                            ArgType::String(msg) => { ui.text_edit_singleline(msg); },
                            ArgType::Vertex(id) => { ui.add(DragValue::new(id).range(0..=usize::MAX)); },
                            ArgType::Edge((source, target)) => { ui.add(DragValue::new(source).range(0..=usize::MAX)); ui.add(DragValue::new(target).range(0..=usize::MAX)); },
                            ArgType::VertexList(verts) => {
                                ui.vertical(|ui| {
                                    for id in verts.iter_mut() {
                                        ui.add(DragValue::new(id).range(0..=usize::MAX));
                                    }

                                    ui.horizontal(|ui| {
                                        if ui.button("+").clicked() {
                                            verts.push(0);
                                        }

                                        if !verts.is_empty() && ui.button("-").clicked() {
                                            verts.pop();
                                        }
                                    });
                                });
                            },
                            ArgType::EdgeList(edges) => {
                                ui.vertical(|ui| {
                                    for (source, target) in edges.iter_mut() {
                                        ui.horizontal(|ui| {
                                            ui.add(DragValue::new(source).range(0..=usize::MAX)); ui.add(DragValue::new(target).range(0..=usize::MAX));
                                        });
                                    }

                                    ui.horizontal(|ui| {
                                        if ui.button("+").clicked() {
                                            edges.push((0, 0));
                                        }

                                        if !edges.is_empty() && ui.button("-").clicked() {
                                            edges.pop();
                                        }
                                    });
                                });
                            },
                        }
                    });
                }
                ui.add_space(20.0);
                if metadata.return_type != "None" {
                    ui.horizontal(|ui| {
                        ui.label(format!("Returns a {}", metadata.return_type));
                    });
                }

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.visible = false;
                        self.metadata = None;
                        self.args = vec![];
                    }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("Run").clicked() {
                            let res = (self.metadata.unwrap().func)(&graph.base, &self.args);

                            match res {
                                ReturnType::String(s) => { self.string_result = Some(s); },
                                ReturnType::Vertex(v) => { graph.clear_highlights(); graph.highlight_set(&HashSet::from([v]), style.highlight_color);},
                                ReturnType::VertexList(verts) => { graph.clear_highlights(); graph.highlight_set(&verts.iter().cloned().collect::<HashSet<VertexID>>(), style.highlight_color);},
                                ReturnType::Edge(e) => { graph.clear_highlights(); graph.highlight_edges(&HashSet::from([e]), style.highlight_color);},
                                ReturnType::EdgeList(edges) => { graph.clear_highlights(); graph.highlight_edges(&edges.iter().cloned().collect::<HashSet<EdgeID>>(), style.edge_highlight_color);},
                                _ => (),
                            }

                            if self.string_result.is_none() {
                                self.visible = false;
                                self.metadata = None;
                                self.args = vec![];
                            }
                        }
                    });
                });

            });

        if let Some(res) = &self.string_result {
            Window::new("Result")
                .open(&mut self.visible)
                .collapsible(false)
                .resizable(false).show(ctx, |ui| {
                    ui.label(res);
                });
        }
    }
}