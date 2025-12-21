
pub trait SetTrait: Clone {
    fn contains(&self, v: usize) -> bool;
    fn num_vertices(&self) -> usize;
    fn union(&self, other: &Self) -> Self;
    fn intersection(&self, other: &Self) -> Self;
    fn iter(&self) -> impl Iterator<Item=&usize>;
}

pub trait GraphTrait {
    type NeighborSet: SetTrait;
    
    fn new() -> Self;

    fn num_vertices(&self) -> usize;
    fn num_edges(&self) -> usize;
    fn has_edge(&self, v1: usize, v2: usize) -> Option<bool>;
        // None if v1 or v2 don't exist in the graph
    fn neighbors(&self, v: usize) -> Option<&Self::NeighborSet>;
        // None if v doesn't exist in the graph
    
    fn add_vertex(&mut self, v: usize);
    fn add_edge(&mut self, v1: usize, v2:usize);
        //adds v1 and v2 if they don't exist
    fn add_neighbors(&mut self, v: usize, nbhrs: impl Iterator<Item=usize>);
        //adds v and nbhrs if they don't exist
}