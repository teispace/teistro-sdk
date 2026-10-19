//! Saravali ch. 45, a woman's chart, and ch. 46 vv. 22 to 45, the 22nd
//! drekkana's causes of death, held to the 93 recorded charts: what each
//! answers, and that the figures Saravali states as Brihat Jataka ch. 24 does
//! answer what that text's do.

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

/// The first verse a rule cites.
fn first_verse(rule: &Rule) -> u32 {
    let verse = rule.source.verse.as_deref().expect("a verse is cited");
    let digits: String = verse.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().expect("the verse starts with its number")
}

/// Saravali ch. 45, and ch. 46 from v. 22: ch. 46 vv. 1 to 21 are
/// `saravali_more.rs`'s.
fn late() -> Vec<Rule> {
    shipped::nabhasas()
        .iter()
        .filter(|rule| {
            rule.source.text == "Saravali"
                && match rule.source.chapter.as_deref() {
                    Some("45") => true,
                    Some("46") => first_verse(rule) >= 22,
                    _ => false,
                }
        })
        .cloned()
        .collect()
}

/// The figures Saravali ch. 45 states as Brihat Jataka ch. 24 does, each with
/// that text's rule (ours without its `SARAVALI_` prefix first). The
/// conditions are one; the outcomes are each text's own words.
const TWINS: [(&str, &str); 67] = [
    (
        "STRI_LAGNA_AND_MOON_IN_EVEN_SIGNS",
        "BJ_STRI_LAGNA_AND_MOON_IN_EVEN_SIGNS",
    ),
    (
        "STRI_LAGNA_AND_MOON_IN_EVEN_SIGNS_ASPECTED_BY_BENEFICS",
        "BJ_STRI_LAGNA_AND_MOON_IN_EVEN_SIGNS_ASPECTED_BY_BENEFICS",
    ),
    (
        "STRI_LAGNA_AND_MOON_IN_ODD_SIGNS",
        "BJ_STRI_LAGNA_AND_MOON_IN_ODD_SIGNS",
    ),
    (
        "STRI_LAGNA_AND_MOON_IN_ODD_SIGNS_WITH_MALEFICS",
        "BJ_STRI_LAGNA_AND_MOON_IN_ODD_SIGNS_WITH_MALEFICS",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_MARS",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_MARS",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_VENUS",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_VENUS",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_MERCURY",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_MERCURY",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_JUPITER",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_JUPITER",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_SATURN",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_SATURN",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_MARS",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_MARS",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_VENUS",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_VENUS",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_MERCURY",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_MERCURY",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_JUPITER",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_JUPITER",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_SATURN",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_SATURN",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_MARS",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_MARS",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_VENUS",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_VENUS",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_MERCURY",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_MERCURY",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_JUPITER",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_JUPITER",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_SATURN",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_SATURN",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_MARS",
        "BJ_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_MARS",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_VENUS",
        "BJ_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_VENUS",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_MERCURY",
        "BJ_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_MERCURY",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_JUPITER",
        "BJ_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_JUPITER",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_SATURN",
        "BJ_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_SATURN",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_MARS",
        "BJ_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_MARS",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_VENUS",
        "BJ_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_VENUS",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_MERCURY",
        "BJ_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_MERCURY",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_JUPITER",
        "BJ_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_JUPITER",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_SATURN",
        "BJ_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_SATURN",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_MARS",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_MARS",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_VENUS",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_VENUS",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_MERCURY",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_MERCURY",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_JUPITER",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_JUPITER",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_SATURN",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_SATURN",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_MARS",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_MARS",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_VENUS",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_VENUS",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_MERCURY",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_MERCURY",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_JUPITER",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_JUPITER",
    ),
    (
        "STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_SATURN",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_SATURN",
    ),
    (
        "STRI_SATURN_AND_VENUS_IN_EACH_OTHERS_NAVAMSHA_ASPECTING_EACH_OTHER",
        "BJ_STRI_SATURN_AND_VENUS_IN_EACH_OTHERS_NAVAMSHA_ASPECTING_EACH_OTHER",
    ),
    (
        "STRI_TAURUS_OR_LIBRA_RISING_IN_AN_AQUARIUS_NAVAMSHA",
        "BJ_STRI_TAURUS_OR_LIBRA_RISING_IN_AN_AQUARIUS_NAVAMSHA",
    ),
    (
        "STRI_MOVABLE_SEVENTH_HUSBAND_ABROAD",
        "BJ_STRI_MOVABLE_SEVENTH_HUSBAND_ABROAD",
    ),
    (
        "STRI_MERCURY_AND_SATURN_SEVENTH_HUSBAND_IMPOTENT",
        "BJ_STRI_MERCURY_AND_SATURN_SEVENTH_HUSBAND_IMPOTENT",
    ),
    (
        "STRI_SUN_SEVENTH_ABANDONED",
        "BJ_STRI_SUN_SEVENTH_ABANDONED",
    ),
    (
        "STRI_SATURN_SEVENTH_ASPECTED_BY_A_MALEFIC_UNMARRIED",
        "BJ_STRI_SATURN_SEVENTH_ASPECTED_BY_A_MALEFIC_UNMARRIED",
    ),
    (
        "STRI_WEAK_MALEFIC_SEVENTH_ASPECTED_BY_A_BENEFIC_REJECTED",
        "BJ_STRI_WEAK_MALEFIC_SEVENTH_ASPECTED_BY_A_BENEFIC_REJECTED",
    ),
    (
        "STRI_MALEFICS_SEVENTH_WIDOWED",
        "BJ_STRI_MALEFICS_SEVENTH_WIDOWED",
    ),
    (
        "STRI_BENEFICS_AND_MALEFICS_SEVENTH_REMARRIES",
        "BJ_STRI_BENEFICS_AND_MALEFICS_SEVENTH_REMARRIES",
    ),
    (
        "STRI_VENUS_AND_MARS_IN_EACH_OTHERS_NAVAMSHA_ADULTERY",
        "BJ_STRI_VENUS_AND_MARS_IN_EACH_OTHERS_NAVAMSHA_ADULTERY",
    ),
    (
        "STRI_MOON_VENUS_AND_MARS_SEVENTH_ADULTERY_BY_LEAVE",
        "BJ_STRI_MOON_VENUS_AND_MARS_SEVENTH_ADULTERY_BY_LEAVE",
    ),
    (
        "STRI_MOON_AND_VENUS_RISING_IN_A_SIGN_OF_SATURN_OR_MARS_ASPECTED_BY_A_MALEFIC",
        "BJ_STRI_MOON_AND_VENUS_RISING_IN_A_SIGN_OF_SATURN_OR_MARS_ASPECTED_BY_A_MALEFIC",
    ),
    (
        "STRI_SETTING_NAVAMSHA_OF_A_BENEFIC",
        "BJ_STRI_SETTING_NAVAMSHA_OF_A_BENEFIC",
    ),
    (
        "STRI_SETTING_SIGN_OR_NAVAMSHA_OF_SATURN",
        "BJ_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_SATURN",
    ),
    (
        "STRI_SETTING_SIGN_OR_NAVAMSHA_OF_MARS",
        "BJ_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_MARS",
    ),
    (
        "STRI_SETTING_SIGN_OR_NAVAMSHA_OF_VENUS",
        "BJ_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_VENUS",
    ),
    (
        "STRI_SETTING_SIGN_OR_NAVAMSHA_OF_MERCURY",
        "BJ_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_MERCURY",
    ),
    (
        "STRI_SETTING_SIGN_OR_NAVAMSHA_OF_THE_MOON",
        "BJ_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_THE_MOON",
    ),
    (
        "STRI_SETTING_SIGN_OR_NAVAMSHA_OF_JUPITER",
        "BJ_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_JUPITER",
    ),
    (
        "STRI_SETTING_SIGN_OR_NAVAMSHA_OF_THE_SUN",
        "BJ_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_THE_SUN",
    ),
    (
        "STRI_MOON_AND_VENUS_RISING",
        "BJ_STRI_MOON_AND_VENUS_RISING",
    ),
    (
        "STRI_MOON_AND_MERCURY_RISING",
        "BJ_STRI_MOON_AND_MERCURY_RISING",
    ),
    (
        "STRI_VENUS_AND_MERCURY_RISING",
        "BJ_STRI_VENUS_AND_MERCURY_RISING",
    ),
    (
        "STRI_THREE_BENEFICS_RISING",
        "BJ_STRI_THREE_BENEFICS_RISING",
    ),
    (
        "STRI_MALEFIC_EIGHTH_WIDOWED",
        "BJ_STRI_MALEFIC_EIGHTH_WIDOWED",
    ),
    (
        "STRI_MALEFIC_EIGHTH_BENEFIC_SECOND_DIES_FIRST",
        "BJ_STRI_MALEFIC_EIGHTH_BENEFIC_SECOND_DIES_FIRST",
    ),
    (
        "STRI_MOON_IN_VIRGO_SCORPIO_TAURUS_OR_LEO_FEW_SONS",
        "BJ_STRI_MOON_IN_VIRGO_SCORPIO_TAURUS_OR_LEO_FEW_SONS",
    ),
    (
        "STRI_SATURN_MIDDLING_THE_SOFT_WEAK_THE_HARD_STRONG_ODD_LAGNA",
        "BJ_STRI_SATURN_MIDDLING_THE_SOFT_WEAK_THE_HARD_STRONG_ODD_LAGNA",
    ),
];

