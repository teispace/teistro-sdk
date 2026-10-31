//! Anniversaries over synthetic days (`03-design/shraddha.md`): the clauses
//! a real sky reaches too rarely to test, the kshaya year's joined lunation
//! above all (C669), and the refusals.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

mod common;

use common::{days_of, ghati};
use teistro_calendar::lunisolar::MonthKind;
use teistro_core::catalogue::{Masa, Nakshatra, Tithi};
use teistro_core::interval::Interval;
use teistro_panchanga::festival::{
    Anniversary, AnniversaryDay, AnniversaryKind, DayClause, FestivalRule, FirstYearAdhika, Guard,
    Half, InSamsarpa, JanmaOptions, MonthClause, Origin, OriginFound, Predicate, TithiShape,
    Window, anniversaries,
};

fn asked(month: Masa, tithi: Tithi, kind: AnniversaryKind) -> (Anniversary, OriginFound) {
    let origin = Origin::Lunar {
        month,
        tithi,
        adhika: false,
        half: Half::First,
        at: None,
    };
    (
        Anniversary {
            key: "ONE".to_owned(),
            origin,
            kind,
            window: None,
            first_year_adhika: FirstYearAdhika::Adhika,
            in_samsarpa: InSamsarpa::Also,
            janma: JanmaOptions::default(),
        },
        OriginFound::lunar(month, tithi, false, Half::First, None),
    )
}

/// The bright 5th from late on day 0 to near day 1's end: day 1 alone
/// holds its kutapa.
fn fifth() -> [(Tithi, f64); 3] {
    [
        (Tithi::ShuklaChaturthi, ghati(0, 54.0)),
        (Tithi::ShuklaPanchami, ghati(1, 57.0)),
        (Tithi::ShuklaShashthi, ghati(4, 0.0)),
    ]
}

/// C669: in a kshaya year the joined lunation answers to both its names,
/// so an anniversary of the skipped month is kept in it on its tithi, and
/// one of the lunation's own first name is kept there as ever.
#[test]
fn a_kshaya_lunation_keeps_both_its_months_anniversaries() {
    let kshaya = days_of(&fifth(), &[], Masa::Margashirsha, MonthKind::Kshaya);
    for month in [Masa::Pausha, Masa::Margashirsha] {
        let found = anniversaries(
            &[asked(
                month,
                Tithi::ShuklaPanchami,
                AnniversaryKind::Ekoddishta,
            )],
            &kshaya,
            &[],
        )
        .unwrap();
        assert_eq!(found.anniversaries.len(), 1, "{month:?}");
        let kept = &found.anniversaries[0];
        assert_eq!(kept.month_by, MonthClause::KshayaJoined, "{month:?}");
        assert_eq!(kept.observance.day.day, 2, "{month:?}");
    }
    // An ordinary Margashirsha keeps Pausha's in no lunation of its own.
    let ordinary = days_of(&fifth(), &[], Masa::Margashirsha, MonthKind::Nija);
    let found = anniversaries(
        &[asked(
            Masa::Pausha,
            Tithi::ShuklaPanchami,
            AnniversaryKind::Ekoddishta,
        )],
        &ordinary,
        &[],
    )
    .unwrap();
    assert_eq!(found.anniversaries, Vec::new());
}

