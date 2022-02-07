use std::{
    borrow::Borrow,
    collections::{hash_map::RandomState, HashMap},
    hash::{BuildHasher, Hash, Hasher},
};

use slab::Slab;

/// A [`HashSlab`] is a collection of deduplicated values that can be quickly
/// accessed using integer-like keys.
#[derive(Default, Clone)]
pub struct HashSlab<V, K, S = RandomState> {
    /// A flat array of `V` values, indexed by the key type `K`.
    values: Slab<V>,

    /// A mapping from hash of `V` to the key type `K`.
    keys: HashMap<u64, K, S>,
}

impl<V, K, S> HashSlab<V, K, S>
where
    V: Eq + Hash,
    K: Copy + From<usize> + Into<usize>,
    S: BuildHasher,
{
    /// Inserts a value into the hash slab and returns a key that can be used to
    /// retrieve the value in later calls to [`get()`].
    ///
    /// If the slab already contained
    ///
    /// Once inserted, values cannot be modified.
    #[inline]
    pub fn insert(&mut self, value: V) -> K {
        let hash = self.get_hash(&value);

        let index_in_slab = self.values.insert(value);

        let key = index_in_slab.into();

        self.keys.insert(hash, key);

        key
    }

    /// Returns a reference to the value corresponding to the key.
    #[inline]
    pub fn get(&self, key: K) -> Option<&V> {
        self.values.get(key.into())
    }

    /// Returns the key corresponding to the value with the same hash as
    /// `value`.
    ///
    /// The key may be any borrowed form of the hash slab's value type, but
    /// [`Hash`] and [`Eq`] on the borrowed form must match those for the value
    /// type.
    #[inline]
    pub fn get_key<Q>(&self, value: &Q) -> Option<K>
    where
        V: Borrow<Q>,
        Q: Eq + Hash,
    {
        let hash = self.get_hash(value);
        self.keys.get(&hash).copied()
    }

    #[inline]
    fn get_hash<Q>(&self, value: &Q) -> u64
    where
        Q: Hash,
    {
        let mut hasher = self.keys.hasher().build_hasher();
        value.hash(&mut hasher);
        hasher.finish()
    }
}
