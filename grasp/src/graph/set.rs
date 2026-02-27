use std::{collections::HashSet, ops::{Deref, DerefMut}, usize};
use crate::graph::VertexID;

pub trait Set {
    type Item;

    fn contains(&self, v: &Self::Item) -> bool;
    fn iter(&self) -> impl Iterator<Item = &Self::Item>;

    fn count(&self) -> usize {self.iter().count()}

    fn self_union(self, other: Self) -> impl Set<Item = Self::Item> 
    where Self: Sized{
        SetUnion::new(self, other)
    }
    fn self_intersection(self, other: Self) -> impl Set<Item = Self::Item> 
    where Self: Sized{
        SetIntersection::new(self, other)
    }
    fn self_difference(self, other: Self) -> impl Set<Item = Self::Item> 
    where Self: Sized{
        SetDifference::new(self, other)
    }

    fn union(self, other: impl IntoSet<Item = Self::Item>) -> impl Set<Item = Self::Item> 
    where Self: Sized {
        SetUnion::new(self, other.into_set())
    }
    fn intersection(self, other: impl IntoSet<Item = Self::Item>) -> impl Set<Item = Self::Item> 
    where Self: Sized {
        SetIntersection::new(self, other.into_set())
    }
    fn difference(self, other: impl IntoSet<Item = Self::Item>) -> impl Set<Item = Self::Item> 
    where Self: Sized {
        SetDifference::new(self, other.into_set())
    }
}
impl<'a, S: Set> Set for &'a S{
    type Item = S::Item;
    fn contains(&self, v: &Self::Item) -> bool {
        S::contains(self, v)
    }
    fn iter(&self) -> impl Iterator<Item = &Self::Item> {
        S::iter(self)
    }
}

pub trait IntoSet{
    type Item;
    type IntoSetType: Set<Item = Self::Item>;
    fn into_set(self) -> Self::IntoSetType;
}
impl<S: Set> IntoSet for S{
    type Item = S::Item;
    type IntoSetType = S;
    #[inline]
    fn into_set(self) -> Self::IntoSetType {
        self
    }
}

pub struct SetUnion<A, B>{
    set_a: A,
    set_b: B
}
impl<A, B> SetUnion<A, B>{
    pub fn new(set_a: A, set_b: B) -> Self{Self{set_a, set_b}}
}
impl<A, B> Set for SetUnion<A, B> where A: Set, B: Set<Item = A::Item>{
    type Item = A::Item;
    fn contains(&self, v: &Self::Item) -> bool {
        self.set_a.contains(v) || self.set_b.contains(v)
    }
    fn iter(&self) -> impl Iterator<Item = &Self::Item> {
        self.set_a.iter().chain(self.set_b.iter().filter(|v| !self.set_a.contains(v)))
    }
}

pub struct SetIntersection<A, B>{
    set_a: A,
    set_b: B
}
impl<A, B> SetIntersection<A, B>{
    pub fn new(set_a: A, set_b: B) -> Self{Self{set_a, set_b}}
}
impl<A, B> Set for SetIntersection<A, B> where A: Set, B: Set<Item = A::Item>{
    type Item = A::Item;
    fn contains(&self, v: &Self::Item) -> bool {
        self.set_a.contains(v) && self.set_b.contains(v)
    }
    fn iter(&self) -> impl Iterator<Item = &Self::Item> {
        self.set_a.iter().filter(|v| self.set_b.contains(v))
    }
}

pub struct SetDifference<A, B>{
    set: A,
    sub: B
}
impl<A, B> SetDifference<A, B>{
    pub fn new(set: A, sub: B) -> Self{Self{set, sub}}
}
impl<A, B> Set for SetDifference<A, B> where A: Set, B: Set<Item = A::Item>{
    type Item = A::Item;
    fn contains(&self, v: &Self::Item) -> bool {
        self.set.contains(v) && !self.sub.contains(v)
    }
    fn iter(&self) -> impl Iterator<Item = &Self::Item> {
        self.set.iter().filter(|v| !self.sub.contains(v))
    }
}

/*
    Helper Struct for Equality
*/

pub struct VertexSet<S: Set<Item = VertexID>>{
    set: S
}
impl<S: Set<Item = VertexID>> Deref for VertexSet<S>{
    type Target = S;
    fn deref(&self) -> &Self::Target {
        &self.set
    }
}
impl<S: Set<Item = VertexID>> DerefMut for VertexSet<S>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.set
    }
}
impl<S: Set<Item = VertexID>> From<S> for VertexSet<S>{
    fn from(value: S) -> Self {
        Self{set: value}
    }
}
impl<S1, S2> PartialEq<VertexSet<S2>> for VertexSet<S1> where S1: Set<Item = VertexID>, S2: Set<Item = VertexID>{
    fn eq(&self, other: &VertexSet<S2>) -> bool {
        self.iter().all(|v| other.contains(v)) &&
        other.iter().all(|v| self.contains(v))
    }
}

/*
    Default Implementations
*/

impl Set for HashSet<VertexID>{
    type Item = VertexID;
    fn contains(&self, v: &Self::Item) -> bool {
        HashSet::contains(self, v)
    }
    fn iter(&self) -> impl Iterator<Item = &Self::Item> {
        HashSet::iter(self)
    }
    fn count(&self) -> usize {
        HashSet::len(self)
    }
}
impl<'a> Set for HashSet<&'a VertexID>{
    type Item = VertexID;
    fn contains(&self, v: &Self::Item) -> bool {
        HashSet::contains(self, v)
    }
    fn count(&self) -> usize {
        HashSet::len(self)
    }
    fn iter(&self) -> impl Iterator<Item = &Self::Item> {
        HashSet::iter(self).map(|v| *v)
    }
}
impl<V: PartialEq, const U: usize> Set for [V; U] where{
    type Item = V;
    fn contains(&self, v: &Self::Item) -> bool {for i in self {if i==v {return true;}}return false;}
    fn count(&self) -> usize {U}
    fn iter(&self) -> impl Iterator<Item = &Self::Item> {IntoIterator::into_iter([])}
}
impl Set for (){
    type Item = VertexID;
    fn contains(&self, _: &Self::Item) -> bool {false}
    fn count(&self) -> usize {0}
    fn iter(&self) -> impl Iterator<Item = &Self::Item> {[].iter()}
}
