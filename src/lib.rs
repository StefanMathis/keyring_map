/*!
[`HashMap`]: std::collections::HashMap
[`IndexMap`]: indexmap::IndexMap
[`Slab`]: slab::Slab
[`KeyringMap`]: crate::KeyringMap
[`KeyringMap::aliases`]: crate::KeyringMap::aliases
[`KeyringMap::iter_unique`]: crate::KeyringMap::iter_unique
[`KeyringMap::iter_unique_mut`]: crate::KeyringMap::iter_unique_mut
[`KeyringMap::iter_full`]: crate::KeyringMap::iter_full

A hash map with multiple keys per value using a ring structure to minimize allocations.

 */
#![doc = include_str!("../docs/main.md")]
#![deny(missing_docs)]

use std::{
    borrow::Borrow,
    hash::{BuildHasher, Hash, RandomState},
    result::Result,
};

use indexmap::IndexMap;
use slab::Slab;

#[cfg(feature = "serde")]
mod serde_impl;

#[cfg(feature = "rayon")]
mod rayon_impl;

/**
An hash map based on [`IndexMap`] with multiple keys / indices per value.

The [`KeyringMap`] combines an [`IndexMap`] and a [`Slab`] to create a hash map
where a single value can be accessed by multiple keys or indices. Its name is
derived from the used implementation strategy, see the last section of this
docstring. Thanks to that strategy, no other allocations between that of the
[`IndexMap`] and the [`Slab`] are required, ensuring high performance and
cache-friendliness.

# Creating a map, accessing and modifying it

```
use keyring_map::KeyringMap;

let mut map = KeyringMap::new();

// Insert one value with three keys
map.insert_many(vec!['a', 'b', 'c'], "abc").expect("none of the keys exist already");

assert_eq!(map.num_keys(), 3);
assert_eq!(map.num_values(), 1);

// Access the same value using different keys
let va = map.get(&'a').expect("has key");
let vb = map.get(&'b').expect("has key");
assert!(std::ptr::eq(va, vb));

// Since the KeyringMap is powered by an IndexMap, it is also possible to use
// indices for accessing values:
let ia = map.get_index_of(&'a').expect("has key");
let ic = map.get_index_of(&'c').expect("has key");

let (_, va) = map.get_index(ia).expect("has index");
let (_, vb) = map.get_index(ic).expect("has index");
assert!(std::ptr::eq(va, vb));
```

It is also possible to insert the keys incrementally:

```
use keyring_map::KeyringMap;

let mut map = KeyringMap::new();

// Insert a value with a single key
map.insert('a', "abc");

assert_eq!(map.num_keys(), 1);
assert_eq!(map.num_values(), 1);

// Now add an alias
map.alias(&'a', 'b');

assert_eq!(map.num_keys(), 2);
assert_eq!(map.num_values(), 1);

let va = map.get(&'a').expect("has key");
let vb = map.get(&'b').expect("has key");
assert!(std::ptr::eq(va, vb));

// Add many aliases at once
map.alias_many(&'b', vec!['c', 'd', 'e']);

assert_eq!(map.num_keys(), 5);
assert_eq!(map.num_values(), 1);
```

If a value is removed, all its keys are removed as well. However, when removing
a key, the value is only removed if there is no other key for it:

```
use keyring_map::KeyringMap;

let mut map = KeyringMap::new();

map.insert_many(vec!['a', 'b', 'c'], "abc").expect("none of the keys exist already");
map.insert('d', "def");

assert_eq!(map.num_keys(), 4);
assert_eq!(map.num_values(), 2);

// The value is kept in the map, because there are other keys for it.
assert_eq!(map.remove_key(&'b'), None);

assert_eq!(map.num_keys(), 3);
assert_eq!(map.num_values(), 2);

// This time, the value is removed as well because there is only one key for it
assert_eq!(map.remove_key(&'d'), Some("def"));

assert_eq!(map.num_keys(), 2);
assert_eq!(map.num_values(), 1);

// Remove the value and with it all keys
assert_eq!(map.remove(&'c'), Some("abc"));

assert_eq!(map.num_keys(), 0);
assert_eq!(map.num_values(), 0);
```

# Iteration

## Serial iteration

[`KeyringMap`] offers iterators for the keys, indices and values (and their
pairs). It is also possible to iterate over each value once while returning one
of their keys with them ([`KeyringMap::iter_unique`] and
[`KeyringMap::iter_unique_mut`]):

```
use keyring_map::KeyringMap;

let mut map = KeyringMap::new();

map.insert_many(vec!['a', 'b', 'c'], "abc").expect("none of the keys exist already");
map.insert('d', "def");

// Iterate over all keys and values individually
assert_eq!(map.keys().count(), 4);
assert_eq!(map.values().count(), 2);
assert_eq!(map.values_mut().count(), 2);

// Iterate over all keys and their associated values. This means that some
// values are returned multiple times. Therefore, there is no "mut" version of
// this iterator.
let pairs: Vec<(&char, &&'static str)> = map.iter().collect();
assert!(pairs.contains(&(&'a', &"abc")));
assert!(pairs.contains(&(&'b', &"abc")));
assert!(pairs.contains(&(&'c', &"abc")));
assert!(pairs.contains(&(&'d', &"def")));

// Also return the indices.
let pairs: Vec<(usize, &char, &&'static str)> = map.iter_full().collect();
assert_eq!(pairs.len(), 4);

// Return each value only once together with one of its keys.
let pairs: Vec<(&char, &&'static str)> = map.iter_unique().collect();
assert!(pairs.contains(&(&'a', &"abc")));
assert!(pairs.contains(&(&'d', &"def")));
assert_eq!(pairs.len(), 2);

// Iterate over all aliases of a key:
let aliases: Vec<char> = map.aliases(&'a').cloned().collect();
assert_eq!(aliases, vec!['a', 'c', 'b']);
```

## Parallel iteration with `rayon`

When the `rayon` feature is enabled, [`KeyringMap`] offers a variety of parallel
iterators:

```
use rayon::prelude::*;
use keyring_map::KeyringMap;

let mut map = KeyringMap::new();

map.insert_many(vec!['a', 'b', 'c'], "abc").expect("none of the keys exist already");
map.insert('d', "def");

assert_eq!(map.par_values().count(), 2);
assert_eq!(map.par_iter().count(), 4);
assert_eq!(map.par_iter_unique().count(), 2);
```

# Serialization and deserialization

One of the key features of [`KeyringMap`] is its intuitive serialized
representation, which hides the complex implementation details. This makes it
possible to write a serialized [`KeyringMap`] just as easy as a normal hash map:

```
use indoc;
use yaml_serde;
use keyring_map::KeyringMap;

let str = indoc::indoc! {"
- keys: [a, b, c]
  value: abc
- keys: [d, e]
  value: de
- keys: [f]
  value: f
"};

let map: KeyringMap<char, String> = yaml_serde::from_str(&str).expect("valid input");
assert_eq!(map.num_keys(), 6);
assert_eq!(map.num_values(), 3);
```

# Implementation

A [`KeyringMap`] consists of three allocations regardless of the number of keys
or values - two in [`IndexMap`] and one in [`Slab`]. The [`IndexMap`] maps the
key or index to an intermediate accessor struct, which holds an index to the
actual value in the [`Slab`] and a second one to the next key pointing to the
same value. All keys belonging to a value hence form a "ring" via the indices
pointing to each other as illustrated below:
 */
