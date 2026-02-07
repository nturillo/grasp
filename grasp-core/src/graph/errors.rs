#[derive(Debug, Eq, PartialEq)]
pub enum GraphError {
    VertexNotInGraph,
    EdgeNotInGraph,
    ArithmeticOverflow
}