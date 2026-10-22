//! The eight mahadvadashis of *Hari Bhakti Vilasa* (`03-design/festival-rules.md`
//! §8.9) on the shared synthetic days: one row per kind, each fork's other
//! reading, and the parana of a nakshatra kind, one row per clause of
//! 13.290–291.
//!
//! The 9th ends on day 0 and the 10th to the fortnight's end where a row
//! puts them, `(day, ghati)` past that day's sunrise; day `k` is the
//! `k + 1`th of January 2024. Sunset is ghati 30 and arunodaya ghati 56 of
//! the day before.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

mod common;

use common::{days_over, ghati};
use teistro_calendar::lunisolar::MonthKind;
use teistro_core::catalogue::{Masa, Nakshatra, Tithi};
use teistro_panchanga::festival::{
    EkadashiFast, EkadashiRule, MahadvadashiKind, MahadvadashiRule, NakshatraJoining, ParanaEdge,
    TrisprishaAt, VyanjuliPast, Which, ekadashis,
};

use MahadvadashiKind::{
    Jaya, Jayanti, Pakshavardhini, Papanashini, Trisprisha, Unmilani, Vijaya, Vyanjuli,
};

/// Where a tithi or a nakshatra ends.
type End = (usize, f64);

/// The bright fortnight from the 9th to the full moon.
const SHUKLA: [Tithi; 7] = [
    Tithi::ShuklaNavami,
    Tithi::ShuklaDashami,
    Tithi::ShuklaEkadashi,
    Tithi::ShuklaDwadashi,
    Tithi::ShuklaTrayodashi,
    Tithi::ShuklaChaturdashi,
    Tithi::Purnima,
];

/// The dark fortnight from the 9th to the new moon.
const KRISHNA: [Tithi; 7] = [
    Tithi::KrishnaNavami,
    Tithi::KrishnaDashami,
    Tithi::KrishnaEkadashi,
    Tithi::KrishnaDwadashi,
    Tithi::KrishnaTrayodashi,
    Tithi::KrishnaChaturdashi,
    Tithi::Amavasya,
];

/// A nakshatra's stay, `from` to `to`, among others.
fn stay(nakshatra: Nakshatra, from: End, to: End) -> Vec<(Nakshatra, End)> {
    vec![
        (Nakshatra::Ashwini, from),
        (nakshatra, to),
        (Nakshatra::Bharani, (11, 0.0)),
    ]
}

/// The fast under `rule` over a fortnight whose 10th to its end end at
/// `ends`, with the Moon's `nakshatras`.
fn fast_under(
    rule: &EkadashiRule,
    paksha: [Tithi; 7],
    ends: [End; 6],
    nakshatras: &[(Nakshatra, End)],
) -> EkadashiFast {
    let at = |(day, ghatis): End| ghati(day, ghatis);
    let mut tithis = vec![(paksha[0], ghati(0, 50.0))];
    tithis.extend(
        paksha[1..]
            .iter()
            .zip(ends)
            .map(|(&tithi, end)| (tithi, at(end))),
    );
    let after = if paksha[6] == Tithi::Purnima {
        Tithi::KrishnaPratipada
    } else {
        Tithi::ShuklaPratipada
    };
    tithis.push((after, ghati(11, 0.0)));
    let nakshatras: Vec<_> = nakshatras.iter().map(|&(n, end)| (n, at(end))).collect();
    let days = days_over(11, &tithis, &nakshatras, Masa::Kartika, MonthKind::Nija);
    let answer = ekadashis(std::slice::from_ref(rule), &days).unwrap();
    assert!(answer.unjudged.is_empty(), "{:?}", answer.unjudged);
    answer.ekadashis.into_iter().next().unwrap()
}

fn fast(paksha: [Tithi; 7], ends: [End; 6], nakshatras: &[(Nakshatra, End)]) -> EkadashiFast {
    fast_under(
        &EkadashiRule::hari_bhakti_vilasa(),
        paksha,
        ends,
        nakshatras,
    )
}

