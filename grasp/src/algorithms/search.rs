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

pub trait ShortestPath<N>
where N: Number + PartialOrd + Default + Copy + Clone, {
    fn dist(&self) -> &HashMap<VertexID, N>;
    fn prev(&self) -> &HashMap<VertexID, VertexID>;

    fn distance_to(&self, v: VertexID) -> Option<N> {
        self.dist().get(&v).cloned()
    }

    fn predecessors(&self) -> &HashMap<VertexID, VertexID> {
        self.prev()
    }

    fn shortest_path_to(&self, target: VertexID) -> Option<Vec<VertexID>> {
        if !self.dist().contains_key(&target) {
            return None;
        }

        let mut path = Vec::new();
        let mut cur = target;
        path.push(cur);

        while let Some(&p) = self.prev().get(&cur) {
            cur = p;
            path.push(cur);
        }

        path.reverse();
        Some(path)
    }
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

pub struct AStar<'a, G: GraphTrait, WF, HF, N>
where WF: Fn(&G, EdgeID) -> Option<N> + 'a,
HF: Fn(VertexID) -> N + 'a,
N: Number + PartialOrd + Default + Copy + 'a, {
    g: &'a G,
    weight: WF,
    heuristic: HF,
    dist: HashMap<VertexID, N>, //g(v)
    prev: HashMap<VertexID, VertexID>,
    heap: BinaryHeap<Reverse<(OrdNumber<N>, VertexID)>>, //f(v)
    finished: HashSet<VertexID>,
    target: VertexID,
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

impl<'a, G: GraphTrait, WF, HF, N> AStar<'a, G, WF, HF, N>
where WF: Fn(&G, EdgeID) -> Option<N> + 'a,
HF: Fn(VertexID) -> N + 'a,
N: Number + PartialOrd + Default + Copy + 'a, {
    pub fn from_source(source: VertexID, target: VertexID, g: &'a G, weight: WF, heuristic: HF) -> Result<Self, GraphError> {
        if !g.contains(source) {
            return Err(GraphError::VertexNotInGraph(source));
        }

        let mut dist = HashMap::new();
        let zero = N::default();
        dist.insert(source, zero);

        let mut heap = BinaryHeap::new();
        let f0 = zero + heuristic(source);
        heap.push(Reverse((OrdNumber(f0), source)));

        Ok(AStar{g, weight, heuristic, dist, prev: HashMap::new(), heap, finished: HashSet::new(), target})
    }
}

impl<'a, G: GraphTrait, WF, HF, N> Iterator for AStar<'a, G, WF, HF, N>
where WF: Fn(&G, EdgeID) -> Option<N> + 'a,
HF: Fn(VertexID) -> N + 'a,
N: Number + PartialOrd + Default + Copy + 'a, {
    type Item = Result<(VertexID, N), GraphError>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(Reverse((ord_f, v))) = self.heap.pop() {
            if self.finished.contains(&v) {
                continue;
            }
            self.finished.insert(v);

            let g_v = *self.dist.get(&v).unwrap_or(&N::default());

            if v == self.target {
                return Some(Ok((v, g_v)));
            }

            let neighbors = match self.g.neighbors(v) {
                Some(n) => n,
                None => continue,
            };

            for u in neighbors.iter() {
                let edge = (v, *u);

                if !self.g.has_edge(edge) {
                    return Some(Err(GraphError::EdgeNotInGraph(edge)));
                }

                let w = match (self.weight)(self.g, edge) {
                    Some(val) => val,
                    None => continue,
                };

                let tentative_g = g_v + w;

                let is_better = match self.dist.get(u) {
                    Some(&old) => tentative_g < old,
                    None => true,
                };

                if is_better {
                    self.dist.insert(*u, tentative_g);
                    self.prev.insert(*u, v);

                    let f_u = tentative_g + (self.heuristic)(*u);
                    self.heap.push(Reverse((OrdNumber(f_u), *u)));
                }
            }

            return Some(Ok((v, g_v)));
        }
        None
    }
}

impl<'a, G, WF, N> ShortestPath<N> for Dijkstra<'a, G, WF, N>
where G: GraphTrait,
WF: Fn(&G, EdgeID) -> Option<N>,
N: Number + PartialOrd + Default + Copy, {
    fn dist(&self) -> &HashMap<VertexID, N> {
        &self.dist
    }

    fn prev(&self) -> &HashMap<VertexID, VertexID> {
        &self.prev
    }
}

impl<'a, G, WF, HF, N> ShortestPath<N> for AStar<'a, G, WF, HF, N>
where G: GraphTrait,
WF: Fn(&G, EdgeID) -> Option<N>,
HF: Fn(VertexID) -> N,
N: Number + PartialOrd + Default + Copy, {
    fn dist(&self) -> &HashMap<VertexID, N> {
        &self.dist
    }

    fn prev(&self) -> &HashMap<VertexID, VertexID> {
        &self.prev
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dijkstra() {
        // Create a simple weighted graph
        let mut graph = SparseSimpleGraph::default();
        graph.add_edge((0, 1));
        graph.add_edge((1, 2));
        graph.add_edge((0, 2));
        graph.add_edge((2, 3));
        
        // Weight function: (0,1)=1, (1,2)=2, (0,2)=5, (2,3)=1
        let weight = |_g: &SparseSimpleGraph, (v1, v2): EdgeID| -> Option<i32> {
            match ((v1.min(v2), v1.max(v2))) {
                (0, 1) => Some(1),
                (1, 2) => Some(2),
                (0, 2) => Some(5),
                (2, 3) => Some(1),
                _ => None,
            }
        };
        
        // Run Dijkstra from vertex 0
        let mut dijkstra = Dijkstra::from_source(0, &graph, weight).unwrap();
        let mut results = Vec::new();
        while let Some(result) = dijkstra.next() {
            if let Ok((v, dist)) = result {
                results.push((v, dist));
            }
        }
        
        // Verify distances
        assert_eq!(dijkstra.distance_to(0), Some(0));
        assert_eq!(dijkstra.distance_to(1), Some(1));
        assert_eq!(dijkstra.distance_to(2), Some(3));
        assert_eq!(dijkstra.distance_to(3), Some(4));
        
        // Verify shortest path
        assert_eq!(dijkstra.shortest_path_to(3), Some(vec![0, 1, 2, 3]));
        assert_eq!(dijkstra.shortest_path_to(0), Some(vec![0]));
    }

    #[test]
    fn astar() {
        let mut graph = SparseSimpleGraph::default();
        graph.add_edge((0, 1));
        graph.add_edge((1, 2));
        graph.add_edge((0, 2));
        graph.add_edge((2, 3));

        let weight = |_g: &SparseSimpleGraph, (v1, v2): EdgeID| -> Option<i32> {
            match (v1.min(v2), v1.max(v2)) {
                (0, 1) => Some(1),
                (1, 2) => Some(2),
                (0, 2) => Some(5),
                (2, 3) => Some(1),
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

        let mut astar = AStar::from_source(0, 3, &graph, weight, heuristic).unwrap();

        let mut results = Vec::new();
        while let Some(result) = astar.next() {
            if let Ok((v, dist)) = result {
                results.push((v, dist));
            }
        }

        assert_eq!(astar.distance_to(0), Some(0));
        assert_eq!(astar.distance_to(1), Some(1));
        assert_eq!(astar.distance_to(2), Some(3));
        assert_eq!(astar.distance_to(3), Some(4));

        assert_eq!(astar.shortest_path_to(3), Some(vec![0, 1, 2, 3]));
        assert_eq!(astar.shortest_path_to(0), Some(vec![0]));
    }
}