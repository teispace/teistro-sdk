//! The rules kernel's budget (`docs/03-design/rules-engine.md`,
//! "Performance"): 900 rules under 2 milliseconds on the native path. This
//! times the recording engine's 597 written rules over one recorded chart,
//! the evaluator's construction included, and the construction alone.

#![allow(
    missing_docs,
    clippy::indexing_slicing,
    reason = "a benchmark binary indexes the fixture it reads"
)]

#[path = "../tests/common/mod.rs"]
mod common;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use teistro_rules::{Evaluator, Readings, Rule};

fn bench(c: &mut Criterion) {
    let rules: Vec<Rule> = common::rules()
        .into_iter()
        .filter(Rule::is_evaluable)
        .collect();
    let (_, file) = common::files().swap_remove(0);
    let chart = common::chart(&file["inputs"]);

    c.bench_function("evaluator construction", |b| {
        b.iter(|| Evaluator::new(black_box(&chart), Readings::RECORDING_ENGINE));
    });
    c.bench_function("597 rules over one chart", |b| {
        b.iter(|| {
            let evaluator = Evaluator::new(black_box(&chart), Readings::RECORDING_ENGINE);
            rules
                .iter()
                .filter(|rule| evaluator.evaluate(rule).present)
                .count()
        });
    });
}

fn explain(c: &mut Criterion) {
    let rules: Vec<Rule> = common::rules()
        .into_iter()
        .filter(Rule::is_evaluable)
        .collect();
    let (_, file) = common::files().swap_remove(0);
    let chart = common::chart(&file["inputs"]);
    c.bench_function("597 rules explained over one chart", |b| {
        b.iter(|| {
            let evaluator = Evaluator::new(black_box(&chart), Readings::RECORDING_ENGINE);
            rules
                .iter()
                .filter(|rule| evaluator.explain(rule).result.present)
                .count()
        });
    });
}

criterion_group!(benches, bench, explain);
criterion_main!(benches);
