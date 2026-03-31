use std::collections::{HashMap, HashSet, VecDeque};
use crate::{algorithms::distance::{graph_distance, shortest_path}, graph::prelude::*};

pub fn maximum_matching<G: SimpleGraph>(graph: &G) -> Matching {
    let mut blossom_graph = BlossomGraph::new(graph);
    let mut matching = Matching::new();
    while let Some(augmenting_path) = find_augmenting_path(&mut blossom_graph, &matching) {
        for edge in augmenting_path.windows(2).skip(1).step_by(2) {
            let edge = (edge[0], edge[1]);
            matching.remove_edge(edge);
        }
        for edge in augmenting_path.windows(2).step_by(2) {
            let edge = (edge[0], edge[1]);
            matching.add_edge(edge).unwrap();
        }
    }
    matching
}

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
        if edge.0 == edge.1 {
            return Err(crate::graph::prelude::GraphError::EdgeNotAddable(
                edge,
                "Loop edges are not allowed in a matching".to_string(),
            ));
        }
        if self.has_edge(edge) {
            return Ok(());
        }
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
    pub fn remove_edge(&mut self, edge: EdgeID) {
        if self.has_edge(edge) {
            self.vertices.remove(&edge.0);
            self.vertices.remove(&edge.1);
            self.edges.remove(&edge.0);
            self.edges.remove(&edge.1);
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

/// Bespoke graph type for the blossom algorithm, which supports contracting and expanding blossoms.
struct BlossomGraph {
    contractions: Vec<(VertexID, SparseSimpleGraph, Vec<EdgeID>)>,
    current_graph: SparseSimpleGraph,
}

impl BlossomGraph {
    pub fn new<G: SimpleGraph>(graph: &G) -> Self {
        let mut current_graph = SparseSimpleGraph::with_capacity(graph.vertex_count(), graph.edge_count());
        for v in graph.vertices() {
            current_graph.add_vertex(v);
        }
        for e in graph.edges() {
            current_graph.add_edge(e);
        }
        BlossomGraph {
            contractions: Vec::new(),
            current_graph,
        }
    }
    pub fn contract(&mut self, blossom: SparseSimpleGraph) -> VertexID {
        let new_vertex = self.current_graph.vertices().max().unwrap() + 1;
        let removed_edges = self.current_graph.edges().filter(|(u,v)| blossom.has_vertex(*u) || blossom.has_vertex(*v)).collect::<Vec<_>>();
        self.current_graph.add_vertex(new_vertex);
        for v in blossom.vertices() {
            let neighbors = self.current_graph.neighbors(v).iter().cloned().collect::<Vec<_>>();
            for u in neighbors {
                self.current_graph.add_edge((new_vertex, u));
            }
        }
        for v in blossom.vertices() {
            let _ = self.current_graph.remove_vertex(v);
        }
        self.contractions.push((new_vertex, blossom, removed_edges));
        new_vertex
    }
    pub fn expand(&mut self) -> Option<(usize, SparseSimpleGraph, Vec<EdgeID>)> {
        if let Some((new_vertex, blossom, removed_edges)) = self.contractions.last() {
            self.current_graph.remove_vertex(*new_vertex);
            for e in removed_edges {
                self.current_graph.add_edge(*e);
            }
        }
        self.contractions.pop()
    }
    pub fn lift_path(&mut self, contracted_path: Vec<VertexID>, matching: &Matching) -> Vec<VertexID> {
        if self.contractions.is_empty() {
            return contracted_path;
        }
        let (new_vertex, mut blossom, _) = self.expand().unwrap();
        let mut path = Vec::with_capacity(contracted_path.len() + blossom.vertex_count() - 1);
        let exposed_vertex = blossom.vertices().find(|&v| {
            !matching.has_vertex(v) || !blossom.has_vertex(matching.neighbor(v).unwrap())
        }).unwrap();
        let mut new_index = contracted_path.iter().position(|&v| v == new_vertex).unwrap();
        let mut contracted_path = contracted_path;
        if contracted_path.last() == Some(&new_vertex) {
            contracted_path = contracted_path.into_iter().rev().collect();
            new_index = contracted_path.len() - 1 - new_index;
        }
        path.extend_from_slice(&contracted_path[0..new_index]);
        let mut left: VertexID;
        let mut right: VertexID;
        if contracted_path.first() == Some(&new_vertex) || self.current_graph.has_edge((contracted_path[new_index-1], exposed_vertex)) {
            left = exposed_vertex;
            right = blossom.vertices().find(|&v| self.current_graph.has_edge((contracted_path[new_index+1], v))).unwrap();
        } else {
            left = blossom.vertices().find(|&v| self.current_graph.has_edge((contracted_path[new_index-1], v))).unwrap();
            right = exposed_vertex;
        }
        let sp= shortest_path(&blossom, left, right).unwrap();
        if sp.len() % 2 == 1 {
            path.extend(sp);
        } else {
            let _ = blossom.remove_edge((left, sp[1]));
            path.extend(shortest_path(&blossom, left, right).unwrap());
        }
        path.extend_from_slice(&contracted_path[new_index+1..]);
        path
    }
}

impl GraphTrait for BlossomGraph {
    fn vertex_count(&self) -> usize {
        self.current_graph.vertex_count()
    }
    fn edge_count(&self) -> usize {
        self.current_graph.edge_count()
    }
    fn has_vertex(&self, v: VertexID) -> bool {
        self.current_graph.has_vertex(v)
    }
    fn has_edge(&self, e: EdgeID) -> bool {
        self.current_graph.has_edge(e)
    }
    fn vertices(&self) -> impl Iterator<Item=VertexID> {
        self.current_graph.vertices()
    }
    fn edges(&self) -> impl Iterator<Item=EdgeID> {
        self.current_graph.edges()
    }
    fn neighbors(&self, v: VertexID) -> impl Set<Item = VertexID> {
        self.current_graph.neighbors(v)
    }
    fn vertex_set(&self) -> impl Set<Item = VertexID> {
        self.current_graph.vertex_set()
    }
}

impl SimpleGraph for BlossomGraph {}

fn find_augmenting_path(graph: &mut BlossomGraph, matching: &Matching) -> Option<Vec<VertexID>> {
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
        let neighbors = graph.neighbors(v).iter().cloned().collect::<Vec<_>>();
        for w in neighbors {
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
                    let new_vertex = graph.contract(blossom.clone());
                    let mut contracted_matching = Matching::new();
                    for e in matching.edges() {
                        match (blossom.has_vertex(e.0), blossom.has_vertex(e.1)) {
                            (true, true) => continue,
                            (true, false) => contracted_matching.add_edge((new_vertex, e.1)).unwrap(),
                            (false, true) => contracted_matching.add_edge((e.0, new_vertex)).unwrap(),
                            (false, false) => contracted_matching.add_edge(e).unwrap(),
                        }
                    }
                    if let Some(contracted_path) = find_augmenting_path(graph, &contracted_matching) {
                        return Some(graph.lift_path(contracted_path, &matching));
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
        let mut blossom_graph = BlossomGraph::new(&graph);
        let new_vertex = blossom_graph.contract(blossom.clone());

        let mut expected_graph = SparseSimpleGraph::with_capacity(4, 0);
        expected_graph.add_edge((0, new_vertex));
        expected_graph.add_edge((4, new_vertex));
        expected_graph.add_edge((5, new_vertex));

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
        graph.add_edge((4,6));

        let blossom = graph.subgraph_vertex([1,2,3]);
        let mut blossom_graph = BlossomGraph::new(&graph);
        let new_vertex = blossom_graph.contract(blossom.clone());

        let mut matching = Matching::new();
        matching.add_edge((1,2)).unwrap();
        matching.add_edge((3,4)).unwrap();

        let contracted_path = vec![0, new_vertex, 4, 6];
        let lifted_path = blossom_graph.lift_path(contracted_path, &matching);
        assert!(lifted_path == vec![6,4,3,2,1,0] || lifted_path == vec![0,1,2,3,4,6]);
    }

    #[test]
    fn augmenting_test() {
        let mut graph = SparseSimpleGraph::with_capacity(7, 8);
        for e in [(0,1), (1,2), (2,3), (3,4), (4,5), (5,6), (6,7), (6,2)] {
            graph.add_edge(e);
        }
        let mut matching = Matching::new();
        for e in [(1,2), (3,4), (5,6)] {
            matching.add_edge(e).unwrap();
        }
        let mut blossom_graph = BlossomGraph::new(&graph);
        let path = find_augmenting_path(&mut blossom_graph, &matching).unwrap();
        assert!((path == (0..=7).collect::<Vec<_>>() || path == (0..=7).rev().collect::<Vec<_>>()));
    }

    #[test]
    fn single_edge() {
        let mut graph = SparseSimpleGraph::with_capacity(2, 1);
        graph.add_edge((0,1));

        let mut expected_matching = Matching::new();
        expected_matching.add_edge((0,1)).unwrap();

        let matching = maximum_matching(&graph);
        assert_graphs_eq!(matching, expected_matching);
    }

    #[test]
    fn two_disjoint_edges() {
        let mut graph = SparseSimpleGraph::with_capacity(4, 2);
        graph.add_edge((0,1));
        graph.add_edge((2,3));

        let mut expected_matching = Matching::new();
        expected_matching.add_edge((0,1)).unwrap();
        expected_matching.add_edge((2,3)).unwrap();

        let matching = maximum_matching(&graph);
        assert_graphs_eq!(matching, expected_matching);
    }

    #[test]
    fn teardrop() {
        let mut graph = SparseSimpleGraph::with_capacity(4,4);
        graph.add_edge((0,1));
        graph.add_edge((1,2));
        graph.add_edge((2,3));
        graph.add_edge((3,1));

        let mut expected_matching = Matching::new();
        expected_matching.add_edge((0,1)).unwrap();
        expected_matching.add_edge((2,3)).unwrap();

        let matching = maximum_matching(&graph);
        assert_graphs_eq!(matching, expected_matching);
    }

    #[test]
    fn cycle_with_chord() {
        let mut graph = SparseSimpleGraph::empty();
        for e in (0..=5).collect::<Vec<usize>>().windows(2) {
            graph.add_edge((e[0], e[1]));
        }
        graph.add_edge((0,5));
        graph.add_edge((1,5));

        let matching = maximum_matching(&graph);
        assert!(matching.edge_count() == 3);
    }
}