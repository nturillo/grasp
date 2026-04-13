use eframe::egui::{Popup, Response, Ui};
use grasp::graph::{EdgeID, VertexID};

use crate::{
    frame::sandbox::Sandbox, graph::storage::Graph
};

fn vertex_primary_click(
    graph: &mut Graph,
    ui: &mut Ui,
    vertex_id: usize,
) {
    let is_selected = graph.selected_list.contains(&vertex_id);
    let single_selection = graph.selected_list.len() == 1;
    let ctrl = ui.input(|input| input.modifiers.ctrl);

    if ui.input(|input| input.modifiers.shift) && !graph.selected_list.is_empty() && !is_selected {
        graph.selected_list.iter().copied().collect::<Vec<VertexID>>().into_iter().for_each(|v| {
            let pair = (v, vertex_id);
            if graph.has_edge(pair) {graph.remove_edge(pair);} else {graph.create_edge(pair);}
        });
    } else {
        match (is_selected, ctrl, single_selection) {
            (false, false, _) | (true, false, false) => graph.selected_list = vec![vertex_id],
            (false, true, _) => graph.selected_list.push(vertex_id),
            (true, false, true) => graph.selected_list = vec![],
            (true, true, _) => graph.selected_list.retain(|&id| id != vertex_id),
        }
    }
}

fn vertex_dragged(graph: &mut Graph, sandbox: &Sandbox, vertex_id: usize, response: &Response) {
    if graph.layout_config.run_per_update {
        match &mut graph.layout_config.partial_data {
            crate::graph::layout::PartialLayout::None => (),
            crate::graph::layout::PartialLayout::FruchtermanReingold(temp) => *temp = temp.max(graph.layout_config.min_temperature_on_drag),
        }
    }

    graph
        .vertex_labels
        .get_mut(&vertex_id)
        .expect("Unexpected error: Interacted with vertex that does not exist.")
        .center = sandbox.screen_to_sandbox(response.interact_pointer_pos().unwrap().to_vec2());
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

pub fn vertex_context(graph: &mut Graph, ui: &mut Ui, vertex_id: &usize, open_vertex_input: &mut Option<VertexID>, open_edge_input: &mut Option<EdgeID>, label: &mut String) {
    let maybe_pair = vertex_context_try_get_pair(graph, vertex_id);

    if let Some(mut vertex_pair) = maybe_pair {
        let is_edge = graph.has_edge(vertex_pair);
        if is_edge && !graph.directed {vertex_pair = graph.verify_and_get_undirected_edge(vertex_pair).unwrap()}

        if !is_edge && ui.button("Connect").clicked() {
            graph.create_edge(vertex_pair);
        } else if is_edge && ui.button("Disconnect").clicked() {
            graph.remove_edge(vertex_pair);
        }

        ui.separator();

        if is_edge && graph.is_labeled && ui.button("Edit Edge Data").clicked() {
            *open_edge_input = Some(vertex_pair);
            *label = graph.edge_labels.get(&vertex_pair).unwrap().data.clone().unwrap_or_default();

            ui.separator();
        }
    }

    if ui.button("Remove Vertex").clicked() {
        graph.remove_vertex(*vertex_id);
    } else if graph.selected_list.len() > 1
        && graph.selected_list.contains(vertex_id)
        && ui.button("Remove Selection").clicked()
    {
        graph.remove_selected();
    }

    if graph.is_labeled {
        ui.separator();

        if ui.button("Edit Vertex Data").clicked() {
            *open_vertex_input = Some(*vertex_id);
            *label = graph.vertex_labels.get(vertex_id).unwrap().data.clone().unwrap_or_default();
        }
    }
}

pub fn handle_vertex_response(
    graph: &mut Graph,
    sandbox: &Sandbox,
    ui: &mut Ui,
    vertex_id: VertexID,
    dragged_vertex: &mut Option<VertexID>,
    response: &Response,
) {
    if let Some(_) = graph.vertex_labels.get(&vertex_id) {
        if !Popup::is_any_open(ui.ctx()) {
            if response.clicked() {
                vertex_primary_click(graph, ui, vertex_id);
            }

            if response.drag_started() {
                graph.layout_config.common_partial_data.vertex_locks.push(vertex_id);
                *dragged_vertex = Some(vertex_id);
            }

            if response.dragged() {
                vertex_dragged(graph, sandbox, dragged_vertex.unwrap_or(vertex_id), &response);
            }

            if response.drag_stopped() {
                graph.layout_config.common_partial_data.vertex_locks.retain(|&v| v != vertex_id);
                *dragged_vertex = None;
            }
        }
    }
}
