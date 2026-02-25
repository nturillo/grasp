use crate::graph::set::Set;

use super::{GraphTrait, VertexID, VertexMap, EdgeID, graph_ops::*, DiGraph, SimpleGraph};
use std::collections::HashMap;

/// Graphs that allow setting labels to vertices and edges
pub trait LabeledGraph{
    type VertexData: Clone;
    type EdgeData: Clone;
    type GraphType: GraphTrait;

    fn get_vertex_label(&self, v: VertexID) -> Option<&Self::VertexData>;
    fn get_edge_label(&self, e: EdgeID) -> Option<&Self::EdgeData>;
    fn vertex_labels(&self) -> impl Iterator<Item=(&VertexID, &Self::VertexData)>;
    fn edge_labels(&self) -> impl Iterator<Item=(&EdgeID, &Self::EdgeData)>;
    fn set_vertex_label(&mut self, v: VertexID, label: Self::VertexData) -> Option<Self::VertexData>;
    fn set_edge_label(&mut self, e: EdgeID, label: Self::EdgeData) -> Option<Self::EdgeData>;
    fn set_vertex_labels(&mut self, labels: impl IntoIterator<Item = (VertexID, Self::VertexData)>);
    fn set_edge_labels(&mut self, labels: impl IntoIterator<Item = (EdgeID, Self::EdgeData)>);
    fn fill_vertex_labels(&mut self, labeler: impl FnMut(VertexID) -> Option<Self::VertexData>);
    fn fill_edge_labels(&mut self, labeler: impl FnMut(EdgeID) -> Option<Self::EdgeData>);

    fn remove_vertex_label(&mut self, v: VertexID) -> Option<Self::VertexData>;
    fn remove_edge_label(&mut self, e: EdgeID) -> Option<Self::EdgeData>;

    fn from_graph(graph: Self::GraphType) -> Self;
    fn to_graph(self) -> Self::GraphType;
    fn graph(&self) -> &Self::GraphType;
    fn graph_mut(&mut self) -> &mut Self::GraphType;
}
impl<G: LabeledGraph> GraphTrait for G{
    fn vertex_count(&self) -> usize {self.graph().vertex_count()}
    fn edge_count(&self) -> usize {self.graph().edge_count()}
    fn vertices(&self) -> impl Iterator<Item=VertexID> {self.graph().vertices()}
    fn edges(&self) -> impl Iterator<Item=EdgeID> {self.graph().edges()}
    fn contains(&self, v: VertexID) -> bool {self.graph().contains(v)}
    fn has_edge(&self, e: EdgeID) -> bool {self.graph().has_edge(e)}
    fn neighbors(&self, v: VertexID) -> Option<impl Set<Item=VertexID>> {self.graph().neighbors(v)}
    fn vertex_set(&self) -> impl Set<Item=VertexID> {self.graph().vertex_set()}
    fn create_vertex(&mut self) -> VertexID {self.graph_mut().create_vertex()}
    fn add_vertex(&mut self, v: VertexID) {self.graph_mut().add_vertex(v)}
    fn add_edge(&mut self, e: EdgeID) {self.graph_mut().add_edge(e)}
    fn add_neighbors(&mut self, v: VertexID, nbhrs: impl Iterator<Item=VertexID>) {self.graph_mut().add_neighbors(v, nbhrs)}

    fn delete_vertex(&mut self, v: VertexID) -> impl Iterator<Item = EdgeID> {
        let edges: Vec<EdgeID> = self.graph_mut().delete_vertex(v).collect();
        for e in edges.iter() {
            // delete edge data
            self.remove_edge_label(*e);
        }
        self.remove_vertex_label(v);
        edges.into_iter()
    }
    fn delete_edge(&mut self, e: EdgeID) {
        self.graph_mut().delete_edge(e);
        self.remove_edge_label(e);
    }
}
impl<G: LabeledGraph> GraphOps for G where G::GraphType: GraphOps{
    fn build_subgraph_vertex(&self, vertices: impl IntoIterator<Item=VertexID>, subgraph: &mut Self) {
        self.graph().build_subgraph_vertex(vertices, subgraph.graph_mut());
        subgraph.fill_vertex_labels(|vertex| self.get_vertex_label(vertex).cloned());
        subgraph.fill_edge_labels(|edge| self.get_edge_label(edge).cloned());
    }

