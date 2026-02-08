use eframe::egui::{InnerResponse, Ui, ViewportCommand};

use crate::{app::GraspAppHandler, graph::layout};

pub fn file_menu(app: &mut GraspAppHandler, ui: &mut Ui) -> InnerResponse<Option<()>> {
    ui.menu_button("File", |ui| {
        if ui.button("New File").clicked() {
            ui.close();
        }

        ui.separator();

        if ui.button("Open File").clicked() {
            ui.close();
        }

        ui.separator();

        if ui.button("Save").clicked() {
            ui.close();
        }

        if ui.button("Save As").clicked() {
            ui.close();
        }

        ui.separator();

        if ui.button("Settings").clicked() {
            ui.close();
        }

        ui.separator();

        if ui.button("Exit").clicked() {
            ui.ctx().send_viewport_cmd(ViewportCommand::Close);
        }
    })
}

pub fn edit_menu(app: &mut GraspAppHandler, ui: &mut Ui) -> InnerResponse<Option<()>> {
    ui.menu_button("Edit", |ui| {
        if app.graph.directed && ui.button("Set Undirected").clicked() {
            app.graph.directed = false;
        } else if !app.graph.directed && ui.button("Set Directed").clicked() {
            app.graph.directed = true;
        }

        ui.menu_button("Layout", |ui| {
            ui.menu_button("Switch Layout", |ui| {
                if ui.button("Fruchterman & Reingold").clicked() {
                    app.graph.layout_config.layout_type = layout::LayoutType::FruchtermanReingold;
                }
            });

            if !app.graph.layout_config.run_per_update {
                if ui.button("Apply (Restart)").clicked() {
                    layout::apply(&mut app.graph);
                }

                if ui.button("Apply (Step)").clicked() {
                    layout::reapply(&mut app.graph);
                }
            }

            if !app.graph.layout_config.run_per_update && ui.button("Run Continously").clicked() {
                app.graph.layout_config.run_per_update = true;
            } else if app.graph.layout_config.run_per_update && ui.button("Stop Running").clicked()
            {
                app.graph.layout_config.run_per_update = false;
            }
        });
    })
}

pub fn view_menu(app: &mut GraspAppHandler, ui: &mut Ui) -> InnerResponse<Option<()>> {
    ui.menu_button("View", |ui| {
        if ui.button("TODO").clicked() {
            ui.close();
        }
    })
}

pub fn tool_menu(app: &mut GraspAppHandler, ui: &mut Ui) -> InnerResponse<Option<()>> {
    ui.menu_button("Tools", |ui| {
        if ui.button("TODO").clicked() {
            ui.close();
        }
    })
}
