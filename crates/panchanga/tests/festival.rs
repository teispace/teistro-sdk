//! Festival rules held to *Dharmasindhu*'s own worked examples
//! (`03-design/festival-rules.md` §5.1).
//!
//! The book states its examples in ghatis after sunrise at which a tithi
//! ends, on a day of sixty. So the days here are synthetic: day `k`
//! rises at Julian day `k`, sets at `k + 0.5`, and a ghati is a sixtieth
//! of a day. The night's middle is ghati 45, the book's niśītha. The
//! expected day is the book's answer, not a reading of its prose.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index fixed lists"
)]

use teistro_calendar::EraNumber;
use teistro_calendar::lunisolar::MonthKind;
use teistro_core::catalogue::{Era, Masa, Nakshatra, Tithi};
use teistro_core::interval::Interval;
use teistro_core::settings::LunarMonth as Convention;
use teistro_panchanga::festival::{
    Case, Choice, DayPart, Decided, Edge, EkadashiRule, FestivalDay, FestivalRule, Guard,
    Observances, Predicate, Unjudged, Which, Window, ekadashis, observances, yugma,
};
use teistro_panchanga::month;

mod common;

use common::{days_of, days_over, ghati};

fn janmashtami() -> FestivalRule {
    FestivalRule::dharmasindhu()
        .into_iter()
        .find(|rule| rule.key == "JANMASHTAMI")
        .unwrap()
}

/// The 7th ends `saptami` ghatis after day 1's sunrise and the 8th
/// `ashtami` ghatis after day 2's.
fn one_janmashtami(saptami: f64, ashtami: f64, nakshatras: &[(Nakshatra, f64)]) -> (u8, Case) {
    let tithis = [
        (Tithi::KrishnaShashthi, ghati(0, 30.0)),
        (Tithi::KrishnaSaptami, ghati(1, saptami)),
        (Tithi::KrishnaAshtami, ghati(2, ashtami)),
        (Tithi::KrishnaNavami, ghati(3, 50.0)),
        (Tithi::KrishnaDashami, ghati(5, 0.0)),
    ];
    let days = days_of(&tithis, nakshatras, Masa::Shravana, MonthKind::Nija);
    let answer = observances(&[janmashtami()], &days).unwrap();
    assert!(answer.unjudged.is_empty(), "{:?}", answer.unjudged);
    let [observance] = answer.observances.as_slice() else {
        panic!("one Janmashtami, not {:?}", answer.observances);
    };
    (observance.day.day - 1, observance.case)
}

#[test]
fn janmashtami_takes_the_book_s_day_in_each_of_its_four_examples() {
    // p. 49, read off the page image: (7th ends, 8th ends) -> the day.
    let none: [(Nakshatra, f64); 0] = [];
    assert_eq!(one_janmashtami(40.0, 42.0, &none), (1, Case::EarlierOnly));
    assert_eq!(one_janmashtami(47.0, 46.0, &none), (2, Case::LaterOnly));
    assert_eq!(one_janmashtami(42.0, 46.0, &none), (2, Case::Both));
    assert_eq!(one_janmashtami(47.0, 42.0, &none), (2, Case::Neither));
}

#[test]
fn rohini_joined_at_nishitha_outranks_the_tithi_alone() {
    // The 8th holds day 1's niśītha alone, which alone would take day 1;
    // Rohini with it at day 2's niśītha takes day 2 once the 8th reaches
    // it too. Here the 8th ends at ghati 46 of day 2 and holds both.
    let rohini_later = [
        (Nakshatra::Krittika, ghati(2, 30.0)),
        (Nakshatra::Rohini, ghati(3, 10.0)),
    ];
    assert_eq!(one_janmashtami(42.0, 46.0, &rohini_later), (2, Case::Both));
    // Rohini at day 1's niśītha only: the earlier, though both hold the 8th.
    let rohini_earlier = [
        (Nakshatra::Krittika, ghati(1, 40.0)),
        (Nakshatra::Rohini, ghati(2, 20.0)),
    ];
    assert_eq!(
        one_janmashtami(42.0, 46.0, &rohini_earlier),
        (1, Case::Both)
    );
}

