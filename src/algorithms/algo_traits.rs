use std::ops::Deref;

use crate::graph::prelude::*;
use crate::algorithms::{search::*};

/// Trait used to represent types that can be used as a number
pub trait Number: Clone+Copy+std::ops::Add<Output=Self>+std::ops::Sub<Output=Self>+std::ops::Mul<Output=Self>+std::ops::Div<Output=Self>+PartialOrd{}
impl<T> Number for T where T: 
    Clone+Copy+PartialOrd+
    std::ops::Add<Output=Self>+std::ops::Sub<Output=Self>+
    std::ops::Mul<Output=Self>+std::ops::Div<Output=Self>
{}

#[derive(Debug, Default, PartialEq, PartialOrd)]
pub struct OrdNumber<N: Number>(pub N);
impl<N: Number> Eq for OrdNumber<N>{}
impl<N: Number> Ord for OrdNumber<N>{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        PartialOrd::partial_cmp(&self.0, &other.0).unwrap_or(std::cmp::Ordering::Equal)
    }
}
impl<N: Number> Deref for OrdNumber<N>{
    type Target = N;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<N: Number> From<N> for OrdNumber<N>{
    fn from(value: N) -> Self {
        Self(value)
    }
}
impl<N: Number> AsRef<N> for OrdNumber<N>{
    fn as_ref(&self) -> &N {
        &self.0
    }
}


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