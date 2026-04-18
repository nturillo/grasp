use eframe::egui::Vec2;
use grasp::{algorithms::planarity::GraphPlanarity, graph::prelude::*};
use gdraw::app::GraspApp;

fn main() -> Result<(), Box<dyn std::error::Error>>{
    // Create a pentagon
    let mut graph = SparseSimpleGraph::default();
    graph.add_edge((0, 1));
    graph.add_edge((0, 2));
    graph.add_edge((0, 3));
    graph.add_edge((0, 4));
    graph.add_edge((1, 2));
    graph.add_edge((1, 3));
    graph.add_edge((1, 4));
    graph.add_edge((2, 3));
    graph.add_edge((2, 4));

    let n = 4;
    for i in 0..(n-1)  {
        let u_a = 5+i; let u_b = 5+i+n;
        let v_a = 5+i+1; let v_b = 5+i+1+n;
        graph.add_edge((u_a, u_b));
        graph.add_edge((u_a, v_a));
        graph.add_edge((u_b, v_b));
        graph.add_edge((u_a, v_b));
    }
    graph.add_edge((n+4, 5));
    graph.add_edge((2*n+4, n+5));
    graph.add_edge((n+4, 2*n+4));
    graph.add_edge((n+4, n+5));

    let mut planarity = GraphPlanarity::from_graph(&graph);
    assert!(planarity.compute_planarity());
    let result = planarity.get_planarity_structure();
    assert!(result.is_ok());

    let mut embedding = result.unwrap();

    let positions = embedding.calculate_euclidean_embedding();

    let mut app = GraspApp::new();
    app.load(&graph);
    for (id, vertex) in app.graph.vertex_labels.iter_mut() {
        let (x, y) = positions.get(id).cloned().unwrap();
        vertex.center = Vec2 { x, y };
    }
    app.start()?;
    Ok(())
}