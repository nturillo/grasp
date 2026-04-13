use eframe::egui::Vec2;
use grasp::{algorithms::{gonality::compute_gonality, planarity::GraphPlanarity}, graph::prelude::*};
use gdraw::app::GraspApp;

fn hypercube(n: usize) -> SparseSimpleGraph {
    let vertex_count  = 2usize.pow(n as u32);
    let edge_count;
    if n > 0 {
        edge_count = n * 2usize.pow((n as u32) - 1);
    } else {
        edge_count = 0;
    }
    let mut g = SparseSimpleGraph::with_capacity(vertex_count, edge_count);
    for v in 0..vertex_count {
        g.add_vertex(v);
        for i in 0..n {
            let bit = 1 << i;
            let u = v ^ bit;
            g.add_edge((u, v));
        }
    }     
    g
}

fn main() -> Result<(), Box<dyn std::error::Error>>{
    let n = 4usize;
    for i in 0..n {
        println!("Calculating gonality for hypercube Q_{}...", i);
        let g = hypercube(i);
        let expected_gonality;
        if i > 0 {
            expected_gonality = 2usize.pow((i as u32)-1);
        } else {
            expected_gonality = 1;
        }
        let gonality = compute_gonality(&g).unwrap();
        if expected_gonality == gonality {
            println!("\tgonality is {}, as expected", gonality);
        } else {
            println!("\twoah!, we've disproven a conjecture. Gonality is {}", gonality);
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use grasp::assert_graphs_eq;

    use super::*;
    #[test]
    fn hypercube_test() {
        let mut g_expected= SparseSimpleGraph::empty();
        g_expected.add_edge((0, 1));
        g_expected.add_edge((0, 2));
        g_expected.add_edge((0, 4));
        g_expected.add_edge((1, 3));
        g_expected.add_edge((1, 5));
        g_expected.add_edge((2, 3));
        g_expected.add_edge((2, 6));
        g_expected.add_edge((3, 7));
        g_expected.add_edge((4, 5));
        g_expected.add_edge((4, 6));
        g_expected.add_edge((5, 7));
        g_expected.add_edge((6, 7));
        
        let g_test = hypercube(3);

        assert_graphs_eq!(g_test, g_expected);
    }
}