use std::collections::{HashMap, HashSet};
use crate::graph::{labeled_graph::LabeledGraphMut, prelude::*};

/*
    Implementation of GraphOps on types can be accomplished by using the #[derive(GraphOps, SimpleGraphOps)] macro
    Use #[graph_ops(labeled)] if the graph is labeled, to use the correct implementations
*/

/// Graph operations that are agnostic to simple graphs and digraphs
/// graph_builder is a FnOnce which creates a Graph to store the result in, Default::default works for graphs that are Default.
pub trait GraphOps: GraphTrait+BuildableGraph+Sized{
    /// Subgraph from a set of vertices
    fn subgraph_vertex(&self, vertices: impl IntoIterator<Item=VertexID>) -> Self;

    /// Subgraph from a set of edges
    fn subgraph_edges(&self, edges: impl IntoIterator<Item=EdgeID>) -> Self;

    /// Combines two graphs into one without any new edges. Also returns two maps (self vertex) -> new vertex, and (other vertex) -> new vertex
    fn merge(&self, other: &Self) -> (Self, VertexMap, VertexMap);

    /// Returns the complement of a graph
    fn complement(&self) -> Self;
}

/// Graph operations that only work for simple graphs
pub trait SimpleGraphOps: GraphOps+SimpleGraph{
    /// Returns the join of two graphs. Also returns two maps (self vertex) -> new vertex and (other vertex) -> new vertex
    fn join(&self, other: &Self) -> (Self, VertexMap, VertexMap);

    /// Returns the cartesian product of two graphs. Also returns a map from (self vertex, other vertex) -> new vertex
    fn product(&self, other: &Self) -> (Self, HashMap<(VertexID, VertexID), VertexID>);
}

/// Places subgraph from a set of vertices into subgraph
pub fn build_subgraph_vertex<G: AnyVertexGraph>(graph: &G, vertices: impl IntoIterator<Item=VertexID>, subgraph: &mut G) {
    for vertex in vertices{
        subgraph.add_vertex(vertex);
    }
    for (v1, v2) in graph.edges(){
        if subgraph.has_vertex(v1) && subgraph.has_vertex(v2) {
            subgraph.add_edge((v1, v2));
        }
    }
}
pub fn build_subgraph_vertex_labeled<G: LabeledGraphMut+AnyVertexGraph>(graph: &G, vertices: impl IntoIterator<Item=VertexID>, subgraph: &mut G)
where G::EdgeData: Clone, G::VertexData: Clone{
    for vertex in vertices{
        subgraph.add_vertex(vertex);
        if let Some(label) = graph.get_vertex_label(vertex){
            subgraph.set_vertex_label(vertex, label.clone());
        }
    }
    for edge in graph.edges(){
        if subgraph.has_vertex(edge.0) && subgraph.has_vertex(edge.1) {
            subgraph.add_edge(edge);
            if let Some(label) = graph.get_edge_label(edge) {
                subgraph.set_edge_label(edge, label.clone());
            }
        }
    }
}

/// Places subgraph from a set of edges into subgraph
pub fn build_subgraph_edges<G: AnyVertexGraph>(graph: &G, edges: impl IntoIterator<Item=EdgeID>, subgraph: &mut G) {
    for edge in edges{
        if !graph.has_edge(edge) {continue;}
        subgraph.add_edge(edge);
    }
}
pub fn build_subgraph_edges_labeled<G: LabeledGraphMut+AnyVertexGraph>(graph: &G, edges: impl IntoIterator<Item=EdgeID>, subgraph: &mut G)
where G::EdgeData: Clone, G::VertexData: Clone{
    for edge in edges{
        if !graph.has_edge(edge) {continue;}
        subgraph.add_edge(edge);
        if let Some(label) = graph.get_edge_label(edge){
            subgraph.set_edge_label(edge, label.clone());
        }
    }
    subgraph.fill_vertex_labels(|vertex| graph.get_vertex_label(vertex).cloned());
}

