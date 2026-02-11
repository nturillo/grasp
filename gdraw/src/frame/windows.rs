use std::ops::RangeInclusive;

use eframe::egui::{self, Ui, widgets};

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

        if app.style.show_vertices {
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
                    ui.label("Highlight Color");
                    ui.color_edit_button_srgba(&mut app.style.highlight_color);
                });

                ui.horizontal(|ui| {
                    ui.label("Highlight Strength");
                    ui.add(
                        widgets::Slider::new(
                            &mut app.style.highlight_strength,
                            RangeInclusive::new(0.0, 1.0),
                        )
                        .clamping(widgets::SliderClamping::Always),
                    )
                });
            });
        }

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
        });
    });

    ui.vertical_centered(|ui| {
        if ui.button("Close").clicked() {
            app.show_settings = false;
        }
    });
}