/// A Gaudiya rule with its mahadvadashi rule changed.
fn read(change: impl FnOnce(&mut MahadvadashiRule)) -> EkadashiRule {
    let mut rule = EkadashiRule::hari_bhakti_vilasa();
    change(rule.mahadvadashi.as_mut().unwrap());
    rule
}

/// The kinds that held, and the fast's day of January.
fn found(fast: &EkadashiFast) -> (Vec<MahadvadashiKind>, u8) {
    (
        fast.mahadvadashi
            .as_ref()
            .map_or_else(Vec::new, |held| held.held.clone()),
        fast.day.day,
    )
}

/// M1, 13.267: a pure 11th holds two sunrises and the 12th one; the fast
/// is the second, as the table had it.
#[test]
fn unmilani_names_the_second_day_of_a_grown_eleventh() {
    let ends = [
        (1, 55.0),
        (3, 2.0),
        (4, 10.0),
        (5, 5.0),
        (5, 58.0),
        (6, 50.0),
    ];
    let fast = fast(KRISHNA, ends, &[]);
    assert_eq!(found(&fast), (vec![Unmilani], 4));
    assert_eq!(fast.choice, Which::Later);
    let held = fast.mahadvadashi.unwrap();
    assert!(held.clause.starts_with("HBV 13.267"));
    assert_eq!(held.joined, None);
}

/// The GCAL program's "Unmilani Trisprsa": the grown 11th's second day holds the
/// whole 12th and the 13th at its end.
#[test]
fn an_unmilani_may_be_a_trisprisha_too() {
    let ends = [
        (1, 55.0),
        (3, 2.0),
        (3, 50.0),
        (4, 45.0),
        (5, 40.0),
        (6, 35.0),
    ];
    assert_eq!(
        found(&fast(KRISHNA, ends, &[])),
        (vec![Unmilani, Trisprisha], 4)
    );
}

/// M2 and M3, 13.268 and C709: the 12th alone grows; any of it past the
/// second sunrise by default, a ghati of that day's daylight under 13.356.
#[test]
fn vyanjuli_is_the_twelfth_grown_by_any_or_by_half_a_muhurta() {
    let grown = |past: f64| {
        [
            (1, 55.0),
            (2, 58.0),
            (4, past),
            (4, 58.0),
            (5, 55.0),
            (6, 50.0),
        ]
    };
    assert_eq!(found(&fast(KRISHNA, grown(1.0), &[])), (vec![Vyanjuli], 4));
    assert_eq!(found(&fast(KRISHNA, grown(0.5), &[])), (vec![Vyanjuli], 4));
    let half = read(|rule| rule.vyanjuli = VyanjuliPast::HalfMuhurta);
    assert_eq!(
        found(&fast_under(&half, KRISHNA, grown(1.5), &[])),
        (vec![Vyanjuli], 4)
    );
    // Half a ghati is less than half a muhurta: no Vyanjuli, the same day.
    let fast = fast_under(&half, KRISHNA, grown(0.5), &[]);
    assert_eq!(found(&fast), (vec![], 4));
    assert!(fast.margin_seconds < 0.5 / 60.0 * 86_400.0 + 1.0);
}

/// C710: a 10th at the 11th's arunodaya makes it no *sampūrṇā* 11th, and
/// the Vyanjuli is refused unless the rule asks for none.
#[test]
fn a_pierced_eleventh_is_no_vyanjuli_unless_the_rule_says_any() {
    let pierced = [
        (1, 57.0),
        (2, 58.0),
        (4, 1.0),
        (4, 58.0),
        (5, 55.0),
        (6, 50.0),
    ];
    let fast = fast(KRISHNA, pierced, &[]);
    assert!(fast.pierced);
    assert_eq!(found(&fast), (vec![], 4));
    let any = read(|rule| rule.sampurna = false);
    assert_eq!(
        found(&fast_under(&any, KRISHNA, pierced, &[])),
        (vec![Vyanjuli], 4)
    );
}

