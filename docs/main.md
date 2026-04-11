This crate offers [`KeyringMap`], a hash map based on [`IndexMap`] which allows
specifying multiple keys per value. It offers the functionality one expects from
a regular hash map, such as **O(1)** lookup, insertion, removal and iteration
over key-value pairs. Besides that, the key features of [`IndexMap`] such as
lookup via index instead of key are also available.

The distinctive feature of [`KeyringMap`] is the ability to specify multiple
keys which _alias_ each other, i.e. point to the same value. This can be done
by providing multiple keys when inserting a value or by defining one or more
aliases for an existing key. Keys can be removed individually - the value stays
in the map as long as at least one key for it exists. Of course, it is also
possible to remove a value and all of its keys at once. The sections below show
the basic usage for insertion, access, removal and iteration.

The name "KeyringMap" stems from the internal implementation, which uses a
"ring" of keys pointing to each other for a single value. This allows minimizing
the number of allocations, resulting in very good cache-locality and
performance. See the [Benchmarks](#benchmarks) section for a comparison with
other maps with similar multi-key capabilities.

## Insertion

```rust
use keyring_map::KeyringMap;

let mut map = KeyringMap::new();

// Insert a value and a single key just as in a regular hash map
map.insert('a', "abc");
assert_eq!(map.get(&'a'), Some(&"abc"));

// Define two aliases for 'a' which also point to "abc"
map.alias(&'a', 'b');
map.alias(&'a', 'c');

// All three keys point to the same value
assert_eq!(map.get(&'a'), map.get(&'b'));
assert_eq!(map.get(&'a'), map.get(&'c'));

// Insert a value and many keys for it at once
map.insert_many(vec!['d', 'e'], "defg");

// Add many aliases at once
map.alias_many(&'d', vec!['f', 'g']);

// All keys d, e, f and g now point to the same value
assert_eq!(map.get(&'d'), Some(&"defg"));
assert_eq!(map.get(&'d'), map.get(&'e'));
assert_eq!(map.get(&'d'), map.get(&'f'));
assert_eq!(map.get(&'d'), map.get(&'g'));
```

## Access

As mentioned initially, [`KeyringMap`] supports access both via key and index
just as the underlying [`IndexMap`]. It also supports the array index notation,
panicking if that index does not exist:

```rust
use keyring_map::KeyringMap;

// Allocate a map with space for at least 20 keys and 10 values without the need
// for reallocation.
let mut map = KeyringMap::with_capacity(20, 10); 

map.insert_many(vec!['a', 'b', 'c'], "abc");
map.insert_many(vec!['x', 'y', 'z'], "xyz");

// Regular access with "get" just as in a regular hash map
assert_eq!(map.get(&'a'), Some(&"abc"));

// Access with index (an usize). First get the index, then use it for lookup.
// The lookup returns both the key and the value.
let idx_x = map.get_index_of(&'x').expect("key exists");
assert_eq!(map.get_index(idx_x), Some((&'x', &"xyz")));

// Access via array indexing
assert_eq!(&map[idx_x], &"xyz");

// Check if the key exists in the map
assert!(map.contains_key(&'c'));
assert!(!map.contains_key(&'d'));
```

## Removal

```rust
use keyring_map::KeyringMap;

let mut map = KeyringMap::new();
map.insert_many(vec!['a', 'b', 'c'], "abc");

assert_eq!(map.num_keys(), 3);
assert_eq!(map.num_values(), 1);

// Remove just a key
map.remove_key(&'a');
assert_eq!(map.num_keys(), 2);
assert_eq!(map.num_values(), 1);

// Remove the value and all keys for it
map.remove(&'b');
assert_eq!(map.num_keys(), 0);
assert_eq!(map.num_values(), 0);

// When all keys to a value are removed, the value is removed as well
map.insert_many(vec!['x', 'y', 'z'], "xyz");

assert_eq!(map.num_keys(), 3);
assert_eq!(map.num_values(), 1);

map.remove_key(&'x');
assert_eq!(map.num_keys(), 2);
assert_eq!(map.num_values(), 1);

map.remove_key(&'y');
assert_eq!(map.num_keys(), 1);
assert_eq!(map.num_values(), 1);

map.remove_key(&'z');
assert_eq!(map.num_keys(), 0);
assert_eq!(map.num_values(), 0);
```

## Iteration

A huge variety of iterators commonly found in hash maps are available: Iterate
over keys, values, key-value pairs (returning a value multiple times if it has
multiple keys) etc. Similar to [`IndexMap`], it is also possible to include the
key indices in the iterator items, see [`KeyringMap::iter_full`].

[`KeyringMap`] also offers a couple of special iterators:
- Iterate over all values and return _one_ of its keys with
[`KeyringMap::iter_unique`] and the mutable variant
[`KeyringMap::iter_unique_mut`].
- Iterate over all aliases of a key with [`KeyringMap::aliases`].

```rust
use keyring_map::KeyringMap;

let mut map = KeyringMap::new();
map.insert_many(vec!['a', 'b', 'c'], "abc");
map.insert_many(vec!['x', 'y', 'z'], "xyz");

// Key iteration
let keys: Vec<&char> = map.keys().collect();
assert_eq!(keys, vec![&'a', &'b', &'c', &'x', &'y', &'z']);

// Value iteration
let keys: Vec<&&'static str> = map.values().collect();
assert_eq!(keys, vec![&"abc", &"xyz"]);

// Iterate over all key-value pairs
let pairs: Vec<_> = map.iter().collect();
assert_eq!(pairs.len(), map.num_keys());
assert!(pairs.contains(&(&'a', &"abc")));
assert!(pairs.contains(&(&'b', &"abc")));
assert!(pairs.contains(&(&'c', &"abc")));
assert!(pairs.contains(&(&'x', &"xyz")));
assert!(pairs.contains(&(&'y', &"xyz")));
assert!(pairs.contains(&(&'z', &"xyz")));

// Return each value once together with one of its keys
let pairs: Vec<_> = map.iter_unique().collect();
assert_eq!(pairs, vec![(&'a', &"abc"), (&'x', &"xyz")]);

// Iterate over all aliases of the given key
let aliases: Vec<_> = map.aliases(&'c').collect();
assert_eq!(aliases, vec![&'c', &'b', &'a']);

// This key does not exist - list of aliases is empty
let aliases: Vec<_> = map.aliases(&'d').collect();
assert!(aliases.is_empty());
```

# Feature flags

## Serialization and deserialization

One of the core features of [`KeyringMap`] is its (de)serialization model, which
hides the complex implementation behind an intuitive representation which is
very similar to that of a normal hash map: A list of `keys`-`value` pairs, where
the `keys` itself is a list containing the individual keys.

```rust
use indoc;
use serde_yaml;
use keyring_map::KeyringMap;

let str = indoc::indoc! {"
- keys: [a, b, c]
  value: abc
- keys: [d, e]
  value: de
- keys: [f]
  value: f
"};

let map: KeyringMap<char, String> = serde_yaml::from_str(&str).expect("valid input");
assert_eq!(map.num_keys(), 6);
assert_eq!(map.num_values(), 3);

let aliases: Vec<_> = map.aliases(&'a').collect();
assert!(aliases.contains(&&'a'));
assert!(aliases.contains(&&'b'));
assert!(aliases.contains(&&'c'));
```

This functionality is gated behind the `serde` feature flag.

## Parallel iterators

If the `rayon` feature is enabled, parallelized variants for some of the serial
iterators are available:

```rust
use rayon::prelude::*;
use keyring_map::KeyringMap;

let mut map = KeyringMap::new();
map.insert_many(vec!['a', 'b', 'c'], "abc");
map.insert_many(vec!['x', 'y', 'z'], "xyz");

// Iterate over all values (references and mutable references)
assert_eq!(map.par_values().count(), 2);
assert_eq!(map.par_values_mut().count(), 2);

// Iterate over all keys
assert_eq!(map.par_keys().count(), 6);

// Iterate over all keys and their values (and their indices)
assert_eq!(map.par_iter().count(), 6);
assert_eq!(map.par_iter_full().count(), 6);

// Iterate over all values and one of their keys
assert_eq!(map.par_iter_unique().count(), 2);
assert_eq!(map.par_iter_unique_mut().count(), 2);
```

# Alternatives and benchmarks

This section includes a comparison and benchmarks to similar crates. If a crate
with similar functionality as `keyring_map` is not listed here, please let me
know so I can include them.

## Alternatives

### `multi_key_map`

The closest equivalent to this crate is
[`multi_key_map`](https://crates.io/crates/multi_key_map), which provides more
or less the same functionality, but with a different implementation approach
(using a vector of keys instead of the ring indexing of `keyring_map`).

### `multi-map`

[`multi-map`](https://crates.io/crates/multi-map) requires exactly two keys per
value. This approach makes it efficient (because it does not need to allocate
for an arbitrary number of keys), but much more limited than either
`keyring_map` or `multi_key_map` (which allow an arbitrary number of keys).
Therefore, it is only included in some benchmarks. Additionally, its keys have a
hierarchy (a primary and an "alternative" key), resulting in a larger API
surface (e.g. there is both a `contains_key` and a `contains_key_alt` method).

## Benchmarks

All values are median executation times in µs as reported by the
[`criterion`](https://crates.io/crates/criterion) crate. To run the benchmark
on your machine, clone the repository and then run `cargo bench`.

Since `multi-map` does not offer the same possibilities as `keyring_map`  or
`keyring_map`, it could only be included in some benchmarks.

### Insertion

File `benches/insertion.rs`

| Test name | Test description | `keyring_map` | `multi_key_map` | `multi-map` |
| -------- | -------- | -------- | -------- | -------- |
| `two_keys_at_once` | Repeatedly insert two keys and a value | 152.25 | 263.49 | 143.16 |
| `two_keys_at_once_prealloc` | Like above, but map has been preallocated | 124.99 | 285.47 | 71.380 |
| `insert_then_alias` | Insert a key-value pair, then create an alias | 133.57 | 311.23 | - |
| `one_alias_at_a_time` | Repeatedly insert an alias | 74.063 | 138.82 | - |
| `many_aliases` | Insert many aliases at once | 79.595 | 61.769 | - |

When inserting only a few keys at once, `keyring_map` is usually about twice as
fast as `multi_key_map`, likely due to the fact that the former does not require
a vector per value. However, it pays for that with a more complex logic (every
time a new key is inserted, it has to be fitted into the key ring). Therefore,
when inserting a lot of keys (see benchmark `many_aliases`), `multi_key_map` can
be faster. Since `multi-map` requires neither storing a vector nor modifying a
key ring, it is the fastest one when storing exactly two keys per value at the
cost of flexibility.

### Access

File `benches/access.rs`

| Test name | Test description | `keyring_map` | `multi_key_map` | `multi-map` |
| -------- | -------- | -------- | -------- | -------- |
| `get_one_key` | Get a value (one key per value) | 23.436 | 64.993 | - |
| `get_two_keys` | Get a value (two keys per value) | 46.164 | 125.84 | 57.495 |
| `get_three_keys` | Get a value (three keys per value) | 67.927 | 154.01 | - |
| `get_many_keys` | Get a value (1000 keys per value) | 1,9623  | 3,856 | - |
| `contains_key_two_keys` | Is key in map (two keys per value) | 47.491 | 45.175 | 46.345 |
| `contains_key_many_keys` | Is key in map (1000 keys per value) | 83,240 | 72,284 | - |

In `keyring_map`, each key is accessed directly from an [`IndexMap`] and gives
the index to the value in the underlying [`Slab`]. This allows for very fast
retrieval compared to the other maps, as seen in the benchmarks. When checking
if a key is within the map, all three maps are roughly equivalent with a small
number of keys per value. If each value has a lot of keys, `multi_key_map`
becomes clearly faster than `keyring_map`.

### Removal

File `benches/removal.rs`

| Test name | Test description | `keyring_map` | `multi_key_map` | `multi-map` |
| -------- | -------- | -------- | -------- | -------- |
| `remove_value_one_key` | Remove a value with one key  | 49.871 | 156.42 | - |
| `remove_value_two_keys` | Remove a value with two keys | 76.553 | 134.17 | 72.268 |
| `remove_many_keys` | Remove all keys at once | 184.98 | 395.68 | - |

When removing a value, `keyring_map` is roughly equivalent to `multi-map` and
about twice as fast as `multi_key_map`. This likely results from the fact that
the latter needs to deallocate the key vector when the value is removed (the
two other maps don't require deallocations).

### Iteration

File `benches/iteration.rs`

| Test name | Test description | `keyring_map` | `multi_key_map` | `multi-map` |
| -------- | -------- | -------- | -------- | -------- |
| `iter_one_key_per_value` | Every value in the map has one key | 6.0742 | 49.143 | - |
| `iter_ten_keys_per_value` | Every value in the map has ten keys | 49.143 | 345.44 | - |

Since `keyring_map` at its core only uses three heap-allocated containers (two
in [`IndexMap`] and one in [`Slab`]), it greatly profits from cache locality
when iterating. This is further helped by the internal structure of the
underlying [`IndexMap`], which generally provides faster iteration than
[`HashMap`] from the standard library.

## Summary

For a lot of use cases, `keyring_map` clearly has a performance edge over
`multi_key_map`. Often, it is even comparable to the much less flexible
`multi-map`. However, there are certain scenarios where the alternatives beat
`keyring_map`. When in doubt, it is recommended to benchmark for your particular
application.
