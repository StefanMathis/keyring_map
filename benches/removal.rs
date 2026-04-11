use criterion::{BatchSize, Criterion, criterion_group, criterion_main};

fn remove_value_one_key(c: &mut Criterion) {
    let num_keys = 1000;
    let mut group = c.benchmark_group("remove_value_one_key");

    // keyring_map
    group.bench_function("keyring_map", |b| {
        b.iter_batched(
            || {
                use keyring_map::*;
                let mut map = KeyringMap::new();

                for (keys, value) in (0..num_keys).map(|i| (i, i)) {
                    let _ = map.insert(keys, value);
                }
                map
            },
            |mut map| {
                for key in 0..num_keys {
                    map.remove(&key).expect("has_elem");
                }
            },
            BatchSize::SmallInput,
        );
    });

    // multi_key_map
    group.bench_function("multi_key_map", |b| {
        b.iter_batched(
            || {
                use multi_key_map::*;
                let mut map = MultiKeyMap::new();

                for (keys, value) in (0..num_keys).map(|i| (i, i)) {
                    let _ = map.insert(keys, value);
                }
                map
            },
            |mut map| {
                for key in 0..num_keys {
                    map.remove(&key).expect("has_elem");
                }
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn remove_value_two_keys(c: &mut Criterion) {
    let num_keys = 1000;
    let mut group = c.benchmark_group("remove_value_two_keys");

    // keyring_map
    group.bench_function("keyring_map", |b| {
        b.iter_batched(
            || {
                use keyring_map::*;
                let mut map = KeyringMap::new();

                for (keys, value) in (0..num_keys).map(|i| (vec![i, 1000 + i], i)) {
                    map.insert_many(keys, value).expect("won't fail");
                }
                map
            },
            |mut map| {
                for key in 0..num_keys {
                    map.remove(&key).expect("has_elem");
                }
            },
            BatchSize::SmallInput,
        );
    });

    // multi_key_map
    group.bench_function("multi_key_map", |b| {
        b.iter_batched(
            || {
                use multi_key_map::*;
                let mut map = MultiKeyMap::new();

                for (keys, value) in (0..num_keys).map(|i| (vec![i, num_keys + i], i)) {
                    map.insert_many(keys, value);
                }
                map
            },
            |mut map| {
                for key in 0..num_keys {
                    let _ = map.remove(&key);
                }
            },
            BatchSize::SmallInput,
        );
    });

    // multi_map
    group.bench_function("multi_map", |b| {
        b.iter_batched(
            || {
                use multi_map::*;
                let mut map = MultiMap::new();

                for (keys, value) in (0..num_keys).map(|i| (vec![i, num_keys + i], i)) {
                    map.insert(keys[0], keys[1], value);
                }
                map
            },
            |mut map| {
                for key in 0..num_keys {
                    map.remove(&key).expect("has_elem");
                }
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn remove_many_keys(c: &mut Criterion) {
    let num_keys = 1000;
    let mut group = c.benchmark_group("remove_many_keys");

    // keyring_map
    group.bench_function("keyring_map", |b| {
        b.iter_batched(
            || {
                use keyring_map::*;
                let mut map = KeyringMap::new();

                for (keys, value) in
                    (0..num_keys).map(|i| (vec![i, num_keys + i, 2 * num_keys + i], i))
                {
                    map.insert_many(keys, value).expect("won't fail");
                }
                map
            },
            |mut map| {
                for key in 0..num_keys {
                    let _ = map.remove_key(&key);
                    let _ = map.remove_key(&(key + num_keys));
                    let _ = map.remove_key(&(key + 2 * num_keys));
                }
            },
            BatchSize::SmallInput,
        );
    });

    // multi_key_map
    group.bench_function("multi_key_map", |b| {
        b.iter_batched(
            || {
                use multi_key_map::*;
                let mut map = MultiKeyMap::new();

                for (keys, value) in
                    (0..num_keys).map(|i| (vec![i, num_keys + i, 2 * num_keys + i], i))
                {
                    map.insert_many(keys, value);
                }
                map
            },
            |mut map| {
                for key in 0..num_keys {
                    map.remove_many([&key, &(key + num_keys), &(key + 2 * num_keys)]);
                }
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(
    benches,
    remove_value_one_key,
    remove_value_two_keys,
    remove_many_keys
);
criterion_main!(benches);
