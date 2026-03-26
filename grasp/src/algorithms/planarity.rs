use std::{collections::{HashMap, HashSet, VecDeque}, fmt::Debug, mem::swap, usize};
use bimap::BiHashMap;

use crate::graph::prelude::*;

/// Trait for graphs with Homogenous vertex ID's, necessary for algorithm speedup
pub trait HomogenousVertexId: GraphTrait{}

#[derive(Debug)]
pub struct HomogenousView<'a, G: GraphTrait>{
    /// Map from original ID to homogenous ID
    vertex_map: BiHashMap<VertexID, VertexID>,
    /// VertexID not used by graph, useful for when we want to map a invalid id to an id invalid in graph
    unused_id: VertexID,
    graph: &'a G
}
impl<'a, G: GraphTrait> HomogenousView<'a, G>{
    pub fn from_graph(graph: &'a G) -> Self{
        let mut vertex_map = BiHashMap::with_capacity(graph.vertex_count());
        let mut unused_id = 0;
        for (i, vertex) in graph.vertices().enumerate(){
            if vertex >= unused_id {unused_id = vertex+1;}
            vertex_map.insert(vertex, i);
        }
        Self{vertex_map, graph, unused_id}
    }
}
impl<'a, G: GraphTrait> GraphTrait for HomogenousView<'a, G>{
    fn is_empty(&self) -> bool {self.graph.is_empty()}
    
    fn vertex_count(&self) -> usize {self.graph.vertex_count()}
    
    fn edge_count(&self) -> usize {self.graph.edge_count()}
    
    fn has_vertex(&self, v: VertexID) -> bool {
        let Some(v) = self.vertex_map.get_by_right(&v) else {return false;};
        self.graph.has_vertex(*v)
    }
    
    fn has_edge(&self, e: EdgeID) -> bool {
        let Some(v1) = self.vertex_map.get_by_right(&e.0) else {return false;};
        let Some(v2) = self.vertex_map.get_by_right(&e.1) else {return false;};
        self.graph.has_edge((*v1, *v2))
    }
    
    fn vertices(&self) -> impl Iterator<Item=VertexID> {(0..self.graph.vertex_count()).into_iter()}
    
    fn edges(&self) -> impl Iterator<Item=EdgeID> {
        self.graph.edges().map(|(v1, v2)| (
            *self.vertex_map.get_by_left(&v1).unwrap(),
            *self.vertex_map.get_by_left(&v2).unwrap()
        ))
    }
    
    fn neighbors(&self, v: VertexID) -> impl Set<Item = VertexID> {
        // inner graphs ID of v, or a unused id
        let v = self.vertex_map.get_by_right(&v).copied().unwrap_or(self.unused_id);
        self.graph.neighbors(v).with_bimap(&self.vertex_map)
    }
    
