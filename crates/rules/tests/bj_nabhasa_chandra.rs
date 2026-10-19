//! Brihat Jataka ch. 12's Nabhasa yogas and ch. 13's yogas of the Moon, held
//! to the 93 recorded charts: what each answers, that those the chapters
//! state as another text does answer what that text's do, and that the
//! figures the verses set apart never meet.

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

/// Every rule the two chapters ship.
fn chapters() -> Vec<Rule> {
    shipped::nabhasas()
        .iter()
        .filter(|rule| {
            rule.source.text == "Brihat Jataka"
                && matches!(rule.source.chapter.as_deref(), Some("12" | "13"))
        })
        .cloned()
        .collect()
}

/// The yogas the chapters state as another text does, each with that text's.
const TWINS: [(&str, &str); 29] = [
    ("NABHASA_GADA", "NABHASA_GADA"),
    ("NABHASA_SAKATA", "NABHASA_SAKATA"),
    ("NABHASA_VIHAGA", "NABHASA_VIHAGA"),
    ("NABHASA_SRINGATAKA", "NABHASA_SRINGATAKA"),
    ("NABHASA_HALA", "NABHASA_HALA"),
    ("NABHASA_KAMALA", "NABHASA_KAMALA"),
    ("NABHASA_VAPI", "NABHASA_VAPI"),
    ("NABHASA_YUPA", "NABHASA_YUPA"),
    ("NABHASA_SHARA", "NABHASA_SARA"),
    ("NABHASA_SHAKTI", "NABHASA_SAKTHI"),
    ("NABHASA_DANDA", "NABHASA_DANDA"),
    ("NABHASA_NAUKA", "NABHASA_NAUKA"),
    ("NABHASA_KUTA", "NABHASA_KOOTA"),
    ("NABHASA_CHHATRA", "NABHASA_CHATRA"),
    ("NABHASA_CHAPA", "NABHASA_CHAPA"),
    ("NABHASA_ARDHA_CHANDRA", "NABHASA_ARDHA_CHANDRA"),
    ("NABHASA_SAMUDRA", "NABHASA_SAMUDRA"),
    ("NABHASA_CHAKRA", "NABHASA_CHAKRA"),
    (
        "CHANDRA_MOON_IN_KENDRA_FROM_THE_SUN",
        "LUNAR_MOON_IN_KENDRA_FROM_THE_SUN",
    ),
    (
        "CHANDRA_MOON_IN_PANAPHARA_FROM_THE_SUN",
        "LUNAR_MOON_IN_PANAPHARA_FROM_THE_SUN",
    ),
    (
        "CHANDRA_MOON_IN_APOKLIMA_FROM_THE_SUN",
        "LUNAR_MOON_IN_APOKLIMA_FROM_THE_SUN",
    ),
    (
        "CHANDRA_MOON_IN_OWN_OR_A_GREAT_FRIENDS_NAVAMSHA_ASPECTED_BY_JUPITER_BY_DAY_VENUS_BY_NIGHT",
        "PARIJATA_MOON_IN_OWN_OR_A_GREAT_FRIENDS_NAVAMSHA_ASPECTED_BY_JUPITER_BY_DAY_VENUS_BY_NIGHT",
    ),
    ("CHANDRA_ADHI", "LUNAR_ADHI_YOGA"),
    ("CHANDRA_SUNAPHA", "PHALADEEPIKA_SUNAPHA"),
    ("CHANDRA_ANAPHA", "PHALADEEPIKA_ANAPHA"),
    ("CHANDRA_DURUDHARA", "PHALADEEPIKA_DURUDHARA"),
    (
        "CHANDRA_ALL_BENEFICS_IN_UPACHAYA_FROM_THE_MOON",
        "LUNAR_DHANA_THREE_BENEFICS_IN_UPACHAYA",
    ),
    (
        "CHANDRA_TWO_BENEFICS_IN_UPACHAYA_FROM_THE_MOON",
        "LUNAR_DHANA_TWO_BENEFICS_IN_UPACHAYA",
    ),
    (
        "CHANDRA_ONE_BENEFIC_IN_UPACHAYA_FROM_THE_MOON",
        "LUNAR_DHANA_ONE_BENEFIC_IN_UPACHAYA",
    ),
];

