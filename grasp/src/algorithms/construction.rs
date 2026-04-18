use graph_ops_macros::register;
use crate::graph::{VertexID, constructors::*, prelude::{SimpleGraph, SparseSimpleGraph}, directed::DigraphProjection};

#[register(name = "Make Complete Graph", desc = "Builds a Simple Complete Graph.", ret = SimpleGraph, simple = "true", params = [("Size", Unsigned)])]
pub fn sparse_complete_graph<G: SimpleGraph>(_g: &G, size: usize) -> SparseSimpleGraph{
    build_complete_graph(size)
}

#[register(name = "Make Cycle Graph", desc = "Builds a Simple Cycle Graph.", ret = SimpleGraph, simple = "true", params = [("Size", Unsigned)])]
pub fn sparse_cycle_graph<G: SimpleGraph>(_g: &G, size: usize) -> SparseSimpleGraph{
    build_cycle(size)
}

#[register(name = "Make Path Graph", desc = "Builds a Simple Path Graph.", ret = SimpleGraph, simple = "true", params = [("Size", Unsigned)])]
pub fn sparse_path_graph<G: SimpleGraph>(_g: &G, size: usize) -> SparseSimpleGraph{
    build_path(size)
}

#[register(name = "Make Partite Graph", desc = "Builds a Simple Partite Graph.", ret = SimpleGraph, simple = "true", params = [("Partite groups", VertexList)])]
pub fn sparse_partite_graph<G: SimpleGraph>(_g: &G, partite_groups: Vec<VertexID>) -> SparseSimpleGraph{
    build_partite_graph(partite_groups)
}

#[register(name = "Make Binary Tree", desc = "Builds a Complete Binary Tree.", ret = SimpleGraph, simple = "true", params = [("layers", Unsigned)])]
pub fn sparse_binary_graph<G: SimpleGraph>(_g: &G, layers: usize) -> SparseSimpleGraph{
    build_binary_tree(layers)
}

#[register(name = "Make Bowtie Graph", desc = "Builds a Simple Bowtie Graph.", ret = SimpleGraph, simple = "true", params = [])]
pub fn sparse_bowtie_graph<G: SimpleGraph>(_g: &G) -> SparseSimpleGraph{
    build_bowtie()
}