    fn vertex_set(&self) -> impl Set<Item = VertexID> {
        self.graph.vertex_set().with_bimap(&self.vertex_map)
    }
}
impl<'a, G: GraphTrait> HomogenousVertexId for HomogenousView<'a, G>{}
impl<'a, G: GraphTrait+SimpleGraph> SimpleGraph for HomogenousView<'a, G>{}

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
    for i in 0..graph.vertex_count() {
        if dfs_index.contains_key(&i) {continue;}
        visitor.root_vertex(i);
        dfs_recurse(graph, i, None, &mut dfs_index, &mut current_dfs_index, visitor);
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

struct GraphEmbedding{
    /// Maps graph vertex to dfs_index
    vtx_map: HashMap<VertexID, usize>,
    /// DFS Info indexed by DFI
    dfs_data: Vec<DfsData>,
    /// Seperated DFS Children Lists per Vertex
    seperated_dfs_children: Vec<SeperatedDfsChildren>,
    /// Embedding info per Vertex
    embedding: Vec<VertexEmbedding>,
    /// Pertinence info per vertex
    pertinence: Vec<VertexPertinence>,
    /// Original Vertex IDs
    ids: Vec<VertexID>,
    /// Represents the embedding of the bicomps. Requires that starting from a root_edge, you can traverse the external face of the bicomp
    edge_data: Vec<HalfEdge>,
    /// Counter used during dfs to get dfs_index
    dfs_index: usize,
    /// Number of vtcs
    vtx_count: usize,
}
impl GraphEmbedding{
    pub fn new<G: GraphTrait>(graph: &G) -> Self{
        let vertex_count = graph.vertex_count();
        let edge_count = graph.edge_count();
        Self{
            vtx_map: HashMap::with_capacity(vertex_count),
            dfs_data: vec![DfsData::default(); vertex_count],
            seperated_dfs_children: vec![SeperatedDfsChildren::default(); vertex_count],
            embedding: vec![VertexEmbedding::default(); vertex_count],
            pertinence: vec![VertexPertinence::default(); vertex_count],
            ids: vec![VertexID::default(); vertex_count],
            edge_data: Vec::with_capacity(2*edge_count),
            dfs_index: 0,
            vtx_count: graph.vertex_count()
        }
    }

    /// in O(n) time, makes the seperated_dfs_children list be a list of dfs_children sorted by lowpoint
    fn create_seperated_dfs_children(&mut self) {
        let mut lowpoint_counts = vec![0; self.vtx_count];
        for l in self.dfs_data.iter() {lowpoint_counts[l.lowpoint] += 1;}
        let mut sum = 0;
        // prefix sum scan
        for item in lowpoint_counts.iter_mut() {
            swap(item, &mut sum);
            sum += *item;
        }
        // Place vtx ids in vector sorted by lowpoint
        let mut lowpoint_offset = vec![0; self.vtx_count];
        let mut sorted_vtcs = vec![0; self.vtx_count];
        for i in 0..self.vtx_count{
            let lowpoint = self.dfs_data[i].lowpoint;
            sorted_vtcs[lowpoint_counts[lowpoint]+lowpoint_offset[lowpoint]] = i;
            lowpoint_offset[lowpoint] += 1;
        }
        for vtx in sorted_vtcs{
            let Some(parent) = self.dfs_data[vtx].parent else {continue;};
            self.push_back_seperated_dfs_child(parent, vtx);
        }
    }
    /// Pushes child to the back of parent's seperated dfs child list
    fn push_back_seperated_dfs_child(&mut self, parent: usize, child: usize) {
        if let Some(prev) = self.seperated_dfs_children[parent].end {
            self.seperated_dfs_children[prev].next = Some(child);
            self.seperated_dfs_children[child].prev = Some(prev);
            self.seperated_dfs_children[parent].end = Some(child);
        }else {
            self.seperated_dfs_children[parent].start = Some(child);
            self.seperated_dfs_children[parent].end = Some(child);
        }
    }
    /// Removes child from the seperated dfs list of parent.
    fn remove_seperated_dfs_child(&mut self, parent: VertexID, child: VertexID){
        let prev = self.seperated_dfs_children[child].prev;
        let next = self.seperated_dfs_children[child].next;
        if let Some(prev) = prev {
            self.seperated_dfs_children[prev].next = next;
        } else {
            self.seperated_dfs_children[parent].start = next;
        }
        if let Some(next) = next {
            self.seperated_dfs_children[next].prev = prev;
        } else {
            self.seperated_dfs_children[parent].end = prev;
        }
    }
   
    /// Marks pertinent vertices for backedge parent->descendent
    fn walkup(&mut self, reference: usize) {
        for descendent in self.dfs_data[reference].back_edges_to_descendents.iter().copied() {
            self.pertinence[descendent].pertinence = reference;

            let mut l_vertex = descendent; let mut r_vertex = descendent;
            let mut l_on_first = true; let mut r_on_first = false; // Different directions.
            // Find pertinent roots
            loop {
                // Visit current vertex
                self.pertinence[l_vertex].visited = reference; self.pertinence[r_vertex].visited = reference;
                // Check if next vertex is root
                if let Some(canon) = self.embedding[l_vertex].canonical_child.or(self.embedding[r_vertex].canonical_child) {
                    let root = self.dfs_data[canon].parent.unwrap(); // Unwrap here since canonical children must have parents
                    // Mark pertinent root. Store optimized
                    if self.dfs_data[canon].lowpoint < reference { // Externally Active Bicomp
                        self.pertinence[root].pertinent_roots.push_front(canon);
                    } else { // Internally Active
                        self.pertinence[root].pertinent_roots.push_back(canon);
                    }
                    // Finished walkup if we got to the reference or the root is visited
                    if root == reference || self.pertinence[root].visited == reference {break;} 
                    // Start searching root's bicomp
                    l_vertex = root; r_vertex = root; l_on_first = true; r_on_first = false;
                    continue;
                }
                // Otherwise continue along external face
                (l_vertex, l_on_first) = self.get_next_external_vertex(l_vertex, l_on_first);
                (r_vertex, r_on_first) = self.get_next_external_vertex(r_vertex, r_on_first);
                // if visited, return early
                if self.pertinence[l_vertex].visited == reference || self.pertinence[r_vertex].visited == reference {break;}
            }
        }
    }
    
    /// Consume pertinence to merge back edges from reference to descendants, merging bicomps and flipping bicomps as needed.
    /// Returns false if failure occurs
    fn walkdown(&mut self, reference: usize) {
        // Stores visited bicomp canonical children and for each bicomp, how it entered the root vertex, and how it left the root vertex in the bicomp
        let mut stack: Vec<(usize, bool, bool)> = vec![];
        loop{
            // Get next pertinent root (canonical child) of reference
            let Some(canon) = self.pertinence[reference].pertinent_roots.pop_front() 
                else {break;};
            stack.clear();
            // Traverse both directions once
            self.walkdown_proc(reference, canon, true, &mut stack);
            if !stack.is_empty() {return;}
            self.walkdown_proc(reference, canon, false, &mut stack);
            if !stack.is_empty() {return;}
        }
    }
    /// For each child of reference, this proc is run twice, once for both directions. Returns false if non planarity detected
    fn walkdown_proc(&mut self, reference: usize, canon: usize, leave_root_on_first: bool, stack: &mut Vec<(usize, bool, bool)>) {
        let root: usize = self.dfs_data[canon].parent.unwrap();
        let (mut vertex, mut in_on_first) = self.get_next_external_vertex_root(canon, !leave_root_on_first);
        while vertex != root{
            // Found a backedge to embed
            if self.pertinence[vertex].pertinence == reference {
                // Merge bicomps in stack
                while let Some((merged_canon, iof, oof)) = stack.pop(){
                    // If we swapped directions, flip bicomp
                    if !(oof ^ iof) {self.flip_bicomp(merged_canon);}
                    self.merge_bicomps(merged_canon, iof);
                }
                // Embed backedge
                self.embed_backedge(reference, canon, vertex, leave_root_on_first, in_on_first, false);
                // Clear Backedge Flag
                self.pertinence[vertex].pertinence = self.vtx_count;
            }
            if !self.pertinence[vertex].pertinent_roots.is_empty() {
                // Traverse into pertinent bicomp to merge next backedge. Once the backedge in that bicomp is merged, \
                // the bicomp will return to being 'canon' (as the bicomps were merged)
                // Get next bicomp, start with internally active (back of list)
                let new_canon = *self.pertinence[vertex].pertinent_roots.back().unwrap(); 
                let (x, x_on_first) = self.find_active_successor(reference, new_canon, true);
                let (y, y_on_first) = self.find_active_successor(reference, new_canon, false);
                if self.is_internally_active(x, reference) {
                    stack.push((new_canon, in_on_first, false));
                    vertex = x; in_on_first = x_on_first;
                } else if self.is_internally_active(y, reference) {
                    stack.push((new_canon, in_on_first, true));
                    vertex = y; in_on_first = y_on_first;
                } else if self.is_pertinent(x, reference) {
                    stack.push((new_canon, in_on_first, false));
                    vertex = x; in_on_first = x_on_first;
                } else {
                    stack.push((new_canon, in_on_first, true));
                    vertex = y; in_on_first = y_on_first;
                }
            } else if !self.is_pertinent(vertex, reference) && !self.is_externally_active(vertex, reference) {
                // vertex is inactive, move forward
                (vertex, in_on_first) = self.get_next_external_vertex(vertex, in_on_first);
            } else {
                // vertex is a stopping vertex, cannot proceed since it is either pertinent, or externally active. \
                // It is not pertinent, since it has no backedgeFlag (already imbedded above) and has no pertinent roots (prior if). \
                // Thus it is externally active, and thus a stopping vertex
                if self.dfs_data[canon].lowpoint < reference && stack.is_empty(){
                    self.embed_backedge(reference, canon, vertex, leave_root_on_first, in_on_first, true);
                }
                break;
            }
        }
    }
    
    /// embed vertex -> reference into bicomp rooted at reference with canon. \
    /// Keeps first canonical edge on the inside if dir is true. \
    /// Needs to know if vertex was reached with its first external edge.
    fn embed_backedge(&mut self, reference: usize, canon: usize, vertex: usize, left_root_on_first: bool, in_on_first: bool, short_circuit: bool) {
        let to_r = self.edge_data.len();
        let to_v = to_r+1;

        let (v_first, v_second) = self.embedding[vertex].external_edges;
        let (r_first, r_second) = self.embedding[canon].canonical_edges;

        self.edge_data[v_second].next = to_r;
        self.edge_data[r_second].next = to_v;

        self.edge_data.push(HalfEdge { twin: to_v, next: v_first, neighbor: reference, short_circuit });
        self.edge_data.push(HalfEdge { twin: to_r, next: r_first, neighbor: vertex, short_circuit });

        if in_on_first {
            self.embedding[vertex].external_edges.0 = to_r;
        } else {
            self.embedding[vertex].external_edges.1 = to_r;
        }
        if left_root_on_first {
            self.embedding[canon].canonical_edges.0 = to_v;
        } else {
            self.embedding[canon].canonical_edges.1 = to_v;
        }
        // Setup canonical child pointer
        self.embedding[vertex].canonical_child = Some(canon);
    }
    /// Merges a bicomp into the bicomp containing its root. Assumes bicomp needs not be flipped. \
    /// Requires whether the root was visited using its first external edge. \
    /// If so, the first external edge is replaced with the bicomps first canonical edge. (traversed path is kept inside).
    fn merge_bicomps(&mut self, canon: usize, in_on_first: bool, ) {
        let root = self.dfs_data[canon].parent.unwrap();
        // remove merged_canon from pertinent roots of merged_canon's parent
        self.pertinence[root].pertinent_roots.retain(|r| *r!=canon);
        // Remove merged_canon from merged_canon's parents 
        self.remove_seperated_dfs_child(root, canon);
        // Circular Union: 
        self.edge_data[self.embedding[canon].canonical_edges.1].next = self.embedding[root].external_edges.0;
        self.edge_data[self.embedding[root].external_edges.1].next = self.embedding[canon].canonical_edges.0;
        // Update external Edges
        if in_on_first {
            self.embedding[root].external_edges.0 = self.embedding[canon].canonical_edges.0;
        }else {
            self.embedding[root].external_edges.1 = self.embedding[canon].canonical_edges.1;
        }
        // Reset canon option
        let (a,b) = self.embedding[canon].canonical_edges;
        self.embedding[self.edge_data[a].neighbor].canonical_child = None;
        self.embedding[self.edge_data[b].neighbor].canonical_child = None;
    }
    /// Flips a bicomp from canonical child.
    fn flip_bicomp(&mut self, canon: usize){
        let (start, end) = &mut self.embedding[canon].canonical_edges;
        // Swap adjacency list direction
        let start_edge = *start;
        let mut prev = start_edge;
        let mut current = self.edge_data[start_edge].next;
        // Traverse the circular list and reverse all next pointers
        while current != start_edge {
            // This is evil :)
            std::mem::swap(&mut self.edge_data[current].next, &mut prev);
            std::mem::swap(&mut prev, &mut current);
        }
        // Complete the circle by fixing the start edge
        self.edge_data[start_edge].next = prev;
        
        // Swap external edge pointers
        std::mem::swap(start, end);
        // set sign of canon vtx to -1
        self.embedding[canon].flipped = true;
    }
    
    /// Same as get_next_external_vertex, but explicitly for root vtcs, described by their canonical child
    fn get_next_external_vertex_root(&self, canon: usize, in_on_first: bool) -> (usize, bool){
        // Get the edge we didn't come in on
        let out_edge = if in_on_first {self.embedding[canon].canonical_edges.1} else {self.embedding[canon].canonical_edges.0};
        // Next vertex should be whatever vtx the out edge points to
        let next_vertex = self.edge_data[out_edge].neighbor;
        // If next vertexs first external edge is the twin of out edge, then we came in_on_first.
        let on_first = self.edge_data[self.embedding[next_vertex].external_edges.0].twin == out_edge;
        (next_vertex, on_first)
    }
    /// Given current vertex and whether the first external edge was used to traverse to it,\
    /// returns next vertex and whether the first external edge was used to traverse to it
    fn get_next_external_vertex(&self, vertex: usize, in_on_first: bool) -> (usize, bool){
        // Get the edge we didn't come in on
        let out_edge = if in_on_first {self.embedding[vertex].external_edges.1} else {self.embedding[vertex].external_edges.0};
        // Next vertex should be whatever vtx the out edge points to
        let next_vertex = self.edge_data[out_edge].neighbor;
        // If next vertexs first external edge is the twin of out edge, then we came in_on_first.
        let on_first = self.edge_data[self.embedding[next_vertex].external_edges.0].twin == out_edge;
        (next_vertex, on_first)
    }
    
    /// Finds the first active (pertinent or externally active) vertex on the external face of a bicomp
    fn find_active_successor(&self, reference: usize, canon: usize, dir: bool) -> (usize, bool){
        let (mut vertex, mut in_on_first) = self.get_next_external_vertex_root(canon, dir);
        while !self.is_pertinent(vertex, reference) && !self.is_externally_active(vertex, reference) {
            (vertex, in_on_first) = self.get_next_external_vertex(vertex, in_on_first);
        }
        (vertex, in_on_first)
    }
    
    /// Whether the vertex is pertinent relative to reference
    fn is_pertinent(&self, vertex: usize, reference: usize) -> bool{
        self.pertinence[vertex].pertinence == reference || 
        !self.pertinence[vertex].pertinent_roots.is_empty()
    }
    /// Whether a vertex is internall active (pertinent but not externally active)
    fn is_internally_active(&self, vertex: usize, reference: usize) -> bool{
        self.is_pertinent(vertex, reference) && !self.is_externally_active(vertex, reference)
    }
    /// Whether the vertex is externally active relative to reference. O(1) due to seperated DFS child list
    fn is_externally_active(&self, vertex: usize, reference: usize) -> bool{
        // Directly connects to ancestor of reference
        self.dfs_data[vertex].least_ancestor < reference || 
        // First seperated dfs child has lowpoint < reference
        self.seperated_dfs_children[vertex].start.is_some_and(|sdfs| self.dfs_data[sdfs].lowpoint < reference) 
    }
    
    /// Gets the circular adjacency list for a vertex. flip changes the orientation
    fn get_adjacency_list(&self, start_edge: usize, flipped: bool) -> VecDeque<usize>{
        let mut list = VecDeque::new();
        if !self.edge_data[start_edge].short_circuit{
            if flipped {list.push_front(self.edge_data[start_edge].neighbor);}
            else {list.push_back(self.edge_data[start_edge].neighbor);}
        }
        let mut current = self.edge_data[start_edge].next;
        while current != start_edge{
            if !self.edge_data[current].short_circuit {
                if flipped {list.push_front(self.edge_data[current].neighbor);}
                else {list.push_back(self.edge_data[current].neighbor);}
            }
            current = self.edge_data[current].next;
        }
        list
    }
    /// Builds the planar embedding. Should only be run after a successful run of the core algorithm
    fn recover_planar_embedding(&self) -> PlanarEmbedding{
        // Starting at top vtx, create clockwise adjacency lists. Keep a stack of booleans for flipping
        let mut flip_stack = vec![];
        let mut adjacency_list = HashMap::with_capacity(self.vtx_count);
        for vertex in 0..self.vtx_count{
            // pop the flip stack to get the current flip value
            let flip_value = flip_stack.pop().unwrap_or(false);
            let flip = flip_value ^ self.embedding[vertex].flipped;
            // push stack once for each dfs child (thus each child knows its parents flip state)
            for _ in 0..self.dfs_data[vertex].dfs_children.len() {flip_stack.push(flip);}
            // If vertex is a root vertex, we need to merge all of its bicomps together.
            if self.dfs_data[vertex].parent.is_none() {
                // Luckily, root vtcs are never flipped and the flip stack should always be empty at a dfs root
                let mut list = Vec::new();
                for child in self.dfs_data[vertex].dfs_children.iter(){
                    let start = self.embedding[*child].canonical_edges.0;
                    list.extend(self.get_adjacency_list(start, flip).into_iter().map(|vtx| self.ids[vtx]));
                }
                adjacency_list.insert(self.ids[vertex], list);
                continue;
            }
            // Otherwise just merge its adjacency list
            let start = self.embedding[vertex].external_edges.0;
            adjacency_list.insert(self.ids[vertex], self.get_adjacency_list(start, flip).into_iter().map(|vtx| self.ids[vtx]).collect());
        }
        // Convert to planar embedding
        PlanarEmbedding { circular_adjacency_lists: adjacency_list }
    }

    /// Finds a kuratowski subgraph from a embedding. Assumes the core algorithm has been run until failure.
    fn find_kuratowski(&self) -> KuratowskiSubgraph{
        KuratowskiSubgraph { edge_set: HashSet::default() }
    }
}
/// Sets up the vtx_map, vtx_data data with 
impl DfsSimpleVisitor for GraphEmbedding{
    fn discover_vertex(&mut self, vertex: VertexID) {
        // Get preorder DFI and record into vtx_map
        let index = self.dfs_index; self.dfs_index += 1;
        self.vtx_map.insert(vertex, index);
        self.ids[index] = vertex;
        self.dfs_data[index].least_ancestor = index;
        // Set flags to false.
        self.pertinence[index].pertinence = usize::MAX;
        self.pertinence[index].visited = usize::MAX;
    }
    fn finish_vertex(&mut self, vertex: VertexID) {
        // Calculate lowpoint. Since finish is called from leaves towards the root, lowpoint propogates up the tree correctly.
        let index = self.vtx_map[&vertex];
        let least_ancestor = self.dfs_data[index].least_ancestor;
        let least_child_lowpoint = self.dfs_data[index].dfs_children.iter()
            .map(|child| self.dfs_data[*child].lowpoint).min().unwrap_or(least_ancestor);
        self.dfs_data[index].lowpoint = least_ancestor.min(least_child_lowpoint);
    }
    fn tree_edge(&mut self, (parent, child): EdgeID) {
        let (parent, child) = (self.vtx_map[&parent], self.vtx_map[&child]);
        self.dfs_data[child].parent = Some(parent);
        self.dfs_data[parent].dfs_children.push(child);
        // Embed tree edge as singleton bicomp
        let root_edge = self.edge_data.len();
        let twin_edge = root_edge+1;
        // Set each singleton bicomp to have external face of 1 half edge. 
        self.edge_data.push(HalfEdge { twin: twin_edge, neighbor: child, next: root_edge, short_circuit: false });
        self.edge_data.push(HalfEdge { twin: root_edge, neighbor: parent, next: twin_edge, short_circuit: false });
        self.embedding[child].external_edges = (twin_edge, twin_edge);
        self.embedding[child].canonical_child = Some(child);
        self.embedding[child].canonical_edges = (root_edge, root_edge);
    }
    fn back_edge(&mut self, (vertex, ancestor): EdgeID) {
        // Update least_ancestor
        let (vertex, ancestor) = (self.vtx_map[&vertex], self.vtx_map[&ancestor]);
        let current = self.dfs_data[vertex].least_ancestor;
        if ancestor < current {self.dfs_data[vertex].least_ancestor = ancestor;}
        // record backedge
        self.dfs_data[ancestor].back_edges_to_descendents.push(vertex);
    }
}
impl Debug for GraphEmbedding{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "GraphEmbedding {{")?;
        writeln!(f, "  vtx_data: [")?;
        for vtx in 0..self.vtx_count {
            writeln!(f, "    {:?},", self.dfs_data[vtx])?;
            writeln!(f, "        {:?},", self.seperated_dfs_children[vtx])?;
            writeln!(f, "        {:?},", self.embedding[vtx])?;
            writeln!(f, "        {:?},", self.pertinence[vtx])?;
        }
        writeln!(f, "  ],")?;
        writeln!(f, "  edge_data: [")?;
        for edge in &self.edge_data {
            writeln!(f, "    {:?},", edge)?;
        }
        writeln!(f, "  ],")?;
        writeln!(f, "  dfs_index: {},", self.dfs_index)?;
        writeln!(f, "  vtx_count: {}", self.vtx_count)?;
        write!(f, "}}")
    }
}

