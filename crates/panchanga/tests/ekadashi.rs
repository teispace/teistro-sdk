//! Ekadashi held to *Dharmasindhu*'s twelve examples, pp. 11–12
//! (`03-design/festival-rules.md` §8), read off the page images.
//!
//! Each example is almanac lines: the ghati past a sunrise at which the
//! 10th, the 11th and the 12th end. `60 । 1` fills a day and ends a
//! ghati past the next sunrise; *kshaya* ends before any sunrise of its
//! own. The days are the shared synthetic ones, and the 11th's own day
//! (D1) is day 2, the 3rd of January; the expected day is the text's.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index fixed lists"
)]

mod common;

use common::{days_over, ghati};
use teistro_calendar::lunisolar::MonthKind;
use teistro_core::catalogue::{Masa, Tithi};
use teistro_panchanga::festival::{EkadashiFast, EkadashiRule, Excess, Vedha, Which, ekadashis};

use Which::{Earlier, Later};

/// Where a tithi ends: `(day, ghati)` past that day's sunrise, D1 day 2.
type End = (usize, f64);

/// One example: where the 10th, the 11th and the 12th end.
struct Example {
    name: &'static str,
    ends: [End; 3],
    pierced_at: Option<Vedha>,
    excess: Excess,
    /// The text's day for the Vaishnava, the Smarta householder and the
    /// renunciant.
    days: [Which; 3],
}

/// The 9th ends on day 0; the 10th, the 11th and the 12th where the
/// example puts them.
fn fasts(ends: [End; 3]) -> Vec<EkadashiFast> {
    let at = |(day, ghatis): End| ghati(day, ghatis);
    let tithis = [
        (Tithi::KrishnaNavami, ghati(0, 50.0)),
        (Tithi::KrishnaDashami, at(ends[0])),
        (Tithi::KrishnaEkadashi, at(ends[1])),
        (Tithi::KrishnaDwadashi, at(ends[2])),
        (Tithi::KrishnaTrayodashi, ghati(5, 40.0)),
        (Tithi::KrishnaChaturdashi, ghati(6, 30.0)),
    ];
    let days = days_over(6, &tithis, &[], Masa::Kartika, MonthKind::Nija);
    let answer = ekadashis(&EkadashiRule::dharmasindhu(), &days).unwrap();
    assert!(answer.unjudged.is_empty(), "{:?}", answer.unjudged);
    answer.ekadashis
}

const EXAMPLES: [Example; 12] = [
    // p. 11, the Vaishnavas' four, each pure.
    Example {
        name: "V1",
        ends: [(1, 55.0), (3, 1.0), (3, 58.0)],
        pierced_at: None,
        excess: Excess::Eleventh,
        days: [Later, Earlier, Later],
    },
    Example {
        name: "V2",
        ends: [(1, 55.0), (2, 58.0), (4, 1.0)],
        pierced_at: None,
        excess: Excess::Twelfth,
        days: [Later, Earlier, Earlier],
    },
    Example {
        name: "V3",
        ends: [(1, 55.0), (3, 1.0), (4, 5.0)],
        pierced_at: None,
        excess: Excess::Both,
        days: [Later, Later, Later],
    },
    Example {
        name: "V4",
        ends: [(1, 55.0), (2, 57.0), (3, 58.0)],
        pierced_at: None,
        excess: Excess::Neither,
        days: [Earlier, Earlier, Earlier],
    },
    // pp. 11–12, the Smartas' eight.
    Example {
        name: "S1",
        ends: [(1, 58.0), (3, 1.0), (3, 58.0)],
        pierced_at: Some(Vedha::Arunodaya),
        excess: Excess::Eleventh,
        days: [Later, Earlier, Later],
    },
    Example {
        name: "S2",
        ends: [(2, 4.0), (3, 2.0), (3, 58.0)],
        pierced_at: Some(Vedha::Sunrise),
        excess: Excess::Eleventh,
        days: [Later, Earlier, Later],
    },
    Example {
        name: "S3",
        ends: [(1, 58.0), (3, 1.0), (4, 4.0)],
        pierced_at: Some(Vedha::Arunodaya),
        excess: Excess::Both,
        days: [Later, Later, Later],
    },
    Example {
        name: "S4",
        ends: [(2, 2.0), (3, 3.0), (4, 4.0)],
        pierced_at: Some(Vedha::Sunrise),
        excess: Excess::Both,
        days: [Later, Later, Later],
    },
    Example {
        name: "S5",
        ends: [(1, 58.0), (2, 59.0), (4, 1.0)],
        pierced_at: Some(Vedha::Arunodaya),
        excess: Excess::Twelfth,
        // Madhava; Hemadri gives all the later, and the closing says the
        // learned give neither two fasts nor one later for all (C182).
        days: [Later, Earlier, Earlier],
    },
    Example {
        name: "S6",
        ends: [(2, 1.0), (2, 58.0), (4, 1.0)],
        pierced_at: Some(Vedha::Sunrise),
        excess: Excess::Twelfth,
        days: [Later, Later, Later],
    },
    Example {
        name: "S7",
        ends: [(1, 57.0), (2, 58.0), (3, 59.0)],
        pierced_at: Some(Vedha::Arunodaya),
        excess: Excess::Neither,
        // Renunciants are not named here, so they keep the Smarta day.
        days: [Later, Earlier, Earlier],
    },
    Example {
        name: "S8",
        ends: [(2, 2.0), (2, 56.0), (3, 59.0)],
        pierced_at: Some(Vedha::Sunrise),
        excess: Excess::Neither,
        days: [Later, Earlier, Later],
    },
];

