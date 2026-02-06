use crate::graph::graph_traits::*;
use crate::graph::errors::GraphError;

use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::cmp::Reverse;

pub type BfsIter<'a, G> = TraversalIter<'a, G, VecDeque<VertexType>>;
pub type DfsIter<'a, G> = TraversalIter<'a, G, Vec<VertexType>>;

pub trait Frontier {
    fn new() -> Self;
    fn push(&mut self, v: VertexType);
    fn pop(&mut self) -> Option<VertexType>;
}

impl Frontier for VecDeque<VertexType> {
    fn new() -> Self {
        Self::new()
    }
    fn push(&mut self, v: VertexType) {
        self.push_back(v);
    }
    fn pop(&mut self) -> Option<VertexType> {
        self.pop_front()
    }
}
impl Frontier for Vec<VertexType> {
    fn new() -> Self {
        Self::new()
    }
    fn push(&mut self, v: VertexType) {
        self.push(v);
    }
    fn pop(&mut self) -> Option<VertexType> {
        self.pop()
    }
}
pub struct TraversalIter<'a, G: GraphTrait, F: Frontier> {
    g: &'a G,
    frontier: F,
    visited: HashSet<VertexType>
}

pub struct Dijkstra<'a, G: GraphTrait> {
    g: &'a G,
    dist: HashMap<VertexType, u64>,
    prev: HashMap<VertexType, VertexType>,
    heap: BinaryHeap<Reverse<(u64, VertexType)>>,
    finished: HashSet<VertexType>
}

impl<'a, G:GraphTrait, F:Frontier> TraversalIter<'a, G, F> {
    pub fn from_source(source: VertexType, g: &'a G) -> Result<Self, GraphError> {
        if !g.contains(source) {
            return Err(GraphError::VertexNotInGraph);
        }

        let mut frontier = F::new();
        frontier.push(source);
        let mut visited = HashSet::new();
        visited.insert(source);
        Ok(TraversalIter{
            g: g,
            frontier: frontier,
            visited: visited
        })
    }
}

impl<'a, G:GraphTrait, F:Frontier> Iterator for TraversalIter<'a, G, F> {
    type Item = VertexType;
    fn next(&mut self) -> Option<Self::Item> {
        let v = self.frontier.pop()?;
        
        for u in self.g.neighbors(v)
            .expect("graph should have vertex")
            .iter()
        {
            if self.visited.contains(u) {
                continue;
            }
            self.visited.insert(*u);
            self.frontier.push(*u);
        }

        Some(v)
    }
}

impl<'a, G: GraphTrait> Dijkstra<'a, G> {
    pub fn from_source(source: VertexType, g: &'a G) -> Result<Self, GraphError> {
        if !g.contains(source) {
            return Err(GraphError::VertexNotInGraph);
        }
        let mut dist = HashMap::new();
        dist.insert(source, 0u64);

        let mut heap = BinaryHeap::new();
        heap.push(Reverse((0u64, source)));

        Ok(Dijkstra {
            g,
            dist,
            prev: HashMap::new(),
            heap,
            finished: HashSet::new(),
        })
    }

    pub fn shortest_path_to(&self, target: VertexType) -> Option<Vec<VertexType>> {
        if !self.dist.contains_key(&target) {
            return None;
        }
        let mut path = Vec::new();
        let mut cur = target;
        path.push(cur);
        while let Some(&p) = self.prev.get(&cur) {
            cur = p;
            path.push(cur);
        }
        path.reverse();
        Some(path)
    }

    pub fn distance_to(&self, v: VertexType) -> Option<u64> {
        self.dist.get(&v).cloned()
    }
}

impl<'a, G: GraphTrait> Iterator for Dijkstra<'a, G> {
    type Item = (VertexType, u64);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(Reverse((d, v))) = self.heap.pop() {
            if let Some(&best) = self.dist.get(&v) {
                if d > best {
                    continue;
                }
            }

            if self.finished.contains(&v) {
                continue;
            }
            self.finished.insert(v);

            //This is where logic for relaxing edges would go,
            //need to wait for edge weights to be implemented

            return Some((v, d));
        }

        return None;
    }
}