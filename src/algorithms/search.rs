use crate::graph::graph_traits::*;
use crate::graph::errors::GraphError;

use std::collections::{HashSet, VecDeque};

pub type BfsIter<'a, G:GraphTrait> = TraversalIter<'a, G, VecDeque<VertexType>>;
pub type DfsIter<'a, G:GraphTrait> = TraversalIter<'a, G, Vec<VertexType>>;

trait Frontier {
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

