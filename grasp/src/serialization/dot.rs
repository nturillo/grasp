use crate::graph::{EdgeID, GraphTrait, VertexID};

/// Format any [crate::graph::GraphTrait] into the DOT format.
///
/// Note: This format does not support labeled data.
pub fn to_dot<G: GraphTrait>(g: G) -> String {
    let mut s = "graph {\n".to_string();
    let mut verts: Vec<VertexID> = g.vertices().collect();
    verts.sort();
    for v in verts {
        s.push_str(&format!("    {};\n", v));
    }
    s.push_str("\n");
    let mut edges: Vec<EdgeID> = g.edges().collect();
    edges.sort();
    for (u, v) in edges {
        s.push_str(&format!("    {} -- {};\n", u, v))
    }
    s.push_str("}");
    s
}

/// Create a [crate::graph::GraphTrait] from a DOT string.
///
/// Note: This format does not support labeled data.
pub fn from_dot<G: GraphTrait + Default>(string: String) -> Result<G, String> {
    let mut graph = G::default();
    let err = Err("Invalid DOT format.".to_string());

    let mut lines = string
        .lines()
        .map(|line| line.trim().replace(" ", "").replace(";", ""));

    match lines.next() {
        None => return err,
        Some(line) => {
            if line != "graph{" {
                return err;
            }
        }
    }

    while let Some(line) = lines.next() {
        if let Ok(n) = line.parse() {
            graph.add_vertex(n);
        } else if let Some((p1, p2)) = line.split_once("--")
            && let (Ok(n1), Ok(n2)) = (p1.parse(), p2.parse())
        {
            graph.add_edge((n1, n2));
        } else if !line.is_empty() && !line.starts_with("//") && line != "}" {
            return err;
        }
    }

    Ok(graph)
}

#[cfg(test)]
mod tests {
    use crate::graph::{GraphTrait, adjacency_list::SparseSimpleGraph};
    use crate::serialization::dot::*;

    #[test]
    fn butterfly_dot() {
        let mut butterfly = SparseSimpleGraph::default();
        butterfly.add_edge((1, 2));
        butterfly.add_edge((2, 3));
        butterfly.add_edge((1, 3));
        butterfly.add_edge((1, 4));
        butterfly.add_edge((1, 5));
        butterfly.add_edge((4, 5));

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

    #[test]
    fn butterfly_from_dot() {
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

        let from = from_dot::<SparseSimpleGraph>(butterfly_dot.to_string());

        pretty_assertions::assert_eq!(from.is_ok(), true);
        pretty_assertions::assert_eq!(butterfly_dot, to_dot(from.expect("Unexpected error")));
    }
}