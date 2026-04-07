use std::{collections::{HashMap, HashSet}, error::Error, fmt::Display};

use crate::graph::{VertexID, prelude::SimpleGraph, set::Set};

#[derive(Debug, Eq, PartialEq)]
pub struct BoundError;
impl Display for BoundError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Could not find the chromatic number within given bounds"))
    }
}
impl Error for BoundError{}

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

fn recurse<G: SimpleGraph>(g: &G, colors: &mut HashMap<VertexID, Option<usize>>, degree: &HashMap<VertexID, usize>, bound: usize) -> bool {
    let v = g.vertices()
        .filter(|u| colors[u].is_none())
        .max_by_key(|u| {
            let saturation = g.neighbors(*u).iter()
                .filter_map(|n| colors[n])
                .collect::<HashSet<_>>().len();
            (saturation, degree[u])
        });

    if v.is_none() {return true;}
    let v = v.unwrap();

    let neighbor_colors: HashSet<usize> = g.neighbors(v).iter()
        .filter_map(|u| colors[u])
        .collect();

    (0..bound).any(|i| {
        if neighbor_colors.contains(&i) {return false;}
        colors.insert(v, Some(i));
        if !recurse(g, colors, degree, bound) {
            colors.insert(v, None);
            return false;
        }
        return true;
    })
}

/// Find the exact chromatic number of a simple graph via backtracking with an upper bound.
pub fn chromatic_number_upper_bound<G: SimpleGraph>(g: &G, upper_bound: usize) -> Result<Vec<impl Set<Item = VertexID>>, BoundError> {
    if upper_bound < 1 {return Err(BoundError);}
    if g.vertex_count() == 0 {return Ok(vec![])}

    let mut colors: HashMap<VertexID, Option<usize>> = g.vertices().map(|v| (v, None)).collect();
    let degree: HashMap<VertexID, usize> = g.vertices().map(|v| (v, g.neighbors(v).len())).collect();

    if recurse(g, &mut colors, &degree, upper_bound) {
        let mut res: Vec<HashSet<VertexID>> = vec![Default::default(); upper_bound];
        for (&vertex, &color) in colors.iter() {
            res[color.unwrap()].insert(vertex);
        }

        Ok(match chromatic_number_upper_bound(g, upper_bound - 1) {
            Ok(opt) => opt,
            Err(_) => res
        })
    } else {
        Err(BoundError)
    }
}

/// Find the exact chromatic number of a simple graph via backtracking with a lower bound.
/// WARNING: This algorithm will return the passed lower bound if it is higher than the chromatic number
pub fn chromatic_number_lower_bound<G: SimpleGraph>(g: &G, lower_bound: usize) -> Vec<impl Set<Item = VertexID>> {
    if g.vertex_count() == 0 {return vec![]}

    let mut colors: HashMap<VertexID, Option<usize>> = g.vertices().map(|v| (v, None)).collect();
    let degree: HashMap<VertexID, usize> = g.vertices().map(|v| (v, g.neighbors(v).len())).collect();

    if recurse(g, &mut colors, &degree, lower_bound) {
        let mut res: Vec<HashSet<VertexID>> = vec![Default::default(); lower_bound];
        for (&vertex, &color) in colors.iter() {
            res[color.unwrap()].insert(vertex);
        }
        res
    } else {
        chromatic_number_lower_bound(g, lower_bound + 1)
    }
}

/// Find the exact chromatic number of a simple graph via backtracking with an upper and lower bound.
pub fn chromatic_number_bounded<G: SimpleGraph>(g: &G, lower_bound: usize, upper_bound: usize) -> Result<Vec<impl Set<Item = VertexID>>, BoundError> {
    if upper_bound < lower_bound {return Err(BoundError);}
    if g.vertex_count() == 0 {return Ok(vec![])}

    let mut colors: HashMap<VertexID, Option<usize>> = g.vertices().map(|v| (v, None)).collect();
    let degree: HashMap<VertexID, usize> = g.vertices().map(|v| (v, g.neighbors(v).len())).collect();

    if recurse(g, &mut colors, &degree, upper_bound) {
        let mut res: Vec<HashSet<VertexID>> = vec![Default::default(); upper_bound];
        for (&vertex, &color) in colors.iter() {
            res[color.unwrap()].insert(vertex);
        }

        Ok(match chromatic_number_bounded(g, lower_bound, upper_bound - 1) {
            Ok(opt) => opt,
            Err(_) => res
        })
    } else {
        Err(BoundError)
    }
}

#[cfg(test)]
mod tests {
    use crate::{algorithms::coloring::*, graph::{AnyVertexGraph, prelude::SparseSimpleGraph}};

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

