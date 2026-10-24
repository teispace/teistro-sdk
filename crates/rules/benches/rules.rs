//! The rules kernel's budget (`docs/03-design/rules-engine.md`,
//! "Performance"): 900 rules under 2 milliseconds on the native path, a
//! rate of 2.22 µs a rule. This times every rule the crate ships
//! (`shipped::packs`) that is evaluable, over one recorded chart, with the
//! evaluator's construction included, and counts them as the group's
//! elements, so `cargo xtask measure-micro` holds it to the rate times the
//! count (`03-design/micro-budgets.md`). The construction is timed alone too.

#![allow(
    missing_docs,
    clippy::indexing_slicing,
    reason = "a benchmark binary indexes the fixture it reads"
)]

#[path = "../tests/common/mod.rs"]
mod common;

use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use teistro_rules::{Evaluator, Readings, Rule, RuleChart};

/// Every shipped rule the evaluator can read, and one recorded chart.
fn fixture() -> (Vec<&'static Rule>, RuleChart) {
    let rules = teistro_rules::shipped::packs()
        .into_iter()
        .flatten()
        .filter(|rule| rule.is_evaluable())
        .collect();
    let (_, file) = common::files().swap_remove(0);
    (rules, common::chart(&file["inputs"]))
}

fn bench(c: &mut Criterion) {
    let (rules, chart) = fixture();
    c.bench_function("evaluator construction", |b| {
        b.iter(|| Evaluator::new(black_box(&chart), Readings::RECORDING_ENGINE));
    });
    let mut group = c.benchmark_group("rules");
    group.throughput(Throughput::Elements(
        u64::try_from(rules.len()).unwrap_or(u64::MAX),
    ));
    group.bench_function("the shipped set over one chart", |b| {
        b.iter(|| {
            let evaluator = Evaluator::new(black_box(&chart), Readings::RECORDING_ENGINE);
            rules
                .iter()
                .filter(|rule| evaluator.evaluate(rule).present)
                .count()
        });
    });
    group.bench_function("the shipped set explained over one chart", |b| {
        b.iter(|| {
            let evaluator = Evaluator::new(black_box(&chart), Readings::RECORDING_ENGINE);
            rules
                .iter()
                .filter(|rule| evaluator.explain(rule).result.present)
                .count()
        });
    });
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
