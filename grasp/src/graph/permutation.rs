use std::collections::{HashMap, HashSet};

use crate::graph::{EdgeID, GraphMut, GraphTrait, VertexID, prelude::{DiGraph, SimpleGraph}, set::Set};

/// Graph where each vertex represents a unique permutation of n elements. Vertices are ID'd by their natural embedding, using the from/to natural functions
#[derive(Debug, Clone)]
pub struct PermutationGraph{
    /// Number of elements being permuted
    element_count: usize,
    /// Factorial of element_count, number of permutations of 'element_count' elements
    vertex_count: usize,
    /// Set of edges in the graph
    edges: HashMap<VertexID, HashSet<VertexID>>
}
impl PermutationGraph{
    /// Builds a new permutation graph for 'element_count' elements
    pub fn new(element_count: usize) -> Self{
        Self{vertex_count: factorial(element_count), element_count, edges: HashMap::default()}
    }
}
impl GraphTrait for PermutationGraph{
    fn vertex_count(&self) -> usize {
        self.vertex_count
    }

    fn edge_count(&self) -> usize {
        self.edges.iter().fold(0, |acc, val| acc+val.1.len())/2
    }

    fn has_vertex(&self, v: super::VertexID) -> bool {
        v<self.vertex_count
    }

    fn has_edge(&self, e: EdgeID) -> bool {
        let (u, v) = e;
        self.edges.get(&u).is_some_and(|n| n.contains(&v))
    }

    fn vertices(&self) -> impl Iterator<Item=super::VertexID> {
        0..factorial(self.element_count)
    }

    fn edges(&self) -> impl Iterator<Item=EdgeID> {
        self.edges.iter().map(|(u, n)| {
            n.iter().map(|v| (*u, *v))
        }).flatten()
    }

    fn neighbors(&self, v: super::VertexID) -> impl Set<Item = super::VertexID> {
        self.edges.get(&v)
    }

    fn vertex_set(&self) -> impl Set<Item = super::VertexID> {
        HomogenousSet::new(self.vertex_count)
    }
}
impl GraphMut for PermutationGraph{
    fn create_vertex(&mut self) -> VertexID {
        panic!("Tried to create a new vertex on a permutation graph")
    }

    fn remove_vertex(&mut self, v: VertexID) -> impl Iterator<Item = EdgeID> {
        panic!("Tried to remove a vertex on a permutation graph");
        vec![].into_iter()
    }

    fn try_add_edge(&mut self, edge: EdgeID) -> Result<(), super::prelude::GraphError> {
        let (u, v) = edge;
        if u>self.vertex_count && v>self.vertex_count {Err(super::error::GraphError::NeitherVertexInGraph(u, v))}
        else if u > self.vertex_count {Err(super::error::GraphError::VertexNotInGraph(u))}
        else if v > self.vertex_count {Err(super::error::GraphError::VertexNotInGraph(v))}
        else {
            self.edges.entry(u).or_default().insert(v);
            self.edges.entry(v).or_default().insert(u);
            Ok(())
        }
    }

    fn remove_edge(&mut self, e: EdgeID) -> bool {
        let (u, v) = e;
        if self.edges.get(&u).is_none_or(|n| !n.contains(&v)) {return false;}
        if let Some(n) = self.edges.get_mut(&u) {n.remove(&v);}
        if let Some(n) = self.edges.get_mut(&v) {n.remove(&u);}
        true
    }
}
impl SimpleGraph for PermutationGraph{}

/// Graph where each vertex represents a unique permutation of n elements. Vertices are ID'd by their natural embedding, using the from/to natural functions
#[derive(Debug, Clone)]
pub struct PermutationDiGraph{
    /// Number of elements being permuted
    element_count: usize,
    /// Factorial of element_count, number of permutations of 'element_count' elements
    vertex_count: usize,
    /// Arcs out from key
    out_adjacency: HashMap<usize, HashSet<usize>>,
    /// Arcs in to key
    in_adjacency: HashMap<usize, HashSet<usize>>
}
impl PermutationDiGraph{
    /// Builds a new permutation graph for 'element_count' elements
    pub fn new(element_count: usize) -> Self{
        Self{vertex_count: factorial(element_count), element_count, 
            in_adjacency: HashMap::default(), out_adjacency: HashMap::default()
        }
    }
}
impl GraphTrait for PermutationDiGraph{
    fn vertex_count(&self) -> usize {
        self.vertex_count
    }

    fn edge_count(&self) -> usize {
        self.in_adjacency.iter().fold(0, |acc, val| acc+val.1.len())
    }

    fn has_vertex(&self, v: super::VertexID) -> bool {
        v<self.vertex_count
    }

    fn has_edge(&self, e: EdgeID) -> bool {
        let (u, v) = e;
        self.out_adjacency.get(&u).is_some_and(|n| n.contains(&v))
    }

    fn vertices(&self) -> impl Iterator<Item=super::VertexID> {
        0..factorial(self.element_count)
    }