#[test]
fn the_yugma_verse_pairs_what_it_names_and_nothing_else() {
    assert_eq!(yugma(Tithi::ShuklaDvitiya), Some(Which::Later));
    assert_eq!(yugma(Tithi::ShuklaTritiya), Some(Which::Earlier));
    assert_eq!(yugma(Tithi::KrishnaEkadashi), Some(Which::Later));
    assert_eq!(yugma(Tithi::KrishnaDwadashi), Some(Which::Earlier));
    assert_eq!(yugma(Tithi::ShuklaChaturdashi), Some(Which::Later));
    assert_eq!(yugma(Tithi::Purnima), Some(Which::Earlier));
    assert_eq!(yugma(Tithi::Amavasya), Some(Which::Later));
    assert_eq!(yugma(Tithi::ShuklaPratipada), Some(Which::Earlier));
    for unpaired in [
        Tithi::ShuklaDashami,
        Tithi::KrishnaTrayodashi,
        Tithi::KrishnaChaturdashi,
        Tithi::KrishnaPratipada,
    ] {
        assert_eq!(yugma(unpaired), None, "{unpaired:?}");
    }
}

fn a_rule(tithi: Tithi, decide: Vec<Guard>, otherwise: Choice) -> FestivalRule {
    FestivalRule {
        key: "TEST".to_owned(),
        source: "a test".to_owned(),
        month: Masa::Shravana,
        convention: Convention::Amanta,
        tithi,
        in_adhika: false,
        at: Window::Part {
            part: DayPart::Madhyahna,
        },
        decide,
        otherwise,
    }
}

#[test]
fn a_rule_that_cannot_be_judged_is_refused_by_the_field_that_is_wrong() {
    let yugma_of_the_tenth = a_rule(Tithi::ShuklaDashami, vec![], Choice::ByYugma);
    assert_eq!(
        yugma_of_the_tenth.check().unwrap_err().field(),
        Some("decide")
    );
    let thirty_one = a_rule(
        Tithi::ShuklaDvitiya,
        vec![Guard::new(
            [Predicate::Lasts {
                day: Which::Later,
                from: Edge::Sunrise,
                ghatis: 31,
            }],
            Choice::Later,
        )],
        Choice::Earlier,
    );
    assert_eq!(
        thirty_one.check().unwrap_err().field(),
        Some("decide.when.ghatis")
    );
    let mut keyless = a_rule(Tithi::ShuklaDvitiya, vec![], Choice::ByYugma);
    keyless.key = " ".to_owned();
    assert_eq!(keyless.check().unwrap_err().field(), Some("key"));
    for rule in FestivalRule::dharmasindhu() {
        rule.check().unwrap();
    }
}

/// A 2nd holding midday on both days goes to the later (joined to the
/// 3rd), and a 3rd to the earlier (joined to the 2nd).
#[test]
fn by_yugma_the_first_of_a_pair_takes_the_later_day_and_the_second_the_earlier() {
    for (tithi, previous, day) in [
        (Tithi::KrishnaDvitiya, Tithi::KrishnaPratipada, 2),
        (Tithi::KrishnaTritiya, Tithi::KrishnaDvitiya, 1),
    ] {
        let tithis = [
            (previous, ghati(1, 10.0)),
            (tithi, ghati(2, 40.0)),
            (Tithi::KrishnaDashami, ghati(5, 0.0)),
        ];
        let days = days_of(&tithis, &[], Masa::Shravana, MonthKind::Nija);
        let rule = a_rule(
            tithi,
            vec![Guard::new(
                [Predicate::Case { case: Case::Both }],
                Choice::ByYugma,
            )],
            Choice::Earlier,
        );
        let answer = observances(&[rule], &days).unwrap();
        let observance = &answer.observances[0];
        assert_eq!(
            (observance.case, observance.day.day - 1),
            (Case::Both, day),
            "{tithi:?}"
        );
        assert_eq!(observance.decided_by, Decided::Guard { index: 0 });
    }
}