/// Sets of which exactly one holds on every chart: v. 3's four yogas around
/// the Moon, and v. 8's harm and favour, one of which her half of the sky
/// and the day or night always gives.
const PARTITIONS: [&[&str]; 2] = [
    &[
        "CHANDRA_SUNAPHA",
        "CHANDRA_ANAPHA",
        "CHANDRA_DURUDHARA",
        "CHANDRA_KEMADRUMA",
    ],
    &[
        "CHANDRA_MOON_HARMFUL_BY_HEMISPHERE",
        "CHANDRA_MOON_FAVOURABLE_BY_HEMISPHERE",
    ],
];

/// Pairs the verses set apart, which never hold together.
const APART: [(&str, &str); 6] = [
    ("NABHASA_SRAK", "NABHASA_SARPA"),
    (
        "CHANDRA_TWO_BENEFICS_IN_UPACHAYA_FROM_THE_LAGNA",
        "CHANDRA_ONE_BENEFIC_IN_UPACHAYA_FROM_THE_LAGNA",
    ),
    (
        "CHANDRA_ALL_BENEFICS_IN_UPACHAYA_FROM_THE_LAGNA",
        "CHANDRA_TWO_BENEFICS_IN_UPACHAYA_FROM_THE_LAGNA",
    ),
    (
        "CHANDRA_TWO_BENEFICS_IN_UPACHAYA_FROM_THE_MOON",
        "CHANDRA_ONE_BENEFIC_IN_UPACHAYA_FROM_THE_MOON",
    ),
    (
        "CHANDRA_ALL_BENEFICS_IN_UPACHAYA_FROM_THE_MOON",
        "CHANDRA_TWO_BENEFICS_IN_UPACHAYA_FROM_THE_MOON",
    ),
    ("NABHASA_VAJRA", "NABHASA_YAVA"),
];

/// The grahas v. 7 gives a result each when it makes the yoga.
const MAKERS: [&str; 5] = ["MARS", "MERCURY", "JUPITER", "VENUS", "SATURN"];

#[test]
fn the_nabhasa_and_lunar_yogas_answer_and_their_twins_agree() {
    let rules = chapters();
    assert_eq!(rules.len(), 54);
    for rule in &rules {
        assert!(rule.is_evaluable(), "{} is evaluable", rule.key);
        assert!(rule.source.verse.is_some(), "{}: cites a verse", rule.key);
    }
    teistro_rules::check_references(&rules).expect("the chapters name themselves soundly");
    let at = |key: &str| {
        rules
            .iter()
            .position(|rule| rule.key == format!("BJ_{key}"))
            .expect("the rule ships")
    };
    let find = |key: &str| {
        shipped::nabhasas()
            .iter()
            .find(|rule| rule.key == key)
            .cloned()
            .expect("the twin ships")
    };
    let twins: Vec<(usize, Rule)> = TWINS
        .iter()
        .map(|(ours, theirs)| (at(ours), find(theirs)))
        .collect();
    let mut fired: BTreeMap<&str, usize> =
        rules.iter().map(|rule| (rule.key.as_str(), 0)).collect();
    for (path, file) in files_in("doshas") {
        let chart = chart_at(&path, &file["inputs"]);
        let divisions = vargas(&chart);
        let evaluator = Evaluator::new(&chart, Readings::TEXTS)
            .with_vargas(&divisions)
            .with_rules(&rules);
        let present: Vec<bool> = rules
            .iter()
            .map(|rule| evaluator.evaluate(rule).present)
            .collect();
        for (rule, _) in rules.iter().zip(&present).filter(|(_, present)| **present) {
            *fired.get_mut(rule.key.as_str()).unwrap() += 1;
        }
        for (ours, theirs) in &twins {
            assert_eq!(
                present[*ours],
                evaluator.evaluate(theirs).present,
                "{} in {}",
                rules[*ours].key,
                path.display()
            );
        }
        for set in PARTITIONS {
            let held = set.iter().filter(|&&key| present[at(key)]).count();
            assert_eq!(held, 1, "{set:?} in {}", path.display());
        }
        for (one, other) in APART {
            assert!(
                !(present[at(one)] && present[at(other)]),
                "{one} and {other} in {}",
                path.display()
            );
        }
        // A graha makes v. 7's yoga exactly where one of the three holds.
        let made = MAKERS
            .iter()
            .any(|graha| present[at(&format!("CHANDRA_YOGA_MADE_BY_{graha}"))]);
        assert_eq!(
            made,
            !present[at("CHANDRA_KEMADRUMA")],
            "{}",
            path.display()
        );
        // V. 6 holds Vajra and Yava impossible: Venus never stands fourth from
        // the Sun.
        assert!(!present[at("NABHASA_VAJRA")] && !present[at("NABHASA_YAVA")]);
    }
    let counts: Vec<(&str, usize)> = fired.into_iter().collect();
    assert_eq!(counts.as_slice(), ANSWERED.as_slice());
}

