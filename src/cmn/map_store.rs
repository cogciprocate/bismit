
// use std::clone::Clone;
use std::borrow::Borrow;
use std::fmt::Debug;
use std::hash::{Hash, /*BuildHasherDefault*/};
use std::collections::HashMap;
use std::ops::{Index, IndexMut};
use std::slice::{Iter, IterMut};
// use twox_hash::XxHash;


/// A store of data indexable by usize or key.
///
/// Values cannot be removed.
///
/// This might need a better name :/
///
// TODO: [impl Iterator]: https://doc.rust-lang.org/std/iter/index.html#implementing-iterator
#[derive(Debug, Clone)]
pub struct MapStore<K, V> where K: Eq + Hash {
    values: Vec<V>,
    indices: HashMap<K, usize>,
}

impl<K, V> MapStore<K, V> where K: Eq + Hash + Debug {
    pub fn new() -> MapStore<K, V> {
        MapStore {
            values: Vec::new(),
            indices: HashMap::new(),
        }
    }

    pub fn with_capacity(cap: usize) -> MapStore<K, V> {
        MapStore {
            values: Vec::with_capacity(cap),
            indices: HashMap::with_capacity(cap),
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<&V> {
        self.values.push(value);

        self.indices.insert(key, (self.values.len() - 1))
            .map(move |idx| &self.values[idx])
    }

    pub fn index_of<Q: ?Sized>(&self, key: &Q) -> Option<usize>
            where K: Borrow<Q>, Q: Hash + Eq
    {
        self.indices.get(key).map(|&idx| idx)
    }

    pub fn by_key<Q: ?Sized>(&self, key: &Q) -> Option<&V>
            where K: Borrow<Q>, Q: Hash + Eq
    {
        match self.index_of(key) {
            Some(idx) => self.values.get(idx),
            None => None,
        }
    }

    pub fn by_key_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
            where K: Borrow<Q>, Q: Hash + Eq
    {
        match self.index_of(key) {
            Some(idx) => self.values.get_mut(idx),
            None => None,
        }
    }

    #[inline] pub fn shrink_to_fit(&mut self) {
        self.values.shrink_to_fit();
        self.indices.shrink_to_fit();
    }

    #[inline] pub fn len(&self) -> usize {
        debug_assert!(self.values.len() == self.indices.len());
        self.values.len()
    }

    #[inline] pub fn by_index<'a>(&'a self, idx: usize) -> Option<&'a V> { self.values.get(idx) }
    #[inline] pub fn by_index_mut<'a>(&'a mut self, idx: usize)
        -> Option<&'a mut V> { self.values.get_mut(idx) }
    #[inline] pub fn indices(&self) -> &HashMap<K, usize> { &self.indices }
    // #[inline] pub fn indices_mut(&mut self) -> &mut HashMap<K, usize> { &mut self.indices }
    #[inline] pub fn values(&self) -> &[V] { self.values.as_slice() }
    #[inline] pub fn values_mut(&mut self) -> &mut [V] { self.values.as_mut_slice() }
    #[inline] pub fn iter(&self) -> Iter<V> { self.values.iter() }
    #[inline] pub fn iter_mut(&mut self) -> IterMut<V> { self.values.iter_mut() }
}

impl<K, V> Index<usize> for MapStore<K, V> where K: Eq + Hash + Debug {
    type Output = V;

    #[inline]
    fn index<'a>(&'a self, idx: usize) -> &'a V {
        &(*self.values)[idx]
    }
}

impl<K, V> IndexMut<usize> for MapStore<K, V> where K: Eq + Hash + Debug {
    #[inline]
    fn index_mut<'a>(&'a mut self, idx: usize) -> &'a mut V {
        &mut (*self.values)[idx]
    }
}

