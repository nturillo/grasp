use std::{collections::HashMap, env};
use grasp::graph::{graph_ops::SubgraphView, permutation::{PermutationDiGraph, lehmer_from_natural, permutation_from_lehmer}, prelude::*};
use gdraw::app::GraspApp;

/// Calculates the number of additions to permutation 1 to get to permutation 2
fn permutation_distance(perm_1: &Vec<usize>, perm_2: &Vec<usize>) -> usize{
    assert!(perm_1.len() == perm_2.len());
    for i in 0..perm_1.len() {
        // test if perm_1[i..] == perm_2[..i]
        if perm_1[i..] == perm_2[..i] {
            return i;
        }
    }
    perm_1.len()
}

fn main() -> Result<(), Box<dyn std::error::Error>>{
    // read in permutation size
    let args: Vec<String> = env::args().collect();
    let permutation_size: usize = if let Some(param) = args.get(1) {
        param.parse::<usize>().unwrap_or(3)
    } else {3};
    // Create permutation graph
    let mut graph = HashMapLabeledDiGraph::<PermutationDiGraph, (), usize>::from_graph(PermutationDiGraph::new(permutation_size));
    // Make complete
    for u in 0..(graph.vertex_count()-1){
        for v in (u+1)..graph.vertex_count() {
            let _ = graph.try_add_edge((u, v)); let _ = graph.try_add_edge((v, u));
        }
    }
    // Create permutation map
    let permutations = graph.vertices().map(|v| {
        (v, permutation_from_lehmer(lehmer_from_natural(v, permutation_size)))
    }).collect::<HashMap<VertexID, Vec<usize>>>();
    // Calculate MST
    let kruskal = grasp::algorithms::trees::kruskal_mst(&graph, |_, (u, v): EdgeID| {
        let perm_u = permutations.get(&u)?;
        let perm_v = permutations.get(&v)?;
        Some(permutation_distance(perm_u, perm_v))
    })?;
    let mst = SubgraphView::new(&graph, None, Some(
        kruskal.into_iter().map(|(u, v, _)| (u, v)).collect()
    ), true);
    // Display graph with mst highlight
    let mut app = GraspApp::new();
    app.load(&mst);
    // TODO: Highlight mst in app
    // TODO: Show labels in app
    app.start()?;
    Ok(())
}