use indoc::indoc;
use keyring_map::KeyringMap;

#[test]
fn test_serialize_and_deserialize() {
    let mut map = KeyringMap::new();

    // Insert two elements and some aliases
    assert!(map.insert(0, 'a').is_none());
    assert!(map.alias(&0, 1).is_ok());
    assert!(map.alias(&0, 3).is_ok());
    assert!(map.insert(2, 'b').is_none());
    assert!(map.alias(&2, 4).is_ok());

    let string: String = yaml_serde::to_string(&map).expect("serialization failed");

    let map: KeyringMap<usize, char> =
        yaml_serde::from_str(&string).expect("deserialization failed");

    // Check the properties of the deserialized map
    assert!(map.get(&0) == Some(&'a'));
    assert!(map.get(&1) == Some(&'a'));
    assert!(map.get(&2) == Some(&'b'));
    assert!(map.get(&3) == Some(&'a'));
    assert!(map.get(&4) == Some(&'b'));
}

#[test]
fn test_deserialize_success() {
    {
        let yaml = indoc! {"
        ---
        - keys: [0, 1, 3]
          value: a
        - keys: [2, 4]
          value: b
        "};

        let map: KeyringMap<usize, char> =
            yaml_serde::from_str(yaml).expect("deserialization failed");

        // Check the properties of the deserialized map
        assert!(map.get(&0) == Some(&'a'));
        assert!(map.get(&1) == Some(&'a'));
        assert!(map.get(&2) == Some(&'b'));
        assert!(map.get(&3) == Some(&'a'));
        assert!(map.get(&4) == Some(&'b'));
    }
}

#[test]
fn test_deserialize_failure() {
    {
        // No key specified
        let yaml = indoc! {"
        ---
        - keys: []
          value: a
        "};

        let res = yaml_serde::from_str::<KeyringMap<usize, char>>(yaml);
        assert!(res.is_err());
    }
    {
        // Key claimed more than once
        let yaml = indoc! {"
        ---
        - keys: [0, 1]
          value: a
        - keys: [0, 2]
          value: b
        "};

        let res = yaml_serde::from_str::<KeyringMap<usize, char>>(yaml);
        assert!(res.is_err());
    }
}