/// M4, 13.269: the 11th at the day's arunodaya and sunrise, the 12th
/// within it, the 13th at the next sunrise; the fast is that day.
#[test]
fn trisprisha_keeps_the_day_the_three_touch() {
    let ends = [
        (1, 55.0),
        (2, 2.0),
        (2, 58.0),
        (3, 55.0),
        (4, 50.0),
        (5, 45.0),
    ];
    let fast = fast(KRISHNA, ends, &[]);
    assert_eq!(found(&fast), (vec![Trisprisha], 3));
    assert_eq!(fast.choice, Which::Earlier);
}

/// M5, C707: the 11th at arunodaya only, its sunrise the 12th's. Not a
/// Trisprisha by default, one by 13.269's letter; the day is the same.
#[test]
fn an_eleventh_at_arunodaya_alone_is_a_trisprisha_by_the_letter() {
    let ends = [
        (1, 20.0),
        (1, 58.0),
        (2, 57.0),
        (3, 55.0),
        (4, 50.0),
        (5, 45.0),
    ];
    assert_eq!(found(&fast(KRISHNA, ends, &[])), (vec![], 3));
    let letter = read(|rule| rule.trisprisha = TrisprishaAt::Arunodaya);
    assert_eq!(
        found(&fast_under(&letter, KRISHNA, ends, &[])),
        (vec![Trisprisha], 3)
    );
}

/// M6, 13.270: the full moon holds two sunrises; the 11th's day the table
/// gives is left and the 12th's fasted.
#[test]
fn pakshavardhini_moves_the_fast_to_the_twelfth() {
    let ends = [
        (1, 55.0),
        (2, 57.0),
        (3, 58.0),
        (4, 57.0),
        (5, 55.0),
        (7, 2.0),
    ];
    let fast = fast(SHUKLA, ends, &[]);
    assert_eq!(fast.choice, Which::Earlier);
    assert_eq!(fast.days[0].day, 3);
    assert_eq!(found(&fast), (vec![Pakshavardhini], 4));
    // The parana is the day after the 12th's.
    assert_eq!(fast.parana.unwrap().day.day, 5);
    // Dropped from the rule's kinds, the table's day stands.
    let without = read(|rule| rule.kinds.retain(|&kind| kind != Pakshavardhini));
    assert_eq!(found(&fast_under(&without, SHUKLA, ends, &[])), (vec![], 3));
}

/// M10, C711: an Unmilani and a Pakshavardhini both hold; the 12th's day,
/// the third, wins by the rule's order.
#[test]
fn the_twelfths_day_wins_over_an_unmilani() {
    let ends = [
        (1, 55.0),
        (3, 2.0),
        (4, 52.0),
        (5, 50.0),
        (6, 45.0),
        (8, 1.0),
    ];
    assert_eq!(
        found(&fast(SHUKLA, ends, &[])),
        (vec![Pakshavardhini, Unmilani], 5)
    );
    let unmilani_first = read(|rule| rule.kinds = vec![Unmilani, Pakshavardhini]);
    assert_eq!(
        found(&fast_under(&unmilani_first, SHUKLA, ends, &[])),
        (vec![Unmilani, Pakshavardhini], 4)
    );
}

/// The bright 12th to day 3's ghati `twelfth`.
fn bright(twelfth: f64) -> [End; 6] {
    [
        (1, 55.0),
        (2, 57.0),
        (3, twelfth),
        (4, 35.0),
        (5, 30.0),
        (6, 25.0),
    ]
}

