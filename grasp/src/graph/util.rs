use std::collections::{HashSet, VecDeque};

use crate::graph::{GraphTrait, VertexID, prelude::LabeledGraph, set::Set};

/// Tests if two graphs are equal, (Not Isomorphic)
pub fn graphs_eq<G: GraphTrait>(graph_a: &G, graph_b: &G) -> bool{
    for vertex in graph_a.vertices(){if !graph_b.has_vertex(vertex) {return false;}}
    for vertex in graph_b.vertices(){if !graph_a.has_vertex(vertex) {return false;}}
    for edge in graph_a.edges(){if !graph_b.has_edge(edge) {return false;}}
    for edge in graph_b.edges(){if !graph_a.has_edge(edge) {return false;}}
    true
}

/// Tests if two labeled graphs are equal (Not Isomorphic)
pub fn labeled_graphs_eq<G: LabeledGraph>(graph_a: &G, graph_b: &G) -> bool
where G::VertexData: PartialEq, G::EdgeData: PartialEq
{
    if !graphs_eq(graph_a, graph_b) {return false;}
    for (v, label) in graph_a.vertex_labels(){
        if !graph_b.get_vertex_label(v).is_some_and(|l| l==label) {return false;}
    }
    for (v, label) in graph_b.vertex_labels(){
        if !graph_a.get_vertex_label(v).is_some_and(|l| l==label) {return false;}
    }
    for (e, label) in graph_a.edge_labels(){
        if !graph_b.get_edge_label(e).is_some_and(|l| l==label) {return false;}
    }
    for (e, label) in graph_b.edge_labels(){
        if !graph_a.get_edge_label(e).is_some_and(|l| l==label) {return false;}
    }
    true
}

/// Gets the degree of a vertex, 0 if not in the graph
pub fn degree<G: GraphTrait>(graph: &G, v: VertexID) -> usize{
    graph.neighbors(v).len()
}

/// Gets a list of VertexID sets which correspond to each component of a graph
pub fn get_components<G: GraphTrait>(graph: &G) -> Vec<impl Set<Item = VertexID>>{
    if graph.vertex_count()==0 {return Vec::new();}
    let mut components = vec![];
    let mut unvisited: HashSet<VertexID> = graph.vertices().collect();
    let mut stack: VecDeque<VertexID> = VecDeque::default();
    
    while !unvisited.is_empty(){
        // get start vertex
        let root = *unvisited.iter().next().unwrap(); 
        unvisited.remove(&root); stack.push_back(root);
        let mut component = HashSet::default(); component.insert(root);
        // start building 
        while let Some(v) = stack.pop_front(){
            for neighbor in graph.neighbors(v).iter(){
                if unvisited.contains(neighbor) {
                    unvisited.remove(neighbor); 
                    stack.push_back(*neighbor);
                    component.insert(*neighbor);
                }
            }
        }
        components.push(component);
    }
    components
}