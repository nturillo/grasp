use std::{collections::{HashMap, HashSet}, vec};

use crate::graph::{EdgeID, GraphMut, GraphTrait, VertexID, prelude::SimpleGraph, set::Set};


/// Special graph type which represents a matching.
/// Each vertex can have degree at most one.
pub struct Matching {
    vertices: HashSet<VertexID>,
    edges: HashMap<VertexID, VertexID>,
}

impl Matching {
    pub fn new() -> Self {
        Matching {
            vertices: HashSet::new(),
            edges: HashMap::new(),
        }
    }
}

impl GraphTrait for Matching {
    fn vertices(&self) -> impl Iterator<Item = VertexID> {
        self.vertices.iter().cloned()
    }
    fn edges(&self) -> impl Iterator<Item = EdgeID> {
        self.edges.iter().map(|(v1, v2)| (*v1, *v2))
    }
    fn vertex_count(&self) -> usize {
        self.vertices.len()
    }
    fn edge_count(&self) -> usize {
        self.edges.len() / 2
    }
    fn has_vertex(&self, v: VertexID) -> bool {
        self.vertices.contains(&v)
    }
    fn has_edge(&self, e: crate::graph::EdgeID) -> bool {
        self.edges.contains_key(&e.0) && self.edges.get(&e.0) == Some(&e.1)
    }
    fn neighbors(&self, v: VertexID) -> impl crate::graph::prelude::Set<Item = VertexID> {
        if let Some(&neighbor) = self.edges.get(&v) {
            std::iter::once(neighbor).collect::<HashSet<_>>()
        } else {
            std::iter::empty().collect::<HashSet<_>>()
        }
    }
    fn vertex_set(&self) -> impl crate::graph::prelude::Set<Item = VertexID> {
        self.vertices.iter().cloned().collect::<HashSet<_>>()
    }
}
impl GraphMut for Matching {
    fn create_vertex(&mut self) -> VertexID {
        let key= self.vertices.iter().max().map(|max| max+1).unwrap_or(0);
        self.vertices.insert(key);
        key
    }
    fn remove_vertex(&mut self, v: VertexID) -> impl Iterator<Item = EdgeID> {
        if !self.edges.contains_key(&v) {
            return vec![].into_iter();
        }
        let neighbor = self.edges.remove(&v).unwrap();
        self.edges.remove(&neighbor);
        self.vertices.remove(&v);
        vec![(v, neighbor)].into_iter()
    }
    fn try_add_edge(&mut self, edge: EdgeID) -> Result<(), crate::graph::prelude::GraphError> {
        match (self.has_vertex(edge.0), self.has_vertex(edge.1)) {
            (false, false) => Err(crate::graph::prelude::GraphError::NeitherVertexInGraph(edge.0, edge.1)),
            (false, true) => Err(crate::graph::prelude::GraphError::VertexNotInGraph(edge.0)),
            (true, false) => Err(crate::graph::prelude::GraphError::VertexNotInGraph(edge.1)),
            (true, true) => {
                if self.edges.contains_key(&edge.0) && self.edges.get(&edge.0) != Some(&edge.1) {
                    return Err(crate::graph::prelude::GraphError::EdgeNotAddable(edge, "Vertex 0 is already matched to a different vertex".to_string()));
                }
                if self.edges.contains_key(&edge.1) && self.edges.get(&edge.1) != Some(&edge.0) {
                    return Err(crate::graph::prelude::GraphError::EdgeNotAddable(edge, "Vertex 1 is already matched to a different vertex".to_string()));
                }
                self.edges.insert(edge.0, edge.1);
                self.edges.insert(edge.1, edge.0);
                Ok(())
            }
        }
    }
    fn remove_edge(&mut self, e: EdgeID) -> bool {
        match (self.has_vertex(e.0), self.has_vertex(e.1)) {
            (false, _) => false,
            (_, false) => false,
            (true, true) => {
                if self.edges.get(&e.0) == Some(&e.1) {
                    self.edges.remove(&e.0);
                    self.edges.remove(&e.1);
                    true
                } else {
                    false
                }
            }
        }
    }
}