/// Places the combination of self and other into merged. Also returns two maps (self vertex) -> new vertex, and (other vertex) -> new vertex
pub fn build_merge<G: AnyVertexGraph>(graph: &G, other: &G, merged: &mut G) -> (VertexMap, VertexMap) {
    let mut self_map = HashMap::default();
    let mut other_map = HashMap::default();
    // vertices
    for v in graph.vertices() {
        let new_vertex = merged.create_vertex();
        self_map.insert(v, new_vertex);
    }
    for v in other.vertices() {
        let new_vertex = merged.create_vertex();
        other_map.insert(v, new_vertex);
    }
    // edges
    for (v1, v2) in graph.edges() {
        let Some(v1) = self_map.get(&v1) else {continue;};
        let Some(v2) = self_map.get(&v2) else {continue;};
        merged.add_edge((*v1, *v2));
    }
    for (v1, v2) in other.edges() {
        let Some(v1) = other_map.get(&v1) else {continue;};
        let Some(v2) = other_map.get(&v2) else {continue;};
        merged.add_edge((*v1, *v2));
    }
    (self_map, other_map)
}
pub fn build_merge_labeled<G: LabeledGraphMut+AnyVertexGraph>(graph: &G, other: &G, merged: &mut G) -> (VertexMap, VertexMap)
where G::EdgeData: Clone, G::VertexData: Clone{
    let mut self_map = HashMap::default();
    let mut other_map = HashMap::default();
    // vertices
    for v in graph.vertices() {
        let new_vertex = merged.create_vertex();
        self_map.insert(v, new_vertex);
        if let Some(label) = graph.get_vertex_label(v){
            merged.set_vertex_label(new_vertex, label.clone());
        }
    }
    for v in other.vertices() {
        let new_vertex = merged.create_vertex();
        other_map.insert(v, new_vertex);
        if let Some(label) = other.get_vertex_label(v){
            merged.set_vertex_label(new_vertex, label.clone());
        }
    }
    // edges
    for (s1, s2) in graph.edges() {
        let Some(v1) = self_map.get(&s1) else {continue;};
        let Some(v2) = self_map.get(&s2) else {continue;};
        merged.add_edge((*v1, *v2));
        if let Some(label) = graph.get_edge_label((s1, s2)){
            merged.set_edge_label((*v1, *v2), label.clone());
        }
    }
    for (o1, o2) in other.edges() {
        let Some(v1) = other_map.get(&o1) else {continue;};
        let Some(v2) = other_map.get(&o2) else {continue;};
        merged.add_edge((*v1, *v2));
        if let Some(label) = other.get_edge_label((o1, o2)){
            merged.set_edge_label((*v1, *v2), label.clone());
        }
    }
    (self_map, other_map)
}

/// Places the complement of self into complement
pub fn build_complement<G: AnyVertexGraph>(graph: &G, complement: &mut G) {
    for v1 in graph.vertices(){
        complement.add_vertex(v1);
        for v2 in graph.vertices(){
            if v1==v2 {continue;} // Complement ignores loops
            if graph.has_edge((v1, v2)){continue;}
            complement.add_edge((v1, v2));
        }
    }
}
pub fn build_complement_labeled<G: LabeledGraphMut+AnyVertexGraph>(graph: &G, complement: &mut G)
where G::EdgeData: Clone, G::VertexData: Clone{
    for v1 in graph.vertices(){
        complement.add_vertex(v1);
        if let Some(label) = graph.get_vertex_label(v1) {
            complement.set_vertex_label(v1, label.clone());
        }
        for v2 in graph.vertices(){
            if v1==v2 {continue;} // Complement ignores loops
            if graph.has_edge((v1, v2)){continue;}
            complement.add_edge((v1, v2));
            if let Some(label) = graph.get_edge_label((v1, v2)){
                complement.set_edge_label((v1, v2), label.clone());
            }
        }
    }
}