/// M7 to M9, 13.288–289: a bright 12th with Punarvasu at its sunrise and
/// the next is Jaya when it lasts to sunset, and nothing when it does not;
/// with Shravana the sunset is not asked.
#[test]
fn a_nakshatra_kind_asks_for_the_sunset_but_shravana_does_not() {
    let punarvasu = stay(Nakshatra::Punarvasu, (2, 50.0), (4, 5.0));
    let fast = fast(SHUKLA, bright(40.0), &punarvasu);
    assert_eq!(found(&fast), (vec![Jaya], 4));
    let joined = fast.mahadvadashi.unwrap().joined.unwrap();
    assert_eq!(joined.nakshatra, Nakshatra::Punarvasu);
    assert_eq!(joined.span.to.get(), ghati(4, 5.0));
    assert_eq!(
        found(&self::fast(SHUKLA, bright(20.0), &punarvasu)),
        (vec![], 3)
    );
    let shravana = stay(Nakshatra::Shravana, (2, 50.0), (4, 5.0));
    assert_eq!(
        found(&self::fast(SHUKLA, bright(20.0), &shravana)),
        (vec![Vijaya], 4)
    );
    // Dark, it is no Vijaya: every defining verse says bright.
    let dark = [
        (1, 55.0),
        (2, 57.0),
        (3, 20.0),
        (4, 15.0),
        (5, 10.0),
        (6, 5.0),
    ];
    assert_eq!(found(&self::fast(KRISHNA, dark, &shravana)), (vec![], 3));
}

/// C708: a Punarvasu at the 12th's sunrise that ends with the 12th, not
/// at the next sunrise, is joined only against the tithi; one begun after
/// sunrise only by 13.286's "from sunrise".
#[test]
fn the_joining_is_read_against_the_day_the_tithi_or_the_daylight() {
    let ends = bright(40.0);
    let outlasting = stay(Nakshatra::Punarvasu, (2, 50.0), (3, 45.0));
    let begun = stay(Nakshatra::Punarvasu, (3, 10.0), (3, 20.0));
    let under = |joining, nakshatras: &[(Nakshatra, End)]| {
        found(&fast_under(
            &read(|rule| rule.joining = joining),
            SHUKLA,
            ends,
            nakshatras,
        ))
        .0
    };
    assert_eq!(under(NakshatraJoining::TwoSunrises, &outlasting), []);
    assert_eq!(
        under(NakshatraJoining::OutlastsTwelfth, &outlasting),
        [Jaya]
    );
    assert_eq!(
        under(NakshatraJoining::BegunInDaylight, &outlasting),
        [Jaya]
    );
    assert_eq!(under(NakshatraJoining::OutlastsTwelfth, &begun), []);
    assert_eq!(under(NakshatraJoining::BegunInDaylight, &begun), [Jaya]);
}

/// The parana of a nakshatra kind, on day 4 in the morning's first fifth,
/// ghatis 0 to 6: one row per clause of 13.290–291, as (from, from by,
/// to, to by).
#[test]
fn a_nakshatra_kinds_parana_keeps_the_meal_after_or_inside_it() {
    let parana = |twelfth: End, nakshatra, to: End| {
        let ends = [
            (1, 55.0),
            (2, 57.0),
            twelfth,
            (4, 35.0),
            (5, 30.0),
            (6, 25.0),
        ];
        let fast = fast(SHUKLA, ends, &stay(nakshatra, (2, 50.0), to));
        assert_eq!(fast.day.day, 4, "the 12th's day");
        let kept = fast.parana.unwrap().kept.unwrap();
        (
            kept.span.from.get(),
            kept.from_by,
            kept.span.to.get(),
            kept.to_by,
        )
    };
    // The 12th stands and outlasts Punarvasu: after it, "पारणं ततः".
    assert_eq!(
        parana((4, 10.0), Nakshatra::Punarvasu, (4, 5.0)),
        (
            ghati(4, 5.0),
            ParanaEdge::JoinedNakshatra,
            ghati(4, 6.0),
            ParanaEdge::Window
        )
    );
    // Punarvasu outlasts the 12th: inside the tithi, "तिथिमध्ये".
    assert_eq!(
        parana((4, 3.0), Nakshatra::Punarvasu, (4, 20.0)),
        (
            ghati(4, 0.0),
            ParanaEdge::Window,
            ghati(4, 3.0),
            ParanaEdge::TithiEnd
        )
    );
    // The 12th gone, Rohini grown: inside it, "तन्मध्ये".
    assert_eq!(
        parana((3, 40.0), Nakshatra::Rohini, (4, 4.0)),
        (
            ghati(4, 0.0),
            ParanaEdge::Window,
            ghati(4, 4.0),
            ParanaEdge::JoinedNakshatra
        )
    );
    // The 12th gone, Pushya grown: after it, "तदतिक्रमे".
    assert_eq!(
        parana((3, 40.0), Nakshatra::Pushya, (4, 3.0)),
        (
            ghati(4, 3.0),
            ParanaEdge::JoinedNakshatra,
            ghati(4, 6.0),
            ParanaEdge::Window
        )
    );
}

