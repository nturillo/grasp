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


    fn dijkstra_unweighted<'a, N>(&'a self, source: VertexID) -> Result<(HashMap<VertexID, N>, HashMap<VertexID, VertexID>), GraphError>
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

    fn astar_unweighted<'a, N>(&'a self, source: VertexID, target: VertexID) -> Result<(Vec<VertexID>, N), GraphError>
    where Self: Sized,
    N: Number + One + PartialOrd + Default + Copy + 'a {
        let mut iter = self.astar_iter(
            source,
            target,
            |_, _| None::<N>,
            |_| N::default(),
        )?;

        let mut found = None;
        while let Some(step) = iter.next() {
            let (v, cost) = step?;
            if v == target {
                found = Some((v, cost));
                break;
            }
        }

        let (end, cost) = found.ok_or(GraphError::VertexNotInGraph(target))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::prelude::*;
    use crate::algorithms::search::ShortestPath;

    #[test]
    fn unweighted_dijkstra_simple_graph() {
        let mut g = SparseSimpleGraph::default();
        g.add_edge((0, 1));
        g.add_edge((1, 2));
        g.add_edge((2, 3));

        let (dist, prev) = g.dijkstra_unweighted::<i32>(0).unwrap();

        assert_eq!(dist.get(&0), Some(&0));
        assert_eq!(dist.get(&1), Some(&1));
        assert_eq!(dist.get(&2), Some(&2));
        assert_eq!(dist.get(&3), Some(&3));

        assert_eq!(prev.get(&1), Some(&0));
        assert_eq!(prev.get(&2), Some(&1));
        assert_eq!(prev.get(&3), Some(&2));
    }

    #[test]
    fn unweighted_astar_simple_graph() {
        let mut g = SparseSimpleGraph::default();
        g.add_edge((0, 1));
        g.add_edge((1, 2));
        g.add_edge((2, 3));

        let (path, cost) = g.astar_unweighted::<i32>(0, 3).unwrap();

        assert_eq!(cost, 3);
        assert_eq!(path, vec![0, 1, 2, 3]);
    }

    #[test]
    fn weighted_dijkstra_simple_graph() {
        let mut g = SparseSimpleGraph::default();
        g.add_edge((0, 1));
        g.add_edge((1, 2));
        g.add_edge((2, 3));
        g.add_edge((0, 3));
        g.add_edge((1, 3));

        let weight = |_g: &SparseSimpleGraph, (u, v): EdgeID| -> Option<i32> {
            match (u.min(v), u.max(v)) {
                (0, 1) => Some(1),
                (1, 2) => Some(2),
                (2, 3) => Some(1),
                (0, 3) => Some(10),
                (1, 3) => Some(6),
                _ => None,
            }
        };

        let mut iter = g.dijkstra_iter(0, weight).unwrap();
        while let Some(step) = iter.next() {
            step.unwrap();
        }

        assert_eq!(iter.distance_to(0), Some(0));
        assert_eq!(iter.distance_to(1), Some(1));
        assert_eq!(iter.distance_to(2), Some(3));
        assert_eq!(iter.distance_to(3), Some(4));
        assert_eq!(iter.shortest_path_to(3), Some(vec![0, 1, 2, 3]));
    }

    #[test]
    fn weighted_astar_simple_graph() {
        let mut g = SparseSimpleGraph::default();
        g.add_edge((0, 1));
        g.add_edge((1, 2));
        g.add_edge((2, 3));
        g.add_edge((0, 3));
        g.add_edge((1, 3));

        let weight = |_g: &SparseSimpleGraph, (u, v): EdgeID| -> Option<i32> {
            match (u.min(v), u.max(v)) {
                (0, 1) => Some(1),
                (1, 2) => Some(2),
                (2, 3) => Some(1),
                (0, 3) => Some(10),
                (1, 3) => Some(6),
                _ => None,
            }
        };

        let heuristic = |v: VertexID| -> i32 {
            match v {
                0 => 3,
                1 => 2,
                2 => 1,
                3 => 0,
                _ => 0,
            }
        };

        let mut iter = g.astar_iter(0, 3, weight, heuristic).unwrap();

        let mut last = None;
        while let Some(step) = iter.next() {
            last = Some(step.unwrap());
        }

        let (end, cost) = last.unwrap();
        let path = iter.shortest_path_to(end).unwrap();

        assert_eq!(cost, 4);
        assert_eq!(path, vec![0, 1, 2, 3]);
    }
}