//! Saravali chs. 13 and 14, the grahas either side of the Moon and of the Sun,
//! held to the 93 recorded charts: what each answers, and that each yoga in
//! general holds exactly where one of its grahas' readings does.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they read"
)]

mod common;

use std::collections::BTreeMap;

use common::{chart_at, files_in};
use teistro_rules::{Evaluator, Readings, Rule, shipped};

/// Every rule the two chapters ship.
fn chapters() -> Vec<Rule> {
    shipped::nabhasas()
        .iter()
        .filter(|rule| {
            rule.source.text == "Saravali"
                && matches!(rule.source.chapter.as_deref(), Some("13" | "14"))
        })
        .cloned()
        .collect()
}

/// The yogas the chapters read in general and again graha by graha.
const NAMED: [&str; 5] = ["SUNAPHA", "ANAPHA", "DURUDHARA", "VESHI", "VOSHI"];

#[test]
fn a_yoga_holds_where_one_of_its_grahas_makes_it() {
    let rules = chapters();
    assert_eq!(rules.len(), 36);
    teistro_rules::check_references(&rules).expect("the chapters name themselves soundly");
    let mut fired: BTreeMap<&str, usize> =
        rules.iter().map(|rule| (rule.key.as_str(), 0)).collect();
    for (path, file) in files_in("doshas") {
        let chart = chart_at(&path, &file["inputs"]);
        let evaluator = Evaluator::new(&chart, Readings::TEXTS).with_rules(&rules);
        let holds: BTreeMap<&str, bool> = rules
            .iter()
            .map(|rule| (rule.key.as_str(), evaluator.evaluate(rule).present))
            .collect();
        for (key, present) in &holds {
            *fired.get_mut(key).unwrap() += usize::from(*present);
        }
        for yoga in NAMED {
            let general = holds[format!("SARAVALI_{yoga}").as_str()];
            let by_graha = holds
                .iter()
                .any(|(key, present)| *present && key.starts_with(&format!("SARAVALI_{yoga}_")));
            assert_eq!(general, by_graha, "{yoga} in {path:?}");
        }
    }
    let counts: Vec<(&str, usize)> = fired.into_iter().collect();
    assert_eq!(counts.as_slice(), ANSWERED.as_slice());
}

/// What each answers over the 93 recorded charts.
const ANSWERED: [(&str, usize); 36] = [
    ("SARAVALI_ANAPHA", 12),
    ("SARAVALI_ANAPHA_JUPITER", 2),
    ("SARAVALI_ANAPHA_MARS", 2),
    ("SARAVALI_ANAPHA_MERCURY", 1),
    ("SARAVALI_ANAPHA_SATURN", 3),
    ("SARAVALI_ANAPHA_VENUS", 6),
    ("SARAVALI_DURUDHARA", 7),
    ("SARAVALI_DURUDHARA_JUPITER_SATURN", 3),
    ("SARAVALI_DURUDHARA_JUPITER_VENUS", 1),
    ("SARAVALI_DURUDHARA_MARS_JUPITER", 3),
    ("SARAVALI_DURUDHARA_MARS_MERCURY", 3),
    ("SARAVALI_DURUDHARA_MARS_SATURN", 0),
    ("SARAVALI_DURUDHARA_MARS_VENUS", 4),
    ("SARAVALI_DURUDHARA_MERCURY_JUPITER", 0),
    ("SARAVALI_DURUDHARA_MERCURY_SATURN", 3),
    ("SARAVALI_DURUDHARA_MERCURY_VENUS", 3),
    ("SARAVALI_DURUDHARA_VENUS_SATURN", 1),
    ("SARAVALI_SUNAPHA", 17),
    ("SARAVALI_SUNAPHA_JUPITER", 6),
    ("SARAVALI_SUNAPHA_MARS", 5),
    ("SARAVALI_SUNAPHA_MERCURY", 3),
    ("SARAVALI_SUNAPHA_SATURN", 7),
    ("SARAVALI_SUNAPHA_VENUS", 5),
    ("SARAVALI_UBHAYACHARI", 33),
    ("SARAVALI_VESHI", 17),
    ("SARAVALI_VESHI_JUPITER", 1),
    ("SARAVALI_VESHI_MARS", 1),
    ("SARAVALI_VESHI_MERCURY", 13),
    ("SARAVALI_VESHI_SATURN", 1),
    ("SARAVALI_VESHI_VENUS", 14),
    ("SARAVALI_VOSHI", 18),
    ("SARAVALI_VOSHI_JUPITER", 2),
    ("SARAVALI_VOSHI_MARS", 10),
    ("SARAVALI_VOSHI_MERCURY", 5),
    ("SARAVALI_VOSHI_SATURN", 5),
    ("SARAVALI_VOSHI_VENUS", 13),
];