#[test]
fn a_rule_holds_in_its_own_nija_month_only_unless_it_asks_for_the_adhika() {
    let tithis = [
        (Tithi::KrishnaSaptami, ghati(1, 40.0)),
        (Tithi::KrishnaAshtami, ghati(2, 42.0)),
        (Tithi::KrishnaDashami, ghati(5, 0.0)),
    ];
    let none: [(Nakshatra, f64); 0] = [];
    let other = days_of(&tithis, &none, Masa::Bhadrapada, MonthKind::Nija);
    assert!(
        observances(&[janmashtami()], &other)
            .unwrap()
            .observances
            .is_empty()
    );
    let adhika = days_of(&tithis, &none, Masa::Shravana, MonthKind::Adhika);
    assert!(
        observances(&[janmashtami()], &adhika)
            .unwrap()
            .observances
            .is_empty()
    );
    let mut in_adhika = janmashtami();
    in_adhika.in_adhika = true;
    assert_eq!(
        observances(&[in_adhika], &adhika)
            .unwrap()
            .observances
            .len(),
        1
    );
    // Under purnimanta the dark fortnight of amanta Shravana is Bhadrapada.
    let mut purnimanta = janmashtami();
    purnimanta.convention = Convention::Purnimanta;
    assert!(
        observances(
            &[purnimanta.clone()],
            &days_of(&tithis, &none, Masa::Shravana, MonthKind::Nija)
        )
        .unwrap()
        .observances
        .is_empty()
    );
    purnimanta.month = Masa::Bhadrapada;
    assert_eq!(
        observances(
            &[purnimanta],
            &days_of(&tithis, &none, Masa::Shravana, MonthKind::Nija)
        )
        .unwrap()
        .observances
        .len(),
        1
    );
}

fn shipped(key: &str) -> FestivalRule {
    FestivalRule::dharmasindhu()
        .into_iter()
        .find(|rule| rule.key == key)
        .unwrap()
}

/// The case, the day (1-based, as the book counts) and the deciding guard.
fn decided(rule: &FestivalRule, days: &[FestivalDay]) -> (Case, u8, Decided) {
    let observance = &observances(std::slice::from_ref(rule), days)
        .unwrap()
        .observances[0];
    (
        observance.case,
        observance.day.day - 1,
        observance.decided_by,
    )
}

