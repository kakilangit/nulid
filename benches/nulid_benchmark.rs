use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use nulid::{Generator, Nulid};

/// Benchmark basic NULID generation
fn bench_generation(c: &mut Criterion) {
    c.bench_function("nulid_new", |b| {
        b.iter(|| {
            let nulid = Nulid::new().unwrap();
            black_box(nulid);
        });
    });
}

/// Benchmark monotonic generation with Generator
fn bench_monotonic_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("monotonic_generation");

    group.bench_function("generator_generate", |b| {
        let generator = Generator::new();
        b.iter(|| {
            let nulid = generator.generate().unwrap();
            black_box(nulid);
        });
    });

    group.bench_function("generator_generate_sequential", |b| {
        let generator = Generator::new();
        b.iter(|| {
            for _ in 0..100 {
                let nulid = generator.generate().unwrap();
                black_box(nulid);
            }
        });
    });

    group.finish();
}

/// Benchmark string encoding and decoding
fn bench_encoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("encoding");
    let nulid = Nulid::new().unwrap();

    group.bench_function("to_string", |b| {
        b.iter(|| {
            let s = nulid.to_string();
            black_box(s);
        });
    });

    let nulid_string = nulid.to_string();
    group.bench_function("from_string", |b| {
        b.iter(|| {
            let parsed = Nulid::from_string(black_box(&nulid_string)).unwrap();
            black_box(parsed);
        });
    });

    group.bench_function("round_trip_string", |b| {
        b.iter(|| {
            let s = nulid.to_string();
            let parsed = Nulid::from_string(&s).unwrap();
            black_box(parsed);
        });
    });

    group.finish();
}

/// Benchmark byte serialization
fn bench_bytes(c: &mut Criterion) {
    let mut group = c.benchmark_group("bytes");
    let nulid = Nulid::new().unwrap();

    group.bench_function("to_bytes", |b| {
        b.iter(|| {
            let bytes = nulid.to_bytes();
            black_box(bytes);
        });
    });

    let bytes = nulid.to_bytes();
    group.bench_function("from_bytes", |b| {
        b.iter(|| {
            let parsed = Nulid::from_bytes(black_box(&bytes)).unwrap();
            black_box(parsed);
        });
    });

    group.bench_function("round_trip_bytes", |b| {
        b.iter(|| {
            let bytes = nulid.to_bytes();
            let parsed = Nulid::from_bytes(&bytes).unwrap();
            black_box(parsed);
        });
    });

    group.finish();
}

/// Benchmark comparison operations
fn bench_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("comparison");

    let nulid1 = Nulid::new().unwrap();
    let nulid2 = Nulid::new().unwrap();

    group.bench_function("equality", |b| {
        b.iter(|| {
            let result = black_box(nulid1) == black_box(nulid2);
            black_box(result);
        });
    });

    group.bench_function("ordering", |b| {
        b.iter(|| {
            let result = black_box(nulid1) < black_box(nulid2);
            black_box(result);
        });
    });

    group.finish();
}

/// Benchmark sorting performance
fn bench_sorting(c: &mut Criterion) {
    let mut group = c.benchmark_group("sorting");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("sort_1000", |b| {
        b.iter_with_setup(
            || {
                // Setup: generate 1000 NULIDs
                let mut nulids = Vec::with_capacity(1000);
                for _ in 0..1000 {
                    nulids.push(Nulid::new().unwrap());
                }
                // Shuffle them
                nulids.reverse();
                nulids
            },
            |mut nulids| {
                // Benchmark: sort them
                nulids.sort();
                black_box(nulids);
            },
        );
    });

    group.finish();
}

/// Benchmark concurrent generation
fn bench_concurrent(c: &mut Criterion) {
    use std::sync::Arc;

    let mut group = c.benchmark_group("concurrent");

    group.bench_function("concurrent_generation_10_threads", |b| {
        b.iter(|| {
            let generator = Arc::new(Generator::new());
            let handles: Vec<_> = (0..10)
                .map(|_| {
                    let generator_clone = Arc::clone(&generator);
                    std::thread::spawn(move || {
                        for _ in 0..100 {
                            let _ = generator_clone.generate();
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    group.finish();
}

/// Benchmark batch generation
fn bench_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch");

    for size in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size));
        group.bench_with_input(format!("generate_{}", size), size, |b, &size| {
            let generator = Generator::new();
            b.iter(|| {
                let mut nulids = Vec::with_capacity(size as usize);
                for _ in 0..size {
                    nulids.push(generator.generate().unwrap());
                }
                black_box(nulids);
            });
        });
    }

    group.finish();
}

/// Benchmark serde serialization (if feature is enabled)
#[cfg(feature = "serde")]
fn bench_serde(c: &mut Criterion) {
    let mut group = c.benchmark_group("serde");
    let nulid = Nulid::new().unwrap();

    group.bench_function("serialize_json", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&nulid)).unwrap();
            black_box(json);
        });
    });

    let json = serde_json::to_string(&nulid).unwrap();
    group.bench_function("deserialize_json", |b| {
        b.iter(|| {
            let nulid: Nulid = serde_json::from_str(black_box(&json)).unwrap();
            black_box(nulid);
        });
    });

    group.bench_function("round_trip_json", |b| {
        b.iter(|| {
            let json = serde_json::to_string(&nulid).unwrap();
            let parsed: Nulid = serde_json::from_str(&json).unwrap();
            black_box(parsed);
        });
    });

    group.finish();
}

#[cfg(feature = "serde")]
criterion_group!(
    benches,
    bench_generation,
    bench_monotonic_generation,
    bench_encoding,
    bench_bytes,
    bench_comparison,
    bench_sorting,
    bench_concurrent,
    bench_batch,
    bench_serde,
);

#[cfg(not(feature = "serde"))]
criterion_group!(
    benches,
    bench_generation,
    bench_monotonic_generation,
    bench_encoding,
    bench_bytes,
    bench_comparison,
    bench_sorting,
    bench_concurrent,
    bench_batch,
);

criterion_main!(benches);
