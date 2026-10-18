//! Phaladeepika ch. 6's yogas, held to the 93 recorded charts: what each
//! answers, and what the chapter's own divisions say must hold between them.

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

/// Every rule the chapter ships.
fn chapter() -> Vec<Rule> {
    shipped::nabhasas()
        .iter()
        .filter(|rule| {
            rule.source.text == "Phaladeepika" && rule.source.chapter.as_deref() == Some("6")
        })
        .cloned()
        .collect()
}

#[test]
fn the_yogas_of_the_chapter_answer_where_they_did() {
    let rules = chapter();
    assert_eq!(rules.len(), 76);
    for rule in &rules {
        assert!(rule.is_evaluable(), "{} is evaluable", rule.key);
        assert!(rule.source.verse.is_some(), "{}: cites a verse", rule.key);
    }
    teistro_rules::check_references(&rules).expect("the chapter names itself soundly");
    let at = |key: &str| {
        rules
            .iter()
            .position(|rule| rule.key == format!("PHALADEEPIKA_{key}"))
            .unwrap()
    };
    // Each list is a partition the verses make: every chart answers exactly
    // one member.
    let partitions = [
        // v. 5: the planets about the Moon, or none.
        vec![
            at("SUNAPHA"),
            at("ANAPHA"),
            at("DURUDHARA"),
            at("KEMADRUMA"),
        ],
        // v. 14: the Moon in a kendra, a panaphara or an apoklima from the Sun.
        vec![at("ADHAMA"), at("SAMA"), at("VARISHTHA")],
    ];
    let great = ["RUCHAKA", "BHADRA", "HAMSA", "MALAVYA", "SASA"].map(at);
    let graded = ["ONE", "TWO", "THREE", "FOUR", "FIVE"].map(|n| at(&format!("MAHAPURUSHA_{n}")));
    let mut fired: BTreeMap<&str, usize> =
        rules.iter().map(|rule| (rule.key.as_str(), 0)).collect();
    let mut kemadruma_cancelled = 0;
    for (path, file) in files_in("doshas") {
        let chart = chart_at(&path, &file["inputs"]);
        let divisions = vargas(&chart);
        let evaluator = Evaluator::new(&chart, Readings::TEXTS)
            .with_vargas(&divisions)
            .with_rules(&rules);
        let results: Vec<_> = rules.iter().map(|rule| evaluator.evaluate(rule)).collect();
        let present: Vec<bool> = results.iter().map(|result| result.present).collect();
        for (rule, _) in rules.iter().zip(&present).filter(|(_, present)| **present) {
            *fired.get_mut(rule.key.as_str()).unwrap() += 1;
        }
        for members in &partitions {
            let held = members.iter().filter(|at| present[**at]).count();
            assert_eq!(held, 1, "{}: one of {members:?}", path.display());
        }
        // v. 4: the grade is the number of great men, and at most one holds.
        let count = great.iter().filter(|at| present[**at]).count();
        for (n, at) in graded.iter().enumerate() {
            assert_eq!(present[*at], count == n + 1, "{}", path.display());
        }
        if present[at("KEMADRUMA")] && !results[at("KEMADRUMA")].cancellations.is_empty() {
            kemadruma_cancelled += 1;
        }
    }
    // What v. 5 gives to "some" removes nearly every Kemadruma: some one of
    // five planets is almost always angular from the lagna or the Moon.
    assert_eq!(kemadruma_cancelled, 57);
    let counts: Vec<(&str, usize)> = fired.into_iter().collect();
    assert_eq!(counts.as_slice(), ANSWERED.as_slice());
}

