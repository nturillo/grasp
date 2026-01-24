// Adjacency list implementation of graph

use crate::graph::{errors::GraphError, graph_traits::*};
use std::collections::{HashMap, HashSet};

impl SetTrait for HashSet<VertexType> {
    fn contains(&self, v: VertexType) -> bool {
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
    fn iter(&self) -> impl Iterator<Item=&VertexType> {
        self.iter()
    }
}

pub struct SparseGraph {
    adjacency_list: HashMap<VertexType, HashSet<VertexType>>
}

impl GraphTrait for SparseGraph {
    type NeighborSet = HashSet<VertexType>;
    
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
    fn vertices(&self) -> impl Iterator<Item=VertexType> {
        self.adjacency_list.keys().cloned()
    }
    fn edges(&self) -> impl Iterator<Item=(VertexType,VertexType)> {
        let mut edges = Vec::new();
        for (&v, nbhrs) in &self.adjacency_list {
            for &u in nbhrs {
                if v < u {
                    edges.push((v,u));
                }

            }
        }
        edges.into_iter()
    }
    fn contains(&self, v: VertexType) -> bool {
        self.adjacency_list.contains_key(&v)
    }
    fn has_edge(&self, v1: VertexType, v2: VertexType) -> Result<bool, GraphError> {
        if !self.adjacency_list.contains_key(&v1) || !self.adjacency_list.contains_key(&v2) {
            return Err(GraphError::VertexNotInGraph)
        }
        Ok(self.adjacency_list[&v1].contains(&v2))
    }
    fn neighbors(&self, v: VertexType) -> Result<&Self::NeighborSet, GraphError> {
        self.adjacency_list.get(&v).ok_or(GraphError::VertexNotInGraph)
    }
    
    fn add_vertex(&mut self, v: VertexType) {
        self.adjacency_list.entry(v).or_default();
    }
    fn add_edge(&mut self, v1: VertexType, v2:VertexType) {
        self.adjacency_list.entry(v1).or_default().insert(v2);
        self.adjacency_list.entry(v2).or_default().insert(v1);
    }
    fn add_neighbors(&mut self, v: VertexType, nbhrs: impl Iterator<Item=VertexType>) {
        let nbhr_vec: Vec<VertexType> = nbhrs.collect();
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
        
        assert!(butterfly.has_edge(1, 6).expect_err("edge shouldn't exist") == GraphError::VertexNotInGraph);
        assert!(butterfly.has_edge(10, 3843).expect_err("edge shouldn't exist") == GraphError::VertexNotInGraph);
        
        assert!(butterfly.num_vertices() == 5);
        assert!(butterfly.num_edges() == 6);
    }
}