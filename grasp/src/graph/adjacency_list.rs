//! Adjacency list implementation of graph
use crate::graph::{UnderlyingGraph, set::Set};

use super::{GraphTrait, SimpleGraph, VertexID, EdgeID, DiGraph, graph_ops::*};
use std::collections::{HashMap, HashSet};


#[derive(Default, Debug)]
pub struct SparseSimpleGraph {
    adjacency_list: HashMap<VertexID, HashSet<VertexID>>
}
impl GraphTrait for SparseSimpleGraph {
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
    fn neighbors(&self, v: VertexID) -> Option<impl Set<Item=VertexID>> {
        self.adjacency_list.get(&v)
    }
    fn vertex_set(&self) -> impl Set<Item=VertexID> {
        &self.adjacency_list
    }
    fn create_vertex(&mut self) -> VertexID {
        let key= self.adjacency_list.keys().max().map(|max| max+1).unwrap_or(0);
        self.add_vertex(key);
        key
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

    fn delete_vertex(&mut self, v: VertexID) -> impl Iterator<Item=EdgeID> {
        let neighbors = self.adjacency_list.remove(&v).unwrap_or_default();
        for v2 in neighbors.iter(){
            if let Some(set) = self.adjacency_list.get_mut(v2){
                set.remove(&v);
            }
        }
        IntoIterator::into_iter(neighbors).map(move |v2| (v, v2))
    }
    fn delete_edge(&mut self, (v1, v2): EdgeID) {
        if let Some(set) = self.adjacency_list.get_mut(&v1) {set.remove(&v2);}
        if let Some(set) = self.adjacency_list.get_mut(&v2) {set.remove(&v1);}
    }
}
impl SimpleGraph for SparseSimpleGraph{}
impl GraphOps for SparseSimpleGraph{}
impl SimpleGraphOps for SparseSimpleGraph{}

#[derive(Default, Debug)]
pub struct SparseDiGraph {
    /// Arcs out from key
    out_adjacency: HashMap<VertexID, HashSet<VertexID>>,
    /// Arcs in to key
    in_adjacency: HashMap<VertexID, HashSet<VertexID>>
}
impl GraphTrait for SparseDiGraph {
    fn vertex_count(&self) -> usize {
        self.out_adjacency.len()
    }
    fn edge_count(&self) -> usize {
        self.out_adjacency.values().map(|s| s.len()).sum::<usize>()
    }
    fn vertices(&self) -> impl Iterator<Item=VertexID> {
        self.out_adjacency.keys().cloned()
    }
    fn edges(&self) -> impl Iterator<Item=(VertexID,VertexID)> {
        let mut edges = Vec::new();
        for (&v, nbhrs) in &self.out_adjacency {
            for &u in nbhrs {
                edges.push((v,u));
            }
        }
        edges.into_iter()
    }
    fn contains(&self, v: VertexID) -> bool {
        self.out_adjacency.contains_key(&v)
    }
    fn has_edge(&self, (v1, v2): EdgeID) -> bool {
        self.out_adjacency.get(&v1).is_some_and(|set| set.contains(&v2))
    }
    fn neighbors(&self, v: VertexID) -> Option<impl Set<Item = VertexID>> {
        if !self.contains(v) {return None;}
        let out_set = self.out_adjacency.get(&v).unwrap();
        let in_set = self.in_adjacency.get(&v).unwrap();
        Some(Set::union(out_set, in_set))
    }
    fn vertex_set(&self) -> impl Set<Item = VertexID> {
        &self.out_adjacency
    }
    fn create_vertex(&mut self) -> VertexID {
        let key = self.out_adjacency.keys().max().map(|max| max+1).unwrap_or(0);
        self.out_adjacency.insert(key, HashSet::default());
        self.in_adjacency.insert(key, HashSet::default());
        key
    }
    
    fn add_vertex(&mut self, v: VertexID) {
        self.out_adjacency.entry(v).or_default();
        self.in_adjacency.entry(v).or_default();
    }
    fn add_edge(&mut self, e: EdgeID) {
        let v1 = e.0;
        let v2 = e.1;
        self.out_adjacency.entry(v1).or_default().insert(v2);
        self.in_adjacency.entry(v2).or_default().insert(v1);
        // create entries for other vertices, but dont add extra arcs
        self.in_adjacency.entry(v1).or_default();
        self.out_adjacency.entry(v2).or_default();
    }
    fn add_neighbors(&mut self, v: VertexID, nbhrs: impl Iterator<Item=VertexID>) {
        let nbhr_vec: Vec<VertexID> = nbhrs.collect();
        for u in nbhr_vec.clone() {
            self.in_adjacency.entry(u).or_default().insert(v);
            self.out_adjacency.entry(u).or_default();
        }
        self.out_adjacency.entry(v).or_default().extend(nbhr_vec);
        self.in_adjacency.entry(v).or_default();
    }

