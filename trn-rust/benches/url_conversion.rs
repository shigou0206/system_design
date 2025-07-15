use criterion::{black_box, criterion_group, criterion_main, Criterion};
use trn_rust::Trn;
use trn_rust::url_to_trn;

fn bench_trn_to_url(c: &mut Criterion) {
    let trn = Trn::parse("trn:user:alice:tool:openapi:github-api:v1.0").unwrap();
    c.bench_function("trn_to_url", |b| {
        b.iter(|| trn.to_url())
    });
}

fn bench_trn_to_http_url(c: &mut Criterion) {
    let trn = Trn::parse("trn:user:alice:tool:openapi:github-api:v1.0").unwrap();
    let base = "https://api.example.com/";
    
    c.bench_function("trn_to_http_url", |b| {
        b.iter(|| trn.to_http_url(black_box(base)))
    });
}

fn bench_url_to_trn(c: &mut Criterion) {
    let url = "trn://user/alice/tool/openapi/github-api/v1.0";
    c.bench_function("url_to_trn", |b| {
        b.iter(|| url_to_trn(black_box(url)))
    });
}

fn bench_bidirectional_conversion(c: &mut Criterion) {
    let trn_str = "trn:user:alice:tool:openapi:github-api:v1.0";
    c.bench_function("bidirectional_conversion", |b| {
        b.iter(|| {
            let trn = Trn::parse(black_box(trn_str)).unwrap();
            let url = trn.to_url().unwrap();
            let _back_to_trn = url_to_trn(&url).unwrap();
        })
    });
}

criterion_group!(benches, bench_trn_to_url, bench_trn_to_http_url, bench_url_to_trn, bench_bidirectional_conversion);
criterion_main!(benches); 