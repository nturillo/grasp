use eframe::egui::{InnerResponse, Ui};

pub fn file_menu(ui: &mut Ui) -> InnerResponse<Option<()>> {
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

pub fn edit_menu(ui: &mut Ui) -> InnerResponse<Option<()>> {
    ui.menu_button("Edit", |ui| {
        if ui.button("TODO").clicked() {
            ui.close();
        }
    })
}

pub fn view_menu(ui: &mut Ui) -> InnerResponse<Option<()>> {
    ui.menu_button("View", |ui| {
        if ui.button("TODO").clicked() {
            ui.close();
        }
    })
}

pub fn tool_menu(ui: &mut Ui) -> InnerResponse<Option<()>> {
    ui.menu_button("Tools", |ui| {
        if ui.button("TODO").clicked() {
            ui.close();
        }
    })
}
