use crate::graph::prelude::*;

use crate::algorithms::{search::*};

pub trait AlgoTrait: SimpleGraph {
    fn bfs_iter<'a>(&'a self, v: VertexID) -> Result<BfsIter<'a, Self>, GraphError> where Self: Sized;
    fn dfs_iter<'a>(&'a self, v: VertexID) -> Result<DfsIter<'a, Self>, GraphError> where Self: Sized;
}

impl<G:SimpleGraph> AlgoTrait for G {
    fn bfs_iter<'a>(&'a self, v: VertexID) -> Result<BfsIter<'a, Self>, GraphError> where
        Self: Sized
    {
        TraversalIter::from_source(v, &self)

    }
    fn dfs_iter<'a>(&'a self, v: VertexID) -> Result<DfsIter<'a, Self>, GraphError> where
        Self: Sized
    {
        TraversalIter::from_source(v, &self)

    }
}