    fn build_subgraph_edges(&self, edges: impl IntoIterator<Item=EdgeID>, subgraph: &mut Self) {
        self.graph().build_subgraph_edges(edges, subgraph.graph_mut());
        subgraph.fill_vertex_labels(|vertex| self.get_vertex_label(vertex).cloned());
        subgraph.fill_edge_labels(|edge| self.get_edge_label(edge).cloned());
    }

    fn build_merge(&self, other: &Self, merged: &mut Self) -> (VertexMap, VertexMap) {
        let (self_map, other_map) = self.graph().build_merge(other.graph(), merged.graph_mut());

        merged.set_vertex_labels(self.vertex_labels().filter_map(|(vertex, label)|
            Some((*self_map.get(vertex)?, label.clone()))
        ));
        merged.set_vertex_labels(other.vertex_labels().filter_map(|(vertex, label)|
            Some((*other_map.get(vertex)?, label.clone()))
        ));
        merged.set_edge_labels(self.edge_labels().filter_map(|((v1, v2), label)|
            Some(((*self_map.get(v1)?, *self_map.get(v2)?), label.clone()))
        ));
        merged.set_edge_labels(other.edge_labels().filter_map(|((v1, v2), label)|
            Some(((*other_map.get(v1)?, *other_map.get(v2)?), label.clone()))
        ));
        (self_map, other_map)
    }

    fn build_complement(&self, complement: &mut Self) {
        self.graph().build_complement(complement.graph_mut());
        complement.set_vertex_labels(self.vertex_labels().map(|(v, l)| (*v, l.clone())));
    }
}
impl<G: LabeledGraph> SimpleGraphOps for G where G::GraphType: SimpleGraphOps{
    fn build_join(&self, other: &Self, joined: &mut Self) -> (VertexMap, VertexMap) {
        let (self_map, other_map) = self.graph().build_join(other.graph(), joined.graph_mut());
        joined.set_vertex_labels(self.vertex_labels().filter_map(|(vertex, label)|
            Some((*self_map.get(vertex)?, label.clone()))
        ));
        joined.set_vertex_labels(other.vertex_labels().filter_map(|(vertex, label)|
            Some((*other_map.get(vertex)?, label.clone()))
        ));
        joined.set_edge_labels(self.edge_labels().filter_map(|((v1, v2), label)|
            Some(((*self_map.get(v1)?, *self_map.get(v2)?), label.clone()))
        ));
        joined.set_edge_labels(other.edge_labels().filter_map(|((v1, v2), label)|
            Some(((*other_map.get(v1)?, *other_map.get(v2)?), label.clone()))
        ));
        (self_map, other_map)
    }

    fn build_product(&self, other: &Self, product: &mut Self) -> HashMap<(VertexID, VertexID), VertexID> {
        let map = self.graph().build_product(other.graph(), product.graph_mut());
        // invert map
        let inv_map: HashMap<VertexID, (VertexID, VertexID)> = map.iter().map(|(k, v)| (*v, *k)).collect();
        product.fill_edge_labels(|(v1, v2)| {
            let Some((s1, o1)) = inv_map.get(&v1) else {return None;};
            let Some((s2, o2)) = inv_map.get(&v2) else {return None;};
            if s1==s2 && o1==o2 {return None;}
            if s1==s2 { // Edge from other
                return other.get_edge_label((*o1, *o2)).cloned();
            } else if o1==o2 { // Edge from self
                return self.get_edge_label((*s1, *s2)).cloned();
            }
            None
        });
        map
    }
}
impl<G: LabeledGraph> SimpleGraph for G where G::GraphType: SimpleGraph{}
impl<G: LabeledGraph> DiGraph for G where G::GraphType: DiGraph{
    fn in_neighbors(&self, v: VertexID) -> Option<impl Set<Item=VertexID>> {
        self.graph().in_neighbors(v)
    }
    fn out_neighbors(&self, v: VertexID) -> Option<impl Set<Item=VertexID>> {
        self.graph().in_neighbors(v)
    }
}

