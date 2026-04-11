use keyring_map::KeyringMap;

#[test]
fn test_insertion() {
    let mut map = KeyringMap::with_capacity(5, 2);

    // Check that all elements are currently empty
    assert!(map.get(&0).is_none());
    assert!(map.get(&1).is_none());
    assert!(map.get(&2).is_none());
    assert!(map.get(&3).is_none());
    assert!(map.get(&4).is_none());

    // Insert an element and some aliases
    assert!(map.insert(0, 'a').is_none());
    assert!(map.alias(&0, 1).is_ok());
    assert!(map.alias(&0, 3).is_ok());

    assert!(map.get(&0) == Some(&'a'));
    assert!(map.get(&1) == Some(&'a'));
    assert!(map.get(&2) == None);
    assert!(map.get(&3) == Some(&'a'));
    assert!(map.get(&4) == None);

    // Insert another element and some aliases
    assert!(map.insert(2, 'b').is_none());
    assert!(map.alias(&2, 4).is_ok());

    assert!(map.get(&0) == Some(&'a'));
    assert!(map.get(&1) == Some(&'a'));
    assert!(map.get(&2) == Some(&'b'));
    assert!(map.get(&3) == Some(&'a'));
    assert!(map.get(&4) == Some(&'b'));

    // Update the value at index 0
    assert!(map.insert(0, 'c').is_some());

    assert!(map.get(&0) == Some(&'c'));
    assert!(map.get(&1) == Some(&'c'));
    assert!(map.get(&2) == Some(&'b'));
    assert!(map.get(&3) == Some(&'c'));
    assert!(map.get(&4) == Some(&'b'));

    // Update the value at index 3
    assert!(map.insert(3, 'd').is_some());

    assert!(map.get(&0) == Some(&'d'));
    assert!(map.get(&1) == Some(&'d'));
    assert!(map.get(&2) == Some(&'b'));
    assert!(map.get(&3) == Some(&'d'));
    assert!(map.get(&4) == Some(&'b'));
}

#[test]
fn test_insert_many() {
    let mut map = KeyringMap::new();

    // Check that all elements are currently empty
    assert!(map.get(&0).is_none());
    assert!(map.get(&1).is_none());
    assert!(map.get(&2).is_none());
    assert!(map.get(&3).is_none());
    assert!(map.get(&4).is_none());

    // Insert an element and some aliases
    assert!(map.insert_many(vec![0, 1, 3], 'a').is_ok());

    assert!(map.get(&0) == Some(&'a'));
    assert!(map.get(&1) == Some(&'a'));
    assert!(map.get(&2) == None);
    assert!(map.get(&3) == Some(&'a'));
    assert!(map.get(&4) == None);

    // Insert another element and some aliases
    assert!(map.insert_many(vec![2, 4], 'b').is_ok());

    assert!(map.get(&0) == Some(&'a'));
    assert!(map.get(&1) == Some(&'a'));
    assert!(map.get(&2) == Some(&'b'));
    assert!(map.get(&3) == Some(&'a'));
    assert!(map.get(&4) == Some(&'b'));

    // Try to insert many, but one of keys is already used
    assert!(map.insert_many(vec![5, 6, 2], 'c').is_err());

    // Try to insert without giving a single key
    assert!(map.insert_many(vec![], 'c').is_err());

    // Try to insert the same key two times (second key is ignored)
    assert!(map.insert_many(vec![5, 5], 'c').is_ok());
}

#[test]
fn test_insert_many_and_remove() {
    {
        let mut map = KeyringMap::new();

        // Insert an element and some aliases
        assert!(map.insert_many(vec![0, 1, 3], 'a').is_ok());

        // Remove the element at key 0 => This also results in key 1 and 3 being removed
        assert!(map.remove(&0).is_some());
        assert_eq!(map.num_values(), 0);
    }

    {
        let mut map = KeyringMap::new();

        // Insert an element and some aliases
        assert!(map.insert_many(vec![0, 1, 3], 'a').is_ok());

        // Remove the element at key 1 => This also results in key 0 and 3 being removed
        assert!(map.remove(&1).is_some());
        assert_eq!(map.num_values(), 0);
    }

    {
        let mut map = KeyringMap::new();

        // Insert an element and some aliases
        assert!(map.insert_many(vec![0, 1, 3], 'a').is_ok());

        // Remove the element at key 3 => This also results in key 0 and 1 being removed
        assert!(map.remove(&3).is_some());
        assert_eq!(map.num_values(), 0);
    }

    {
        let mut map = KeyringMap::new();

        // Insert an element and some aliases
        assert!(map.insert_many(vec![0], 'a').is_ok());

        // Remove the element at key 0 => This also results in key 1 and 3 being removed
        assert!(map.remove(&0).is_some());
        assert_eq!(map.num_values(), 0);
    }
}