/// Vijaya Dashami (p. 71): the earlier day holds aparahna alone, but the
/// later day's 10th lasts three muhurtas and Shravana joins it there alone,
/// standing in its aparahna, so the later.
#[test]
fn vijaya_dashami_yields_to_a_later_day_with_three_muhurtas_and_shravana_alone() {
    let rule = shipped("VIJAYA_DASHAMI");
    // Aparahna is the fourth fifth of the daylight: ghatis 18 to 24.
    // The 10th from ghati 10 of day 1 to ghati 7 of day 2; a daylight
    // muhurta is two ghatis, so three of them end at ghati 6.
    let tenth_until = |ends| {
        [
            (Tithi::ShuklaNavami, ghati(1, 10.0)),
            (Tithi::ShuklaDashami, ends),
            (Tithi::ShuklaDwadashi, ghati(5, 0.0)),
        ]
    };
    let shravana = |from, to| [(Nakshatra::UttaraAshadha, from), (Nakshatra::Shravana, to)];
    let ashwina = |tithis: &[(Tithi, f64)], nakshatras: &[(Nakshatra, f64)]| {
        days_of(tithis, nakshatras, Masa::Ashwina, MonthKind::Nija)
    };
    let later_alone = shravana(ghati(2, 1.0), ghati(3, 0.0));
    let days = ashwina(&tenth_until(ghati(2, 7.0)), &later_alone);
    assert_eq!(
        decided(&rule, &days),
        (Case::EarlierOnly, 2, Decided::Guard { index: 4 })
    );
    // Ending at ghati 5, short of three muhurtas: the earlier.
    let days = ashwina(&tenth_until(ghati(2, 5.0)), &later_alone);
    assert_eq!(
        decided(&rule, &days),
        (Case::EarlierOnly, 1, Decided::Guard { index: 5 })
    );
    // Shravana joining the 10th on the earlier day too: "on the later day
    // only" fails, and both days joined take the earlier.
    let both = shravana(ghati(1, 50.0), ghati(3, 0.0));
    let days = ashwina(&tenth_until(ghati(2, 7.0)), &both);
    assert_eq!(
        decided(&rule, &days),
        (Case::EarlierOnly, 1, Decided::Guard { index: 3 })
    );
    // Shravana over before the later day's aparahna: the Nirnaya-sindhu's
    // condition fails, so the earlier.
    let before_aparahna = [
        (Nakshatra::UttaraAshadha, ghati(2, 1.0)),
        (Nakshatra::Shravana, ghati(2, 15.0)),
        (Nakshatra::Dhanishtha, ghati(4, 0.0)),
    ];
    let days = ashwina(&tenth_until(ghati(2, 7.0)), &before_aparahna);
    assert_eq!(
        decided(&rule, &days),
        (Case::EarlierOnly, 1, Decided::Guard { index: 5 })
    );
}

/// Vijaya Dashami (p. 71), the author's own view: the later day alone
/// holds aparahna, but Shravana joins the 10th only on the earlier day,
/// in its evening, so the earlier.
#[test]
fn vijaya_dashami_held_later_alone_yields_to_shravana_on_the_earlier_evening() {
    let rule = shipped("VIJAYA_DASHAMI");
    // The 10th from ghati 26 of day 1, after its aparahna, to ghati 40 of
    // day 2.
    let tithis = [
        (Tithi::ShuklaNavami, ghati(1, 26.0)),
        (Tithi::ShuklaDashami, ghati(2, 40.0)),
        (Tithi::ShuklaDwadashi, ghati(5, 0.0)),
    ];
    let evening = [
        (Nakshatra::UttaraAshadha, ghati(1, 20.0)),
        (Nakshatra::Shravana, ghati(1, 50.0)),
        (Nakshatra::Dhanishtha, ghati(4, 0.0)),
    ];
    let days = days_of(&tithis, &evening, Masa::Ashwina, MonthKind::Nija);
    assert_eq!(
        decided(&rule, &days),
        (Case::LaterOnly, 1, Decided::Guard { index: 1 })
    );
    let days = days_of(&tithis, &[], Masa::Ashwina, MonthKind::Nija);
    assert_eq!(
        decided(&rule, &days),
        (Case::LaterOnly, 2, Decided::Guard { index: 2 })
    );
}

/// Vijaya Dashami (p. 71): both days holding aparahna, or neither, take
/// the earlier unless Shravana joins the 10th on one day only, and then
/// that day.
#[test]
fn vijaya_dashami_held_on_both_days_or_neither_goes_to_shravana_alone() {
    let rule = shipped("VIJAYA_DASHAMI");
    let tenth = |from, to| {
        [
            (Tithi::ShuklaNavami, from),
            (Tithi::ShuklaDashami, to),
            (Tithi::ShuklaDwadashi, ghati(5, 0.0)),
        ]
    };
    let later_alone = [
        (Nakshatra::UttaraAshadha, ghati(2, 1.0)),
        (Nakshatra::Shravana, ghati(3, 0.0)),
    ];
    // Aparahna is ghatis 18 to 24: from ghati 15 of day 1 to ghati 25 of
    // day 2 holds both, from 25 to 17 neither.
    for (tithis, case) in [
        (tenth(ghati(1, 15.0), ghati(2, 25.0)), Case::Both),
        (tenth(ghati(1, 25.0), ghati(2, 17.0)), Case::Neither),
    ] {
        let days = days_of(&tithis, &later_alone, Masa::Ashwina, MonthKind::Nija);
        assert_eq!(
            decided(&rule, &days),
            (case, 2, Decided::Guard { index: 6 })
        );
        let days = days_of(&tithis, &[], Masa::Ashwina, MonthKind::Nija);
        assert_eq!(decided(&rule, &days), (case, 1, Decided::Otherwise));
    }
}

