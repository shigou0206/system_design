use criterion::{black_box, criterion_group, criterion_main, Criterion};
use trn_rust::{parse, validate};

fn bench_parse_simple(c: &mut Criterion) {
    let trn = "trn:user:alice:tool:openapi:github-api:v1.0";
    c.bench_function("parse_simple_trn", |b| {
        b.iter(|| parse(black_box(trn)))
    });
}

fn bench_parse_complex(c: &mut Criterion) {
    let trn = "trn:org:company:tool:openapi:async:complex-api:v2.1.3:stable@sha256:abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
    c.bench_function("parse_complex_trn", |b| {
        b.iter(|| parse(black_box(trn)))
    });
}

fn bench_validate_simple(c: &mut Criterion) {
    let trn = "trn:user:alice:tool:openapi:github-api:v1.0";
    c.bench_function("validate_simple_trn", |b| {
        b.iter(|| validate(black_box(trn)))
    });
}

fn bench_validate_batch(c: &mut Criterion) {
    let trns = vec![
        "trn:user:alice:tool:openapi:github-api:v1.0".to_string(),
        "trn:user:bob:tool:python:script:v2.0".to_string(),
        "trn:org:company:tool:workflow:pipeline:latest".to_string(),
        "trn:aiplatform:tool:system:backup:v1.5".to_string(),
    ];
    
    c.bench_function("validate_batch_trns", |b| {
        b.iter(|| {
            trn_rust::batch_validate(black_box(&trns))
        })
    });
}

criterion_group!(benches, bench_parse_simple, bench_parse_complex, bench_validate_simple, bench_validate_batch);
criterion_main!(benches); 