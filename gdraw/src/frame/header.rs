use eframe::egui::{InnerResponse, Ui};

use crate::app::GraspApp;

pub fn file_menu(app: &mut GraspApp, ui: &mut Ui) -> InnerResponse<Option<()>> {
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
            ui.close();
        }
    })
}

pub fn edit_menu(app: &mut GraspApp, ui: &mut Ui) -> InnerResponse<Option<()>> {
    ui.menu_button("Edit", |ui| {
        if app.graph.directed && ui.button("Set Undirected").clicked() {
            app.graph.directed = false;
        } else if !app.graph.directed && ui.button("Set Directed").clicked() {
            app.graph.directed = true;
        }

        if ui.button("TODO").clicked() {
            ui.close();
        }
    })
}

pub fn view_menu(app: &mut GraspApp, ui: &mut Ui) -> InnerResponse<Option<()>> {
    ui.menu_button("View", |ui| {
        if ui.button("TODO").clicked() {
            ui.close();
        }
    })
}

pub fn tool_menu(app: &mut GraspApp, ui: &mut Ui) -> InnerResponse<Option<()>> {
    ui.menu_button("Tools", |ui| {
        if ui.button("TODO").clicked() {
            ui.close();
        }
    })
}
