use criterion::{criterion_group, criterion_main, Criterion};

static LARGE_SPEC: &str = include_str!("../../libdoctave/examples/open_api_specs/github.yaml");

fn bench_openapi(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_openapi");

    group.sample_size(10);

    group.bench_function("parse", |b| {
        b.iter(|| openapi_parser::openapi30::parser::parse_yaml(LARGE_SPEC))
    });

    group.finish();
}

criterion_group!(benches, bench_openapi);
criterion_main!(benches);
