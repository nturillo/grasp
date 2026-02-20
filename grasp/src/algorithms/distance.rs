use std::collections::{HashSet, VecDeque};

use crate::graph::prelude::*;

pub fn graph_distance<G: SimpleGraph>(g: &G, u: VertexID, v: VertexID) -> Option<u64> {
    let mut queue: VecDeque<(VertexID, u64)> = VecDeque::new();
    let mut visited: HashSet<VertexID> = HashSet::new();

    queue.push_back((u, 0));
    
    while !queue.is_empty() {
        let (vertex, distance) = queue.pop_front().unwrap();
        if vertex == v {
            return Some(distance)
        }
        if visited.contains(&vertex) {
            continue;
        }
        visited.insert(vertex);
        for &nbr in g.neighbors(vertex).unwrap().iter() {
            queue.push_back((nbr, distance + 1));
        }
    }

    None
}