/// DFS Data per Vertex
#[derive(Debug, Default, Clone)]
struct DfsData{
    /// Lowest DFI reachable by traversing dfs subtree and a single backedge
    lowpoint: usize,
    /// Lowest DFI reachable by a single backedge in the dfs tree tree edges excluded
    least_ancestor: usize,
    /// DFI of parent in dfs tree. None if root node
    parent: Option<usize>,
    /// All child DFI of the vtx
    dfs_children: Vec<usize>,
    /// List of backedges to descendents
    back_edges_to_descendents: Vec<usize>,
}

/// Doubly Linked List of Seperated DFS Children. Stored Per Vertex
#[derive(Debug, Default, Clone)]
struct SeperatedDfsChildren{
    /// Start node for this vtx's seperated dfs list
    start: Option<usize>,
    /// end node for this vtx's seperated dfs list
    end: Option<usize>,
    /// next node for the parent vtx's seperated dfs list
    next: Option<usize>,
    /// prev node for the parent vtx's seperated dfs list
    prev: Option<usize>
}

/// Embedding Info per Vertex \
/// By convention for external edges, second->next = first. IE first is first in adjacency list, second is last in list.
#[derive(Debug, Default, Clone)]
struct VertexEmbedding{
    /// The two external edges of this vertex in its containing bicomp. Both should go from this vertex to another vertex
    external_edges: (usize, usize),
    /// External edges of root vertex of a bicomp, stored on canonical child.
    canonical_edges: (usize, usize),
    /// The canonical child of the containing bicomp, only valid on external neighbors of root vertex. \
    /// Can be used to test if a vertex is an external neighbor of the root vtx
    /// Canonical Child: For bicomp with root a and minimal dfs child b, b is the canonical child.
    canonical_child: Option<usize>,
    /// Whether or not the bicomp with this canonical child was flipped. If a vtx is flipped, it means all descendent vtcs in the dfs tree are also flipped.
    flipped: bool,
}

