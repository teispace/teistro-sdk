//! BPHS ch. 43's Pindayu, Nisargayu and Amsayu over the 93 recorded charts:
//! what the verses' arithmetic says must hold of every chart, and where the
//! spans fall.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they read"
)]

mod common;

use common::{chart_at, files_in};
use teistro_rules::longevity::{AyurdayaRules, Giver, Method, Nisarga, full_years};
use teistro_rules::{Evaluator, Readings};

#[test]
fn every_span_is_the_sum_of_what_its_givers_give_within_their_bounds() {
    let mut spans: [Vec<f64>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    for (path, file) in files_in("doshas") {
        let chart = chart_at(&path, &file["inputs"]);
        let evaluator = Evaluator::new(&chart, Readings::TEXTS);
        let listed = evaluator.ayurdaya(AyurdayaRules {
            nisarga: Nisarga::Listed,
            ..AyurdayaRules::default()
        });
        assert!(
            (listed.nisargayu.years - 120.0).abs() < 1e-9,
            "{}",
            path.display()
        );
        let reading = evaluator.ayurdaya(AyurdayaRules::default());
        for (at, span) in [reading.pindayu, reading.nisargayu, reading.amsayu]
            .iter()
            .enumerate()
        {
            let sum: f64 = span.contributions.iter().map(|given| given.net).sum();
            assert!((sum - span.years).abs() < 1e-9);
            for given in &span.contributions {
                assert!(
                    given.net >= 0.0 && given.net <= given.basic + 1e-9,
                    "{given:?}"
                );
                match (span.method, given.giver) {
                    // Half at deep debilitation, the whole at deep exaltation.
                    (Method::Pindayu | Method::Nisargayu, Giver::Graha(graha)) => {
                        let full = full_years(span.method, graha).unwrap();
                        assert!(given.basic >= full / 2.0 - 1e-9 && given.basic <= full + 1e-9);
                    }
                    // A year a navamsha, less twelves; a year a sign.
                    (_, _) => assert!(given.basic >= 0.0 && given.basic < 12.0 + 1e-9),
                }
            }
            spans[at].push(span.years);
        }
    }
    let summary: Vec<(f64, f64, f64)> = spans
        .iter()
        .map(|years| {
            let low = years.iter().copied().fold(f64::INFINITY, f64::min);
            let high = years.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let mean = years.iter().sum::<f64>() / 93.0;
            let round = |value: f64| (value * 100.0).round() / 100.0;
            (round(low), round(mean), round(high))
        })
        .collect();
    // Lowest, mean and highest over the 93, in years of 360 days: Pindayu and
    // Nisargayu near their full 127 and 120 less the reductions; Amsayu near
    // the 48 that eight givers of 0 to 12 years average, less the same.
    assert_eq!(
        summary,
        [
            (56.84, 85.01, 109.09),
            (38.43, 78.94, 99.29),
            (20.37, 38.92, 66.14)
        ]
    );
}
