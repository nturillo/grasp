use std::collections::HashMap;
use crate::graph::{EdgeID, GraphTrait, VertexID, set::{CowIteratorAsCloned, Set}};

pub trait DfsSimpleVisitor{
    // Called on the root vtcs of the search. (VTCS without parent)
    fn root_vertex(&mut self, _vertex: VertexID){}
    // Called on vtcs in preorder
    fn discover_vertex(&mut self, _vertex: VertexID){}
    // Called on vtcs in postorder
    fn finish_vertex(&mut self, _vertex: VertexID){}
    // Called once per tree edge
    fn tree_edge(&mut self, _edge: EdgeID){}
    // Called once per back edge
    fn back_edge(&mut self, _edge: EdgeID){}
}

/// DFS stack based implementation using a visitor. Can support both pre order, post order, tree edges, back edges, and root vertices
pub fn dfs_simple_recursive<G: GraphTrait, V: DfsSimpleVisitor>(
    graph: &G,
    vertex: VertexID,
    visitor: &mut V
){
    let mut dfs_index = HashMap::default();
    let mut current_dfs_index = 0;
    // Run over starting vtx
    visitor.root_vertex(vertex);
    dfs_recurse(graph, vertex, None, &mut dfs_index, &mut current_dfs_index, visitor);
    // Run over any remaining vtcs
    for v in graph.vertices() {
        if dfs_index.contains_key(&v) {continue;}
        visitor.root_vertex(v);
        dfs_recurse(graph, v, None, &mut dfs_index, &mut current_dfs_index, visitor);
    }
}
fn dfs_recurse<G: GraphTrait, V: DfsSimpleVisitor>(
    graph: &G,
    vertex: VertexID, 
    parent: Option<VertexID>,
    dfs_index: &mut HashMap<VertexID, usize>,
    current_dfs_index: &mut usize, 
    visitor: &mut V
){
    dfs_index.insert(vertex, *current_dfs_index);
    *current_dfs_index += 1;

    visitor.discover_vertex(vertex);
    // Visit children
    for vtx in graph.neighbors(vertex).iter().clone_cow(){
        if dfs_index.contains_key(&vtx) { // back edge or parent tree edge
            if parent != Some(vtx) && dfs_index[&vertex] > dfs_index[&vtx] {
                visitor.back_edge((vertex, vtx));
            }
            continue;
        }
        dfs_recurse(graph, vtx, Some(vertex), dfs_index, current_dfs_index, visitor);
        visitor.tree_edge((vertex, vtx)); // tree edge
    }
    visitor.finish_vertex(vertex);
}
