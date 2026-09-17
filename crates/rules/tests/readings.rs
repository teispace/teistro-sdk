//! Which readings the texts settle, measured
//! (`03-design/rules-engine.md`, "What the texts settle, and what they leave").
//!
//! `Readings::TEXTS` is meant to be "the texts' reading wherever a text read
//! settles one, the recording engine's elsewhere". This pass holds that claim
//! to the corpus: for each knob, how many of the SDK's own answers move when
//! it is flipped. A knob a text settles should be set to what the text says,
//! and a knob that moves nothing should say so rather than look decided.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they read"
)]

mod common;

use common::{chart_at, files_in};
use teistro_rules::{Evaluator, Houses, NodeMotion, Readings, Rule, shipped};

/// Everything the SDK writes from a text.
fn rules() -> Vec<Rule> {
    shipped::arishtas()
        .iter()
        .chain(shipped::gandantas())
        .chain(shipped::nabhasas())
        .chain(shipped::readings())
        .cloned()
        .collect()
}

/// How many of the SDK's answers differ between two readings, over the 93.
fn moved(one: Readings, other: Readings, rules: &[Rule]) -> usize {
    let mut moved = 0;
    for (path, file) in files_in("doshas") {
        let chart = chart_at(&path, &file["inputs"]);
        let here = Evaluator::new(&chart, one).with_rules(rules);
        let there = Evaluator::new(&chart, other).with_rules(rules);
        for rule in rules {
            if here.evaluate(rule).present != there.evaluate(rule).present {
                moved += 1;
            }
        }
    }
    moved
}

#[test]
fn what_each_reading_moves_over_the_sdk_s_own_rules() {
    let rules = rules();
    assert_eq!(rules.len(), 972);

    // The one knob `TEXTS` already sets against the engine: how a table's
    // degree is counted (crux C82). None of the rules the SDK writes from a
    // text reads a degree table — those are the engine's own doshas — so it
    // moves nothing here, and the corpus says so rather than the page
    // claiming it.
    assert_eq!(
        moved(Readings::TEXTS, Readings::RECORDING_ENGINE, &rules),
        0
    );

    // The two knobs the texts settle and `TEXTS` now sets against the
    // engine's. Neither moves an answer on this corpus — the nodes' motion
    // because no rule the SDK writes asks whether a node is retrograde, the
    // houses because every house the corpus records is already whole-sign —
    // so **the corpus cannot decide either, and the verses did**. What the
    // measurement buys is the knowledge that the change is safe.
    let as_engine = Readings {
        node_motion: NodeMotion::NeverRetrograde,
        ..Readings::TEXTS
    };
    assert_eq!(moved(Readings::TEXTS, as_engine, &rules), 0);
    let as_engine = Readings {
        houses: Houses::Recorded,
        ..Readings::TEXTS
    };
    assert_eq!(moved(Readings::TEXTS, as_engine, &rules), 0);

    // And the two are set, so a later edit that quietly reverted them would
    // fail here rather than pass silently on a corpus that cannot see it.
    assert_eq!(Readings::TEXTS.node_motion, NodeMotion::AlwaysRetrograde);
    assert_eq!(Readings::TEXTS.houses, Houses::WholeSign);
    assert_eq!(
        Readings::RECORDING_ENGINE.node_motion,
        NodeMotion::NeverRetrograde
    );
    assert_eq!(Readings::RECORDING_ENGINE.houses, Houses::Recorded);
}
