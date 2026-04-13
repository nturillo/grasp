use std::ops::RangeInclusive;

use eframe::egui::{self, Align, Color32, Layout, Ui, widgets};

use crate::{app::GraspAppHandler, graph::layout::LayoutType};

pub fn settings_window(app: &mut GraspAppHandler, ui: &mut Ui) {
    ui.collapsing("Control", |ui| {
        ui.horizontal(|ui| {
            ui.label("Scroll Speed");
            ui.add(
                widgets::Slider::new(
                    &mut app.style.scroll_sensitivity,
                    RangeInclusive::new(0.001, 0.05),
                )
                .clamping(widgets::SliderClamping::Always)
                .fixed_decimals(3),
            )
        });
    });

    ui.collapsing("Visuals", |ui| {
        ui.horizontal(|ui| {
            ui.label("Show Vertices");
            ui.checkbox(&mut app.style.show_vertices, "");
        });

        ui.horizontal(|ui| {
            ui.label("Show Edge Data");
            ui.checkbox(&mut app.style.display_edge_data, "");
        });

        if app.style.show_vertices {
            ui.collapsing("Vertex Labels", |ui| {
                if ui.checkbox(&mut app.style.display_ids, "Show IDs").clicked() && app.style.display_ids {
                    app.style.display_vertex_data = false;
                }

                if ui.checkbox(&mut app.style.display_vertex_data, "Show Data").clicked() && app.style.display_vertex_data {
                    app.style.display_ids = false;
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Vertex Radius");
                ui.add(
                    widgets::Slider::new(
                        &mut app.style.vertex_radius,
                        RangeInclusive::new(1.0, 50.0),
                    )
                    .clamping(widgets::SliderClamping::Always),
                )
            });

            ui.horizontal(|ui| {
                ui.label("Outline Thickness");
                ui.add(
                    widgets::Slider::new(
                        &mut app.style.outline_thickness,
                        RangeInclusive::new(0.0, 10.0),
                    )
                    .clamping(widgets::SliderClamping::Always),
                )
            });

            ui.collapsing("Vertex Colors", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Vertex Color");
                    ui.color_edit_button_srgba(&mut app.style.vertex_color);
                });

                ui.horizontal(|ui| {
                    ui.label("Outline Color");
                    ui.color_edit_button_srgba(&mut app.style.outline_color);
                });

                ui.horizontal(|ui| {
                    ui.label("Selection Color");
                    ui.color_edit_button_srgba(&mut app.style.select_color);
                });

                ui.horizontal(|ui| {
                    ui.label("Selection Color Strength");
                    ui.add(
                        widgets::Slider::new(
                            &mut app.style.select_color_strength,
                            RangeInclusive::new(0.0, 1.0),
                        )
                        .clamping(widgets::SliderClamping::Always),
                    )
                });

                ui.horizontal(|ui| {
                    ui.label("Highlight Color");
                    ui.color_edit_button_srgba(&mut app.style.highlight_color);
                });

                ui.horizontal(|ui| {
                    ui.label("Cluster Colors");
                    ui.vertical(|ui| {
                        for color in app.style.cluster_colors.iter_mut(){
                            ui.color_edit_button_srgba(color);
                        }

                        ui.horizontal(|ui| {
                            if ui.button("+").clicked() {
                                app.style.cluster_colors.push(Color32::WHITE);
                            }

                            if app.style.cluster_colors.len() > 2 && ui.button("-").clicked() {
                                app.style.cluster_colors.pop();
                            }
                        });
                    });
                });
            });
        }

        ui.collapsing("Edge Colors", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Edge Color");
                    ui.color_edit_button_srgba(&mut app.style.edge_color);
                });

                ui.horizontal(|ui| {
                    ui.label("Edge Highlight Color");
                    ui.color_edit_button_srgba(&mut app.style.edge_highlight_color);
                });
            });

        ui.horizontal(|ui| {
            ui.label("Edge Thickness");
            ui.add(
                widgets::Slider::new(
                    &mut app.style.edge_thickness,
                    RangeInclusive::new(1.0, 10.0),
                )
                .clamping(widgets::SliderClamping::Always),
            )
        });

        ui.horizontal(|ui| {
            ui.label("Arrow Size");
            ui.add(
                widgets::Slider::new(&mut app.style.arrow_size, RangeInclusive::new(10.0, 100.0))
                    .clamping(widgets::SliderClamping::Always),
            )
        });
    });

    ui.collapsing("Layout", |ui| {
        ui.horizontal(|ui| {
            ui.label("Layout Model");
            egui::ComboBox::from_label("")
                .selected_text(format!("{}", app.graph.layout_config.layout_type))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut app.graph.layout_config.layout_type,
                        LayoutType::FruchtermanReingold,
                        "Fruchterman-Reingold",
                    )
                })
        });

        ui.horizontal(|ui| {
            ui.label("Iterations");
            ui.add(
                widgets::Slider::new(
                    &mut app.graph.layout_config.iterations,
                    RangeInclusive::new(1, 10000),
                )
                .clamping(widgets::SliderClamping::Always),
            )
        });

        ui.horizontal(|ui| {
            ui.label("Iterations (Realtime)");
            ui.add(
                widgets::Slider::new(
                    &mut app.graph.layout_config.iterations_per_update,
                    RangeInclusive::new(1, 1000),
                )
                .clamping(widgets::SliderClamping::Always),
            )
        });

        ui.collapsing("Fruchterman-Reingold", |ui| {
            ui.horizontal(|ui| {
                ui.label("Temperature Decay Factor");
                ui.add(
                    widgets::Slider::new(
                        &mut app.graph.layout_config.temperature_decay_factor,
                        RangeInclusive::new(0.0, 1.0),
                    )
                    .clamping(widgets::SliderClamping::Always)
                    .fixed_decimals(4),
                )
            });

            ui.horizontal(|ui| {
                ui.label("Temperature Decay Factor (Realtime)");
                ui.add(
                    widgets::Slider::new(
                        &mut app.graph.layout_config.temperature_decay_factor_per_update,
                        RangeInclusive::new(0.0, 1.0),
                    )
                    .clamping(widgets::SliderClamping::Always)
                    .fixed_decimals(4),
                )
            });

            ui.horizontal(|ui| {
                ui.label("Minimum Temperature on Drag");
                ui.add(
                    widgets::Slider::new(
                        &mut app.graph.layout_config.min_temperature_on_drag,
                        RangeInclusive::new(0.0001, 0.001),
                    )
                    .clamping(widgets::SliderClamping::Always)
                    .fixed_decimals(5),
                )
            });
        });
    });

    ui.vertical_centered(|ui| {
        if ui.button("Close").clicked() {
            app.show_settings = false;
        }
    });
}

