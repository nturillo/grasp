use std::collections::{HashMap, HashSet, VecDeque};

use crate::graph::prelude::*;

pub fn graph_distance<G: SimpleGraph>(g: &G, u: VertexID, v: VertexID) -> Option<u64> {
    if !g.has_vertex(u) || !g.has_vertex(v) {
        return None;
    }
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back((u, 0));
    visited.insert(u);
    while let Some((current, distance)) = queue.pop_front() {
        if current == v {
            return Some(distance);
        }
        for neighbor in g.neighbors(current).iter() {
            if !visited.contains(&neighbor) {
                visited.insert(*neighbor);
                queue.push_back((*neighbor, distance + 1));
            }
        }
    }
    None
}

pub fn shortest_path<G: SimpleGraph>(g: &G, u: VertexID, v: VertexID) -> Option<Vec<VertexID>> {
    if !g.has_vertex(u) || !g.has_vertex(v) {
        return None;
    }
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut parent: HashMap<VertexID, VertexID> = HashMap::new();
    queue.push_back(u);
    visited.insert(u);
    while let Some(current) = queue.pop_front() {
        if current == v {
            let mut path = Vec::new();
            let mut cur = v;
            while cur != u {
                path.push(cur);
                cur = parent[&cur];
            }
            path.push(u);
            path.reverse();
            return Some(path);
        }
        for neighbor in g.neighbors(current).iter() {
            if !visited.contains(&neighbor) {
                visited.insert(*neighbor);
                parent.insert(*neighbor, current);
                queue.push_back(*neighbor);
            }
        }
    }
    None
}