#[test]
fn test_remove_key() {
    {
        let mut map = KeyringMap::new();
        assert!(map.insert(0, 'a').is_none());

        assert_eq!(map.num_values(), 1);
        assert_eq!(map.num_keys(), 1);

        assert!(map.remove_key(&0).is_some());

        assert_eq!(map.num_values(), 0);
        assert_eq!(map.num_keys(), 0);
    }

    {
        let mut map = KeyringMap::new();
        assert!(map.insert(0, 'a').is_none());
        assert!(map.alias(&0, 1).is_ok());
        assert!(map.alias(&0, 2).is_ok());
        assert!(map.alias(&0, 3).is_ok());

        for k in 0..4 {
            let mut map_cl = map.clone();

            assert_eq!(map_cl.num_values(), 1);
            assert_eq!(map_cl.num_keys(), 4);

            assert!(map_cl.remove_key(&k).is_none());

            assert_eq!(map_cl.num_values(), 1);
            assert_eq!(map_cl.num_keys(), 3);
        }
    }

    {
        let mut map = KeyringMap::new();
        assert!(map.insert(0, 'a').is_none());
        assert!(map.alias(&0, 1).is_ok());
        assert!(map.alias(&0, 3).is_ok());
        assert!(map.insert(2, 'b').is_none());
        assert!(map.alias(&2, 4).is_ok());

        for k in 0..5 {
            let mut map_cl = map.clone();

            assert_eq!(map_cl.num_values(), 2);
            assert_eq!(map_cl.num_keys(), 5);

            assert!(map_cl.remove_key(&k).is_none());

            assert_eq!(map_cl.num_values(), 2);
            assert_eq!(map_cl.num_keys(), 4);
        }
    }

    {
        let mut map = KeyringMap::new();
        assert!(map.insert(0, 'a').is_none());
        assert!(map.alias(&0, 1).is_ok());
        assert!(map.alias(&0, 3).is_ok());
        assert!(map.insert(2, 'b').is_none());
        assert!(map.alias(&2, 4).is_ok());

        assert_eq!(map.num_values(), 2);
        assert_eq!(map.num_keys(), 5);

        assert!(map.remove_key(&3).is_none());

        assert_eq!(map.num_values(), 2);
        assert_eq!(map.num_keys(), 4);

        assert!(map.remove_key(&2).is_none());

        assert_eq!(map.num_values(), 2);
        assert_eq!(map.num_keys(), 3);

        assert!(map.remove_key(&4).is_some());

        assert_eq!(map.num_values(), 1);
        assert_eq!(map.num_keys(), 2);
    }
}

#[test]
fn test_remove() {
    // Empty a map via random accesses
    {
        let mut map = KeyringMap::new();
        assert!(map.insert(0, 'a').is_none());
        assert!(map.alias(&0, 1).is_ok());
        assert!(map.alias(&0, 3).is_ok());
        assert!(map.insert(2, 'b').is_none());
        assert!(map.alias(&2, 4).is_ok());

        for k in 0..5 {
            let mut map_cl = map.clone();

            assert_eq!(map_cl.num_values(), 2);
            assert_eq!(map_cl.num_keys(), 5);

            let element = map_cl.remove(&k).unwrap();
            if element == 'a' {
                assert_eq!(map_cl.num_values(), 1);
                assert_eq!(map_cl.num_keys(), 2);

                map_cl.remove(&2).unwrap();

                assert_eq!(map_cl.num_values(), 0);
                assert_eq!(map_cl.num_keys(), 0);
            } else {
                assert_eq!(map_cl.num_values(), 1);
                assert_eq!(map_cl.num_keys(), 3);

                map_cl.remove(&3).unwrap();

                assert_eq!(map_cl.num_values(), 0);
                assert_eq!(map_cl.num_keys(), 0);
            }
        }
    }

    {
        let mut map = KeyringMap::new();

        // Insert two elements and some aliases
        assert!(map.insert(0, 'a').is_none());
        assert!(map.alias(&0, 1).is_ok());
        assert!(map.alias(&0, 3).is_ok());
        assert!(map.insert(2, 'b').is_none());
        assert!(map.alias(&2, 4).is_ok());

        assert_eq!(map.num_values(), 2);
        assert_eq!(map.num_keys(), 5);

        // Remove the element at key 2 => This also results in key 4 being removed
        assert!(map.remove(&2).is_some());

        assert_eq!(map.num_values(), 1);
        assert_eq!(map.num_keys(), 3);

        assert!(map.get(&0) == Some(&'a'));
        assert!(map.get(&1) == Some(&'a'));
        assert!(map.get(&2) == None);
        assert!(map.get(&3) == Some(&'a'));
        assert!(map.get(&4) == None);

        // Remove key 0, but keep the value alive
        assert!(map.remove_key(&0).is_none());

        assert_eq!(map.num_values(), 1);
        assert_eq!(map.num_keys(), 2);

        assert!(map.get(&0) == None);
        assert!(map.get(&1) == Some(&'a'));
        assert!(map.get(&2) == None);
        assert!(map.get(&3) == Some(&'a'));
        assert!(map.get(&4) == None);

        // Remove key 1, but keep the value alive
        assert!(map.remove_key(&1).is_none());

        assert_eq!(map.num_values(), 1);
        assert_eq!(map.num_keys(), 1);

        assert!(map.get(&0) == None);
        assert!(map.get(&1) == None);
        assert!(map.get(&2) == None);
        assert!(map.get(&3) == Some(&'a'));
        assert!(map.get(&4) == None);

        // Remove key 3 => Since it is the last key of 'a', 'a' is removed as well and
        // the hashmap is now empty
        assert!(map.remove_key(&3).is_some());

        assert_eq!(map.num_values(), 0);
        assert_eq!(map.num_keys(), 0);
    }

    {
        let mut map = KeyringMap::new();

        // Insert two elements and some aliases
        assert!(map.insert(0, 'a').is_none());
        assert!(map.alias(&0, 1).is_ok());
        assert!(map.alias(&0, 3).is_ok());
        assert!(map.insert(2, 'b').is_none());
        assert!(map.alias(&2, 4).is_ok());

        assert_eq!(map.num_values(), 2);
        assert_eq!(map.num_keys(), 5);

        // Remove the element at key 0 => This also results in key 1 and 3 being removed
        assert!(map.remove(&0).is_some());
    }
}