    fn edges(&self) -> impl Iterator<Item=EdgeID> {
        self.out_adjacency.iter().map(|(u, nbhrs)| {
            nbhrs.iter().map(|v| (*u, *v).into())
        }).flatten()
    }

    fn neighbors(&self, v: super::VertexID) -> impl Set<Item = super::VertexID> {
        self.out_adjacency.get(&v)
    }

    fn vertex_set(&self) -> impl Set<Item = super::VertexID> {
        HomogenousSet::new(self.vertex_count)
    }
}
impl GraphMut for PermutationDiGraph{
    fn create_vertex(&mut self) -> VertexID {
        panic!("Tried to create a new vertex on a permutation graph")
    }

    fn remove_vertex(&mut self, v: VertexID) -> impl Iterator<Item = EdgeID> {
        panic!("Tried to remove a vertex on a permutation graph");
        vec![].into_iter()
    }

    fn try_add_edge(&mut self, edge: EdgeID) -> Result<(), super::prelude::GraphError> {
        let (u, v) = edge;
        if u>self.vertex_count && v>self.vertex_count {Err(super::error::GraphError::NeitherVertexInGraph(u, v))}
        else if u > self.vertex_count {Err(super::error::GraphError::VertexNotInGraph(u))}
        else if v > self.vertex_count {Err(super::error::GraphError::VertexNotInGraph(v))}
        else {
            self.out_adjacency.entry(u).or_default().insert(v);
            self.in_adjacency.entry(v).or_default().insert(u);
            Ok(())
        }
    }

    fn remove_edge(&mut self, e: EdgeID) -> bool {
        let (u, v) = e;
        if self.out_adjacency.get(&u).is_none_or(|n| !n.contains(&v)) {return false;}
        if let Some(n) = self.out_adjacency.get_mut(&u) {n.remove(&v);}
        if let Some(n) = self.in_adjacency.get_mut(&v) {n.remove(&u);}
        true
    }
}
impl DiGraph for PermutationDiGraph{
    fn in_neighbors(&self, v: VertexID) -> impl Set<Item = VertexID> {
        self.in_adjacency.get(&v)
    }
}

pub struct HomogenousSet{
    elements: Vec<VertexID>
}
impl HomogenousSet {
    pub fn new(element_count: usize) -> Self {
        Self {
            elements: (0..element_count).collect()
        }
    }
}
impl Set for HomogenousSet{
    type Item = VertexID;

    fn contains(&self, v: &Self::Item) -> bool {
        *v < self.elements.len()
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a Self::Item> {
        self.elements.iter()
    }

}

/// Given a permutation of 0..elements, produce the lehmer code
pub fn to_lehmer(mut permutation: Vec<usize>, elements: usize) -> Vec<usize> {
    assert!(permutation.len()==elements);
    // count inversions
    for i in 0..(elements-1){
        for j in (i+1)..(elements) {
            if permutation[j]>permutation[i] {permutation[j] -= 1;}
        }
    }
    permutation
}

/// Given a lehmer code of a permutation, recover the permutation
pub fn from_lehmer(mut inversions: Vec<usize>, elements: usize) -> Vec<usize> {
    assert!(inversions.len()==elements);
    // count inversions
    for i in (0..(elements-1)).rev() {
        for j in i+1..elements {
            if inversions[j]>=inversions[i] {
                inversions[j] += 1;
            }
        }
    }
    inversions
}

/// Given a permutation calculate lehmer code and convert to a natural number
pub fn to_natural(mut permutation: Vec<usize>, elements: usize) -> usize{
    assert!(permutation.len()==elements);
    // count inversions
    permutation = to_lehmer(permutation, elements);
    // sum
    let sum = permutation.into_iter().rev().enumerate().map(
        |(i, inversions)| inversions*factorial(i)
    ).sum();
    sum
}

/// Given a natural embedding of the permutation using lehmer code, recover the permutation
pub fn from_natural(mut code: usize, elements: usize) -> Vec<usize>{
    let mut permutation = vec![0; elements];
    // put inversions into permutation
    for i in 1..elements{
        let cur = code%factorial(i);
        println!("{} {} {} {}", i, code, cur, code/factorial(i));
        code = code/factorial(i);
        permutation[elements-1-i] = cur;
    }
    // recreate permutation from inversions
    permutation = from_lehmer(permutation, elements);
    permutation
}

pub fn factorial(n: usize) -> usize{if n == 0 {1} else {(1..=n).product()}}

#[cfg(test)]
mod test{
    use crate::graph::permutation::{from_lehmer, from_natural, to_lehmer, to_natural};

    #[test]
    fn test_lehmer(){
        let permutation = vec![6, 5, 4, 3, 2, 1, 0];
        let lehmer = to_lehmer(permutation.clone(), 7);
        let recovered = from_lehmer(lehmer.clone(), 7);
        let natural = to_natural(permutation.clone(), 7);
        let recovered_natural = from_natural(natural, 7);
        println!("{:?} {:?} {:?} {:?} {:?}", permutation, lehmer, recovered, natural, recovered_natural);
        assert!(permutation == recovered);
        assert!(permutation == recovered_natural);
    }
}