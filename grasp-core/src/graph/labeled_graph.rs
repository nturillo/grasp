use std::collections::HashSet;

use crate::graph::graph_traits::{EdgeType, GraphTrait, VertexType};

pub trait LabeledGraphTrait<V, E>: GraphTrait {
    fn get_vertex_label(v: VertexType) -> V;
    fn get_edge_label(e: EdgeType) -> E;
}

struct LabeledGraph<G, V, E> {
    graph: G,
    vertex_data: HashSet<VertexType, V>,
    edge_data: HashSet<EdgeType, E>,    
}