const KEYS: [&str; 3] = [
    "EKADASHI_VAISHNAVA",
    "EKADASHI_SMARTA",
    "EKADASHI_SMARTA_RENUNCIANT",
];

#[test]
fn each_of_the_twelve_examples_takes_the_text_s_day_under_each_rule() {
    for example in &EXAMPLES {
        let found = fasts(example.ends);
        let keys: Vec<&str> = found.iter().map(|fast| fast.rule.as_str()).collect();
        assert_eq!(keys, KEYS, "{}: one fast a rule", example.name);
        for (fast, want) in found.iter().zip(example.days) {
            let name = example.name;
            assert_eq!(
                fast.days.each_ref().map(|day| day.day),
                [3, 4],
                "{name}: D1 is day 2"
            );
            assert_eq!(fast.pierced_at, example.pierced_at, "{name}");
            assert_eq!(fast.excess, example.excess, "{name}");
            assert_eq!(fast.choice, want, "{name} {}", fast.rule);
            let day = match want {
                Earlier => 3,
                Later => 4,
            };
            assert_eq!(fast.day.day, day, "{name} {}", fast.rule);
            assert_eq!(
                (fast.tithi, fast.month),
                (Tithi::KrishnaEkadashi, Masa::Kartika)
            );
        }
    }
}

#[test]
fn hemadri_s_view_is_one_cell_away() {
    // C182: V2 and S5, pure with the 12th alone in excess, take the later
    // day for all in Hemadri's view.
    let mut hemadri = EkadashiRule::dharmasindhu().remove(1);
    hemadri.key = "EKADASHI_SMARTA_HEMADRI".to_owned();
    hemadri.table.pure.twelfth = Later;
    let tithis = |ends: [End; 3]| {
        [
            (Tithi::KrishnaNavami, ghati(0, 50.0)),
            (Tithi::KrishnaDashami, ghati(ends[0].0, ends[0].1)),
            (Tithi::KrishnaEkadashi, ghati(ends[1].0, ends[1].1)),
            (Tithi::KrishnaDwadashi, ghati(ends[2].0, ends[2].1)),
            (Tithi::KrishnaTrayodashi, ghati(5, 40.0)),
        ]
    };
    for name in ["V2", "S5"] {
        let example = EXAMPLES.iter().find(|e| e.name == name).unwrap();
        let days = days_over(
            6,
            &tithis(example.ends),
            &[],
            Masa::Kartika,
            MonthKind::Nija,
        );
        let answer = ekadashis(std::slice::from_ref(&hemadri), &days).unwrap();
        assert_eq!(answer.ekadashis[0].choice, Later, "{name}");
    }
}

#[test]
fn an_eleventh_without_its_neighbours_is_unjudged_by_each_rule() {
    // The 11th is the first tithi the days hold: no 10th to pierce it.
    let tithis = [
        (Tithi::KrishnaEkadashi, ghati(1, 30.0)),
        (Tithi::KrishnaDwadashi, ghati(2, 30.0)),
        (Tithi::KrishnaTrayodashi, ghati(3, 30.0)),
    ];
    let days = days_over(6, &tithis, &[], Masa::Kartika, MonthKind::Nija);
    let answer = ekadashis(&EkadashiRule::dharmasindhu(), &days).unwrap();
    assert!(answer.ekadashis.is_empty(), "{:?}", answer.ekadashis);
    let keys: Vec<&str> = answer.unjudged.iter().map(|u| u.rule.as_str()).collect();
    assert_eq!(keys, KEYS);
}

#[test]
fn a_rule_without_a_key_is_refused_by_it() {
    let mut keyless = EkadashiRule::dharmasindhu().remove(0);
    keyless.key.clear();
    let error = ekadashis(&[keyless], &[]).unwrap_err();
    assert_eq!(error.field(), Some("key"));
}
