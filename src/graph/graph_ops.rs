use std::collections::HashMap;
use super::{GraphTrait, VertexID, EdgeID, VertexMap, SimpleGraph};

/// Graph operations that are agnostic to simple graphs and digraphs
pub trait GraphOps: GraphTrait+Sized{
    fn subgraph_vertex(&self, vertices: impl IntoIterator<Item=VertexID>, graph_builder: impl FnOnce() -> Self) -> Self {
        let mut subgraph = graph_builder();
        for vertex in vertices{
            subgraph.add_vertex(vertex);
        }
        for (v1, v2) in self.edges(){
            if subgraph.contains(v1) && subgraph.contains(v2) {
                subgraph.add_edge((v1, v2));
            }
        }
        subgraph
    }

    fn subgraph_edges(&self, edges: impl IntoIterator<Item=EdgeID>, graph_builder: impl FnOnce() -> Self) -> Self {
        let mut subgraph = graph_builder();
        for edge in edges{
            if !self.has_edge(edge) {continue;}
            subgraph.add_edge(edge);
        }
        subgraph
    }

    fn merge(&self, other: &Self, graph_builder: impl FnOnce() -> Self) -> (Self, VertexMap, VertexMap) {
        let mut self_map = HashMap::default();
        let mut other_map = HashMap::default();
        let mut merged = graph_builder();
        // vertices
        for v in self.vertices() {
            let new_vertex = merged.create_vertex();
            self_map.insert(v, new_vertex);
        }
        for v in other.vertices() {
            let new_vertex = merged.create_vertex();
            other_map.insert(v, new_vertex);
        }
        // edges
        for (v1, v2) in self.edges() {
            let Some(v1) = self_map.get(&v1) else {continue;};
            let Some(v2) = self_map.get(&v2) else {continue;};
            merged.add_edge((*v1, *v2));
        }
        for (v1, v2) in other.edges() {
            let Some(v1) = other_map.get(&v1) else {continue;};
            let Some(v2) = other_map.get(&v2) else {continue;};
            merged.add_edge((*v1, *v2));
        }
        (merged, self_map, other_map)
    }

    fn complement(&self, graph_builder: impl FnOnce() -> Self) -> Self {
        let mut complement = graph_builder();
        for v1 in self.vertices(){
            complement.add_vertex(v1);
            for v2 in self.vertices(){
                if v1==v2 {continue;} // Complement ignores loops
                if self.has_edge((v1, v2)){continue;}
                complement.add_edge((v1, v2));
            }
        }
        complement
    }
}

/// Graph operations that only work for simple graphs
pub trait SimpleGraphOps: GraphOps+SimpleGraph{
    fn join(&self, other: &Self, graph_builder: impl FnOnce() -> Self) -> (Self, VertexMap, VertexMap) {
        let (mut joined, self_map, other_map) = self.merge(other, graph_builder);
        for v1 in self.vertices(){
            let Some(v1) = self_map.get(&v1) else {continue;};
            for v2 in other.vertices(){
                let Some(v2) = other_map.get(&v2) else {continue;};
                joined.add_edge((*v1, *v2));
            }
        }
        (joined, self_map, other_map)
    }

    fn product(&self, other: &Self, graph_builder: impl FnOnce() -> Self) -> (Self, HashMap<(VertexID, VertexID), VertexID>) {
        let mut map = HashMap::default();
        let mut product = graph_builder();
        // Vertices and map
        for v1 in self.vertices(){
            for v2 in other.vertices(){
                let v = product.create_vertex();
                map.insert((v1, v2), v);
            }
        }
        // edges part 1
        for (s1, s2) in self.edges(){
            for o in other.vertices(){
                let Some(v1) = map.get(&(s1, o)) else {continue;};
                let Some(v2) = map.get(&(s2, o)) else {continue;};
                product.add_edge((*v1, *v2));
            }
        }
        // edges part 2
        for (o1, o2) in other.edges(){
            for s in self.vertices(){
                let Some(v1) = map.get(&(s, o1)) else {continue;};
                let Some(v2) = map.get(&(s, o2)) else {continue;};
                product.add_edge((*v1, *v2));
            }
        }
        (product, map)
    }
}