#[test]
fn test_iterate_associated_keys() {
    let mut map = KeyringMap::new();

    // Insert two elements and some aliases
    assert!(map.insert(0, 'a').is_none());
    assert!(map.alias(&0, 1).is_ok());
    assert!(map.alias(&0, 3).is_ok());
    assert!(map.insert(2, 'b').is_none());
    assert!(map.alias(&2, 4).is_ok());

    let keys: Vec<usize> = map.aliases(&0).cloned().collect();
    assert!(keys.contains(&0));
    assert!(keys.contains(&1));
    assert!(keys.contains(&3));
    assert!(keys.len() == 3);

    let keys: Vec<usize> = map.aliases(&3).cloned().collect();
    assert!(keys.contains(&0));
    assert!(keys.contains(&1));
    assert!(keys.contains(&3));
    assert!(keys.len() == 3);

    let keys: Vec<usize> = map.aliases(&2).cloned().collect();
    assert!(keys.contains(&2));
    assert!(keys.contains(&4));
    assert!(keys.len() == 2);
}

#[test]
fn test_iter_keys_and_values() {
    {
        let mut map = KeyringMap::new();
        assert!(map.insert(0, 'a').is_none());
        assert!(map.alias(&0, 1).is_ok());
        assert!(map.alias(&0, 3).is_ok());
        assert!(map.insert(2, 'b').is_none());
        assert!(map.alias(&2, 4).is_ok());

        let keys: Vec<i32> = map.keys().cloned().collect();
        assert_eq!(keys.len(), 5);
    }
    {
        let mut map = KeyringMap::new();
        assert!(map.insert(0, 'a').is_none());
        assert!(map.alias(&0, 1).is_ok());
        assert!(map.alias(&0, 3).is_ok());
        assert!(map.insert(2, 'b').is_none());
        assert!(map.alias(&2, 4).is_ok());

        let values: Vec<char> = map.values().cloned().collect();
        assert_eq!(values.len(), 2);
    }
    {
        // Modify the values of the map
        let mut map = KeyringMap::new();
        assert!(map.insert(0, 'a').is_none());
        assert!(map.alias(&0, 1).is_ok());
        assert!(map.alias(&0, 3).is_ok());
        assert!(map.insert(2, 'b').is_none());
        assert!(map.alias(&2, 4).is_ok());

        for value in map.values_mut() {
            *value = 'c';
        }

        assert!(map.get(&0) == Some(&'c'));
        assert!(map.get(&1) == Some(&'c'));
        assert!(map.get(&2) == Some(&'c'));
        assert!(map.get(&3) == Some(&'c'));
        assert!(map.get(&4) == Some(&'c'));
    }

    {
        let mut map = KeyringMap::new();
        assert!(map.insert(0, 'a').is_none());
        assert!(map.alias(&0, 1).is_ok());
        assert!(map.alias(&0, 3).is_ok());
        assert!(map.insert(2, 'b').is_none());
        assert!(map.alias(&2, 4).is_ok());

        // Remove a key
        map.remove_key(&2);

        let keys: Vec<i32> = map.keys().cloned().collect();
        assert_eq!(keys.len(), 4);

        let values: Vec<char> = map.values().cloned().collect();
        assert_eq!(values.len(), 2);

        // Remove a key and its value
        map.remove(&0);

        let keys: Vec<i32> = map.keys().cloned().collect();
        assert_eq!(keys.len(), 1);

        let values: Vec<char> = map.values().cloned().collect();
        assert_eq!(values.len(), 1);
    }
}

