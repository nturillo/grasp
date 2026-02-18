use crate::graph::graph_traits::*;
use crate::graph::errors::GraphError;

use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::cmp::{Reverse, Ordering};

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

pub struct OrderedNumber<N>(pub N);

impl<N: PartialEq> PartialEq for OrderedNumber<N> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<N: PartialEq> Eq for OrderedNumber<N> {}

impl<N: PartialOrd> PartialOrd for OrderedNumber<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}
impl<N: PartialOrd> Ord for OrderedNumber<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

pub struct Dijkstra<'a, G: GraphTrait, WF, N>
where WF: Fn(&G, EdgeType) -> Option<N> + 'a,
N: Number + PartialOrd + Default + Copy + 'a, {
    g: &'a G,
    weight: WF,
    dist: HashMap<VertexType, N>,
    prev: HashMap<VertexType, VertexType>,
    heap: BinaryHeap<Reverse<(OrderedNumber<N>, VertexType)>>,
    finished: HashSet<VertexType>,
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
        
        for u in self.g.neighbors(v).expect("graph should have vertex").iter() {
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
where WF: Fn(&G, EdgeType) -> Option<N> + 'a,
N: Number + PartialOrd + Default + Copy + 'a, {
    pub fn from_source(source: VertexType, g: &'a G, weight: WF) -> Result<Self, GraphError> {
        if !g.contains(source) {
            return Err(GraphError::VertexNotInGraph);
        }

        let mut dist = HashMap::new();
        let zero: N = N::default();
        dist.insert(source, zero);

        let mut heap = BinaryHeap::new();
        heap.push(Reverse((OrderedNumber(zero), source)));

        Ok(Dijkstra {
            g,
            weight,
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

    pub fn distance_to(&self, v: VertexType) -> Option<N> {
        self.dist.get(&v).cloned()
    }
}

impl<'a, G: GraphTrait, WF, N> Iterator for Dijkstra<'a, G, WF, N>
where WF: Fn(&G, EdgeType) -> Option<N> + 'a,
N: Number + PartialOrd + Default + Copy + 'a, {
    type Item = Result<(VertexType, N), GraphError>;

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
                Ok(n) => n,
                Err(e) => return Some(Err(e)),
            };

            for u in neighbor_list.iter() {
                let edge = (v, *u);
                match self.g.has_edge(edge) {
                    Ok(true) => {},
                    Ok(false) => return Some(Err(GraphError::EdgeNotInGraph)),
                    Err(e) => return Some(Err(e)),
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
                    self.heap.push(Reverse((OrderedNumber(alt), *u)));
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