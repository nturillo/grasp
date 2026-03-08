use std::collections::{BTreeMap, HashMap, HashSet};

use crate::{algorithms::algo_traits::AlgoTrait, graph::{DiGraph, Set, SimpleGraph, UnderlyingGraph, VertexID}};

/// Determine if a simple graph is connected.
pub fn is_connected<G: SimpleGraph>(g: &G) -> bool {
    g.vertices().next().map_or(false, |vert| g.dfs_iter(vert).unwrap().count() == g.vertex_count())
}

/// Determine if a digraph is weakly connected.
pub fn is_weakly_connected<G: UnderlyingGraph>(g: &G) -> bool {
    is_connected(&g.underlying_graph())
}

/// Determine if a digraph is strongly connected.
pub fn is_strongly_connected<G: DiGraph>(g: &G) -> bool {
    strongly_connected_components(g).len() == 1
}

/// Return the strongly connected components of a digraph.
pub fn strongly_connected_components<G: DiGraph>(g: &G) -> Vec<G::VertexSet> {
    #[derive(Clone, Copy)]
    struct VertexWrapper {
        pub disc: u32,
        pub low: u32,
        pub on_stack: bool,
    }

    let mut stack = vec![];
    let mut index = 0;
    let mut vertex_map: HashMap<VertexID, VertexWrapper> = HashMap::new();
    let mut comps: Vec<G::VertexSet> = Vec::new();

    fn visit<G: DiGraph>(index: &mut u32, vertex_id: VertexID, g: &G, vertex_map: &mut HashMap<VertexID, VertexWrapper>, stack: &mut Vec<VertexID>, comps: &mut Vec<G::VertexSet>) {
        vertex_map.insert(vertex_id, VertexWrapper { disc: *index, low: *index, on_stack: true });
        *index += 1;
        stack.push(vertex_id);

        for target_id in g.out_neighbors(vertex_id).unwrap().into_owned().into_iter() {
            if let Some(target) = vertex_map.get(&target_id).copied() {
                if target.on_stack {
                    let vertex = vertex_map.get_mut(&vertex_id).unwrap();
                    vertex.low = vertex.low.min(target.disc);
                }
            } else {
                visit(index, target_id, g, vertex_map, stack, comps);
                let target = vertex_map.get(&target_id).copied().unwrap();
                let vertex = vertex_map.get_mut(&vertex_id).unwrap();
                vertex.low = vertex.low.min(target.low);
            }
        }

        let vertex = vertex_map.get_mut(&vertex_id).unwrap();
        if vertex.low == vertex.disc {
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

#[cfg(test)]
mod test {
    use crate::{algorithms::connectivity::*, graph::{GraphTrait, prelude::{SparseDiGraph, SparseSimpleGraph}}};

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
        pretty_assertions::assert_eq!(true, sscs.contains(&v1));
        pretty_assertions::assert_eq!(true, sscs.contains(&v2));
        pretty_assertions::assert_eq!(true, sscs.contains(&v3));
    }
}