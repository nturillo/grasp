use core::panic;
use std::{collections::{HashMap, HashSet, VecDeque}, f32::consts::TAU, fmt::Debug, hash::Hash, mem::swap, usize};
use graph_ops_macros::register;

use crate::graph::prelude::*;
use super::search_visitors::*;

#[register(name = "Planarity", desc = "Determines the planarity of the graph. Returns an embedding if planar, or a kuratowski minor if not.", simple = "true", ret = Planarity, params = [])]
/// Get planarity
pub fn get_straightedge_embedding<G: SimpleGraph>(g: &G) -> Result<HashMap<usize, (f32, f32)>, HashSet<(usize, usize)>> {
    let mut embedding = GraphPlanarity::from_graph(g);
    match embedding.get_planarity_structure() {
        Ok(mut embedding) => Ok(embedding.calculate_euclidean_embedding()),
        Err(subgraph) => Err(subgraph.edge_set)
    }
}

/// Struct used to calculate planarity and build planarity structures
#[derive(Clone)]
pub struct GraphPlanarity<'a, G: GraphTrait>{
    graph: &'a G,
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
    /// Number of vtcs
    vtx_count: usize,
    /// Whether the algorithm has been run
    is_planar: Option<bool>,
    /// DFS index tracker for use in dfs visitor
    dfs_index: usize
}
impl<'a, G: GraphTrait> GraphPlanarity<'a, G>{
    pub fn from_graph(graph: &'a G) -> Self{
        let vertex_count = graph.vertex_count();
        let edge_count = graph.edge_count();
        Self{
            graph,
            vtx_map: HashMap::with_capacity(vertex_count),
            dfs_data: vec![DfsData::default(); vertex_count],
            seperated_dfs_children: vec![SeperatedDfsChildren::default(); vertex_count],
            embedding: vec![VertexEmbedding::default(); vertex_count],
            pertinence: vec![VertexPertinence::default(); vertex_count],
            ids: vec![VertexID::default(); vertex_count],
            edge_data: Vec::with_capacity(2*edge_count),
            vtx_count: graph.vertex_count(),
            is_planar: None,
            dfs_index: 0
        }
    }

