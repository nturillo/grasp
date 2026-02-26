use eframe::egui::Color32;
use gdraw::app::GraspApp;
use grasp::graph::{adjacency_list::SparseSimpleGraph, graph_ops::GraphOps, GraphTrait};

#[allow(unused)]
fn main() {
    let mut butterfly = SparseSimpleGraph::default();
    butterfly.add_edge((1, 2));
    butterfly.add_edge((2, 3));
    butterfly.add_edge((1, 3));
    butterfly.add_edge((1, 4));
    butterfly.add_edge((1, 5));
    butterfly.add_edge((4, 5));
    butterfly.add_edge((5, 6));
    butterfly.add_edge((2, 5));
    butterfly.add_edge((6, 7));
    butterfly.add_edge((7, 8));
    butterfly.add_edge((6, 8));
    butterfly.add_edge((6, 9));
    butterfly.add_edge((9, 8));
    butterfly.add_edge((9, 1));
    butterfly.add_edge((9, 10));

    let mut app = GraspApp::new();

    app.load(&butterfly);
    app.highlight_set(butterfly.neighbors(1).expect(""), Color32::RED);

    let _ = app.start();

    println!("App closed, now main continues");
}
