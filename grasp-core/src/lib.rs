pub mod graph;
pub mod algorithms;

pub use graph::errors::GraphError;
pub use graph::graph_traits::{
    GraphTrait,
    SetTrait,
    VertexType,
    EdgeType,
};
pub use graph::adjacency_list::SparseGraph;
pub use algorithms::algo_traits::AlgoTrait;
pub use algorithms::search::{
    BfsIter,
    DfsIter,
    TraversalIter,
    Dijkstra,
};