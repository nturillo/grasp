use crate::graph::{EdgeID, GraphTrait, VertexID, set::Set};

/// Tag Trait Used to represent the promise that edge ab=ba
pub trait SimpleGraph: GraphTrait{}
/// Trait Used to represent the promise that edge ab!=ba
pub trait DiGraph: GraphTrait{
    /// Set of vertices that have arcs going to the specified vertex.
    fn in_neighbors(&self, v: VertexID) -> impl Set<Item = VertexID>;
    /// Set of vertices that have arcs coming from the specified vertex
    fn out_neighbors(&self, v: VertexID) -> impl Set<Item = VertexID>{self.neighbors(v)}
    /// Set of vertices that have arcs coming from the specified vertex or to.
    fn all_neighbors(&self, v: VertexID) -> impl Set<Item = VertexID> {
        self.neighbors(v).union_with(self.in_neighbors(v))
    }
}
/// Trait that allows DiGraphs to be converted into SimpleGraphs
pub trait DigraphProjection: DiGraph{
    /// Gets a simple graph with a~b whenever a~b or b~a in self.
    fn as_simple<'b>(&'b self) -> impl SimpleGraph;
    /// gets a simple graph with a~b whenever a~b and b~a in self.
    fn as_underlying<'b>(&'b self) -> impl SimpleGraph;
}

pub struct SimpleView<'a, G: GraphTrait>{
    graph: &'a G
}
impl<'a, G: GraphTrait> From<&'a G> for SimpleView<'a, G>{
    fn from(graph: &'a G) -> Self {
        Self{graph}
    }
}
impl<'a, G: DiGraph> GraphTrait for SimpleView<'a, G>{
    fn vertex_count(&self) -> usize {self.graph.vertex_count()}
    fn has_vertex(&self, v: VertexID) -> bool {self.graph.has_vertex(v)}
    fn vertices(&self) -> impl Iterator<Item=VertexID> {self.graph.vertices()}
    fn vertex_set(&self) -> impl Set<Item = VertexID> {self.graph.vertex_set()}
    fn is_empty(&self) -> bool {self.graph.is_empty()}

    // Non default Implementations
    fn neighbors(&self, v: VertexID) -> impl Set<Item = VertexID> {
        self.graph.all_neighbors(v)
    }
    fn has_edge(&self, e: EdgeID) -> bool {
        let (v1, v2) = e;
        self.graph.has_edge((v1, v2)) || self.graph.has_edge((v2, v1))
    }
    fn edges(&self) -> impl Iterator<Item=EdgeID> {
        self.graph.edges()
        .filter(|(v1, v2)| {
            // Keep if only one direction exists, or if u <= v when both exist
            !self.graph.has_edge((*v2, *v1)) || v1 <= v2
        })
    }
    fn edge_count(&self) -> usize {
        self.edges().count()
    }
}
impl<'a, G: DiGraph> SimpleGraph for SimpleView<'a, G>{}

pub struct UnderlyingView<'a, G: GraphTrait>{
    graph: &'a G
}
impl<'a, G: GraphTrait> From<&'a G> for UnderlyingView<'a, G>{
    fn from(graph: &'a G) -> Self {
        Self{graph}
    }
}
impl<'a, G: DiGraph> GraphTrait for UnderlyingView<'a, G>{
    fn vertex_count(&self) -> usize {self.graph.vertex_count()}
    fn has_vertex(&self, v: VertexID) -> bool {self.graph.has_vertex(v)}
    fn vertices(&self) -> impl Iterator<Item=VertexID> {self.graph.vertices()}
    fn vertex_set(&self) -> impl Set<Item = VertexID> {self.graph.vertex_set()}
    fn is_empty(&self) -> bool {self.graph.is_empty()}

    // Non default Implementations
    fn neighbors(&self, v: VertexID) -> impl Set<Item = VertexID> {
        self.graph.neighbors(v).filter(move |_, v2| self.graph.has_edge((*v2, v)))
    }
    fn has_edge(&self, (v1, v2): EdgeID) -> bool {
        self.graph.has_edge((v1, v2)) && self.graph.has_edge((v2, v1))
    }
    fn edges(&self) -> impl Iterator<Item=EdgeID> {
        self.graph.edges()
        .filter(|(v1, v2)| {
            // Keep if both directions exist and u <= v
            v1 <= v2 && self.graph.has_edge((*v2, *v1))
        })
    }
    fn edge_count(&self) -> usize {
        self.edges().count()
    }
}
impl<'a, G: DiGraph> SimpleGraph for UnderlyingView<'a, G>{}
