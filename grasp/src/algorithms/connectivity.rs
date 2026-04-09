use std::{collections::{HashMap, HashSet, VecDeque}, vec};

use graph_ops_macros::register;

use crate::{
    algorithms::algo_traits::AlgoTrait, 
    graph::prelude::*
};

#[register(name = "Is Connected", desc = "Returns if the graph is connected.", ret = String, simple = "true", params = [])]
/// Determine if a simple graph is connected.
pub fn is_connected<G: SimpleGraph>(g: &G) -> bool {
    g.vertices().next().map_or(false, |vert| g.dfs_iter(vert).unwrap().count() == g.vertex_count())
}

#[register(name = "Is Weakly Connected", desc = "Returns if the graph is weakly connected.", ret = String, simple = "false", params = [])]
/// Determine if a digraph is weakly connected.
pub fn is_weakly_connected<G: DigraphProjection>(g: &G) -> bool {
    is_connected(&g.as_simple())
}

#[register(name = "Is Strongly Connected", desc = "Returns if the graph is strongly connected.", ret = String, simple = "false", params = [])]
/// Determine if a digraph is strongly connected.
pub fn is_strongly_connected<G: DiGraph>(g: &G) -> bool {
    strongly_connected_components(g).len() == 1
}

#[register(name = "Strongly Connected Components", desc = "Colors each vertex in accordance to the component they belong to.", ret = VertexCluster, simple = "false", params = [])]
/// Return the strongly connected components of a digraph.
pub fn strongly_connected_components<G: DiGraph>(g: &G) -> Vec<impl Set<Item = VertexID>> {
    struct VertexWrapper {
        pub disc: u32,
        pub low: u32,
        pub on_stack: bool,
    }

    let mut stack = vec![];
    let mut index = 0;
    let mut vertex_map: HashMap<VertexID, VertexWrapper> = HashMap::new();
    let mut comps: Vec<HashSet<VertexID>> = Vec::new();

    fn visit<G: DiGraph>(index: &mut u32, vertex_id: VertexID, g: &G, vertex_map: &mut HashMap<VertexID, VertexWrapper>, stack: &mut Vec<VertexID>, comps: &mut Vec<HashSet<VertexID>>) {
        let mut low = *index;
        let disc = *index;
        vertex_map.insert(vertex_id, VertexWrapper { disc: disc, low: low, on_stack: true });
        *index += 1;
        stack.push(vertex_id);

        for target_id in g.out_neighbors(vertex_id).iter() {
            if let Some(target) = vertex_map.get(&target_id) {
                if target.on_stack { low = low.min(target.disc); }
            } else {
                visit(index, *target_id, g, vertex_map, stack, comps);
                low = low.min(vertex_map.get(&target_id).unwrap().low);
            }
        }

        vertex_map.get_mut(&vertex_id).unwrap().low = low;
        if low == disc {
            let mut scc = HashSet::new();

            loop {
                let w = stack.pop().unwrap();
                vertex_map.get_mut(&w).unwrap().on_stack = false;
                scc.insert(w);

                if w == vertex_id {
                    break;
                }
            };

            comps.push(scc.iter().copied().collect());
        }
    }

    for vertex in g.vertices() {
        if !vertex_map.contains_key(&vertex) {
            visit(&mut index, vertex, g, &mut vertex_map, &mut stack, &mut comps);
        }
    }

    comps
}

#[register(name = "Cut Vertices", desc = "Highlights the graph's cut vertices.", ret = VertexList, simple = "true", params = [])]
/// Returns a simple graph's cut vertices.
pub fn cut_vertices<G: SimpleGraph>(g: &G) -> impl Set<Item = VertexID> {
    struct VertexWrapper {
        pub disc: u32,
        pub low: u32,
    }

    let mut index = 0;
    let mut vertex_map: HashMap<VertexID, VertexWrapper> = HashMap::new();
    let mut points: HashSet<VertexID> = HashSet::new();

    fn visit<G: SimpleGraph>(index: &mut u32, vertex_id: VertexID, g: &G, parent: Option<VertexID>, vertex_map: &mut HashMap<VertexID, VertexWrapper>, points: &mut HashSet<VertexID>) {
        let mut low = *index;
        let mut children: u32 = 0;
        let disc = *index;
        vertex_map.insert(vertex_id, VertexWrapper { disc: disc, low: low });
        *index += 1;

        for target_id in g.neighbors(vertex_id).iter() {
            if let Some(target) = vertex_map.get(&target_id) {
                if Some(*target_id) != parent { low = low.min(target.disc); }
            } else {
                children += 1;
                visit(index, *target_id, g, Some(vertex_id), vertex_map, points);

                let target_low = vertex_map.get(&target_id).unwrap().low;
                low = low.min(target_low);

                if parent.is_some() && target_low >= disc { points.insert(vertex_id); }
            }
        }

        vertex_map.get_mut(&vertex_id).unwrap().low = low;
        if parent.is_none() && children > 1 {
            points.insert(vertex_id);
        }
    }

    for vertex in g.vertices() {
        if !vertex_map.contains_key(&vertex) {
            visit(&mut index, vertex, g, None, &mut vertex_map, &mut points);
        }
    }

    points
}

