use gdraw::app::GraspApp;
use grasp::graph::prelude::*;

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

    type TestGraph = HashMapLabeledSimpleGraph<SparseSimpleGraph, String, f32>;
    let mut triangle = TestGraph::default();
        triangle.add_edge((1, 0));triangle.add_edge((2, 1));triangle.add_edge((0, 2));
        triangle.set_edge_labels([((1, 0), 1.5), ((2, 1), 2.0), ((0, 2), 3.0)]);
        triangle.set_vertex_labels([(0, "A".to_string()), (1, "B".to_string()), (2, "C".to_string())]);

    app.load_labeled(&triangle);
    let _ = app.start();

    println!("App closed, now main continues");
}
