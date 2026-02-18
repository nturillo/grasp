use std::ops::Add;

use crate::graph::adjacency_list::SparseGraph;
use crate::graph::graph_traits::{GraphTrait, VertexType};

#[cfg(feature = "serde")]
use crate::graph::labeled_graph::LabeledGraph;
#[cfg(feature = "serde")]
use serde::Serialize;
#[cfg(feature = "serde")]
use serde_json::Value;

#[cfg(feature = "serde")]
fn serialize<V: Serialize>(data: &V) -> serde_json::Value {
    serde_json::to_value(data).unwrap_or(serde_json::Value::Null)
}

#[cfg(feature = "serde")]
fn wrap_value(json: serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
    use serde_json::Map;

    match json {
        serde_json::Value::Object(map) => map,
        serde_json::Value::Array(vec) => {
            let mut map = Map::new();
            map.insert("list".to_string(), serde_json::Value::Array(vec));
            return map;
        }
        serde_json::Value::Null => Map::new(),
        serde_json::Value::Bool(_) => {
            let mut map = Map::new();
            map.insert("bool".to_string(), json);
            return map;
        }
        serde_json::Value::Number(_) => {
            let mut map = Map::new();
            map.insert("number".to_string(), json);
            return map;
        }
        serde_json::Value::String(_) => {
            let mut map = Map::new();
            map.insert("string".to_string(), json);
            return map;
        }
    }
}

pub fn to_dot<G: GraphTrait>(g: G) -> String {
    let mut s = "graph {\n".to_string();
    let mut verts: Vec<VertexType> = g.vertices().collect();
    verts.sort();
    for v in verts {
        s.push_str(&format!("    {};\n", v));
    }
    s.push_str("\n");
    let mut edges: Vec<(VertexType, VertexType)> = g.edges().collect();
    edges.sort();
    for (u, v) in edges {
        s.push_str(&format!("    {} -- {};\n", u, v))
    }
    s.push_str("}");
    s
}

pub fn from_dot<G: GraphTrait>(string: String) -> Result<G, String> {
    let mut graph = G::new();
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
        } else if !line.is_empty() && !line.starts_with("//") {
            return err;
        }
    }

    Ok(graph)
}

pub fn to_tgf<G: GraphTrait>(g: G) -> String {
    let mut s = String::new();
    let mut verts: Vec<VertexType> = g.vertices().collect();
    verts.sort();
    for v in verts {
        s.push_str(&format!("{}\n", v));
    }
    s.push_str("#\n");
    let mut edges: Vec<(VertexType, VertexType)> = g.edges().collect();
    edges.sort();
    for (u, v) in edges {
        s.push_str(&format!("{} {}\n", u, v))
    }
    s.pop();
    s
}

