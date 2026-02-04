use crate::graph::errors::*;

pub type VertexType = usize;
pub type EdgeType = (VertexType, VertexType);
pub trait SetTrait: Clone {
    fn contains(&self, v: VertexType) -> bool;
    fn num_vertices(&self) -> usize;
    fn union(&self, other: &Self) -> Self;
    fn intersection(&self, other: &Self) -> Self;
    fn iter(&self) -> impl Iterator<Item=&VertexType>;
}

pub trait GraphTrait {
    type NeighborSet: SetTrait;
    
    fn new() -> Self;

    fn num_vertices(&self) -> usize;
    fn num_edges(&self) -> usize;
    fn vertices(&self) -> impl Iterator<Item=VertexType>;
    fn edges(&self) -> impl Iterator<Item=(VertexType,VertexType)>;
    fn contains(&self, v: VertexType) -> bool;
    fn has_edge(&self, e: EdgeType) -> Result<bool, GraphError>;
    fn neighbors(&self, v: VertexType) -> Result<&Self::NeighborSet, GraphError>;
    
    fn add_vertex(&mut self, v: VertexType);
    fn add_edge(&mut self, e: EdgeType);
        //adds v1 and v2 if they don't exist
    fn add_neighbors(&mut self, v: VertexType, nbhrs: impl Iterator<Item=VertexType>);
        //adds v and nbhrs if they don't exist
}