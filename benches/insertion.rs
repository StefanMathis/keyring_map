use criterion::{BatchSize, Criterion, criterion_group, criterion_main};

fn generate_two_keys_at_once(n: i32) -> Vec<(Vec<i32>, i32)> {
    (0..n).map(|i| (vec![i, 1000 + i], i)).collect()
}

fn two_keys_at_once(c: &mut Criterion) {
    let inputs = generate_two_keys_at_once(1000);

    let mut group = c.benchmark_group("two_keys_at_once");

    // keyring_map
    group.bench_function("keyring_map", |b| {
        b.iter_batched(
            || inputs.clone(),
            |inputs| {
                use keyring_map::*;
                let mut map = KeyringMap::new();

                for (keys, value) in inputs {
                    map.insert_many(keys, value).expect("won't fail");
                }
            },
            BatchSize::SmallInput,
        );
    });

    // multi_key_map
    group.bench_function("multi_key_map", |b| {
        b.iter_batched(
            || inputs.clone(),
            |inputs| {
                use multi_key_map::*;
                let mut map = MultiKeyMap::new();

                for (keys, value) in inputs {
                    map.insert_many(keys, value);
                }
            },
            BatchSize::SmallInput,
        );
    });

    // multi-map
    group.bench_function("multi_map", |b| {
        b.iter_batched(
            || inputs.clone(),
            |inputs| {
                use multi_map::*;
                let mut map = MultiMap::new();

                for (keys, value) in inputs {
                    map.insert(keys[0], keys[1], value);
                }
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn two_keys_at_once_prealloc(c: &mut Criterion) {
    let inputs = generate_two_keys_at_once(1000);

    let mut group = c.benchmark_group("two_keys_at_once_prealloc");

    // keyring_map
    group.bench_function("keyring_map", |b| {
        b.iter_batched(
            || inputs.clone(),
            |inputs| {
                use keyring_map::*;
                let mut map = KeyringMap::with_capacity(2000, 1000);

                for (keys, value) in inputs {
                    map.insert_many(keys, value).expect("won't fail");
                }
            },
            BatchSize::SmallInput,
        );
    });

    // multi_key_map
    group.bench_function("multi_key_map", |b| {
        b.iter_batched(
            || inputs.clone(),
            |inputs| {
                use multi_key_map::*;
                let mut map = MultiKeyMap::new();

                for (keys, value) in inputs {
                    map.insert_many(keys, value);
                }
            },
            BatchSize::SmallInput,
        );
    });

    // multi-map
    group.bench_function("multi_map", |b| {
        b.iter_batched(
            || inputs.clone(),
            |inputs| {
                use multi_map::*;
                let mut map = MultiMap::with_capacity(1000);

                for (keys, value) in inputs {
                    map.insert(keys[0], keys[1], value);
                }
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn insert_then_alias(c: &mut Criterion) {
    let num_iterations = 1000;
    let mut group = c.benchmark_group("insert_then_alias");

    // keyring_map
    group.bench_function("keyring_map", |b| {
        b.iter_batched(
            || {},
            |_| {
                use keyring_map::*;
                let mut map = KeyringMap::with_capacity(2000, 1000);

                for i in 0..num_iterations {
                    let _ = map.insert(i, i);
                    map.alias(&i, i + num_iterations).expect("won't fail");
                }
            },
            BatchSize::SmallInput,
        );
    });

    // multi_key_map
    group.bench_function("multi_key_map", |b| {
        b.iter_batched(
            || {},
            |_| {
                use multi_key_map::*;
                let mut map = MultiKeyMap::new();

                for i in 0..num_iterations {
                    let _ = map.insert(i, i);
                    map.alias(&i, i + num_iterations).expect("won't fail");
                }
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn one_alias_at_a_time(c: &mut Criterion) {
    let num_iterations = 1000;
    let mut group = c.benchmark_group("one_alias_at_a_time");

    // keyring_map
    group.bench_function("keyring_map", |b| {
        b.iter_batched(
            || {},
            |_| {
                use keyring_map::*;
                let mut map = KeyringMap::with_capacity(2000, 1000);

                let _ = map.insert(0, 0);
                for i in 0..num_iterations {
                    map.alias(&0, i + 1).expect("won't fail");
                }
            },
            BatchSize::SmallInput,
        );
    });

    // multi_key_map
    group.bench_function("multi_key_map", |b| {
        b.iter_batched(
            || {},
            |_| {
                use multi_key_map::*;
                let mut map = MultiKeyMap::new();

                let _ = map.insert(0, 0);
                for i in 0..num_iterations {
                    map.alias(&0, i + 1).expect("won't fail");
                }
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn many_aliases(c: &mut Criterion) {
    let mut group = c.benchmark_group("many_aliases");
    let aliases: Vec<i32> = (1..1001).collect();

    // keyring_map
    group.bench_function("keyring_map", |b| {
        b.iter_batched(
            || aliases.clone(),
            |aliases| {
                use keyring_map::*;
                let mut map = KeyringMap::with_capacity(2000, 1000);

                let _ = map.insert(0, 0);
                map.alias_many(&0, aliases).expect("won't fail");
            },
            BatchSize::SmallInput,
        );
    });

    // multi_key_map
    group.bench_function("multi_key_map", |b| {
        b.iter_batched(
            || aliases.clone(),
            |aliases| {
                use multi_key_map::*;
                let mut map = MultiKeyMap::new();

                let _ = map.insert(0, 0);
                map.alias_many(&0, aliases).expect("won't fail");
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(
    benches,
    two_keys_at_once,
    two_keys_at_once_prealloc,
    insert_then_alias,
    one_alias_at_a_time,
    many_aliases,
);
criterion_main!(benches);
