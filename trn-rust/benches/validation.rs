use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use trn_rust::{Trn, is_valid_trn, validate_multiple_trns, generate_validation_report};

fn bench_validate_single_trn(c: &mut Criterion) {
    let trn = Trn::new("user", "alice", "tool", "myapi", "v1.0").unwrap();
    
    c.bench_function("validate_single_trn", |b| {
        b.iter(|| trn.is_valid())
    });
}

fn bench_validate_trn_string(c: &mut Criterion) {
    let trn_str = "trn:user:alice:tool:myapi:v1.0";
    
    c.bench_function("validate_trn_string", |b| {
        b.iter(|| is_valid_trn(trn_str))
    });
}

fn bench_validate_different_platforms(c: &mut Criterion) {
    let mut group = c.benchmark_group("validate_platforms");
    
    let platform_trns = vec![
        ("user", Trn::new("user", "alice", "tool", "myapi", "v1.0").unwrap()),
        ("org", Trn::new("org", "company", "model", "bert", "v2.1").unwrap()),
        ("aiplatform", Trn::new("aiplatform", "system", "dataset", "training", "latest").unwrap()),
        ("custom", Trn::new("my-platform", "scope", "pipeline", "etl", "v1.0").unwrap()),
    ];
    
    for (platform_name, trn) in platform_trns {
        group.bench_with_input(
            BenchmarkId::new("platform", platform_name),
            &trn,
            |b, trn| {
                b.iter(|| trn.is_valid())
            },
        );
    }
    
    group.finish();
}

fn bench_validate_batch_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("validate_batch_sizes");
    
    for size in [10, 100, 1000, 5000].iter() {
        let trn_strings: Vec<String> = (0..*size)
            .map(|i| format!("trn:user:user{}:tool:api{}:v1.0", i, i))
            .collect();
        
        group.bench_with_input(BenchmarkId::new("batch_validate", size), size, |b, _| {
            b.iter(|| {
                for trn_str in &trn_strings {
                    let _ = is_valid_trn(trn_str);
                }
            })
        });
    }
    
    group.finish();
}

fn bench_validate_multiple_trns(c: &mut Criterion) {
    let trn_strings = vec![
        "trn:user:alice:tool:weather-api:v1.0".to_string(),
        "trn:org:openai:model:gpt-4:v1.0".to_string(),
        "trn:aiplatform:huggingface:dataset:common-crawl:latest".to_string(),
        "invalid-trn-format".to_string(),
        "trn:user:bob:pipeline:data-prep:v2.1".to_string(),
        "trn::empty:tool:api:v1.0".to_string(),
        "trn:org:anthropic:model:claude-3:v3.0".to_string(),
    ];
    
    c.bench_function("validate_multiple_trns", |b| {
        b.iter(|| validate_multiple_trns(&trn_strings))
    });
}

fn bench_validation_report_generation(c: &mut Criterion) {
    let trn_strings: Vec<String> = (0..1000)
        .map(|i| format!("trn:user:user{}:tool:api{}:v1.0", i, i))
        .collect();
    
    c.bench_function("generate_validation_report", |b| {
        b.iter(|| generate_validation_report(&trn_strings))
    });
}

fn bench_validate_vs_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("validate_vs_parse");
    
    let trn_str = "trn:user:alice:tool:myapi:v1.0";
    
    group.bench_function("parse_and_validate", |b| {
        b.iter(|| {
            if let Ok(trn) = Trn::parse(trn_str) {
                trn.is_valid()
            } else {
                false
            }
        })
    });
    
    group.bench_function("validate_string_only", |b| {
        b.iter(|| is_valid_trn(trn_str))
    });
    
    group.finish();
}

