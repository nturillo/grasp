use crate::graph::GraphTrait;
use crate::graph::{DiGraph, SimpleGraph};
use crate::graph::{EdgeID, VertexID};

#[cfg(feature = "xml")]
use {
    std::io::Cursor,
    quick_xml::Writer,
    quick_xml::Reader,
    quick_xml::events::{BytesText, BytesDecl, Event, BytesStart},
};

#[cfg(feature = "serde")]
use {
    crate::graph::labeled_graph::LabeledGraph,
    serde::de::DeserializeOwned,
    serde::{Serialize},
    crate::serialization::ser::*,
    std::collections::BTreeMap,
};

#[cfg(feature = "xml")]
fn to_graphml_helper<G: GraphTrait>(graph: G, edge_type: &str) -> String {
    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);

    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None))).expect("Unexpected error in GraphML setup");
    writer.create_element("graphml").with_attributes([("xmlns", "http://graphml.graphdrawing.org/xmlns"), ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"), ("xsi:schemaLocation", "http://graphml.graphdrawing.org/xmlns/1.0/graphml.xsd")]).write_inner_content(|writer| {
        writer.create_element("graph").with_attribute(("edgedefault", edge_type)).write_inner_content(|writer| {
            let mut verts: Vec<VertexID> = graph.vertices().collect();
            verts.sort();
            for node in verts {
                writer.create_element("node").with_attribute(("id", node.to_string().as_str())).write_empty()?;
            }

            let mut edges: Vec<EdgeID> = graph.edges().collect();
            edges.sort();
            for (source, target) in edges {
                writer.create_element("edge").with_attributes([("source", source.to_string().as_str()), ("target", target.to_string().as_str())]).write_empty()?;
            }

            Ok(())
        })?;

        Ok(())
    }).expect("Unexpected error while creating GraphML string");

    String::from_utf8(writer.into_inner().into_inner()).expect("Unexpected error while converting XML into string")
}

#[cfg(feature = "xml")]
/// Format any [crate::graph::SimpleGraph] into the GraphML format.
///
/// Note: This function does not support labeled data. To include labeled data, use [crate::serialization::graphml::labeled_to_graphml_simple].
pub fn to_graphml_simple<G: SimpleGraph>(graph: G) -> String {
    to_graphml_helper(graph, "undirected")
}

#[cfg(feature = "xml")]
/// Format any [crate::graph::DiGraph] into the GraphML format.
///
/// Note: This function does not support labeled data. To include labeled data, use [crate::serialization::graphml::labeled_to_graphml_digraph].
pub fn to_graphml_digraph<G: DiGraph>(graph: G) -> String {
    to_graphml_helper(graph, "directed")
}

#[cfg(feature = "xml")]
/// Create a [crate::graph::GraphTrait] from a GraphML string.
///
/// Note: This format does not support labeled data. To include labeled data, use [crate::serialization::graphml::labeled_from_graphml].
pub fn from_graphml<G: GraphTrait + Default>(string: String) -> Result<G, String> {
    let mut buf = Vec::new();
    let mut reader = Reader::from_str(string.as_str());
    reader.config_mut().trim_text(true);

    let mut graph = G::default();

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => return Err(e.to_string()),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                match e.name().as_ref() {
                    b"node" => {
                        if let Some(id) = e.try_get_attribute("id").map_err(|e| e.to_string())? {
                            graph.add_vertex(id.unescape_value().unwrap().to_owned().parse::<usize>().map_err(|_| "Expected ID to be unsigned int".to_string())?);
                        } else {
                            return Err("Node missing id attribute".to_string());
                        }
                    },
                    b"edge" => {
                        let source = e.try_get_attribute("source").map_err(|e| e.to_string())?.and_then(|a| Some(a.unescape_value().unwrap().into_owned()));
                        let target = e.try_get_attribute("target").map_err(|e| e.to_string())?.and_then(|a| Some(a.unescape_value().unwrap().into_owned()));

                        if let (Some(source), Some(target)) = (source, target) {
                            graph.add_edge((source.parse::<usize>().map_err(|_| "Expected source to be unsigned int".to_string())?, target.parse::<usize>().map_err(|_| "Expected target to be unsigned int".to_string())?));
                        } else {
                            return Err("Edge missing source/target attributes".to_string());
                        }
                    },
                    _ => (),
                }
            },
            _ => (),
        }
        buf.clear();
    }

    Ok(graph)
}

