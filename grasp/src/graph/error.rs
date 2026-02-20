use std::{error::Error, fmt::{Display}};
use super::{EdgeID, VertexID};


#[derive(Debug, Eq, PartialEq)]
pub enum GraphError {
    VertexNotInGraph(VertexID),
    EdgeNotInGraph(EdgeID)
}
impl Display for GraphError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            &Self::VertexNotInGraph(v) => {f.write_fmt(format_args!("Vertex {} is not in Graph", v))},
            &Self::EdgeNotInGraph(e) => {f.write_fmt(format_args!("Edge {:?} is not in Graph", e))}
        }
    }
}
impl Error for GraphError{}