#[register(name = "Bridges", desc = "Highlights the graph's bridges.", ret = EdgeList, simple = "true", params = [])]
/// Returns a simple graph's bridges.
pub fn bridges<G: SimpleGraph>(g: &G) -> impl Set<Item = EdgeID> {
    struct VertexWrapper {
        pub disc: u32,
        pub low: u32,
    }

    let mut index = 0;
    let mut vertex_map: HashMap<VertexID, VertexWrapper> = HashMap::new();
    let mut points: HashSet<EdgeID> = HashSet::new();

    fn visit<G: SimpleGraph>(index: &mut u32, vertex_id: VertexID, g: &G, parent: Option<VertexID>, vertex_map: &mut HashMap<VertexID, VertexWrapper>, points: &mut HashSet<EdgeID>) {
        let mut low = *index;
        let disc = *index;
        vertex_map.insert(vertex_id, VertexWrapper { disc: disc, low: low });
        *index += 1;

        for target_id in g.neighbors(vertex_id).iter() {
            if let Some(target) = vertex_map.get(&target_id) {
                if Some(*target_id) != parent { low = low.min(target.disc); }
            } else {
                visit(index, *target_id, g, Some(vertex_id), vertex_map, points);

                let target_low = vertex_map.get(&target_id).unwrap().low;
                low = low.min(target_low);

                if target_low > disc { points.insert((vertex_id.min(*target_id), (*target_id).max(vertex_id))); }
            }
        }

        vertex_map.get_mut(&vertex_id).unwrap().low = low;
    }

    for vertex in g.vertices() {
        if !vertex_map.contains_key(&vertex) {
            visit(&mut index, vertex, g, None, &mut vertex_map, &mut points);
        }
    }

    points
}

#[register(name = "Is Complete", desc = "Returns if the graph is complete.", ret = String, simple = "true", params = [])]
/// Returns if a simple graph is complete.
pub fn simple_graph_is_complete<G: SimpleGraph>(g: &G) -> bool {
    let n = g.vertex_count();
    g.edge_count() == n * (n - 1) / 2
}

#[register(name = "Is Complete", desc = "Returns if the graph is complete.", ret = String, simple = "false", params = [])]
/// Returns if a digraph is complete.
pub fn digraph_is_complete<G: DiGraph>(g: &G) -> bool {
    let n = g.vertex_count();
    g.edge_count() == n * (n - 1)
}

#[register(name = "Edge Connectivity", desc = "Returns the graph's edge connectivity.", ret = String, simple = "true", params = [])]
/// Returns a graph's edge connectivity.
pub fn edge_connectivity<G: SimpleGraph>(g: &G) -> u32 {
    if g.vertex_count() < 2 { return 0; };

    let mut flow_graph = HashMapLabeledDiGraph::<SparseDiGraph, (), u32>::default();
    for edge in g.edges() {
        flow_graph.add_edge(edge);
        flow_graph.set_edge_label(edge, 1);
        flow_graph.add_edge(edge.inv());
        flow_graph.set_edge_label(edge.inv(), 1);
    }

    let vertices: Vec<VertexID> = g.vertices().collect();
    let source = vertices[0];
    vertices[1..].iter().map(|&target| max_flow(&flow_graph, source, target)).min().unwrap_or(0)
}

#[register(name = "Vertex Connectivity", desc = "Returns the graph's vertex connectivity.", ret = String, simple = "true", params = [])]
/// Returns a graph's vertex connectivity.
pub fn vertex_connectivity<G: SimpleGraph>(g: &G) -> u32 {
    let mut min_flow = (g.vertex_count() - 1) as u32;
    if min_flow == 0 || g.is_empty() { return 0; };

    let mut flow_graph = HashMapLabeledDiGraph::<SparseDiGraph, (), u32>::default();
    let mut pairs = HashMap::<VertexID, (VertexID, VertexID)>::default();
    for v in g.vertices() {
        let (v1, v2) = (flow_graph.create_vertex(), flow_graph.create_vertex());
        pairs.insert(v,  (v1, v2));
        flow_graph.add_edge((v1, v2));
        flow_graph.set_edge_label((v1, v2), 1);
    }

    for (s, t) in g.edges() {
        let (u1, u2) = pairs[&s];
        let (v1, v2) = pairs[&t];

        flow_graph.add_edge((u2, v1));
        flow_graph.set_edge_label((u2, v1), u32::MAX);
        flow_graph.add_edge((v2, u1));
        flow_graph.set_edge_label((v2, u1), u32::MAX);
    }

    let vertices: Vec<VertexID> = g.vertices().collect();
    for i in 0..vertices.len() {
        for j in (i+1)..vertices.len() {
            let s = vertices[i];
            let t = vertices[j];
            if !g.has_edge((s, t)) {
                min_flow = min_flow.min(max_flow(&flow_graph, pairs[&s].1, pairs[&t].0));
            }
        }
    }
    min_flow
}