pub fn from_tgf<G: GraphTrait>(string: String) -> Result<G, String> {
    let mut lines = string.lines().map(|line| line.trim());
    let mut graph = G::new();

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

pub fn to_gml<G: GraphTrait>(g: G) -> String {
    let mut s = "graph [\n".to_string();
    let mut index = 0;

    let mut push =
        |string: String, index: &usize| s.push_str(&"\t".repeat(*index).add(&string).add("\n"));

    index += 1;

    let mut verts: Vec<VertexType> = g.vertices().collect();
    verts.sort();

    for v in verts {
        push("node [".to_string(), &index);
        index += 1;

        push(format!("id {}", v), &index);
        index -= 1;
        push("]".to_string(), &index);
    }

    let mut edges: Vec<(VertexType, VertexType)> = g.edges().collect();
    edges.sort();

    for e in edges {
        push("edge [".to_string(), &index);
        index += 1;

        push(format!("source {}", &e.0), &index);
        push(format!("target {}", &e.1), &index);
        index -= 1;
        push("]".to_string(), &index);
    }
    index -= 1;
    s.push_str(&"]".to_string());
    s
}

pub fn from_gml<G: GraphTrait>(string: String) -> Result<G, String> {
    let mut graph = G::new();
    let err = Err("Invalid GML format.".to_string());
    let mut history = Vec::new();
    let mut edge_builder = (None, None);

    let mut lines = string.lines().map(|line| line.trim());

    match lines.next() {
        None => return err,
        Some(line) => {
            if line != "graph [" {
                return err;
            }

            history.push("graph");
        }
    }

    while let Some(line) = lines.next() {
        if line.is_empty() || line.starts_with("#") || line.starts_with("comment") {
            continue;
        }

        if line == "]" {
            history.pop();
        } else if let Some((p1, p2)) = line.split_once(" ") {
            if p2 == "[" {
                history.push(p1);
                edge_builder = (None, None);
                continue;
            }

            match history.last() {
                None => return err,
                Some(&"node") => {
                    if p1 == "id" {
                        match p2.parse() {
                            Ok(n) => graph.add_vertex(n),
                            Err(_) => return err,
                        }
                    }
                }
                Some(&"edge") => {
                    if p1 == "source" {
                        match p2.parse::<usize>() {
                            Ok(n) => edge_builder.0 = Some(n),
                            Err(_) => return err,
                        }
                    } else if p1 == "target" {
                        match p2.parse::<usize>() {
                            Ok(n) => edge_builder.1 = Some(n),
                            Err(_) => return err,
                        }
                    }

                    if let (Some(source), Some(target)) = edge_builder {
                        graph.add_edge((source, target));
                        edge_builder = (None, None);
                    }
                }
                _ => (),
            };
        }
    }

    Ok(graph)
}

#[cfg(feature = "serde")]
pub fn labeled_to_gml<G, V, E>(g: &LabeledGraph<G, V, E>) -> String
where
    G: GraphTrait,
    V: Serialize,
    E: Serialize,
{
    use serde_json::Map;

    let mut s = "graph [\n".to_string();
    let mut index = 0;

    let push = |s: String, string: String, index: &usize| {
        let mut s = s;
        s.push_str(&"\t".repeat(*index).add(&string).add("\n"));
        s
    };

    fn flatten(s: String, map: Map<String, serde_json::Value>, index: &usize) -> String {
        let mut res = s.clone();

        for entry in map {
            if let serde_json::Value::Object(obj) = entry.1 {
                res.push_str(&"\t".repeat(*index).add(&format!("{} [", entry.0)).add("\n"));
                res = flatten(res, obj, &(index + 1));
                res.push_str(&"\t".repeat(*index).add("]").add("\n"));
            } else if let serde_json::Value::Array(arr) = entry.1 {
                res.push_str(&"\t".repeat(*index).add(&format!("{} [", entry.0)).add("\n"));
                for val in arr {
                    res = flatten(res, wrap_value(val), &(index + 1));
                }
                res.push_str(&"\t".repeat(*index).add("]").add("\n"));
            } else {
                let value: String = match entry.1 {
                    serde_json::Value::Null => "null".to_string(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::String(s) => format!("\"{}\"", s),
                    serde_json::Value::Bool(b) => b.to_string(),
                    _ => continue,
                };

                res.push_str(
                    &"\t"
                        .repeat(*index)
                        .add(&format!("{} {}", entry.0, value))
                        .add("\n"),
                );
            }
        }

        s
    }

    index += 1;

    let mut verts: Vec<VertexType> = g.vertices().collect();
    verts.sort();

    for v in verts {
        use crate::graph::labeled_graph::LabeledGraphTrait;

        s = push(s, "node [".to_string(), &index);
        index += 1;

        s = push(s, format!("id {}", v), &index);
        if let Some(data) = g.get_vertex_label(&v) {
            s = push(s, "data [".to_string(), &index);
            s = flatten(s, wrap_value(serialize(data)), &(index + 1));
            s = push(s, "]".to_string(), &index);
        }

        index -= 1;
        s = push(s, "]".to_string(), &index);
    }

    let mut edges: Vec<(VertexType, VertexType)> = g.edges().collect();
    edges.sort();

    for e in edges {
        use crate::graph::labeled_graph::LabeledGraphTrait;

        s = push(s, "edge [".to_string(), &index);
        index += 1;

        s = push(s, format!("source {}", &e.0), &index);
        s = push(s, format!("target {}", &e.1), &index);
        if let Some(data) = g.get_edge_label(&e) {
            s = push(s, "data [".to_string(), &index);
            s = flatten(s, wrap_value(serialize(data)), &(index + 1));
            s = push(s, "]".to_string(), &index);
        }
        index -= 1;
        s = push(s, "]".to_string(), &index);
    }
    index -= 1;
    s.push_str(&"]".to_string());
    s
}

#[cfg(test)]
use pretty_assertions::{assert_eq, assert_ne};
mod tests {
    use crate::graph::labeled_graph::LabeledGraphTrait;

    use super::*;
    #[test]
    fn butterfly_dot() {
        let mut butterfly = SparseGraph::new();
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
        let mut butterfly = SparseGraph::new();
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

        let from = from_dot::<SparseGraph>(butterfly_dot.to_string());

        pretty_assertions::assert_eq!(from.is_ok(), true);
        pretty_assertions::assert_eq!(butterfly_dot, to_dot(from.expect("Unexpected error")));
    }

    #[test]
    fn butterfly_from_tgf() {
        let mut butterfly = SparseGraph::new();
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

        let from = from_tgf::<SparseGraph>(butterfly_tgf.to_string());

        pretty_assertions::assert_eq!(from.is_ok(), true);
        pretty_assertions::assert_eq!(butterfly_tgf, to_tgf(from.expect("Unexpected error")));
    }

    #[test]
    fn butterfly_tgf() {
        let mut butterfly = SparseGraph::new();
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

    #[test]
    fn butterfly_gml() {
        let mut butterfly = SparseGraph::new();
        butterfly.add_edge((1, 2));
        butterfly.add_edge((2, 3));
        butterfly.add_edge((1, 3));
        butterfly.add_edge((1, 4));
        butterfly.add_edge((1, 5));
        butterfly.add_edge((4, 5));

        let butterfly_gml = "graph [\
            \n\tnode [\
            \n\t\tid 1\
            \n\t]\
            \n\tnode [\
            \n\t\tid 2\
            \n\t]\
            \n\tnode [\
            \n\t\tid 3\
            \n\t]\
            \n\tnode [\
            \n\t\tid 4\
            \n\t]\
            \n\tnode [\
            \n\t\tid 5\
            \n\t]\
            \n\tedge [\
            \n\t\tsource 1\
            \n\t\ttarget 2\
            \n\t]\
            \n\tedge [\
            \n\t\tsource 1\
            \n\t\ttarget 3\
            \n\t]\
            \n\tedge [\
            \n\t\tsource 1\
            \n\t\ttarget 4\
            \n\t]\
            \n\tedge [\
            \n\t\tsource 1\
            \n\t\ttarget 5\
            \n\t]\
            \n\tedge [\
            \n\t\tsource 2\
            \n\t\ttarget 3\
            \n\t]\
            \n\tedge [\
            \n\t\tsource 4\
            \n\t\ttarget 5\
            \n\t]\
            \n]";

        pretty_assertions::assert_eq!(butterfly_gml, to_gml(butterfly));
    }

    #[test]
    fn butterfly_from_gml() {
        let mut butterfly = SparseGraph::new();
        butterfly.add_edge((1, 2));
        butterfly.add_edge((2, 3));
        butterfly.add_edge((1, 3));
        butterfly.add_edge((1, 4));
        butterfly.add_edge((1, 5));
        butterfly.add_edge((4, 5));

        let butterfly_gml = "graph [\
            \n\tnode [\
            \n\t\tid 1\
            \n\t]\
            \n\tnode [\
            \n\t\tid 2\
            \n\t]\
            \n\tnode [\
            \n\t\tid 3\
            \n\t]\
            \n\tnode [\
            \n\t\tid 4\
            \n\t]\
            \n\tnode [\
            \n\t\tid 5\
            \n\t]\
            \n\tedge [\
            \n\t\tsource 1\
            \n\t\ttarget 2\
            \n\t]\
            \n\tedge [\
            \n\t\tsource 1\
            \n\t\ttarget 3\
            \n\t]\
            \n\tedge [\
            \n\t\tsource 1\
            \n\t\ttarget 4\
            \n\t]\
            \n\tedge [\
            \n\t\tsource 1\
            \n\t\ttarget 5\
            \n\t]\
            \n\tedge [\
            \n\t\tsource 2\
            \n\t\ttarget 3\
            \n\t]\
            \n\tedge [\
            \n\t\tsource 4\
            \n\t\ttarget 5\
            \n\t]\
            \n]";

        let from = from_gml::<SparseGraph>(butterfly_gml.to_string());

        pretty_assertions::assert_eq!(from.is_ok(), true);
        pretty_assertions::assert_eq!(butterfly_gml, to_gml(from.expect("Unexpected error")));
    }

    #[test]
    fn labeled_gml() {
        let mut base = SparseGraph::new();
        base.add_edge((1, 2));
        base.add_edge((2, 3));
        //let mut labeled: LabeledGraph<SparseGraph, String, i32> = LabeledGraph::from_graph(base);
    } // TODO
}