#[cfg(all(feature = "xml", feature = "serde"))]
fn labeled_to_graphml_helper<G: LabeledGraph>(graph: G, edge_type: &str) -> String
where
    G::VertexData: Serialize,
    G::EdgeData: Serialize,
{
    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);

    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None))).expect("Unexpected error in GraphML setup");
    writer.create_element("graphml").with_attributes([("xmlns", "http://graphml.graphdrawing.org/xmlns"), ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"), ("xsi:schemaLocation", "http://graphml.graphdrawing.org/xmlns/1.0/graphml.xsd")]).write_inner_content(|writer| {
        let node_map = if graph.vertex_count() != 0 && let Some(v_data) = graph.get_vertex_label(graph.vertices().next().expect("Unexpected error: list is not empty")) {
            let mut flat = BTreeMap::new();
            let val = to_value(v_data).expect("Unexpected error: is serializable");
            get_flat_map(if val.is_primative() {"0N".to_string()} else {String::new()}, val, &mut flat);

            for pair in &flat {
                writer.create_element("key").with_attributes([("id", pair.0.as_str()), ("for", "node"), ("attr.name", pair.0.as_str()), ("attr.type", pair.1.1.as_str())]).write_empty()?;
            }

            Some(flat)
        } else {None};

        let edge_map = if graph.edge_count() != 0 && let Some(e_data) = graph.get_edge_label(graph.edges().next().expect("Unexpected error: list is not empty")) {
            let mut flat = BTreeMap::new();
            let val = to_value(e_data).expect("Unexpected error: is serializable");
            get_flat_map(if val.is_primative() {"0E".to_string()} else {String::new()}, val, &mut flat);

            for pair in &flat {
                writer.create_element("key").with_attributes([("id", pair.0.as_str()), ("for", "edge"), ("attr.name", pair.0.as_str()), ("attr.type", pair.1.1.as_str())]).write_empty()?;
            }

            Some(flat)
        } else {None};

        writer.create_element("graph").with_attribute(("edgedefault", edge_type)).write_inner_content(|writer| {
            let mut verts: Vec<VertexID> = graph.vertices().collect();
            verts.sort();
            for node in verts {
                writer.create_element("node").with_attribute(("id", node.to_string().as_str())).write_inner_content(|writer| {
                    if let Some(v_data) = graph.get_vertex_label(node) && let Some(map) = node_map.as_ref() {
                        let mut data_map = BTreeMap::new();
                        let val = to_value(v_data).expect("Unexpected error: is serializable");
                        get_flat_map(if val.is_primative() {"0N".to_string()} else {String::new()}, val, &mut data_map);

                        for (key, _) in map {
                            writer.create_element("data").with_attribute(("key", key.as_str())).write_text_content(BytesText::new(data_map.get(key.as_str()).expect("Unexpected error: mismatched types").0.as_str()))?;
                        }
                    }

                    Ok(())
                })?;
            }

            let mut edges: Vec<EdgeID> = graph.edges().collect();
            edges.sort();
            for edge in edges {
                writer.create_element("edge").with_attributes([("source", edge.0.to_string().as_str()), ("target", edge.1.to_string().as_str())]).write_inner_content(|writer| {
                    if let Some(e_data) = graph.get_edge_label(edge) && let Some(map) = edge_map.as_ref() {
                        let mut data_map = BTreeMap::new();
                        let val = to_value(e_data).expect("Unexpected error: is serializable");
                        get_flat_map(if val.is_primative() {"0E".to_string()} else {String::new()}, val, &mut data_map);

                        for (key, _) in map {
                            writer.create_element("data").with_attribute(("key", key.as_str())).write_text_content(BytesText::new(data_map.get(key.as_str()).expect("Unexpected error: mismatched types").0.as_str()))?;
                        }
                    }

                    Ok(())
                })?;
            }

            Ok(())
        })?;

        Ok(())
    }).expect("Unexpected error while creating GraphML string");

    String::from_utf8(writer.into_inner().into_inner()).expect("Unexpected error while converting XML into string")
}