/// Basic implementation of a labeled graph which stores labels in a std HashMap
#[derive(Debug)]
pub struct HashMapLabeledGraph<G, V=(), E=()> where G: SimpleGraph {
    pub graph: G,
    pub vertex_labels: HashMap<VertexID, V>,
    pub edge_labels: HashMap<EdgeID, E>
}
impl<G: SimpleGraph+Default, V, E> Default for HashMapLabeledGraph<G, V, E>{
    fn default() -> Self {
        Self{graph: G::default(), vertex_labels: HashMap::default(), edge_labels: HashMap::default()}
    }
}
impl<G: SimpleGraph, V, E> AsRef<G> for HashMapLabeledGraph<G, V, E>{
    fn as_ref(&self) -> &G {
        &self.graph
    }
}
impl<G: SimpleGraph, V, E> AsMut<G> for HashMapLabeledGraph<G, V, E>{
    fn as_mut(&mut self) -> &mut G {
        &mut self.graph
    }
}
impl<G: SimpleGraph, V: Clone, E: Clone> LabeledGraph for HashMapLabeledGraph<G, V, E>{
    type VertexData = V;
    type EdgeData = E;
    type GraphType = G;

    fn from_graph(graph: Self::GraphType) -> Self {
        Self{graph, vertex_labels: HashMap::default(), edge_labels: HashMap::default()}
    }
    fn to_graph(self) -> Self::GraphType {
        self.graph
    }
    fn graph(&self) -> &Self::GraphType {
        &self.graph
    }
    fn graph_mut(&mut self) -> &mut Self::GraphType {
        &mut self.graph
    }

    fn get_vertex_label(&self, vertex: VertexID) -> Option<&Self::VertexData> {
        if self.graph.contains(vertex) {
            Some(self.vertex_labels.get(&vertex)?)
        }else {None}
    }
    fn get_edge_label(&self, edge: EdgeID) -> Option<&Self::EdgeData> {
        if self.graph.has_edge(edge) {
            Some(self.edge_labels.get(&edge).or(self.edge_labels.get(&(edge.1, edge.0)))?)
        }else {None}
    }
    fn vertex_labels(&self) -> impl Iterator<Item=(&VertexID, &Self::VertexData)> {
        self.vertex_labels.iter()
    }
    fn edge_labels(&self) -> impl Iterator<Item=(&EdgeID, &Self::EdgeData)> {
        self.edge_labels.iter()
    }
    fn set_vertex_label(&mut self, vertex: VertexID, label: Self::VertexData) -> Option<Self::VertexData> {
        if !self.graph.contains(vertex) {self.graph.add_vertex(vertex);}
        self.vertex_labels.insert(vertex, label)
    }
    fn set_edge_label(&mut self, edge: EdgeID, label: Self::EdgeData) -> Option<Self::EdgeData> {
        if !self.graph.has_edge(edge) {self.graph.add_edge(edge);}
        self.edge_labels.insert(edge, label)
    }
    fn set_vertex_labels(&mut self, labels: impl IntoIterator<Item = (VertexID, Self::VertexData)>) {
        for (v, l) in labels{
            if !self.contains(v) {self.add_vertex(v);}
            self.set_vertex_label(v, l);
        }
    }
    fn set_edge_labels(&mut self, labels: impl IntoIterator<Item = (EdgeID, Self::EdgeData)>) {
        for (e, l) in labels{
            if !self.has_edge(e) {self.add_edge(e);}
            self.set_edge_label(e, l);
        }
    }
    fn fill_vertex_labels(&mut self, mut labeler: impl FnMut(VertexID) -> Option<Self::VertexData>) {
        for vertex in self.graph.vertices() {
            let Some(label) = labeler(vertex) else {continue;};
            self.vertex_labels.insert(vertex, label);
        }
    }
    fn fill_edge_labels(&mut self, mut labeler: impl FnMut(EdgeID) -> Option<Self::EdgeData>) {
        for edge in self.graph.edges() {
            let Some(label) = labeler(edge) else {continue;};
            self.edge_labels.insert(edge, label);
        }
    }

    fn remove_vertex_label(&mut self, v: VertexID) -> Option<Self::VertexData> {
        self.vertex_labels.remove(&v)
    }
    fn remove_edge_label(&mut self, e: EdgeID) -> Option<Self::EdgeData> {
        self.edge_labels.remove(&e)
    }
}

#[cfg(test)]
mod test{
    use std::collections::HashSet;
    use crate::graph::prelude::*;

