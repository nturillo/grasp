use std::{collections::{HashMap, HashSet, VecDeque}, vec};

use crate::{algorithms::distance::{graph_distance, shortest_path}, graph::prelude::*};


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
    pub fn add_edge(&mut self, edge: EdgeID) -> Result<(), crate::graph::prelude::GraphError> {
        if self.has_vertex(edge.0) || self.has_vertex(edge.1) {
            Err(crate::graph::prelude::GraphError::EdgeNotAddable(edge, "One or both vertices are already matched".to_string()))
        } else {
        self.vertices.insert(edge.0);
        self.vertices.insert(edge.1);
        self.edges.insert(edge.0, edge.1);
        self.edges.insert(edge.1, edge.0);
        Ok(())
        }
    }
    pub fn neighbor(&self, v: VertexID) -> Option<VertexID> {
        self.edges.get(&v).cloned()
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

/// Niche graph type for the blossom algorithm, which supports contracting and expanding blossoms.
struct BlossomGraph<'a, G: SimpleGraph> {
    graph: &'a G,
    new_vertex: VertexID,
    blossom: SparseSimpleGraph, 
    blossom_neighbors: HashSet<VertexID>,
}

impl<'a, G: SimpleGraph> BlossomGraph<'a, G> {
    pub fn new(graph: &'a G, blossom: SparseSimpleGraph) -> Self {
        let new_vertex = graph.vertices().max().map(|max| max + 1).unwrap_or(0);
        let blossom_neighbors = blossom.vertices().flat_map(|v| graph.neighbors(v).iter().cloned().collect::<Vec<_>>()).filter(|n| !blossom.has_vertex(*n)).collect();
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
    pub fn lift_path(&mut self, contracted_path: Vec<VertexID>, matching: &Matching) -> Vec<VertexID> {
        let mut path = Vec::with_capacity(contracted_path.len() + self.blossom.vertex_count() - 1);
        let exposed_vertex = self.blossom.vertices().find(|&v| {
            !matching.has_vertex(v) || !self.blossom.has_vertex(matching.neighbor(v).unwrap())
        }).unwrap();
        let mut new_index = contracted_path.iter().position(|&v| v == self.new_vertex).unwrap();
        let mut contracted_path = contracted_path;
        if !(new_index == 0 || self.graph.has_edge((contracted_path[new_index-1], exposed_vertex))) {
            contracted_path = contracted_path.into_iter().rev().collect();
            new_index = contracted_path.len() - 1 - new_index;
        }
        path.extend_from_slice(&contracted_path[0..new_index]);
        let next_vertex = contracted_path[new_index+1];
        let u = *self.graph.neighbors(next_vertex).iter().find(|&&n| self.blossom.has_vertex(n)).unwrap();
        let sp= shortest_path(&self.blossom, exposed_vertex, u).unwrap();
        if sp.len() % 2 == 1 {
            path.extend(sp);
        } else {
            let _ = self.blossom.remove_edge((exposed_vertex, sp[1]));
            path.extend(shortest_path(&self.blossom, exposed_vertex, u).unwrap());
        }
        path.extend_from_slice(&contracted_path[new_index+1..]);
        path
   }
}

impl<G: SimpleGraph> GraphTrait for BlossomGraph<'_, G> {
    fn vertex_count(&self) -> usize {
        self.graph.vertex_count() - self.blossom.vertex_count() + 1
    }
    fn edge_count(&self) -> usize {
        self.edges().collect::<Vec<_>>().len()
    }
    fn has_vertex(&self, v: VertexID) -> bool {
        if v == self.new_vertex {
            true
        } else if self.blossom.has_vertex(v) {
            false
        } else {
            self.graph.has_vertex(v)
        }
    }
    fn has_edge(&self, e: EdgeID) -> bool {
        if self.blossom.has_vertex(e.0) || self.blossom.has_vertex(e.1) {
            return false;
        }
        if e.0 == self.new_vertex {
            self.blossom.vertices().any(|v| self.graph.has_edge((v, e.1)))
        } else if e.1 == self.new_vertex {
            self.blossom.vertices().any(|v| self.graph.has_edge((e.0, v)))
        } else {
            self.graph.has_edge(e)
        }
    }
    fn vertices(&self) -> impl Iterator<Item=VertexID> {
        self.graph.vertices().filter(|v| !self.blossom.has_vertex(*v)).chain(std::iter::once(self.new_vertex))
    }
    fn edges(&self) -> impl Iterator<Item=EdgeID> {
        self.graph.edges().filter_map(|(u,v)| {
            if self.blossom.has_vertex(u) || self.blossom.has_vertex(v) {
                None
            } else {
                Some((u,v))
            }
        }).chain(self.blossom_neighbors.iter().map(move |&v| (self.new_vertex, v)))
    }
    fn neighbors(&self, v: VertexID) -> impl Set<Item = VertexID> {
        if self.blossom.has_vertex(v) {
            HashSet::new()
        } else if v == self.new_vertex {
            self.blossom_neighbors.clone()
        } else {
            let mut neighbors = self.graph.neighbors(v).iter().filter(|&&n| !self.blossom.has_vertex(n)).cloned().collect::<HashSet<_>>();
            if self.blossom.vertices().any(|b| self.graph.has_edge((v, b))) {
                neighbors.insert(self.new_vertex);
            }
            neighbors
        }
    }
    fn vertex_set(&self) -> impl Set<Item = VertexID> {
        self.graph.vertices().filter(|v| !self.blossom.has_vertex(*v)).chain(std::iter::once(self.new_vertex)).collect::<HashSet<_>>()
    }
}

impl<'a, G: SimpleGraph> SimpleGraph for BlossomGraph<'a, G> {}

