use std::collections::{HashMap, HashSet};
use crate::graph::prelude::*;
use crate::algorithms::algo_traits::Number;
use graph_ops_macros::register;

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


#[register(
    name = "Kruskal MST Crop",
    desc = "Removes edges from the graph to create a minimum spanning tree.",
    ret = SimpleGraph,
    simple = "true",
    params = []
)]
pub fn kruskal_mst_crop<G: SimpleGraph>(g: &G) -> SparseSimpleGraph {
    let mst = kruskal_mst(g, |_, _| Some(1usize)).unwrap_or_default();

    let mut out = SparseSimpleGraph::with_capacity(g.vertex_count(), mst.len());
    for v in g.vertices() {
        out.add_vertex(v);
    }
    for (u, v, _) in mst {
        out.add_edge((u, v));
    }
    out
}

#[register(
    name = "Kruskal MST Highlight",
    desc = "Highlights the edges in the minimum spanning tree.",
    ret = EdgeList,
    simple = "true",
    params = []
)]
pub fn kruskal_mst_highlight<G: SimpleGraph>(g: &G) -> impl Set<Item = EdgeID> {
    let mst = kruskal_mst(g, |_, _| Some(1usize)).unwrap_or_default();

    mst.into_iter()
        .map(|(u, v, _)| (u.min(v), u.max(v))) // normalize like bridges
        .collect::<HashSet<EdgeID>>()
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