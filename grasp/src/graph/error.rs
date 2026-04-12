use std::{error::Error, fmt::{Debug, Display}};

use crate::graph::{EdgeID, VertexID};

#[derive(Debug, Eq, PartialEq)]
pub enum GraphError{
    VertexNotInGraph(VertexID),
    NeitherVertexInGraph(VertexID, VertexID),
    EdgeNotInGraph(EdgeID),
    DisconnectedGraph,
    EdgeNotAddable(EdgeID, String),
}
impl Display for GraphError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            &Self::VertexNotInGraph(v) => f.write_fmt(format_args!("Vertex {} is not in Graph", v)),
            &Self::NeitherVertexInGraph(v1, v2) => f.write_fmt(format_args!("Neither {v1} nor {v2} are in the Graph.")),
            &Self::EdgeNotInGraph(e) => f.write_fmt(format_args!("Edge {:?} is not in Graph", e)),
            &Self::DisconnectedGraph => f.write_str("The graph should be connected but is."),
            &Self::EdgeNotAddable(e, ref reason) => f.write_fmt(format_args!("Edge {:?} cannot be added to the Graph: {reason}", e)),
        }
    }
}
impl Error for GraphError{}