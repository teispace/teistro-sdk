//! In whose periods a result is delivered, held over the 93 recorded charts
//! against what the verses say directly of each chart, not against the
//! evaluator's own reading of the rule.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they read"
)]

mod common;

use common::{chart_at, files_in};
use teistro_core::catalogue::{Graha, Rashi};
use teistro_rules::{Body, Evaluator, House, Levels, Readings, Running, Timing, shipped};

const GRAHAS: [Graha; 9] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
    Graha::Rahu,
    Graha::Ketu,
];

#[test]
fn a_result_is_delivered_in_the_periods_of_what_formed_it() {
    let rules = shipped::nabhasas();
    let find = |key: &str| rules.iter().find(|rule| rule.key == key).unwrap();
    let wealth = find("BPHS_NINTH_AND_FIFTH_LORDS_AND_THEIR_COMPANIONS_GIVE_WEALTH");
    let (mut charts, mut givers, mut throughout) = (0, 0, 0);
    for (path, file) in files_in("doshas") {
        let chart = chart_at(&path, &file["inputs"]);
        let evaluator = Evaluator::new(&chart, Readings::TEXTS).with_rules(rules);
        let sign_of = |graha: Graha| chart.placement(Body::Graha(graha)).sign;
        let lord_of = |house: u8| {
            evaluator
                .house_sign(House::try_new(house).unwrap())
                .attributes()
                .lord
        };

        // BPHS ch. 41 v. 16, read off the chart directly: the ninth and fifth
        // lords, and every graha sharing a sign with either.
        let (ninth, fifth) = (lord_of(9), lord_of(5));
        let giver = |graha: Graha| {
            graha == ninth
                || graha == fifth
                || sign_of(graha) == sign_of(ninth)
                || sign_of(graha) == sign_of(fifth)
        };
        let result = evaluator.evaluate(wealth);
        assert!(result.present, "{}", path.display());
        for graha in GRAHAS {
            let delivered = !evaluator
                .delivery(wealth, &result, [Running::graha(graha)])
                .is_empty();
            assert_eq!(delivered, giver(graha), "{graha:?} in {}", path.display());
            givers += usize::from(delivered);
        }

        for rule in rules {
            let result = evaluator.evaluate(rule);
            // A sign's period delivers a result formed there: in a sign-based
            // dasha the lord it is given does not decide, the sign does.
            for sign in Rashi::ALL {
                let formed_there = result
                    .participants
                    .iter()
                    .any(|body| chart.placement(body).sign == sign);
                let delivered = !evaluator
                    .delivery(rule, &result, [Running::sign(sign, Graha::Sun)])
                    .is_empty();
                let expected =
                    result.present && (rule.timing == Timing::Throughout || formed_there);
                assert_eq!(delivered, expected, "{} in {sign:?}", rule.key);
            }
            // A rule given throughout delivers at every level of any chain.
            if result.present && rule.timing == Timing::Throughout {
                let chain = GRAHAS.map(Running::graha);
                assert_eq!(
                    evaluator.delivery(rule, &result, chain),
                    Levels::all(Levels::MAX)
                );
                throughout += 1;
            }
        }
        charts += 1;
    }
    // Three or four wealth-giving grahas a chart on average, and 168 Nabhasa
    // results, every chart answering one sankhya yoga at least, each delivered
    // in every period.
    assert_eq!((charts, givers, throughout), (93, 327, 168));
}
