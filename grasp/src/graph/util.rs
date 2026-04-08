use std::collections::{HashMap, HashSet, VecDeque};

use crate::graph::{EdgeID, GraphTrait, VertexID, prelude::LabeledGraph, set::Set};

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

/// The difference between two graphs, used for testing
pub struct GraphDiff {
    pub left_extra_vertices: Vec<VertexID>,
    pub right_extra_vertices: Vec<VertexID>,
    pub left_extra_edges: Vec<EdgeID>,
    pub right_extra_edges: Vec<EdgeID>,
}

pub fn graph_diff(left_graph: &impl GraphTrait, right_graph: &impl GraphTrait) -> Option<GraphDiff> {
    let mut left_extra_vertices= Vec::new();
    let mut right_extra_vertices = Vec::new();
    let mut left_extra_edges = Vec::new();
    let mut right_extra_edges = Vec::new();

    left_extra_vertices.extend(left_graph.vertices().filter(|v| !right_graph.has_vertex(*v)));
    right_extra_vertices.extend(right_graph.vertices().filter(|v| !left_graph.has_vertex(*v)));
    left_extra_edges.extend(left_graph.edges().filter(|e| !right_graph.has_edge(*e)));
    right_extra_edges.extend(right_graph.edges().filter(|e| !left_graph.has_edge(*e)));

    if left_extra_edges.is_empty() && right_extra_edges.is_empty() && left_extra_vertices.is_empty() && right_extra_vertices.is_empty() {
        return None
    } 
    Some(GraphDiff {
        left_extra_vertices,
        right_extra_vertices,
        left_extra_edges,
        right_extra_edges,
    })
}

#[macro_export]
macro_rules! assert_graphs_eq {
    ($expected:expr, $actual:expr) => {
        if let Some(diff) = $crate::graph::util::graph_diff(&$expected, &$actual) {
            panic!(
                "Graphs are not equal!\n\
                {} vs {}\n\
                Extra vertices in left: {:?}\n\
                Extra vertices in right: {:?}\n\
                Extra edges in left: {:?}\n\
                Extra edges in right: {:?}\n\
                ",
                stringify!($expected),
                stringify!($actual),
                diff.left_extra_vertices,
                diff.right_extra_vertices,
                diff.left_extra_edges,
                diff.right_extra_edges,
            );
        }
    };
}

/// Get the degeneracy of a graph, and set *out* to be the degeneracy ordering
pub fn degeneracy<G: GraphTrait>(graph: &G, out: &mut Vec<VertexID>) -> usize {
    out.clear();
    out.reserve(graph.vertex_count());

    let mut degree_map: HashMap<VertexID, usize> = graph.vertices()
        .map(|v| (v, degree(graph, v)))
        .collect();

    let max_degree = degree_map.values().copied().max().unwrap_or(0);
    let mut buckets: Vec<HashSet<VertexID>> = vec![HashSet::new(); max_degree + 1];
    for (&v, &d) in &degree_map {
        buckets[d].insert(v);
    }

    let mut removed: HashSet<VertexID> = HashSet::with_capacity(graph.vertex_count());
    let mut min_degree = 0;
    let mut k = 0;

    for _ in 0..graph.vertex_count() {
        while buckets[min_degree].is_empty() {
            min_degree += 1;
        }

        let v = *buckets[min_degree].iter().next().unwrap();
        buckets[min_degree].remove(&v);
        removed.insert(v);

        k = k.max(min_degree);
        out.push(v);

        for &u in graph.neighbors(v).difference_with(&removed).iter() {
            let d = degree_map.get_mut(&u).unwrap();
            buckets[*d].remove(&u);
            *d -= 1;
            buckets[*d].insert(u);
            min_degree = min_degree.min(*d);
        }
    }

    out.reverse();
    k
}