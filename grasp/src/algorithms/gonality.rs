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

struct Divisor<'a, G: SimpleGraph> {
    g: &'a G,
    chips: HashMap<VertexID, i32>,
}

fn dhar<G: SimpleGraph>(g: &G, divisor: HashMap<VertexID, i32>, q: VertexID) -> bool {
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
                let burning_neighbors = g.neighbors(v).iter().filter(|n| burning.contains(n)).count() as i32;
                if burning_neighbors > divisor[&v] {
                    burning.insert(v);
                    still_burning = true;
                    break;
                }
            }
        }
        
    }

    false
}