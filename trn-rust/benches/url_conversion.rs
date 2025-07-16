use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use trn_rust::{Trn, url_to_trn};

fn bench_trn_to_url(c: &mut Criterion) {
    let trn = Trn::new("user", "alice", "tool", "myapi", "v1.0").unwrap();
    
    c.bench_function("trn_to_url", |b| {
        b.iter(|| trn.to_url())
    });
}

fn bench_trn_to_http_url(c: &mut Criterion) {
    let trn = Trn::new("user", "alice", "tool", "myapi", "v1.0").unwrap();
    let base_url = "https://platform.example.com";
    
    c.bench_function("trn_to_http_url", |b| {
        b.iter(|| trn.to_http_url(base_url))
    });
}

fn bench_url_to_trn(c: &mut Criterion) {
    let trn_url = "trn://user/alice/tool/myapi/v1.0";
    
    c.bench_function("url_to_trn", |b| {
        b.iter(|| url_to_trn(trn_url))
    });
}

fn bench_url_roundtrip(c: &mut Criterion) {
    let original_trn = Trn::new("user", "alice", "tool", "myapi", "v1.0").unwrap();
    
    c.bench_function("url_roundtrip", |b| {
        b.iter(|| {
            let url = original_trn.to_url().unwrap();
            url_to_trn(&url).unwrap()
        })
    });
}

fn bench_url_conversion_batch(c: &mut Criterion) {
    let trns: Vec<Trn> = (0..100)
        .map(|i| Trn::new("user", &format!("user{}", i), "tool", &format!("api{}", i), "v1.0").unwrap())
        .collect();
    
    c.bench_function("url_conversion_batch", |b| {
        b.iter(|| {
            for trn in &trns {
                let _ = trn.to_url();
            }
        })
    });
}

fn bench_url_conversion_different_platforms(c: &mut Criterion) {
    let mut group = c.benchmark_group("url_platforms");
    
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
                b.iter(|| trn.to_url())
            },
        );
    }
    
    group.finish();
}

fn bench_http_url_different_bases(c: &mut Criterion) {
    let mut group = c.benchmark_group("http_url_bases");
    let trn = Trn::new("user", "alice", "tool", "myapi", "v1.0").unwrap();
    
    let base_urls = vec![
        ("short", "https://api.com"),
        ("medium", "https://platform.example.com"),
        ("long", "https://very-long-domain-name.example.organization.com/api/v1/resources"),
    ];
    
    for (base_name, base_url) in base_urls {
        group.bench_with_input(
            BenchmarkId::new("base", base_name),
            base_url,
            |b, base_url| {
                b.iter(|| trn.to_http_url(base_url))
            },
        );
    }
    
    group.finish();
}

fn bench_url_with_special_characters(c: &mut Criterion) {
    let trn = Trn::new("user", "alice-smith", "custom-tool", "my_api.v2", "v1.0-beta").unwrap();
    
    c.bench_function("url_special_chars", |b| {
        b.iter(|| trn.to_url())
    });
}

fn bench_url_parsing_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("url_parsing");
    
    let url_cases = vec![
        ("simple", "trn://user/alice/tool/myapi/v1.0"),
        ("complex", "trn://aiplatform/huggingface-transformers/model/bert-large-uncased/v2.1.3"),
        ("with_special", "trn://user/alice-smith/custom-tool/my_api.v2/v1.0-beta"),
    ];
    
    for (case_name, url) in url_cases {
        group.bench_with_input(
            BenchmarkId::new("case", case_name),
            url,
            |b, url| {
                b.iter(|| url_to_trn(url))
            },
        );
    }
    
    group.finish();
}

fn bench_url_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("url_errors");
    
    let invalid_urls = vec![
        ("empty", ""),
        ("wrong_scheme", "https://example.com/path"),
        ("incomplete", "trn://user/alice"),
        ("too_many", "trn://user/alice/tool/myapi/v1.0/extra/components"),
        ("invalid_chars", "trn://user with spaces/alice/tool/myapi/v1.0"),
    ];
    
    for (error_type, url) in invalid_urls {
        group.bench_with_input(
            BenchmarkId::new("error", error_type),
            url,
            |b, url| {
                b.iter(|| {
                    let _ = url_to_trn(url);
                })
            },
        );
    }
    
    group.finish();
}

fn bench_url_memory_allocation(c: &mut Criterion) {
    let trns: Vec<Trn> = (0..1000)
        .map(|i| Trn::new("user", &format!("user{}", i), "tool", &format!("api{}", i), "v1.0").unwrap())
        .collect();
    
    c.bench_function("url_memory_allocation", |b| {
        b.iter(|| {
            let mut urls = Vec::with_capacity(1000);
            for trn in &trns {
                if let Ok(url) = trn.to_url() {
                    urls.push(url);
                }
            }
            urls
        })
    });
}

fn bench_url_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("url_string_ops");
    
    let trn = Trn::new("user", "alice", "tool", "myapi", "v1.0").unwrap();
    
    group.bench_function("to_string_then_url", |b| {
        b.iter(|| {
            let trn_str = trn.to_string();
            let parsed_trn = Trn::parse(&trn_str).unwrap();
            parsed_trn.to_url().unwrap()
        })
    });
    
    group.bench_function("direct_to_url", |b| {
        b.iter(|| trn.to_url().unwrap())
    });
    
    group.finish();
}

fn bench_url_encoding_decoding(c: &mut Criterion) {
    // Test TRN with characters that need URL encoding
    let trn = Trn::new("user", "alice", "tool", "my api", "v1.0").unwrap();
    
    c.bench_function("url_encoding", |b| {
        b.iter(|| {
            let url = trn.to_url().unwrap();
            url_to_trn(&url).unwrap()
        })
    });
}

fn bench_concurrent_url_conversion(c: &mut Criterion) {
    use std::sync::Arc;
    
    let trns: Arc<Vec<Trn>> = Arc::new(
        (0..100)
            .map(|i| Trn::new("user", &format!("user{}", i), "tool", &format!("api{}", i), "v1.0").unwrap())
            .collect()
    );
    
    c.bench_function("concurrent_url_conversion", |b| {
        b.iter(|| {
            let trns_clone = trns.clone();
            let handles: Vec<_> = (0..4).map(|_| {
                let trns = trns_clone.clone();
                std::thread::spawn(move || {
                    for trn in trns.iter() {
                        let _ = trn.to_url();
                    }
                })
            }).collect();
            
            for handle in handles {
                handle.join().unwrap();
            }
        })
    });
}

criterion_group!(
    benches,
    bench_trn_to_url,
    bench_trn_to_http_url,
    bench_url_to_trn,
    bench_url_roundtrip,
    bench_url_conversion_batch,
    bench_url_conversion_different_platforms,
    bench_http_url_different_bases,
    bench_url_with_special_characters,
    bench_url_parsing_performance,
    bench_url_error_handling,
    bench_url_memory_allocation,
    bench_url_string_operations,
    bench_url_encoding_decoding,
    bench_concurrent_url_conversion
);

criterion_main!(benches); 