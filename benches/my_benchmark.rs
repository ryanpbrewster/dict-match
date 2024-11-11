use std::collections::BTreeMap;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rule_match::{bag, LinearScan, Matcher, Rule};

fn criterion_benchmark(c: &mut Criterion) {
    let rules: Vec<Rule> = itertools::iproduct!(0..10, 0..10, 0..10).map(|(i, j, k)| {
        Rule(BTreeMap::from([
            ("a".to_owned(), i.to_string()),
            ("b".to_owned(), j.to_string()),
            ("c".to_owned(), k.to_string()),
        ]))
    }).collect();
    let m = LinearScan::new(rules);

    c.bench_function("no_match", |b| b.iter(|| {
        m.find(black_box(bag! { "a" => "garbage", "b" => "garbage", "c" => "garbage" }))
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
