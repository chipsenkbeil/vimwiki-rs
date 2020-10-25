use criterion::{criterion_group, criterion_main, Criterion};
use std::{fs, path::PathBuf, time::Duration};
use vimwiki::{elements::*, Language};

fn parse_page_benchmark(c: &mut Criterion) {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let target = PathBuf::from("vimwiki/specification.wiki");
    let path = PathBuf::from(base).join(target);
    let file_contents =
        fs::read_to_string(path).expect("Failed to load fixture");
    let language = Language::from_vimwiki_string(file_contents);

    c.bench_function("parse specification.wiki", |b| {
        b.iter(|| language.parse::<Page>().expect("Failed to parse"))
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(Duration::new(15, 0));
    targets = parse_page_benchmark
}
criterion_main!(benches);
