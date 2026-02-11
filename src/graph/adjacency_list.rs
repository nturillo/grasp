//! Adjacency list implementation of graph
use super::{Graph, VertexID, EdgeID};
use std::collections::{HashMap, HashSet};


#[derive(Default, Debug)]
pub struct SparseGraph {
    adjacency_list: HashMap<VertexID, HashSet<VertexID>>
}
impl Graph for SparseGraph {
    type VertexSet = HashSet<VertexID>;

    fn vertex_count(&self) -> usize {
        self.adjacency_list.len()
    }
    fn edge_count(&self) -> usize {
        self.adjacency_list.values().map(|s| s.len()).sum::<usize>()/2
    }
    fn vertices(&self) -> impl Iterator<Item=VertexID> {
        self.adjacency_list.keys().cloned()
    }
    fn edges(&self) -> impl Iterator<Item=(VertexID,VertexID)> {
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
    fn contains(&self, v: VertexID) -> bool {
        self.adjacency_list.contains_key(&v)
    }
    fn has_edge(&self, e: EdgeID) -> bool {
        let v1 = e.0;
        let v2 = e.1;

        if !self.adjacency_list.contains_key(&v1) || !self.adjacency_list.contains_key(&v2) {
            return false;
        }
        self.adjacency_list[&v1].contains(&v2)
    }
    fn neighbors(&self, v: VertexID) -> Option<&Self::VertexSet> {
        self.adjacency_list.get(&v)
    }
    fn vertex_set(&self) -> Self::VertexSet {
        self.adjacency_list.keys().cloned().collect()
    }
    fn create_vertex(&mut self) -> VertexID {
        if let Some(max) = self.adjacency_list.keys().max() {max+1} else {0}
    }
    
    fn add_vertex(&mut self, v: VertexID) {
        self.adjacency_list.entry(v).or_default();
    }
    fn add_edge(&mut self, e: EdgeID) {
        let v1 = e.0;
        let v2 = e.1;
        self.adjacency_list.entry(v1).or_default().insert(v2);
        self.adjacency_list.entry(v2).or_default().insert(v1);
    }
    fn add_neighbors(&mut self, v: VertexID, nbhrs: impl Iterator<Item=VertexID>) {
        let nbhr_vec: Vec<VertexID> = nbhrs.collect();
        for u in nbhr_vec.clone() {
            self.adjacency_list.entry(u).or_default().insert(v);
        }
        self.adjacency_list.entry(v).or_default().extend(nbhr_vec);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn butterfly_graph() {
        let mut butterfly = SparseGraph::default();
        butterfly.add_edge((1,2));
        butterfly.add_edge((2,3));
        butterfly.add_edge((1,3));
        butterfly.add_edge((1,4));
        butterfly.add_edge((1,5));
        butterfly.add_edge((4,5));
        
        assert!(butterfly.has_edge((1, 2)));
        assert!(butterfly.has_edge((2, 3)));
        assert!(butterfly.has_edge((1, 3)));
        assert!(butterfly.has_edge((1, 4)));
        assert!(butterfly.has_edge((1, 5)));
        assert!(butterfly.has_edge((4, 5)));
        
        assert!(!butterfly.has_edge((3, 4)));
        assert!(!butterfly.has_edge((2, 5)));
        
        assert!(!butterfly.has_edge((1, 6)));
        assert!(!butterfly.has_edge((10, 3843)));
        
        assert!(butterfly.vertex_count() == 5);
        assert!(butterfly.edge_count() == 6);
    }
}