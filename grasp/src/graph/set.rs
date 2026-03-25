use std::{borrow::Cow, collections::HashSet, hash::Hash, iter::Map, marker::PhantomData, ops::Range};

use bimap::BiHashMap;

pub trait Set {
    type Item: Eq+Clone;

    fn contains(&self, v: &Self::Item) -> bool;
    fn iter<'a>(&'a self) -> impl Iterator<Item = Cow<'a, Self::Item>>;

    /// Default O(n) operation using iter.count
    fn len(&self) -> usize {self.iter().count()}
    fn is_empty(&self) -> bool {self.iter().next().is_some()}
    fn set_eq<S: Set<Item = Self::Item>>(&self, other: &S) -> bool where Self: Sized{
        let len = self.len();
        len == other.len() && self.union_with(other).len() == len
    }
    fn is_disjoint<S: Set<Item = Self::Item>>(&self, other: &S) -> bool{
        self.iter().all(|v| !other.contains(&v))
    }

    fn filter(self, filter: impl Fn(&Self, &Self::Item) -> bool) -> impl Set<Item = Self::Item> 
    where Self: Sized{
        SetFilter::new(self, filter)
    }
    fn biject<F, G, B>(self, map_to: F, map_from: G) -> SetBijection<Self, F, G, B> where 
        Self: Sized, Self::Item: Clone, B: Eq+Clone, F: Fn(Cow<Self::Item>)->B, G: Fn(&B) -> Self::Item
    {
        SetBijection::new(self, map_to, map_from)
    }
    fn with_bimap(self, map: &BiHashMap<Self::Item, Self::Item>) -> BiMappedSet<Self> where 
        Self: Sized, Self::Item: Hash
    {
        BiMappedSet::new(self, map)
    }

    fn union_with(self, other: impl Set<Item = Self::Item>) -> impl Set<Item = Self::Item> 
    where Self: Sized{
        SetUnion::new(self, other)
    }
    fn intersection_with(self, other: impl Set<Item = Self::Item>) -> impl Set<Item = Self::Item> 
    where Self: Sized {
        SetIntersection::new(self, other)
    }
    fn difference_with(self, other: impl Set<Item = Self::Item>) -> impl Set<Item = Self::Item> 
    where Self: Sized {
        SetDifference::new(self, other)
    }

    fn union(self, other: Self) -> impl Set<Item = Self::Item> where Self: Sized{self.union_with(other)}
    fn intersection(self, other: Self) -> impl Set<Item = Self::Item> where Self: Sized{self.intersection_with(other)}
    fn difference(self, other: Self) -> impl Set<Item = Self::Item> where Self: Sized{self.difference_with(other)}
}
impl<'a, S: Set> Set for &'a S{
    type Item = S::Item;
    fn contains(&self, v: &Self::Item) -> bool {
        (*self).contains(v)
    }
    fn len(&self) -> usize {
        (*self).len()
    }
    fn iter<'b>(&'b self) -> impl Iterator<Item = Cow<'b, Self::Item>> {
        (**self).iter()
    }
    fn is_empty(&self) -> bool {
        (*self).is_empty()
    }
    fn set_eq<V: Set<Item = Self::Item>>(&self, other: &V) -> bool {
        (*self).set_eq(other)
    }
    fn is_disjoint<V: Set<Item = Self::Item>>(&self, other: &V) -> bool {
        (*self).is_disjoint(other)
    }
}
impl<S: Set> Set for Option<S>{
    type Item = S::Item;
    fn contains(&self, v: &Self::Item) -> bool {
        self.as_ref().is_some_and(|s| s.contains(v))
    }
    fn iter<'a>(&'a self) -> impl Iterator<Item = Cow<'a, Self::Item>> {
        self.iter().map(|s| s.iter()).flatten()
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
    fn iter(&self) -> impl Iterator<Item = Cow<'_, Self::Item>> {
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
    fn iter(&self) -> impl Iterator<Item = Cow<'_, Self::Item>> {
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
    fn iter(&self) -> impl Iterator<Item = Cow<'_, Self::Item>> {
        self.set.iter().filter(|v| !self.sub.contains(v))
    }
}

pub struct SetFilter<A, F>{
    set: A,
    filter: F
}
impl<A, F> SetFilter<A, F>{
    pub fn new(set: A, filter: F) -> Self{Self{set, filter}}
}
impl<A, F> Set for SetFilter<A, F> where A: Set, F: Fn(&A, &A::Item) -> bool{
    type Item = A::Item;
    fn contains(&self, v: &Self::Item) -> bool {
        (self.filter)(&self.set, v)
    }
    fn iter<'a>(&'a self) -> impl Iterator<Item = Cow<'a, Self::Item>> {
        self.set.iter().filter(|v| (self.filter)(&self.set, v))
    }
}