/// C669: an adhika month the almanac found to be the samsarpa keeps an
/// ordinary anniversary of its name as a second day; one that is not, or
/// an anniversary asking for the nija month alone, keeps none there.
#[test]
fn a_samsarpa_keeps_its_months_anniversary_as_a_second_day() {
    let adhika = days_of(&fifth(), &[], Masa::Kartika, MonthKind::Adhika);
    let samsarpa = [Interval::literal(
        adhika[0].sunrise.get(),
        adhika[4].next_sunrise.get(),
    )];
    let kept = |anniversary: (Anniversary, OriginFound), samsarpas: &[Interval]| {
        anniversaries(&[anniversary], &adhika, samsarpas)
            .unwrap()
            .anniversaries
            .iter()
            .map(|kept| (kept.month_by, kept.observance.day.day))
            .collect::<Vec<_>>()
    };
    let shraddha = || {
        asked(
            Masa::Kartika,
            Tithi::ShuklaPanchami,
            AnniversaryKind::Ekoddishta,
        )
    };
    assert_eq!(kept(shraddha(), &samsarpa), [(MonthClause::Samsarpa, 2)]);
    assert_eq!(kept(shraddha(), &[]), []);
    // A samsarpa elsewhere is not this adhika month.
    let elsewhere = [Interval::literal(
        adhika[0].sunrise.get() - 60.0,
        adhika[0].sunrise.get() - 31.0,
    )];
    assert_eq!(kept(shraddha(), &elsewhere), []);
    let (mut nija, origin) = shraddha();
    nija.in_samsarpa = InSamsarpa::Nija;
    assert_eq!(kept((nija, origin), &samsarpa), []);
    let janma = asked(
        Masa::Kartika,
        Tithi::ShuklaPanchami,
        AnniversaryKind::JanmaTithi,
    );
    assert_eq!(kept(janma, &samsarpa), []);
    // Another month's anniversary is not this samsarpa's.
    let other = asked(
        Masa::Ashwina,
        Tithi::ShuklaPanchami,
        AnniversaryKind::Ekoddishta,
    );
    assert_eq!(kept(other, &samsarpa), []);
}

/// DS p. 266 (C663): a tithi holding both days' kutapa wholly gives the
/// earlier day; a shraddha of a death in Bhadrapada's dark half is a
/// parvana, at aparahna, and any other an ekoddishta (C665).
#[test]
fn the_choice_and_the_kind_follow_the_texts() {
    let both = [
        (Tithi::ShuklaChaturthi, ghati(0, 12.0)),
        (Tithi::ShuklaPanchami, ghati(1, 24.0)),
        (Tithi::ShuklaShashthi, ghati(4, 0.0)),
    ];
    let days = days_of(&both, &[], Masa::Kartika, MonthKind::Nija);
    let found = anniversaries(
        &[asked(
            Masa::Kartika,
            Tithi::ShuklaPanchami,
            AnniversaryKind::Shraddha,
        )],
        &days,
        &[],
    )
    .unwrap();
    let kept = &found.anniversaries[0];
    assert_eq!(kept.observance.day.day, 1);
    assert_eq!(kept.kind, AnniversaryKind::Ekoddishta);
    assert_eq!(
        kept.observance.extents[0].window.days(),
        kept.observance.extents[1].window.days()
    );
    let dark = OriginFound::lunar(
        Masa::Bhadrapada,
        Tithi::KrishnaPanchami,
        false,
        Half::First,
        None,
    );
    assert!(dark.is_parvana());
    assert!(
        OriginFound::lunar(Masa::Kartika, Tithi::Amavasya, false, Half::First, None).is_parvana()
    );
    assert!(
        !OriginFound::lunar(
            Masa::Ashwina,
            Tithi::KrishnaPanchami,
            false,
            Half::First,
            None
        )
        .is_parvana()
    );
    assert_eq!(
        AnniversaryKind::Parvana.window(),
        Window::Part {
            part: teistro_panchanga::festival::DayPart::Aparahna
        }
    );
}