#[test]
fn the_figures_answer_and_their_twins_agree() {
    let rules = late();
    assert_eq!(rules.len(), 116);
    for rule in &rules {
        assert!(rule.is_evaluable(), "{} is evaluable", rule.key);
        assert!(rule.key.starts_with("SARAVALI_"), "{}", rule.key);
    }
    teistro_rules::check_references(&rules).expect("the chapters name no rule unsoundly");
    let find = |key: &str| {
        shipped::nabhasas()
            .iter()
            .find(|rule| rule.key == key)
            .cloned()
            .expect("the twin ships")
    };
    let twins: Vec<(Rule, Rule)> = TWINS
        .iter()
        .map(|(ours, theirs)| (find(&format!("SARAVALI_{ours}")), find(theirs)))
        .collect();
    for (ours, theirs) in &twins {
        assert_eq!(ours.conditions, theirs.conditions, "{}", ours.key);
    }
    let mut fired: BTreeMap<&str, usize> =
        rules.iter().map(|rule| (rule.key.as_str(), 0)).collect();
    for (path, file) in files_in("doshas") {
        let chart = chart_at(&path, &file["inputs"]);
        let divisions = vargas(&chart);
        let evaluator = Evaluator::new(&chart, Readings::TEXTS).with_vargas(&divisions);
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

/// Every chart has a 22nd drekkana, so exactly one of ch. 46's thirty-six
/// readings of it holds unless Aries's first is the one and no malefic is in it
/// or aspects it, v. 23 asking that of Aries's first alone.
#[test]
fn one_drekkana_reading_holds_in_each_chart() {
    let rules: Vec<Rule> = late()
        .into_iter()
        .filter(|rule| {
            rule.key
                .starts_with("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_")
                && !rule.key.contains("_RULED_BY_")
        })
        .collect();
    assert_eq!(rules.len(), 36);
    let mut aries_first = rules
        .iter()
        .find(|rule| rule.key == "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_ARIES_FIRST")
        .cloned()
        .expect("Aries's first ships");
    // Its placement alone, without v. 23's malefic.
    aries_first.conditions.truncate(1);
    for (path, file) in files_in("doshas") {
        let chart = chart_at(&path, &file["inputs"]);
        let divisions = vargas(&chart);
        let evaluator = Evaluator::new(&chart, Readings::TEXTS).with_vargas(&divisions);
        let held = rules
            .iter()
            .filter(|rule| evaluator.evaluate(rule).present)
            .count();
        let unclaimed = usize::from(evaluator.evaluate(&aries_first).present);
        assert!(held <= 1, "{path:?}: {held} readings");
        assert!(held + unclaimed >= 1, "{path:?}: no reading");
    }
}

/// What each answers over the 93 recorded charts. A placeholder: the
/// integrator fills it from the evaluator's own run.
const ANSWERED: [(&str, usize); 116] = [
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_AQUARIUS_FIRST", 2),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_AQUARIUS_SECOND", 0),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_AQUARIUS_THIRD", 1),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_ARIES_FIRST", 1),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_ARIES_SECOND", 6),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_ARIES_THIRD", 0),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_CANCER_FIRST", 2),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_CANCER_SECOND", 5),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_CANCER_THIRD", 0),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_CAPRICORN_FIRST", 3),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_CAPRICORN_SECOND", 1),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_CAPRICORN_THIRD", 1),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_GEMINI_FIRST", 8),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_GEMINI_SECOND", 2),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_GEMINI_THIRD", 2),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_LEO_FIRST", 1),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_LEO_SECOND", 1),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_LEO_THIRD", 0),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_LIBRA_FIRST", 2),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_LIBRA_SECOND", 9),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_LIBRA_THIRD", 11),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_PISCES_FIRST", 1),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_PISCES_SECOND", 5),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_PISCES_THIRD", 1),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_RULED_BY_JUPITER", 5),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_RULED_BY_MARS", 10),
    (
        "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_RULED_BY_MERCURY",
        29,
    ),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_RULED_BY_MOON", 9),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_RULED_BY_SATURN", 23),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_RULED_BY_SUN", 9),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_RULED_BY_VENUS", 8),
    (
        "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_SAGITTARIUS_FIRST",
        3,
    ),
    (
        "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_SAGITTARIUS_SECOND",
        1,
    ),
    (
        "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_SAGITTARIUS_THIRD",
        2,
    ),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_SCORPIO_FIRST", 1),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_SCORPIO_SECOND", 0),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_SCORPIO_THIRD", 2),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_TAURUS_FIRST", 2),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_TAURUS_SECOND", 1),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_TAURUS_THIRD", 5),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_VIRGO_FIRST", 8),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_VIRGO_SECOND", 2),
    ("SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_VIRGO_THIRD", 0),
    ("SARAVALI_STRI_BENEFICS_AND_MALEFICS_SEVENTH_REMARRIES", 7),
    (
        "SARAVALI_STRI_EMPTY_SEVENTH_UNASPECTED_BY_A_BENEFIC_HUSBAND_UNMANLY",
        42,
    ),
    ("SARAVALI_STRI_LAGNA_AND_MOON_IN_EVEN_SIGNS", 38),
    (
        "SARAVALI_STRI_LAGNA_AND_MOON_IN_EVEN_SIGNS_ASPECTED_BY_BENEFICS",
        11,
    ),
    ("SARAVALI_STRI_LAGNA_AND_MOON_IN_ODD_SIGNS", 21),
    (
        "SARAVALI_STRI_LAGNA_AND_MOON_IN_ODD_SIGNS_WITH_MALEFICS",
        10,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_JUPITER",
        20,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_MARS",
        13,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_MERCURY",
        5,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_SATURN",
        7,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_VENUS",
        2,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_JUPITER",
        4,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_MARS",
        1,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_MERCURY",
        18,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_SATURN",
        4,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_VENUS",
        5,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_JUPITER",
        12,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_MARS",
        3,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_MERCURY",
        8,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_SATURN",
        3,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_VENUS",
        4,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_JUPITER",
        3,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_MARS",
        0,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_MERCURY",
        9,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_SATURN",
        9,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_VENUS",
        3,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_JUPITER",
        5,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_MARS",
        3,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_MERCURY",
        4,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_SATURN",
        3,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_VENUS",
        6,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_JUPITER",
        1,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_MARS",
        0,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_MERCURY",
        3,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_SATURN",
        2,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_VENUS",
        2,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_JUPITER",
        6,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_MARS",
        1,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_MERCURY",
        7,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_SATURN",
        2,
    ),
    (
        "SARAVALI_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_VENUS",
        1,
    ),
    ("SARAVALI_STRI_MALEFICS_SEVENTH_WIDOWED", 11),
    ("SARAVALI_STRI_MALEFIC_EIGHTH_BENEFIC_SECOND_DIES_FIRST", 1),
    ("SARAVALI_STRI_MALEFIC_EIGHTH_WIDOWED", 37),
    ("SARAVALI_STRI_MALEFIC_SEVENTH_A_GRAHA_NINTH_ASCETIC", 3),
    ("SARAVALI_STRI_MARS_SEVENTH_WIDOWED_A_BRIDE", 5),
    (
        "SARAVALI_STRI_MERCURY_AND_SATURN_SEVENTH_HUSBAND_IMPOTENT",
        1,
    ),
    ("SARAVALI_STRI_MOON_AND_MERCURY_RISING", 2),
    ("SARAVALI_STRI_MOON_AND_VENUS_RISING", 0),
    (
        "SARAVALI_STRI_MOON_AND_VENUS_RISING_IN_A_SIGN_OF_SATURN_OR_MARS_ASPECTED_BY_A_MALEFIC",
        0,
    ),
    (
        "SARAVALI_STRI_MOON_IN_VIRGO_SCORPIO_TAURUS_OR_LEO_FEW_SONS",
        41,
    ),
    (
        "SARAVALI_STRI_MOON_VENUS_AND_MARS_SEVENTH_ADULTERY_BY_LEAVE",
        0,
    ),
    ("SARAVALI_STRI_MOVABLE_SEVENTH_HUSBAND_ABROAD", 31),
    (
        "SARAVALI_STRI_SATURN_AND_VENUS_IN_EACH_OTHERS_NAVAMSHA_ASPECTING_EACH_OTHER",
        0,
    ),
    (
        "SARAVALI_STRI_SATURN_MIDDLING_THE_SOFT_WEAK_THE_HARD_STRONG_ODD_LAGNA",
        0,
    ),
    (
        "SARAVALI_STRI_SATURN_SEVENTH_ASPECTED_BY_A_MALEFIC_UNMARRIED",
        3,
    ),
    ("SARAVALI_STRI_SETTING_NAVAMSHA_OF_A_BENEFIC", 45),
    (
        "SARAVALI_STRI_SETTING_NAVAMSHA_OF_MARS_ASPECTED_BY_THE_MOON",
        1,
    ),
    ("SARAVALI_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_JUPITER", 34),
    ("SARAVALI_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_MARS", 26),
    ("SARAVALI_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_MERCURY", 44),
    ("SARAVALI_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_SATURN", 16),
    ("SARAVALI_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_THE_MOON", 9),
    ("SARAVALI_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_THE_SUN", 23),
    ("SARAVALI_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_VENUS", 27),
    (
        "SARAVALI_STRI_STRONG_MERCURY_JUPITER_VENUS_WITH_THE_MOON_IN_AN_EVEN_LAGNA",
        0,
    ),
    ("SARAVALI_STRI_SUN_SEVENTH_ABANDONED", 17),
    (
        "SARAVALI_STRI_TAURUS_OR_LIBRA_RISING_IN_AN_AQUARIUS_NAVAMSHA",
        1,
    ),
    ("SARAVALI_STRI_THREE_BENEFICS_RISING", 0),
    ("SARAVALI_STRI_THREE_MALEFICS_RISING", 0),
    (
        "SARAVALI_STRI_VENUS_AND_MARS_IN_EACH_OTHERS_NAVAMSHA_ADULTERY",
        4,
    ),
    ("SARAVALI_STRI_VENUS_AND_MERCURY_RISING", 0),
    (
        "SARAVALI_STRI_WEAK_MALEFIC_SEVENTH_ASPECTED_BY_A_BENEFIC_REJECTED",
        0,
    ),
];