    /// ensures basic labeled graph functionality works
    #[test]
    fn hashmap_labeled_graph_test(){
        let mut graph = HashMapLabeledGraph::<SparseSimpleGraph, u8, f32>::default();
        graph.add_edge((0, 1)); graph.add_edge((1, 2));
        graph.set_vertex_label(1, 3_u8);
        graph.set_edge_label((1, 2), 3.14);
        assert_eq!(graph.get_edge_label((0, 1)), None);
        assert_eq!(graph.get_vertex_label(0), None);
        assert_eq!(graph.get_vertex_label(2), None);
        assert_eq!(graph.get_vertex_label(1), Some(&3_u8));
        assert_eq!(graph.get_edge_label((1, 2)), Some(&3.14));
        let edges: HashSet<EdgeID> = graph.delete_vertex(1).collect();
        assert!(edges.len()==2);
        assert_eq!(graph.get_vertex_label(1), None);
        assert_eq!(graph.get_edge_label((1, 2)), None);
    }

    /// Ensures graphops work with labels being repositioned after operations
    #[test]
    fn hashmap_labeled_graphops_test(){
        type TestGraph = HashMapLabeledGraph<SparseSimpleGraph, u8, f32>;
        let mut dot = TestGraph::default();
        let mut line = TestGraph::default();
        dot.set_vertex_label(0, 1_u8);
        line.set_edge_label((0, 1), 5.0);
        line.set_vertex_label(1, 8_u8);
        // Test graph ops
        // Merged
        let (merged, map_dot, map_line) = dot.merge(&line);
        let mut test_merged = TestGraph::default();
        test_merged.set_vertex_label(*map_dot.get(&0).unwrap(), 1_u8);
        test_merged.set_edge_label((*map_line.get(&0).unwrap(), *map_line.get(&1).unwrap()), 5.0);
        test_merged.set_vertex_label(*map_line.get(&1).unwrap(), 8_u8);
        assert!(labeled_graphs_eq(&merged, &test_merged));
        // Subgraph
        let mut triangle = TestGraph::default();
        triangle.set_edge_labels([((1, 0), 1.0), ((2, 1), 2.0), ((0, 2), 3.0)]);
        triangle.set_vertex_labels([(0, 1_u8), (1, 2_u8), (2, 3_u8)]);
        let subgraph_vertex = triangle.subgraph_vertex([0, 1]);
        let subgraph_edge = triangle.subgraph_edges([(0, 1)]);
        let mut test_subgraph = TestGraph::default();
        test_subgraph.set_edge_label((1, 0), 1.0);
        test_subgraph.set_vertex_labels([(0, 1_u8), (1, 2_u8)]);
        assert!(labeled_graphs_eq(&subgraph_vertex, &test_subgraph));
        assert!(labeled_graphs_eq(&subgraph_edge, &test_subgraph));
        // join
        let (join, map_dot, map_line) = dot.join(&line);
        let mut test_join = TestGraph::default();
        test_join.set_vertex_label(*map_dot.get(&0).unwrap(), 1_u8);
        test_join.set_vertex_label(*map_line.get(&1).unwrap(), 8_u8);
        test_join.set_edge_label((*map_line.get(&0).unwrap(), *map_line.get(&1).unwrap()), 5.0);
        test_join.add_edge((*map_dot.get(&0).unwrap(), *map_line.get(&0).unwrap()));
        test_join.add_edge((*map_dot.get(&0).unwrap(), *map_line.get(&1).unwrap()));
        assert!(labeled_graphs_eq(&join, &test_join));
        // product
        let (product, map) = line.product(&line);
        let mut test_product = TestGraph::default();
        test_product.add_edge((*map.get(&(0, 0)).unwrap(), *map.get(&(1, 0)).unwrap()));
        test_product.add_edge((*map.get(&(0, 1)).unwrap(), *map.get(&(1, 1)).unwrap()));
        test_product.add_edge((*map.get(&(0, 0)).unwrap(), *map.get(&(0, 1)).unwrap()));
        test_product.add_edge((*map.get(&(1, 0)).unwrap(), *map.get(&(1, 1)).unwrap()));
        assert!(labeled_graphs_eq(&product, &test_product));
        // complement
        line.set_edge_label((1, 2), 10.0);
        let complement = line.complement();
        let mut test_complement = TestGraph::default();
        test_complement.set_vertex_label(1, 8_u8);
        test_complement.add_edge((0, 2));
        assert!(labeled_graphs_eq(&complement, &test_complement));
    }
}