#[cfg(all(feature = "xml", feature = "serde"))]
/// Format any undirected [crate::graph::labeled_graph::LabeledGraph] into the GraphML format.
///
/// Attached data types must be serializable.
pub fn labeled_to_graphml_simple<G: LabeledGraph>(graph: G) -> String
where G::GraphType: SimpleGraph,
    G::VertexData: Serialize,
    G::EdgeData: Serialize,
{
    labeled_to_graphml_helper(graph, "undirected")
}

#[cfg(all(feature = "xml", feature = "serde"))]
/// Format any directed [crate::graph::labeled_graph::LabeledGraph] into the GraphML format.
///
/// Attached data types must be serializable.
pub fn labeled_to_graphml_digraph<G: LabeledGraph>(graph: G) -> String
where G::GraphType: DiGraph,
    G::VertexData: Serialize,
    G::EdgeData: Serialize,
{
    labeled_to_graphml_helper(graph, "directed")
}

#[cfg(all(feature = "xml", feature = "serde"))]
/// Create a [crate::graph::labeled_graph::LabeledGraph] from a GraphML string.
///
/// Chosen data types must be deserializable.
pub fn labeled_from_graphml<G: LabeledGraph + Default>(string: String) -> Result<G, String>
where
    G::VertexData: DeserializeOwned,
    G::EdgeData: DeserializeOwned,
{
    let mut buf = Vec::new();
    let mut reader = Reader::from_str(string.as_str());
    reader.config_mut().trim_text(true);

    let mut graph = G::default();

    let mut node_map = BTreeMap::new();
    let mut edge_map = BTreeMap::new();
    let mut data_map = BTreeMap::new();

    let mut parent_id = (0, 0);

    fn build_map(nests: &mut dyn Iterator<Item = String>, value: &mut Value, data: Value) {
        if let Some(key) = nests.next() {
            if let Ok(_) = key.parse::<i64>() {
                let mut new_value = Value::Object(BTreeMap::new());
                build_map(nests, &mut new_value, data);
                if let Value::Array(arr) = value {
                    arr.push(new_value);
                } else {
                    *value = Value::Array(vec![new_value]);
                }
            } else if let Value::Object(map) = value {
                let elem = map.entry(key).or_insert(Value::Object(BTreeMap::new()));
                build_map(nests, elem, data);
            }
        } else {
            *value = data;
        }
    }

    fn get_value<'a>(root: &'a mut BTreeMap<String, Value>, full_key: String) -> Result<&'a mut Value, String> {
        let mut nests = full_key.split(".");
        let mut value = root.get_mut(nests.next().ok_or("Key is empty".to_string())?).ok_or("Could not find key".to_string())?;

        for nest in nests {
            if let Ok(num) = nest.to_string().parse::<usize>() {
                if let Value::Array(arr) = value {
                    value = arr.get_mut(num).ok_or("Could not find index".to_string())?;
                } else {
                    return Err("Cannot parse map object with an index".to_string());
                }
            } else if let Value::Object(obj) = value {
                value = obj.get_mut(nest).ok_or("Could not find key".to_string())?;
            } else {
                return Err("Cannot parse primitive types as a map".to_string());
            }
        }

        Ok(value)
    }

    fn get_attr(e: &BytesStart, attr: &str) -> Result<String, String> {
        e.try_get_attribute(attr).map_err(|e| e.to_string())?.ok_or_else(|| format!("Missing attribute {}", attr))?.unescape_value().map(|a| a.into_owned()).map_err(|e| e.to_string())
    }

    fn get_attr_as_usize(e: &BytesStart, attr: &str) -> Result<usize, String> {
        get_attr(e, attr)?.parse::<usize>().map_err(|_| format!("Expected {} to be unsigned int", attr))
    }

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => return Err(e.to_string()),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                match e.name().as_ref() {
                    b"node" => {
                        data_map = node_map.clone();
                        let id = get_attr_as_usize(&e, "id")?;
                        graph.add_vertex(id);
                        parent_id = (id, id);
                    },
                    b"edge" => {
                        data_map = edge_map.clone();
                        let source = get_attr_as_usize(&e, "source")?;
                        let target = get_attr_as_usize(&e, "target")?;

                        graph.add_edge((source, target));
                        parent_id = (source, target);
                    },
                    b"data" => {
                        if data_map.is_empty() {
                            continue;
                        }

                        let key = get_attr(&e, "key")?;
                        let val = get_value(&mut data_map, key)?;

                        if let Value::String(data_type) = val {
                            let data = reader.read_text(e.name()).map_err(|e| e.to_string())?.to_owned().to_string();

                            *val = match data_type.as_str() {
                                "boolean" => Value::Bool(data.parse::<bool>().map_err(|e| e.to_string())?),
                                "string" => Value::String(data),
                                "long" => Value::Int(data.parse::<i64>().map_err(|e| e.to_string())?),
                                "double" => Value::Float(data.parse::<f64>().map_err(|e| e.to_string())?),
                                _ => Err("Unknown type")?,
                            };
                        }
                    }
                    _ => (),
                }
            },
            Ok(Event::Empty(e)) => {
                match e.name().as_ref() {
                    b"node" => {
                        let id = get_attr_as_usize(&e, "id")?;
                        graph.add_vertex(id);
                        parent_id = (id, id);
                    },
                    b"edge" => {
                        let source = get_attr_as_usize(&e, "source")?;
                        let target = get_attr_as_usize(&e, "target")?;

                        graph.add_edge((source, target));
                        parent_id = (source, target);
                    },
                    b"key" => {
                        let key = get_attr(&e, "id")?;
                        let key_type = get_attr(&e, "attr.type").unwrap_or("string".to_string());
                        let key_for = get_attr(&e, "for")?;

                        let map = if key_for.as_str() == "node" {&mut node_map} else {&mut edge_map};
                        let nests = &mut key.split(".").map(String::from);

                        if let Some(first) = nests.next() {
                            let elem = map.entry(first).or_insert(Value::Object(BTreeMap::new()));
                            build_map(nests, elem, Value::String(key_type));
                        }
                    },
                    _ => (),
                }
            }
            Ok(Event::End(e)) => {
                match e.name().as_ref() {
                    b"node" => {
                        let mut done = std::mem::take(&mut data_map);
                        let val = if done.contains_key("0N") { done.remove("0N").unwrap_or(Value::Null) } else { Value::Object(done) };

                        graph.set_vertex_label(parent_id.0, from_value::<G::VertexData>(val).map_err(|e| e.to_string())?);
                    },
                    b"edge" => {
                        let mut done = std::mem::take(&mut data_map);
                        let val = if done.contains_key("0E") { done.remove("0E").unwrap_or(Value::Null) } else { Value::Object(done) };

                        graph.set_edge_label(parent_id, from_value::<G::EdgeData>(val).map_err(|e| e.to_string())?);
                    }
                    _ => (),
                }
            }
            _ => (),
        }
        buf.clear();
    }

    Ok(graph)
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use crate::graph::prelude::SparseDiGraph;
    use crate::serialization::graphml::*;
    use crate::{graph::prelude::HashMapLabeledGraph};
    use crate::graph::{GraphTrait, adjacency_list::SparseSimpleGraph};

    #[test]
    fn simple_graphml() {
        let mut butterfly = SparseSimpleGraph::default();
        butterfly.add_edge((1, 2));
        butterfly.add_edge((2, 3));
        butterfly.add_edge((1, 3));
        butterfly.add_edge((1, 4));
        butterfly.add_edge((1, 5));
        butterfly.add_edge((4, 5));

        let s = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<graphml xmlns=\"http://graphml.graphdrawing.org/xmlns\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xsi:schemaLocation=\"http://graphml.graphdrawing.org/xmlns/1.0/graphml.xsd\">\n  \
        <graph edgedefault=\"undirected\">\n    \
        <node id=\"1\"/>\n    \
        <node id=\"2\"/>\n    \
        <node id=\"3\"/>\n    \
        <node id=\"4\"/>\n    \
        <node id=\"5\"/>\n    \
        <edge source=\"1\" target=\"2\"/>\n    \
        <edge source=\"1\" target=\"3\"/>\n    \
        <edge source=\"1\" target=\"4\"/>\n    \
        <edge source=\"1\" target=\"5\"/>\n    \
        <edge source=\"2\" target=\"3\"/>\n    \
        <edge source=\"4\" target=\"5\"/>\n  \
        </graph>\n</graphml>";

        pretty_assertions::assert_eq!(s, to_graphml_simple(butterfly));
    }

    #[test]
    fn digraph_graphml() {
        let mut butterfly = SparseDiGraph::default();
        butterfly.add_edge((1, 2));
        butterfly.add_edge((2, 3));
        butterfly.add_edge((1, 3));
        butterfly.add_edge((1, 4));
        butterfly.add_edge((1, 5));
        butterfly.add_edge((4, 5));

        let s = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<graphml xmlns=\"http://graphml.graphdrawing.org/xmlns\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xsi:schemaLocation=\"http://graphml.graphdrawing.org/xmlns/1.0/graphml.xsd\">\n  \
        <graph edgedefault=\"directed\">\n    \
        <node id=\"1\"/>\n    \
        <node id=\"2\"/>\n    \
        <node id=\"3\"/>\n    \
        <node id=\"4\"/>\n    \
        <node id=\"5\"/>\n    \
        <edge source=\"1\" target=\"2\"/>\n    \
        <edge source=\"1\" target=\"3\"/>\n    \
        <edge source=\"1\" target=\"4\"/>\n    \
        <edge source=\"1\" target=\"5\"/>\n    \
        <edge source=\"2\" target=\"3\"/>\n    \
        <edge source=\"4\" target=\"5\"/>\n  \
        </graph>\n</graphml>";

        pretty_assertions::assert_eq!(s, to_graphml_digraph(butterfly));
    }

    #[test]
    fn test_from_graphml() {
        let s = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<graphml xmlns=\"http://graphml.graphdrawing.org/xmlns\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xsi:schemaLocation=\"http://graphml.graphdrawing.org/xmlns/1.0/graphml.xsd\">\n  \
        <graph edgedefault=\"undirected\">\n    \
        <node id=\"1\"/>\n    \
        <node id=\"2\"/>\n    \
        <node id=\"3\"/>\n    \
        <node id=\"4\"/>\n    \
        <node id=\"5\"/>\n    \
        <edge source=\"1\" target=\"2\"/>\n    \
        <edge source=\"1\" target=\"3\"/>\n    \
        <edge source=\"1\" target=\"4\"/>\n    \
        <edge source=\"1\" target=\"5\"/>\n    \
        <edge source=\"2\" target=\"3\"/>\n    \
        <edge source=\"4\" target=\"5\"/>\n  \
        </graph>\n</graphml>";

        let from = from_graphml::<SparseSimpleGraph>(s.to_string());

        pretty_assertions::assert_eq!(true, from.is_ok());
        pretty_assertions::assert_eq!(s, to_graphml_simple(from.expect("preverified")));
    }

    #[test]
    fn to_labeled_graphml() {
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

        let s = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<graphml xmlns=\"http://graphml.graphdrawing.org/xmlns\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xsi:schemaLocation=\"http://graphml.graphdrawing.org/xmlns/1.0/graphml.xsd\">\n  <key id=\"color\" for=\"node\" attr.name=\"color\" attr.type=\"string\"/>\n  <key id=\"size\" for=\"node\" attr.name=\"size\" attr.type=\"long\"/>\n  <key id=\"0E\" for=\"edge\" attr.name=\"0E\" attr.type=\"long\"/>\n  <graph edgedefault=\"undirected\">\n    <node id=\"1\">\n      <data key=\"color\">Red</data>\n      <data key=\"size\">0</data>\n    </node>\n    <node id=\"2\">\n      <data key=\"color\">Green</data>\n      <data key=\"size\">3</data>\n    </node>\n    <node id=\"3\">\n      <data key=\"color\">Blue</data>\n      <data key=\"size\">-2</data>\n    </node>\n    <node id=\"4\">\n      <data key=\"color\">Yellow</data>\n      <data key=\"size\">64</data>\n    </node>\n    <edge source=\"1\" target=\"2\">\n      <data key=\"0E\">12</data>\n    </edge>\n    <edge source=\"2\" target=\"3\">\n      <data key=\"0E\">2</data>\n    </edge>\n    <edge source=\"2\" target=\"4\">\n      <data key=\"0E\">7</data>\n    </edge>\n  </graph>\n</graphml>";
        pretty_assertions::assert_eq!(s, labeled_to_graphml_simple(base));
    }

    #[test]
    fn to_nested_graphml() {
        #[derive(Clone, Serialize)]
        struct AData {
            size: i32,
            color: &'static str,
        }

        #[derive(Clone, Serialize)]
        struct BData {
            shape: String,
            adata: AData,
        }

        let mut base = HashMapLabeledGraph::<SparseSimpleGraph, BData, i32>::default();
        base.add_edge((1, 2));

        base.edge_labels.insert((1, 2), 12);

        base.vertex_labels.insert(
            1,
            BData {
                shape: "round".to_string(),
                adata: AData {
                    size: 0,
                    color: "Red",
                }
            },
        );
        base.vertex_labels.insert(
            2,
            BData {
                shape: "square".to_string(),
                adata: AData {
                    size: 3,
                    color: "Green",
                }
            },
        );

        let s = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<graphml xmlns=\"http://graphml.graphdrawing.org/xmlns\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xsi:schemaLocation=\"http://graphml.graphdrawing.org/xmlns/1.0/graphml.xsd\">\n  <key id=\"adata.color\" for=\"node\" attr.name=\"adata.color\" attr.type=\"string\"/>\n  <key id=\"adata.size\" for=\"node\" attr.name=\"adata.size\" attr.type=\"long\"/>\n  <key id=\"shape\" for=\"node\" attr.name=\"shape\" attr.type=\"string\"/>\n  <key id=\"0E\" for=\"edge\" attr.name=\"0E\" attr.type=\"long\"/>\n  <graph edgedefault=\"undirected\">\n    <node id=\"1\">\n      <data key=\"adata.color\">Red</data>\n      <data key=\"adata.size\">0</data>\n      <data key=\"shape\">round</data>\n    </node>\n    <node id=\"2\">\n      <data key=\"adata.color\">Green</data>\n      <data key=\"adata.size\">3</data>\n      <data key=\"shape\">square</data>\n    </node>\n    <edge source=\"1\" target=\"2\">\n      <data key=\"0E\">12</data>\n    </edge>\n  </graph>\n</graphml>";
        pretty_assertions::assert_eq!(s, labeled_to_graphml_simple(base));
    }

    #[test]
    fn from_nested_graphml() {
        #[derive(Clone, Serialize, Deserialize)]
        struct AData {
            size: i32,
            color: String,
        }

        #[derive(Clone, Serialize, Deserialize)]
        struct BData {
            shape: String,
            adata: AData,
        }

        let s = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<graphml xmlns=\"http://graphml.graphdrawing.org/xmlns\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xsi:schemaLocation=\"http://graphml.graphdrawing.org/xmlns/1.0/graphml.xsd\">\n  <key id=\"adata.color\" for=\"node\" attr.name=\"adata.color\" attr.type=\"string\"/>\n  <key id=\"adata.size\" for=\"node\" attr.name=\"adata.size\" attr.type=\"long\"/>\n  <key id=\"shape\" for=\"node\" attr.name=\"shape\" attr.type=\"string\"/>\n  <key id=\"0E\" for=\"edge\" attr.name=\"0E\" attr.type=\"long\"/>\n  <graph edgedefault=\"undirected\">\n    <node id=\"1\">\n      <data key=\"adata.color\">Red</data>\n      <data key=\"adata.size\">0</data>\n      <data key=\"shape\">round</data>\n    </node>\n    <node id=\"2\">\n      <data key=\"adata.color\">Green</data>\n      <data key=\"adata.size\">3</data>\n      <data key=\"shape\">square</data>\n    </node>\n    <edge source=\"1\" target=\"2\">\n      <data key=\"0E\">12</data>\n    </edge>\n  </graph>\n</graphml>";
        let from = labeled_from_graphml::<HashMapLabeledGraph::<SparseSimpleGraph, BData, i32>>(s.to_string());

        pretty_assertions::assert_eq!(true, from.is_ok());
        pretty_assertions::assert_eq!(s, labeled_to_graphml_simple(from.expect("preverified")));
    }
}
