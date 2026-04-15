use std::f32::EPSILON;

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

    let n = 3;
    for i in 0..(n-1)  {
        let u_a = 5+i; let u_b = 5+i+n;
        let v_a = 5+i+1; let v_b = 5+i+1+n;
        graph.add_edge((u_a, u_b));
        graph.add_edge((u_a, v_a));
        graph.add_edge((u_b, v_b));
        //graph.add_edge((u_a, v_b));
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

    let edges: Vec<EdgeID> = graph.edges().collect();
    for i in 0..edges.len()-1 {
        for j in i+1..edges.len(){
            let (v_a, v_b) = edges[i];
            let (u_a, u_b) = edges[j];
            let (x1, y1) = positions[&v_a];
            let (x2, y2) = positions[&v_b];
            let (x3, y3) = positions[&u_a];
            let (x4, y4) = positions[&u_b];
            let denom = (x1-x2)*(y3-y4) - (y1-y2)*(x3-x4);
            if denom == 0.0 {
                // Cant intersect lines, unsure non colinear, or non overlapping
                //println!("Edge ({}, {})->({}, {}), and ({}, {})->({}, {}), are vertical", x1, y1, x2, y2, x3, y3, x4, y4);
                if x1!=x3 {continue;}
                let (a1, b1) = (y1.min(y2), y1.max(y2));
                let (a2, b2) = (y3.min(y4), y3.max(y4));
                let start = a1.max(b1); let end = a2.min(b2);
                assert!(end-start < EPSILON);
                continue;
            }
            let x = ((x1*y2-y1*x2)*(x3-x4)-(x1-x2)*(x3*y4-y3*x4)) / denom;
            let y = ((x1*y2-y1*x2)*(y3-y4)-(y1-y2)*(x3*y4-y3*x4)) / denom;
            //println!("Edge ({}, {})->({}, {}), and ({}, {})->({}, {}), intersect at ({}, {})", x1, y1, x2, y2, x3, y3, x4, y4, x, y);
            // Insure intersection occurs out of one of the segment bounds
            assert!(
                x-x1.max(x2)>-10.0*EPSILON || x - x1.min(x2)<10.0*EPSILON ||
                x-x3.max(x4)>-10.0*EPSILON || x - x3.min(x4)<10.0*EPSILON ||
                y-y1.max(y2)>-10.0*EPSILON || y - y1.min(y2)<10.0*EPSILON ||
                y-y3.max(y4)>-10.0*EPSILON || y - y3.min(y4)<10.0*EPSILON
            );
        }
    }
    Ok(())
}