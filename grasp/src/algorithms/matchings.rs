use std::collections::{HashSet, HashMap};

use crate::{algorithms::distance, graph::{EdgeID, SimpleGraph, VertexID}};



pub fn blossom<G: SimpleGraph>(g: &G) -> Vec<EdgeID> {
    let mut res = Vec::new();
    

    res
}

fn find_augmenting_path<G: SimpleGraph>(g: &G, m: &HashMap<VertexID, EdgeID>) -> Vec<VertexID> {
    let mut path = Vec::new();
    let mut F = G::default();
    
    let mut unmarked_vertices: HashSet<_> = g.vertices().collect();
    let mut marked_edges: HashSet<_> = m.values().cloned().collect();
    
    for v in g.vertices() {
        if m.contains_key(&v) {
            continue
        }
        F.add_vertex(v);
    }
    
    for v in unmarked_vertices.iter().filter(|&&v| g)

    path
}

//fn max_matching<G: SimpleGraph>(g: &G, m: &Vec<EdgeID>) -> Vec<Edge