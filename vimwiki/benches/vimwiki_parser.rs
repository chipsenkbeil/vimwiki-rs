use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::{fs, path::PathBuf, time::Duration};
use vimwiki::{Language, Page};

fn parse_page_benchmark(c: &mut Criterion) {
    let base =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("benches/fixtures");
    let target = PathBuf::from("vimwiki/specification.wiki");
    let path = base.join(target);
    let file_contents =
        fs::read_to_string(path).expect("Failed to load fixture");

    c.bench_with_input(
        BenchmarkId::new("parse page", "specification.wiki"),
        &file_contents,
        |b, s| {
            let language = Language::from_vimwiki_str(&s);
            b.iter(|| language.parse::<Page>().expect("Failed to parse"))
        },
    );
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(Duration::new(15, 0));
    targets = parse_page_benchmark
}
criterion_main!(benches);