/// What each answers over the 93 recorded charts, recorded by a run of the
/// test; the figures that hold on none of them are measured on the computed
/// sky and its fixtures instead (`rule-cofiring-measured.md`).
const ANSWERED: [(&str, usize); 54] = [
    ("BJ_CHANDRA_ADHI", 0),
    ("BJ_CHANDRA_ALL_BENEFICS_IN_UPACHAYA_FROM_THE_LAGNA", 2),
    ("BJ_CHANDRA_ALL_BENEFICS_IN_UPACHAYA_FROM_THE_MOON", 4),
    ("BJ_CHANDRA_ANAPHA", 12),
    ("BJ_CHANDRA_DURUDHARA", 7),
    ("BJ_CHANDRA_KEMADRUMA", 57),
    ("BJ_CHANDRA_MOON_FAVOURABLE_BY_HEMISPHERE", 40),
    ("BJ_CHANDRA_MOON_HARMFUL_BY_HEMISPHERE", 53),
    ("BJ_CHANDRA_MOON_IN_APOKLIMA_FROM_THE_SUN", 40),
    ("BJ_CHANDRA_MOON_IN_KENDRA_FROM_THE_SUN", 18),
    (
        "BJ_CHANDRA_MOON_IN_OWN_OR_A_GREAT_FRIENDS_NAVAMSHA_ASPECTED_BY_JUPITER_BY_DAY_VENUS_BY_NIGHT",
        0,
    ),
    ("BJ_CHANDRA_MOON_IN_PANAPHARA_FROM_THE_SUN", 35),
    ("BJ_CHANDRA_ONE_BENEFIC_IN_UPACHAYA_FROM_THE_LAGNA", 38),
    ("BJ_CHANDRA_ONE_BENEFIC_IN_UPACHAYA_FROM_THE_MOON", 35),
    ("BJ_CHANDRA_SUNAPHA", 17),
    ("BJ_CHANDRA_TWO_BENEFICS_IN_UPACHAYA_FROM_THE_LAGNA", 5),
    ("BJ_CHANDRA_TWO_BENEFICS_IN_UPACHAYA_FROM_THE_MOON", 16),
    ("BJ_CHANDRA_YOGA_MADE_BY_JUPITER", 12),
    ("BJ_CHANDRA_YOGA_MADE_BY_MARS", 13),
    ("BJ_CHANDRA_YOGA_MADE_BY_MERCURY", 10),
    ("BJ_CHANDRA_YOGA_MADE_BY_SATURN", 13),
    ("BJ_CHANDRA_YOGA_MADE_BY_VENUS", 16),
    ("BJ_NABHASA_ARDHA_CHANDRA", 0),
    ("BJ_NABHASA_CHAKRA", 0),
    ("BJ_NABHASA_CHAPA", 0),
    ("BJ_NABHASA_CHHATRA", 0),
    ("BJ_NABHASA_DAMINI", 8),
    ("BJ_NABHASA_DANDA", 0),
    ("BJ_NABHASA_GADA", 0),
    ("BJ_NABHASA_GOLA", 0),
    ("BJ_NABHASA_HALA", 0),
    ("BJ_NABHASA_KAMALA", 0),
    ("BJ_NABHASA_KEDARA", 24),
    ("BJ_NABHASA_KUTA", 0),
    ("BJ_NABHASA_MUSALA", 0),
    ("BJ_NABHASA_NALA", 0),
    ("BJ_NABHASA_NAUKA", 0),
    ("BJ_NABHASA_PASHA", 46),
    ("BJ_NABHASA_RAJJU", 0),
    ("BJ_NABHASA_SAKATA", 0),
    ("BJ_NABHASA_SAMUDRA", 0),
    ("BJ_NABHASA_SARPA", 0),
    ("BJ_NABHASA_SHAKTI", 0),
    ("BJ_NABHASA_SHARA", 1),
    ("BJ_NABHASA_SHULA", 6),
    ("BJ_NABHASA_SRAK", 0),
    ("BJ_NABHASA_SRINGATAKA", 0),
    ("BJ_NABHASA_VAJRA", 0),
    ("BJ_NABHASA_VALLAKI", 9),
    ("BJ_NABHASA_VAPI", 0),
    ("BJ_NABHASA_VIHAGA", 0),
    ("BJ_NABHASA_YAVA", 0),
    ("BJ_NABHASA_YUGA", 0),
    ("BJ_NABHASA_YUPA", 0),
];