/// Places the join of two graphs into joined. Also returns two maps (self vertex) -> new vertex and (other vertex) -> new vertex
pub fn build_join<G: AnyVertexGraph+SimpleGraph>(graph: &G, other: &G, joined: &mut G) -> (VertexMap, VertexMap) {
    let (self_map, other_map) = build_merge(graph, other, joined);
    for v1 in graph.vertices(){
        let Some(v1) = self_map.get(&v1) else {continue;};
        for v2 in other.vertices(){
            let Some(v2) = other_map.get(&v2) else {continue;};
            joined.add_edge((*v1, *v2));
        }
    }
    (self_map, other_map)
}
pub fn build_join_labeled<G: LabeledGraphMut+AnyVertexGraph+SimpleGraph>(graph: &G, other: &G, joined: &mut G) -> (VertexMap, VertexMap)
where G::EdgeData: Clone, G::VertexData: Clone{
    let (self_map, other_map) = build_merge_labeled(graph, other, joined);
    for v1 in graph.vertices(){
        let Some(v1) = self_map.get(&v1) else {continue;};
        for v2 in other.vertices(){
            let Some(v2) = other_map.get(&v2) else {continue;};
            joined.add_edge((*v1, *v2));
        }
    }
    (self_map, other_map)
}

/// Places the cartesian product of two graphs into product. Also returns a map from (self vertex, other vertex) -> new vertex
pub fn build_product<G: AnyVertexGraph+SimpleGraph>(graph: &G, other: &G, product: &mut G) -> HashMap<(VertexID, VertexID), VertexID> {
    let mut map = HashMap::default();
    // Vertices and map
    for v1 in graph.vertices(){
        for v2 in other.vertices(){
            let v = product.create_vertex();
            map.insert((v1, v2), v);
        }
    }
    // edges part 1
    for (s1, s2) in graph.edges(){
        for o in other.vertices(){
            let Some(v1) = map.get(&(s1, o)) else {continue;};
            let Some(v2) = map.get(&(s2, o)) else {continue;};
            product.add_edge((*v1, *v2));
        }
    }
    // edges part 2
    for (o1, o2) in other.edges(){
        for s in graph.vertices(){
            let Some(v1) = map.get(&(s, o1)) else {continue;};
            let Some(v2) = map.get(&(s, o2)) else {continue;};
            product.add_edge((*v1, *v2));
        }
    }
    map
}
pub fn build_product_labeled<G: LabeledGraphMut+AnyVertexGraph+SimpleGraph>(graph: &G, other: &G, product: &mut G) -> HashMap<(VertexID, VertexID), VertexID>
where G::EdgeData: Clone, G::VertexData: Clone{
    let mut map = HashMap::default();
    // Vertices and map
    for v1 in graph.vertices(){
        for v2 in other.vertices(){
            let v = product.create_vertex();
            map.insert((v1, v2), v);
        }
    }
    // edges part 1
    for (s1, s2) in graph.edges(){
        for o in other.vertices(){
            let Some(v1) = map.get(&(s1, o)) else {continue;};
            let Some(v2) = map.get(&(s2, o)) else {continue;};
            product.add_edge((*v1, *v2));
            if let Some(label) = graph.get_edge_label((s1, s2)){
                product.set_edge_label((*v1, *v2), label.clone());
            }
        }
    }
    // edges part 2
    for (o1, o2) in other.edges(){
        for s in graph.vertices(){
            let Some(v1) = map.get(&(s, o1)) else {continue;};
            let Some(v2) = map.get(&(s, o2)) else {continue;};
            product.add_edge((*v1, *v2));
            if let Some(label) = other.get_edge_label((o1, o2)){
                product.set_edge_label((*v1, *v2), label.clone());
            }
        }
    }
    map
}

