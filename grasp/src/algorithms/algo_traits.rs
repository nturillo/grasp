use std::ops::Deref;
use std::collections::HashMap;

use crate::graph::prelude::*;
use crate::algorithms::{search::*};

/// Trait used to represent types that can be used as a number
pub trait Number: Clone+Copy+std::ops::Add<Output=Self>+std::ops::Sub<Output=Self>+std::ops::Mul<Output=Self>+std::ops::Div<Output=Self>+PartialOrd{}
impl<T> Number for T where T: 
    Clone+Copy+PartialOrd+
    std::ops::Add<Output=Self>+std::ops::Sub<Output=Self>+
    std::ops::Mul<Output=Self>+std::ops::Div<Output=Self>
{}

pub trait One {
    fn one() -> Self;
}

impl One for f64 { fn one() -> Self { 1.0 } }
impl One for f32 { fn one() -> Self { 1.0 } }
impl One for i32 { fn one() -> Self { 1 } }
impl One for usize { fn one() -> Self { 1 } }

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


pub trait AlgoTrait: GraphTrait {
    fn bfs_iter<'a>(&'a self, v: VertexID) -> Result<BfsIter<'a, Self>, GraphError> where Self: Sized;
    fn dfs_iter<'a>(&'a self, v: VertexID) -> Result<DfsIter<'a, Self>, GraphError> where Self: Sized;

    fn dijkstra_iter<'a, WF, N>(&'a self, source: VertexID, weight: WF) -> Result<Dijkstra<'a, Self, WF, N>, GraphError>
    where Self: Sized + GraphTrait,
    WF: Fn(&Self, EdgeID) -> Option<N> + 'a,
    N: Number + One + PartialOrd + Default + Copy + 'a;

    fn astar_iter<'a, WF, HF, N>(&'a self, source: VertexID, target: VertexID, weight: WF, heuristic: HF) -> Result<AStar<'a, Self, WF, HF, N>, GraphError>
    where Self: Sized + GraphTrait,
    WF: Fn(&Self, EdgeID) -> Option<N> + 'a,
    HF: Fn(VertexID) -> N + 'a,
    N: Number + One + PartialOrd + Default + Copy + 'a;


    fn dijkstra<'a, N>(&'a self, source: VertexID) -> Result<(HashMap<VertexID, N>, HashMap<VertexID, VertexID>), GraphError>
    where Self: Sized,
    N: Number + One + PartialOrd + Default + Copy + 'a {
        let mut iter = self.dijkstra_iter(source, |_, _| None::<N>)?;

        let mut dist = HashMap::new();
        while let Some(step) = iter.next() {
            let (v, d) = step?;
            dist.insert(v, d);
        }

        Ok((dist, iter.predecessors().clone()))
    }

    fn astar<'a, N>(&'a self, source: VertexID, target: VertexID) -> Result<(Vec<VertexID>, N), GraphError>
    where Self: Sized,
    N: Number + One + PartialOrd + Default + Copy + 'a {
        let mut iter = self.astar_iter(
            source,
            target,
            |_, _| None::<N>,
            |_| N::default(),
        )?;

        let mut last = None;
        while let Some(step) = iter.next() {
            last = Some(step?);
        }

        let (end, cost) = last.ok_or(GraphError::VertexNotInGraph(target))?;
        let path = iter.shortest_path_to(end).unwrap_or_else(|| vec![end]);

        Ok((path, cost))
    }
}

impl<G:GraphTrait> AlgoTrait for G {
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

    fn dijkstra_iter<'a, WF, N>(&'a self, source: VertexID, weight: WF) -> Result<Dijkstra<'a, Self, WF, N>, GraphError>
    where Self: Sized + GraphTrait,
    WF: Fn(&Self, EdgeID) -> Option<N> + 'a,
    N: Number + One + PartialOrd + Default + Copy + 'a {
        Dijkstra::from_source(source, self, weight)
    }

    fn astar_iter<'a, WF, HF, N>(&'a self, source: VertexID, target: VertexID, weight: WF, heuristic: HF) -> Result<AStar<'a, Self, WF, HF, N>, GraphError>
    where Self: Sized + GraphTrait,
    WF: Fn(&Self, EdgeID) -> Option<N> + 'a,
    HF: Fn(VertexID) -> N + 'a,
    N: Number + One + PartialOrd + Default + Copy + 'a {
        AStar::from_source(source, target, self, weight, heuristic)
    }
}