/// Lakshmi puja (p. 77): the new moon reaching the later day's pradosha in
/// part takes the later day only when it lasts more than a ghati into the
/// night. The synthetic night is thirty ghatis, so a night's ghati is one.
#[test]
fn lakshmi_puja_takes_the_later_day_only_a_ghati_into_its_night() {
    let rule = shipped("LAKSHMI_PUJA");
    // The new moon from ghati 32 of day 1, inside its pradosha (30 to 36).
    let new_moon_until = |ends| {
        [
            (Tithi::KrishnaChaturdashi, ghati(1, 32.0)),
            (Tithi::Amavasya, ends),
            (Tithi::ShuklaDvitiya, ghati(5, 0.0)),
        ]
    };
    let ashwina = |tithis: &[(Tithi, f64)]| days_of(tithis, &[], Masa::Ashwina, MonthKind::Nija);
    let days = ashwina(&new_moon_until(ghati(2, 32.0)));
    assert_eq!(
        decided(&rule, &days),
        (Case::UnequalParts, 2, Decided::Guard { index: 0 })
    );
    // Half a ghati past sunset: the later day holds more of pradosha's
    // span than none, but not a ghati, so the earlier.
    let days = ashwina(&new_moon_until(ghati(2, 30.5)));
    assert_eq!(
        decided(&rule, &days),
        (Case::UnequalParts, 1, Decided::Otherwise)
    );
}

/// The new moon ends its month, so its month is read at a sunrise inside
/// it even when the next sunrise is already in the next month.
#[test]
fn a_new_moon_is_read_in_the_month_it_ends() {
    let rule = shipped("LAKSHMI_PUJA");
    // The 14th until ghati 5 of day 2, the new moon until ghati 55 of
    // day 2: kshaya, holding no sunrise. Day 2's sunrise is in the 14th,
    // of Ashwina; day 3's in the next month.
    let tithis = [
        (Tithi::KrishnaChaturdashi, ghati(2, 5.0)),
        (Tithi::Amavasya, ghati(2, 55.0)),
        (Tithi::ShuklaDvitiya, ghati(5, 0.0)),
    ];
    let mut days = days_of(&tithis, &[], Masa::Ashwina, MonthKind::Nija);
    for day in &mut days[3..] {
        day.month = month::of(
            Masa::Kartika,
            Tithi::ShuklaPratipada,
            Convention::Amanta,
            MonthKind::Nija,
        );
    }
    let answer = observances(&[rule], &days).unwrap();
    let observance = &answer.observances[0];
    // Pradosha on day 2 is ghatis 30 to 36: held whole; day 3's is not.
    assert_eq!(
        (observance.case, observance.day.day - 1),
        (Case::EarlierOnly, 2)
    );
}

