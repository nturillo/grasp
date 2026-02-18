use crate::graph::{GraphTrait, prelude::LabeledGraph};

pub fn graphs_eq<G: GraphTrait>(graph_a: &G, graph_b: &G) -> bool{
    for vertex in graph_a.vertices(){if !graph_b.contains(vertex) {return false;}}
    for vertex in graph_b.vertices(){if !graph_a.contains(vertex) {return false;}}
    for edge in graph_a.edges(){if !graph_b.has_edge(edge) {return false;}}
    for edge in graph_b.edges(){if !graph_a.has_edge(edge) {return false;}}
    true
}

pub fn labeled_graphs_eq<G: LabeledGraph>(graph_a: &G, graph_b: &G) -> bool
where G::VertexData: PartialEq, G::EdgeData: PartialEq
{
    if !graphs_eq(graph_a, graph_b) {return false;}
    for (v, label) in graph_a.vertex_labels(){
        if !graph_b.get_vertex_label(*v).is_some_and(|l| l==label) {return false;}
    }
    for (v, label) in graph_b.vertex_labels(){
        if !graph_a.get_vertex_label(*v).is_some_and(|l| l==label) {return false;}
    }
    for (e, label) in graph_a.edge_labels(){
        if !graph_b.get_edge_label(*e).is_some_and(|l| l==label) {return false;}
    }
    for (e, label) in graph_b.edge_labels(){
        if !graph_a.get_edge_label(*e).is_some_and(|l| l==label) {return false;}
    }
    true
}