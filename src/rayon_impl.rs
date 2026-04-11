/*!
This module contains parallel iterator methods for [`KeyringMap`] (only
available if the feature `rayon` is enabled).
 */

use super::*;
use rayon::prelude::*;

impl<K: Eq + Hash + Sync, V, S: BuildHasher + Default + Sync> KeyringMap<K, V, S> {
    /**
    Returns a parallel iterator over all keys of the map.

    This is essentially the parallelized version of [`KeyringMap::keys`].

    # Examples

    ```
    use rayon::prelude::*;
    use keyring_map::KeyringMap;

    let mut map = KeyringMap::new();
    map.insert_many(vec!['a', 'b', 'c'], "abc");
    map.insert_many(vec!['x', 'y', 'z'], "xyz");

    let keys: Vec<&char> = map.par_keys().collect();
    assert_eq!(keys.len(), 6);
    assert!(keys.contains(&&'a'));
    assert!(keys.contains(&&'b'));
    assert!(keys.contains(&&'c'));
    assert!(keys.contains(&&'x'));
    assert!(keys.contains(&&'y'));
    assert!(keys.contains(&&'z'));
    ```
     */
    pub fn par_keys(&self) -> impl ParallelIterator<Item = &K> {
        return self.keys.par_keys();
    }
}

impl<K: Eq + Hash + Sync, V: Sync, S: BuildHasher + Default + Sync> KeyringMap<K, V, S> {
    /**
    Returns a parallel iterator over references to all values of the map.

    This is essentially the parallelized version of [`KeyringMap::values`].

    # Examples

    ```
    use rayon::prelude::*;
    use keyring_map::KeyringMap;

    let mut map = KeyringMap::new();
    map.insert_many(vec!['a', 'b', 'c'], "abc");
    map.insert_many(vec!['x', 'y', 'z'], "xyz");

    let values: Vec<_> = map.values().collect();
    assert_eq!(values.len(), 2);
    assert!(values.contains(&&"abc"));
    assert!(values.contains(&&"xyz"));
    ```
     */
    pub fn par_values(&self) -> impl ParallelIterator<Item = &V> {
        return self.values.iter().map(|(_, value)| value).par_bridge();
    }

    /**
    Returns a parallel iterator over all key-value pairs.

    Since multiple keys can point to the same value, some values can be returned
    multiple times and the total number of returned pairs is equal to the number
    of keys. Therefore, there exists no `_mut` version of this iterator.

    This is essentially the parallelized version of [`KeyringMap::iter`].

    # Examples

    ```
    use rayon::prelude::*;
    use keyring_map::KeyringMap;

    let mut map = KeyringMap::new();
    map.insert_many(vec!['a', 'b', 'c'], "abc");
    map.insert_many(vec!['x', 'y', 'z'], "xyz");

    let pairs: Vec<_> = map.par_iter().collect();
    assert_eq!(pairs.len(), map.num_keys());
    assert!(pairs.contains(&(&'a', &"abc")));
    assert!(pairs.contains(&(&'b', &"abc")));
    assert!(pairs.contains(&(&'c', &"abc")));
    assert!(pairs.contains(&(&'x', &"xyz")));
    assert!(pairs.contains(&(&'y', &"xyz")));
    assert!(pairs.contains(&(&'z', &"xyz")));
    ```
     */
    pub fn par_iter(&self) -> impl ParallelIterator<Item = (&K, &V)> {
        return self.keys.par_iter().map(|(key, accessor)| {
            (
                key,
                self.values
                    .get(accessor.slab_index)
                    .expect("internal error of KeyringMap - this is a bug"),
            )
        });
    }

