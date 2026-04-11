use criterion::{BatchSize, Criterion, criterion_group, criterion_main};

fn iter_one_key_per_value(c: &mut Criterion) {
    let num_keys = 1000;
    let mut group = c.benchmark_group("iter_one_key_per_value");

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
                assert_eq!(map.iter().count(), num_keys);
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
                assert_eq!(map.iter().count(), num_keys);
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn iter_ten_keys_per_value(c: &mut Criterion) {
    let num_keys = 1000;
    let m = 10;
    let mut group = c.benchmark_group("iter_ten_keys_per_value");

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
                assert_eq!(map.iter().count(), m * num_keys);
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
                assert_eq!(map.iter().count(), m * num_keys);
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(benches, iter_one_key_per_value, iter_ten_keys_per_value);
criterion_main!(benches);
