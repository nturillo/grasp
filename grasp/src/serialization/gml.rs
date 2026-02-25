#[cfg(feature = "serde")]
use {
    crate::graph::labeled_graph::LabeledGraph,
    serde::de::DeserializeOwned,
    serde::{Serialize},
    crate::serialization::ser::*,
    std::collections::BTreeMap,
    crate::serialization::error::Error,
    std::slice::from_ref,
};

use crate::graph::{EdgeID, GraphTrait, VertexID};
use std::ops::Add;

struct Parse<'a> {
        file: &'a str,
        index: usize,
        mark: usize,
    }

impl<'a> Parse<'a> {
    fn new(file: &'a str) -> Self {
        Self {
            index: 0,
            mark: 0,
            file: file
        }
    }

    fn peek(&self) -> Option<char> {
        self.file[self.index..].chars().next()
    }

    fn next(&mut self) -> Option<char> {
        let ret = self.peek()?;
        self.index += ret.len_utf8();
        Some(ret)
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(|v| v.is_whitespace()) {
            self.next();
        };
    }

    fn next_word(&mut self) -> Option<&str> {
        self.skip_whitespace();

        if self.peek().is_none() {
            return None;
        }

        let start = self.index;
        while self.peek().is_some_and(|c| !c.is_whitespace()) {
            self.next();
        };

        Some(&self.file[start..self.index])
    }

    fn mark(&mut self) {
        self.mark = self.index;
    }

    fn read_from_mark(&self) -> &str {
        &self.file[self.mark..self.index]
    }
}

/// Format any [crate::graph::GraphTrait] into the GML format.
///
/// Note: This function does not support labeled data. To include labeled data, use [crate::serialization::gml::labeled_to_gml].
pub fn to_gml<G: GraphTrait>(g: G) -> String {
    let mut s = "graph [\n".to_string();
    let mut index = 0;

    let mut push =
        |string: String, index: &usize| s.push_str(&"\t".repeat(*index).add(&string).add("\n"));

    index += 1;

    let mut verts: Vec<VertexID> = g.vertices().collect();
    verts.sort();

    for v in verts {
        push("node [".to_string(), &index);
        index += 1;

        push(format!("id {}", v), &index);
        index -= 1;
        push("]".to_string(), &index);
    }

    let mut edges: Vec<EdgeID> = g.edges().collect();
    edges.sort();

    for e in edges {
        push("edge [".to_string(), &index);
        index += 1;

        push(format!("source {}", &e.0), &index);
        push(format!("target {}", &e.1), &index);
        index -= 1;
        push("]".to_string(), &index);
    }
    s.push_str(&"]".to_string());
    s
}

