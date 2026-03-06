use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Attribute};

/// Returns true if the item has `#[graph_ops(labeled)]` on it.
fn has_labeled_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| 
        attr.path().is_ident("graph_ops") && 
        attr.parse_args::<syn::Ident>().is_ok_and(|id| id == "labeled")
    )
}

/// Derives GraphOps, switching implementation on prescence of labeled tag`
#[proc_macro_derive(GraphOps, attributes(graph_ops))]
pub fn derive_graph_ops(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let labeled = has_labeled_attr(&input.attrs);
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    
    let self_bound = if labeled {
        quote!{
            Self: ArbitraryIDGraph+BuildableGraph+Sized+LabeledGraphMut, 
            <Self as LabeledGraph>::EdgeData: Clone, 
            <Self as LabeledGraph>::VertexData: Clone
        }
    } else {
        quote!{Self: ArbitraryIDGraph+BuildableGraph+Sized}
    };

    let where_clause = match where_clause{
        Some(wc) => quote!{#wc, #self_bound,},
        None => quote!{where #self_bound}
    };

    let body = if labeled {
        quote! {
            fn subgraph_vertex(&self, vertices: impl IntoIterator<Item=VertexID>) -> Self {
                let mut sub = Self::empty();
                crate::graph::graph_ops::build_subgraph_vertex_labeled(self, vertices, &mut sub);
                sub
            }
            fn subgraph_edges(&self, edges: impl IntoIterator<Item=EdgeID>) -> Self {
                let mut sub = Self::empty();
                crate::graph::graph_ops::build_subgraph_edges_labeled(self, edges, &mut sub);
                sub
            }
            fn merge(&self, other: &Self) -> (Self, VertexMap, VertexMap) {
                let mut merged = Self::with_capacity(self.vertex_count()+other.vertex_count(), self.edge_count()+other.edge_count());
                let (m1, m2) = crate::graph::graph_ops::build_merge_labeled(self, other, &mut merged);
                (merged, m1, m2)
            }
            fn complement(&self) -> Self {
                let mut comp = Self::with_capacity(self.vertex_count(), 0);
                crate::graph::graph_ops::build_complement_labeled(self, &mut comp);
                comp
            }
        }
    } else {
        quote! {
            fn subgraph_vertex(&self, vertices: impl IntoIterator<Item=VertexID>) -> Self {
                let mut sub = Self::empty();
                crate::graph::graph_ops::build_subgraph_vertex(self, vertices, &mut sub);
                sub
            }
            fn subgraph_edges(&self, edges: impl IntoIterator<Item=EdgeID>) -> Self {
                let mut sub = Self::empty();
                crate::graph::graph_ops::build_subgraph_edges(self, edges, &mut sub);
                sub
            }
            fn merge(&self, other: &Self) -> (Self, VertexMap, VertexMap) {
                let mut merged = Self::with_capacity(self.vertex_count()+other.vertex_count(), self.edge_count()+other.edge_count());
                let (m1, m2) = crate::graph::graph_ops::build_merge(self, other, &mut merged);
                (merged, m1, m2)
            }
            fn complement(&self) -> Self {
                let mut comp = Self::with_capacity(self.vertex_count(), 0);
                crate::graph::graph_ops::build_complement(self, &mut comp);
                comp
            }
        }
    };

    let expanded = quote! {
        impl #impl_generics GraphOps for #name #ty_generics #where_clause {
            #body
        }
    };

    expanded.into()
}

/// Implements SimpleGraphOps, switching implementation on labeled tag
#[proc_macro_derive(SimpleGraphOps, attributes(graph_ops))]
pub fn derive_simple_graph_ops(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let labeled = has_labeled_attr(&input.attrs);
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let self_bound = if labeled {
        quote!{
            Self: GraphOps+SimpleGraph+LabeledGraphMut+ArbitraryIDGraph+BuildableGraph,
            <Self as LabeledGraph>::EdgeData: Clone, 
            <Self as LabeledGraph>::VertexData: Clone
        }
    } else {
        quote!{Self: GraphOps+SimpleGraph+ArbitraryIDGraph+BuildableGraph}
    };

    let where_clause = match where_clause{
        Some(wc) => quote!{#wc, #self_bound,},
        None => quote!{where #self_bound}
    };

    let body = if labeled {
        quote! {
            fn join(&self, other: &Self) -> (Self, VertexMap, VertexMap) {
                let mut joined = Self::with_capacity(self.vertex_count()+other.vertex_count(), self.edge_count()+other.edge_count()+self.vertex_count()*other.vertex_count());
                let (m1, m2) = crate::graph::graph_ops::build_join_labeled(self, other, &mut joined);
                (joined, m1, m2)
            }
            fn product(&self, other: &Self) -> (Self, std::collections::HashMap<(VertexID, VertexID), VertexID>) {
                let mut prod = Self::with_capacity(self.vertex_count()*other.vertex_count(), self.edge_count()*other.vertex_count()+self.vertex_count()+other.edge_count());
                let map = crate::graph::graph_ops::build_product_labeled(self, other, &mut prod);
                (prod, map)
            }
        }
    } else {
        quote! {
            fn join(&self, other: &Self) -> (Self, VertexMap, VertexMap) {
                let mut joined = Self::with_capacity(self.vertex_count()+other.vertex_count(), self.edge_count()+other.edge_count()+self.vertex_count()*other.vertex_count());
                let (m1, m2) = crate::graph::graph_ops::build_join(self, other, &mut joined);
                (joined, m1, m2)
            }
            fn product(&self, other: &Self) -> (Self, std::collections::HashMap<(VertexID, VertexID), VertexID>) {
                let mut prod = Self::with_capacity(self.vertex_count()*other.vertex_count(), self.edge_count()*other.vertex_count()+self.vertex_count()+other.edge_count());
                let map = crate::graph::graph_ops::build_product(self, other, &mut prod);
                (prod, map)
            }
        }
    };

    let expanded = quote! {
        impl #impl_generics SimpleGraphOps for #name #ty_generics #where_clause {
            #body
        }
    };

    expanded.into()
}