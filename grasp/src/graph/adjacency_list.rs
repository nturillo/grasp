//! Adjacency list implementation of graph
use crate::graph::prelude::*;
use std::collections::{HashMap, HashSet};


#[derive(Default, Debug, GraphOps, SimpleGraphOps, Clone)]
pub struct SparseSimpleGraph {
    adjacency_list: HashMap<usize, HashSet<usize>>
}
impl GraphTrait for SparseSimpleGraph {
    fn vertex_count(&self) -> usize {
        self.adjacency_list.len()
    }
    fn edge_count(&self) -> usize {
        self.adjacency_list.values().map(|s| s.len()).sum::<usize>()/2
    }
    fn vertices(&self) -> impl Iterator<Item=usize> {
        self.adjacency_list.keys().cloned()
    }
    fn edges(&self) -> impl Iterator<Item=(usize,usize)> {
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
    fn has_vertex(&self, v: usize) -> bool {
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
    fn neighbors(&self, v: usize) -> impl Set<Item=usize> {
        self.adjacency_list.get(&v)
    }
    fn vertex_set(&self) -> impl Set<Item=usize> {
        &self.adjacency_list
    }
}
impl AnyVertexGraph for SparseSimpleGraph{
    fn add_vertex(&mut self, id: usize) {
        self.adjacency_list.entry(id).or_default();
    }
}
impl GraphMut for SparseSimpleGraph{
    fn create_vertex(&mut self) -> usize {
        let key= self.adjacency_list.keys().max().map(|max| max+1).unwrap_or(0);
        self.add_vertex(key);
        key
    }
    fn try_add_edge(&mut self, (u, v): EdgeID) -> Result<(), GraphError> {
        if u == v {
            return Err(GraphError::EdgeNotAddable((u,v), "No loops allowed in simple graph".to_string()));
        }
        let (has_u, has_v) = (self.has_vertex(u), self.has_vertex(v));
        if !has_u && !has_v {return Err(GraphError::NeitherVertexInGraph(u, v));}
        else if !has_u {return Err(GraphError::VertexNotInGraph(u));}
        else if !has_v {return Err(GraphError::VertexNotInGraph(v));}
        self.adjacency_list.get_mut(&u).unwrap().insert(v);
        self.adjacency_list.get_mut(&v).unwrap().insert(u);
        Ok(())
    }

    fn remove_vertex(&mut self, v: usize) -> impl Iterator<Item=EdgeID> {
        let neighbors = self.adjacency_list.remove(&v).unwrap_or_default();
        for v2 in neighbors.iter(){
            if let Some(set) = self.adjacency_list.get_mut(v2){
                set.remove(&v);
            }
        }
        IntoIterator::into_iter(neighbors).map(move |v2| (v, v2).into())
    }
    fn remove_edge(&mut self, (v1, v2): EdgeID) -> bool {
        if let Some(set) = self.adjacency_list.get_mut(&v1) {set.remove(&v2);}
        if let Some(set) = self.adjacency_list.get_mut(&v2) {
            set.remove(&v1);
            return true; // If v1,v2 in graph, it should always get here
        }
        false
    }
}
impl SimpleGraph for SparseSimpleGraph{}

#[derive(Default, Debug, GraphOps)]
pub struct SparseDiGraph {
    /// Arcs out from key
    out_adjacency: HashMap<usize, HashSet<usize>>,
    /// Arcs in to key
    in_adjacency: HashMap<usize, HashSet<usize>>
}
impl GraphTrait for SparseDiGraph {
    fn vertex_count(&self) -> usize {
        self.out_adjacency.len()
    }
    fn edge_count(&self) -> usize {
        self.out_adjacency.values().map(|s| s.len()).sum::<usize>()
    }

    fn vertices(&self) -> impl Iterator<Item=usize> {
        self.out_adjacency.keys().cloned()
    }
    fn edges(&self) -> impl Iterator<Item=EdgeID> {
        self.out_adjacency.iter().map(|(u, nbhrs)| {
            nbhrs.iter().map(|v| (*u, *v).into())
        }).flatten()
    }
    
    fn has_vertex(&self, v: usize) -> bool {
        self.out_adjacency.contains_key(&v)
    }
    fn has_edge(&self, (v1, v2): EdgeID) -> bool {
        self.out_adjacency.get(&v1).is_some_and(|set| set.contains(&v2))
    }
    
    fn neighbors(&self, v: usize) -> impl Set<Item = usize> {
        self.out_adjacency.get(&v).unwrap()
    }
    fn vertex_set(&self) -> impl Set<Item = usize> {
        &self.out_adjacency
    }
}
impl AnyVertexGraph for SparseDiGraph{
    fn add_vertex(&mut self, id: VertexID) {
        self.out_adjacency.entry(id).or_default();
        self.in_adjacency.entry(id).or_default();
    }
}
impl GraphMut for SparseDiGraph{
    fn create_vertex(&mut self) -> usize {
        let key = self.out_adjacency.keys().max().map(|max| max+1).unwrap_or(0);
        self.out_adjacency.insert(key, HashSet::default());
        self.in_adjacency.insert(key, HashSet::default());
        key
    }
    
    fn try_add_edge(&mut self, (u, v): EdgeID) -> Result<(), GraphError> {
        let (has_u, has_v) = (self.has_vertex(u), self.has_vertex(v));
        if !has_u && !has_v {return Err(GraphError::NeitherVertexInGraph(u, v));}
        else if !has_u {return Err(GraphError::VertexNotInGraph(u));}
        else if !has_v {return Err(GraphError::VertexNotInGraph(v));}
        self.out_adjacency.get_mut(&u).unwrap().insert(v);
        self.in_adjacency.get_mut(&v).unwrap().insert(u);
        Ok(())
    }

    fn remove_vertex(&mut self, v: usize) -> impl Iterator<Item=EdgeID> {
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
        out_neighbors.into_iter().map(move |v2| (v, v2).into()).chain(
            in_neighbors.into_iter().map(move |v1| (v1, v).into())
        )
    }
    fn remove_edge(&mut self, (v1, v2): EdgeID) -> bool {
        if let Some(set) = self.out_adjacency.get_mut(&v1) {set.remove(&v2);}
        if let Some(set) = self.in_adjacency.get_mut(&v2) {
            set.remove(&v1);
            return true; // If v1, v2 in graph it will get here
        }
        false
    }
}
impl DiGraph for SparseDiGraph{
    fn out_neighbors(&self, v: usize) -> impl Set<Item = usize> {
        self.out_adjacency.get(&v)
    }
    fn in_neighbors(&self, v: usize) -> impl Set<Item = usize> {
        self.in_adjacency.get(&v)
    }
    fn all_neighbors(&self, v: usize) -> impl Set<Item = usize> {
        if self.has_vertex(v) {
            Some(Set::union(&self.in_adjacency[&v], &self.out_adjacency[&v]))
        } else {None}
    }
}
impl DigraphProjection for SparseDiGraph{
    fn as_simple<'b>(&'b self) -> impl SimpleGraph {
        SimpleView::from(self)
    }
    fn as_underlying<'b>(&'b self) -> impl SimpleGraph {
        UnderlyingView::from(self)
    }
}


/// Placed here instead of in set.rs since it is not standard behaviour
impl<'a, K> Set for &'a HashMap<VertexID, K>{
    type Item = VertexID;
    fn contains(&self, v: &Self::Item) -> bool {
        self.contains_key(v)
    }
    fn len(&self) -> usize {
        (*self).len()
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

        butterfly.remove_edge((4, 5));
        assert!(butterfly.vertex_count() == 5);
        assert!(butterfly.edge_count() == 5);
        let _ = butterfly.remove_vertex(2);
        assert!(butterfly.vertex_count() == 4);
        assert!(butterfly.edge_count() == 3);
        assert!(!butterfly.has_edge((2, 3)));
        assert!(!butterfly.has_edge((4, 5)));
        assert!(!butterfly.has_vertex(2));
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
        digraph_projection_test::<SparseDiGraph>();
    }
}