#[doc = ""]
#[cfg_attr(
    feature = "doc-images",
    doc = "![Structure of the KeyringMap][structure]"
)]
#[cfg_attr(
    feature = "doc-images",
    embed_doc_image::embed_doc_image("structure", "docs/img/structure.svg")
)]
#[cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with
    `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
/**

In this particular example, all keys in the ring `d -> a -> b -> g -> e`
hold the same slab index "3" and hence point to the same value / alias each
other.

If a new alias is added with [`map.alias(&'a', 'h')`](KeyringMap::alias), the
ring is opened between `a` and `b` and the key `h` is inserted, resulting in the
new ring `d -> a -> h -> b -> g -> e`. Similarily, if `b` is removed with
[`map.remove_key(&'b')`](KeyringMap::remove_key), the accessor belonging to `a`
(the key previously pointing to `b`) would be modified so it now points to `g`:
`d -> a -> g -> e`.
 */
#[derive(Clone, Debug, Default)]
pub struct KeyringMap<K: Eq + Hash, V, S: BuildHasher + Default = RandomState> {
    keys: IndexMap<K, Accessor, S>,
    values: Slab<V>,
}

impl<K: Eq + Hash, V> KeyringMap<K, V> {
    /// Creates a new map. This does not allocate until values are inserted.
    pub fn new() -> Self {
        return Self {
            keys: IndexMap::new(),
            values: Slab::new(),
        };
    }

    /**
    Creates a new map with capacity for the specified number of `keys` and
    `values`. (Does not allocate if both are zero.)
     */
    pub fn with_capacity(keys: usize, values: usize) -> Self {
        return Self {
            keys: IndexMap::with_capacity(keys),
            values: Slab::with_capacity(values),
        };
    }
}

impl<K: Eq + Hash, V, S: BuildHasher + Default> KeyringMap<K, V, S> {
    /**
    Creates a new map with `hash_builder` used for the underlying [`IndexMap`].

    This function is `const`, so it can be called in `static` contexts.
     */
    pub const fn with_hasher(hash_builder: S) -> Self {
        return Self {
            keys: IndexMap::with_hasher(hash_builder),
            values: Slab::new(),
        };
    }

    /**
    Creates a new map with apacity for the specified number of `keys` and
    `values` and the specified `hash_builder`. (Does not allocate if both
    `keys` and `values` are zero.)

    This is a combination of [`KeyringMap::with_capacity`] and
    [`KeyringMap::with_hasher`].
     */
    pub fn with_capacity_and_hasher(keys: usize, values: usize, hash_builder: S) -> Self {
        return Self {
            keys: IndexMap::with_capacity_and_hasher(keys, hash_builder),
            values: Slab::with_capacity(values),
        };
    }

    /**
    Removes all keys and values from the map while preserving its capacity.
     */
    pub fn clear(&mut self) {
        self.keys.clear();
        self.values.clear();
    }

    /**
    Returns the number of keys in the map, which is equal to or greater than the
    number of values (see [`KeyringMap::num_values`]).

    Computes in **O(1)** time.
     */
    pub fn num_keys(&self) -> usize {
        return self.keys.len();
    }

    /**
    Returns the number of values in the map, which is equal to or smaller than
    the number of keys (see [`KeyringMap::num_keys`]).

    Computes in **O(1)** time.
     */
    pub fn num_values(&self) -> usize {
        return self.values.len();
    }

    /**
    Returns the capacities for `(keys, values)` the map can hoold without
    reallocating one of its underlying [`IndexMap`] or [`Slab`] containers.

    This number is a lower bound; the map might be able to hold more, but is
    guaranteed to be able to hold at least this many.

    Computes in **O(1)** time.
     */
    pub fn capacities(&self) -> (usize, usize) {
        return (self.keys.capacity(), self.values.capacity());
    }

    /**
    Returns a reference to the stored value for `key`, if it is present, else
    `None`.

    Computes in **O(1)** time (average).
     */
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let accessor = self.keys.get(key)?;
        return self.values.get(accessor.slab_index);
    }

    /**
    Returns references to the key-value pair via the key `index`, if it is
    valid.

    Valid indices are `0 <= index < self.len()`.

    Computes in **O(1)** time.
     */
    pub fn get_index(&self, index: usize) -> Option<(&K, &V)> {
        let (key, accessor) = self.keys.get_index(index)?;
        let value = self.values.get(accessor.slab_index)?;
        return Some((key, value));
    }

    /**
    Returns references to the key-value pair via the `key`, if it is present,
    else `None`.

    Computes in **O(1)** time (average).
     */
    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let (k, accessor) = self.keys.get_key_value(key)?;
        let value = self.values.get(accessor.slab_index)?;
        return Some((k, value));
    }

    /**
    Returns the index, the key reference and the value reference, if the given
    `key` is present, else `None`.

    Computes in **O(1)** time (average).
     */
    pub fn get_full<Q>(&self, key: &Q) -> Option<(usize, &K, &V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let (idx, k, accessor) = self.keys.get_full(key)?;
        let value = self.values.get(accessor.slab_index)?;
        return Some((idx, k, value));
    }

    /**
    Returns the index for `key`, if it is present, else `None`.

    Computes in **O(1)** time (average).
     */
    pub fn get_index_of<Q>(&self, key: &Q) -> Option<usize>
    where
        Q: ?Sized + Hash + indexmap::Equivalent<K>,
    {
        return self.keys.get_index_of(key);
    }

    /**
    Returns a mutable reference to the stored value for `key`, if it is present,
    else `None`.

    Computes in **O(1)** time (average).
     */
    pub fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        // Read the value from the accessor
        let accessor = self.keys.get(k)?;
        return self.values.get_mut(accessor.slab_index);
    }

    /**
    Returns a reference to the key and a mutable one to the value via the key
    `index`, if it is valid.

    Valid indices are `0 <= index < self.len()`.

    Computes in **O(1)** time.
     */
    pub fn get_index_mut(&mut self, index: usize) -> Option<(&K, &mut V)> {
        let (key, accessor) = self.keys.get_index(index)?;
        let value = self.values.get_mut(accessor.slab_index)?;
        return Some((key, value));
    }

    /**
    Returns a reference to the key and a mutable one to the value via the `key`,
    if it is present, else `None`.

    Computes in **O(1)** time (average).
     */
    pub fn get_key_value_mut<Q>(&mut self, key: &Q) -> Option<(&K, &mut V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let (k, accessor) = self.keys.get_key_value(key)?;
        let value = self.values.get_mut(accessor.slab_index)?;
        return Some((k, value));
    }

    /**
    Returns the index, the key reference and the mutable value reference, if the
    given `key` is present, else `None`.

    Computes in **O(1)** time (average).
     */
    pub fn get_full_mut<Q>(&mut self, key: &Q) -> Option<(usize, &K, &mut V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let (idx, k, accessor) = self.keys.get_full(key)?;
        let value = self.values.get_mut(accessor.slab_index)?;
        return Some((idx, k, value));
    }

    /**
    Checks if the given `key` is contained within `self`.

    Computes in **O(1)** time (average).
     */
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: ?Sized + Hash + indexmap::Equivalent<K>,
    {
        return self.keys.contains_key(key);
    }

    /**
    Inserts a `key`-`value` pair into the map.

    If the map did not have this key present, `None` is returned. If the map did
    have this key present, the value is updated, and the old value is returned.

    Computes in **O(1)** time (average).
     */
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        match self.get_mut(&key) {
            Some(value_ref) => {
                // Replace the mutable reference
                let old_value = std::mem::replace(value_ref, value);
                return Some(old_value);
            }
            None => {
                // Insert the value
                let slab_index = self.values.insert(value);

                // Create the accessor
                let accessor = Accessor {
                    slab_index,
                    next_key: 0, // Dummy value
                };
                let (next, _) = self.keys.insert_full(key, accessor);

                // Modify the accessor which was just been inserted
                let (_, accessor) = self
                    .keys
                    .get_index_mut(next)
                    .expect("accessor has just been in inserted");
                accessor.next_key = next;
                return None;
            }
        }
    }

    /**
    Inserts a `value` and many `keys` for it simultaneously into the map.

    In the following cases, an [`InsertionError`] holding the `value` and the
    `keys` is returned:
    - `keys` is empty
    - One of the keys is already used in the map.
    Duplicates are ignored.

    Computes in **O(n)** time (average), where **n** is the length of `keys`.

    # Examples

    ```
    use keyring_map::KeyringMap;

    let mut map = KeyringMap::new();

    // Successfull insertion with duplicates
    assert!(map.insert_many(vec!['a', 'b', 'c', 'a'], "abc").is_ok());

    assert_eq!(map.num_keys(), 3);
    assert_eq!(map.num_values(), 1);

    // Empty keys vector
    assert!(map.insert_many(vec![], "def").is_err());

    // Key already in use
    assert!(map.insert_many(vec!['d', 'e', 'a'], "def").is_err());
    ```
     */
    pub fn insert_many(
        &mut self,
        mut keys: Vec<K>,
        value: V,
    ) -> Result<(), InsertionError<(Vec<K>, V)>> {
        // Check if one of the keys is already in use
        for key in keys.iter() {
            if self.contains_key(key) {
                return Err(InsertionError {
                    input: (keys, value),
                    error_type: InsertionErrorType::KeyClaimed,
                });
            }
        }

        let last_key = match keys.pop() {
            Some(key) => key,
            None => {
                return Err(InsertionError {
                    input: (keys, value),
                    error_type: InsertionErrorType::NoKeysGiven,
                });
            }
        };

        // Insert the value
        let slab_index = self.values.insert(value);

        // Create the references
        let mut next_key = 0; // Dummy value, will be replaced later
        let mut first_reference: Option<usize> = None;
        for key in keys.into_iter() {
            // Skip duplicates
            if self.contains_key(&key) {
                continue;
            }

            let accessor = Accessor {
                slab_index,
                next_key,
            };
            let (nk, _) = self.keys.insert_full(key, accessor);
            if first_reference.is_none() {
                first_reference = Some(nk);
            }
            next_key = nk;
        }

        // Manually create the last accessor (which wraps) around from the last to the
        // first value
        let accessor = Accessor {
            slab_index,
            next_key,
        };
        let (next_key, _) = self.keys.insert_full(last_key, accessor);

        // Modify the first accessor again
        if let Some(first_reference) = first_reference {
            let (_, accessor) = self
                .keys
                .get_index_mut(first_reference)
                .expect("accessor has just been in inserted");
            accessor.next_key = next_key;
        }

        return Ok(());
    }

    /**
    Inserts an `alias` for `key`, if `keys` is in the keys of the map and
    `alias` is not. Otherwise, the `alias` is returned wrapped in an [`Err`].

    Computes in **O(1)** time (average).

    See the [struct docstring](KeyringMap), section "Implementation" for
    technical background information on how this is implemented.

    # Examples

    ```
    use keyring_map::KeyringMap;

    let mut map = KeyringMap::new();
    map.insert('a', "abc");

    assert_eq!(map.num_keys(), 1);
    assert_eq!(map.num_values(), 1);

    // Successfull aliasing
    assert!(map.alias(&'a', 'b').is_ok());
    assert!(map.alias(&'a', 'c').is_ok());

    assert_eq!(map.num_keys(), 3);
    assert_eq!(map.num_values(), 1);

    // Aliased key does not exist
    assert!(map.alias(&'d', 'e').is_err());

    // Given alias is already a key of the map
    assert!(map.alias(&'a', 'b').is_err());

    assert_eq!(map.num_keys(), 3);
    assert_eq!(map.num_values(), 1);
    ```
     */
    pub fn alias<Q: ?Sized>(&mut self, key: &Q, alias: K) -> Result<(), K>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        // Does the alias already exist in the map?
        if self.contains_key(alias.borrow()) {
            return Err(alias);
        }

        let (old_next_key, slab_index) = match self.keys.get_mut(key) {
            Some(accessor) => {
                // Adjust the key by "inserting" the new alias. 0 is a dummy
                // value which is replaced later
                let old_next_key = std::mem::replace(&mut accessor.next_key, 0);
                let slab_index = accessor.slab_index;
                (old_next_key, slab_index)
            }
            None => return Err(alias),
        };

        // Insert the new key
        let (next, _) = self.keys.insert_full(
            alias,
            Accessor {
                slab_index,
                next_key: old_next_key,
            },
        );

        // Adjust the accessor at k
        let accessor = self
            .keys
            .get_mut(key)
            .expect("key k must exist - this is a bug");
        accessor.next_key = next;

        return Ok(());
    }

    /**
    Inserts many `aliases` for `key` simultaneously into the map.

    In the following cases, an [`InsertionError`] holding the given `aliases` is
    returned:
    - One of the keys is already used in the map.
    Duplicates are ignored.

    Computes in **O(n)** time (average), where **n** is the length of `aliases`.

    See the [struct docstring](KeyringMap), section "Implementation" for
    technical background information on how this is implemented.

    # Examples

    ```
    use keyring_map::KeyringMap;

    let mut map = KeyringMap::new();
    map.insert('a', "abc");

    assert_eq!(map.num_keys(), 1);
    assert_eq!(map.num_values(), 1);

    // Successfull aliasing
    assert!(map.alias_many(&'a', vec!['b', 'c', 'c']).is_ok());

    assert_eq!(map.num_keys(), 3);
    assert_eq!(map.num_values(), 1);

    // One of the provided alias is already a key of the map -> error (the map
    // is left unchanged).
    assert!(map.alias_many(&'a', vec!['b', 'e', 'f']).is_err());

    assert_eq!(map.num_keys(), 3);
    assert_eq!(map.num_values(), 1);
    ```
    */
    pub fn alias_many<Q: ?Sized>(
        &mut self,
        key: &Q,
        mut aliases: Vec<K>,
    ) -> Result<(), InsertionError<Vec<K>>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        // Check if all aliases are still available and return the alias vector
        // if they aren't
        for alias in aliases.iter() {
            if self.contains_key(alias.borrow()) {
                return Err(InsertionError {
                    input: aliases,
                    error_type: InsertionErrorType::KeyClaimed,
                });
            }
        }

        // Simply iterate through all aliases and insert them one by one.
        let last_alias = match aliases.pop() {
            Some(alias) => alias,
            None => return Ok(()), // aliases was empty - this is not an error!
        };

        // self.alias will return an error if alias or last_alias is already
        // part of the map. This is ok, because duplicates are silently ignored
        // as stated in the method docstring.
        for alias in aliases.into_iter() {
            let _ = self.alias(key, alias);
        }

        // Insert the last alias
        let _ = self.alias(key, last_alias);
        return Ok(());
    }

    /**
    Removes the `key`, its associated value and all of its aliases from the map.
    If the `key` existed, the `value` is returned wrapped in `Some`.

    Computes in **O(n)** time (average), where **n** is the number of aliases
    `key` has (see method [`KeyringMap::aliases`]).

    # Examples

    ```
    use keyring_map::KeyringMap;

    let mut map = KeyringMap::new();
    map.insert_many(vec!['a', 'b', 'c'], "abc");

    assert_eq!(map.num_keys(), 3);
    assert_eq!(map.num_values(), 1);

    assert_eq!(map.remove(&'a'), Some("abc"));

    // All three keys pointing to abc have been removed
    assert_eq!(map.num_keys(), 0);
    assert_eq!(map.num_values(), 0);
    ```
     */
    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        // Loop over all elements of the accessor chain and remove the keys individually
        let (mut idx_to_be_removed, slab_index) = {
            let (index, _, accessor) = self.keys.get_full(key)?;
            (index, accessor.slab_index)
        };

        // Remove all elements of the accessor chain
        loop {
            self.remove_from_accessor_chain(idx_to_be_removed);
            let (_, removed_accessor) = self
                .keys
                .swap_remove_index(idx_to_be_removed)
                .expect("index must exist");
            self.adjust_accessor_chain_after_swap_removal(idx_to_be_removed);

            if removed_accessor.next_key == idx_to_be_removed {
                // If an accessor points to itself, it is the last one ==> Break the loop
                break;
            } else if removed_accessor.next_key != self.keys.len() {
                // If the removed accessor points to the last element of self.keys BEFORE the
                // swap removal, it now points to "idx_to_be_removed" -> Simply remove
                // "idx_to_be_removed" again
                idx_to_be_removed = removed_accessor.next_key;
            }
        }

        // Remove the value
        return Some(self.values.remove(slab_index));
    }

    /**
    Removes the `key` from the map. If it was the only key pointing to a value,
    the value is returned as well (wrapped in `Some`).

    Computes in **O(n)** time (average), where **n** is the number of aliases
    `key` has (see method [`KeyringMap::aliases`]).

    # Examples

    ```
    use keyring_map::KeyringMap;

    let mut map = KeyringMap::new();
    map.insert_many(vec!['a', 'b', 'c'], "abc");

    assert_eq!(map.num_keys(), 3);
    assert_eq!(map.num_values(), 1);

    // After removal of 'a', 'b' and 'c' still point to "abc"
    assert_eq!(map.remove_key(&'a'), None);
    assert_eq!(map.num_keys(), 2);
    assert_eq!(map.num_values(), 1);

    // Attempt to remove a key which doesn't exist -> nothing happens
    assert_eq!(map.remove_key(&'d'), None);
    assert_eq!(map.num_keys(), 2);
    assert_eq!(map.num_values(), 1);

    // Remove 'c': 'b' is still there to point to "abc"
    assert_eq!(map.remove_key(&'c'), None);
    assert_eq!(map.num_keys(), 1);
    assert_eq!(map.num_values(), 1);

    // 'b' is now the only key, so when it is removed, "abc" is removed as well
    assert_eq!(map.remove_key(&'b'), Some("abc"));
    assert_eq!(map.num_keys(), 0);
    assert_eq!(map.num_values(), 0);
    ```
    */
    pub fn remove_key<Q: ?Sized + std::fmt::Debug>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        // Get data of the to-be-removed-accessor
        let idx_to_be_removed = {
            let (idx_to_be_removed, _, _) = self.keys.get_full(k)?;
            idx_to_be_removed
        };

        self.remove_from_accessor_chain(idx_to_be_removed);
        let (_, removed_accessor) = self
            .keys
            .swap_remove_index(idx_to_be_removed)
            .expect("index must exist");
        self.adjust_accessor_chain_after_swap_removal(idx_to_be_removed);

        // The key k is the only one which points to the value
        // accessor.slab_index => Remove the value
        if idx_to_be_removed == removed_accessor.next_key {
            return Some(self.values.remove(removed_accessor.slab_index));
        } else {
            return None;
        }
    }

    /**
    After a swap-removal, the former last element of the "keys" indexmap now is
    at the position of the removed element, which means that its accessor chain
    needs to be adjusted accordingly.
     */
    fn adjust_accessor_chain_after_swap_removal(&mut self, new_index: usize) {
        // The index pointing to the former last element is the length of the keys
        // vector (since an element has been removed)
        let former_index = self.keys.len();

        // Get the entry point into the chain
        let mut next_key = {
            match self.keys.get_index(new_index) {
                Some(tuple) => tuple.1.next_key,
                None => return (),
            }
        };

        // If the entry points to the former index, set the new index as entry point
        if next_key == former_index {
            next_key = new_index;
        }

        loop {
            let (_, accessor) = self
                .keys
                .get_index_mut(next_key)
                .expect("chain is broken - internal error in adjustment after swap removal");
            if accessor.next_key == former_index {
                accessor.next_key = new_index;
                return ();
            } else if accessor.next_key == new_index {
                return ();
            } else {
                next_key = accessor.next_key;
            }
        }
    }

    /// Removes the accessor at the given index from its accessor chain.
    fn remove_from_accessor_chain(&mut self, index: usize) {
        // Get the entry point into the chain
        let next_for_initial_idx = {
            match self.keys.get_index(index) {
                Some(tuple) => {
                    let accessor = tuple.1;
                    accessor.next_key
                }
                None => return (),
            }
        };

        let mut next_key = next_for_initial_idx;
        loop {
            let (_, accessor) = self
                .keys
                .get_index_mut(next_key)
                .expect("chain is broken - internal error when removing accessor from chain");

            if accessor.next_key == index {
                accessor.next_key = next_for_initial_idx;
                return ();
            } else {
                next_key = accessor.next_key;
            }
        }
    }

    /**
    Returns an iterator over all keys which point to the same value as `key`.

    If the `key` does not exist in the map, the iterator is empty. If the key
    does exist, the first value returned by the iterator is `key` itself.

    # Examples

    ```
    use keyring_map::KeyringMap;

    let mut map = KeyringMap::new();
    map.insert_many(vec!['a', 'b', 'c'], "abc");
    map.insert_many(vec!['x', 'y', 'z'], "xyz");

    // First returned key is the input
    let aliases: Vec<&char> = map.aliases(&'c').collect();
    assert_eq!(aliases, vec![&'c', &'b', &'a']);

    // Key does not exist in map
    assert_eq!(map.aliases(&'d').next(), None);
    ```
     */
    pub fn aliases<'a>(&'a self, key: &K) -> Aliases<'a, K, V, S> {
        return Aliases::new(self, key);
    }

    /**
    Returns an iterator over all keys of the map.

    Since the underlying [`IndexMap`] returns the keys in insertion order, this
    method does that too.

    # Examples

    ```
    use keyring_map::KeyringMap;

    let mut map = KeyringMap::new();
    map.insert_many(vec!['a', 'b', 'c'], "abc");
    map.insert_many(vec!['x', 'y', 'z'], "xyz");

    let keys: Vec<&char> = map.keys().collect();
    assert_eq!(keys, vec![&'a', &'b', &'c', &'x', &'y', &'z']);
    ```
     */
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        return self.keys.keys();
    }

    /**
    Returns an iterator over references to all values of the map in arbitrary
    order.

    # Examples

    ```
    use keyring_map::KeyringMap;

    let mut map = KeyringMap::new();
    map.insert_many(vec!['a', 'b', 'c'], "abc");
    map.insert_many(vec!['x', 'y', 'z'], "xyz");

    // Since no value has been removed yet, in this case the iteration order
    // equals the insertion order.
    let values: Vec<_> = map.values().collect();
    assert_eq!(values, vec![&"abc", &"xyz"]);
    ```
     */
    pub fn values(&self) -> impl Iterator<Item = &V> {
        return self.values.iter().map(|(_, value)| value);
    }

    /**
    Returns an iterator over mutable references to all values of the map in
    arbitrary order.

    # Examples

    ```
    use keyring_map::KeyringMap;

    let mut map = KeyringMap::new();
    map.insert_many(vec!['a', 'b', 'c'], "abc");
    map.insert_many(vec!['x', 'y', 'z'], "xyz");

    // Since no value has been removed yet, in this case the iteration order
    // equals the insertion order.
    let values: Vec<_> = map.values_mut().collect();
    assert_eq!(values, vec![&mut "abc", &mut "xyz"]);
    ```
     */
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        return self.values.iter_mut().map(|(_, value)| value);
    }

    /**
    Consumes the map and returns an iterator over all contained values.

    # Examples

    ```
    use keyring_map::KeyringMap;

    let mut map = KeyringMap::new();
    map.insert_many(vec!['a', 'b', 'c'], "abc");
    map.insert_many(vec!['x', 'y', 'z'], "xyz");

    // Since no value has been removed yet, in this case the iteration order
    // equals the insertion order.
    let values: Vec<_> = map.into_values().collect();
    assert_eq!(values, vec!["abc", "xyz"]);
    ```
     */
    pub fn into_values(self) -> IntoValues<V> {
        return self.values.into_iter().map(|(_, value)| value);
    }

    /**
    Returns an iterator over all key-value pairs.

    Since multiple keys can point to the same value, some values can be returned
    multiple times and the total number of returned pairs is equal to the number
    of keys. Therefore, there exists no `_mut` version of this iterator.

    # Examples

    ```
    use keyring_map::KeyringMap;

    let mut map = KeyringMap::new();
    map.insert_many(vec!['a', 'b', 'c'], "abc");
    map.insert_many(vec!['x', 'y', 'z'], "xyz");

    let pairs: Vec<_> = map.iter().collect();
    assert_eq!(pairs.len(), map.num_keys());
    assert!(pairs.contains(&(&'a', &"abc")));
    assert!(pairs.contains(&(&'b', &"abc")));
    assert!(pairs.contains(&(&'c', &"abc")));
    assert!(pairs.contains(&(&'x', &"xyz")));
    assert!(pairs.contains(&(&'y', &"xyz")));
    assert!(pairs.contains(&(&'z', &"xyz")));
    ```
     */
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        return self.keys.iter().map(|(key, accessor)| {
            (
                key,
                self.values
                    .get(accessor.slab_index)
                    .expect("internal error of KeyringMap - this is a bug"),
            )
        });
    }

    /**
    Like [`KeyringMap::iter`], but also returns the key index of the underlying
    [`IndexMap`].

    # Examples

    ```
    use keyring_map::KeyringMap;

    let mut map = KeyringMap::new();
    map.insert_many(vec!['a', 'b', 'c'], "abc");
    map.insert_many(vec!['x', 'y', 'z'], "xyz");

    let pairs: Vec<_> = map.iter_full().collect();
    assert_eq!(pairs.len(), map.num_keys());
    assert!(pairs.contains(&(0, &'a', &"abc")));
    assert!(pairs.contains(&(1, &'b', &"abc")));
    assert!(pairs.contains(&(2, &'c', &"abc")));
    assert!(pairs.contains(&(3, &'x', &"xyz")));
    assert!(pairs.contains(&(4, &'y', &"xyz")));
    assert!(pairs.contains(&(5, &'z', &"xyz")));
    ```
     */
    pub fn iter_full(&self) -> impl Iterator<Item = (usize, &K, &V)> {
        return self.keys.iter().map(|(key, accessor)| {
            (
                self.get_index_of(key)
                    .expect("internal error of KeyringMap - this is a bug"),
                key,
                self.values
                    .get(accessor.slab_index)
                    .expect("internal error of KeyringMap - this is a bug"),
            )
        });
    }

    /**
    Returns an iterator over references to all values and to _one_ of their keys
    (the one with the smallest index).

    # Examples

    ```
    use keyring_map::KeyringMap;

    let mut map = KeyringMap::new();
    map.insert_many(vec!['a', 'b', 'c'], "abc");
    map.insert_many(vec!['x', 'y', 'z'], "xyz");

    let pairs: Vec<_> = map.iter_unique().collect();
    assert_eq!(pairs, vec![(&'a', &"abc"), (&'x', &"xyz")]);
    ```
    */
    pub fn iter_unique(&self) -> impl Iterator<Item = (&K, &V)> {
        return self.keys.iter().filter_map(|(key, accessor)| {
            if self.accessor_has_smallest_idx(accessor) {
                let value = self
                    .values
                    .get(accessor.slab_index)
                    .expect("internal error of KeyringMap - this is a bug");
                return Some((key, value));
            }
            return None;
        });
    }

    /**
    Returns an iterator over mutable references to all values and to _one_ of
    their keys (the one with the smallest index).

    # Examples

    ```
    use keyring_map::KeyringMap;

    let mut map = KeyringMap::new();
    map.insert_many(vec!['a', 'b', 'c'], "abc");
    map.insert_many(vec!['x', 'y', 'z'], "xyz");

    let pairs: Vec<_> = map.iter_unique_mut().collect();
    assert_eq!(pairs, vec![(&'a', &mut "abc"), (&'x', &mut "xyz")]);
    ```
    */
    pub fn iter_unique_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        let slab_ptr: *mut Slab<V> = &mut self.values;
        let keys = &self.keys;
        let this = &*self;

        return keys.iter().filter_map(move |(key, accessor)| {
            if this.accessor_has_smallest_idx(accessor) {
                // SAFETY: Since each accessor chain only has a single smallest
                // index, this "unsafe" does not create aliasing mutable
                // references. However, the borrow checker can't know that,
                // which is why we need to use "unsafe" here.
                let value = unsafe {
                    (*slab_ptr)
                        .get_mut(accessor.slab_index)
                        .expect("internal error") as *mut V
                };
                let value = unsafe { &mut *value };
                return Some((key, value));
            }
            return None;
        });
    }

    fn accessor_has_smallest_idx(&self, accessor: &Accessor) -> bool {
        let mut smallest = accessor.next_key;
        let mut next = accessor.next_key;
        let mut current;
        loop {
            current = next;
            next = self
                .keys
                .get_index(current)
                .expect("internal error of KeyringMap - this is a bug")
                .1
                .next_key;

            // Visited all accessors
            if next == accessor.next_key {
                if smallest == current {
                    return true;
                } else {
                    return false;
                }
            }
            if next < smallest {
                smallest = next;
            }
        }
    }
}

impl<K: Eq + Hash, V> std::ops::Index<&K> for KeyringMap<K, V> {
    type Output = V;

    fn index(&self, key: &K) -> &Self::Output {
        return self.get(key).expect("no entry found for key");
    }
}

impl<K: Eq + Hash, V> std::ops::Index<usize> for KeyringMap<K, V> {
    type Output = V;

    fn index(&self, index: usize) -> &Self::Output {
        return self.get_index(index).expect("no entry found for key").1;
    }
}

/**
An iterator over all keys which point to the same value.

This struct is created by the [`KeyringMap::aliases`] method. See its
documentation for more.
 */
pub struct Aliases<'a, K: Eq + Hash, V, S: BuildHasher + Default> {
    first: usize,
    next: usize,
    first_iteration: bool,
    map: &'a KeyringMap<K, V, S>,
}

impl<'a, K: Eq + Hash, V, S: BuildHasher + Default> Aliases<'a, K, V, S> {
    fn new<Q: ?Sized>(map: &'a KeyringMap<K, V, S>, key: &Q) -> Self
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        match map.keys.get_index_of(key) {
            Some(index) => {
                return Self {
                    first: index,
                    next: index,
                    first_iteration: true,
                    map,
                };
            }
            None => {
                return Self {
                    first: 0,
                    next: 0,
                    first_iteration: false,
                    map,
                };
            }
        }
    }
}

impl<'a, K: Eq + Hash, V, S: BuildHasher + Default> Iterator for Aliases<'a, K, V, S> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        if self.first_iteration {
            self.first_iteration = false;
        } else {
            if self.first == self.next {
                return None;
            }
        }

        let (key, accessor) = self.map.keys.get_index(self.next)?;
        self.next = accessor.next_key;
        return Some(key);
    }
}

/**
The "value" in the [`IndexMap`], which points both to the underlying value in
the [`Slab`] and to the next key in the ring.
 */
#[derive(Clone, Debug)]
struct Accessor {
    /// Index of the value in the `values` [`Slab`] of [`KeyringMap`].
    slab_index: usize,
    /// Next key in the ring. If the value only has one key, this index points
    /// to itself.
    next_key: usize,
}

/**
Field type of [`InsertionError::error_type`]. This enum is produced by some
insertion methods of [`KeyringMap`] like [`KeyringMap::insert_many`] or
[`KeyringMap::alias_many`].
 */
#[derive(Debug)]
pub enum InsertionErrorType {
    /// One of the provided keys already exists in the map.
    KeyClaimed,
    /// This variant is returned by [`KeyringMap::insert_many`] if the provided
    /// key vector is empty.
    NoKeysGiven,
}

impl std::fmt::Display for InsertionErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InsertionErrorType::KeyClaimed => {
                write!(f, "at least one key already exists in the map")
            }
            InsertionErrorType::NoKeysGiven => {
                write!(f, "value does not have any associated keys")
            }
        }
    }
}

impl std::error::Error for InsertionErrorType {}

/**
Error type for failed insertion attempts. This struct is produced by some
insertion methods of [`KeyringMap`] like [`KeyringMap::insert_many`] or
[`KeyringMap::alias_many`].
 */
pub struct InsertionError<T> {
    /// Input to the insertion method (value and / or keys)
    pub input: T,
    /// Why the insertion failed.
    pub error_type: InsertionErrorType,
}

impl<T> std::fmt::Debug for InsertionError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InsertionError")
            .field("error_type", &self.error_type)
            .finish()
    }
}

impl<T> std::fmt::Display for InsertionError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return self.error_type.fmt(f);
    }
}

impl<T> std::error::Error for InsertionError<T> {}

/// An alias type for the returned iterator of [`KeyringMap::into_values`].
pub type IntoValues<V> = std::iter::Map<slab::IntoIter<V>, fn((usize, V)) -> V>;
