use criterion::{BatchSize, Criterion, criterion_group, criterion_main};

fn get_one_key(c: &mut Criterion) {
    let num_keys = 1000;
    let mut group = c.benchmark_group("get_one_key");

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
            |map| {
                for key in 0..num_keys {
                    map.get(&key).expect("has_elem");
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
            |map| {
                for key in 0..num_keys {
                    map.get(&key).expect("has_elem");
                }
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn get_two_keys(c: &mut Criterion) {
    let num_keys = 1000;
    let mut group = c.benchmark_group("get_two_keys");

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
            |map| {
                for key in 0..(2 * num_keys) {
                    map.get(&key).expect("has_elem");
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
            |map| {
                for key in 0..(2 * num_keys) {
                    map.get(&key).expect("has_elem");
                }
            },
            BatchSize::SmallInput,
        );
    });

    // multi-map
    group.bench_function("multi-map", |b| {
        b.iter_batched(
            || {
                use multi_map::*;
                let mut map = MultiMap::new();

                for (keys, value) in (0..num_keys).map(|i| (vec![i, num_keys + i], i)) {
                    map.insert(keys[0], keys[1], value);
                }
                map
            },
            |map| {
                for key in 0..num_keys {
                    map.get(&key).expect("has_elem");
                }
                for key in num_keys..(2 * num_keys) {
                    map.get_alt(&key).expect("has_elem");
                }
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn get_three_keys(c: &mut Criterion) {
    let num_keys = 1000;
    let mut group = c.benchmark_group("get_three_keys");

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
            |map| {
                for key in 0..(3 * num_keys) {
                    map.get(&key).expect("has_elem");
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
            |map| {
                for key in 0..(3 * num_keys) {
                    map.get(&key).expect("has_elem");
                }
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn get_many_keys(c: &mut Criterion) {
    let num_keys = 1000;
    let m = 1000;
    let mut group = c.benchmark_group("get_many_keys");

    // keyring_map
    group.bench_function("keyring_map", |b| {
        b.iter_batched(
            || {
                use keyring_map::*;
                let mut map = KeyringMap::new();

                for (keys, value) in (0..num_keys).map(|i| {
                    let mut keys = Vec::new();
                    for k in 0..m {
                        keys.push(i + k * num_keys);
                    }
                    (keys, i)
                }) {
                    map.insert_many(keys, value).expect("won't fail");
                }
                map
            },
            |map| {
                for key in 0..(3 * num_keys) {
                    map.get(&key).expect("has_elem");
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

                for (keys, value) in (0..num_keys).map(|i| {
                    let mut keys = Vec::new();
                    for k in 0..m {
                        keys.push(i + k * num_keys);
                    }
                    (keys, i)
                }) {
                    map.insert_many(keys, value);
                }
                map
            },
            |map| {
                for key in 0..(3 * num_keys) {
                    map.get(&key).expect("has_elem");
                }
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn contains_key_two_keys(c: &mut Criterion) {
    let num_keys = 1000;
    let mut group = c.benchmark_group("contains_key_two_keys");

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
            |map| {
                for key in 0..(2 * num_keys) {
                    map.contains_key(&key);
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
            |map| {
                for key in 0..(2 * num_keys) {
                    map.contains_key(&key);
                }
            },
            BatchSize::SmallInput,
        );
    });

    // multi-map
    group.bench_function("multi-map", |b| {
        b.iter_batched(
            || {
                use multi_map::*;
                let mut map = MultiMap::new();

                for (keys, value) in (0..num_keys).map(|i| (vec![i, num_keys + i], i)) {
                    map.insert(keys[0], keys[1], value);
                }
                map
            },
            |map| {
                for key in 0..(2 * num_keys) {
                    if !map.contains_key(&key) {
                        map.contains_key_alt(&key);
                    }
                }
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn contains_key_many_keys(c: &mut Criterion) {
    let num_keys = 1000;
    let m = 1000;
    let mut group = c.benchmark_group("contains_key_many_keys");

    // keyring_map
    group.bench_function("keyring_map", |b| {
        b.iter_batched(
            || {
                use keyring_map::*;
                let mut map = KeyringMap::new();

                for (keys, value) in (0..num_keys).map(|i| {
                    let mut keys = Vec::new();
                    for k in 0..m {
                        keys.push(i + k * num_keys);
                    }
                    (keys, i)
                }) {
                    map.insert_many(keys, value).expect("won't fail");
                }
                map
            },
            |map| {
                for key in 0..(m * num_keys) {
                    map.contains_key(&key);
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

                for (keys, value) in (0..num_keys).map(|i| {
                    let mut keys = Vec::new();
                    for k in 0..m {
                        keys.push(i + k * num_keys);
                    }
                    (keys, i)
                }) {
                    map.insert_many(keys, value);
                }
                map
            },
            |map| {
                for key in 0..(m * num_keys) {
                    map.contains_key(&key);
                }
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(
    benches,
    get_one_key,
    get_two_keys,
    get_three_keys,
    get_many_keys,
    contains_key_two_keys,
    contains_key_many_keys,
);
criterion_main!(benches);