pub struct SubgraphView<'a, G: GraphTrait>{
    graph: &'a G,
    vertices: Option<HashSet<VertexID>>,
    edges: Option<HashSet<EdgeID>>,
    directed: bool
}
impl<'a, G: GraphTrait> SubgraphView<'a, G>{
    pub fn new(graph: &'a G, mut vertices: Option<HashSet<VertexID>>, mut edges: Option<HashSet<EdgeID>>, directed: bool) -> Self{
        // Make sure edges and vertices only contain relevent elements
        if let Some(vertices) = vertices.as_mut() {
            vertices.retain(|v| graph.has_vertex(*v));
            if let Some(edges) = edges.as_mut(){
                edges.retain(|e| vertices.contains(&e.0) && vertices.contains(&e.1));
            }
        }
        if let Some(edges) = edges.as_mut() {
            edges.retain(|e| graph.has_edge(*e));
        }
        Self{graph, vertices, edges, directed}
    }

    pub fn underlying(&self) -> &G {
        self.graph
    }
}
impl<'a, G: GraphTrait> GraphTrait for SubgraphView<'a, G>{
    fn vertex_count(&self) -> usize {
        if let Some(vertices) = &self.vertices {
            vertices.len()
        } else {
            self.graph.vertex_count()
        }
    }
    
    fn edge_count(&self) -> usize {
        // Count edges from iterator impl since it is easiest
        self.edges().count()
    }
    
    fn has_vertex(&self, v: VertexID) -> bool {
        if let Some(vertices) = &self.vertices {vertices.contains(&v)}
        else {self.graph.has_vertex(v)}
    }
    
    fn has_edge(&self, e: EdgeID) -> bool {
        if let Some(edges) = &self.edges {
            if !self.directed {edges.contains(&e) || edges.contains(&e.inv())}
            else {edges.contains(&e)}
        }else if let Some(vertices) = &self.vertices{
            if !(vertices.contains(&e.0) && vertices.contains(&e.1)) {false}
            else {self.graph.has_edge(e)}
        } else {
            self.graph.has_edge(e)
        }
    }
    
    fn vertices(&self) -> impl Iterator<Item=VertexID> {
        self.graph.vertices().filter(|v| {
            if let Some(vertices) = &self.vertices {
                vertices.contains(&v)
            }else {
                true
            }
        })
    }
    
    fn edges(&self) -> impl Iterator<Item=EdgeID> {
        self.graph.edges().filter(|e| {
            if let Some(edges) = &self.edges{
                if self.directed {
                    edges.contains(&e)
                } else {
                    edges.contains(&e) || edges.contains(&e.inv())
                }
            }else if let Some(vertices) = &self.vertices {
                vertices.contains(&e.0) && vertices.contains(&e.1)
            }else {
                true
            }
        })
    }
    
    fn neighbors(&self, v: VertexID) -> impl Set<Item = VertexID> {
        self.graph.neighbors(v).filter(move |_, u| {
            if let Some(edges) = &self.edges{
                if self.directed {
                    edges.contains(&(v, *u))
                } else {
                    edges.contains(&(v, *u)) || edges.contains(&(*u, v))
                }
            }else if let Some(vertices) = &self.vertices {
                vertices.contains(&v) && vertices.contains(u)
            }else {
                true
            }
        })
    }
    
    fn vertex_set(&self) -> impl Set<Item = VertexID> {
        self.graph.vertex_set().filter(|_, v| {
            if let Some(vertices) = &self.vertices {
                vertices.contains(v)
            } else {true}
        })
    }
}

#[cfg(test)]
pub mod test{
    use std::collections::HashSet;

    use crate::graph::prelude::*;