/// What cannot be kept is refused by its field.
#[test]
fn a_refusal_names_the_field() {
    let (mut janma, origin) = asked(
        Masa::Kartika,
        Tithi::ShuklaPanchami,
        AnniversaryKind::JanmaTithi,
    );
    janma.window = Some(Window::Sunset);
    let days = days_of(&fifth(), &[], Masa::Kartika, MonthKind::Nija);
    let refused = anniversaries(&[(janma, origin.clone())], &days, &[]).unwrap_err();
    assert_eq!(refused.field(), Some("window"));
    let (mut shraddha, _) = asked(
        Masa::Kartika,
        Tithi::ShuklaPanchami,
        AnniversaryKind::Shraddha,
    );
    shraddha.janma.two_muhurta_floor = true;
    let refused = anniversaries(&[(shraddha, origin.clone())], &days, &[]).unwrap_err();
    assert_eq!(refused.field(), Some("janma"));
    let (mut blank, _) = asked(
        Masa::Kartika,
        Tithi::ShuklaPanchami,
        AnniversaryKind::Shraddha,
    );
    blank.key = " ".to_owned();
    assert_eq!(
        anniversaries(&[(blank, origin)], &days, &[])
            .unwrap_err()
            .field(),
        Some("key")
    );
}

/// The bright 5th of Kartika from one moment to another, and the day an
/// anniversary of `kind` keeps on it, optionally with `shape` applied.
fn kept_on(
    from: f64,
    to: f64,
    nakshatras: &[(Nakshatra, f64)],
    shape: impl FnOnce(&mut Anniversary),
) -> AnniversaryDay {
    let tithis = [
        (Tithi::ShuklaChaturthi, from),
        (Tithi::ShuklaPanchami, to),
        (Tithi::ShuklaShashthi, ghati(6, 0.0)),
    ];
    let days = days_of(&tithis, nakshatras, Masa::Kartika, MonthKind::Nija);
    let (mut anniversary, origin) = asked(
        Masa::Kartika,
        Tithi::ShuklaPanchami,
        AnniversaryKind::Ekoddishta,
    );
    shape(&mut anniversary);
    let origin = OriginFound {
        nakshatra: Some(Nakshatra::Rohini),
        ..origin
    };
    let mut found = anniversaries(&[(anniversary, origin)], &days, &[]).unwrap();
    assert_eq!(found.anniversaries.len(), 1);
    found.anniversaries.remove(0)
}

fn kind(kind: AnniversaryKind) -> impl FnOnce(&mut Anniversary) {
    move |anniversary| anniversary.kind = kind
}

type Row = (
    &'static str,
    AnniversaryDay,
    TithiShape,
    DayClause,
    u8,
    &'static str,
);

/// Each row holds a clause of `anniversary-span.md` §2 on the case that
/// reaches it,
/// with how the tithi stood. Kutapa to rauhina is 14 to 18 ghatis past
/// sunrise of a thirty-ghati daylight.
fn assert_named<const N: usize>(rows: [Row; N]) {
    for (name, day, shape, clause, found, needs) in rows {
        let rule = &day.day_rule;
        assert_eq!(rule.tithi, shape, "{name}");
        assert_eq!(rule.clause, clause, "{name}");
        assert_eq!(day.observance.day.day, found, "{name}");
        assert!(rule.source.contains(needs), "{name}: {}", rule.source);
    }
}
#[test]
fn an_ekoddishtas_day_names_the_clause_and_the_tithis_shape() {
    use AnniversaryKind::Ekoddishta;
    let n = &[][..];
    assert_named([
        (
            "one sunrise, one day's kutapa",
            kept_on(ghati(0, 54.0), ghati(1, 57.0), n, kind(Ekoddishta)),
            TithiShape::Ordinary,
            DayClause::OneDay,
            2,
            "kutapa to rauhina alone",
        ),
        (
            "no sunrise, still one day's kutapa",
            kept_on(ghati(1, 5.0), ghati(1, 50.0), n, kind(Ekoddishta)),
            TithiShape::Kshaya,
            DayClause::OneDay,
            2,
            "kutapa to rauhina alone",
        ),
        (
            "no sunrise and neither kutapa",
            kept_on(ghati(1, 20.0), ghati(1, 55.0), n, kind(Ekoddishta)),
            TithiShape::Kshaya,
            DayClause::Neither,
            2,
            "the Gaudas' later refused",
        ),
        (
            "two sunrises, both kutapas whole",
            kept_on(ghati(0, 50.0), ghati(2, 20.0), n, kind(Ekoddishta)),
            TithiShape::Vriddhi,
            DayClause::BothWhole,
            2,
            "on both days or equally",
        ),
        (
            "both in part, the earlier more",
            kept_on(ghati(1, 16.0), ghati(2, 15.0), n, kind(Ekoddishta)),
            TithiShape::Ordinary,
            DayClause::GreaterShare,
            2,
            "the greater",
        ),
        (
            "both in part, the later more",
            kept_on(ghati(1, 17.0), ghati(2, 17.0), n, kind(Ekoddishta)),
            TithiShape::Ordinary,
            DayClause::GreaterShare,
            3,
            "the greater",
        ),
    ]);
}