/// Pertinence information per Vertex. Set by Walkup, consumed by Walkdown
#[derive(Debug, Default, Clone)]
struct VertexPertinence{
    /// Flag set by walkup and consumed by walkdown. Indicates that this vertex is pertinent towards the embedding of all backedges connected to some vertex. \
    /// Set to DFI of reference, thus flag is true if flag = reference
    pertinence: usize,
    /// Subset of DFS children. set of bicomps that are pertinent \
    /// Stored by canonical child \
    /// VecDeque to allow externally active roots to be placed first and internally active roots placed after.
    pertinent_roots: VecDeque<usize>,
    /// flag used by walkup to avoid duplicate work. \
    /// Set to DFI of current reference as a timestamp. Thus flag is true if flag = reference.
    visited: usize
}

/// Data per half edge in the graph embedding, used to allow fast face traversal and other operations
/// Adjacency list has for external edges, second->next = first.
#[derive(Debug, Default, Clone)]
struct HalfEdge{
    /// opposite direction edge index in edge_data
    twin: usize,
    /// Next edge in the circular adjacency of the incident vertex's adjacency list
    next: usize,
    /// id of vertex this edge goes to, index in vtx_data
    neighbor: VertexID,
    /// Flag for edges added as short circuit edges
    short_circuit: bool,
}