#[test]
fn test_iter() {
    {
        let mut map = KeyringMap::new();

        // Insert two elements and some aliases
        assert!(map.insert(0, 'a').is_none());
        assert!(map.alias(&0, 1).is_ok());
        assert!(map.alias(&0, 3).is_ok());
        assert!(map.insert(2, 'b').is_none());
        assert!(map.alias(&2, 4).is_ok());

        let iterator = map.iter();
        let vec: Vec<(i32, char)> = iterator
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        assert_eq!(vec.len(), 5);

        // Remove a key, but not the associated value
        map.remove_key(&0);
        let iterator = map.iter();
        let vec: Vec<(i32, char)> = iterator
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        assert_eq!(vec.len(), 4);

        // Remove a value
        map.remove(&4);
        let iterator = map.iter();
        let vec: Vec<(i32, char)> = iterator
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        assert_eq!(vec.len(), 2);
    }
}

#[test]
fn test_iter_unique() {
    {
        let mut map = KeyringMap::new();

        // Insert two elements, one with some aliases
        assert!(map.insert(0, 'a').is_none());
        assert!(map.alias(&0, 1).is_ok());
        assert!(map.alias(&0, 3).is_ok());
        assert!(map.insert(2, 'b').is_none());

        let iterator = map.iter_unique();
        let vec: Vec<(i32, char)> = iterator
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        assert_eq!(vec.len(), 2);
        assert_eq!(vec[0], (0, 'a'));
        assert_eq!(vec[1], (2, 'b'));
    }
    {
        let mut map = KeyringMap::new();

        // Insert two elements and some aliases
        assert!(map.insert(0, 'a').is_none());
        assert!(map.alias(&0, 1).is_ok());
        assert!(map.alias(&0, 3).is_ok());
        assert!(map.insert(2, 'b').is_none());
        assert!(map.alias(&2, 4).is_ok());

        let iterator = map.iter_unique();
        let vec: Vec<(i32, char)> = iterator
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        assert_eq!(vec.len(), 2);
        assert_eq!(vec[0], (0, 'a'));
        assert_eq!(vec[1], (2, 'b'));

        // Remove a key, but not the associated value ==> Inner of values stays the same
        map.remove_key(&0);
        let iterator = map.iter_unique();
        let vec: Vec<(i32, char)> = iterator
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        assert_eq!(vec.len(), 2);
        assert_eq!(vec[0], (4, 'b'));
        assert_eq!(vec[1], (1, 'a'));

        // Remove a value
        map.remove(&4);
        let iterator = map.iter_unique();
        let vec: Vec<(i32, char)> = iterator
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        assert_eq!(vec.len(), 1);
        assert_eq!(vec[0], (3, 'a'));
    }
}

#[test]
fn test_iter_unique_mut() {
    {
        let mut map = KeyringMap::new();

        // Insert two elements and some aliases
        assert!(map.insert(0, 'a').is_none());
        assert!(map.alias(&0, 1).is_ok());
        assert!(map.alias(&0, 3).is_ok());
        assert!(map.insert(2, 'b').is_none());
        assert!(map.alias(&2, 4).is_ok());

        map.iter_unique_mut().for_each(|(_, value)| {
            *value = 'c';
        });

        for k in map.keys() {
            assert_eq!('c', *map.get(k).unwrap());
        }
    }
}

#[test]
fn test_values() {
    {
        let mut map = KeyringMap::new();

        // Insert two elements and some aliases
        assert!(map.insert(0, 'a').is_none());
        assert!(map.alias(&0, 1).is_ok());
        assert!(map.alias(&0, 3).is_ok());
        assert!(map.insert(2, 'b').is_none());
        assert!(map.alias(&2, 4).is_ok());

        let iterator = map.values();
        let vec: Vec<char> = iterator.cloned().collect();
        assert_eq!(vec.len(), 2);
    }
    {
        // Manipulate the values during iteration
        let mut map = KeyringMap::new();

        // Insert two elements and some aliases
        assert!(map.insert(0, 'a').is_none());
        assert!(map.alias(&0, 1).is_ok());
        assert!(map.alias(&0, 3).is_ok());
        assert!(map.insert(2, 'b').is_none());
        assert!(map.alias(&2, 4).is_ok());

        map.values_mut().for_each(|value| {
            *value = 'c';
        });

        for k in map.keys() {
            assert_eq!('c', *map.get(k).unwrap());
        }
    }
}