fn bench_validate_error_cases(c: &mut Criterion) {
    let mut group = c.benchmark_group("validate_errors");
    
    let error_cases = vec![
        ("empty", ""),
        ("invalid_format", "not-a-trn"),
        ("too_few", "trn:user:alice"),
        ("too_many", "trn:user:alice:tool:myapi:v1.0:extra"),
        ("empty_component", "trn:user::tool:myapi:v1.0"),
        ("reserved_word", "trn:system:admin:tool:hack:v1.0"),
    ];
    
    for (error_type, trn_str) in error_cases {
        group.bench_with_input(
            BenchmarkId::new("error", error_type),
            trn_str,
            |b, trn_str| {
                b.iter(|| is_valid_trn(trn_str))
            },
        );
    }
    
    group.finish();
}

fn bench_validate_component_lengths(c: &mut Criterion) {
    let mut group = c.benchmark_group("validate_lengths");
    
    let length_cases = vec![
        ("short", "trn:a:b:c:d:e"),
        ("medium", "trn:user:alice:tool:myapi:v1.0"),
        ("long", "trn:very-long-platform-name:very-long-scope-name:very-long-resource-type:very-long-resource-identifier:very-long-version-string"),
    ];
    
    for (length_name, trn_str) in length_cases {
        group.bench_with_input(
            BenchmarkId::new("length", length_name),
            trn_str,
            |b, trn_str| {
                b.iter(|| is_valid_trn(trn_str))
            },
        );
    }
    
    group.finish();
}

fn bench_validate_mixed_valid_invalid(c: &mut Criterion) {
    let mixed_trns = vec![
        "trn:user:alice:tool:myapi:v1.0".to_string(),     // Valid
        "invalid-format".to_string(),                      // Invalid
        "trn:org:company:model:bert:v2.1".to_string(),    // Valid
        "trn:user::tool:myapi:v1.0".to_string(),          // Invalid
        "trn:aiplatform:system:dataset:training:latest".to_string(), // Valid
        "trn:user:alice:tool".to_string(),                // Invalid
        "trn:user:alice:tool:api:v1.0".to_string(),       // Valid
    ];
    
    c.bench_function("validate_mixed_batch", |b| {
        b.iter(|| {
            let mut valid_count = 0;
            for trn_str in &mixed_trns {
                if is_valid_trn(trn_str) {
                    valid_count += 1;
                }
            }
            valid_count
        })
    });
}

fn bench_validation_with_caching(c: &mut Criterion) {
    use trn_rust::ValidationCache;
    
    let cache = ValidationCache::new(1000, 300);
    let trn_strings: Vec<String> = (0..100)
        .map(|i| format!("trn:user:user{}:tool:api{}:v1.0", i % 10, i % 10)) // Repeat patterns
        .collect();
    
    c.bench_function("validation_with_cache", |b| {
        b.iter(|| {
            for trn_str in &trn_strings {
                // Simulate cache lookup and validation
                if cache.get(trn_str).is_none() {
                    let is_valid = is_valid_trn(trn_str);
                    cache.insert(trn_str.clone(), is_valid);
                }
            }
        })
    });
}

fn bench_validate_naming_conventions(c: &mut Criterion) {
    let trns = vec![
        Trn::new("user", "alice", "tool", "myapi", "v1.0").unwrap(),     // Valid naming
        Trn::new("USER", "ALICE", "TOOL", "MYAPI", "V1.0").unwrap(),     // Invalid naming
        Trn::new("User", "Alice", "Tool", "MyAPI", "V1.0").unwrap(),     // Invalid naming
    ];
    
    c.bench_function("validate_naming_conventions", |b| {
        b.iter(|| {
            for trn in &trns {
                let _ = trn_rust::validate_naming_conventions(trn);
            }
        })
    });
}

criterion_group!(
    benches,
    bench_validate_single_trn,
    bench_validate_trn_string,
    bench_validate_different_platforms,
    bench_validate_batch_sizes,
    bench_validate_multiple_trns,
    bench_validation_report_generation,
    bench_validate_vs_parse,
    bench_validate_error_cases,
    bench_validate_component_lengths,
    bench_validate_mixed_valid_invalid,
    bench_validation_with_caching,
    bench_validate_naming_conventions
);

criterion_main!(benches); 