/// Returns the max flow between a source and target.
pub fn max_flow<G: LabeledGraph<EdgeData = u32> + DiGraph>(g: &G, source: VertexID, target: VertexID) -> u32
{
    fn bfs(g: &HashMapLabeledDiGraph::<SparseDiGraph, (), u32>, source: VertexID, target: VertexID, parent: &mut HashMap<VertexID, Option<VertexID>>) -> bool {
        let mut visited: HashSet<VertexID> = HashSet::with_capacity(g.vertex_count());
        let mut queue: VecDeque<VertexID> = Default::default();
        queue.push_back(source);
        parent.insert(source, None);
        visited.insert(source);

        while !queue.is_empty() {
            let cur = queue.pop_front().unwrap();

            for neighbor in g.out_neighbors(cur).iter() {
                if !visited.contains(&neighbor) && g.get_edge_label((cur, *neighbor)).is_some_and(|&cap| cap > 0) {
                    parent.insert(*neighbor, Some(cur));
                    if *neighbor == target {
                        return true;
                    }

                    queue.push_back(*neighbor);
                    visited.insert(*neighbor);
                }
            }
        }

        false
    }

    let mut flow = 0;
    let mut parent: HashMap<VertexID, Option<VertexID>> = Default::default();
    let mut residual = HashMapLabeledDiGraph::<SparseDiGraph, (), u32>::default();

    for (edge, cap) in g.edge_labels() {
        residual.add_edge(edge);
        residual.add_edge(edge.inv());
        residual.set_edge_label(edge, *cap);
    }

    while bfs(&residual, source, target, &mut parent) {
        let mut path = u32::MAX;
        let mut cur = target;
        while let &Some(cur_parent) = parent.get(&cur).unwrap() {
            path = path.min(*residual.get_edge_label((cur_parent, cur)).unwrap());
            cur = cur_parent;
        }

        let mut cur = target;
        while let &Some(cur_parent) = parent.get(&cur).unwrap() {
            let edge = (cur_parent, cur);
            residual.set_edge_label(edge, *residual.get_edge_label(edge).unwrap() - path);
            residual.set_edge_label(edge.inv(), *residual.get_edge_label(edge.inv()).unwrap_or(&0) + path);
            cur = cur_parent;
        }

        flow += path;
    }

    flow
}

#[cfg(test)]
mod test {
    use crate::{algorithms::connectivity::*, graph::{AnyVertexGraph, prelude::{SparseDiGraph, SparseSimpleGraph}}};

    #[test]
    pub fn empty_simple_connected() {
        let graph = SparseSimpleGraph::default();
        pretty_assertions::assert_eq!(false, is_connected(&graph));
    }

    #[test]
    pub fn simple_connected() {
        let mut graph = SparseSimpleGraph::default();
        graph.add_edge((1, 2));
        graph.add_edge((2, 3));
        graph.add_edge((2, 4));
        pretty_assertions::assert_eq!(true, is_connected(&graph));
    }

    #[test]
    pub fn simple_not_connected() {
        let mut graph = SparseSimpleGraph::default();
        graph.add_edge((1, 2));
        graph.add_edge((2, 3));
        graph.add_vertex(4);
        pretty_assertions::assert_eq!(false, is_connected(&graph));
    }

    #[test]
    pub fn digraph_only_weak() {
        let mut graph = SparseDiGraph::default();
        graph.add_edge((1, 2));
        graph.add_edge((2, 3));
        graph.add_edge((2, 4));
        pretty_assertions::assert_eq!(true, is_weakly_connected(&graph));
        pretty_assertions::assert_eq!(false, is_strongly_connected(&graph));
    }

    #[test]
    pub fn digraph_strongly_connected() {
        let mut graph = SparseDiGraph::default();
        graph.add_edge((1, 2));
        graph.add_edge((2, 3));
        graph.add_edge((3, 1));
        pretty_assertions::assert_eq!(true, is_strongly_connected(&graph));
    }

