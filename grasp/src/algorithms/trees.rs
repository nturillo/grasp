use std::collections::HashMap;
use crate::graph::prelude::*;
use crate::algorithms::algo_traits::Number;

struct DSU {
    parent: HashMap<VertexID, VertexID>,
}

impl DSU {
    fn new(vertices: impl Iterator<Item = VertexID>) -> Self {
        let mut parent = HashMap::new();
        for v in vertices {
            parent.insert(v, v);
        }
        Self { parent }
    }

    fn find(&mut self, v: VertexID) -> VertexID {
        let p = self.parent[&v];
        if p != v {
            let root = self.find(p);
            self.parent.insert(v, root);
        }
        self.parent[&v]
    }

    fn union(&mut self, a: VertexID, b: VertexID) -> bool {
        let root_a = self.find(a);
        let root_b = self.find(b);

        if root_a == root_b {
            return false;
        }

        self.parent.insert(root_a, root_b);
        true
    }
}

pub fn kruskal_mst<G, WF, N>(g: &G, weight: WF) -> Result<Vec<(VertexID, VertexID, N)>, GraphError>
where G: GraphTrait,
WF: Fn(&G, EdgeID) -> Option<N>,
N: Number + PartialOrd + Copy, {
    let mut edges: Vec<(VertexID, VertexID, N)> = Vec::new();

    for (u, v) in g.edges() {
        if let Some(w) = weight(g, (u, v)) {
            edges.push((u, v, w));
        }
    }

    edges.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

    let mut dsu = DSU::new(g.vertices());

    let mut mst = Vec::new();

    for (u, v, w) in edges {
        if dsu.union(u, v) {
            mst.push((u, v, w));
        }
    }

    Ok(mst)
}

#[test]
fn kruskal_basic() {
    let mut graph = SparseSimpleGraph::default();
    graph.add_edge((0, 1));
    graph.add_edge((1, 2));
    graph.add_edge((0, 2));
    graph.add_edge((2, 3));

    let weight = |_g: &SparseSimpleGraph, (u, v): EdgeID| -> Option<i32> {
        match (u.min(v), u.max(v)) {
            (0, 1) => Some(1),
            (1, 2) => Some(2),
            (0, 2) => Some(5),
            (2, 3) => Some(1),
            _ => None,
        }
    };

    let mst = kruskal_mst(&graph, weight).unwrap();

    // total weight should be 1 + 1 + 2 = 4
    let total: i32 = mst.iter().map(|(_, _, w)| *w).sum();
    assert_eq!(total, 4);

    // MST should have |V|-1 edges = 3
    assert_eq!(mst.len(), 3);
}