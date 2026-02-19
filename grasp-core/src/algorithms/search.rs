use crate::graph::prelude::*;
use crate::algorithms::algo_traits::Number;

use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::cmp::Reverse;

pub type BfsIter<'a, G> = TraversalIter<'a, G, VecDeque<VertexID>>;
pub type DfsIter<'a, G> = TraversalIter<'a, G, Vec<VertexID>>;

pub trait Frontier {
    fn new() -> Self;
    fn push(&mut self, v: VertexID);
    fn pop(&mut self) -> Option<VertexID>;
}

impl Frontier for VecDeque<VertexID> {
    fn new() -> Self {
        Self::new()
    }
    fn push(&mut self, v: VertexID) {
        self.push_back(v);
    }
    fn pop(&mut self) -> Option<VertexID> {
        self.pop_front()
    }
}
impl Frontier for Vec<VertexID> {
    fn new() -> Self {
        Self::new()
    }
    fn push(&mut self, v: VertexID) {
        self.push(v);
    }
    fn pop(&mut self) -> Option<VertexID> {
        self.pop()
    }
}
pub struct TraversalIter<'a, G: SimpleGraph, F: Frontier> {
    g: &'a G,
    frontier: F,
    visited: HashSet<VertexID>
}


// Use OrdNumber from algo_traits instead
use crate::algorithms::algo_traits::OrdNumber;

pub struct Dijkstra<'a, G: GraphTrait, WF, N>
where WF: Fn(&G, EdgeID) -> Option<N> + 'a,
N: Number + PartialOrd + Default + Copy + 'a, {
    g: &'a G,
    weight: WF,
    dist: HashMap<VertexID, N>,
    prev: HashMap<VertexID, VertexID>,
    heap: BinaryHeap<Reverse<(OrdNumber<N>, VertexID)>>,
    finished: HashSet<VertexID>,
}

impl<'a, G:SimpleGraph, F:Frontier> TraversalIter<'a, G, F> {
    pub fn from_source(source: VertexID, g: &'a G) -> Result<Self, GraphError> {
        if !g.contains(source) {
            return Err(GraphError::VertexNotInGraph(source));
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

impl<'a, G:SimpleGraph, F:Frontier> Iterator for TraversalIter<'a, G, F> {
    type Item = VertexID;
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

impl<'a, G: GraphTrait, WF, N> Dijkstra<'a, G, WF, N>
where WF: Fn(&G, EdgeID) -> Option<N> + 'a,
N: Number + PartialOrd + Default + Copy + 'a, {
    pub fn from_source(source: VertexID, g: &'a G, weight: WF) -> Result<Self, GraphError> {
        if !g.contains(source) {
            return Err(GraphError::VertexNotInGraph(source));
        }

        let mut dist = HashMap::new();
        let zero: N = N::default();
        dist.insert(source, zero);

        let mut heap = BinaryHeap::new();
        heap.push(Reverse((OrdNumber(zero), source)));

        Ok(Dijkstra {
            g,
            weight,
            dist,
            prev: HashMap::new(),
            heap,
            finished: HashSet::new(),
        })
    }

    pub fn shortest_path_to(&self, target: VertexID) -> Option<Vec<VertexID>> {
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

    pub fn distance_to(&self, v: VertexID) -> Option<N> {
        self.dist.get(&v).cloned()
    }
}

impl<'a, G: GraphTrait, WF, N> Iterator for Dijkstra<'a, G, WF, N>
where WF: Fn(&G, EdgeID) -> Option<N> + 'a,
N: Number + PartialOrd + Default + Copy + 'a, {
    type Item = Result<(VertexID, N), GraphError>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(Reverse((ord_d, v))) = self.heap.pop() {
            let d_val: N = ord_d.0;

            if let Some(&best) = self.dist.get(&v) {
                if d_val > best {
                    continue;
                }
            }

            if self.finished.contains(&v) {
                continue;
            }
            self.finished.insert(v);

            let neighbor_list = match self.g.neighbors(v) {
                Some(n) => n,
                None => continue,
            };

            for u in neighbor_list.iter() {
                let edge = (v, *u);
                if !self.g.has_edge(edge) {
                    return Some(Err(GraphError::EdgeNotInGraph(edge)));
                }

                let w: N = match (self.weight)(self.g, edge) {
                    Some(val) => val,
                    None => continue,
                };
                let alt: N = d_val + w;

                let is_better = match self.dist.get(u) {
                    Some(&old) => alt < old,
                    None => true,
                };

                if is_better {
                    self.dist.insert(*u, alt);
                    self.prev.insert(*u, v);
                    self.heap.push(Reverse((OrdNumber(alt), *u)));
                }
            }
            return Some(Ok((v, d_val)));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn dijkstra() {}
}