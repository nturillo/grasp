use bimap::BiHashMap;
use crate::graph::{EdgeID, GraphTrait, VertexID, prelude::{DiGraph, SimpleGraph}, set::Set};

/// Trait for graphs with Homogenous vertex ID's
pub trait HomogenousVertexId: GraphTrait{}

#[derive(Debug)]
pub struct HomogenousView<'a, G: GraphTrait>{
    /// Map from original ID to homogenous ID
    vertex_map: BiHashMap<VertexID, VertexID>,
    /// VertexID not used by graph, useful for when we want to map a invalid id to an id invalid in graph
    unused_id: VertexID,
    graph: &'a G
}
impl<'a, G: GraphTrait> HomogenousView<'a, G>{
    pub fn from_graph(graph: &'a G) -> Self{
        let mut vertex_map = BiHashMap::with_capacity(graph.vertex_count());
        let mut unused_id = 0;
        for (i, vertex) in graph.vertices().enumerate(){
            if vertex >= unused_id {unused_id = vertex+1;}
            vertex_map.insert(vertex, i);
        }
        Self{vertex_map, graph, unused_id}
    }
}
impl<'a, G: GraphTrait> GraphTrait for HomogenousView<'a, G>{
    fn is_empty(&self) -> bool {self.graph.is_empty()}
    
    fn vertex_count(&self) -> usize {self.graph.vertex_count()}
    
    fn edge_count(&self) -> usize {self.graph.edge_count()}
    
    fn has_vertex(&self, v: VertexID) -> bool {
        let Some(v) = self.vertex_map.get_by_right(&v) else {return false;};
        self.graph.has_vertex(*v)
    }
    
    fn has_edge(&self, e: EdgeID) -> bool {
        let Some(v1) = self.vertex_map.get_by_right(&e.0) else {return false;};
        let Some(v2) = self.vertex_map.get_by_right(&e.1) else {return false;};
        self.graph.has_edge((*v1, *v2))
    }
    
    fn vertices(&self) -> impl Iterator<Item=VertexID> {(0..self.graph.vertex_count()).into_iter()}
    
    fn edges(&self) -> impl Iterator<Item=EdgeID> {
        self.graph.edges().map(|(v1, v2)| (
            *self.vertex_map.get_by_left(&v1).unwrap(),
            *self.vertex_map.get_by_left(&v2).unwrap()
        ))
    }
    
    fn neighbors(&self, v: VertexID) -> impl Set<Item = VertexID> {
        // inner graphs ID of v, or a unused id
        let v = self.vertex_map.get_by_right(&v).copied().unwrap_or(self.unused_id);
        self.graph.neighbors(v).with_bimap(&self.vertex_map)
    }
    
    fn vertex_set(&self) -> impl Set<Item = VertexID> {
        self.graph.vertex_set().with_bimap(&self.vertex_map)
    }
}
impl<'a, G: GraphTrait> HomogenousVertexId for HomogenousView<'a, G>{}
impl<'a, G: GraphTrait+SimpleGraph> SimpleGraph for HomogenousView<'a, G>{}
impl<'a, G: GraphTrait+DiGraph> DiGraph for HomogenousView<'a, G>{
    fn in_neighbors(&self, v: VertexID) -> impl Set<Item = VertexID> {
        let v = self.vertex_map.get_by_right(&v).copied().unwrap_or(self.unused_id);
        self.graph.in_neighbors(v).with_bimap(&self.vertex_map)
    }
    fn out_neighbors(&self, v: VertexID) -> impl Set<Item = VertexID> {
        let v = self.vertex_map.get_by_right(&v).copied().unwrap_or(self.unused_id);
        self.graph.out_neighbors(v).with_bimap(&self.vertex_map)
    }
    fn all_neighbors(&self, v: VertexID) -> impl Set<Item = VertexID> {
        let v = self.vertex_map.get_by_right(&v).copied().unwrap_or(self.unused_id);
        self.graph.all_neighbors(v).with_bimap(&self.vertex_map)
    }
}