/// Simple type to hold planar embeddings
pub struct PlanarEmbedding{
    circular_adjacency_lists: HashMap<VertexID, Vec<VertexID>>,
}

/// Simple type to hold kuratowski subgraph for non planar graphs
pub struct KuratowskiSubgraph{
    edge_set: HashSet<EdgeID>,
}

/// O(n) algorithm for determining if a graph is planar
pub fn check_planarity<G: SimpleGraph>(graph: &G) -> Result<PlanarEmbedding, KuratowskiSubgraph>{
    if graph.is_empty() {return Ok(PlanarEmbedding { circular_adjacency_lists: HashMap::default() });}
    // Create embedding structure
    let mut embedding = GraphEmbedding::new(graph);
    // Run DFS to build vtx data
    dfs_simple_recursive(graph, 0, &mut embedding);
    // Efficiently create sorted seperated_dfs_children lists on the vtcs.
    // Also embeds tree edges into the embedding.
    embedding.create_seperated_dfs_children();
    // Start adding back edges to embedding in reverse DFS order
    for reference in (0..graph.vertex_count()).rev(){
        // Walkup
        embedding.walkup(reference);
        // Walkdown
        embedding.walkdown(reference);
        // make sure all backedges were imbedded
        for descendent in embedding.dfs_data[reference].back_edges_to_descendents.iter(){
            // If failed to embed backedge, not
            if embedding.pertinence[*descendent].pertinence == reference {return Err(embedding.find_kuratowski());}
        }
    }
    // Is planar, return the planar embedding
    Ok(embedding.recover_planar_embedding())
}