    #[test]
    fn upper_backtracking_test() {
        let mut g = SparseSimpleGraph::default();
        g.add_edge((1,2));
        g.add_edge((2,3));
        g.add_edge((1,3));
        let coloring = chromatic_number_upper_bound(&g, 3);
        pretty_assertions::assert_eq!(coloring.unwrap().len(), 3);

        let mut g = SparseSimpleGraph::default();
        g.add_edge((0,1));
        g.add_edge((0,3));
        g.add_edge((1,2));
        g.add_edge((2,3));
        let coloring = chromatic_number_upper_bound(&g, 3);
        pretty_assertions::assert_eq!(coloring.unwrap().len(), 2);

        let mut g = SparseSimpleGraph::default();
        g.add_edge((0,1));
        g.add_edge((0,2));
        g.add_edge((0,3));
        g.add_edge((1,2));
        g.add_edge((1,3));
        g.add_edge((2,3));
        let coloring = chromatic_number_upper_bound(&g, 3);
        pretty_assertions::assert_eq!(coloring.is_err(), true);

        let g = SparseSimpleGraph::default();
        let coloring = chromatic_number_upper_bound(&g, 1);
        pretty_assertions::assert_eq!(coloring.unwrap().len(), 0);

        let mut g = SparseSimpleGraph::default();
        g.add_vertex(0);
        g.add_vertex(1);
        let coloring = chromatic_number_upper_bound(&g, 3);
        pretty_assertions::assert_eq!(coloring.unwrap().len(), 1);
    }

    #[test]
    fn lower_backtracking_test() {
        let mut g = SparseSimpleGraph::default();
        g.add_edge((1,2));
        g.add_edge((2,3));
        g.add_edge((1,3));
        let coloring = chromatic_number_lower_bound(&g, 2);
        pretty_assertions::assert_eq!(coloring.len(), 3);

        let mut g = SparseSimpleGraph::default();
        g.add_edge((0,1));
        g.add_edge((0,3));
        g.add_edge((1,2));
        g.add_edge((2,3));
        let coloring = chromatic_number_lower_bound(&g, 1);
        pretty_assertions::assert_eq!(coloring.len(), 2);

        let mut g = SparseSimpleGraph::default();
        g.add_edge((0,1));
        g.add_edge((0,2));
        g.add_edge((0,3));
        g.add_edge((1,2));
        g.add_edge((1,3));
        g.add_edge((2,3));
        let coloring = chromatic_number_lower_bound(&g, 5);
        pretty_assertions::assert_eq!(coloring.len(), 5);

        let g = SparseSimpleGraph::default();
        let coloring = chromatic_number_lower_bound(&g, 1);
        pretty_assertions::assert_eq!(coloring.len(), 0);

        let mut g = SparseSimpleGraph::default();
        g.add_vertex(0);
        g.add_vertex(1);
        let coloring = chromatic_number_lower_bound(&g, 1);
        pretty_assertions::assert_eq!(coloring.len(), 1);
    }

    #[test]
    fn bounded_backtracking_test() {
        let mut g = SparseSimpleGraph::default();
        g.add_edge((1,2));
        g.add_edge((2,3));
        g.add_edge((1,3));
        let coloring = chromatic_number_bounded(&g, 2, 5);
        pretty_assertions::assert_eq!(coloring.unwrap().len(), 3);

        let mut g = SparseSimpleGraph::default();
        g.add_edge((0,1));
        g.add_edge((0,3));
        g.add_edge((1,2));
        g.add_edge((2,3));
        let coloring = chromatic_number_bounded(&g, 1, 3);
        pretty_assertions::assert_eq!(coloring.unwrap().len(), 2);

        let mut g = SparseSimpleGraph::default();
        g.add_edge((0,1));
        g.add_edge((0,2));
        g.add_edge((0,3));
        g.add_edge((1,2));
        g.add_edge((1,3));
        g.add_edge((2,3));
        let coloring = chromatic_number_bounded(&g, 1, 3);
        pretty_assertions::assert_eq!(coloring.is_err(), true);

        let g = SparseSimpleGraph::default();
        let coloring = chromatic_number_bounded(&g, 2, 5);
        pretty_assertions::assert_eq!(coloring.unwrap().len(), 0);

        let mut g = SparseSimpleGraph::default();
        g.add_vertex(0);
        g.add_vertex(1);
        let coloring = chromatic_number_bounded(&g, 1, 3);
        pretty_assertions::assert_eq!(coloring.unwrap().len(), 1);
    }
}