use criterion::{black_box, criterion_group, criterion_main, Criterion};
use trn_rust::{validate, batch_validate};

fn bench_validation_simple(c: &mut Criterion) {
    let trn = "trn:user:alice:tool:openapi:github-api:v1.0";
    c.bench_function("validation_simple", |b| {
        b.iter(|| validate(black_box(trn)))
    });
}

fn bench_validation_cached(c: &mut Criterion) {
    let trn = "trn:user:alice:tool:openapi:github-api:v1.0";
    // Warm up cache
    let _ = validate(trn);
    
    c.bench_function("validation_cached", |b| {
        b.iter(|| validate(black_box(trn)))
    });
}

fn bench_batch_validation_small(c: &mut Criterion) {
    let trns = vec![
        "trn:user:alice:tool:openapi:github-api:v1.0".to_string(),
        "trn:user:bob:tool:python:script:v2.0".to_string(),
        "trn:org:company:tool:workflow:pipeline:latest".to_string(),
    ];
    
    c.bench_function("batch_validation_small", |b| {
        b.iter(|| batch_validate(black_box(&trns)))
    });
}

fn bench_batch_validation_large(c: &mut Criterion) {
    let mut trns = Vec::new();
    for i in 0..100 {
        trns.push(format!("trn:user:user{}:tool:openapi:api{}:v1.0", i, i));
    }
    
    c.bench_function("batch_validation_large", |b| {
        b.iter(|| batch_validate(black_box(&trns)))
    });
}

criterion_group!(benches, bench_validation_simple, bench_validation_cached, bench_batch_validation_small, bench_batch_validation_large);
criterion_main!(benches); 