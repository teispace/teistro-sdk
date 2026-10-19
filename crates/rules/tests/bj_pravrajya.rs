//! Brihat Jataka ch. 15's pravrajya yogas, held to the 93 recorded charts:
//! what each answers, and that the yogas the chapter states as BPHS ch. 79
//! does answer what BPHS's do.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they read"
)]

mod common;

use std::collections::BTreeMap;

use common::{chart_at, files_in, vargas};
use teistro_rules::{Evaluator, Readings, Rule, shipped};

/// The rules of one Brihat Jataka chapter.
fn chapter(number: &str) -> Vec<Rule> {
    shipped::nabhasas()
        .iter()
        .filter(|rule| {
            rule.source.text == "Brihat Jataka" && rule.source.chapter.as_deref() == Some(number)
        })
        .cloned()
        .collect()
}

/// The yogas the chapter states as BPHS ch. 79 does, each with BPHS's. The
/// conditions are one; v. 1's orders are paired with the grahas otherwise,
/// which is the outcome's and not the yoga's.
const TWINS: [(&str, &str); 9] = [
    ("SUN", "PRAVRAJYA_SUN"),
    ("MOON", "PRAVRAJYA_MOON"),
    ("MARS", "PRAVRAJYA_MARS"),
    ("MERCURY", "PRAVRAJYA_MERCURY"),
    ("JUPITER", "PRAVRAJYA_JUPITER"),
    ("VENUS", "PRAVRAJYA_VENUS"),
    ("SATURN", "PRAVRAJYA_SATURN"),
    (
        "MOON_LORD_ASPECTS_SATURN_AND_IS_STRONGER",
        "PRAVRAJYA_MOON_LORD_ASPECTS_SATURN_AND_IS_STRONGER",
    ),
    (
        "MOON_LORD_ASPECTS_SATURN_AND_SATURN_IS_STRONGER",
        "PRAVRAJYA_MOON_LORD_ASPECTS_SATURN_AND_SATURN_IS_STRONGER",
    ),
];

#[test]
fn the_ascetics_answer_and_their_twins_agree() {
    let rules = chapter("15");
    assert_eq!(rules.len(), 19);
    for rule in &rules {
        assert!(rule.is_evaluable(), "{} is evaluable", rule.key);
        assert!(rule.source.verse.is_some(), "{}: cites a verse", rule.key);
    }
    // V. 4's raja yoga names ch. 11's by key, so the set they are asked in
    // holds both chapters.
    let mut set = rules.clone();
    set.extend(chapter("11"));
    teistro_rules::check_references(&set).expect("the chapter names ch. 11 soundly");
    let find = |key: &str| {
        shipped::nabhasas()
            .iter()
            .find(|rule| rule.key == key)
            .cloned()
            .expect("the twin ships")
    };
    let twins: Vec<(Rule, Rule)> = TWINS
        .iter()
        .map(|(ours, theirs)| (find(&format!("BJ_PRAVRAJYA_{ours}")), find(theirs)))
        .collect();
    for (ours, theirs) in &twins {
        assert_eq!(ours.conditions, theirs.conditions, "{}", ours.key);
    }
    let mut fired: BTreeMap<&str, usize> =
        rules.iter().map(|rule| (rule.key.as_str(), 0)).collect();
    for (path, file) in files_in("doshas") {
        let chart = chart_at(&path, &file["inputs"]);
        let divisions = vargas(&chart);
        let evaluator = Evaluator::new(&chart, Readings::TEXTS)
            .with_vargas(&divisions)
            .with_rules(&set);
        for rule in &rules {
            *fired.get_mut(rule.key.as_str()).unwrap() +=
                usize::from(evaluator.evaluate(rule).present);
        }
        for (ours, theirs) in &twins {
            assert_eq!(
                evaluator.evaluate(ours).present,
                evaluator.evaluate(theirs).present,
                "{} in {path:?}",
                ours.key
            );
        }
    }
    let counts: Vec<(&str, usize)> = fired.into_iter().collect();
    assert_eq!(counts.as_slice(), ANSWERED.as_slice());
}

/// What each answers over the 93 recorded charts.
const ANSWERED: [(&str, usize); 19] = [
    ("BJ_PRAVRAJYA_JUPITER", 0),
    ("BJ_PRAVRAJYA_JUPITER_COMBUST", 0),
    (
        "BJ_PRAVRAJYA_JUPITER_NINTH_JUPITER_MOON_AND_LAGNA_ASPECTED_BY_SATURN_IN_A_RAJA_YOGA",
        0,
    ),
    ("BJ_PRAVRAJYA_MARS", 0),
    ("BJ_PRAVRAJYA_MARS_COMBUST", 0),
    ("BJ_PRAVRAJYA_MERCURY", 0),
    ("BJ_PRAVRAJYA_MERCURY_COMBUST", 0),
    ("BJ_PRAVRAJYA_MOON", 0),
    ("BJ_PRAVRAJYA_MOON_COMBUST", 0),
    (
        "BJ_PRAVRAJYA_MOON_IN_SATURNS_DREKKANA_AND_A_NAVAMSHA_OF_MARS_OR_SATURN_ASPECTED_BY_SATURN",
        3,
    ),
    ("BJ_PRAVRAJYA_MOON_LORD_ASPECTS_SATURN_AND_IS_STRONGER", 0),
    (
        "BJ_PRAVRAJYA_MOON_LORD_ASPECTS_SATURN_AND_SATURN_IS_STRONGER",
        0,
    ),
    ("BJ_PRAVRAJYA_SATURN", 1),
    ("BJ_PRAVRAJYA_SATURN_ASPECTS_THE_WEAK_MOON_LORD", 4),
    ("BJ_PRAVRAJYA_SATURN_COMBUST", 1),
    ("BJ_PRAVRAJYA_SATURN_NINTH_UNASPECTED_IN_A_RAJA_YOGA", 1),
    ("BJ_PRAVRAJYA_SUN", 0),
    ("BJ_PRAVRAJYA_VENUS", 0),
    ("BJ_PRAVRAJYA_VENUS_COMBUST", 0),
];
