#[path = "../tests/common.rs"]
mod common;

use common::ProjectBuilder;

use criterion::{criterion_group, criterion_main, Criterion};
use libdoctave::RenderOptions;

static LARGE_SPEC: &str = include_str!("../examples/open_api_specs/large_openapi.json");

fn bench_openapi(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_openapi");

    group.sample_size(10);

    group.bench_function("parse", |b| {
        b.iter(|| {
            ProjectBuilder::default()
                .with_openapi(LARGE_SPEC.to_owned())
                .build()
        })
    });

    group.bench_function("parse & render one page", |b| {
        b.iter(|| {
            ProjectBuilder::default()
                .with_openapi(LARGE_SPEC.to_owned())
                .build()
                .unwrap()
                .get_page_by_uri_path("/api/actions")
                .unwrap()
                .ast(None)
        })
    });

    group.bench_function("parse & verify", |b| {
        b.iter(|| {
            ProjectBuilder::default()
                .with_openapi(LARGE_SPEC.to_owned())
                .build()
                .unwrap()
                .verify(None, None)
        })
    });

    group.bench_function("parse & verify without syntax highlighting", |b| {
        b.iter(|| {
            ProjectBuilder::default()
                .with_openapi(LARGE_SPEC.to_owned())
                .build()
                .unwrap()
                .verify(
                    Some(&RenderOptions {
                        disable_syntax_highlighting: true,
                        ..RenderOptions::default()
                    }),
                    None,
                )
        })
    });

    group.finish();
}

criterion_group!(benches, bench_openapi);
criterion_main!(benches);
