use crate::graph::graph_traits::{GraphTrait, VertexType};
use crate::graph::adjacency_list::SparseGraph;



pub fn to_dot<G: GraphTrait>(g: G) -> String {
    let mut s= "graph {\n".to_string();
    let mut verts: Vec<VertexType> = g.vertices().collect();
    verts.sort();
    for v in verts {
        s.push_str(&format!("    {};\n", v));
    }
    s.push_str("\n");
    let mut edges: Vec<(VertexType, VertexType)> = g.edges().collect();
    edges.sort();
    for (u,v) in edges {
        s.push_str(&format!("    {} -- {};\n", u, v))
    }
    s.push_str("}");
    s
}

#[cfg(test)]
use pretty_assertions::{assert_eq, assert_ne};
mod tests {
    use super::*;
    #[test]
    fn butterfly_dot() {
        let mut butterfly = SparseGraph::new();
        butterfly.add_edge((1,2));
        butterfly.add_edge((2,3));
        butterfly.add_edge((1,3));
        butterfly.add_edge((1,4));
        butterfly.add_edge((1,5));
        butterfly.add_edge((4,5));
        
        let butterfly_dot = "graph {\
        \n    1;\
        \n    2;\
        \n    3;\
        \n    4;\
        \n    5;\
        \n\
        \n    1 -- 2;\
        \n    1 -- 3;\
        \n    1 -- 4;\
        \n    1 -- 5;\
        \n    2 -- 3;\
        \n    4 -- 5;\
        \n}";
       
        pretty_assertions::assert_eq!(butterfly_dot, to_dot(butterfly));
    }
}