#[cfg(test)]
pub mod test{
    use crate::graph::prelude::*;

    /// Assures Graph Ops functionality
    pub fn graph_ops_test<G: GraphOps+Default>(){
        let mut graph_a =  G::default();
        graph_a.add_edge((0, 1)); graph_a.add_edge((1, 2)); graph_a.add_edge((2, 0));
        // ensure subgraphs work
        let subgraph_vertices_a = graph_a.subgraph_vertex([0, 1], G::default);
        let subgraph_edges_a = graph_a.subgraph_edges([(0, 1)], G::default);
        let mut test_subgraph = G::default(); test_subgraph.add_edge((0, 1));
        assert!(graphs_eq(&subgraph_edges_a, &test_subgraph));
        assert!(graphs_eq(&subgraph_vertices_a, &test_subgraph));
        // create merged graph manually and test to ensure equality
        let (merged, map_a, map_b) = graph_a.merge(&graph_a, G::default);
        let mut test_graph = G::default();
        test_graph.add_edge((*map_a.get(&0).unwrap(), *map_a.get(&1).unwrap()));
        test_graph.add_edge((*map_a.get(&1).unwrap(), *map_a.get(&2).unwrap()));
        test_graph.add_edge((*map_a.get(&2).unwrap(), *map_a.get(&0).unwrap()));
        test_graph.add_edge((*map_b.get(&0).unwrap(), *map_b.get(&1).unwrap()));
        test_graph.add_edge((*map_b.get(&1).unwrap(), *map_b.get(&2).unwrap()));
        test_graph.add_edge((*map_b.get(&2).unwrap(), *map_b.get(&0).unwrap()));
        assert!(graphs_eq(&merged, &test_graph));
    }

    pub fn simple_graph_complement_test<G: GraphOps+Default>(){
        // Complement
        let mut graph = G::default();
        graph.add_edge((0, 1)); graph.add_edge((1, 2));
        let complement = graph.complement(G::default);
        let mut test_complement = G::default();
        test_complement.add_edge((0, 2)); test_complement.add_vertex(1);
        assert!(graphs_eq(&complement, &test_complement));
    }

    pub fn digraph_complement_test<G: GraphOps+Default>(){
        // Complement
        let mut graph = G::default();
        graph.add_edge((0, 1)); graph.add_edge((1, 2));
        let complement = graph.complement(G::default);
        let mut test_complement = G::default();
        test_complement.add_edge((1, 0)); test_complement.add_edge((2, 1));
        test_complement.add_edge((0, 2)); test_complement.add_edge((2, 0));
        assert!(graphs_eq(&complement, &test_complement));
    }

    /// Assures SimpleGraphs Ops (Join, Product, Complement) work
    pub fn simple_graph_ops_test<G: SimpleGraphOps+Default>(){
        let mut line = G::default();
        line.add_vertex(0); line.add_vertex(1);
        // Join
        let (join, map_a, map_b) = line.join(&line, G::default);
        let mut test_join = G::default();
        test_join.add_edge((*map_a.get(&0).unwrap(), *map_b.get(&0).unwrap()));
        test_join.add_edge((*map_a.get(&0).unwrap(), *map_b.get(&1).unwrap()));
        test_join.add_edge((*map_a.get(&1).unwrap(), *map_b.get(&0).unwrap()));
        test_join.add_edge((*map_a.get(&1).unwrap(), *map_b.get(&1).unwrap()));
        assert!(graphs_eq(&join, &test_join));
        // Product
        line.add_edge((0, 1));
        let (square, map) = line.product(&line, G::default);
        let mut test_square = G::default();
        test_square.add_edge((*map.get(&(0, 0)).unwrap(), *map.get(&(0, 1)).unwrap()));
        test_square.add_edge((*map.get(&(1, 0)).unwrap(), *map.get(&(1, 1)).unwrap()));
        test_square.add_edge((*map.get(&(0, 0)).unwrap(), *map.get(&(1, 0)).unwrap()));
        test_square.add_edge((*map.get(&(0, 1)).unwrap(), *map.get(&(1, 1)).unwrap()));
        assert!(graphs_eq(&square, &test_square));
    }
}