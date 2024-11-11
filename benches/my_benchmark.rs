use std::collections::BTreeMap;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rule_match::{bag, LinearScan, LowCardinalityTree, Matcher, Rule};

fn make_rules() -> Vec<Rule> {
    itertools::iproduct!(0..10, 0..10, 0..10).map(|(i, j, k)| {
        let mut r = BTreeMap::new();
        if i > 0 {
            r.insert("a".to_owned(), i.to_string());
        }
        if j > 0 {
            r.insert("b".to_owned(), j.to_string());
        }
        if k > 0 {
            r.insert("c".to_owned(), k.to_string());
        }
        Rule(r)
    }).skip(1).collect()
}

fn linear_scan(c: &mut Criterion) {
    let m = LinearScan::new(make_rules());

    c.bench_function("no_match", |b| b.iter(|| {
        m.find(black_box(bag! { "a" => "garbage", "b" => "garbage", "c" => "garbage" }))
    }));
}

fn low_cardinality_tree(c: &mut Criterion) {
    let m = LowCardinalityTree::new(make_rules());

    c.bench_function("no_match", |b| b.iter(|| {
        m.find(black_box(bag! { "a" => "garbage", "b" => "garbage", "c" => "garbage" }))
    }));
}

criterion_group!(benches, linear_scan, low_cardinality_tree);
criterion_main!(benches);
