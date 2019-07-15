#[macro_use]
extern crate criterion;

use std::iter;

use criterion::{BatchSize, Criterion, ParameterizedBenchmark};
use rand::prelude::*;
use sled::Db;
use tempfile::TempDir;

use kvs::{KvsEngine, KvStore, KvStorePingCap, SledKvsEngine};

fn set_bench(c: &mut Criterion) {
    let bench = ParameterizedBenchmark::new(
        "kvs",
        |b, _| {
            b.iter_batched(
                || {
                    let temp_dir = TempDir::new().unwrap();
                    let mut rng = SmallRng::from_seed([0; 16]);
                    (KvStore::open(temp_dir.path()).unwrap(), temp_dir, rng)
                },
                |(mut store, _temp_dir, mut rng)| {
                    for i in 1..(1 << 12) {

                        let key = rng.gen_range(1, 1 << 12);

                        store.set(format!("key{}", key), "value".to_string()).unwrap();
                    }
                },
                BatchSize::SmallInput,
            )
        },
        iter::once(()),
    )
        .with_function(
            "kvs-pingcap",
            |b, _| {
                b.iter_batched(
                    || {
                        let temp_dir = TempDir::new().unwrap();
                        let mut rng = SmallRng::from_seed([0; 16]);
                        (KvStorePingCap::open(temp_dir.path()).unwrap(), rng)
                    },
                    |(mut store, mut rng)| {
                        for i in 1..(1 << 12) {

                            let key = rng.gen_range(1, 1 << 12);

                            store.set(format!("key{}", key), "value".to_string()).unwrap();
                        }
                    },
                    BatchSize::SmallInput,
                )
            },
        )
//        .with_function("sled", |b, _| {
//            b.iter_batched(
//                || {
//                    let temp_dir = TempDir::new().unwrap();
//                    SledKvsEngine::new(Db::start_default(&temp_dir).unwrap())
//                },
//                |mut db| {
//                    for i in 1..(1 << 12) {
//                        db.set(format!("key{}", i), "value".to_string()).unwrap();
//                    }
//                },
//                BatchSize::SmallInput,
//            )
//        })
        ;
    c.bench("set_bench", bench);
}

fn get_bench(c: &mut Criterion) {
    let bench = ParameterizedBenchmark::new(
        "kvs",
        |b, i| {
            let temp_dir = TempDir::new().unwrap();
            let mut store = KvStore::open(temp_dir.path()).unwrap();
            for key_i in 1..(1 << i) {
                store
                    .set(format!("key{}", key_i), "value".to_string())
                    .unwrap();
            }
            let mut rng = SmallRng::from_seed([0; 16]);
            b.iter(|| {
                let _t = &temp_dir;
                store
                    .get(format!("key{}", rng.gen_range(1, 1 << i)))
                    .unwrap();
            })
        },
        vec![8, 12, 16,], // 20
    )
        .with_function(
            "kvs-pingcap",
            |b, i| {
                let temp_dir = TempDir::new().unwrap();
                let mut store = KvStorePingCap::open(temp_dir.path()).unwrap();
                for key_i in 1..(1 << i) {
                    store
                        .set(format!("key{}", key_i), "value".to_string())
                        .unwrap();
                }
                let mut rng = SmallRng::from_seed([0; 16]);
                b.iter(|| {
                    store
                        .get(format!("key{}", rng.gen_range(1, 1 << i)))
                        .unwrap();
                })
            },
        )
//        .with_function("sled", |b, i| {
//            let temp_dir = TempDir::new().unwrap();
//            let mut db = SledKvsEngine::new(Db::start_default(&temp_dir).unwrap());
//            for key_i in 1..(1 << i) {
//                db.set(format!("key{}", key_i), "value".to_string())
//                    .unwrap();
//            }
//            let mut rng = SmallRng::from_seed([0; 16]);
//            b.iter(|| {
//                db.get(format!("key{}", rng.gen_range(1, 1 << i))).unwrap();
//            })
//        })
        ;
    c.bench("get_bench", bench);
}

criterion_group!(benches, set_bench, get_bench);
criterion_main!(benches);
