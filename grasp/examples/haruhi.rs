/*
    This example creates a permutation graph where edge weights are defined by their minimal super-permutation
    We then run Kruskal's algorithm to find the mst of this graph.
    This is a crude lower bound for the length of the shortest hamiltonian path.
    When the permutation size is 14, this is known as the haruhi problem.
*/

use std::{collections::{HashMap, HashSet}, env};
use eframe::egui::Color32;
use grasp::graph::{graph_ops::SubgraphView, permutation::{PermutationDiGraph, lehmer_from_natural, permutation_from_lehmer}, prelude::*};
use gdraw::app::GraspApp;

/// Calculates the number of additions to permutation 1 to get to permutation 2
fn permutation_distance(perm_1: &Vec<usize>, perm_2: &Vec<usize>) -> usize{
    assert!(perm_1.len() == perm_2.len());
    for i in 0..perm_1.len() {
        // test if perm_1[i..] == perm_2[..i]
        if perm_1[i..] == perm_2[..(perm_2.len()-i)] {return i;}
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
    // Calculate lower bound
    let path_weight: usize = kruskal.iter().map(|(_, _, w)| *w).sum();
    println!("A Minimal Hamiltonian path of the super-permutation graph of {}-permutations will have length at least {}", permutation_size, path_weight);
    println!("Thus a sequence containing every permutation must have at least {} elements", path_weight+permutation_size);
    if permutation_size <=5 {
        println!("The known shortest sequence is {} elements long", vec![1, 3, 9, 33, 153][permutation_size-1]);
    }
    // Dont try rendering more than 5, its just too much
    if permutation_size <= 5 {
        let mst = SubgraphView::new(&graph, None, Some(
            kruskal.into_iter().map(|(u, v, _)| (u, v)).collect()
        ), true);
        // Display graph with mst highlight
        let mut app = GraspApp::new();
        app.load(&mst);
        // TODO: Highlight mst in app
        let edge_set = mst.edges().collect::<HashSet<EdgeID>>();
        app.highlight_edges(&edge_set, Color32::RED);
        // TODO: Show labels in app
        for (id, label) in app.graph.vertex_labels.iter_mut(){
            let Some(permutation) = permutations.get(id) else {continue;};
            label.data = Some(format!("{:?}", *permutation));
        }
        for ((a, b), label) in app.graph.edge_labels.iter_mut(){
            let Some(perm_a) = permutations.get(a) else {continue;};
            let Some(perm_b) = permutations.get(b) else {continue;};
            let weight = permutation_distance(perm_a, perm_b);
            label.data = Some(format!("{}", weight));
        }
        app.start()?;
    }
    Ok(())
}