#[test]
fn a_parvanas_day_names_the_clause_and_the_tithis_shape() {
    use AnniversaryKind::Parvana;
    let n = &[][..];
    assert_named([
        (
            "a parvana's aparahna, one day",
            kept_on(ghati(0, 54.0), ghati(1, 57.0), n, kind(Parvana)),
            TithiShape::Ordinary,
            DayClause::OneDay,
            2,
            "the aparahna alone",
        ),
        (
            "a parvana's aparahna, on both",
            kept_on(ghati(0, 50.0), ghati(2, 26.0), n, kind(Parvana)),
            TithiShape::Vriddhi,
            DayClause::BothWhole,
            2,
            "on both days, on neither",
        ),
        (
            "a parvana's aparahna, in part",
            kept_on(ghati(1, 19.0), ghati(2, 19.0), n, kind(Parvana)),
            TithiShape::Ordinary,
            DayClause::GreaterShare,
            2,
            "the greater",
        ),
    ]);
}

#[test]
fn a_celebrations_day_names_the_clause_and_the_tithis_shape() {
    use AnniversaryKind::{Event, JanmaTithi};
    let n = &[][..];
    assert_named([
        (
            "a janma tithi on no sunrise, the day it runs",
            kept_on(ghati(1, 5.0), ghati(1, 50.0), n, kind(JanmaTithi)),
            TithiShape::Kshaya,
            DayClause::Neither,
            2,
            "Kane V.1 p. 89",
        ),
        (
            "a janma tithi on two sunrises, the earlier",
            kept_on(ghati(0, 50.0), ghati(2, 20.0), n, kind(JanmaTithi)),
            TithiShape::Vriddhi,
            DayClause::BothWhole,
            2,
            "on two sunrises",
        ),
        (
            "a janma tithi on its sunrise",
            kept_on(ghati(0, 54.0), ghati(1, 57.0), n, kind(JanmaTithi)),
            TithiShape::Ordinary,
            DayClause::OneDay,
            2,
            "the sunrise tithi",
        ),
        (
            "a sunrise tithi past under two muhurtas",
            kept_on(ghati(0, 54.0), ghati(1, 2.0), n, |a| {
                a.kind = JanmaTithi;
                a.janma.two_muhurta_floor = true;
            }),
            TithiShape::Ordinary,
            DayClause::TwoMuhurtaFloor,
            1,
            "taken only when asked",
        ),
        (
            "two sunrises, the birth nakshatra on the later",
            kept_on(
                ghati(0, 50.0),
                ghati(2, 20.0),
                &[
                    (Nakshatra::Mrigashira, ghati(1, 30.0)),
                    (Nakshatra::Rohini, ghati(4, 0.0)),
                ],
                |a| {
                    a.kind = JanmaTithi;
                    a.janma.nakshatra_tie = true;
                },
            ),
            TithiShape::Vriddhi,
            DayClause::BirthNakshatra,
            3,
            "taken only when asked",
        ),
        (
            "an event on no sunrise",
            kept_on(ghati(1, 5.0), ghati(1, 50.0), n, kind(Event)),
            TithiShape::Kshaya,
            DayClause::Neither,
            2,
            "a convention",
        ),
    ]);
}