    #[test]
    pub fn digraph_not_connected() {
        let mut graph = SparseDiGraph::default();
        graph.add_edge((1, 2));
        graph.add_edge((2, 3));
        graph.add_vertex(4);
        pretty_assertions::assert_eq!(false, is_weakly_connected(&graph));
    }

    #[test]
    pub fn digraph_ssc() {
        let mut graph = SparseDiGraph::default();
        graph.add_edge((1, 2));
        graph.add_edge((2, 3));
        graph.add_edge((2, 4));
        graph.add_edge((3, 1));
        graph.add_edge((4, 5));
        graph.add_edge((5, 4));
        graph.add_edge((6, 1));
        let sscs = strongly_connected_components(&graph);
        let v1: HashSet<VertexID> = HashSet::from([6]);
        let v2: HashSet<VertexID> = HashSet::from([1, 2, 3]);
        let v3: HashSet<VertexID> = HashSet::from([4, 5]);
        pretty_assertions::assert_eq!(true, sscs.iter().map(|v| v.iter().map(|v| v.into_owned()).collect()).collect::<Vec<HashSet<VertexID>>>().contains(&v1));
        pretty_assertions::assert_eq!(true, sscs.iter().map(|v| v.iter().map(|v| v.into_owned()).collect()).collect::<Vec<HashSet<VertexID>>>().contains(&v2));
        pretty_assertions::assert_eq!(true, sscs.iter().map(|v| v.iter().map(|v| v.into_owned()).collect()).collect::<Vec<HashSet<VertexID>>>().contains(&v3));
    }

    #[test]
    pub fn complete_simple() {
        let mut graph = SparseSimpleGraph::default();
        graph.add_edge((1, 2));
        graph.add_edge((2, 3));
        graph.add_edge((3, 1));
        pretty_assertions::assert_eq!(true, simple_graph_is_complete(&graph));
    }

    #[test]
    pub fn not_complete_simple() {
        let mut graph = SparseSimpleGraph::default();
        graph.add_edge((1, 2));
        graph.add_edge((2, 3));
        graph.add_edge((3, 1));
        graph.add_edge((3, 4));
        pretty_assertions::assert_eq!(false, simple_graph_is_complete(&graph));
    }

    #[test]
    pub fn complete_digraph() {
        let mut graph = SparseDiGraph::default();
        graph.add_edge((1, 2));
        graph.add_edge((2, 1));
        graph.add_edge((2, 3));
        graph.add_edge((3, 2));
        graph.add_edge((3, 1));
        graph.add_edge((1, 3));
        pretty_assertions::assert_eq!(true, digraph_is_complete(&graph));
    }

    #[test]
    pub fn not_complete_digraph() {
        let mut graph = SparseDiGraph::default();
        graph.add_edge((1, 2));
        graph.add_edge((2, 3));
        graph.add_edge((3, 1));
        pretty_assertions::assert_eq!(false, digraph_is_complete(&graph));
    }

    #[test]
    pub fn check_cut_vertices() {
        let mut graph = SparseSimpleGraph::default();
        graph.add_edge((1, 2));
        graph.add_edge((2, 3));
        graph.add_edge((3, 1));
        graph.add_edge((3, 4));
        graph.add_edge((4, 5));
        graph.add_edge((6, 1));
        pretty_assertions::assert_eq!(HashSet::from([1, 3, 4]), cut_vertices(&graph).iter().map(|v| v.into_owned()).collect());
    }

    #[test]
    pub fn check_bridges() {
        let mut graph = SparseSimpleGraph::default();
        graph.add_edge((1, 2));
        graph.add_edge((2, 3));
        graph.add_edge((3, 1));
        graph.add_edge((3, 4));
        graph.add_edge((4, 5));
        graph.add_edge((6, 1));
        pretty_assertions::assert_eq!(HashSet::from([(1, 6), (3, 4), (4, 5)]), bridges(&graph).iter().map(|v| v.into_owned()).collect());
    }

    #[test]
    pub fn edge_connectivity_test() {
        let mut graph = SparseSimpleGraph::default();
        graph.add_edge((1, 2));
        graph.add_edge((2, 3));
        graph.add_edge((3, 1));
        graph.add_edge((3, 4));
        graph.add_edge((4, 5));
        graph.add_edge((6, 1));
        graph.add_edge((6, 5));
        pretty_assertions::assert_eq!(2, edge_connectivity(&graph));
    }

    #[test]
    pub fn vertex_connectivity_test() {
        let mut graph = SparseSimpleGraph::default();
        graph.add_edge((1, 2));
        graph.add_edge((2, 3));
        graph.add_edge((3, 1));
        graph.add_edge((3, 4));
        graph.add_edge((4, 5));
        graph.add_edge((6, 1));
        graph.add_edge((6, 5));
        pretty_assertions::assert_eq!(2, vertex_connectivity(&graph));
    }
}