/// Each nakshatra kind is named by its own nakshatra.
#[test]
fn each_nakshatra_kind_is_its_nakshatras() {
    for (kind, nakshatra) in [
        (Jaya, Nakshatra::Punarvasu),
        (Vijaya, Nakshatra::Shravana),
        (Jayanti, Nakshatra::Rohini),
        (Papanashini, Nakshatra::Pushya),
    ] {
        let joined = stay(nakshatra, (2, 50.0), (4, 5.0));
        assert_eq!(found(&fast(SHUKLA, bright(40.0), &joined)), (vec![kind], 4));
    }
}

/// The rule's spelling, its defaults from `{}`, and its refusals by field.
#[test]
fn a_mahadvadashi_rule_reads_from_its_spelling_and_is_refused_by_its_field() {
    let rule = MahadvadashiRule::hari_bhakti_vilasa();
    let spelt = serde_json::to_value(&rule).unwrap();
    assert_eq!(
        spelt,
        serde_json::json!({
            "kinds": [
                "VIJAYA", "JAYA", "JAYANTI", "PAPANASHINI",
                "VYANJULI", "PAKSHAVARDHINI", "UNMILANI", "TRISPRISHA"
            ],
            "trisprisha": "SUNRISE",
            "joining": "TWO_SUNRISES",
            "vyanjuli": "ANY",
            "sampurna": true
        })
    );
    assert_eq!(
        serde_json::from_str::<MahadvadashiRule>("{}").unwrap(),
        rule
    );
    assert!(serde_json::from_str::<MahadvadashiRule>(r#"{"kind": []}"#).is_err());
    let refused = |kinds| {
        read(|rule| rule.kinds = kinds)
            .check()
            .unwrap_err()
            .field()
            .map(str::to_owned)
    };
    assert_eq!(refused(vec![]).as_deref(), Some("mahadvadashi.kinds"));
    assert_eq!(
        refused(vec![Jaya, Vijaya, Jaya]).as_deref(),
        Some("mahadvadashi.kinds[2]")
    );
}

/// A rule without mahadvadashis answers as before, and says none.
#[test]
fn the_vaishnava_rule_keeps_its_table() {
    let vaishnava = EkadashiRule::dharmasindhu().remove(0);
    assert_eq!(vaishnava.key, "EKADASHI_VAISHNAVA");
    let ends = [
        (1, 55.0),
        (2, 57.0),
        (3, 58.0),
        (4, 57.0),
        (5, 55.0),
        (7, 2.0),
    ];
    let fast = fast_under(&vaishnava, SHUKLA, ends, &[]);
    assert_eq!((fast.day.day, fast.mahadvadashi), (3, None));
    assert_eq!(vaishnava.reach(), 0);
    assert_eq!(EkadashiRule::hari_bhakti_vilasa().reach(), 6);
}