#[cfg(test)]
mod test{
    use crate::{algorithms::planarity::check_planarity, graph::prelude::*};
    #[test]
    fn test_planarity(){
        let mut k33 = SparseSimpleGraph::default();
        k33.add_vertex(0); k33.add_vertex(1); k33.add_vertex(2); k33.add_vertex(3); k33.add_vertex(4);
        assert!(check_planarity(&k33).is_ok());
        k33.add_edge((0, 3));
        k33.add_edge((0, 4));
        k33.add_edge((0, 5));
        k33.add_edge((1, 3));
        k33.add_edge((1, 4));
        k33.add_edge((1, 5));
        k33.add_edge((2, 3));
        k33.add_edge((2, 4));
        assert!(check_planarity(&k33).is_ok());
        k33.add_edge((2, 5));
        assert!(check_planarity(&k33).is_err());
        let mut k5 = SparseSimpleGraph::default();
        assert!(check_planarity(&k5).is_ok());
        k5.add_edge((0, 1));
        k5.add_edge((0, 2));
        k5.add_edge((0, 3));
        k5.add_edge((0, 4));
        k5.add_edge((1, 2));
        k5.add_edge((1, 3));
        k5.add_edge((1, 4));
        k5.add_edge((2, 3));
        k5.add_edge((2, 4));
        assert!(check_planarity(&k5).is_ok());
        k5.add_edge((3, 4));
        assert!(check_planarity(&k5).is_err());
    }
}
