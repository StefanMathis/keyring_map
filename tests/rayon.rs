use keyring_map::KeyringMap;
use rayon::prelude::*;

#[test]
fn test_par_iter() {
    {
        let mut map = KeyringMap::new();

        // Insert two elements and some aliases
        assert!(map.insert(0, 'a').is_none());
        assert!(map.alias(&0, 1).is_ok());
        assert!(map.alias(&0, 3).is_ok());
        assert!(map.insert(2, 'b').is_none());
        assert!(map.alias(&2, 4).is_ok());

        let iterator = map.par_iter();
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
fn test_par_iter_unique() {
    {
        let mut map = KeyringMap::new();

        // Insert two elements and some aliases
        assert!(map.insert(0, 'a').is_none());
        assert!(map.alias(&0, 1).is_ok());
        assert!(map.alias(&0, 3).is_ok());
        assert!(map.insert(2, 'b').is_none());
        assert!(map.alias(&2, 4).is_ok());

        let iterator = map.par_iter_unique();
        let vec: Vec<(i32, char)> = iterator
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        assert_eq!(vec.len(), 2);

        // Remove a key, but not the associated value ==> Inner of values  stays the
        // same
        map.remove_key(&0);
        let iterator = map.par_iter_unique();
        let vec: Vec<(i32, char)> = iterator
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        assert_eq!(vec.len(), 2);

        // Remove a value
        map.remove(&4);
        let iterator = map.par_iter_unique();
        let vec: Vec<(i32, char)> = iterator
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        assert_eq!(vec.len(), 1);
    }
}

#[test]
fn test_par_values() {
    {
        let mut map = KeyringMap::new();

        // Insert two elements and some aliases
        assert!(map.insert(0, 'a').is_none());
        assert!(map.alias(&0, 1).is_ok());
        assert!(map.alias(&0, 3).is_ok());
        assert!(map.insert(2, 'b').is_none());
        assert!(map.alias(&2, 4).is_ok());

        let iterator = map.par_values();
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

        map.par_values_mut().for_each(|value| {
            *value = 'c';
        });

        for k in map.keys() {
            assert_eq!('c', *map.get(k).unwrap());
        }
    }
}

#[test]
fn test_par_iter_unique_mut() {
    {
        let mut map = KeyringMap::new();

        // Insert two elements and some aliases
        assert!(map.insert(0, 'a').is_none());
        assert!(map.alias(&0, 1).is_ok());
        assert!(map.alias(&0, 3).is_ok());
        assert!(map.insert(2, 'b').is_none());
        assert!(map.alias(&2, 4).is_ok());

        map.par_iter_unique_mut().for_each(|(_, value)| {
            *value = 'c';
        });

        for k in map.keys() {
            assert_eq!('c', *map.get(k).unwrap());
        }
    }
}