fn find_augmenting_path<G: SimpleGraph>(graph: &G, matching: &Matching) -> Option<Vec<VertexID>> {
    let mut forest = SparseSimpleGraph::empty();
    let mut roots: HashMap<VertexID, VertexID> = HashMap::new();
    let mut edges_visited: HashSet<EdgeID> = HashSet::new();
    for e in matching.edges() {
        edges_visited.insert(e);
    }
    for v in graph.vertices() {
        if !matching.has_vertex(v) {
            forest.add_vertex(v);
            roots.insert(v, v);
        }
    }
    let mut queue: VecDeque<VertexID> = forest.vertices().collect();
    while let Some(v) = queue.pop_front() {
        if graph_distance(&forest, v, roots[&v]).unwrap() % 2 == 1 {
            continue;
        }
        for &w in graph.neighbors(v).iter() {
            let edge = (v, w);
            if edges_visited.contains(&edge) || edges_visited.contains(&(w, v)) {
                continue;
            }
            edges_visited.insert(edge);
            if !forest.has_vertex(w) {
                let x = matching.neighbor(w).unwrap();
                forest.add_edge((v, w));
                forest.add_edge((w, x));
                roots.insert(w, roots[&v]);
                roots.insert(x, roots[&v]);
                queue.push_back(x);
            } else {
                if graph_distance(&forest, w, roots[&w]).unwrap() % 2 == 1 {
                    continue;
                }
                if roots[&v] != roots[&w] {
                    //we have found an augmenting path
                    let mut root_to_v = shortest_path(&forest, roots[&v], v).unwrap();
                    let w_to_root = shortest_path(&forest, w, roots[&w]).unwrap();
                    root_to_v.extend(w_to_root);
                    return Some(root_to_v);
                } else {
                    let blossom_vertices= shortest_path(&forest, v, w).unwrap();
                    let mut blossom = SparseSimpleGraph::with_capacity(blossom_vertices.len(), 0);
                    blossom.add_edge((*blossom_vertices.first().unwrap(), *blossom_vertices.last().unwrap()));
                    for edge in blossom_vertices.windows(2) {
                        blossom.add_edge((edge[0], edge[1]));
                    }
                    let mut contracted_graph = BlossomGraph::new(graph, blossom.clone());
                    let new_vertex = contracted_graph.get_new_vertex();
                    let mut contracted_matching = Matching::new();
                    for e in matching.edges() {
                        match (blossom.has_vertex(e.0), blossom.has_vertex(e.1)) {
                            (true, true) => continue,
                            (true, false) => contracted_matching.add_edge((new_vertex, e.1)).unwrap(),
                            (false, true) => contracted_matching.add_edge((e.0, new_vertex)).unwrap(),
                            (false, false) => contracted_matching.add_edge(e).unwrap(),
                        }
                    }
                    if let Some(contracted_path) = find_augmenting_path(&contracted_graph, &contracted_matching) {
                        return Some(contracted_graph.lift_path(contracted_path, &contracted_matching));
                    } else {
                        return None;
                    }
                }
            }
        } 
    }

    None
}

#[cfg(test)]
mod tests {
    use crate::{assert_graphs_eq, graph::{AnyVertexGraph, BuildableGraph, prelude::SparseSimpleGraph}};

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

        let blossom = graph.subgraph_vertex([1,2,3]);
        let blossom_graph = BlossomGraph::new(&graph, blossom);

        let mut expected_graph = SparseSimpleGraph::with_capacity(4, 0);
        expected_graph.add_edge((0, blossom_graph.get_new_vertex()));
        expected_graph.add_edge((4, blossom_graph.get_new_vertex()));
        expected_graph.add_edge((5, blossom_graph.get_new_vertex()));

        assert_graphs_eq!(blossom_graph, expected_graph);        
    }

    #[test]
    fn lift_test1() {
        let mut graph = SparseSimpleGraph::with_capacity(6, 0);
        graph.add_edge((0,1));
        graph.add_edge((1,2));
        graph.add_edge((1,3));
        graph.add_edge((2,3));
        graph.add_edge((3,4));
        graph.add_edge((3,5));

        let blossom = graph.subgraph_vertex([1,2,3]);
        let mut blossom_graph = BlossomGraph::new(&graph, blossom.clone());

        let mut matching = Matching::new();
        matching.add_edge((1,2)).unwrap();
        matching.add_edge((3,4)).unwrap();

        let contracted_path = vec![0, blossom_graph.get_new_vertex(), 5];
        let lifted_path = blossom_graph.lift_path(contracted_path, &matching);
        pretty_assertions::assert_eq!(lifted_path, vec![5, 3, 2, 1, 0]);

        graph.add_edge((6,0));
        let mut matching = Matching::new();
        matching.add_edge((0,1)).unwrap();
        matching.add_edge((2,3)).unwrap();

        let mut blossom_graph = BlossomGraph::new(&graph, blossom.clone());
        let contracted_path = vec![6, 0, blossom_graph.get_new_vertex(), 4];
        let lifted_path = blossom_graph.lift_path(contracted_path, &matching);
        pretty_assertions::assert_eq!(lifted_path, vec![6, 0, 1, 2, 3, 4]);
    }

    #[test]
    fn fing_augmenting_test() {
        let mut graph = SparseSimpleGraph::with_capacity(7, 8);
        for e in [(0,1), (1,2), (2,3), (3,4), (4,5), (5,6), (6,7), (6,2)] {
            graph.add_edge(e);
        }
        let mut matching = Matching::new();
        for e in [(1,2), (3,4), (5,6)] {
            matching.add_edge(e).unwrap();
        }
        let path = find_augmenting_path(&graph, &matching).unwrap();
        pretty_assertions::assert_eq!(path, vec![0, 1, 2, 3, 4, 5, 6, 7]);
    }
}