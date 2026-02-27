use macros::register;
use crate::linkme;

use crate::graph::prelude::*;

#[register(name = "that", ret = None, desc = "Literally does nothing")]
pub fn graph_distance<G: DiGraph>(
    _g: &G,
    #[param("u", "String")]
    _u: String,
    #[param("v", "Vertex")]
    _v: usize) {
    ()
}

#[register(name = "this", ret = Edge)]
pub fn fun_func<G: DiGraph>(
    _g: &G,
    #[param("u", "String")]
    _u: String,
    #[param("v", "Vertex")]
    _v: usize,
    #[param("big", "Vertex")]
    _gg: usize,
    #[param("d", "Edge")]
    _d: EdgeID)
    -> (usize, usize) {
    return _d;
}

#[register(name = "vertex set select", ret = VertexList)]
pub fn vs_func<G: DiGraph>(
    _g: &G,
    #[param("u", "String")]
    _u: String,
    #[param("v", "Vertex")]
    _v: usize,
    #[param("big", "VertexList")]
    _gg: Vec<usize>,
    #[param("d", "Edge")]
    _d: EdgeID)
    -> Vec<VertexID> {
    return _gg;
}

#[register(name = "edge set select", simple = true, ret = EdgeList)]
pub fn edge_select<G: SimpleGraph>(_g: &G,
    #[param("check", "Boolean")] b: bool,
    #[param("big", "EdgeList")]
    _gg: Vec<(usize, usize)>) -> Vec<EdgeID> {
        return _gg;
    }