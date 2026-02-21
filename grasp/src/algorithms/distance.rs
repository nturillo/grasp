use std::collections::{HashSet, VecDeque};

use crate::graph::prelude::*;

pub fn graph_distance<G: SimpleGraph>(g: &G, u: VertexID, v: VertexID) -> Option<u64> {
    if !g.contains(v) || !g.contains(u) {
        return None
    }

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

#[cfg(test)]
mod tests {
    use crate::{algorithms::distance::graph_distance, graph::{GraphTrait, prelude::SparseSimpleGraph}};

    #[test]
    fn test_graph_distance() {
        let mut g = SparseSimpleGraph::default();
        g.add_edge((0, 1));
        g.add_edge((0, 2));
        g.add_edge((2, 3));
        g.add_edge((2, 4));
        g.add_edge((3, 5));
        g.add_edge((5, 6));

        assert_eq!(graph_distance(&g, 0, 6), Some(4));
        assert_eq!(graph_distance(&g, 0, 100), None);
        
        g.add_edge((0,6));
        assert_eq!(graph_distance(&g, 0, 6), Some(1));
    }
}