pub struct SetBijection<A, F, G, B>{
    set: A,
    map_to: F,
    map_from: G,
    _out: PhantomData<B>
}
impl<A, F, G, B> SetBijection<A, F, G, B>{
    pub fn new(set: A, map_to: F, map_from: G) -> Self{Self{set, map_from, map_to, _out: PhantomData::default()}}
}
impl<A, F, G, B> Set for SetBijection<A, F, G, B> 
where 
    A: Set, B: Eq+Clone, A::Item: Clone,
    F: Fn(Cow<A::Item>)->B, G: Fn(&B) -> A::Item
{
    type Item = B;
    fn contains(&self, v: &B) -> bool {
        self.set.contains(&(self.map_from)(v))
    }
    fn iter<'a>(&'a self) -> impl Iterator<Item = Cow<'a, Self::Item>> {
        self.set.iter().map(|a| Cow::Owned((self.map_to)(a)))
    }
}

pub struct BiMappedSet<'a, S: Set>{
    set: S,
    /// Map from set item to mapped item
    map: &'a BiHashMap<S::Item, S::Item>,
}
impl<'a, S: Set> BiMappedSet<'a, S>{
    pub fn new(set: S, map: &'a BiHashMap<S::Item, S::Item>) -> Self{Self{set, map}}
}
impl<'m, S: Set> Set for BiMappedSet<'m, S>
where S::Item: Hash,
{
    type Item = S::Item;
    fn contains(&self, v: &Self::Item) -> bool {
        let Some(item) = self.map.get_by_right(v) else {return false;};
        self.set.contains(item)
    }
    fn iter<'a>(&'a self) -> impl Iterator<Item = Cow<'a, Self::Item>> {
        self.set.iter().filter_map(|v| self.map.get_by_left(&v)).cow_borrowed()
    }
}

/*
    Default Implementations
*/

/// Extension trait for mapping iterators to yield `Cow` values
pub trait IteratorAsCow: Iterator + Sized {
    /// Map an iterator of owned values `T` to yield `Cow::Owned<T>`
    fn cow_owned(self) -> Map<Self, fn(Self::Item) -> Cow<'static, Self::Item>>
    where Self::Item: Clone {
        self.map(Cow::Owned)
    }

    /// Map an iterator of references `&'a T` to yield `Cow::Borrowed<&'a T>`
    fn cow_borrowed<'a, T>(self) -> Map<Self, fn(&'a T) -> Cow<'a, T>>
    where Self: Iterator<Item = &'a T>, T: 'a+Clone {
        self.map(Cow::Borrowed)
    }
}
impl<I: Iterator> IteratorAsCow for I {}
/// Extension trait for turning cow iterators into iterators of underlying type
pub trait CowIteratorAsCloned<T>: Iterator + Sized{
    fn clone_cow(self) -> Map<Self, fn(Self::Item)->T>;
}
impl<'a, V: Clone, I> CowIteratorAsCloned<V> for I where I: Iterator<Item = Cow<'a, V>>, V: 'a{
    fn clone_cow(self) -> Map<Self, fn(Self::Item)->V> {
        self.map(|cow| cow.into_owned())
    }
}

impl<V: Eq+Hash+Clone> Set for HashSet<V>{
    type Item = V;
    fn contains(&self, v: &Self::Item) -> bool {
        HashSet::contains(self, v)
    }
    fn iter(&self) -> impl Iterator<Item = Cow<'_, V>> {
        HashSet::iter(self).cow_borrowed()
    }
    fn len(&self) -> usize {
        HashSet::len(self)
    }
    fn is_empty(&self) -> bool {
        HashSet::is_empty(self)
    }
    fn set_eq<S: Set<Item = Self::Item>>(&self, other: &S) -> bool {
        self.len() == other.len() &&
        self.iter().all(|v| other.contains(v))
    }
}

impl<V: Eq+Clone, const U: usize> Set for [V; U] where{
    type Item = V;
    fn contains(&self, v: &Self::Item) -> bool {for i in self {if i==v {return true;}}return false;}
    fn iter(&self) -> impl Iterator<Item = Cow<'_, V>> {
        <[V]>::iter(self).cow_borrowed()
    }
}

/// Homogenous set of items in a range
impl<Idx: Eq+Clone+PartialOrd+'static> Set for Range<Idx> where Range<Idx>: Iterator<Item = Idx>{
    type Item = Idx;
    fn contains(&self, v: &Self::Item) -> bool {
        *v >= self.start && *v < self.end
    }
    fn iter<'a>(&'a self) -> impl Iterator<Item = Cow<'a, Self::Item>> {
        self.clone().cow_owned()
    }
}

/// Empty set
pub struct EmptySet<V>(PhantomData<V>);
impl<V> Default for EmptySet<V>{fn default() -> Self {Self(PhantomData::default())}}
impl<V: Eq+Clone> Set for EmptySet<V>{
    type Item = V;
    fn contains(&self, _: &Self::Item) -> bool {false}
    fn iter(&self) -> impl Iterator<Item = Cow<'_, Self::Item>> {[].iter()}
}
