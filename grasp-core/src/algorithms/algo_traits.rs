use crate::graph::graph_traits::*;
use crate::graph::errors::*;

use crate::algorithms::{search::*};

pub trait AlgoTrait: GraphTrait {
    fn bfs_iter<'a>(&'a self, v: VertexType) -> Result<BfsIter<'a, Self>, GraphError> where Self: Sized;
    fn dfs_iter<'a>(&'a self, v: VertexType) -> Result<DfsIter<'a, Self>, GraphError> where Self: Sized;
}

impl<G:GraphTrait> AlgoTrait for G {
    fn bfs_iter<'a>(&'a self, v: VertexType) -> Result<BfsIter<'a, Self>, GraphError> where
        Self: Sized
    {
        TraversalIter::from_source(v, &self)

    }
    fn dfs_iter<'a>(&'a self, v: VertexType) -> Result<DfsIter<'a, Self>, GraphError> where
        Self: Sized
    {
        TraversalIter::from_source(v, &self)

    }
}