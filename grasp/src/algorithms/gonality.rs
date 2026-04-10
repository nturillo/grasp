use crate::{algorithms::connectivity::is_connected, graph::prelude::*};

use std::{arch::x86_64::_SIDD_NEGATIVE_POLARITY, collections::{HashMap, HashSet}, hash::Hash};
use itertools::Itertools;

pub fn compute_gonality<G>(g: &G) -> Result<usize, GraphError>
where G: SimpleGraph {
    if !is_connected(g) {
        return Err(GraphError::DisconnectedGraph);
    }
    let mut upper_bound = g.vertices().count();
    let mut lower_bound = 0usize;
    
    while upper_bound - lower_bound > 1 {
        let mid = (upper_bound + lower_bound) / 2;
        if gonality_le(g, mid) {
            upper_bound = mid;
        } else {
            lower_bound = mid;
        }
    }
    if gonality_le(g, lower_bound) {
        Ok(lower_bound)
    } else {
        Ok(upper_bound)
    }
}

fn gonality_le<G: SimpleGraph>(g: &G, n: usize) -> bool {
    let vertices = g.vertices().collect::<Vec<_>>();
    if n >= vertices.len() {
        return true;
    }

    let cart_prod = vec![vertices; n].into_iter().multi_cartesian_product();
    
    let mut player_a_wins = false;
    for chip_placement in cart_prod {
        let mut divisor = HashMap::new();
        for v in chip_placement {
            *divisor.entry(v).or_insert(0i32) += 1;
        }
        let mut chipless_vertices = Vec::new();
        for v in g.vertices() {
            if divisor.contains_key(&v) {
                continue;
            }
            chipless_vertices.push(v);
            divisor.insert(v, 0);
        }
        let mut player_b_wins = false;
        for v in chipless_vertices {
            let mut divisor_copy = divisor.clone();
            *divisor_copy.get_mut(&v).unwrap() -= 1;
            if !dhar(g, divisor_copy, v) {
                player_b_wins = true;
                break;
            }
        }
        if !player_b_wins {
            player_a_wins = true;
            break;
        }
    }

    player_a_wins
}

fn fire_vertex<G: SimpleGraph>(g: &G, divisor: &mut HashMap<VertexID, i32>, v: VertexID) {
    let neighbors = g.neighbors(v);
    for u in neighbors.iter() {
        *divisor.get_mut(&v).unwrap() -= 1;
        *divisor.get_mut(&u).unwrap() += 1;
    }
}

fn dhar<G: SimpleGraph>(g: &G, mut divisor: HashMap<VertexID, i32>, q: VertexID) -> bool {
    while divisor[&q] < 0 {
        let mut burning = HashSet::new();
        burning.insert(q);
        let mut still_burning = true;
        while still_burning {
            still_burning = false;
            for v in g.vertices() {
                if burning.contains(&v) {
                    continue;
                }
                let burning_neighbor_count = g.neighbors(v).iter().filter(|n| burning.contains(n)).count() as i32;
                if burning_neighbor_count > divisor[&v] {
                    burning.insert(v);
                    still_burning = true;
                    break;
                }
            }
        }
        if burning.iter().count() == g.vertex_count() {
            return false;
        }
        for v in g.vertices() {
            if burning.contains(&v) {
                continue;
            }
            fire_vertex(g, &mut divisor, v);
        }
        if divisor[&q] >= 0 {
            break;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dhar_test() {
        let mut g = SparseSimpleGraph::empty();
        g.add_edge((0,1));
        g.add_edge((1,2));
        g.add_edge((2,3));
        g.add_edge((3,4));
        g.add_edge((4,5));
        g.add_edge((5,0));
        
        g.add_edge((0,3));

        let mut losing_strat = HashMap::new();
        losing_strat.insert(0, 0);
        losing_strat.insert(1, 1);
        losing_strat.insert(2, 0);
        losing_strat.insert(3, 0);
        losing_strat.insert(4,-1);
        losing_strat.insert(5, 1);
        assert!(!dhar(&g, losing_strat, 4));

        let mut winning_strat = HashMap::new();
        winning_strat.insert(0, 1);
        winning_strat.insert(1, 0);
        winning_strat.insert(2, 0);
        winning_strat.insert(3, 1);
        winning_strat.insert(4,-1);
        winning_strat.insert(5, 0);
        assert!(dhar(&g, winning_strat, 4));
    }
    
    #[test]
    fn cube_test() {
        let mut cube = SparseSimpleGraph::empty();
        cube.add_edge((0,1));
        cube.add_edge((1,2));        
        cube.add_edge((2,3));        
        cube.add_edge((3,0));        
        
        cube.add_edge((4,5));
        cube.add_edge((5,6));
        cube.add_edge((6,7));
        cube.add_edge((7,4));
        
        cube.add_edge((0,4));
        cube.add_edge((1,5));
        cube.add_edge((2,6));
        cube.add_edge((3,7));
        
        assert!(!gonality_le(&cube, 0));
        assert!(!gonality_le(&cube, 1));
        assert!(!gonality_le(&cube, 2));
        assert!(!gonality_le(&cube, 3));

        assert!(gonality_le(&cube, 4));
        assert!(gonality_le(&cube, 5));
        assert!(gonality_le(&cube, 6));
        assert!(gonality_le(&cube, 7));
        
        assert_eq!(compute_gonality(&cube), Ok(4))
    }
    
    #[test]
    fn tetrahedron_test() {
        let mut tetra = SparseSimpleGraph::empty();
        tetra.add_edge((0,1));
        tetra.add_edge((1,2));
        tetra.add_edge((2,0));
        
        tetra.add_edge((0,3));
        tetra.add_edge((1,3));
        tetra.add_edge((2,3));
        
        assert!(!gonality_le(&tetra, 0));
        assert!(!gonality_le(&tetra, 1));
        assert!(!gonality_le(&tetra, 2));

        assert!(gonality_le(&tetra, 3));
        assert!(gonality_le(&tetra, 4));
        assert!(gonality_le(&tetra, 5));
        
        assert_eq!(compute_gonality(&tetra), Ok(3));
    }
}