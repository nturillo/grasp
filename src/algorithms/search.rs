use crate::graph::prelude::*;
use std::collections::{HashSet, VecDeque};

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
pub struct TraversalIter<'a, G: Graph, F: Frontier> {
    g: &'a G,
    frontier: F,
    visited: HashSet<VertexID>
}

impl<'a, G:Graph, F:Frontier> TraversalIter<'a, G, F> {
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

impl<'a, G:Graph, F:Frontier> Iterator for TraversalIter<'a, G, F> {
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

