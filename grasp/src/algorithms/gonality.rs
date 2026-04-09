use crate::{algorithms::connectivity::is_connected, graph::prelude::*};

use std::collections::{HashMap, HashSet};

pub fn compute_gonality<G>(g: &G) -> Result<usize, GraphError>
where G: SimpleGraph {
    if !is_connected(g) {
        return Err(GraphError::DisconnectedGraph);
    }
    // Placeholder implementation
    Ok(0)
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
}