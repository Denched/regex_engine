use criterion::{Criterion, criterion_group, criterion_main};
use regex_engine::{compile_search, parse, run};
use std::hint::black_box;
// head-to-head against the production `regex` crate
fn vs_regex_crate(c: &mut Criterion) {
    // pathological cases
    for n in [10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30] {
        let pattern = "(a+)+b";
        let input = "a".repeat(n);

        let mut group = c.benchmark_group(format!("vs_crate_pathological_n{}", n));

        let mine = compile_search(&parse(pattern).unwrap());
        group.bench_function("regex_engine", |b| {
            b.iter(|| run(black_box(&mine), black_box(&input)))
        });

        let theirs = regex::Regex::new(pattern).unwrap();
        group.bench_function("regex_crate", |b| {
            b.iter(|| theirs.is_match(black_box(&input)))
        });

        group.finish();
    }

    // realistic cases
    let cases = [
        ("literal", "hello", "hello world hello"),
        ("star", "a*b", "aaaaaaaaaab"),
        ("alternation", "(cat|dog|bird)", "I have a bird at home"),
        ("anchored", "^abc$", "abc"),
    ];

    for (name, pattern, input) in cases {
        let mut group = c.benchmark_group(format!("vs_crate_{}", name));

        let mine = compile_search(&parse(pattern).unwrap());
        group.bench_function("regex_engine", |b| {
            b.iter(|| run(black_box(&mine), black_box(input)))
        });

        let theirs = regex::Regex::new(pattern).unwrap();
        group.bench_function("regex_crate", |b| {
            b.iter(|| theirs.is_match(black_box(input)))
        });

        group.finish();
    }
}
criterion_group!(benches, vs_regex_crate);
criterion_main!(benches);
