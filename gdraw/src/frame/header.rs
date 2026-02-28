use std::{collections::BTreeMap, str::Split};

use eframe::egui::{InnerResponse, Ui, ViewportCommand};
use grasp::algorithms::registry::{ALGORITHMS, FunctionData};

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
            app.show_settings = true;
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
                if ui.button("Fruchterman-Reingold").clicked() {
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
                app.graph.layout_config.partial_data = layout::PartialLayout::None;
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
    struct ModMap {
        value: Option<&'static FunctionData>,
        map: BTreeMap<String, ModMap>
    }

    impl ModMap {
        fn new() -> Self {
            Self {
                value: None,
                map: BTreeMap::new()
            }
        }

        fn new_with(func: &'static FunctionData) -> Self {
            Self {
                value: Some(func),
                map: BTreeMap::new()
            }
        }
    }

    fn nest_funcs(map: &ModMap, app: &mut GraspAppHandler, ui: &mut Ui) {
        for (key, val) in &map.map {
            match val.value {
                Some(func) => {
                    let button = ui.button(func.name);

                    if !func.desc.is_empty() {
                        if button.on_hover_text_at_pointer(func.desc).clicked() {
                            app.func_window.open(func);
                        }
                    } else {
                        if button.clicked() {
                            app.func_window.open(func);
                        }
                    }
                },
                None => { ui.menu_button(key, |ui| nest_funcs(val, app, ui)); },
            }
        }
    }

    ui.menu_button("Tools", |ui| {
        ui.menu_button("Functions", |ui| {
            let mut map = ModMap::new();

            for func in ALGORITHMS {
                let path = match func.module.split_once("algorithms::") {
                    None => {continue;}
                    Some((_, path)) => path,
                }.split("::");

                let mut root = &mut map;
                for mod_name in path {
                    root = root.map.entry(mod_name.to_string()).or_insert(ModMap::new());
                }

                root.map.insert(func.name.to_string(), ModMap::new_with(func));
            }

            nest_funcs(&map, app, ui);
        });
    })
}