/// What each answers over the 93 recorded charts. The recorded charts carry
/// no day or night, so a rule asking a birth by day never holds on them and
/// one asking a birth by night holds wherever its signs do: Mahabhagya's two
/// forms are measured on the computed sky instead (`rule-cofiring-measured.md`).
const ANSWERED: [(&str, usize); 76] = [
    ("PHALADEEPIKA_ADHAMA", 18),
    ("PHALADEEPIKA_ADHI", 0),
    ("PHALADEEPIKA_AMALA", 14),
    ("PHALADEEPIKA_ANAPHA", 12),
    ("PHALADEEPIKA_ASTRA", 44),
    ("PHALADEEPIKA_ASUBHAMALA", 6),
    ("PHALADEEPIKA_ASURA", 21),
    ("PHALADEEPIKA_AVA", 20),
    ("PHALADEEPIKA_BHADRA", 10),
    ("PHALADEEPIKA_BHAGYA", 42),
    ("PHALADEEPIKA_CHAMARA", 36),
    ("PHALADEEPIKA_CHHATRA", 32),
    ("PHALADEEPIKA_DAINYA_PARIVARTANA", 18),
    ("PHALADEEPIKA_DARIDRA", 32),
    ("PHALADEEPIKA_DHENU", 26),
    ("PHALADEEPIKA_DURUDHARA", 7),
    ("PHALADEEPIKA_DURYOGA", 16),
    ("PHALADEEPIKA_DURYOGA_OF_THE_LORDS", 0),
    ("PHALADEEPIKA_DUSHKRITI", 29),
    ("PHALADEEPIKA_GAURI", 1),
    ("PHALADEEPIKA_HAMSA", 14),
    ("PHALADEEPIKA_HARSHA", 24),
    ("PHALADEEPIKA_JALADHI", 51),
    ("PHALADEEPIKA_KAHALA", 29),
    ("PHALADEEPIKA_KAMA", 27),
    ("PHALADEEPIKA_KEMADRUMA", 57),
    ("PHALADEEPIKA_KEMADRUMA_BHANGA", 89),
    ("PHALADEEPIKA_KESARI", 25),
    ("PHALADEEPIKA_KHALA_PARIVARTANA", 2),
    ("PHALADEEPIKA_KHYATI", 40),
    ("PHALADEEPIKA_KING_BY_THE_LORDS", 0),
    ("PHALADEEPIKA_KUHU", 14),
    ("PHALADEEPIKA_LAKSHMI", 10),
    ("PHALADEEPIKA_MAHABHAGYA_FEMALE", 14),
    ("PHALADEEPIKA_MAHABHAGYA_MALE", 0),
    ("PHALADEEPIKA_MAHAPURUSHA_FIVE", 0),
    ("PHALADEEPIKA_MAHAPURUSHA_FOUR", 0),
    ("PHALADEEPIKA_MAHAPURUSHA_ONE", 26),
    ("PHALADEEPIKA_MAHAPURUSHA_THREE", 0),
    ("PHALADEEPIKA_MAHAPURUSHA_TWO", 19),
    ("PHALADEEPIKA_MAHA_PARIVARTANA", 1),
    ("PHALADEEPIKA_MALAVYA", 17),
    ("PHALADEEPIKA_MRITI", 25),
    ("PHALADEEPIKA_MUSALA", 48),
    ("PHALADEEPIKA_NIRBHAGYA", 27),
    ("PHALADEEPIKA_NISSVA", 38),
    ("PHALADEEPIKA_PAMARA", 22),
    ("PHALADEEPIKA_PAPA_KARTARI", 17),
    ("PHALADEEPIKA_PAPA_UBHAYACHARI", 0),
    ("PHALADEEPIKA_PAPA_VASI", 23),
    ("PHALADEEPIKA_PAPA_VESI", 9),
    ("PHALADEEPIKA_PARIJATA", 34),
    ("PHALADEEPIKA_PARVATA", 23),
    ("PHALADEEPIKA_PUSHKALA", 0),
    ("PHALADEEPIKA_RAJA", 9),
    ("PHALADEEPIKA_RUCHAKA", 17),
    ("PHALADEEPIKA_SAKATA", 24),
    ("PHALADEEPIKA_SAMA", 35),
    ("PHALADEEPIKA_SANKHA", 38),
    ("PHALADEEPIKA_SARALA", 48),
    ("PHALADEEPIKA_SARASWATI", 2),
    ("PHALADEEPIKA_SASA", 6),
    ("PHALADEEPIKA_SAURYA", 33),
    ("PHALADEEPIKA_SHUBHA_KARTARI", 2),
    ("PHALADEEPIKA_SRIKANTHA", 3),
    ("PHALADEEPIKA_SRINATHA", 4),
    ("PHALADEEPIKA_SUBHAMALA", 1),
    ("PHALADEEPIKA_SUBHA_UBHAYACHARI", 19),
    ("PHALADEEPIKA_SUBHA_VASI", 20),
    ("PHALADEEPIKA_SUBHA_VESI", 26),
    ("PHALADEEPIKA_SUNAPHA", 17),
    ("PHALADEEPIKA_SUSUBHA", 4),
    ("PHALADEEPIKA_VARISHTHA", 40),
    ("PHALADEEPIKA_VASUMAT", 7),
    ("PHALADEEPIKA_VIMALA", 16),
    ("PHALADEEPIKA_VIRINCHI", 0),
];