    /// Assures Graph Ops functionality
    pub fn graph_ops_test<G: GraphOps+Default+AnyVertexGraph>(){
        let mut graph_a =  G::default();
        graph_a.add_edge((0, 1)); graph_a.add_edge((1, 2)); graph_a.add_edge((2, 0));
        // ensure subgraphs work
        let subgraph_vertices_a = graph_a.subgraph_vertex([0, 1]);
        let subgraph_edges_a = graph_a.subgraph_edges([(0, 1)]);
        let mut test_subgraph = G::default(); test_subgraph.add_edge((0, 1));
        assert!(graphs_eq(&subgraph_edges_a, &test_subgraph));
        assert!(graphs_eq(&subgraph_vertices_a, &test_subgraph));
        // create merged graph manually and test to ensure equality
        let (merged, map_a, map_b) = graph_a.merge(&graph_a);
        let mut test_graph = G::default();
        test_graph.add_edge((*map_a.get(&0).unwrap(), *map_a.get(&1).unwrap()));
        test_graph.add_edge((*map_a.get(&1).unwrap(), *map_a.get(&2).unwrap()));
        test_graph.add_edge((*map_a.get(&2).unwrap(), *map_a.get(&0).unwrap()));
        test_graph.add_edge((*map_b.get(&0).unwrap(), *map_b.get(&1).unwrap()));
        test_graph.add_edge((*map_b.get(&1).unwrap(), *map_b.get(&2).unwrap()));
        test_graph.add_edge((*map_b.get(&2).unwrap(), *map_b.get(&0).unwrap()));
        assert!(graphs_eq(&merged, &test_graph));
        // Test graph components
        let mut disc_graph = G::default();
        disc_graph.add_edge((0, 1)); disc_graph.add_edge((2, 3));
        let components = get_components(&disc_graph);
        let comp_1 = HashSet::from([0, 1]);
        let comp_2 = HashSet::from([2, 3]);
        assert!(components.len()==2);
        assert!(
            comp_1.set_eq(&components[0]) && comp_2.set_eq(&components[1]) ||
            comp_1.set_eq(&components[1]) && comp_2.set_eq(&components[0])
        );
    }

    pub fn simple_graph_complement_test<G: GraphOps+Default+AnyVertexGraph>(){
        // Complement
        let mut graph = G::default();
        graph.add_edge((0, 1)); graph.add_edge((1, 2));
        let complement = graph.complement();
        let mut test_complement = G::default();
        test_complement.add_edge((0, 2)); test_complement.add_vertex(1);
        assert!(graphs_eq(&complement, &test_complement));
    }

    pub fn digraph_complement_test<G: GraphOps+Default+AnyVertexGraph>(){
        // Complement
        let mut graph = G::default();
        graph.add_edge((0, 1)); graph.add_edge((1, 2));
        let complement = graph.complement();
        let mut test_complement = G::default();
        test_complement.add_edge((1, 0)); test_complement.add_edge((2, 1));
        test_complement.add_edge((0, 2)); test_complement.add_edge((2, 0));
        assert!(graphs_eq(&complement, &test_complement));
    }

    /// Assures SimpleGraphs Ops (Join, Product, Complement) work
    pub fn simple_graph_ops_test<G: SimpleGraphOps+Default+AnyVertexGraph>(){
        let mut line = G::default();
        line.add_vertex(0); line.add_vertex(1);
        // Join
        let (join, map_a, map_b) = line.join(&line);
        let mut test_join = G::default();
        test_join.add_edge((*map_a.get(&0).unwrap(), *map_b.get(&0).unwrap()));
        test_join.add_edge((*map_a.get(&0).unwrap(), *map_b.get(&1).unwrap()));
        test_join.add_edge((*map_a.get(&1).unwrap(), *map_b.get(&0).unwrap()));
        test_join.add_edge((*map_a.get(&1).unwrap(), *map_b.get(&1).unwrap()));
        assert!(graphs_eq(&join, &test_join));
        // Product
        line.add_edge((0, 1));
        let (square, map) = line.product(&line);
        let mut test_square = G::default();
        test_square.add_edge((*map.get(&(0, 0)).unwrap(), *map.get(&(0, 1)).unwrap()));
        test_square.add_edge((*map.get(&(1, 0)).unwrap(), *map.get(&(1, 1)).unwrap()));
        test_square.add_edge((*map.get(&(0, 0)).unwrap(), *map.get(&(1, 0)).unwrap()));
        test_square.add_edge((*map.get(&(0, 1)).unwrap(), *map.get(&(1, 1)).unwrap()));
        assert!(graphs_eq(&square, &test_square));
    }
}