    /**
    Like [`KeyringMap::par_iter`], but also returns the key index of the
    underlying [`IndexMap`].

    # Examples

    ```
    use rayon::prelude::*;
    use keyring_map::KeyringMap;

    let mut map = KeyringMap::new();
    map.insert_many(vec!['a', 'b', 'c'], "abc");
    map.insert_many(vec!['x', 'y', 'z'], "xyz");

    let pairs: Vec<_> = map.par_iter_full().collect();
    assert_eq!(pairs.len(), map.num_keys());
    assert!(pairs.contains(&(0, &'a', &"abc")));
    assert!(pairs.contains(&(1, &'b', &"abc")));
    assert!(pairs.contains(&(2, &'c', &"abc")));
    assert!(pairs.contains(&(3, &'x', &"xyz")));
    assert!(pairs.contains(&(4, &'y', &"xyz")));
    assert!(pairs.contains(&(5, &'z', &"xyz")));
    ```
     */
    pub fn par_iter_full(&self) -> impl ParallelIterator<Item = (usize, &K, &V)> {
        return self.keys.par_iter().map(|(key, accessor)| {
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

    This is essentially the parallelized version of [`KeyringMap::iter_unique`].

    # Examples

    ```
    use rayon::prelude::*;
    use keyring_map::KeyringMap;

    let mut map = KeyringMap::new();
    map.insert_many(vec!['a', 'b', 'c'], "abc");
    map.insert_many(vec!['x', 'y', 'z'], "xyz");

    let pairs: Vec<_> = map.par_iter_unique().collect();
    assert_eq!(pairs, vec![(&'a', &"abc"), (&'x', &"xyz")]);
    ```
    */
    pub fn par_iter_unique(&self) -> impl ParallelIterator<Item = (&K, &V)> {
        return self.keys.par_iter().filter_map(|(key, accessor)| {
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
}

impl<K: Eq + Hash + Sync, V: Send + Sync, S: BuildHasher + Default + Sync> KeyringMap<K, V, S> {
    /**
    Returns a parallel iterator over mutable references to all values of the
    map.

    This is essentially the parallelized version of [`KeyringMap::values_mut`].

    # Examples

    ```
    use rayon::prelude::*;
    use keyring_map::KeyringMap;

    let mut map = KeyringMap::new();
    map.insert_many(vec!['a', 'b', 'c'], "abc");
    map.insert_many(vec!['x', 'y', 'z'], "xyz");

    let values: Vec<_> = map.par_values_mut().collect();
    assert_eq!(values.len(), 2);
    assert!(values.contains(&&mut "abc"));
    assert!(values.contains(&&mut "xyz"));
    ```
     */
    pub fn par_values_mut(&mut self) -> impl ParallelIterator<Item = &mut V> {
        return self.values_mut().par_bridge();
    }

    /**
    Consumes the map and returns a parallel iterator over all contained values.

    This is essentially the parallelized version of [`KeyringMap::values_mut`].

    # Examples

    ```
    use rayon::prelude::*;
    use keyring_map::KeyringMap;

    let mut map = KeyringMap::new();
    map.insert_many(vec!['a', 'b', 'c'], "abc");
    map.insert_many(vec!['x', 'y', 'z'], "xyz");

    let values: Vec<_> = map.into_values().collect();
    assert_eq!(values.len(), 2);
    assert!(values.contains(&"abc"));
    assert!(values.contains(&"xyz"));
    ```
     */
    pub fn into_par_values(self) -> impl ParallelIterator<Item = V> {
        return self.into_values().par_bridge();
    }

    /**
    Returns an iterator over mutable references to all values and to _one_ of
    their keys (the one with the smallest index).

    This is essentially the parallelized version of
    [`KeyringMap::iter_unique_mut`].

    # Examples

    ```
    use rayon::prelude::*;
    use keyring_map::KeyringMap;

    let mut map = KeyringMap::new();
    map.insert_many(vec!['a', 'b', 'c'], "abc");
    map.insert_many(vec!['x', 'y', 'z'], "xyz");

    let pairs: Vec<_> = map.par_iter_unique_mut().collect();
    assert_eq!(pairs, vec![(&'a', &mut "abc"), (&'x', &mut "xyz")]);
    ```
    */
    pub fn par_iter_unique_mut(&mut self) -> impl ParallelIterator<Item = (&K, &mut V)> {
        struct SlabPtr<V>(*mut Slab<V>);

        unsafe impl<V> Send for SlabPtr<V> {}

        // SAFETY:
        // - Each index is accessed at most once across all threads
        //   (accessor_has_smallest_idx ensures that, since we iterate over all keys and
        //   only select the one with the smallest index - hence, this function will
        //   only ever return true once for an accessor chain and therefore for one
        //   single thread)
        // - No concurrent structural modification of the slab
        // - Therefore, mutable aliasing does not occur
        unsafe impl<V> Sync for SlabPtr<V> {}

        impl<V> SlabPtr<V> {
            fn get_mut(&self, idx: usize) -> &mut V {
                unsafe { (*self.0).get_mut(idx).expect("internal error") }
            }
        }

        let slab_ptr: SlabPtr<V> = SlabPtr(&mut self.values as *mut _);
        let keys = &self.keys;
        let this = &*self;

        return keys.par_iter().filter_map(move |(key, accessor)| {
            if this.accessor_has_smallest_idx(accessor) {
                // SAFETY:
                // This indirection here from &mut V -> *mut V -> &mut V is
                // needed so the lifetime of the closure is erased. It is safe
                // because the value itself stems from self.values which is
                // guaranteed to life for the lifetime of the iterator (it lives
                // as long as self.keys and therefore as self.values).
                let value = slab_ptr.get_mut(accessor.slab_index) as *mut V;
                let value = unsafe { &mut *value };
                return Some((key, value));
            }
            return None;
        });
    }
}
