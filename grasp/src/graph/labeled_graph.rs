use std::collections::HashMap;

use crate::graph::prelude::*;

/// Graphs that allow accessing labels of vertices and edges
pub trait LabeledGraph: GraphTrait{
    type VertexData;
    type EdgeData;

    /// Returns the label of a vertex if one exists
    fn get_vertex_label(&self, v: VertexID) -> Option<&Self::VertexData>;
    /// Returns the label of an edge if one exists
    fn get_edge_label(&self, e: EdgeID) -> Option<&Self::EdgeData>;
    /// Iterates over vertices with labels
    fn vertex_labels(&self) -> impl Iterator<Item=(VertexID, &Self::VertexData)>;
    /// Iterates over edges with labels
    fn edge_labels(&self) -> impl Iterator<Item=(EdgeID, &Self::EdgeData)>;
}
/// Graphs that allow mutating the labels of vertices and edges
pub trait LabeledGraphMut: LabeledGraph{
    /// Sets the vertex label of v, returns an optionally replaced label. May panic if v is not in the graph.
    fn set_vertex_label(&mut self, v: VertexID, label: Self::VertexData) -> Option<Self::VertexData>;
    /// Sets the edge label of e, returns an optionally replaced label. May panic if e is not in the graph.
    fn set_edge_label(&mut self, e: EdgeID, label: Self::EdgeData) -> Option<Self::EdgeData>;
    /// Sets a collection of vertex labels at once. May panic if a vertex is not in the graph.
    fn set_vertex_labels(&mut self, labels: impl IntoIterator<Item = (VertexID, Self::VertexData)>);
    /// Sets a collection of edge labels at once. May panic if an edge is not in the graph.
    fn set_edge_labels(&mut self, labels: impl IntoIterator<Item = (EdgeID, Self::EdgeData)>);
    /// Labels all vertices in the graph by calling a function with its id that returns a label.
    fn fill_vertex_labels(&mut self, labeler: impl FnMut(VertexID) -> Option<Self::VertexData>);
    /// Labels all edges in the graph by calling a function with its id that returns a label.
    fn fill_edge_labels(&mut self, labeler: impl FnMut(EdgeID) -> Option<Self::EdgeData>);
    /// Removes the label of a vertex, and returns it if it existed.
    fn remove_vertex_label(&mut self, v: VertexID) -> Option<Self::VertexData>;
    /// Removes the label of an edge and returns it if it existed.
    fn remove_edge_label(&mut self, e: EdgeID) -> Option<Self::EdgeData>;
}

/// Basic implementation of a labeled graph which stores labels in a std HashMap
#[derive(Debug, GraphOps, SimpleGraphOps)]
#[graph_ops(labeled)]
pub struct HashMapLabeledGraph<G, V=(), E=()> where G: GraphTrait {
    pub graph: G,
    pub vertex_labels: HashMap<VertexID, V>,
    pub edge_labels: HashMap<EdgeID, E>
}
impl<G: GraphTrait+Default, V, E> Default for HashMapLabeledGraph<G, V, E>{
    fn default() -> Self {
        Self{graph: G::default(), vertex_labels: HashMap::default(), edge_labels: HashMap::default()}
    }
}
impl<G: GraphTrait+SimpleGraph, V, E> SimpleGraph for HashMapLabeledGraph<G, V, E>{}

impl<G: GraphTrait, V, E> GraphTrait for HashMapLabeledGraph<G, V, E>{
    fn vertex_count(&self) -> usize {self.graph.vertex_count()}
    fn edge_count(&self) -> usize {self.graph.edge_count()}
    fn has_vertex(&self, v: VertexID) -> bool {self.graph.has_vertex(v)}
    fn has_edge(&self, e: EdgeID) -> bool {self.graph.has_edge(e)}
    fn vertices(&self) -> impl Iterator<Item=VertexID> {self.graph.vertices()}
    fn edges(&self) -> impl Iterator<Item=EdgeID> {self.graph.edges()}
    fn neighbors(&self, v: VertexID) -> impl Set<Item = VertexID> {self.graph.neighbors(v)}
    fn vertex_set(&self) -> impl Set<Item = VertexID> {self.graph.vertex_set()}
}
impl<G: GraphMut, V, E> GraphMut for HashMapLabeledGraph<G, V, E>{
    fn create_vertex(&mut self) -> VertexID {self.graph.create_vertex()}
    
    fn remove_vertex(&mut self, v: VertexID) -> impl Iterator<Item = EdgeID> {
        let edges = self.graph.remove_vertex(v);
        self.vertex_labels.remove(&v);
        self.edge_labels.retain(|(v1, v2), _| *v1!=v && *v2!=v);
        edges
    }
    
    fn try_add_edge(&mut self, edge: EdgeID) -> Result<(), GraphError> {self.graph.try_add_edge(edge)}
    
    fn remove_edge(&mut self, e: EdgeID) -> bool {
        if self.graph.remove_edge(e) {
            self.edge_labels.remove(&e.to_simple());
            // Ensures correct behaviour with digraphs
            if !self.graph.has_edge(e.inv()) {self.edge_labels.remove(&e.inv());}
            true
        }else {false}
    }
}
impl<G: ArbitraryIDGraph, V, E> ArbitraryIDGraph for HashMapLabeledGraph<G, V, E>{
    fn add_vertex(&mut self, id: VertexID) {self.graph.add_vertex(id);}
}

impl<G: GraphTrait, V, E> LabeledGraph for HashMapLabeledGraph<G, V, E>{
    type VertexData = V;
    type EdgeData = E;

    fn get_vertex_label(&self, v: VertexID) -> Option<&Self::VertexData> {
        self.vertex_labels.get(&v)
    }