/// Create a [crate::graph::GraphTrait] from a GML string.
///
/// Note: This function does not support labeled data. To include labeled data, use [crate::serialization::gml::labeled_from_gml].
pub fn from_gml<G: GraphTrait + Default>(string: String) -> Result<G, String> {
    let mut graph = G::default();
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
/// Format any [crate::graph::labeled_graph::LabeledGraph] into the GML format.
///
/// Attached data types must be serializable.
pub fn labeled_to_gml<G: LabeledGraph>(g: &G) -> String
where
    G::VertexData: Serialize,
    G::EdgeData: Serialize,
{
    let mut s = "graph [\n".to_string();
    let mut index = 0;

    let push = |s: String, string: String, index: &usize| {
        let mut s = s;
        s.push_str(&"\t".repeat(*index).add(&string).add("\n"));
        s
    };

    fn flatten(s: String, map: BTreeMap<String, Value>, index: &usize) -> String {
        let mut res = s.clone();

        for entry in map {
            if let Value::Object(obj) = entry.1 {
                res.push_str(&"\t".repeat(*index).add(&format!("{} [", entry.0)).add("\n"));
                res = flatten(res, obj, &(index + 1));
                res.push_str(&"\t".repeat(*index).add("]").add("\n"));
            } else if let Value::Array(arr) = entry.1 {
                res.push_str(&"\t".repeat(*index).add(&format!("{} [", entry.0)).add("\n"));
                for val in arr {
                    res = flatten(res, wrap_value(val), &(index + 1));
                }
                res.push_str(&"\t".repeat(*index).add("]").add("\n"));
            } else {
                let value: String = match entry.1 {
                    Value::Null => "null".to_string(),
                    Value::Int(n) => n.to_string(),
                    Value::Float(n) => n.to_string(),
                    Value::Unsigned(n) => n.to_string(),
                    Value::String(s) => format!("\"{}\"", s),
                    Value::Bool(b) => b.to_string(),
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

        res
    }

    index += 1;

    let mut verts: Vec<VertexID> = g.vertices().collect();
    verts.sort();

    for v in verts {
        s = push(s, "node [".to_string(), &index);
        index += 1;

        s = push(s, format!("id {}", v), &index);
        if let Some(data) = g.get_vertex_label(v) {
            let val = serialize(data);

            if let Value::Object(_) = val {
                s = push(s, "data [".to_string(), &index);
                s = flatten(s, wrap_value(serialize(data)), &(index + 1));
                s = push(s, "]".to_string(), &index);
            } else {
                s = flatten(s, wrap_value(serialize(data)), &index);
            }
        }

        index -= 1;
        s = push(s, "]".to_string(), &index);
    }

    let mut edges: Vec<EdgeID> = g.edges().collect();
    edges.sort();

    for e in edges {
        s = push(s, "edge [".to_string(), &index);
        index += 1;

        s = push(s, format!("source {}", &e.0), &index);
        s = push(s, format!("target {}", &e.1), &index);
        if let Some(data) = g.get_edge_label(e) {
            let val = serialize(data);

            if let Value::Object(_) = val {
                s = push(s, "data [".to_string(), &index);
                s = flatten(s, wrap_value(serialize(data)), &(index + 1));
                s = push(s, "]".to_string(), &index);
            } else {
                s = flatten(s, wrap_value(serialize(data)), &index);
            }
        }
        index -= 1;
        s = push(s, "]".to_string(), &index);
    }
    s.push_str(&"]".to_string());
    s
}

#[cfg(feature = "serde")]
/// Create a [crate::graph::labeled_graph::LabeledGraph] from a GML string.
///
/// Chosen data types must be deserializable.
pub fn labeled_from_gml<G: LabeledGraph + Default>(string: String) -> Result<G, String>
where
    G::VertexData: DeserializeOwned,
    G::EdgeData: DeserializeOwned,
{
    let mut graph = G::default();
    let err = Err("Invalid GML format.".to_string());

    fn parse_map(parser: &mut Parse) -> Result<Value, Error> {
        let mut map = BTreeMap::<String, Value>::new();

        while let Some(word) = parser.next_word() {
            if word == "]" {
                return Ok(Value::Object(map));
            }

            if word == "#" || word == "comment" {
                continue;
            }

            let key = word.to_string();
            let value = parse_val(parser)?;

            if map.contains_key(key.as_str()) {
                let val = map.get_mut(key.as_str()).expect("Validated containment beforehand");

                if let Value::Array(arr) = val {
                    arr.push(value.clone());
                } else {
                    *val = Value::Array(vec![val.clone(), value.clone()]);
                }
            } else {
                map.insert(key, value);
            }
        }

        Err(Error::Message("Invalid GML format".to_string()))
    }

    fn parse_val(parser: &mut Parse) -> Result<Value, Error> {
        parser.skip_whitespace();
        if let Some(ch) = parser.peek() {
            if ch == '[' {
                parser.next();
                return parse_map(parser);
            }

            if ch == '\"' {
                parser.next();
                parser.mark();
                while parser.peek().ok_or(Error::Message("String not closed".to_string()))? != '\"' {
                    parser.next();
                }
                let ret = parser.read_from_mark().to_string();
                parser.next();
                return Ok(Value::String(ret));
            }

            let word = parser.next_word().ok_or(Error::Message("Expected value, found none".to_string()))?;

            if word == "true" { return Ok(Value::Bool(true)); }
            if word == "false" { return Ok(Value::Bool(false)); }

            if let Ok(num) = word.parse::<u64>() { return Ok(Value::Unsigned(num)); }
            if let Ok(num) = word.parse::<i64>() { return Ok(Value::Int(num)); }
            if let Ok(num) = word.parse::<f64>() { return Ok(Value::Float(num)); }

            return Err(Error::Message("Unknown value type: ".to_string().add(word)));
        }

        Err(Error::Message("Missing value after key".to_string()))
    }

    let parser = &mut Parse::new(string.as_str());
    if parser.next_word() == Some("graph") && parser.next_word() == Some("[") && let Value::Object(map) = parse_map(parser).map_err(|er| er.to_string())? {
        if let Some(val) = map.get("node") {
            let mut nodes: &[Value] = &[];

            if let Value::Object(_) = val {
                nodes = from_ref(val)
            } else if let Value::Array(arr) = val {
                nodes = arr.as_slice();
            }

            for node in nodes {
                if let Value::Object(node_map) = node {
                    let id = match node_map.get("id") {
                        Some(Value::Unsigned(num)) => Ok(*num),
                        _ => Err("Missing or invalid ID".to_string()),
                    }? as usize;

                    graph.add_vertex(id);

                    if let Some(data) = node_map.get("data").or(node_map.get("label")) {
                        graph.set_vertex_label(id, from_value::<G::VertexData>(data.clone()).map_err(|er| er.to_string())?);
                    }
                } else {
                    return err;
                }
            }
        }

        if let Some(val) = map.get("edge") {
            let mut edges: &[Value] = &[];

            if let Value::Object(_) = val {
                edges = from_ref(val)
            } else if let Value::Array(arr) = val {
                edges = arr.as_slice();
            }

            for edge in edges {
                if let Value::Object(edge_map) = edge {
                    let source = match edge_map.get("source") {
                        Some(Value::Unsigned(num)) => Ok(*num),
                        _ => Err("Missing or invalid source".to_string()),
                    }? as usize;

                    let target = match edge_map.get("target") {
                        Some(Value::Unsigned(num)) => Ok(*num),
                        _ => Err("Missing or invalid target".to_string()),
                    }? as usize;

                    let edge_id = (source, target);
                    graph.add_edge(edge_id);

                    if let Some(data) = edge_map.get("data").or(edge_map.get("label")) {
                        graph.set_edge_label(edge_id, from_value::<G::EdgeData>(data.clone()).map_err(|er| er.to_string())?);
                    }
                } else {
                    return err;
                }
            }
        }
    } else {
        return err;
    }

    Ok(graph)
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use crate::{graph::{GraphTrait, prelude::{HashMapLabeledGraph, SparseSimpleGraph}}, serialization::gml::{from_gml, labeled_from_gml, labeled_to_gml, to_gml}};

    #[test]
    fn butterfly_gml() {
        let mut butterfly = SparseSimpleGraph::default();
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

        let from = from_gml::<SparseSimpleGraph>(butterfly_gml.to_string());

        pretty_assertions::assert_eq!(from.is_ok(), true);
        pretty_assertions::assert_eq!(butterfly_gml, to_gml(from.expect("Unexpected error")));
    }

    #[test]
    fn to_labeled_gml() {
        #[derive(Clone, Serialize)]
        struct VData {
            size: i32,
            color: &'static str,
        }

        let mut base = HashMapLabeledGraph::<SparseSimpleGraph, VData, i32>::default();
        base.add_edge((1, 2));
        base.add_edge((2, 3));
        base.add_edge((2, 4));

        base.edge_labels.insert((1, 2), 12);
        base.edge_labels.insert((2, 3), 2);
        base.edge_labels.insert((2, 4), 7);

        base.vertex_labels.insert(
            1,
            VData {
                size: 0,
                color: "Red",
            },
        );
        base.vertex_labels.insert(
            2,
            VData {
                size: 3,
                color: "Green",
            },
        );
        base.vertex_labels.insert(
            3,
            VData {
                size: -2,
                color: "Blue",
            },
        );
        base.vertex_labels.insert(
            4,
            VData {
                size: 64,
                color: "Yellow",
            },
        );

        let s = "graph [\n\
                    \tnode [\n\
                    \t\tid 1\n\
                    \t\tdata [\n\
                    \t\t\tcolor \"Red\"\n\
                    \t\t\tsize 0\n\
                    \t\t]\n\
                    \t]\n\
                    \tnode [\n\
                    \t\tid 2\n\
                    \t\tdata [\n\
                    \t\t\tcolor \"Green\"\n\
                    \t\t\tsize 3\n\
                    \t\t]\n\
                    \t]\n\
                    \tnode [\n\
                    \t\tid 3\n\
                    \t\tdata [\n\
                    \t\t\tcolor \"Blue\"\n\
                    \t\t\tsize -2\n\
                    \t\t]\n\
                    \t]\n\
                    \tnode [\n\
                    \t\tid 4\n\
                    \t\tdata [\n\
                    \t\t\tcolor \"Yellow\"\n\
                    \t\t\tsize 64\n\
                    \t\t]\n\
                    \t]\n\
                    \tedge [\n\
                    \t\tsource 1\n\
                    \t\ttarget 2\n\
                    \t\tdata 12\n\
                    \t]\n\
                    \tedge [\n\
                    \t\tsource 2\n\
                    \t\ttarget 3\n\
                    \t\tdata 2\n\
                    \t]\n\
                    \tedge [\n\
                    \t\tsource 2\n\
                    \t\ttarget 4\n\
                    \t\tdata 7\n\
                    \t]\n\
                    ]";

        pretty_assertions::assert_eq!(s, labeled_to_gml(&base));
    }

    #[test]
    fn from_labeled_gml() {
        #[derive(Clone, Serialize, Deserialize)]
        struct VData {
            size: i32,
            color: String,
        }

        let s = "graph [\n\
                    \tnode [\n\
                    \t\tid 1\n\
                    \t\tdata [\n\
                    \t\t\tcolor \"Red\"\n\
                    \t\t\tsize 0\n\
                    \t\t]\n\
                    \t]\n\
                    \tnode [\n\
                    \t\tid 2\n\
                    \t\tdata [\n\
                    \t\t\tcolor \"Green\"\n\
                    \t\t\tsize 3\n\
                    \t\t]\n\
                    \t]\n\
                    \tnode [\n\
                    \t\tid 3\n\
                    \t\tdata [\n\
                    \t\t\tcolor \"Blue\"\n\
                    \t\t\tsize -2\n\
                    \t\t]\n\
                    \t]\n\
                    \tnode [\n\
                    \t\tid 4\n\
                    \t\tdata [\n\
                    \t\t\tcolor \"Yellow\"\n\
                    \t\t\tsize 64\n\
                    \t\t]\n\
                    \t]\n\
                    \tedge [\n\
                    \t\tsource 1\n\
                    \t\ttarget 2\n\
                    \t\tdata 12\n\
                    \t]\n\
                    \tedge [\n\
                    \t\tsource 2\n\
                    \t\ttarget 3\n\
                    \t\tdata 2\n\
                    \t]\n\
                    \tedge [\n\
                    \t\tsource 2\n\
                    \t\ttarget 4\n\
                    \t\tdata 7\n\
                    \t]\n\
                    ]";

        let from = labeled_from_gml::<HashMapLabeledGraph<SparseSimpleGraph, VData, i32>>(s.to_string());

        pretty_assertions::assert_eq!(from.is_ok(), true);
        pretty_assertions::assert_eq!(s, labeled_to_gml(&from.expect("Unexpected error")));
    }
}