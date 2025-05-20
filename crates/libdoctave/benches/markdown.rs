#[path = "../tests/common.rs"]
mod common;

use common::ProjectBuilder;

use criterion::{criterion_group, criterion_main, Criterion};
use libdoctave::RenderOptions;

fn bench_markdown(c: &mut Criterion) {
    let mut group = c.benchmark_group("markdown");

    let files = ProjectBuilder::n_project_files(3001);

    group.sample_size(10);

    group.bench_with_input("3000_md_files", &files, |b, files| {
        b.iter(|| {
            // There is some overhead from cloning the files, but it doesn't seem
            // possible to move setup values into the benchmark.
            ProjectBuilder {
                inputs: files.clone(),
            }
            .build()
            .unwrap()
            .verify(None, None)
        })
    });

    group.bench_with_input(
        "3000_md_files no syntax highlighting",
        &files,
        |b, files| {
            b.iter(|| {
                // There is some overhead from cloning the files, but it doesn't seem
                // possible to move setup values into the benchmark.
                ProjectBuilder {
                    inputs: files.clone(),
                }
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
        },
    );

    group.finish();
}

criterion_group!(benches, bench_markdown);
criterion_main!(benches);
