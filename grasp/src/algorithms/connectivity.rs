use std::collections::{BTreeMap, HashSet};

use crate::{algorithms::algo_traits::AlgoTrait, graph::{DiGraph, SimpleGraph, UnderlyingGraph, VertexID}};

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

pub fn strongly_connected_components<G: DiGraph>(g: &G) -> Vec<HashSet<VertexID>> {
    struct VertexWrapper {
        pub disc: Option<u32>,
        pub low: Option<u32>,
        pub on_stack: bool,
    }

    let mut stack = vec![];
    let mut index = 0;
    let mut vertex_map = BTreeMap::new();
    let mut comps: Vec<HashSet<VertexID>> = Vec::new();

    for vertex in g.vertices() {
        vertex_map.insert(vertex, VertexWrapper {
            disc: None,
            low: None,
            on_stack: false,
        });
    }

    fn visit<G: DiGraph>(index: &mut u32, vertex_id: VertexID, g: &G, vertex_map: &mut BTreeMap<usize, VertexWrapper>, stack: &mut Vec<VertexID>, comps: &mut Vec<HashSet<VertexID>>) {
        let vertex = vertex_map.get_mut(&vertex_id).unwrap();

        vertex.disc = Some(*index);
        vertex.low = Some(*index);

        *index += 1;

        stack.push(vertex_id);
        vertex.on_stack = true;

        for target_id in g.neighbors(vertex_id).unwrap().into_owned().into_iter() {
            if vertex_map.get(&target_id).unwrap().disc.is_none() {
                visit(index, target_id, g, vertex_map, stack, comps);
                vertex_map.get_mut(&vertex_id).unwrap().low = vertex_map.get_mut(&vertex_id).unwrap().low.min(vertex_map.get(&target_id).unwrap().low);
            } else if  vertex_map.get(&target_id).unwrap().on_stack {
                vertex_map.get_mut(&vertex_id).unwrap().low = vertex_map.get_mut(&vertex_id).unwrap().low.min(vertex_map.get(&target_id).unwrap().disc);
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

            comps.push(scc);
        }
    }

    for vertex in g.vertices() {
        if vertex_map.get(&vertex).unwrap().disc.is_none() {
            visit(&mut index, vertex, g, &mut vertex_map, &mut stack, &mut comps);
        }
    }

    comps
}