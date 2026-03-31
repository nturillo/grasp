use std::collections::{HashMap, HashSet};

use crate::graph::{VertexID, prelude::SimpleGraph, set::Set};

// Applies DSatur on a simple graph.
pub fn dsatur<G: SimpleGraph>(g: &G) -> Vec<impl Set<Item = VertexID>> {
    let mut colors: HashMap<VertexID, Option<usize>> = g.vertices().map(|v| (v, None)).collect();
    let mut saturation: HashMap<VertexID, HashSet<usize>> = g.vertices().map(|v| (v, Default::default())).collect();
    let degree: HashMap<VertexID, usize> = g.vertices().map(|v| (v, g.neighbors(v).len())).collect();

    for _ in 0..g.vertex_count() {
        let v = g.vertices()
            .filter(|u| colors[u].is_none())
            .max_by_key(|u| (saturation[u].len(), degree[u]))
            .unwrap();

        let neighbor_colors: HashSet<usize> = g.neighbors(v).iter()
            .filter_map(|u| colors[u])
            .collect();

        let color = (0..).find(|i| !neighbor_colors.contains(i)).unwrap();
        colors.insert(v, Some(color));

        for neighbor in g.neighbors(v).iter() {
            if colors[neighbor].is_none() {
                saturation.get_mut(neighbor).unwrap().insert(color);
            }
        }
    }

    let max_color = colors.values().filter_map(|&c| c).max().unwrap_or(0);
    let mut res: Vec<HashSet<VertexID>> = vec![Default::default(); max_color + 1];
    for (&vertex, &color) in colors.iter() {
        res[color.unwrap()].insert(vertex);
    }

    res
}

#[cfg(test)]
mod tests {
    use crate::{algorithms::coloring::dsatur, graph::{AnyVertexGraph, prelude::SparseSimpleGraph}};

    #[test]
    fn dsatur_test() {
        let mut g = SparseSimpleGraph::default();
        g.add_edge((1,2));
        g.add_edge((2,3));
        g.add_edge((1,3));
        g.add_edge((1,4));
        g.add_edge((1,5));
        g.add_edge((4,5));
        let coloring = dsatur(&g);
        pretty_assertions::assert_eq!(coloring.len(), 3);

        let mut g = SparseSimpleGraph::default();
        g.add_edge((0,1));
        g.add_edge((0,3));
        g.add_edge((1,2));
        g.add_edge((2,3));
        let coloring = dsatur(&g);
        pretty_assertions::assert_eq!(coloring.len(), 2);
    }
}