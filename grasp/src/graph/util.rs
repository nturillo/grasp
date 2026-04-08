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
    let mut components: HashMap<usize, HashSet<usize>> = HashMap::default();
    let mut component_map: HashMap<VertexID, usize> = HashMap::default();
    let mut unvisited: HashSet<VertexID> = graph.vertices().collect();
    let mut stack: VecDeque<VertexID> = VecDeque::default();
    let mut component_index = 0;
    while !unvisited.is_empty(){
        // get start vertex
        let root = *unvisited.iter().next().unwrap(); 
        unvisited.remove(&root); stack.push_back(root);
        let mut component = HashSet::default(); component.insert(root);
        let mut intersected_components: HashSet<usize> = HashSet::default();
        // start building 
        while let Some(v) = stack.pop_front(){
            for neighbor in graph.neighbors(v).iter(){
                if unvisited.contains(&neighbor) {
                    unvisited.remove(&neighbor); 
                    stack.push_back(*neighbor);
                    component.insert(*neighbor);
                } else if component_map.contains_key(&neighbor) {
                    intersected_components.insert(*component_map.get(&neighbor).unwrap());
                }
            }
        }
        // Merge intersected components or add new one
        if intersected_components.is_empty() {
            for v in component.iter(){component_map.insert(*v, component_index);}
            components.insert(component_index, component);
            component_index += 1;
        } else {
            let mut c_iter = intersected_components.into_iter();
            let index = c_iter.next().unwrap();
            while let Some(comp) = c_iter.next(){
                let comp = components.remove(&comp).unwrap();
                for v in comp.iter(){
                    component_map.insert(*v, index);
                }
                components.get_mut(&index).unwrap().extend(comp);
            }
            for v in component.iter() {component_map.insert(*v, index);}
            components.get_mut(&index).unwrap().extend(component);
        }
    }
    components.into_values().collect()
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