pub fn metrics_window(app: &mut GraspAppHandler, ui: &mut Ui) {
    let v_count = app.graph.vertex_count();
    let e_count = if app.graph.directed {app.graph.edge_count()} else {app.graph.simple_edge_count()};
    
    ui.label(format!("Vertices: {}", v_count));
    ui.label(format!("Edges: {}", e_count));

    ui.separator();

    ui.label(format!("Directed: {}", app.graph.directed));

    ui.separator();

    if v_count > 1 {
        let density = if app.graph.directed {1} else {2} as f32 * e_count as f32 / (v_count * (v_count - 1)) as f32;
        ui.label(format!("Density: {}", density));
    } else {
        ui.label("Density: N/A");
    }

    ui.vertical_centered(|ui| {
        if ui.button("Close").clicked() {
            app.show_metrics = false;
        }
    });
}

pub fn vertex_input_window(app: &mut GraspAppHandler, ui: &mut Ui) {
    ui.text_edit_singleline(&mut app.label);
    
    ui.horizontal(|ui| {
        if ui.button("Cancel").clicked() {
            app.show_vertex_input = None;
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.button("Apply").clicked() && let Some(v) = app.show_vertex_input {
                app.graph.vertex_labels.get_mut(&v).unwrap().data = Some(app.label.clone());
                app.show_vertex_input = None;
            }
        });
    });
}

pub fn edge_input_window(app: &mut GraspAppHandler, ui: &mut Ui) {
    ui.text_edit_singleline(&mut app.label);
    
    ui.horizontal(|ui| {
        if ui.button("Cancel").clicked() {
            app.show_edge_input = None;
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.button("Apply").clicked() && let Some(e) = app.show_edge_input {
                app.graph.edge_labels.get_mut(&e).unwrap().data = Some(app.label.clone());
                app.show_edge_input = None;
            }
        });
    });
}