    /// O(n) algorithm for determining if a graph is planar
    pub fn compute_planarity(&mut self) -> bool {
        if let Some(planarity) = self.is_planar {return planarity;}
        else if self.vtx_count == 0 {
            self.is_planar = Some(true); return true;
        }
        let vertex = self.graph.vertices().next().unwrap();
        dfs_simple_recursive(self.graph, vertex, self);
        // Efficiently create sorted seperated_dfs_children lists on the vtcs.
        // Also embeds tree edges into the embedding.
        self.create_seperated_dfs_children();
        // Start adding back edges to embedding in reverse DFS order
        for reference in (0..self.vtx_count).rev(){
            // Walkup
            self.walkup(reference);
            // Walkdown
            self.walkdown(reference);
            // make sure all backedges were imbedded
            for descendent in self.dfs_data[reference].back_edges_to_descendents.iter(){
                // If failed to embed backedge, not
                if self.pertinence[*descendent].pertinence == reference {
                    self.is_planar = Some(false);
                    return false;
                }
            }
        }
        // Is planar, return true
        self.is_planar = Some(true);
        true
    }
    /// Returns a planar embedding if the graph is planar, and a kuratowski subgraph if not
    pub fn get_planarity_structure(&mut self) -> Result<PlanarEmbedding, KuratowskiSubgraph>{
        if self.compute_planarity() {
            Ok(self.recover_planar_embedding())
        } else {
            Err(self.find_kuratowski())
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
                // println!("Walkup: {} {:?} {} {:?}", l_vertex, self.embedding[l_vertex].canonical_child, r_vertex, self.embedding[r_vertex].canonical_child);
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
        // println!("Begin walkdown: {}, leave_root_on_first: {}", root, leave_root_on_first);
        let (mut vertex, mut in_on_first) = self.get_next_external_vertex_root(canon, !leave_root_on_first);
        let mut i = 0;
        while vertex != root{
            i += 1;
            if i > 20 {panic!();}
            // println!("Vertex: {}", vertex);
            // Found a backedge to embed
            if self.pertinence[vertex].pertinence == reference {
                // println!("Embed: {}", vertex);
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
                // println!("Descend: {}", vertex);
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
                    // TODO: Figure out why this breaks the algorithm.
                    // self.embed_backedge(reference, canon, vertex, leave_root_on_first, in_on_first, true);
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

        // println!("Setting {} {} next to {} {}", vertex, self.edge_data[v_second].neighbor, vertex, reference);
        self.edge_data[v_second].next = to_r;
        // println!("Setting {} {} next to {} {}", reference, self.edge_data[r_second].neighbor, reference, vertex);
        self.edge_data[r_second].next = to_v;

        // println!("Setting {} {} next to {} {}", vertex, reference, vertex, self.edge_data[v_first].neighbor);
        self.edge_data.push(HalfEdge { twin: to_v, next: v_first, neighbor: reference, short_circuit });
        // println!("Setting {} {} next to {} {}", reference, vertex, reference, self.edge_data[r_first].neighbor);
        self.edge_data.push(HalfEdge { twin: to_r, next: r_first, neighbor: vertex, short_circuit });
        
        // If we are biconnecting a tree edge, just make it work with the neighbor edges
        if self.embedding[vertex].external_edges.0 == self.embedding[vertex].external_edges.1 {
            let neighbor = self.edge_data[self.embedding[vertex].external_edges.0].neighbor;
            let from_neighbors_first = self.edge_data[self.embedding[neighbor].external_edges.0].neighbor == vertex;
            if from_neighbors_first {
                // println!("Setting {} first -> {}", vertex, reference);
                self.embedding[vertex].external_edges.0 = to_r;
            } else {
                // println!("Setting {} second -> {}", vertex, reference);
                self.embedding[vertex].external_edges.1 = to_r;
            }
            self.embedding[neighbor].canonical_child = None;
        }else if in_on_first {
            // println!("Setting {} first -> {}", vertex, reference);
            self.embedding[vertex].external_edges.0 = to_r;
            let neighbor = self.edge_data[self.embedding[vertex].external_edges.1].neighbor;
            self.embedding[neighbor].canonical_child = None;
        } else {
            // println!("Setting {} second -> {}", vertex, reference);
            self.embedding[vertex].external_edges.1 = to_r;
            let neighbor = self.edge_data[self.embedding[vertex].external_edges.0].neighbor;
            self.embedding[neighbor].canonical_child = None;
        }
        if left_root_on_first {
            // println!("Setting canon {} first -> {}", reference, vertex);
            self.embedding[canon].canonical_edges.0 = to_v;
        } else {
            // println!("Setting canon {} second -> {}", reference, vertex);
            self.embedding[canon].canonical_edges.1 = to_v;
        }
        // Setup canonical child pointer
        self.embedding[vertex].canonical_child = Some(canon);
        self.embedding[canon].canonical_child = Some(canon);
    }
    /// Merges a bicomp into the bicomp containing its root. Assumes bicomp needs not be flipped. \
    /// Requires whether the root was visited using its first external edge. \
    /// If so, the first external edge is replaced with the bicomps first canonical edge. (traversed path is kept inside).
    fn merge_bicomps(&mut self, canon: usize, in_on_first: bool) {
        let root = self.dfs_data[canon].parent.unwrap();
        // remove merged_canon from pertinent roots of merged_canon's parent
        self.pertinence[root].pertinent_roots.retain(|r| *r!=canon);
        // Remove merged_canon from merged_canon's parents 
        self.remove_seperated_dfs_child(root, canon);
        // Circular Union: 
        // println!("Setting {} {} next to {} {}", 
        //     root, self.edge_data[self.embedding[canon].canonical_edges.1].neighbor, 
        //     root, self.edge_data[self.embedding[root].external_edges.0].neighbor
        // );
        self.edge_data[self.embedding[canon].canonical_edges.1].next = self.embedding[root].external_edges.0;
        // println!("Setting {} {} next to {} {}", 
        //     root, self.edge_data[self.embedding[root].external_edges.1].neighbor, 
        //     root, self.edge_data[self.embedding[canon].canonical_edges.0].neighbor
        // );
        self.edge_data[self.embedding[root].external_edges.1].next = self.embedding[canon].canonical_edges.0;
        // Update external Edges
        if in_on_first {
            // println!("Merge: Setting {} first -> {}", root, self.edge_data[self.embedding[canon].canonical_edges.0].neighbor);
            self.embedding[root].external_edges.0 = self.embedding[canon].canonical_edges.0;
        }else {
            // println!("Merge: Setting {} second -> {}", root, self.edge_data[self.embedding[canon].canonical_edges.1].neighbor);
            self.embedding[root].external_edges.1 = self.embedding[canon].canonical_edges.1;
        }
        // Reset canon option
        let (a,b) = self.embedding[canon].canonical_edges;
        self.embedding[self.edge_data[a].neighbor].canonical_child = None;
        self.embedding[self.edge_data[b].neighbor].canonical_child = None;
    }
    /// Flips a bicomp from canonical child.
    fn flip_bicomp(&mut self, canon: usize){
        // If we are a singleton bicomp, don't flip its extraneous and messes up the planar embedding
        if self.embedding[canon].canonical_edges.0 == self.embedding[canon].canonical_edges.1 {return;}
        let (start, end) = &mut self.embedding[canon].canonical_edges;
        // println!("Reversing list from {} {} to {} {}", self.dfs_data[canon].parent.unwrap(), self.edge_data[*start].neighbor, self.dfs_data[canon].parent.unwrap(), self.edge_data[*end].neighbor);
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
        // println!("Swapping vertex {}'s external edges, {} {}", self.dfs_data[canon].parent.unwrap(), self.edge_data[*start].neighbor, self.edge_data[*end].neighbor);
        std::mem::swap(start, end);
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
        // If next vertex is end of tree edge, then in_on_first is always false
        if self.embedding[canon].canonical_edges.1 == self.embedding[canon].canonical_edges.0 {
            (next_vertex, false)
        }else {
            (next_vertex, on_first)
        }
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
        // If next vertex is end of tree edge, then in_on_first is always false
        if self.embedding[vertex].external_edges.1 == self.embedding[vertex].external_edges.0 {
            (next_vertex, false)
        }else {
            (next_vertex, on_first)
        }
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
        
    pub fn get_circular_adjacency_list(&self) -> HashMap<VertexID, Vec<VertexID>>{
        // println!("{:?}", self);
        // Starting at top vtx, create clockwise adjacency lists. Keep a stack of booleans for flipping
        let mut flip_stack = vec![];
        let mut adjacency_list = HashMap::with_capacity(self.vtx_count);
        for vertex in 0..self.vtx_count{
            // pop the flip stack to get the current flip value
            let flip_value = flip_stack.pop().unwrap_or(false);
            let flip = flip_value ^ self.embedding[vertex].flipped;
            // push stack once for each dfs child (thus each child knows its parents flip state)
            for _ in 0..self.dfs_data[vertex].dfs_children.len() {flip_stack.push(flip);}
            // If the vertex has seperated dfs children, they are roots of bicomps that need to be merged
            if let Some(start) = self.seperated_dfs_children[vertex].start {
                // If vertex is not root vertex, there is external edges adjacency list to merge
                let mut list = if self.dfs_data[vertex].parent.is_some(){
                    let start = self.embedding[vertex].external_edges.0;
                    self.get_adjacency_list(start, flip).into_iter().map(|vtx| self.ids[vtx]).collect()
                } else {Vec::new()};
                let a = self.embedding[start].canonical_edges.0;
                let child_flip = flip ^ self.embedding[start].flipped;
                list.extend(self.get_adjacency_list(a, child_flip).into_iter().map(|vtx| self.ids[vtx]));
                let mut cur = self.seperated_dfs_children[start].next;
                while cur != None{
                    let child = cur.unwrap();
                    let a = self.embedding[child].canonical_edges.0;
                    let child_flip = flip ^ self.embedding[child].flipped;
                    list.extend(self.get_adjacency_list(a, child_flip).into_iter().map(|vtx| self.ids[vtx]));
                    cur = self.seperated_dfs_children[child].next;
                }
                adjacency_list.insert(self.ids[vertex], list);
                continue;
            }
            // Otherwise just merge its adjacency list
            let start = self.embedding[vertex].external_edges.0;
            let adj_list = self.get_adjacency_list(start, flip).into_iter().map(|vtx| self.ids[vtx]).collect();
            adjacency_list.insert(self.ids[vertex], adj_list);
        }
        // println!("{:?}", adjacency_list);
        //panic!();
        adjacency_list
    }

    /// Builds the planar embedding. Should only be run after a successful run of the core algorithm
    fn recover_planar_embedding(&self) -> PlanarEmbedding{
        // Convert to DCEL Planar Embedding
        PlanarEmbedding::from_circular_adjacency_list(self.get_circular_adjacency_list())
    }

    /// Finds a kuratowski subgraph from a embedding. Assumes the core algorithm has been run until failure.
    fn find_kuratowski(&self) -> KuratowskiSubgraph{
        // TODO: write this part of the algorithm. 
        KuratowskiSubgraph { edge_set: HashSet::default() }
    }
}
/// Sets up the vtx_map, vtx_data data with 
impl<'a, G: GraphTrait> DfsSimpleVisitor for GraphPlanarity<'a, G>{
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
impl<'a, G: GraphTrait> Debug for GraphPlanarity<'a, G>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Planarity{{")?;
        writeln!(f, "Half Edges: ")?;
        for i in 0..self.edge_data.len() {
            let u = self.edge_data[i].neighbor;
            let v = self.edge_data[self.edge_data[i].twin].neighbor;
            let next = self.edge_data[i].next;
            let u_next = self.edge_data[next].neighbor;
            let v_next = self.edge_data[self.edge_data[next].twin].neighbor;
            writeln!(f, "    {}: {} -> {}, twin: {}, next: {}: {} -> {}, short_circuit: {}", 
                i, u, v, self.edge_data[i].twin, next, u_next, v_next, self.edge_data[i].short_circuit
            )?;
        }
        writeln!(f, "Vertices: ")?;
        for i in 0..self.dfs_data.len(){
            let vertex = self.dfs_data[i].clone();
            writeln!(f, "    {}: {}: {:?}", i, self.ids[i], vertex)?;
            writeln!(f, "      {:?}", self.embedding[i])?;
        }

        Ok(())
    }
}
/// Simple type to hold planar embeddings. Initially represents a DCEL, but can be expanded to get straight line embeddings
#[derive(Debug, Default, Clone)]
pub struct PlanarEmbedding{
    pub fake_vertices: HashSet<VertexID>,
    // Adjacency list
    pub vertex_adjacency: HashMap<VertexID, HashSet<VertexID>>,
    // Map from vertex to set of half edges originating from it
    pub vertex_map: HashMap<VertexID, HashSet<usize>>,
    /// Map from vertex to group, or usize::MAX if singleton
    pub vertex_group: HashMap<VertexID, usize>,
    // List of halfedges in the embedding
    pub half_edges: Vec<DCELHalfEdge>,
    // Index to halfedge on the face, its length, its group (component), and whether the face is external
    pub faces: HashMap<usize, (usize, usize, usize, bool)>,
    // Set of faces per group(component)
    pub groups: Vec<Vec<usize>>,
    pub unused_id: VertexID
}
impl PlanarEmbedding{
    /// Given a circular adjacency list, calculates the DCEL embedding structure. This is definitely not an optimal solution, but it works
    pub fn from_circular_adjacency_list(list: HashMap<VertexID, Vec<VertexID>>) -> Self{
        // println!("{:?}", list);
        let mut embedding = Self::default();
        embedding.vertex_group = list.keys().map(|v| (*v, usize::MAX)).collect();
        let mut half_edge_index: HashMap<(usize, usize), usize> = HashMap::default();
        for (u, adj_list) in list.iter(){
            if *u >= embedding.unused_id {embedding.unused_id = *u+1;}
            let mut adj_list_iter = adj_list.iter();
            let Some(first_neighbor) = adj_list_iter.next() else {continue;};
            let first_out_index = if let Some(i) = half_edge_index.get(&(*u, *first_neighbor)) {*i} else {
                let half_edge = DCELHalfEdge{endpoints: (*u, *first_neighbor), face_trav: (0, 0), face: usize::MAX, twin: 0};
                half_edge_index.insert((*u, *first_neighbor), embedding.half_edges.len());
                embedding.half_edges.push(half_edge);
                embedding.half_edges.len()-1
            };
            let first_in_index = if let Some(i) = half_edge_index.get(&(*first_neighbor, *u)) {*i} else {
                let half_edge = DCELHalfEdge{endpoints: (*first_neighbor, *u), face_trav: (0, 0), face: usize::MAX, twin: 0};
                half_edge_index.insert((*first_neighbor, *u), embedding.half_edges.len());
                embedding.half_edges.push(half_edge);
                embedding.half_edges.len()-1
            };
            embedding.half_edges[first_in_index].twin = first_out_index;
            embedding.half_edges[first_out_index].twin = first_in_index;
            let mut prev_in_index = first_in_index;
            for v in adj_list_iter{
                let out_index = if let Some(i) = half_edge_index.get(&(*u, *v)) {*i} else {
                    let half_edge = DCELHalfEdge{endpoints: (*u, *v), face_trav: (0, 0), face: usize::MAX, twin: 0};
                    half_edge_index.insert((*u, *v), embedding.half_edges.len());
                    embedding.half_edges.push(half_edge);
                    embedding.half_edges.len()-1
                };
                let in_index = if let Some(i) = half_edge_index.get(&(*v, *u)) {*i} else {
                    let half_edge = DCELHalfEdge{endpoints: (*v, *u), face_trav: (0, 0), face: usize::MAX, twin: 0};
                    half_edge_index.insert((*v, *u), embedding.half_edges.len());
                    embedding.half_edges.push(half_edge);
                    embedding.half_edges.len()-1
                };
                embedding.half_edges[out_index].twin = in_index;
                embedding.half_edges[in_index].twin = out_index;
                // prev_ins next = current out
                embedding.half_edges[prev_in_index].face_trav.1 = out_index;
                embedding.half_edges[out_index].face_trav.0 = prev_in_index;
                // Update pointers
                prev_in_index = in_index;
            }
            // adj list updates
            embedding.half_edges[prev_in_index].face_trav.1 = first_out_index;
            embedding.half_edges[first_out_index].face_trav.0 = prev_in_index;
        }
        // Calculate faces
        let mut face_id = 0;
        for i in 0..embedding.half_edges.len(){
            // If we encounter an undiscovered face, trace it and add it to faces
            if embedding.half_edges[i].face != usize::MAX{continue;}
            let current_face = embedding.faces.len();
            embedding.half_edges[i].face = current_face;
            let mut face_size = 1;
            let mut cur_edge = embedding.half_edges[i].face_trav.1;
            while cur_edge != i {
                face_size += 1;
                embedding.half_edges[cur_edge].face = current_face;
                cur_edge = embedding.half_edges[cur_edge].face_trav.1;
            }
            embedding.faces.insert(face_id, (i, face_size, usize::MAX, false));
            face_id += 1;
        }
        // Determine face groups (connected components)
        let mut component_id = 0;
        for face_idx in 0..embedding.faces.len() {
            // Skip if this face is already assigned to a component
            if embedding.faces[&face_idx].2 != usize::MAX {
                continue;
            }

            // BFS to find all faces in this connected component
            let mut queue = VecDeque::new();
            queue.push_back(face_idx);
            embedding.faces.get_mut(&face_idx).unwrap().2 = component_id;
            embedding.groups.push(vec![face_idx]);

            while let Some(current_face) = queue.pop_front() {
                let start_edge = embedding.faces[&current_face].0;
                let mut edge = start_edge;

                // Traverse all edges in this face
                loop {
                    // Get the twin edge - it belongs to an adjacent face
                    let twin_edge = embedding.half_edges[edge].twin;
                    let adjacent_face = embedding.half_edges[twin_edge].face;

                    // If adjacent face is not yet assigned, assign it to this component
                    if embedding.faces[&adjacent_face].2 == usize::MAX {
                        embedding.faces.get_mut(&adjacent_face).unwrap().2 = component_id;
                        embedding.groups[component_id].push(adjacent_face);
                        queue.push_back(adjacent_face);
                    }

                    // Move to next edge in the face
                    edge = embedding.half_edges[edge].face_trav.1;
                    if edge == start_edge {
                        break;
                    }
                }
            }

            // Move to next component
            component_id += 1;
        }
        // Set largest face in group as external face
        for group in embedding.groups.iter(){
            let mut max = 0; let mut maxdex = 0;
            for face in group.iter() {
                if embedding.faces[face].1 > max {
                    max = embedding.faces[face].1;
                    maxdex = *face;
                }
            }
            embedding.faces.get_mut(&maxdex).unwrap().3 = true;
        }
        // Setup vertex map
        for (i, edge) in embedding.half_edges.iter().enumerate(){
            embedding.vertex_map.entry(edge.endpoints.0).or_default().insert(i);
            embedding.vertex_adjacency.entry(edge.endpoints.0).or_default().insert(edge.endpoints.1);
            let group = embedding.faces[&edge.face].2;
            *embedding.vertex_group.entry(edge.endpoints.0).or_default() = group;
        }
        embedding
    }
    
    /// Adds edge between u and the endpoint of uv.next, ie uv->vw, add uw and wu
    fn add_edge_between(&mut self, uv: usize){
        // Find vertices on face
        let vw = self.half_edges[uv].face_trav.1;
        let w_next = self.half_edges[vw].face_trav.1;
        let u_prev = self.half_edges[uv].face_trav.0;
        let face = self.half_edges[uv].face;
        let new_face = self.faces.len();
        let u = self.half_edges[uv].endpoints.0;
        let w = self.half_edges[vw].endpoints.1;
        // Add edge
        let uw = self.half_edges.len(); let wu = uw+1;
        self.half_edges.push(DCELHalfEdge { endpoints: (u, w), face_trav: (u_prev, w_next), face, twin: wu });
        self.half_edges.push(DCELHalfEdge { endpoints: (w, u), face_trav: (vw, uv), face: new_face, twin: uw });
        // Update vertex map
        self.vertex_map.entry(u).or_default().insert(uw);
        self.vertex_map.entry(w).or_default().insert(wu);
        self.vertex_adjacency.entry(u).or_default().insert(w);
        self.vertex_adjacency.entry(w).or_default().insert(u);
        // update pointers
        self.half_edges[vw].face_trav.1 = wu;
        self.half_edges[uv].face_trav.0 = wu;
        self.half_edges[u_prev].face_trav.1 = uw;
        self.half_edges[w_next].face_trav.0 = uw;
        self.half_edges[uv].face = new_face;
        self.half_edges[vw].face = new_face;
        // add face
        let group = self.faces[&face].2;
        self.faces.insert(new_face, (uv, 3, group, false));
        // set old face length and entry point
        // Original face had N edges, new face has 'length' edges (including out_index)
        // Old face gets: N - (length - 1) original edges + in_index = N - length + 2
        let new_length = self.faces[&face].1 - 1;
        self.faces.get_mut(&face).unwrap().1 = new_length;
        self.faces.get_mut(&face).unwrap().0 = uw;
        self.groups[group].push(new_face);
    }
    
    /// adds a fake vertex to the graph any time edge.face == twin.face, to ensure both are not part of the same face
    pub fn remove_thin_faces(&mut self) {
        // Find edges where face == twin.face
        let mut edge_set: HashSet<usize> = HashSet::default();
        for (i, edge) in self.half_edges.iter().enumerate(){
            if edge_set.contains(&i) {continue;}
            if edge.face == self.half_edges[edge.twin].face {
                edge_set.insert(edge.twin);
            }
        }
        // Add fake vertices to the embeding, turning these lines into triangles
        for edge_uv in edge_set.into_iter(){
            let w = self.unused_id; self.unused_id += 1;
            let (u, v) = self.half_edges[edge_uv].endpoints;
            let uv_prev = self.half_edges[edge_uv].face_trav.0;
            let uv_next = self.half_edges[edge_uv].face_trav.1;
            let face = self.half_edges[edge_uv].face;
            let new_face = self.faces.len();
            // Add four new halfedges edges with the vertex
            let wu = self.half_edges.len();
            let uw = wu+1;
            let wv = uw+1;
            let vw = wv+1;
            self.half_edges.push(DCELHalfEdge { endpoints: (w, u), face_trav: (vw, edge_uv), face: new_face, twin: uw });
            self.half_edges.push(DCELHalfEdge { endpoints: (u, w), face_trav: (uv_prev, wv), face: face, twin: wu });
            self.half_edges.push(DCELHalfEdge { endpoints: (w, v), face_trav: (uw, uv_next), face: face, twin: vw });
            self.half_edges.push(DCELHalfEdge { endpoints: (v, w), face_trav: (edge_uv, wu), face: new_face, twin: wv });
            // Setup face traversals
            self.half_edges[uv_prev].face_trav.1 = uw;
            self.half_edges[uv_next].face_trav.0 = wv;
            self.half_edges[edge_uv].face_trav = (wu, vw);
            // Add new face
            self.half_edges[edge_uv].face = new_face;
            let group = self.faces[&face].2;
            self.faces.insert(new_face, (edge_uv, 3, group, false));
            self.groups[group].push(new_face);
            self.faces.get_mut(&face).unwrap().0 = uw;
            self.faces.get_mut(&face).unwrap().1 += 1;  
            // Setup vertex map
            self.vertex_map.insert(w, HashSet::from([wu, wv]));
            self.vertex_map.get_mut(&u).unwrap().insert(uw);
            self.vertex_map.get_mut(&v).unwrap().insert(vw);
            self.vertex_adjacency.entry(u).or_default().insert(w);
            self.vertex_adjacency.entry(v).or_default().insert(w);
            self.vertex_adjacency.entry(w).or_default().extend([u, v]);
            // add to fake vertices
            self.fake_vertices.insert(w);
            // Setup w's vertex group
            self.vertex_group.insert(w, group);
        }
    }

    /// Adds edges to cut vertices
    pub fn make_biconnected(&mut self) {
        // Remove cut vertices
        // Find vertices on external faces that are visited more than once
        let mut cut_vertices: HashSet<usize> = HashSet::default();
        for (_, (edge, _, _, external)) in self.faces.iter(){
            if !*external {continue;}
            // traverse face, keep track of visited nodes
            let mut visited: HashSet<VertexID> = HashSet::default();
            visited.insert(self.half_edges[*edge].endpoints.0);
            let mut cur = self.half_edges[*edge].face_trav.1;
            while cur != *edge {
                if visited.contains(&self.half_edges[cur].endpoints.0) {
                    cut_vertices.insert(cur);
                } else {
                    visited.insert(self.half_edges[cur].endpoints.0);
                }
                cur = self.half_edges[cur].face_trav.1;
            }
        }
        // For each cut vertex, add an edge to create a triangle, removing the cut vertex
        while !cut_vertices.is_empty() {
            let uv = *cut_vertices.iter().next().unwrap(); cut_vertices.remove(&uv);
            let wu = self.half_edges[uv].face_trav.0;
            let w = self.half_edges[wu].endpoints.0;
            let v = self.half_edges[uv].endpoints.1;
            let face = self.half_edges[uv].face;
            let new_face = self.faces.len();
            let v_next = self.half_edges[uv].face_trav.1;
            let w_prev = self.half_edges[wu].face_trav.0;
            let wv = self.half_edges.len();
            let vw = wv+1;
            self.half_edges.push(DCELHalfEdge { endpoints: (w, v), face_trav: (w_prev, v_next), face, twin: vw });
            self.half_edges.push(DCELHalfEdge { endpoints: (v, w), face_trav: (uv, wu), face: new_face, twin: wv });
            self.half_edges[uv].face_trav.1 = vw;
            self.half_edges[wu].face_trav.0 = vw;
            self.half_edges[w_prev].face_trav.1 = wv;
            self.half_edges[v_next].face_trav.0 = wv;
            self.half_edges[uv].face = new_face;
            self.half_edges[wu].face = new_face;
            // If we enclosed another pertinent external edge, add a equivalent edge to the cut vertices
            if cut_vertices.contains(&wu) {cut_vertices.remove(&wu); cut_vertices.insert(wv);}
            let group = self.faces[&face].2;
            self.faces.insert(new_face, (vw, 3, group, false));
            self.groups[group].push(new_face);
            self.vertex_map.get_mut(&v).unwrap().insert(vw);
            self.vertex_map.get_mut(&w).unwrap().insert(wv);
            self.vertex_adjacency.entry(v).or_default().insert(w);
            self.vertex_adjacency.entry(w).or_default().insert(v);
            // Update other faces length
            self.faces.get_mut(&face).unwrap().1 -= 1;
        }
    }

    /// adds edges to ensure each face is a triangle
    pub fn triangularize(&mut self) {
        // First ensure the graph is biconnected
        self.remove_thin_faces();
        self.make_biconnected();

        // Go over all faces and split them by adding edges when they are not triangles
        let pertinent_faces = self.faces.iter().filter_map(
            |(index, &(_, length, _, _))| {
                if length <= 3 {None}
                else {Some(*index)}
            }
        ).collect::<Vec<usize>>();

        for face in pertinent_faces.into_iter() {
            // Find 3 consecutive points on face where the first and last are not adjacent
            // Create triangle edge between them
            let mut start_edge = self.faces[&face].0;

            let mut cur_edge = start_edge;
            let mut next_edge = self.half_edges[start_edge].face_trav.1;
            let mut i = 0;
            loop{
                i += 1; if i > 20 {panic!();}
                // If first and last not adjacent, merge the edges
                if !self.vertex_adjacency[&self.half_edges[cur_edge].endpoints.0].contains(&self.half_edges[next_edge].endpoints.1) {
                    next_edge = self.half_edges[next_edge].face_trav.1;
                    self.add_edge_between(cur_edge);
                    if cur_edge == start_edge {
                        cur_edge = self.half_edges[next_edge].face_trav.0;
                        start_edge = cur_edge;
                    }else {
                        cur_edge = self.half_edges[next_edge].face_trav.0;
                    }
                    continue;
                }
                // Otherwise just move forward, breaking when we finish
                cur_edge = next_edge;
                next_edge = self.half_edges[next_edge].face_trav.1;
                if cur_edge == start_edge {break;}
            }
        }
    }

    // Iterates the circular adjacency list of vertex
    pub fn iterate_adjacent(&self, vertex: VertexID) -> Vec<VertexID>{
        let Some(edges) = self.vertex_map.get(&vertex) else {return vec![];};
        let Some(start_edge) = edges.iter().next() else {return vec![];};
        let mut adjacent = Vec::with_capacity(edges.len());
        adjacent.push(self.half_edges[*start_edge].endpoints.1);
        let mut cur = self.half_edges[self.half_edges[*start_edge].twin].face_trav.1;
        while cur != *start_edge {
            adjacent.push(self.half_edges[cur].endpoints.1);
            cur = self.half_edges[self.half_edges[cur].twin].face_trav.1;
        }
        adjacent
    }

    // Calculates the canonical order for a maximally planar graph
    pub fn canonical_order(&self) -> HashMap<usize, Vec<VertexID>>{
        let mut orders = HashMap::default();
        for (_, (edge, _, group, external)) in self.faces.iter(){
            // Only run for external faces and once per component
            if !*external || orders.contains_key(group){continue;}
            // Collect vertices in this component
            let vertices: Vec<VertexID> = self.vertex_group.iter()
                .filter(|(_, g)| **g==*group)
                .map(|(&v, _)| v).collect();

            // skip for small graphs
            let n = vertices.len();
            if n <= 3 {orders.insert(*group, vertices); continue;}

            // Order starts with two consecutive external face vtcs
            let (v1, v2) = self.half_edges[*edge].endpoints;
            let vn = self.half_edges[self.half_edges[*edge].face_trav.0].endpoints.0;
            let mut order = vec![0; vertices.len()];
            order[0] = v1; order[1] = v2;
            // Setup chord, out and mark sets
            // Chord: number of chords that prevent a vertex from being chosen (edges between non consecutive external vertices)
            // out: external face list
            // mark: Used to set vertices as already chosen
            let mut chords: HashMap<VertexID, usize> = HashMap::default();
            let mut mark: HashSet<VertexID> = HashSet::default();
            let mut out: Vec<VertexID> = Vec::from([v1, vn, v2]);
            for v in vertices.iter() {chords.insert(*v, 0);}
            // Get rest of ordering
            for k in (3..vertices.len()).rev(){
                //println!("k: {}, out: {:?}, chords: {:?}", k, out, chords.iter().filter(|(_, c)| **c>0).map(|(v, c)| (*v, *c)).collect::<Vec<(VertexID, usize)>>());
                // find external unmarked vtx with no coords
                let (index, &v) = out.iter().enumerate().find(|(_, v)| {
                    **v != v1 && **v != v2 && 
                    chords[*v] == 0
                }).unwrap();
                order[k] = v; mark.insert(v); out.remove(index);
                // set unmarked neighbors of v as part of external face. Update chords
                let neighbors: Vec<VertexID>= self.iterate_adjacent(v)
                    .into_iter().filter(|v| !mark.contains(v)).collect();
                // Order unmarked neighbors by external face
                let wp: VertexID = out[index-1]; let wq: VertexID = out[index];
                let wp_idx = neighbors.iter().position(|n| *n == wp).unwrap();
                let wq_idx = neighbors.iter().position(|n| *n == wq).unwrap();
                let second = if wp_idx.max(wq_idx) - wp_idx.min(wq_idx) > 1 {
                    wp_idx.min(wq_idx)
                } else {wp_idx.max(wq_idx)};
                // Set wq, wp as the endpoints
                let mut ordered_neighbors = neighbors.clone();
                ordered_neighbors.rotate_left(second);
                assert!(ordered_neighbors[0] == wp || ordered_neighbors[0] == wq);
                assert!(*ordered_neighbors.last().unwrap() == wp || *ordered_neighbors.last().unwrap() == wq);
                // Ensure wp is 0
                if ordered_neighbors[0] != wp {
                    ordered_neighbors.reverse();
                }
                for i in 1..(ordered_neighbors.len()-1){
                    // insert into out
                    out.insert(index+i-1, ordered_neighbors[i]);
                    for u in self.iterate_adjacent(ordered_neighbors[i])
                        .into_iter().filter(|n| {
                            *n != ordered_neighbors[i-1] && *n != ordered_neighbors[i+1]
                        })
                    {
                        if out.contains(&u) {
                            *chords.get_mut(&ordered_neighbors[i]).unwrap() += 1;
                            *chords.get_mut(&u).unwrap() += 1;
                        }
                    }
                }
                if ordered_neighbors.len() == 2 {
                    *chords.get_mut(&ordered_neighbors[0]).unwrap() -= 1;
                    *chords.get_mut(&ordered_neighbors[1]).unwrap() -= 1;
                }
            }
            // Trivial last insertion
            order[2] = out[1];
            orders.insert(*group, order);
        }
        orders
    }

    pub fn calculate_euclidean_embedding(&mut self) -> HashMap<VertexID, (f32, f32)>{
        // Ensure the embedding is internally triangular for each group, by adding fake vertices and edges.
        self.triangularize();
        // Calculate canonical orderings per component
        let orders = self.canonical_order();
        // Generate positions per component
        let mut positions = HashMap::with_capacity(self.vertex_adjacency.len());
        // Add singleton vertices first
        let mut group_offset = 0;
        let singletons: Vec<VertexID> = self.vertex_group.iter()
            .filter(|(_, g)| **g == usize::MAX).map(|(v, _)| *v).collect();
        if singletons.len() > 0 {group_offset += 1;}
        let count = singletons.len() as f32;
        for (i, vertex) in singletons.into_iter().enumerate(){
            let theta = i as f32 * TAU/count;
            positions.insert(vertex, (theta.cos()*0.8, theta.sin()*0.8));
        }
        // Run alg on each component
        for (_, canon) in orders.into_iter() {
            // Calculate position offset to ensure groups don't overlap
            let n = group_offset as f32; let k = n.sqrt(); let t = n-k*k;
            let x_offset = t;
            let y_offset = 2.0*k-t;
            group_offset += 1;
            // Calculate vertex positions
            let mut delta_x: HashMap<VertexID, isize> = HashMap::default();
            let mut y: HashMap<VertexID, isize> = HashMap::default();
            let mut left: HashMap<VertexID, VertexID> = HashMap::default();
            let mut right: HashMap<VertexID, VertexID> = HashMap::default();
            // External edge
            let mut out: Vec<VertexID> = vec![canon[0], canon[2], canon[1]];
            // Initial triangle
            delta_x.insert(canon[0], 0);
            y.insert(canon[0], 0);
            delta_x.insert(canon[2], 1);
            y.insert(canon[2], 1);
            delta_x.insert(canon[1], 1);
            y.insert(canon[1], 0);
            right.insert(canon[0], canon[2]);
            right.insert(canon[2], canon[1]);
            // Rest of triangles
            for k in 3..canon.len(){
                //println!("k: {}, out: {:?}\ndelta_x: {:?}\ny: {:?}\nleft: {:?}\nright: {:?}", k, out, delta_x, y, left, right);
                let adj = self.vertex_adjacency.get(&canon[k]).unwrap();
                let mut cur = 0;
                while !adj.contains(&out[cur]) {cur += 1;}
                let wp = cur;
                let wq = if adj.contains(&canon[1]) {out.len()-1} 
                else {
                    while cur+1 < out.len() && adj.contains(&out[cur+1]) {cur += 1;}
                    cur
                };
                //println!("wp: {}, wq: {}", wp, wq);
                *delta_x.get_mut(&out[wp+1]).unwrap() += 1;
                *delta_x.get_mut(&out[wq]).unwrap() += 1;
                let mut delta_sum = 0;
                for i in (wp+1)..=wq {
                    delta_sum += *delta_x.get(&out[i]).unwrap();
                }
                let vk_delta = (delta_sum + y[&out[wq]] - y[&out[wp]]) / 2;
                delta_x.insert(canon[k], vk_delta);
                y.insert(canon[k], (delta_sum + y[&out[wq]] + y[&out[wp]]) / 2);
                right.insert(out[wp], canon[k]);
                if wp+1 != wq {
                    left.insert(canon[k], out[wp+1]);
                }
                right.insert(canon[k], out[wq]);
                if wq-1 != wp {
                    right.remove(&out[wq-1]);
                }
                delta_x.insert(out[wq], delta_sum-vk_delta);
                if wp+1 != wq{
                    *delta_x.get_mut(&out[wp+1]).unwrap() -= vk_delta;
                }
                // Update external edge
                out.insert(wp+1, canon[k]);
                for _ in (wp+1)..wq {
                    out.remove(wp+2);
                }
            }
            // compute x coords
            let mut x: HashMap<VertexID, isize> = HashMap::default();
            x.insert(canon[0], 0);
            Self::accumulate_offset(canon[0], 0, &mut x, &delta_x, &left, &right);
            // Rescale vertices into unit square and shift by group offset, put into positions
            let width = (2*canon.len() - 4) as f32;
            let height = (canon.len() - 2) as f32;
            positions.extend(canon.into_iter().filter_map(|v| {
                if self.fake_vertices.contains(&v) {return None;}
                let x = *x.get(&v).unwrap() as f32;
                let y = *y.get(&v).unwrap() as f32;
                Some((v, (x/width + x_offset, y/height + y_offset)))
            }));
        }
        positions.retain(|v, _| !self.fake_vertices.contains(v));
        positions
    }

    /// Recrusive function to calculate x offsets from delta_x in shift method
    fn accumulate_offset(
        vertex: VertexID, 
        offset: isize, 
        x: &mut HashMap<VertexID, isize>, 
        delta_x: &HashMap<VertexID, isize>,
        left: &HashMap<VertexID, VertexID>, 
        right: &HashMap<VertexID, VertexID>
    ) {
        let cur_offset = offset+delta_x[&vertex];
        x.insert(vertex, cur_offset);
        if let Some(v_l) = left.get(&vertex).cloned() {
            Self::accumulate_offset(v_l, cur_offset, x, delta_x, left, right);
        }
        if let Some(v_r) = right.get(&vertex).cloned() {
            Self::accumulate_offset(v_r, cur_offset, x, delta_x, left, right);
        }
    }
}

/// Descriptor for an edge of a vertices external or canonical edges. Both is used if both edges are the same
pub enum EdgeDescriptor{
    First,
    Second,
    Both
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct DCELHalfEdge{
    // Half edge vertex endpoints
    endpoints: (usize, usize),
    // Next and prev halfedges in face list
    face_trav: (usize, usize),
    // Index in face list
    face: usize,
    // Twin edge
    twin: usize,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct DCELFace{
    // entry halfedge index
    entry: usize,
    // size of the face
    length: usize,
    // whether the face is external
    external: bool,
}

/// Simple type to hold kuratowski subgraph for non planar graphs
#[derive(Debug, Default, Clone)]
pub struct KuratowskiSubgraph{
    pub edge_set: HashSet<EdgeID>,
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

#[cfg(test)]
mod test{
    use std::{collections::HashMap, f32::EPSILON};

    use crate::{algorithms::planarity::GraphPlanarity, graph::{homogenous::HomogenousView, prelude::*}};

    pub fn test_line_crossing<G: GraphTrait>(graph: &G, positions: &HashMap<usize, (f32, f32)>){
        let edges: Vec<EdgeID> = graph.edges().collect();
        for i in 0..edges.len()-1 {
            for j in i+1..edges.len(){
                let (v_a, v_b) = edges[i];
                let (u_a, u_b) = edges[j];
                let (x1, y1) = positions[&v_a];
                let (x2, y2) = positions[&v_b];
                let (x3, y3) = positions[&u_a];
                let (x4, y4) = positions[&u_b];
                let denom = (x1-x2)*(y3-y4) - (y1-y2)*(x3-x4);
                if denom == 0.0 {
                    // Cant intersect lines, unsure non colinear, or non overlapping
                    println!("Edge ({}, {})->({}, {}), and ({}, {})->({}, {}), are vertical", x1, y1, x2, y2, x3, y3, x4, y4);
                    if x1!=x3 {continue;}
                    let (a1, b1) = (y1.min(y2), y1.max(y2));
                    let (a2, b2) = (y3.min(y4), y3.max(y4));
                    let start = a1.max(b1); let end = a2.min(b2);
                    assert!(end-start < EPSILON);
                    continue;
                }
                let x = ((x1*y2-y1*x2)*(x3-x4)-(x1-x2)*(x3*y4-y3*x4)) / denom;
                let y = ((x1*y2-y1*x2)*(y3-y4)-(y1-y2)*(x3*y4-y3*x4)) / denom;
                println!("Edge ({}, {})->({}, {}), and ({}, {})->({}, {}), intersect at ({}, {})", x1, y1, x2, y2, x3, y3, x4, y4, x, y);
                // Insure intersection occurs out of one of the segment bounds
                assert!(
                    x-x1.max(x2)>-10.0*EPSILON || x - x1.min(x2)<10.0*EPSILON ||
                    x-x3.max(x4)>-10.0*EPSILON || x - x3.min(x4)<10.0*EPSILON ||
                    y-y1.max(y2)>-10.0*EPSILON || y - y1.min(y2)<10.0*EPSILON ||
                    y-y3.max(y4)>-10.0*EPSILON || y - y3.min(y4)<10.0*EPSILON
                );
            }
        }
    }

    #[test]
    fn test_planarity(){
        let mut k33 = SparseSimpleGraph::default();
        k33.add_vertex(0); k33.add_vertex(1); k33.add_vertex(2); k33.add_vertex(3); k33.add_vertex(4);
        assert!(GraphPlanarity::from_graph(&k33).compute_planarity());
        k33.add_edge((0, 3));
        k33.add_edge((0, 4));
        k33.add_edge((0, 5));
        k33.add_edge((1, 3));
        k33.add_edge((1, 4));
        k33.add_edge((1, 5));
        k33.add_edge((2, 3));
        k33.add_edge((2, 4));
        assert!(GraphPlanarity::from_graph(&k33).compute_planarity());
        k33.add_edge((2, 5));
        assert!(!GraphPlanarity::from_graph(&k33).compute_planarity());
        let mut k5 = SparseSimpleGraph::default();
        assert!(GraphPlanarity::from_graph(&k5).compute_planarity());
        k5.add_edge((0, 1));
        k5.add_edge((0, 2));
        k5.add_edge((0, 3));
        k5.add_edge((0, 4));
        k5.add_edge((1, 2));
        k5.add_edge((1, 3));
        k5.add_edge((1, 4));
        k5.add_edge((2, 3));
        k5.add_edge((2, 4));
        assert!(GraphPlanarity::from_graph(&k5).compute_planarity());
        k5.add_edge((3, 4));
        assert!(!GraphPlanarity::from_graph(&k5).compute_planarity());
        let mut p5 = SparseSimpleGraph::default();
        p5.add_vertex(0); p5.add_vertex(1); p5.add_vertex(2); p5.add_vertex(3); p5.add_vertex(4);
        p5.add_edge((0, 1));
        p5.add_edge((1, 2));
        p5.add_edge((2, 3));
        p5.add_edge((3, 4));
        assert!(GraphPlanarity::from_graph(&p5).compute_planarity());
    }

    #[test]
    fn test_triangularization(){
        // Create a pentagon
        let mut graph = SparseSimpleGraph::default();
        graph.add_edge((0, 1));
        graph.add_edge((1, 2));
        graph.add_edge((2, 3));
        graph.add_edge((3, 4));
        graph.add_edge((4, 0));

        let mut planarity = GraphPlanarity::from_graph(&graph);
        assert!(planarity.compute_planarity());
        let result = planarity.get_planarity_structure();
        assert!(result.is_ok());

        let mut embedding = result.unwrap();
        println!("Pentagon before triangularization:");
        println!("Faces: {}", embedding.faces.len());
        for (face_id, &(_start, size, _component, is_external)) in embedding.faces.iter() {
            println!("  Face {}: size={}, external={}", face_id, size, is_external);
        }

        // Full triangularization including biconnectivity
        embedding.triangularize();

        println!("\nAfter triangularization:");
        println!("Faces: {}", embedding.faces.len());
        println!("Fake vertices: {}", embedding.fake_vertices.len());

        let mut triangle_count = 0;
        let mut non_triangle_count = 0;
        for (face_id, &(_start, size, _component, is_external)) in embedding.faces.iter() {
            println!("  Face {}: size={}, external={}", face_id, size, is_external);
            if !is_external {
                if size == 3 {
                    triangle_count += 1;
                } else {
                    non_triangle_count += 1;
                }
            }
        }

        println!("Vertices: {}", embedding.vertex_adjacency.len());
        for adj in embedding.vertex_adjacency.iter() {
            print!("  Vertex {}: ", *adj.0);
            for adj in adj.1 {print!("{}, ", *adj);}
            println!()
        }

        println!("Triangle faces: {}, Non-triangle internal faces: {}", triangle_count, non_triangle_count);
        assert_eq!(non_triangle_count, 0, "All internal faces should be triangles");
    }

    #[test]
    fn test_path_triangularization(){
        // Create a pentagon
        let mut graph = SparseSimpleGraph::default();
        graph.add_edge((0, 1));
        graph.add_edge((1, 2));
        graph.add_edge((2, 3));
        graph.add_edge((3, 4));

        let mut planarity = GraphPlanarity::from_graph(&graph);
        assert!(planarity.compute_planarity());
        let result = planarity.get_planarity_structure();
        assert!(result.is_ok());

        let mut embedding = result.unwrap();

        println!("P5 before triangularization:");
        println!("Faces: {}", embedding.faces.len());
        for (face_id, &(_start, size, _component, is_external)) in embedding.faces.iter() {
            println!("  Face {}: size={}, external={}", face_id, size, is_external);
        }

        // Full triangularization including biconnectivity
        embedding.triangularize();

        println!("\nAfter triangularization:");
        println!("Faces: {}", embedding.faces.len());
        println!("Fake vertices: {}", embedding.fake_vertices.len());

        let mut triangle_count = 0;
        let mut non_triangle_count = 0;
        for (face_id, &(start, size, _component, is_external)) in embedding.faces.iter() {
            print!("  Face {}: size={}, external={}", face_id, size, is_external);
            if !is_external {
                if size == 3 {
                    triangle_count += 1;
                } else {
                    non_triangle_count += 1;
                }
            }
            print!(", Vertices: {}", embedding.half_edges[start].endpoints.0);
            let mut cur = embedding.half_edges[start].face_trav.1;
            while cur != start {
                print!(", {}", embedding.half_edges[cur].endpoints.0);
                cur = embedding.half_edges[cur].face_trav.1;
            }
            println!();
        }

        println!("Vertices: {}", embedding.vertex_adjacency.len());
        for adj in embedding.vertex_adjacency.iter() {
            print!("  Vertex {}: ", *adj.0);
            for adj in adj.1 {print!("{}, ", *adj);}
            println!()
        }

        println!("Triangle faces: {}, Non-triangle internal faces: {}", triangle_count, non_triangle_count);
        assert_eq!(non_triangle_count, 0, "All internal faces should be triangles");
    }

    #[test]
    fn test_k5_planarity(){
        // Create a pentagon
        let mut graph = SparseSimpleGraph::default();
        graph.add_edge((0, 1));
        graph.add_edge((0, 2));
        graph.add_edge((0, 3));
        graph.add_edge((0, 4));
        graph.add_edge((1, 2));
        graph.add_edge((1, 3));
        graph.add_edge((1, 4));
        graph.add_edge((2, 3));
        graph.add_edge((2, 4));

        let mut planarity = GraphPlanarity::from_graph(&graph);
        assert!(planarity.compute_planarity());
        let result = planarity.get_planarity_structure();
        assert!(result.is_ok());

        let mut embedding = result.unwrap();

        println!("CircularAdjacency: {:?}", planarity.get_circular_adjacency_list());
        println!("K5-1 before triangularization:");
        println!("Faces: {}", embedding.faces.len());
        for (face_id, &(_start, size, _component, is_external)) in embedding.faces.iter() {
            println!("  Face {}: size={}, external={}", face_id, size, is_external);
        }

        // Full triangularization including biconnectivity
        embedding.triangularize();

        println!("\nAfter triangularization:");
        println!("Faces: {}", embedding.faces.len());
        println!("Fake vertices: {}", embedding.fake_vertices.len());

        let mut triangle_count = 0;
        let mut non_triangle_count = 0;
        for (face_id, &(start, size, _component, is_external)) in embedding.faces.iter() {
            print!("  Face {}: size={}, external={}", face_id, size, is_external);
            if !is_external {
                if size == 3 {
                    triangle_count += 1;
                } else {
                    non_triangle_count += 1;
                }
            }
            print!(", Vertices: {}", embedding.half_edges[start].endpoints.0);
            let mut cur = embedding.half_edges[start].face_trav.1;
            while cur != start {
                print!(", {}", embedding.half_edges[cur].endpoints.0);
                cur = embedding.half_edges[cur].face_trav.1;
            }
            println!();
        }

        println!("Vertices: {}", embedding.vertex_adjacency.len());
        for adj in embedding.vertex_adjacency.iter() {
            print!("  Vertex {}: ", *adj.0);
            for adj in adj.1 {print!("{}, ", *adj);}
            println!()
        }

        println!("Triangle faces: {}, Non-triangle internal faces: {}", triangle_count, non_triangle_count);
        assert_eq!(non_triangle_count, 0, "All internal faces should be triangles");

        let positions = embedding.calculate_euclidean_embedding();
        let edges: Vec<EdgeID> = graph.edges().collect();
        for i in 0..edges.len()-1 {
            for j in i+1..edges.len(){
                let (v_a, v_b) = edges[i];
                let (u_a, u_b) = edges[j];
                let (x1, y1) = positions[&v_a];
                let (x2, y2) = positions[&v_b];
                let (x3, y3) = positions[&u_a];
                let (x4, y4) = positions[&u_b];
                let denom = (x1-x2)*(y3-y4) - (y1-y2)*(x3-x4);
                if denom == 0.0 {
                    // Cant intersect lines, unsure non colinear, or non overlapping
                    println!("Edge ({}, {})->({}, {}), and ({}, {})->({}, {}), are vertical", x1, y1, x2, y2, x3, y3, x4, y4);
                    if x1!=x3 {continue;}
                    let (a1, b1) = (y1.min(y2), y1.max(y2));
                    let (a2, b2) = (y3.min(y4), y3.max(y4));
                    let start = a1.max(b1); let end = a2.min(b2);
                    assert!(end-start < EPSILON);
                    continue;
                }
                let x = ((x1*y2-y1*x2)*(x3-x4)-(x1-x2)*(x3*y4-y3*x4)) / denom;
                let y = ((x1*y2-y1*x2)*(y3-y4)-(y1-y2)*(x3*y4-y3*x4)) / denom;
                println!("Edge ({}, {})->({}, {}), and ({}, {})->({}, {}), intersect at ({}, {})", x1, y1, x2, y2, x3, y3, x4, y4, x, y);
                // Insure intersection occurs out of one of the segment bounds
                assert!(
                    x-x1.max(x2)>-EPSILON || x - x1.min(x2)<EPSILON ||
                    x-x3.max(x4)>-EPSILON || x - x3.min(x4)<EPSILON ||
                    y-y1.max(y2)>-EPSILON || y - y1.min(y2)<EPSILON ||
                    y-y3.max(y4)>-EPSILON || y - y3.min(y4)<EPSILON
                );
            }
        }
    }

    #[test]
    fn test_k23_planarity(){
        let mut k23 = SparseSimpleGraph::default();
        k23.add_edge((0, 2));
        k23.add_edge((0, 3));
        k23.add_edge((0, 4));
        k23.add_edge((1, 2));
        k23.add_edge((1, 3));
        k23.add_edge((1, 4));
        let mut planarity = GraphPlanarity::from_graph(&k23);
        let result = planarity.get_planarity_structure();
        assert!(result.is_ok());
        let mut embedding = result.unwrap();
        let positions = embedding.calculate_euclidean_embedding();
        test_line_crossing(&k23, &positions);
    }

    #[test]
    fn test_band_planarity(){
        for _ in 0..1000{
            let mut graph = SparseSimpleGraph::default();
            graph.add_edge((0, 1));
            graph.add_edge((2, 3));
            graph.add_edge((4, 5));
            graph.add_edge((0, 2));
            graph.add_edge((1, 3));
            graph.add_edge((2, 4));
            graph.add_edge((3, 5));
            graph.add_edge((4, 0));
            graph.add_edge((5, 1));
            graph.add_edge((0, 5));
            let scrambled = HomogenousView::from_graph(&graph);
            let mut planarity = GraphPlanarity::from_graph(&scrambled);
            let result = planarity.get_planarity_structure();
            assert!(result.is_ok());
            let mut embedding = result.unwrap();
            for edge in embedding.half_edges.iter() {
                let (a, b) = edge.endpoints;
                assert!(scrambled.has_edge((a, b)));
            }
            let positions = embedding.calculate_euclidean_embedding();
            test_line_crossing(&graph, &positions);
        }
    }

    #[test]
    fn test_straight_edge_embedding(){
        // Create a pentagon
        let mut graph = SparseSimpleGraph::default();
        graph.add_edge((0, 1));
        graph.add_edge((1, 2));
        graph.add_edge((2, 3));
        graph.add_edge((3, 4));

        let mut planarity = GraphPlanarity::from_graph(&graph);
        assert!(planarity.compute_planarity());
        let result = planarity.get_planarity_structure();
        assert!(result.is_ok());

        let mut embedding = result.unwrap();

        let positions = embedding.calculate_euclidean_embedding();

        test_line_crossing(&graph, &positions);
    }
}
