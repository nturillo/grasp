use graph_ops_macros::register;
use crate::graph::{BuildableGraph, GraphMut, VertexID, directed::DigraphProjection, prelude::{SimpleGraph, SparseSimpleGraph}};

pub fn build_complete_graph<G: GraphMut+BuildableGraph>(size: usize) -> G{
    let mut graph = G::with_capacity(size, size*(size-1)/2);
    let mut vertex_set = Vec::with_capacity(size);
    for _ in 0..size{
        let vertex = graph.create_vertex();
        let _ = graph.try_add_neighbors(vertex, vertex_set.iter().copied());
        vertex_set.push(vertex);
    }
    graph
}

pub fn build_cycle<G: GraphMut+BuildableGraph>(size: usize) -> G{
    let mut graph = G::with_capacity(size, size);
    if size == 0 {return graph;}
    let start = graph.create_vertex();
    if size == 1 {return graph;}
    let mut prev = start;
    for _ in 1..size {
        let vertex = graph.create_vertex();
        let _ = graph.try_add_edge((prev, vertex));
        prev = vertex;
    }
    let _ = graph.try_add_edge((prev, start));
    graph
}

pub fn build_path<G: GraphMut+BuildableGraph>(size: usize) -> G{
    let mut graph = G::with_capacity(size, size-1);
    if size == 0 {return graph;}
    let start = graph.create_vertex();
    if size == 1 {return graph;}
    let mut prev = start;
    for _ in 1..size {
        let vertex = graph.create_vertex();
        let _ = graph.try_add_edge((prev, vertex));
        prev = vertex;
    }
    graph
}

pub fn build_partite_graph<G: GraphMut+BuildableGraph>(mut partite_groups: Vec<usize>) -> G{
    partite_groups.retain(|g| *g!=0);
    let vertex_count = partite_groups.iter().sum();
    let edge_count = partite_groups.iter().map(|g| g*(vertex_count-g)).sum::<usize>()/2;
    let mut graph = G::with_capacity(vertex_count, edge_count);
    if partite_groups.is_empty() {return graph;}
    let mut vertex_set = Vec::with_capacity(vertex_count);
    for g in partite_groups {
        let mut current_vertices = Vec::with_capacity(g);
        for _ in 0..g {
            let vertex = graph.create_vertex();
            let _ = graph.try_add_neighbors(vertex, vertex_set.iter().copied());
            current_vertices.push(vertex);
        }
        vertex_set.extend_from_slice(&current_vertices);
    }
    graph
}

pub fn build_binary_tree<G: GraphMut+BuildableGraph>(layers: usize) -> G{
    if layers == 0 {return G::with_capacity(0, 0);}
    let vertex_count = 2_usize.pow(layers as u32)-1;
    let mut graph = G::with_capacity(vertex_count, vertex_count - 1);
    let mut stack: Vec<(VertexID, usize)> = Vec::with_capacity(layers+1);
    let vertex = graph.create_vertex();
    if layers == 1 {return graph;}
    stack.push((vertex, 0));
    // add child vertices recursively
    while let Some((vertex, layer)) = stack.pop(){
        let child_1 = graph.create_vertex();
        let child_2 = graph.create_vertex();
        let _ = graph.try_add_edge((vertex, child_1));
        let _ = graph.try_add_edge((vertex, child_2));
        if layer < layers-2{
            stack.push((child_1, layer+1));
            stack.push((child_2, layer+1));
        }
    }
    graph
}

pub fn build_bowtie<G: GraphMut+BuildableGraph>() -> G{
    let mut graph = G::with_capacity(5, 6);
    let center = graph.create_vertex();
    let l1 = graph.create_vertex();
    let l2 = graph.create_vertex();
    let r1 = graph.create_vertex();
    let r2 = graph.create_vertex();
    let _ = graph.try_add_neighbors(center, [l1, l2, r1, r2].into_iter());
    let _ = graph.try_add_edge((l1, l2));
    let _ = graph.try_add_edge((r1, r2));
    graph
}