#[test]
fn the_shipped_rules_round_trip_through_their_json_record() {
    let rules = FestivalRule::dharmasindhu();
    let text = serde_json::to_string(&rules).unwrap();
    let back: Vec<FestivalRule> = serde_json::from_str(&text).unwrap();
    assert_eq!(back, rules);
    assert!(text.contains(r#""window":"NISHITHA""#), "{text}");
    assert!(text.contains(r#""is":"JOINED""#), "{text}");
}

/// Every string an answer's JSON holds, by its path with list indices
/// dropped.
fn string_paths(
    value: &serde_json::Value,
    at: &str,
    into: &mut std::collections::BTreeSet<String>,
) {
    match value {
        serde_json::Value::String(_) => {
            into.insert(at.to_owned());
        }
        serde_json::Value::Array(list) => {
            for item in list {
                string_paths(item, at, into);
            }
        }
        serde_json::Value::Object(fields) => {
            for (field, inner) in fields {
                let path = if at.is_empty() {
                    field.clone()
                } else {
                    format!("{at}.{field}")
                };
                string_paths(inner, &path, into);
            }
        }
        _ => {}
    }
}

/// `Observances::MEMBERS` names every catalogue member an answer holds,
/// and nothing else: each string in its JSON is either listed there or
/// listed here as not a member, and each of both lists is present.
#[test]
fn an_answer_names_its_catalogue_members_where_its_table_says() {
    // Every string an answer holds that is not a catalogue member.
    const NOT_MEMBERS: [&str; 14] = [
        "observances.rule",
        "observances.day.resolution.kind",
        "observances.extents.day.resolution.kind",
        "observances.case",
        "observances.decidedBy.by",
        "observances.choice",
        "ekadashis.rule",
        "ekadashis.days.resolution.kind",
        "ekadashis.day.resolution.kind",
        "ekadashis.piercedAt",
        "ekadashis.excess",
        "ekadashis.choice",
        "unjudged.rule",
        "unjudged.why",
    ];
    let tithis = [
        (Tithi::KrishnaShashthi, ghati(0, 30.0)),
        (Tithi::KrishnaSaptami, ghati(1, 40.0)),
        (Tithi::KrishnaAshtami, ghati(2, 42.0)),
        (Tithi::KrishnaNavami, ghati(3, 50.0)),
        (Tithi::KrishnaDashami, ghati(5, 0.0)),
    ];
    let days = days_of(&tithis, &[], Masa::Shravana, MonthKind::Nija);
    let mut answer: Observances = observances(&[janmashtami()], &days).unwrap();
    let era = Some(EraNumber {
        era: Era::Vikrama,
        year: 2081,
    });
    answer.observances[0].day.era = era;
    for extent in &mut answer.observances[0].extents {
        extent.day.era = era;
    }
    answer.unjudged.push(Unjudged {
        rule: "JANMASHTAMI".to_owned(),
        tithi: Interval::literal(0.0, 1.0),
        why: "a reason".to_owned(),
    });
    // An 11th pierced at arunodaya, so every field of a fast is written.
    let eleventh = [
        (Tithi::KrishnaNavami, ghati(0, 50.0)),
        (Tithi::KrishnaDashami, ghati(1, 58.0)),
        (Tithi::KrishnaEkadashi, ghati(3, 1.0)),
        (Tithi::KrishnaDwadashi, ghati(3, 58.0)),
        (Tithi::KrishnaTrayodashi, ghati(5, 40.0)),
    ];
    let days = days_over(6, &eleventh, &[], Masa::Kartika, MonthKind::Nija);
    let mut answer = answer.merged(ekadashis(&EkadashiRule::dharmasindhu()[..1], &days).unwrap());
    let fast = &mut answer.ekadashis[0];
    assert!(fast.pierced_at.is_some());
    fast.day.era = era;
    for day in &mut fast.days {
        day.era = era;
    }
    let mut found = std::collections::BTreeSet::new();
    string_paths(&serde_json::to_value(&answer).unwrap(), "", &mut found);
    let members: std::collections::BTreeSet<String> = Observances::MEMBERS
        .iter()
        .map(|(list, path, _)| format!("{list}.{path}"))
        .collect();
    let plain: std::collections::BTreeSet<String> =
        NOT_MEMBERS.iter().map(|path| (*path).to_owned()).collect();
    assert!(members.is_disjoint(&plain));
    let listed: std::collections::BTreeSet<String> = members.union(&plain).cloned().collect();
    assert_eq!(
        found, listed,
        "a string path is unlisted, or a listed one is gone"
    );
}