    fn get_edge_label(&self, e: EdgeID) -> Option<&Self::EdgeData> {
        self.edge_labels.get(&e.to_simple())
    }

    fn vertex_labels(&self) -> impl Iterator<Item=(VertexID, &Self::VertexData)> {
        self.vertex_labels.iter().map(|(v, l)| (*v, l))
    }

    fn edge_labels(&self) -> impl Iterator<Item=(EdgeID, &Self::EdgeData)> {
        self.edge_labels.iter().map(|(e, l)|(*e, l))
    }
}
impl<G: GraphTrait, V, E> LabeledGraphMut for HashMapLabeledGraph<G, V, E>{
    fn set_vertex_label(&mut self, vertex: VertexID, label: Self::VertexData) -> Option<Self::VertexData> {
        assert!(self.graph.has_vertex(vertex));
        self.vertex_labels.insert(vertex, label)
    }
    fn set_edge_label(&mut self, edge: EdgeID, label: Self::EdgeData) -> Option<Self::EdgeData> {
        assert!(self.graph.has_edge(edge));
        self.edge_labels.insert(edge.to_simple(), label)
    }
    fn set_vertex_labels(&mut self, labels: impl IntoIterator<Item = (VertexID, Self::VertexData)>) {
        for (v, l) in labels{
            self.set_vertex_label(v, l);
        }
    }
    fn set_edge_labels(&mut self, labels: impl IntoIterator<Item = (EdgeID, Self::EdgeData)>) {
        for (e, l) in labels{
            self.set_edge_label(e.to_simple(), l);
        }
    }
    fn fill_vertex_labels(&mut self, mut labeler: impl FnMut(VertexID) -> Option<Self::VertexData>) {
        for vertex in self.graph.vertices() {
            let Some(label) = labeler(vertex) else {continue;};
            self.vertex_labels.insert(vertex, label);
        }
    }
    fn fill_edge_labels(&mut self, mut labeler: impl FnMut(EdgeID) -> Option<Self::EdgeData>) {
        for edge in self.graph.edges().map(|e| e.to_owned()) {
            let Some(label) = labeler(edge) else {continue;};
            self.edge_labels.insert(edge, label);
        }
    }
    fn remove_vertex_label(&mut self, v: VertexID) -> Option<Self::VertexData> {
        self.vertex_labels.remove(&v)
    }
    fn remove_edge_label(&mut self, e: EdgeID) -> Option<Self::EdgeData> {
        self.edge_labels.remove(&e.to_simple())
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
        let edges: HashSet<_> = graph.remove_vertex(1).collect();
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
        dot.add_vertex(0); line.add_edge((0, 1));
        dot.set_vertex_label(0, 1_u8);
        line.set_edge_label((0, 1), 5.0);
        line.set_vertex_label(1, 8_u8);
        // Test graph ops
        // Merged
        let (merged, map_dot, map_line) = dot.merge(&line);
        let mut test_merged = TestGraph::default();
        test_merged.add_edge((*map_line.get(&0).unwrap(), *map_line.get(&1).unwrap()));
        test_merged.add_vertex(*map_dot.get(&0).unwrap());
        test_merged.set_vertex_label(*map_dot.get(&0).unwrap(), 1_u8);
        test_merged.set_edge_label((*map_line.get(&0).unwrap(), *map_line.get(&1).unwrap()), 5.0);
        test_merged.set_vertex_label(*map_line.get(&1).unwrap(), 8_u8);
        assert!(labeled_graphs_eq(&merged, &test_merged));
        // Subgraph
        let mut triangle = TestGraph::default();
        triangle.add_edge((1, 0));triangle.add_edge((2, 1));triangle.add_edge((0, 2));
        triangle.set_edge_labels([((1, 0), 1.0), ((2, 1), 2.0), ((0, 2), 3.0)]);
        triangle.set_vertex_labels([(0, 1_u8), (1, 2_u8), (2, 3_u8)]);
        let subgraph_vertex = triangle.subgraph_vertex([0, 1]);
        let subgraph_edge = triangle.subgraph_edges([(0, 1)]);
        let mut test_subgraph = TestGraph::default();
        test_subgraph.add_edge((1, 0));
        test_subgraph.set_edge_label((1, 0), 1.0);
        test_subgraph.set_vertex_labels([(0, 1_u8), (1, 2_u8)]);
        assert!(labeled_graphs_eq(&subgraph_vertex, &test_subgraph));
        assert!(labeled_graphs_eq(&subgraph_edge, &test_subgraph));
        // join
        let (join, map_dot, map_line) = dot.join(&line);
        let mut test_join = TestGraph::default();
        test_join.add_edge((*map_line.get(&0).unwrap(), *map_line.get(&1).unwrap()));
        test_join.add_vertex(*map_dot.get(&0).unwrap());
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
        test_product.set_edge_label((*map.get(&(0, 0)).unwrap(), *map.get(&(1, 0)).unwrap()), 5.0);
        test_product.set_edge_label((*map.get(&(0, 1)).unwrap(), *map.get(&(1, 1)).unwrap()), 5.0);
        test_product.set_edge_label((*map.get(&(0, 0)).unwrap(), *map.get(&(0, 1)).unwrap()), 5.0);
        test_product.set_edge_label((*map.get(&(1, 0)).unwrap(), *map.get(&(1, 1)).unwrap()), 5.0);
        assert!(labeled_graphs_eq(&product, &test_product));
        // complement
        line.add_edge((1, 2));
        line.set_edge_label((1, 2), 10.0);
        let complement = line.complement();
        let mut test_complement = TestGraph::default();
        test_complement.add_vertex(1);
        test_complement.set_vertex_label(1, 8_u8);
        test_complement.add_edge((0, 2));
        assert!(labeled_graphs_eq(&complement, &test_complement));
    }
}