/// Two exactly equal parts, the one clause a real sky never gives: C170
/// reads "equal" to the last digit, so the test walks the last bits of a
/// tithi halving both windows for the one whose shares round alike.
#[test]
fn two_equal_parts_are_named_and_keep_the_earlier_day() {
    // Kutapa to rauhina is 14 to 18 ghatis here, so 16 halves it each day.
    // Days 2 and 3 share a binade, so their windows are equally wide.
    let bits = |at: f64, ulps: i64| f64::from_bits(at.to_bits().wrapping_add_signed(ulps));
    let found = (-64..64)
        .flat_map(|early| {
            (-64..64).map(move |late| (bits(ghati(2, 16.0), early), bits(ghati(3, 16.0), late)))
        })
        .map(|(from, to)| kept_on(from, to, &[], kind(AnniversaryKind::Ekoddishta)))
        .find(|day| day.day_rule.clause == DayClause::EqualParts);
    assert!(found.is_some(), "a pair of shares rounding alike");
    let Some(day) = found else { return };
    assert_eq!(day.observance.day.day, 3);
    assert!(day.day_rule.source.contains("equally"));
}

/// An event states its window, a janma tithi may not; an event takes no
/// janma options, and neither does it take the adhika a shraddha would.
#[test]
fn an_event_states_its_window_and_keeps_the_nija() {
    assert!(AnniversaryKind::Event.keeps_the_nija());
    assert!(!AnniversaryKind::Parvana.keeps_the_nija());
    assert_eq!(AnniversaryKind::Event.window(), Window::Sunrise);
    let at_noon = kept_on(ghati(0, 54.0), ghati(1, 57.0), &[], |a| {
        a.kind = AnniversaryKind::Event;
        a.window = Some(Window::DayMuhurta { from: 8, to: 9 });
    });
    assert_eq!(at_noon.day_rule.at, Window::DayMuhurta { from: 8, to: 9 });
    assert_eq!(at_noon.kind, AnniversaryKind::Event);
    let days = days_of(&fifth(), &[], Masa::Kartika, MonthKind::Adhika);
    let (mut event, origin) = asked(Masa::Kartika, Tithi::ShuklaPanchami, AnniversaryKind::Event);
    let found = anniversaries(&[(event.clone(), origin.clone())], &days, &[]).unwrap();
    assert_eq!(
        found.anniversaries.len(),
        0,
        "an event is not kept in the adhika"
    );
    event.janma.nakshatra_tie = true;
    let refused = anniversaries(&[(event, origin)], &days, &[]).unwrap_err();
    assert_eq!(refused.field(), Some("janma"));
}

/// A caller's rule is held to its limits, each by its field
/// (`anniversary-span.md` §6), and a rule at each limit stands.
#[test]
fn a_rule_past_its_limits_is_refused_by_field() {
    use teistro_core::error::Status;
    let rule = FestivalRule {
        key: "A".repeat(FestivalRule::MOST_KEY_BYTES),
        ..FestivalRule::dharmasindhu()[0].clone()
    };
    rule.check().unwrap();
    let guard = |predicates: usize| {
        Guard::new(
            vec![
                Predicate::Wholly {
                    day: teistro_panchanga::festival::Which::Later
                };
                predicates
            ],
            teistro_panchanga::festival::Choice::Later,
        )
    };
    for (broken, field) in [
        (
            FestivalRule {
                key: "A".repeat(FestivalRule::MOST_KEY_BYTES + 1),
                ..rule.clone()
            },
            "key",
        ),
        (
            FestivalRule {
                decide: vec![guard(1); FestivalRule::MOST_GUARDS + 1],
                ..rule.clone()
            },
            "decide",
        ),
        (
            FestivalRule {
                decide: vec![guard(FestivalRule::MOST_PREDICATES + 1)],
                ..rule.clone()
            },
            "decide.when",
        ),
    ] {
        let refused = broken.check().unwrap_err();
        assert_eq!(
            (refused.status, refused.field()),
            (Status::Limit, Some(field))
        );
    }
}
