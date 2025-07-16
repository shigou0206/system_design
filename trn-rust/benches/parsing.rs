use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use trn_rust::Trn;

fn bench_parse_simple_trn(c: &mut Criterion) {
    let trn_str = "trn:user:alice:tool:myapi:v1.0";
    
    c.bench_function("parse_simple_trn", |b| {
        b.iter(|| Trn::parse(trn_str))
    });
}

fn bench_parse_complex_trn(c: &mut Criterion) {
    let trn_str = "trn:aiplatform:huggingface-transformers:model:bert-large-uncased:v2.1.3";
    
    c.bench_function("parse_complex_trn", |b| {
        b.iter(|| Trn::parse(trn_str))
    });
}

fn bench_parse_multiple_trns(c: &mut Criterion) {
    let trn_strings = vec![
        "trn:user:alice:tool:weather-api:v1.0",
        "trn:org:openai:model:gpt-4:v1.0",
        "trn:aiplatform:huggingface:dataset:common-crawl:latest",
        "trn:user:bob:pipeline:data-preprocessing:v2.1",
        "trn:org:anthropic:model:claude-3:v3.0",
    ];
    
    c.bench_function("parse_multiple_trns", |b| {
        b.iter(|| {
            for trn_str in &trn_strings {
                let _ = Trn::parse(trn_str);
            }
        })
    });
}

fn bench_parse_batch_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_batch_sizes");
    
    for size in [10, 100, 1000, 5000].iter() {
        let trn_strings: Vec<String> = (0..*size)
            .map(|i| format!("trn:user:user{}:tool:api{}:v1.0", i, i))
            .collect();
        
        group.bench_with_input(BenchmarkId::new("batch_parse", size), size, |b, _| {
            b.iter(|| {
                for trn_str in &trn_strings {
                    let _ = Trn::parse(trn_str);
                }
            })
        });
    }
    
    group.finish();
}

fn bench_parse_different_platforms(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_platforms");
    
    let platforms = vec![
        ("user", "trn:user:alice:tool:myapi:v1.0"),
        ("org", "trn:org:company:model:bert:v2.1"),
        ("aiplatform", "trn:aiplatform:system:dataset:training:latest"),
        ("custom", "trn:my-custom-platform:scope:pipeline:etl:v1.0"),
    ];
    
    for (platform_name, trn_str) in platforms {
        group.bench_with_input(
            BenchmarkId::new("platform", platform_name),
            trn_str,
            |b, trn_str| {
                b.iter(|| Trn::parse(trn_str))
            },
        );
    }
    
    group.finish();
}

fn bench_parse_different_lengths(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_lengths");
    
    let test_cases = vec![
        ("short", "trn:a:b:c:d:e"),
        ("medium", "trn:user:alice:tool:myapi:v1.0"),
        ("long", "trn:aiplatform:huggingface-transformers:model:bert-large-uncased-whole-word-masking:v2.1.3-beta"),
    ];
    
    for (length_name, trn_str) in test_cases {
        group.bench_with_input(
            BenchmarkId::new("length", length_name),
            trn_str,
            |b, trn_str| {
                b.iter(|| Trn::parse(trn_str))
            },
        );
    }
    
    group.finish();
}

fn bench_parse_vs_constructor(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_vs_constructor");
    
    let trn_str = "trn:user:alice:tool:myapi:v1.0";
    
    group.bench_function("parse_from_string", |b| {
        b.iter(|| Trn::parse(trn_str))
    });
    
    group.bench_function("construct_direct", |b| {
        b.iter(|| Trn::new("user", "alice", "tool", "myapi", "v1.0"))
    });
    
    group.finish();
}

fn bench_parse_error_cases(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_errors");
    
    let error_cases = vec![
        ("empty", ""),
        ("invalid_format", "not-a-trn"),
        ("too_few_components", "trn:user:alice"),
        ("too_many_components", "trn:user:alice:tool:myapi:v1.0:extra:more"),
        ("empty_component", "trn:user::tool:myapi:v1.0"),
    ];
    
    for (error_type, trn_str) in error_cases {
        group.bench_with_input(
            BenchmarkId::new("error", error_type),
            trn_str,
            |b, trn_str| {
                b.iter(|| {
                    let _ = Trn::parse(trn_str);
                })
            },
        );
    }
    
    group.finish();
}

fn bench_parse_memory_usage(c: &mut Criterion) {
    let trn_strings: Vec<String> = (0..1000)
        .map(|i| format!("trn:user:user{}:tool:api{}:v1.0", i, i))
        .collect();
    
    c.bench_function("parse_memory_reuse", |b| {
        b.iter(|| {
            let mut parsed_trns = Vec::with_capacity(1000);
            for trn_str in &trn_strings {
                if let Ok(trn) = Trn::parse(trn_str) {
                    parsed_trns.push(trn);
                }
            }
            parsed_trns
        })
    });
}

criterion_group!(
    benches,
    bench_parse_simple_trn,
    bench_parse_complex_trn,
    bench_parse_multiple_trns,
    bench_parse_batch_sizes,
    bench_parse_different_platforms,
    bench_parse_different_lengths,
    bench_parse_vs_constructor,
    bench_parse_error_cases,
    bench_parse_memory_usage
);

criterion_main!(benches); 