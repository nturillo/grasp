use crate::graph::{EdgeID, GraphTrait, VertexID};

pub fn to_tgf<G: GraphTrait>(g: G) -> String {
    let mut s = String::new();
    let mut verts: Vec<VertexID> = g.vertices().collect();
    verts.sort();
    for v in verts {
        s.push_str(&format!("{}\n", v));
    }
    s.push_str("#\n");
    let mut edges: Vec<EdgeID> = g.edges().collect();
    edges.sort();
    for (u, v) in edges {
        s.push_str(&format!("{} {}\n", u, v))
    }
    s.pop();
    s
}

pub fn from_tgf<G: GraphTrait + Default>(string: String) -> Result<G, String> {
    let mut lines = string.lines().map(|line| line.trim());
    let mut graph = G::default();

    while let Some(line) = lines.next() {
        if let Ok(n) = line.parse() {
            graph.add_vertex(n);
        } else if let Some((p1, p2)) = line.split_once(" ")
            && let (Ok(n1), Ok(n2)) = (p1.parse(), p2.parse())
        {
            graph.add_edge((n1, n2));
        } else if line != "#" {
            return Err("Invalid TGF format.".to_string());
        }
    }

    Ok(graph)
}

#[cfg(test)]
mod tests {
    use crate::{graph::prelude::SparseSimpleGraph, serialization::tgf::*};

    #[test]
    fn butterfly_from_tgf() {
        let butterfly_tgf = "\
        1\n\
        2\n\
        3\n\
        4\n\
        5\n\
        #\n\
        1 2\n\
        1 3\n\
        1 4\n\
        1 5\n\
        2 3\n\
        4 5";

        let from = from_tgf::<SparseSimpleGraph>(butterfly_tgf.to_string());

        pretty_assertions::assert_eq!(from.is_ok(), true);
        pretty_assertions::assert_eq!(butterfly_tgf, to_tgf(from.expect("Unexpected error")));
    }

    #[test]
    fn butterfly_tgf() {
        let mut butterfly = SparseSimpleGraph::default();
        butterfly.add_edge((1, 2));
        butterfly.add_edge((2, 3));
        butterfly.add_edge((1, 3));
        butterfly.add_edge((1, 4));
        butterfly.add_edge((1, 5));
        butterfly.add_edge((4, 5));

        let butterfly_tgf = "\
        1\n\
        2\n\
        3\n\
        4\n\
        5\n\
        #\n\
        1 2\n\
        1 3\n\
        1 4\n\
        1 5\n\
        2 3\n\
        4 5";

        pretty_assertions::assert_eq!(butterfly_tgf, to_tgf(butterfly));
    }
}