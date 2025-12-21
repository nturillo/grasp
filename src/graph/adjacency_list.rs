// Adjacency list implementation of graph

use crate::graph::{adjacency_list, graph_traits::*};
use std::collections::{HashMap, HashSet};

impl SetTrait for HashSet<usize> {
    fn contains(&self, v: usize) -> bool {
        self.contains(&v)
    }
    fn num_vertices(&self) -> usize {
        self.len()
    }
    fn intersection(&self, other: &Self) -> Self {
        self & other
    }
    fn union(&self, other: &Self) -> Self {
        self | other
    }
    fn iter(&self) -> impl Iterator<Item=&usize> {
        self.iter()
    }
}

struct SparseGraph {
    adjacency_list: HashMap<usize, HashSet<usize>>
}

impl GraphTrait for SparseGraph {
    type NeighborSet = HashSet<usize>;
    
    fn new() -> Self {
        Self {
            adjacency_list: HashMap::new()
        }
    }

    fn num_vertices(&self) -> usize {
        self.adjacency_list.len()
    }
    fn num_edges(&self) -> usize {
        self.adjacency_list.values()
            .fold(0, |acc, set| acc + set.len()) / 2
    }
    fn has_edge(&self, v1: usize, v2: usize) -> Option<bool> {
        if !self.adjacency_list.contains_key(&v1) || !self.adjacency_list.contains_key(&v2) {
            return None
        }
        Some(self.adjacency_list[&v1].contains(&v2))
    }
    fn neighbors(&self, v: usize) -> Option<&Self::NeighborSet> {
        self.adjacency_list.get(&v)
    }
    
    fn add_vertex(&mut self, v: usize) {
        self.adjacency_list.entry(v).or_default();
    }
    fn add_edge(&mut self, v1: usize, v2:usize) {
        self.adjacency_list.entry(v1).or_default().insert(v2);
        self.adjacency_list.entry(v2).or_default().insert(v1);
    }
    fn add_neighbors(&mut self, v: usize, nbhrs: impl Iterator<Item=usize>) {
        let nbhr_vec: Vec<usize> = nbhrs.collect();
        for u in nbhr_vec.clone() {
            self.adjacency_list.entry(u).or_default();
        }
        self.adjacency_list.entry(v).or_default().extend(nbhr_vec);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn butterfly_graph() {
        let mut butterfly = SparseGraph::new();
        butterfly.add_edge(1, 2);
        butterfly.add_edge(2,3);
        butterfly.add_edge(1,3);
        butterfly.add_edge(1,4);
        butterfly.add_edge(1,5);
        butterfly.add_edge(4,5);
        
        assert!(butterfly.has_edge(1, 2).unwrap());
        assert!(butterfly.has_edge(2, 3).unwrap());
        assert!(butterfly.has_edge(1, 3).unwrap());
        assert!(butterfly.has_edge(1, 4).unwrap());
        assert!(butterfly.has_edge(1, 5).unwrap());
        assert!(butterfly.has_edge(4, 5).unwrap());
        
        assert!(!butterfly.has_edge(3, 4).unwrap());
        assert!(!butterfly.has_edge(2, 5).unwrap());
        
        assert!(butterfly.has_edge(1, 6).is_none());
        assert!(butterfly.has_edge(10, 3939).is_none());
        
        assert!(butterfly.num_vertices() == 5);
        assert!(butterfly.num_edges() == 6);
    }
}