/// Niche graph type for the blossom algorithm, which supports contracting and expanding blossoms.
struct BlossomGraph<'a, G: SimpleGraph> {
    graph: &'a G,
    new_vertex: VertexID,
    blossom: HashSet<VertexID>,
    blossom_neighbors: HashSet<VertexID>,
}

impl<G: SimpleGraph> BlossomGraph<'_, G> {
    pub fn new(graph: &G, blossom: HashSet<VertexID>) -> Self {
        let new_vertex = graph.vertices().max().map(|max| max + 1).unwrap_or(0);
        let blossom_neighbors = blossom.iter().flat_map(|&v| graph.neighbors(v).iter().collect::<Vec<_>>()).filter(|n| !blossom.contains(n)).collect();
        BlossomGraph {
            graph,
            new_vertex,
            blossom,
            blossom_neighbors,
        }
    }
    pub fn get_new_vertex(&self) -> VertexID {
        self.new_vertex
    }
}

impl<G: SimpleGraph> GraphTrait for BlossomGraph<'_, G> {
    fn vertex_count(&self) -> usize {
        self.graph.vertex_count() - self.blossom.len() + 1
    }
    fn edge_count(&self) -> usize {
        self.edges().collect::<Vec<_>>().len()
    }
    fn has_vertex(&self, v: VertexID) -> bool {
        if v == self.new_vertex {
            true
        } else if self.blossom.contains(&v) {
            false
        } else {
            self.graph.has_vertex(v)
        }
    }
    fn has_edge(&self, e: EdgeID) -> bool {
        if self.blossom.contains(&e.0) || self.blossom.contains(&e.1) {
            return false;
        }
        if e.0 == self.new_vertex {
            self.blossom.iter().any(|&v| self.graph.has_edge((v, e.1)))
        } else if e.1 == self.new_vertex {
            self.blossom.iter().any(|&v| self.graph.has_edge((e.0, v)))
        } else {
            self.graph.has_edge(e)
        }
    }
    fn vertices(&self) -> impl Iterator<Item=VertexID> {
        self.graph.vertices().filter(|v| !self.blossom.contains(v)).chain(std::iter::once(self.new_vertex))
    }
    fn edges(&self) -> impl Iterator<Item=EdgeID> {
        self.graph.edges().filter_map(|(u,v)| {
            if self.blossom.contains(&u) || self.blossom.contains(&v) {
                None
            } else {
                Some((u,v))
            }
        }).chain(self.blossom_neighbors.iter().map(move |&v| (self.new_vertex, v)))
    }
    fn neighbors(&self, v: VertexID) -> impl Set<Item = VertexID> {
        if self.blossom.contains(&v) {
            HashSet::new()
        } else if v == self.new_vertex {
            self.blossom_neighbors.clone()
        } else {
            let mut neighbors = self.graph.neighbors(v).iter().filter(|&&n| !self.blossom.contains(&n)).cloned().collect::<HashSet<_>>();
            if self.blossom.iter().any(|&b| self.graph.has_edge((v, b))) {
                neighbors.insert(self.new_vertex);
            }
            neighbors
        }
    }
    fn vertex_set(&self) -> impl Set<Item = VertexID> {
        self.graph.vertices().filter(|v| !self.blossom.contains(v)).chain(std::iter::once(self.new_vertex)).collect::<HashSet<_>>()
    }
}

#[cfg(test)]
mod tests {
    use crate::graph::{AnyVertexGraph, BuildableGraph, prelude::SparseSimpleGraph};

    use super::*;

    #[test]
    fn blossom_test() {
        let mut graph = SparseSimpleGraph::with_capacity(6, 0);
        graph.add_edge((0,1));
        graph.add_edge((1,2));
        graph.add_edge((1,3));
        graph.add_edge((2,3));
        graph.add_edge((3,4));
        graph.add_edge((3,5));

        let blossom = HashSet::from([1,2,3]);
        let blossom_graph = BlossomGraph::new(&graph, blossom);

        let mut expected_graph = SparseSimpleGraph::with_capacity(4, 0);
        expected_graph.add_edge((0, blossom_graph.get_new_vertex()));
        expected_graph.add_edge((4, blossom_graph.get_new_vertex()));
        expected_graph.add_edge((5, blossom_graph.get_new_vertex()));

        
    }
}