    fn delete_vertex(&mut self, v: VertexID) -> impl Iterator<Item=EdgeID> {
        let out_neighbors = self.out_adjacency.remove(&v).unwrap_or_default();
        let in_neighbors = self.in_adjacency.remove(&v).unwrap_or_default();
        for v2 in out_neighbors.iter(){
            if let Some(set) = self.in_adjacency.get_mut(v2){
                set.remove(&v);
            }
        }
        for v1 in in_neighbors.iter(){
            if let Some(set) = self.out_adjacency.get_mut(v1){
                set.remove(&v);
            }
        }
        IntoIterator::into_iter(out_neighbors).map(move |v2| (v, v2)).chain(
            IntoIterator::into_iter(in_neighbors).map(move |v1| (v1, v))
        )
    }
    fn delete_edge(&mut self, (v1, v2): EdgeID) {
        if let Some(set) = self.out_adjacency.get_mut(&v1) {set.remove(&v2);}
        if let Some(set) = self.in_adjacency.get_mut(&v2) {set.remove(&v1);}
    }
}
impl DiGraph for SparseDiGraph{
    fn out_neighbors(&self, v: VertexID) -> Option<impl Set<Item = VertexID>> {
        self.out_adjacency.get(&v).map(|set| set)
    }
    fn in_neighbors(&self, v: VertexID) -> Option<impl Set<Item = VertexID>> {
        self.in_adjacency.get(&v).map(|set| set)
    }
}
impl UnderlyingGraph for SparseDiGraph{
    type UnderlyingGraph = SparseSimpleGraph;
    fn underlying_graph(&self) -> Self::UnderlyingGraph{
        let mut graph = Self::UnderlyingGraph::default();
        for v in self.vertices(){
            let n = self.out_neighbors(v).unwrap();
            graph.add_neighbors(v, n.iter().cloned());
        }
        graph
    }
}
impl GraphOps for SparseDiGraph{}

/// Placed here instead of in set.rs since it is not standard behaviour
impl<'a, K> Set for &'a HashMap<VertexID, K>{
    type Item = VertexID;
    fn contains(&self, v: &Self::Item) -> bool {
        self.contains_key(v)
    }
    fn count(&self) -> usize {
        self.len()
    }
    fn iter(&self) -> impl Iterator<Item = &Self::Item> {
        self.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn butterfly_graph() {
        let mut butterfly = SparseSimpleGraph::default();
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

        assert!(butterfly.has_edge((2, 1)));
        
        assert!(!butterfly.has_edge((3, 4)));
        assert!(!butterfly.has_edge((2, 5)));
        
        assert!(!butterfly.has_edge((1, 6)));
        assert!(!butterfly.has_edge((10, 3843)));
        
        assert!(butterfly.vertex_count() == 5);
        assert!(butterfly.edge_count() == 6);

        butterfly.delete_edge((4, 5));
        assert!(butterfly.vertex_count() == 5);
        assert!(butterfly.edge_count() == 5);
        let _ = butterfly.delete_vertex(2);
        assert!(butterfly.vertex_count() == 4);
        assert!(butterfly.edge_count() == 3);
        assert!(!butterfly.has_edge((2, 3)));
        assert!(!butterfly.has_edge((4, 5)));
        assert!(!butterfly.contains(2));
    }

    #[test]
    fn sparse_graph_ops(){
        use crate::graph::{test::*, graph_ops::test::*};
        graph_vs_digraph_test::<SparseSimpleGraph, SparseDiGraph>();
        digraph_fn_test::<SparseDiGraph>();
        graph_ops_test::<SparseSimpleGraph>();
        simple_graph_ops_test::<SparseSimpleGraph>();
        simple_graph_complement_test::<SparseSimpleGraph>();
        graph_ops_test::<SparseDiGraph>();
        digraph_complement_test::<SparseDiGraph>();
        underlying_graph_test::<SparseDiGraph>();
    }
}