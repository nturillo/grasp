use std::{collections::{HashMap, HashSet}, hash::Hash};

use crate::{algorithms::distance::{self, graph_distance}, graph::{EdgeID, Set, SimpleGraph, VertexID}};



pub fn blossom<G: SimpleGraph>(g: &G) -> Vec<EdgeID> {
    let mut res = Vec::new();
    

    res
}

fn find_augmenting_path<G: SimpleGraph>(g: &G, m: &HashMap<VertexID, EdgeID>) -> Vec<VertexID> {
    let mut path = Vec::new();
    let mut F: HashMap<VertexID, VertexID> = HashMap::new();

    fn F_distance(F: &HashMap<VertexID, VertexID>, u: VertexID, v: VertexID) -> Option<u64> {
        if !F.contains_key(&u) || !F.contains_key(&v) {
            return None;
        }
        let mut t = u;
        let mut res = 0;
        while t != v {
            res += 1;
            t = F[&u];
        }
        Some(res)
    }
    fn get_root(F: &HashMap<VertexID, VertexID>, v: VertexID) -> Option<VertexID> {
        if !F.contains_key(&v) {
            return None;
        }
        let mut t = v;
        while F[&t] != t {
            t = F[&t];
        }
        Some(t)
    }
    fn path_to_root(F: &HashMap<VertexID, VertexID>, v: VertexID) -> Vec<VertexID> {
        let mut res = Vec::new();
        if !F.contains_key(&v) {
            return res;
        }
        let mut t = v;
        while F[&t] != t {
            res.push(t.clone());
            t = F[&t];
        }
        res
    }
    
    let mut marked_vertices: HashSet<VertexID> = HashSet::new();
    let mut marked_edges = m.clone();
    
    for v in g.vertices() {
        if m.contains_key(&v) {
            continue;
        }
        F.insert(v, v);
    }
    
    
    loop {
        let mut v = None;
        for &vert in F.keys() {
            if !marked_vertices.contains(&vert) && (F_distance(&F, vert, get_root(&F, vert).unwrap()).unwrap() % 2 == 0) {
                v = Some(vert);
                break;
            }
        }
        if v.is_none() {
            break;
        }
        let v = v.unwrap();

        for &w in g.neighbors(v).unwrap().iter() {
            if marked_edges.get(&v).is_some_and(|&e| e == (v, w)) {
                if !F.contains_key(&w) {
                    let x = m[&w].1;
                    F.insert(w, v);
                    F.insert(x, w);
                } else {
                    if F_distance(&F, get_root(&F, w).unwrap(), w).unwrap() % 2 == 0 {
                        if get_root(&F, v) != get_root(&F, w) {
                            let P = path_to_root(&F, v).into_iter()
                                .rev()
                                .chain(
                                    path_to_root(&F, w).into_iter()
                                );
                            return P.collect();
                        } else {

                        }

                    }
                }
            }
        }
    }

    path
}

//fn max_matching<G: SimpleGraph>(g: &G, m: &Vec<EdgeID>) -> Vec<Edge