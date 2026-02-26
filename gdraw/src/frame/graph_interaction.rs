use eframe::egui::{Popup, Response, Ui};
use grasp::graph::VertexID;

use crate::{
    app::GraspAppHandler,
    graph::storage::{Graph},
};

fn vertex_primary_click(
    app: &mut GraspAppHandler,
    ui: &mut Ui,
    vertex_id: usize,
) {
    let is_selected = app.graph.selected_list.contains(&vertex_id);
    let single_selection = app.graph.selected_list.len() == 1;
    let shift_held = ui.input(|input| input.modifiers.shift);

    match (is_selected, shift_held, single_selection) {
        (false, false, _) | (true, false, false) => app.graph.selected_list = vec![vertex_id],
        (false, true, _) => app.graph.selected_list.push(vertex_id),
        (true, false, true) => app.graph.selected_list = vec![],
        (true, true, _) => app.graph.selected_list.retain(|&id| id != vertex_id),
    }
}

fn vertex_dragged(app: &mut GraspAppHandler, vertex_id: usize, response: &Response) {
    app.graph
        .vertex_labels
        .get_mut(&vertex_id)
        .expect("Unexpected error: Interacted with vertex that does not exist.")
        .center += app
        .sandbox
        .screen_dist_to_sandbox_dist(response.drag_delta());
}

fn vertex_context_try_get_pair(graph: &mut Graph, vertex_id: &usize) -> Option<(VertexID, VertexID)> {
    let selected_len = graph.selected_list.len();

    if (selected_len == 1 && !graph.selected_list.contains(vertex_id))
        || (selected_len == 2 && graph.selected_list.contains(vertex_id))
    {
        let start_vertex = if selected_len == 1 {
            graph.selected_list[0]
        } else {
            *graph
                .selected_list
                .iter()
                .find(|vert| *vert != vertex_id)
                .expect("Unexpected error: Vertex selected twice")
        };

        Some((start_vertex, *vertex_id))
    } else {
        None
    }
}

fn vertex_context(app: &mut GraspAppHandler, ui: &mut Ui, vertex_id: &usize) {
    let maybe_pair = vertex_context_try_get_pair(&mut app.graph, vertex_id);

    if let Some(vertex_pair) = maybe_pair {
        let is_edge = app.graph.has_edge(vertex_pair);

        if !is_edge && ui.button("Connect").clicked() {
            app.graph.create_edge(vertex_pair);
        } else if is_edge && ui.button("Disconnect").clicked() {
            app.graph.remove_edge(vertex_pair);
        }

        ui.separator();
    }

    if ui.button("Remove Vertex").clicked() {
        app.graph.remove_vertex(*vertex_id);
    } else if app.graph.selected_list.len() > 1
        && app.graph.selected_list.contains(vertex_id)
        && ui.button("Remove Selection").clicked()
    {
        app.graph.remove_selected();
    }

    //ui.separator();
}

pub fn handle_vertex_response(
    app: &mut GraspAppHandler,
    ui: &mut Ui,
    vertex_id: usize,
    response: Response,
) {
    if let Some(_) = app.graph.vertex_labels.get(&vertex_id) {
        if !Popup::is_any_open(ui.ctx()) {
            if response.clicked() {
                vertex_primary_click(app, ui, vertex_id);
            }

            if response.dragged() && !app.graph.layout_config.run_per_update {
                vertex_dragged(app, vertex_id, &response);
            }
        }

        response.context_menu(|ui| vertex_context(app, ui, &vertex_id));
    }
}
