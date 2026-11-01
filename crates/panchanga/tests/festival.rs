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
use teistro_calendar::hijri::HijriOffset;
use teistro_calendar::lunisolar::MonthKind;
use teistro_core::catalogue::{Era, Masa, Nakshatra, Paksha, Rashi, Tithi, Vara};
use teistro_core::interval::Interval;
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
use teistro_core::settings::LunarMonth as Convention;
use teistro_panchanga::festival::{
    AddedDay, AnniversaryDay, AnniversaryKind, Case, Choice, Dated, DatedObservance, DatedRule,
    DayClause, DayPart, DayRule, Decided, Edge, EkadashiRule, FestivalDay, FestivalRule,
    FollowingRule, Guard, Half, InAdhika, MonthClause, MoonArcs, Moonrise, MoonriseAt,
    NakshatraBar, Observances, Occurs, OriginFound, ParanaBarred, Predicate, Published,
    PublishedStatus, Sait, SaitCandidate, SunEntry, TithiShape, Unjudged, Unplaced, VrataEdge,
    VrataParana, VrataParanaTime, Which, Window, dated, ekadashis, following, observances, yugma,
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

/// The margin is the least time between an edge of what the rule read
/// and an instant the days were decided from (`festival-rules.md` §7.5):
/// the 8th runs from day 1's ghati 40 to day 2's ghati 42, so day 2's
/// nishitha at ghati 45 is three ghatis from its end, nearer than day
/// 1's at five from its start; and a Rohini the guards name moves it to
/// her own edge when that is nearer still.
#[test]
fn an_observance_says_how_far_its_instants_may_move() {
    let tithis = [
        (Tithi::KrishnaShashthi, ghati(0, 30.0)),
        (Tithi::KrishnaSaptami, ghati(1, 40.0)),
        (Tithi::KrishnaAshtami, ghati(2, 42.0)),
        (Tithi::KrishnaNavami, ghati(3, 50.0)),
        (Tithi::KrishnaDashami, ghati(5, 0.0)),
    ];
    let seconds = |ghatis: f64| ghatis * 86_400.0 / 60.0;
    let margin = |nakshatras: &[(Nakshatra, f64)]| {
        let days = days_of(&tithis, nakshatras, Masa::Shravana, MonthKind::Nija);
        let answer = observances(&[janmashtami()], &days).unwrap();
        answer.observances[0].margin_seconds
    };
    assert!((margin(&[]) - seconds(3.0)).abs() < 1e-3, "{}", margin(&[]));
    // Rohini ends a ghati after day 1's nishitha: one ghati of room.
    let rohini = [
        (Nakshatra::Krittika, ghati(1, 10.0)),
        (Nakshatra::Rohini, ghati(1, 46.0)),
    ];
    assert!(
        (margin(&rohini) - seconds(1.0)).abs() < 1e-3,
        "{}",
        margin(&rohini)
    );
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
        month: Some(Masa::Shravana),
        convention: Convention::Amanta,
        occurs: tithi.into(),
        in_adhika: InAdhika::Nija,
        at: Window::Part {
            part: DayPart::Madhyahna,
        },
        decide,
        otherwise,
        parana: Vec::new(),
        meta: teistro_panchanga::festival::FestivalMeta::default(),
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
            .is_empty(),
        "{:?}",
        observances(&[janmashtami()], &other).unwrap().observances
    );
    let adhika = days_of(&tithis, &none, Masa::Shravana, MonthKind::Adhika);
    assert!(
        observances(&[janmashtami()], &adhika)
            .unwrap()
            .observances
            .is_empty(),
        "{:?}",
        observances(&[janmashtami()], &adhika).unwrap().observances
    );
    let mut in_adhika = janmashtami();
    in_adhika.in_adhika = InAdhika::Also;
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
        .is_empty(),
        "{:?}",
        observances(
            &[purnimanta.clone()],
            &days_of(&tithis, &none, Masa::Shravana, MonthKind::Nija)
        )
        .unwrap()
        .observances
    );
    purnimanta.month = Some(Masa::Bhadrapada);
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

#[test]
fn a_doubled_month_keeps_a_first_rule_in_its_adhika_and_an_only_rule_nowhere_else() {
    let tithis = [
        (Tithi::KrishnaSaptami, ghati(1, 40.0)),
        (Tithi::KrishnaAshtami, ghati(2, 42.0)),
        (Tithi::KrishnaDashami, ghati(5, 0.0)),
    ];
    let none: [(Nakshatra, f64); 0] = [];
    let kept = |in_adhika: InAdhika, days: &[FestivalDay]| {
        let mut rule = janmashtami();
        rule.in_adhika = in_adhika;
        let answer = observances(&[rule], days).unwrap();
        (answer.observances.len(), answer.unjudged.len())
    };
    let adhika = days_of(&tithis, &none, Masa::Shravana, MonthKind::Adhika);
    let nija = days_of(&tithis, &none, Masa::Shravana, MonthKind::Nija);
    // The month before the 8th's day, which says whether it is doubled.
    let after = |before: Masa, kind: MonthKind| {
        let mut days = nija.clone();
        days[0].month = month::of(before, Tithi::KrishnaShashthi, Convention::Amanta, kind);
        days
    };
    let doubled = after(Masa::Shravana, MonthKind::Adhika);
    let single = after(Masa::Ashadha, MonthKind::Nija);
    for (value, on_adhika, on_doubled, on_single) in [
        (InAdhika::Nija, (0, 0), (1, 0), (1, 0)),
        (InAdhika::Also, (1, 0), (1, 0), (1, 0)),
        (InAdhika::Only, (1, 0), (0, 0), (0, 0)),
        (InAdhika::First, (1, 0), (0, 0), (1, 0)),
    ] {
        assert_eq!(kept(value, &adhika), on_adhika, "{value:?} adhika");
        assert_eq!(kept(value, &doubled), on_doubled, "{value:?} doubled");
        assert_eq!(kept(value, &single), on_single, "{value:?} single");
    }
    // Days that never leave the nija month cannot say: unjudged, not guessed.
    let mut first = janmashtami();
    first.in_adhika = InAdhika::First;
    let answer = observances(&[first.clone()], &nija).unwrap();
    assert_eq!(answer.observances, []);
    assert_eq!(
        answer.unjudged[0].why,
        "the days do not reach the month next to this SHRAVANA, which says whether it is doubled"
    );
    // Purnimanta names amanta Shravana's dark half Bhadrapada; an adhika
    // Bhadrapada comes after it, before the nija Bhadrapada's bright half.
    first.convention = Convention::Purnimanta;
    first.month = Some(Masa::Bhadrapada);
    let mut days = nija.clone();
    days[4].month = month::of(
        Masa::Bhadrapada,
        Tithi::ShuklaPratipada,
        Convention::Amanta,
        MonthKind::Adhika,
    );
    assert_eq!(
        observances(&[first.clone()], &days).unwrap().observances,
        []
    );
    days[4].month = month::of(
        Masa::Bhadrapada,
        Tithi::ShuklaPratipada,
        Convention::Amanta,
        MonthKind::Nija,
    );
    assert_eq!(observances(&[first], &days).unwrap().observances.len(), 1);
}

#[test]
fn in_adhika_reads_both_its_spellings_and_writes_the_old_one_where_it_can() {
    for value in InAdhika::ALL {
        let key = serde_json::to_value(value.key()).unwrap();
        assert_eq!(serde_json::from_value::<InAdhika>(key).unwrap(), value);
        let written = serde_json::to_value(value).unwrap();
        assert_eq!(
            serde_json::from_value::<InAdhika>(written.clone()).unwrap(),
            value
        );
        let flag = matches!(value, InAdhika::Nija | InAdhika::Also);
        assert_eq!(written.is_boolean(), flag, "{value:?}");
    }
    let wrong = serde_json::from_str::<InAdhika>(r#""BOTH""#).unwrap_err();
    assert!(wrong.to_string().contains(r#"not "BOTH""#), "{wrong}");
    // A rule written before keeps its bytes.
    let rule = janmashtami();
    assert_eq!(serde_json::to_value(&rule).unwrap()["inAdhika"], false);
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
        observance.decided_by.clone(),
    )
}

/// The Devi's visarjana (DS p. 70) on a 10th that holds the earlier
/// day's morning and ends before the later sunrise: the earlier day, by
/// its second guard, Shravana joining it nowhere.
#[test]
fn devi_visarjan_keeps_a_tenth_that_never_reaches_the_later_morning() {
    let rule = shipped("DEVI_VISARJAN");
    let tithis = [
        (Tithi::ShuklaNavami, ghati(1, 5.0)),
        (Tithi::ShuklaDashami, ghati(1, 58.0)),
        (Tithi::ShuklaEkadashi, ghati(5, 0.0)),
    ];
    let mut days = days_of(&tithis, &[], Masa::Ashwina, MonthKind::Nija);
    for day in &mut days {
        day.moon_arcs = MoonArcs::Found(Vec::new());
    }
    assert_eq!(
        decided(&rule, &days),
        (Case::EarlierOnly, 1, Decided::Guard { index: 1 })
    );
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

/// The case, the day and the deciding guard when `tithi` runs from
/// `from` to `to`, in `amanta`, between neighbours of its own fortnight.
fn one(rule: &str, tithi: Tithi, from: f64, to: f64, amanta: Masa) -> (Case, u8, Decided) {
    one_of(&shipped(rule), tithi, from, to, amanta)
}

/// As [`one`], for a rule given whole.
fn one_of(
    rule: &FestivalRule,
    tithi: Tithi,
    from: f64,
    to: f64,
    amanta: Masa,
) -> (Case, u8, Decided) {
    // Neighbours no shipped rule keeps, so the rule judges one occurrence.
    let tithis = [
        (Tithi::KrishnaNavami, ghati(0, 20.0)),
        (Tithi::KrishnaDashami, from),
        (tithi, to),
        (Tithi::KrishnaNavami, ghati(5, 0.0)),
    ];
    let days = days_of(&tithis, &[], amanta, MonthKind::Nija);
    decided(rule, &days)
}

/// Nepal's reading (§9.5, C197): every rite of the night as the text has
/// it, every rite of the daylight on the day whose sunrise holds its tithi;
/// and Janai purnima 2083's shape, a full moon holding the later sunrise
/// nine ghatis, is where the two part.
#[test]
fn nepal_keeps_the_text_by_night_and_the_sunrise_tithi_by_day() {
    let (mut text, nepal) = (FestivalRule::dharmasindhu(), FestivalRule::nepal());
    // The text's rules less the three vratas at moonrise, which the
    // committee does not print, and the Navaratri's close, which
    // `NEPAL_PANCHANGA` keeps by its own print, then the monthly full-moon
    // fast no text states.
    let at_moonrise: Vec<String> = text
        .iter()
        .filter(|rule| rule.at == Window::Moonrise)
        .map(|rule| rule.key.clone())
        .collect();
    assert_eq!(
        at_moonrise,
        ["SANKASHTA_CHATURTHI", "KARAKA_CHATURTHI", "ASHUNYA_SHAYANA"]
    );
    text.retain(|rule| {
        rule.at != Window::Moonrise
            && !["MAHA_NAVAMI", "DEVI_VISARJAN"].contains(&rule.key.as_str())
    });
    assert_eq!(text.len() + 1, nepal.len());
    assert_eq!(
        nepal.last().map(|rule| rule.key.as_str()),
        Some("PURNIMA_VRATA")
    );
    for (text, nepal) in text.iter().zip(&nepal) {
        if text.at.in_daylight() {
            assert_eq!(*nepal, text.clone().udaya(), "{}", text.key);
            assert_eq!(nepal.at, Window::Sunrise);
        } else {
            assert_eq!(nepal, text, "{}", text.key);
        }
    }
    let janai = |rule: &FestivalRule| {
        one_of(
            rule,
            Tithi::Purnima,
            ghati(1, 30.0),
            ghati(2, 9.0),
            Masa::Shravana,
        )
    };
    let find = |rules: &[FestivalRule]| {
        rules
            .iter()
            .find(|rule| rule.key == "UPAKARMA_MADHYANDINA")
            .cloned()
            .unwrap()
    };
    assert_eq!(
        janai(&find(&text)),
        (Case::LaterOnly, 1, Decided::Otherwise)
    );
    assert_eq!(
        janai(&find(&nepal)),
        (Case::LaterOnly, 2, Decided::Guard { index: 0 })
    );
}

fn purnima_vrata() -> FestivalRule {
    FestivalRule::nepal()
        .into_iter()
        .find(|rule| rule.key == "PURNIMA_VRATA")
        .unwrap()
}

/// The full moon from `from` to `to` in a month named `amanta` of `kind`,
/// judged by Nepal's monthly fast: the case, the day (1-based) and the
/// month the observance reports.
fn full_moon(from: f64, to: f64, amanta: Masa, kind: MonthKind) -> (Case, u8, Masa, bool) {
    let tithis = [
        (Tithi::ShuklaTrayodashi, ghati(0, 20.0)),
        (Tithi::ShuklaChaturdashi, from),
        (Tithi::Purnima, to),
        (Tithi::KrishnaPratipada, ghati(5, 0.0)),
    ];
    let days = days_of(&tithis, &[], amanta, kind);
    let answer = observances(&[purnima_vrata()], &days).unwrap();
    let [observance] = answer.observances.as_slice() else {
        panic!("one full moon, not {:?}", answer.observances);
    };
    (
        observance.case,
        observance.day.day - 1,
        observance.month,
        observance.adhika,
    )
}

/// Nepal's monthly full-moon fast (§9.6, C200), on each shape the
/// committee's 24 printed rows take: a sunset is ghati 30 of its day.
#[test]
fn the_full_moon_fast_keeps_the_evening_the_full_moon_holds_the_later_first() {
    let shravana = |from, to| full_moon(from, to, Masa::Shravana, MonthKind::Nija);
    // Shravana 2082: begun before the earlier sunset, ended before the later.
    assert_eq!(
        shravana(ghati(1, 20.0), ghati(2, 18.0)),
        (Case::EarlierOnly, 1, Masa::Shravana, false)
    );
    // Vaishakha 2082: begun after the earlier sunset, past the later.
    assert_eq!(
        shravana(ghati(1, 33.0), ghati(2, 35.0)),
        (Case::LaterOnly, 2, Masa::Shravana, false)
    );
    // Ashadha 2083: both sunsets, the later.
    assert_eq!(
        shravana(ghati(1, 28.0), ghati(2, 33.0)),
        (Case::Both, 2, Masa::Shravana, false)
    );
    // Pausha 2082: begun after the earlier sunset, ended before the later;
    // neither, so the later, though it touched the earlier's pradosha.
    assert_eq!(
        shravana(ghati(1, 33.0), ghati(2, 28.0)),
        (Case::Neither, 2, Masa::Shravana, false)
    );
    // Margashirsha 2082: kshaya, inside the earlier day, which it keeps.
    assert_eq!(
        shravana(ghati(1, 3.0), ghati(1, 58.0)),
        (Case::EarlierOnly, 1, Masa::Shravana, false)
    );
}

/// The fast is kept every month, the adhika month too, as the committee
/// prints it in VS 2083's adhika Jyeshtha, and the observance says which.
#[test]
fn the_full_moon_fast_is_kept_every_month_and_says_which() {
    assert_eq!(
        full_moon(
            ghati(1, 20.0),
            ghati(2, 18.0),
            Masa::Kartika,
            MonthKind::Nija
        ),
        (Case::EarlierOnly, 1, Masa::Kartika, false)
    );
    assert_eq!(
        full_moon(
            ghati(1, 20.0),
            ghati(2, 18.0),
            Masa::Jyeshtha,
            MonthKind::Adhika
        ),
        (Case::EarlierOnly, 1, Masa::Jyeshtha, true)
    );
    // The window is an instant of the evening, so Nepal keeps the rule as
    // it stands rather than reading it at sunrise.
    assert!(!Window::Sunset.in_daylight());
    // A rule naming its month still keeps only that one.
    let mut kartika_only = purnima_vrata();
    kartika_only.month = Some(Masa::Kartika);
    let tithis = [
        (Tithi::ShuklaChaturdashi, ghati(1, 20.0)),
        (Tithi::Purnima, ghati(2, 18.0)),
        (Tithi::KrishnaPratipada, ghati(5, 0.0)),
    ];
    let days = days_of(&tithis, &[], Masa::Shravana, MonthKind::Nija);
    let kept = observances(&[kartika_only], &days).unwrap().observances;
    assert!(kept.is_empty(), "{kept:?}");
}

/// A rule spelt out without a month is kept every month, and one written
/// back out leaves the month out rather than writing a null.
#[test]
fn a_rule_without_a_month_reads_and_writes_without_one() {
    let rule = purnima_vrata();
    let written = serde_json::to_value(&rule).unwrap();
    assert!(written.get("month").is_none(), "{written}");
    assert_eq!(
        serde_json::from_value::<FestivalRule>(written).unwrap(),
        rule
    );
    let sunset = serde_json::to_value(Window::Sunset).unwrap();
    assert_eq!(sunset, serde_json::json!({ "window": "SUNSET" }));
}

fn samavedi() -> FestivalRule {
    FestivalRule::dharmasindhu()
        .into_iter()
        .find(|rule| rule.key == "UPAKARMA_SAMAVEDI")
        .unwrap()
}

/// Hasta from `from` to `to` in the bright half of a month named `amanta`,
/// judged by `rule`: the observances' (case, day) pairs, 1-based.
fn hasta(rule: &FestivalRule, from: f64, to: f64, amanta: Masa) -> Vec<(Case, u8)> {
    let tithis = [
        (Tithi::Amavasya, ghati(0, 10.0)),
        (Tithi::ShuklaPratipada, ghati(1, 30.0)),
        (Tithi::ShuklaDvitiya, ghati(2, 40.0)),
        (Tithi::ShuklaTritiya, ghati(4, 0.0)),
        (Tithi::ShuklaChaturthi, ghati(5, 0.0)),
    ];
    let nakshatras = [
        (Nakshatra::UttaraPhalguni, from),
        (Nakshatra::Hasta, to),
        (Nakshatra::Chitra, ghati(5, 0.0)),
    ];
    let days = days_of(&tithis, &nakshatras, amanta, MonthKind::Nija);
    let answer = observances(std::slice::from_ref(rule), &days).unwrap();
    assert!(answer.unjudged.is_empty(), "{:?}", answer.unjudged);
    answer
        .observances
        .iter()
        .map(|observance| {
            assert_eq!(observance.tithi, Interval::literal(from, to));
            (observance.case, observance.day.day)
        })
        .collect()
}

/// The Samavedis' upakarma (p. 47, read off the page image): the earlier
/// day only when Hasta pervades its aparahna wholly and the later's not at
/// all. Aparahna is ghatis 18 to 24 of a thirty-ghati daylight here.
#[test]
fn the_samavedi_upakarma_takes_the_earlier_day_only_when_it_alone_is_wholly_pervaded() {
    let rule = samavedi();
    let at = |from, to| hasta(&rule, ghati(1, from), ghati(2, to), Masa::Bhadrapada);
    // The earlier's aparahna wholly, the later's not at all.
    assert_eq!(at(10.0, 10.0), [(Case::EarlierOnly, 2)]);
    // The earlier's touched in part only: the later.
    assert_eq!(at(20.0, 10.0), [(Case::EarlierOnly, 3)]);
    // Both wholly, the later in part, or neither: the later.
    assert_eq!(at(10.0, 30.0), [(Case::Both, 3)]);
    assert_eq!(at(10.0, 20.0), [(Case::UnequalParts, 3)]);
    assert_eq!(at(30.0, 15.0), [(Case::Neither, 3)]);
    assert_eq!(at(30.0, 30.0), [(Case::LaterOnly, 3)]);
}

/// A nakshatra-keyed rule keeps its month and its paksha, read from the
/// tithi running at the nakshatra's middle.
#[test]
fn a_rule_kept_on_a_nakshatra_keeps_its_month_and_its_paksha() {
    let rule = samavedi();
    let shravana = hasta(&rule, ghati(1, 10.0), ghati(2, 10.0), Masa::Shravana);
    assert!(shravana.is_empty(), "{shravana:?}");
    let dark = FestivalRule {
        occurs: Occurs::Nakshatra {
            nakshatra: Nakshatra::Hasta,
            paksha: Paksha::Krishna,
        },
        ..rule.clone()
    };
    let bright = hasta(&dark, ghati(1, 10.0), ghati(2, 10.0), Masa::Bhadrapada);
    assert!(bright.is_empty(), "{bright:?}");
    // Through the pratipada into the dvitiya, the middle in the dvitiya: kept.
    assert_eq!(
        hasta(&rule, ghati(1, 25.0), ghati(2, 50.0), Masa::Bhadrapada).len(),
        1
    );
}

#[test]
fn a_rule_kept_on_a_nakshatra_reads_and_writes_flat_and_is_refused_by_name() {
    let rule = samavedi();
    let written = serde_json::to_value(&rule).unwrap();
    assert_eq!(
        (
            &written["nakshatra"],
            &written["paksha"],
            written.get("tithi")
        ),
        (
            &serde_json::json!("HASTA"),
            &serde_json::json!("SHUKLA"),
            None
        )
    );
    assert_eq!(
        serde_json::from_value::<FestivalRule>(written.clone()).unwrap(),
        rule
    );
    let tithi = serde_json::to_value(purnima_vrata()).unwrap();
    assert_eq!(tithi["tithi"], serde_json::json!("PURNIMA"));
    let refused = |edit: &dyn Fn(&mut serde_json::Value)| {
        let mut value = written.clone();
        edit(&mut value);
        serde_json::from_value::<FestivalRule>(value)
            .unwrap_err()
            .to_string()
    };
    assert!(refused(&|v| v["tithi"] = serde_json::json!("PURNIMA")).contains("not both"));
    assert!(refused(&|v| drop(v.as_object_mut().unwrap().remove("paksha"))).contains("`paksha`"));
    assert!(
        refused(&|v| drop(v.as_object_mut().unwrap().remove("nakshatra"))).contains("on a `tithi`")
    );
    let yugma = FestivalRule {
        otherwise: Choice::ByYugma,
        ..rule.clone()
    };
    assert_eq!(yugma.check().unwrap_err().field(), Some("decide"));
    let joined = FestivalRule {
        decide: vec![Guard::new(
            [Predicate::Joined {
                day: Which::Later,
                nakshatra: Nakshatra::Shravana,
                at: None,
                portion: None,
            }],
            Choice::Later,
        )],
        ..rule
    };
    assert_eq!(joined.check().unwrap_err().field(), Some("decide.when.is"));
}

/// Haritalika (p. 55): the later day whenever its sunrise holds the 3rd,
/// however briefly and however long the earlier held it; the earlier only
/// when the 3rd is kshaya.
#[test]
fn haritalika_takes_the_later_sunrise_and_the_earlier_only_when_kshaya() {
    let third = |from, to| {
        one(
            "HARITALIKA",
            Tithi::ShuklaTritiya,
            from,
            to,
            Masa::Bhadrapada,
        )
    };
    // A ghati into the later day.
    assert_eq!(
        third(ghati(1, 10.0), ghati(2, 1.0)),
        (Case::LaterOnly, 2, Decided::Otherwise)
    );
    // Sixty ghatis of the earlier day and half a ghati of the later.
    assert_eq!(
        third(ghati(0, 59.0), ghati(2, 0.5)),
        (Case::Both, 2, Decided::Otherwise)
    );
    // Kshaya: begun after one sunrise and ended before the next.
    assert_eq!(
        third(ghati(1, 5.0), ghati(1, 55.0)),
        (Case::Neither, 1, Decided::Guard { index: 0 })
    );
}

/// Navaratra arambha (p. 65): the later day when its 1st lasts a muhurta
/// past sunrise, else the day the new moon joins it; the earlier too when
/// the 1st holds that whole day and grows into the next.
#[test]
fn navaratra_arambha_needs_a_muhurta_past_the_later_sunrise() {
    let first = |from, to| {
        one(
            "NAVARATRA_ARAMBHA",
            Tithi::ShuklaPratipada,
            from,
            to,
            Masa::Ashwina,
        )
    };
    // A daylight muhurta is two ghatis: three past sunrise is enough.
    assert_eq!(
        first(ghati(1, 30.0), ghati(2, 3.0)),
        (Case::LaterOnly, 2, Decided::Guard { index: 1 })
    );
    // A ghati and a half is not: the day the new moon joins it.
    assert_eq!(
        first(ghati(1, 30.0), ghati(2, 1.5)),
        (Case::LaterOnly, 1, Decided::Otherwise)
    );
    // Whole on the earlier day and growing: the earlier.
    assert_eq!(
        first(ghati(0, 58.0), ghati(2, 4.0)),
        (Case::Both, 1, Decided::Guard { index: 0 })
    );
    // Kshaya: no sunrise holds it, so the day it falls in.
    assert_eq!(
        first(ghati(1, 5.0), ghati(1, 50.0)),
        (Case::Neither, 1, Decided::Otherwise)
    );
}

/// Yama dwitiya (p. 79): aparahna, ghatis 18 to 24; the earlier day only
/// when it alone holds it.
#[test]
fn yama_dwitiya_takes_the_earlier_day_only_when_it_alone_holds_aparahna() {
    let second = |from, to| {
        one(
            "YAMA_DWITIYA",
            Tithi::ShuklaDvitiya,
            from,
            to,
            Masa::Kartika,
        )
    };
    assert_eq!(
        second(ghati(1, 10.0), ghati(2, 15.0)),
        (Case::EarlierOnly, 1, Decided::Guard { index: 0 })
    );
    assert_eq!(
        second(ghati(1, 10.0), ghati(2, 25.0)),
        (Case::Both, 2, Decided::Otherwise)
    );
    assert_eq!(
        second(ghati(1, 26.0), ghati(2, 20.0)),
        (Case::LaterOnly, 2, Decided::Otherwise)
    );
    assert_eq!(
        second(ghati(1, 20.0), ghati(2, 22.0)),
        (Case::UnequalParts, 2, Decided::Otherwise)
    );
}

/// Shivaratri (p. 90): niśītha is the night's eighth muhurta, ghatis 44
/// to 46 of the synthetic day; each of the page's clauses in turn.
#[test]
fn shivaratri_takes_the_book_s_day_in_each_of_its_clauses() {
    let fourteenth = |from, to| {
        one(
            "SHIVARATRI",
            Tithi::KrishnaChaturdashi,
            from,
            to,
            Masa::Magha,
        )
    };
    // The earlier day only.
    assert_eq!(
        fourteenth(ghati(1, 40.0), ghati(2, 30.0)),
        (Case::EarlierOnly, 1, Decided::Guard { index: 0 })
    );
    // The later day only.
    assert_eq!(
        fourteenth(ghati(1, 50.0), ghati(2, 50.0)),
        (Case::LaterOnly, 2, Decided::Otherwise)
    );
    // Whole on the earlier, part on the later: the earlier.
    assert_eq!(
        fourteenth(ghati(1, 40.0), ghati(2, 45.0)),
        (Case::UnequalParts, 1, Decided::Guard { index: 1 })
    );
    // Part on the earlier, whole on the later: the later.
    assert_eq!(
        fourteenth(ghati(1, 45.0), ghati(2, 50.0)),
        (Case::UnequalParts, 2, Decided::Otherwise)
    );
    // Whole on both: the later, with the many (C195).
    assert_eq!(
        fourteenth(ghati(1, 40.0), ghati(2, 47.0)),
        (Case::Both, 2, Decided::Otherwise)
    );
    // Neither: the later.
    assert_eq!(
        fourteenth(ghati(1, 47.0), ghati(2, 43.0)),
        (Case::Neither, 2, Decided::Otherwise)
    );
}

/// Rakshabandhan (p. 49): the later day when the full moon holds its
/// sunrise more than three muhurtas, six ghatis; else the earlier.
#[test]
fn rakshabandhan_takes_the_later_day_past_three_muhurtas_of_its_sunrise() {
    let full = |to| {
        one(
            "RAKSHABANDHAN",
            Tithi::Purnima,
            ghati(1, 30.0),
            to,
            Masa::Shravana,
        )
    };
    assert_eq!(
        full(ghati(2, 7.0)),
        (Case::LaterOnly, 2, Decided::Guard { index: 0 })
    );
    assert_eq!(
        full(ghati(2, 5.0)),
        (Case::LaterOnly, 1, Decided::Otherwise)
    );
}

/// Bali pratipada (p. 78): the later day when its 1st holds nine
/// muhurtas, eighteen ghatis, past sunrise; else the day the new moon
/// pierces.
#[test]
fn bali_pratipada_needs_nine_muhurtas_past_the_later_sunrise() {
    let first = |to| {
        one(
            "BALI_PRATIPADA",
            Tithi::ShuklaPratipada,
            ghati(1, 30.0),
            to,
            Masa::Kartika,
        )
    };
    assert_eq!(
        first(ghati(2, 19.0)),
        (Case::LaterOnly, 2, Decided::Guard { index: 0 })
    );
    assert_eq!(
        first(ghati(2, 17.0)),
        (Case::LaterOnly, 1, Decided::Otherwise)
    );
}

/// The Yajurvedis' upakarma (p. 47): over the same full moons, the
/// Madhyandina wants six muhurtas (twelve ghatis) past the later sunrise,
/// the Taittiriya two (four), and both take the earlier day when the full
/// moon holds both sunrises.
#[test]
fn upakarma_parts_the_yajurvedis_on_how_long_the_later_day_holds_the_full_moon() {
    let full = |key, from, to| one(key, Tithi::Purnima, from, to, Masa::Shravana);
    let both = |from, to| {
        (
            full("UPAKARMA_MADHYANDINA", from, to),
            full("UPAKARMA_TAITTIRIYA", from, to),
        )
    };
    let later = (Case::LaterOnly, 2, Decided::Guard { index: 1 });
    let earlier = (Case::LaterOnly, 1, Decided::Otherwise);
    // More than six muhurtas: the later for both.
    assert_eq!(
        both(ghati(1, 30.0), ghati(2, 13.0)),
        (later.clone(), later.clone())
    );
    // Between two and six: the Taittiriya's later, the Madhyandina's earlier.
    assert_eq!(
        both(ghati(1, 30.0), ghati(2, 8.0)),
        (earlier.clone(), later)
    );
    // Fewer than two: the earlier for both.
    assert_eq!(
        both(ghati(1, 30.0), ghati(2, 3.0)),
        (earlier.clone(), earlier)
    );
    // Holding both sunrises: the earlier for every Yajurvedi.
    let grown = (Case::Both, 1, Decided::Guard { index: 0 });
    assert_eq!(both(ghati(0, 58.0), ghati(2, 14.0)), (grown.clone(), grown));
}

/// Holika (p. 94): pradosha is ghatis 30 to 36 of the synthetic day; the
/// later day whenever its pradosha holds any of the full moon.
#[test]
fn holika_takes_the_later_day_whenever_its_pradosha_holds_the_full_moon() {
    let full = |from, to| one("HOLIKA", Tithi::Purnima, from, to, Masa::Phalguna);
    assert_eq!(
        full(ghati(1, 29.0), ghati(2, 37.0)),
        (Case::Both, 2, Decided::Otherwise)
    );
    assert_eq!(
        full(ghati(1, 32.0), ghati(2, 40.0)),
        (Case::UnequalParts, 2, Decided::Otherwise)
    );
    assert_eq!(
        full(ghati(1, 25.0), ghati(2, 28.0)),
        (Case::EarlierOnly, 1, Decided::Guard { index: 0 })
    );
    assert_eq!(
        full(ghati(1, 37.0), ghati(2, 29.0)),
        (Case::Neither, 1, Decided::Guard { index: 1 })
    );
}

/// Holi in the hills keeps the Holika day and the Terai the day after,
/// each carrying the facts of the Holika it counts from.
#[test]
fn a_following_rule_counts_days_from_the_observance_it_follows() {
    let tithis = [
        (Tithi::ShuklaChaturdashi, ghati(1, 25.0)),
        (Tithi::Purnima, ghati(2, 28.0)),
        (Tithi::KrishnaDvitiya, ghati(5, 0.0)),
    ];
    let days = days_of(&tithis, &[], Masa::Phalguna, MonthKind::Nija);
    let found = observances(&[shipped("HOLIKA")], &days).unwrap();
    let holika = &found.observances[0];
    assert_eq!(holika.day.day - 1, 1);
    let counted = following(&FollowingRule::nepal(), &found.observances, &days).unwrap();
    let days_of_each: Vec<(&str, u8)> = counted
        .observances
        .iter()
        .map(|o| (o.rule.as_str(), o.day.day - 1))
        .collect();
    assert_eq!(days_of_each, [("HOLI_HILLS", 1), ("HOLI_TERAI", 2)]);
    let terai = &counted.observances[1];
    assert_eq!(
        terai.decided_by,
        Decided::After {
            rule: "HOLIKA".to_owned(),
            days: 1
        }
    );
    assert_eq!((terai.tithi, terai.case), (holika.tithi, holika.case));
    // Past the days given: unjudged, naming why.
    let mut far = FollowingRule::nepal().remove(1);
    far.days = 9;
    let counted = following(&[far], &found.observances, &days).unwrap();
    assert!(counted.observances.is_empty(), "{:?}", counted.observances);
    assert_eq!(counted.unjudged[0].rule, "HOLI_TERAI");
}

/// Bhumiraj on the civil day the Sun enters Ardra and the Bhumi puja
/// three days on; `on` names the entry's day. A day whose entry was not
/// read is refused, never taken for a day the Sun entered none.
#[test]
fn a_rule_on_the_suns_entry_keeps_that_day_or_counts_from_it() {
    let mut days = days_of(
        &[(Tithi::Purnima, 9.0)],
        &[],
        Masa::Jyeshtha,
        MonthKind::Nija,
    );
    for (k, day) in days.iter_mut().enumerate() {
        day.sun_entry = match k {
            1 => SunEntry::Entered(Nakshatra::Ardra),
            _ => SunEntry::Stayed,
        };
    }
    let shipped: Vec<DatedRule> = DatedRule::nepal_panchanga()
        .into_iter()
        .filter(DatedRule::reads_sun_entry)
        .collect();
    let kept: Vec<(&str, u8, u8)> = Vec::new();
    let found = dated(&shipped, &days, HijriOffset::NONE).unwrap();
    let kept = found.dated.iter().fold(kept, |mut kept, one| {
        kept.push((one.rule.as_str(), one.day.day - 1, one.on.day - 1));
        kept
    });
    assert_eq!(kept, [("BHUMIRAJ", 1, 1), ("BHUMI_PUJA", 4, 1)]);
    // Another nakshatra's entry keeps nothing.
    days[1].sun_entry = SunEntry::Entered(Nakshatra::Punarvasu);
    assert_eq!(dated(&shipped, &days, HijriOffset::NONE).unwrap().dated, []);
    days[1].sun_entry = SunEntry::Unsearched;
    let refused = dated(&shipped, &days, HijriOffset::NONE).unwrap_err();
    assert!(refused.message.contains("sun_entries"), "{refused:?}");
    // A count is checked before any day is read.
    let far = DatedRule {
        key: "FAR".to_owned(),
        source: "a test".to_owned(),
        on: Dated::SunEnters {
            nakshatra: Nakshatra::Ardra,
            days: Dated::MOST_DAYS + 1,
        },
        meta: teistro_panchanga::festival::FestivalMeta::default(),
    };
    assert_eq!(far.check().unwrap_err().field(), Some("sunEnters.days"));
    assert_eq!((shipped[0].reach(), shipped[1].reach()), (0, 3));
    let spelt: DatedRule = serde_json::from_str(
        r#"{"key": "BHUMIRAJ", "source": "mine", "sunEnters": {"nakshatra": "ARDRA"}}"#,
    )
    .unwrap();
    assert_eq!(spelt.on, shipped[0].on);
    let text = serde_json::to_string(&shipped[1]).unwrap();
    assert!(
        text.contains(r#""sunEnters":{"nakshatra":"ARDRA","days":3}"#),
        "{text}"
    );
}

#[test]
fn a_following_rule_that_cannot_be_judged_is_refused_by_its_field() {
    let terai = FollowingRule::nepal().remove(1);
    for (rule, field) in [
        (
            FollowingRule {
                key: " ".to_owned(),
                ..terai.clone()
            },
            "key",
        ),
        (
            FollowingRule {
                after: "HOLI_TERAI".to_owned(),
                ..terai.clone()
            },
            "after",
        ),
        (
            FollowingRule {
                after: String::new(),
                ..terai.clone()
            },
            "after",
        ),
        (
            FollowingRule {
                days: FollowingRule::MOST_DAYS + 1,
                ..terai.clone()
            },
            "days",
        ),
    ] {
        assert_eq!(rule.check().unwrap_err().field(), Some(field), "{rule:?}");
    }
    for rule in FollowingRule::nepal() {
        rule.check().unwrap();
    }
}

#[test]
fn a_night_muhurta_outside_the_night_s_fifteen_is_refused() {
    for muhurta in [0, 16] {
        let mut rule = shipped("SHIVARATRI");
        rule.at = Window::NightMuhurta { muhurta };
        assert_eq!(rule.check().unwrap_err().field(), Some("at.muhurta"));
        let mut rule = shipped("SHIVARATRI");
        rule.decide.push(Guard::new(
            [Predicate::Stands {
                day: Which::Later,
                nakshatra: Nakshatra::Shravana,
                at: Window::NightMuhurta { muhurta },
            }],
            Choice::Later,
        ));
        assert_eq!(
            rule.check().unwrap_err().field(),
            Some("decide.when.at.muhurta")
        );
    }
}

/// The maintainer's rule over a window (C601, C603): the later day only
/// when it alone holds the window or holds more of it, in time; the
/// earlier on a tie, both wholly being one, and when neither holds it.
fn greater_coverage(at: Window) -> FestivalRule {
    FestivalRule {
        at,
        ..a_rule(
            Tithi::KrishnaEkadashi,
            vec![
                Guard::new(
                    [Predicate::Case {
                        case: Case::LaterOnly,
                    }],
                    Choice::Later,
                ),
                Guard::new([Predicate::Case { case: Case::Both }], Choice::Earlier),
                Guard::new([Predicate::Greater { day: Which::Later }], Choice::Later),
            ],
            Choice::Earlier,
        )
    }
}

/// Purvahna as six day muhurtas, ghatis 0 to 12 of the synthetic day (a
/// day muhurta is two of its ghatis): each clause of the rule in turn,
/// and the deciding guard.
#[test]
fn greater_coverage_of_six_day_muhurtas_takes_the_day_holding_more_time() {
    let rule = greater_coverage(Window::DayMuhurta { from: 1, to: 6 });
    rule.check().unwrap();
    let eleventh = |from, to| one_of(&rule, Tithi::KrishnaEkadashi, from, to, Masa::Shravana);
    // Four ghatis of the earlier window, six of the later.
    assert_eq!(
        eleventh(ghati(1, 8.0), ghati(2, 6.0)),
        (Case::UnequalParts, 2, Decided::Guard { index: 2 })
    );
    // Eight against six.
    assert_eq!(
        eleventh(ghati(1, 4.0), ghati(2, 6.0)),
        (Case::UnequalParts, 1, Decided::Otherwise)
    );
    // A quarter day and a quarter day of the whole daylight, instants
    // a binary fraction can hold exactly: a tie, the earlier.
    let whole = greater_coverage(Window::DayMuhurta { from: 1, to: 15 });
    assert_eq!(
        one_of(&whole, Tithi::KrishnaEkadashi, 1.25, 2.25, Masa::Shravana),
        (Case::EqualParts, 1, Decided::Otherwise)
    );
    // Begun after the earlier window and ended before the later: neither.
    assert_eq!(
        eleventh(ghati(1, 14.0), ghati(1, 50.0)),
        (Case::Neither, 1, Decided::Otherwise)
    );
    // Ended past the seventh muhurta's start: the later holds the window
    // wholly and the earlier in part.
    assert_eq!(
        eleventh(ghati(1, 2.0), ghati(2, 13.0)),
        (Case::UnequalParts, 2, Decided::Guard { index: 2 })
    );
}

/// `GREATER` weighs time, not the share of each window: a longer later
/// daylight holding less of its own window can hold more of the tithi.
/// And it does not see two whole windows as a tie, which is why the rule
/// puts `BOTH` before it.
#[test]
fn greater_weighs_time_and_the_rule_ties_two_whole_windows() {
    let whole = Window::DayMuhurta { from: 1, to: 15 };
    let rule = greater_coverage(whole);
    let tithis = |from: f64, to: f64| {
        let mut days = days_of(
            &[
                (Tithi::KrishnaNavami, ghati(0, 20.0)),
                (Tithi::KrishnaDashami, from),
                (Tithi::KrishnaEkadashi, to),
                (Tithi::KrishnaNavami, ghati(5, 0.0)),
            ],
            &[],
            Masa::Shravana,
            MonthKind::Nija,
        );
        // The later day's daylight is 36 ghatis, the earlier's 30.
        days[2].sunset = teistro_core::quantity::JulianDay::literal(ghati(2, 36.0));
        days
    };
    // 12 of the earlier's 30 ghatis (0.40), 13 of the later's 36 (0.36).
    let days = tithis(ghati(1, 18.0), ghati(2, 13.0));
    let observance = &observances(std::slice::from_ref(&rule), &days)
        .unwrap()
        .observances[0];
    assert!(observance.extents[0].held > observance.extents[1].held);
    assert_eq!(
        (observance.day.day - 1, observance.decided_by.clone()),
        (2, Decided::Guard { index: 2 })
    );
    // Both daylights wholly: the rule's tie, though `GREATER` alone would
    // give the later, longer one.
    let days = tithis(ghati(0, 40.0), ghati(2, 40.0));
    assert_eq!(
        decided(&rule, &days),
        (Case::Both, 1, Decided::Guard { index: 1 })
    );
    let greater_alone = FestivalRule {
        decide: vec![Guard::new(
            [Predicate::Greater { day: Which::Later }],
            Choice::Later,
        )],
        ..rule
    };
    assert_eq!(
        decided(&greater_alone, &days),
        (Case::Both, 2, Decided::Guard { index: 0 })
    );
}

/// A rule weighing two times stands while the place moves by half their
/// difference: four ghatis against six stand a ghati, closer than any
/// edge of the tithi to an instant of the windows (four ghatis).
#[test]
fn a_rule_weighing_two_times_stands_by_half_their_difference() {
    let rule = greater_coverage(Window::DayMuhurta { from: 1, to: 6 });
    let days = days_of(
        &[
            (Tithi::KrishnaNavami, ghati(0, 20.0)),
            (Tithi::KrishnaDashami, ghati(1, 8.0)),
            (Tithi::KrishnaEkadashi, ghati(2, 6.0)),
            (Tithi::KrishnaNavami, ghati(5, 0.0)),
        ],
        &[],
        Masa::Shravana,
        MonthKind::Nija,
    );
    let observance = &observances(std::slice::from_ref(&rule), &days)
        .unwrap()
        .observances[0];
    let ghati_seconds = 86_400.0 / 60.0;
    assert!(
        (observance.margin_seconds - ghati_seconds).abs() < 1e-3,
        "{}",
        observance.margin_seconds
    );
}

#[test]
fn day_muhurtas_outside_the_daylight_s_fifteen_or_backwards_are_refused() {
    for (from, to, field) in [(0, 6, "from"), (1, 16, "to"), (7, 6, "to")] {
        let rule = greater_coverage(Window::DayMuhurta { from, to });
        assert_eq!(
            rule.check().unwrap_err().field(),
            Some(format!("at.{field}").as_str())
        );
        let mut rule = greater_coverage(Window::Sunrise);
        rule.decide.push(Guard::new(
            [Predicate::Stands {
                day: Which::Later,
                nakshatra: Nakshatra::Shravana,
                at: Window::DayMuhurta { from, to },
            }],
            Choice::Later,
        ));
        assert_eq!(
            rule.check().unwrap_err().field(),
            Some(format!("decide.when.at.{field}").as_str())
        );
    }
}

#[test]
fn day_muhurtas_and_greater_read_and_write_as_the_language_spells_them() {
    let rule = greater_coverage(Window::DayMuhurta { from: 1, to: 6 });
    let text = serde_json::to_string(&rule).unwrap();
    assert!(
        text.contains(r#""at":{"window":"DAY_MUHURTA","from":1,"to":6}"#),
        "{text}"
    );
    assert!(text.contains(r#"{"is":"GREATER","day":"LATER"}"#), "{text}");
    let back: FestivalRule = serde_json::from_str(&text).unwrap();
    assert_eq!(back, rule);
    for rule in FestivalRule::nepal_panchanga() {
        rule.check().unwrap();
    }
}

/// A rule on the udaya day, kept only when that day is `vara`, and not
/// at all otherwise: the fit the committee's weekday fasts take (C618).
fn on_vara(occurs: Occurs, vara: Vara) -> FestivalRule {
    let on = |day| Predicate::OnVara { day, vara };
    FestivalRule {
        month: None,
        occurs,
        in_adhika: InAdhika::Also,
        at: Window::Sunrise,
        decide: vec![
            Guard::new(
                [
                    Predicate::Case {
                        case: Case::LaterOnly,
                    },
                    on(Which::Later),
                ],
                Choice::Later,
            ),
            Guard::new(
                [Predicate::Case {
                    case: Case::LaterOnly,
                }],
                Choice::None,
            ),
            Guard::new([on(Which::Earlier)], Choice::Earlier),
        ],
        otherwise: Choice::None,
        ..a_rule(Tithi::KrishnaEkadashi, vec![], Choice::Earlier)
    }
}

/// The 11th holds only the sunrise of day 2, 3 January 2024, a
/// Wednesday: kept there on a Wednesday rule, and on a Monday rule
/// neither kept nor left unjudged.
#[test]
fn on_vara_keeps_the_udaya_day_only_on_its_vara() {
    let eleventh: Occurs = Tithi::KrishnaEkadashi.into();
    let days = days_of(
        &[
            (Tithi::KrishnaNavami, ghati(0, 20.0)),
            (Tithi::KrishnaDashami, ghati(1, 40.0)),
            (Tithi::KrishnaEkadashi, ghati(2, 30.0)),
            (Tithi::KrishnaNavami, ghati(5, 0.0)),
        ],
        &[],
        Masa::Shravana,
        MonthKind::Nija,
    );
    assert_eq!(days[2].vara, Vara::Budhavara);
    assert_eq!(
        decided(&on_vara(eleventh, Vara::Budhavara), &days),
        (Case::LaterOnly, 2, Decided::Guard { index: 0 })
    );
    let monday = observances(&[on_vara(eleventh, Vara::Somavara)], &days).unwrap();
    assert!(monday.observances.is_empty(), "{:?}", monday.observances);
    assert!(monday.unjudged.is_empty(), "{:?}", monday.unjudged);
}

/// Thirty tithis of a day each, every one holding one sunrise: the 13th
/// of both pakshas is kept, and nothing else.
#[test]
fn a_day_of_either_paksha_is_kept_in_both() {
    let tithis: Vec<(Tithi, f64)> = Tithi::ALL
        .iter()
        .enumerate()
        .map(|(k, &tithi)| (tithi, ghati(k + 1, 20.0)))
        .collect();
    let days = days_over(33, &tithis, &[], Masa::Shravana, MonthKind::Nija);
    let rule = FestivalRule {
        month: None,
        occurs: Occurs::DayOfPaksha { day_of_paksha: 13 },
        at: Window::Sunrise,
        ..a_rule(
            Tithi::KrishnaEkadashi,
            vec![Guard::new(
                [Predicate::Case {
                    case: Case::LaterOnly,
                }],
                Choice::Later,
            )],
            Choice::Earlier,
        )
    };
    rule.check().unwrap();
    let kept = observances(std::slice::from_ref(&rule), &days).unwrap();
    assert!(kept.unjudged.is_empty(), "{:?}", kept.unjudged);
    // The bright 13th runs from day 13's ghati 20 to day 14's and holds
    // day 14's sunrise; the dark 13th day 29's.
    let given: Vec<u8> = kept.observances.iter().map(|o| o.day.day - 1).collect();
    assert_eq!(given, [13, 28]);
}

#[test]
fn a_day_of_either_paksha_and_a_weekday_read_and_write_as_the_language_spells_them() {
    let rule = on_vara(Occurs::DayOfPaksha { day_of_paksha: 13 }, Vara::Shanivara);
    let text = serde_json::to_string(&rule).unwrap();
    assert!(text.contains(r#""dayOfPaksha":13"#), "{text}");
    assert!(
        text.contains(r#"{"is":"ON_VARA","day":"LATER","vara":"SHANIVARA"}"#),
        "{text}"
    );
    assert!(text.contains(r#""otherwise":"NONE""#), "{text}");
    let back: FestivalRule = serde_json::from_str(&text).unwrap();
    assert_eq!(back, rule);
    for (wrong, says) in [
        (r#""dayOfPaksha":15"#, "PURNIMA"),
        (r#""dayOfPaksha":0"#, "1 to 14"),
        (r#""dayOfPaksha":13,"tithi":"PURNIMA""#, "both pakshas"),
    ] {
        let text = text.replace(r#""dayOfPaksha":13"#, wrong);
        let why = serde_json::from_str::<FestivalRule>(&text)
            .unwrap_err()
            .to_string();
        assert!(why.contains(says), "{wrong}: {why}");
    }
}

#[test]
fn the_shipped_rules_round_trip_through_their_json_record() {
    let rules = FestivalRule::dharmasindhu();
    let text = serde_json::to_string(&rules).unwrap();
    let back: Vec<FestivalRule> = serde_json::from_str(&text).unwrap();
    assert_eq!(back, rules);
    assert!(text.contains(r#""window":"NISHITHA""#), "{text}");
    assert!(text.contains(r#""is":"JOINED""#), "{text}");
    assert!(
        text.contains(r#""window":"NIGHT_MUHURTA","muhurta":8"#),
        "{text}"
    );
    assert!(text.contains(r#""is":"WHOLLY","day":"EARLIER""#), "{text}");
}

/// A rite at moonrise (§3.1): the 4th at each day's moonrise, the earlier
/// day when both moonrises hold it (*Dharmasindhu* p. 9), and a day the
/// Moon does not rise on leaves the occurrence unjudged rather than read
/// as holding nothing.
#[test]
fn a_rite_at_moonrise_reads_each_days_first_moonrise() {
    for (key, month) in [
        ("SANKASHTA_CHATURTHI", Masa::Magha),
        ("KARAKA_CHATURTHI", Masa::Ashwina),
    ] {
        at_moonrise(key, month);
    }
}

/// The place the synthetic moonrises are said to be found at.
const DELHI: Place = Place::new(
    Latitude::literal(28.6139),
    Longitude::literal(77.209),
    Altitude::literal(216.0),
);

/// *Dharmasindhu* p. 44's Ashunya-shayana vrata: Ashadha's dark 2nd at
/// moonrise, the earlier day only when its moonrise alone holds it, the
/// later when both or neither do; and the moonrise it is broken at, with
/// its place, on the answer.
#[test]
fn the_ashunya_shayana_takes_the_later_day_unless_the_earlier_moonrise_alone_holds_it() {
    let mut ashunya = FestivalRule::dharmasindhu()
        .into_iter()
        .find(|rule| rule.key == "ASHUNYA_SHAYANA")
        .unwrap();
    assert!(ashunya.reads_moonrise());
    let tithis = [
        (Tithi::KrishnaPratipada, ghati(1, 8.0)),
        (Tithi::KrishnaDvitiya, ghati(2, 40.0)),
        (Tithi::KrishnaTritiya, ghati(5, 0.0)),
    ];
    let mut days = days_of(&tithis, &[], Masa::Ashadha, MonthKind::Nija);
    let rise = |day: usize, at: f64| {
        Moonrise::Rose(MoonriseAt {
            at: JulianDay::literal(ghati(day, at)),
            place: DELHI,
        })
    };
    let kept = |rule: &FestivalRule, days: &[FestivalDay]| {
        let answer = observances(std::slice::from_ref(rule), days).unwrap();
        let kept = answer.observances.first().unwrap().clone();
        (kept.day.day - 1, kept.case, kept.moonrise)
    };
    // Both, neither and the later alone: the later (दिनद्वये सत्त्वेऽसत्त्वे वा परैव).
    days[1].moonrise = rise(1, 50.0);
    days[2].moonrise = rise(2, 30.0);
    assert_eq!(kept(&ashunya, &days).0, 2);
    days[1].moonrise = rise(1, 5.0);
    assert_eq!(kept(&ashunya, &days).0, 2);
    days[2].moonrise = rise(2, 45.0);
    assert_eq!(kept(&ashunya, &days).0, 2);
    // The earlier alone: the earlier, with that day's moonrise and place.
    days[1].moonrise = rise(1, 50.0);
    let (day, case, moonrise) = kept(&ashunya, &days);
    assert_eq!((day, case), (1, Case::EarlierOnly));
    assert_eq!(
        moonrise,
        Some(MoonriseAt {
            at: JulianDay::literal(ghati(1, 50.0)),
            place: DELHI
        })
    );
    // A rule kept at sunrise answers no moonrise, unless its fast is
    // broken at one; then the day the fast falls on carries it.
    ashunya.at = Window::Sunrise;
    assert_eq!(kept(&ashunya, &days).2, None);
    ashunya.meta = teistro_panchanga::festival::FestivalMeta::of_text("ASHUNYA_SHAYANA");
    assert!(ashunya.breaks_at_moonrise());
    // The later day's sunrise alone holds the 2nd.
    let (day, _, moonrise) = kept(&ashunya, &days);
    assert_eq!(
        (day, moonrise.map(|seen| seen.at)),
        (2, Some(JulianDay::literal(ghati(2, 45.0))))
    );
}

/// [`a_rite_at_moonrise_reads_each_days_first_moonrise`] for one rule.
fn at_moonrise(key: &str, month: Masa) {
    let sankashta = FestivalRule::dharmasindhu()
        .into_iter()
        .find(|rule| rule.key == key)
        .unwrap();
    assert!(sankashta.reads_moonrise());
    // The 4th from day 1's eighth ghati to day 2's fortieth, judged
    // between days 1 and 2.
    let tithis = [
        (Tithi::KrishnaTritiya, ghati(1, 8.0)),
        (Tithi::KrishnaChaturthi, ghati(2, 40.0)),
        (Tithi::KrishnaPanchami, ghati(5, 0.0)),
    ];
    let mut days = days_of(&tithis, &[], month, MonthKind::Nija);
    // Unsearched, the rule is refused, naming the remedy.
    let why = observances(std::slice::from_ref(&sankashta), &days).unwrap_err();
    assert!(why.to_string().contains("Almanac::moonrises"), "{why}");
    let rise = |day: usize, at: f64| {
        Moonrise::Rose(MoonriseAt {
            at: JulianDay::literal(ghati(day, at)),
            place: DELHI,
        })
    };
    let kept = |days: &[FestivalDay]| {
        let answer = observances(std::slice::from_ref(&sankashta), days).unwrap();
        assert!(answer.unjudged.is_empty(), "{:?}", answer.unjudged);
        answer
            .observances
            .first()
            .map(|kept| (kept.day.day - 1, kept.case))
    };
    // Both moonrises in the 4th: the earlier, joined to the 3rd.
    days[1].moonrise = rise(1, 50.0);
    days[2].moonrise = rise(2, 30.0);
    assert_eq!(kept(&days), Some((1, Case::Both)));
    // Only the later's: the later.
    days[1].moonrise = rise(1, 5.0);
    assert_eq!(kept(&days), Some((2, Case::LaterOnly)));
    // Only the earlier's: the earlier.
    days[1].moonrise = rise(1, 50.0);
    days[2].moonrise = rise(2, 45.0);
    assert_eq!(kept(&days), Some((1, Case::EarlierOnly)));
    // Neither: the later.
    days[1].moonrise = rise(1, 5.0);
    assert_eq!(kept(&days), Some((2, Case::Neither)));
    // No moonrise on one of the two days: unjudged, with the reason.
    days[2].moonrise = Moonrise::NoRise;
    let answer = observances(std::slice::from_ref(&sankashta), &days).unwrap();
    assert_eq!(answer.observances, []);
    assert!(
        answer.unjudged[0].why.contains("does not rise"),
        "{:?}",
        answer.unjudged
    );
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

/// Names a parana day's era and gives it a barred share, so a bar's
/// members are written.
fn barred(parana: &mut teistro_panchanga::festival::Parana, era: Option<EraNumber>) {
    parana.day.era = era;
    parana.barred = Some(ParanaBarred {
        bar: NakshatraBar::dharmasindhu()[0],
        span: Interval::literal(0.0, 1.0),
        set_aside: false,
    });
}

/// Adds a reading of each further shape to `parana`, so a window's end
/// and a reading's own window are written, and names each day's era.
fn every_reading(parana: &mut Vec<VrataParanaTime>, era: Option<EraNumber>) {
    let mut within = parana[0].clone();
    within.reading = VrataParana::Within { yamas: 3 };
    within.to = Some(within.from);
    within.to_by = Some(VrataEdge::Yamas);
    let mut after = parana[0].clone();
    after.reading = VrataParana::AfterWindow {
        at: Window::Part {
            part: DayPart::Pratah,
        },
    };
    parana.extend([within, after]);
    for time in parana {
        time.day.era = era;
    }
}

/// The string paths of a festival answer that name no catalogue member.
const NOT_MEMBERS: [&str; 69] = [
    "observances.rule",
    "observances.decidedBy.rule",
    "observances.day.resolution.kind",
    "observances.extents.day.resolution.kind",
    "observances.case",
    "observances.decidedBy.by",
    "observances.choice",
    "observances.parana.day.resolution.kind",
    "observances.parana.reading.kind",
    "observances.parana.reading.at.window",
    "observances.parana.reading.at.part",
    "observances.parana.fromBy",
    "observances.parana.toBy",
    "ekadashis.rule",
    "ekadashis.days.resolution.kind",
    "ekadashis.day.resolution.kind",
    "ekadashis.piercedAt",
    "ekadashis.excess",
    "ekadashis.choice",
    "ekadashis.parana.day.resolution.kind",
    "ekadashis.parana.kept.at.window",
    "ekadashis.parana.kept.at.part",
    "ekadashis.parana.kept.fromBy",
    "ekadashis.parana.kept.toBy",
    "unjudged.rule",
    "unjudged.why",
    "dated.rule",
    "dated.day.resolution.kind",
    "dated.on.resolution.kind",
    "anniversaries.key",
    "anniversaries.dayRule.at.window",
    "anniversaries.dayRule.clause",
    "anniversaries.dayRule.source",
    "anniversaries.dayRule.tithi",
    "anniversaries.kind",
    "anniversaries.monthBy",
    "anniversaries.origin.kind",
    "anniversaries.origin.half",
    "anniversaries.observance.rule",
    "anniversaries.observance.case",
    "anniversaries.observance.choice",
    "anniversaries.observance.decidedBy.by",
    "anniversaries.observance.day.resolution.kind",
    "anniversaries.observance.extents.day.resolution.kind",
    "observances.published.status",
    "observances.published.publisher",
    "observances.published.set",
    "observances.published.source",
    "observances.published.computed.resolution.kind",
    "ekadashis.published.status",
    "ekadashis.published.publisher",
    "ekadashis.published.set",
    "ekadashis.published.source",
    "ekadashis.published.computed.resolution.kind",
    "dated.published.status",
    "dated.published.publisher",
    "dated.published.set",
    "dated.published.source",
    "dated.published.computed.resolution.kind",
    "added.key",
    "added.names.en-Latn",
    "added.day.resolution.kind",
    "added.published.status",
    "added.published.publisher",
    "added.published.set",
    "added.published.source",
    "unplaced.rule",
    "unplaced.set",
    "unplaced.day.resolution.kind",
];

/// `Observances::MEMBERS` names every catalogue member an answer holds,
/// and nothing else: each string in its JSON is either listed there or
/// listed here as not a member, and each of both lists is present.
/// A sait with one whole candidate, its lagna and navamsa named.
fn elected() -> Sait {
    let span = Interval::literal(0.25, 0.5);
    Sait {
        window: 0,
        within: span,
        candidates: vec![SaitCandidate {
            span,
            whole: true,
            lagna: Some(Rashi::Sagittarius),
            navamsa: Some(Rashi::Gemini),
        }],
        chosen: Some(0),
        at: span.at_fraction(0.5).ok(),
    }
}

/// Marks an answer's first observance, fast and dated day published, and
/// adds a published day and an unplaced entry, so each is written.
fn mark_published(answer: &mut Observances) {
    let published = Published {
        status: PublishedStatus::Published,
        publisher: "a publisher".to_owned(),
        set: "a set".to_owned(),
        version: 1,
        source: "a page".to_owned(),
        computed: Some(answer.dated[0].day.clone()),
        window: None,
        subject_to: None,
    };
    answer.observances[0].published = Some(published.clone());
    answer.ekadashis[0].published = Some(published.clone());
    answer.dated[0].published = Some(published.clone());
    answer.added.push(AddedDay {
        key: "A_DAY".to_owned(),
        day: answer.dated[0].day.clone(),
        names: std::collections::BTreeMap::from([("en-Latn".to_owned(), "A day".to_owned())]),
        published: Published {
            computed: None,
            ..published
        },
    });
    answer.unplaced.push(Unplaced {
        set: "a set".to_owned(),
        entry: 0,
        rule: "JANMASHTAMI".to_owned(),
        day: answer.dated[0].day.clone(),
    });
}

#[test]
fn an_answer_names_its_catalogue_members_where_its_table_says() {
    // Every string an answer holds that is not a catalogue member.
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
    every_reading(&mut answer.observances[0].parana, era);
    // A following rule's observance, so `decidedBy` is written whole.
    let mut counted = answer.observances[0].clone();
    counted.decided_by = Decided::After {
        rule: "JANMASHTAMI".to_owned(),
        days: 1,
    };
    answer.observances.push(counted);
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
    barred(fast.parana.as_mut().unwrap(), era);
    // A dated observance, its day named in both calendars.
    let day = fast.day.clone();
    answer.dated.push(DatedObservance {
        rule: "JANMASHTAMI".to_owned(),
        day: day.clone(),
        on: day,
        published: None,
    });
    // An anniversary whose origin is written whole.
    let mut observance = answer.observances[0].clone();
    observance.parana.clear();
    answer.anniversaries.push(AnniversaryDay {
        key: "GRANDFATHER".to_owned(),
        count: Some(1),
        kind: AnniversaryKind::Ekoddishta,
        origin: OriginFound {
            nakshatra: Some(Nakshatra::Rohini),
            at: Some(JulianDay::literal(2_460_000.0)),
            assumed_sunrise: true,
            candidates: vec![Tithi::KrishnaShashthi, Tithi::KrishnaSaptami],
            ..OriginFound::lunar(
                Masa::Kartika,
                Tithi::KrishnaSaptami,
                false,
                Half::First,
                None,
            )
        },
        month_by: MonthClause::Nija,
        day_rule: DayRule {
            tithi: TithiShape::Vriddhi,
            at: Window::Sunrise,
            clause: DayClause::BothWhole,
            source: "a test".to_owned(),
        },
        observance,
    });
    // A published set's marks, its added day and its unplaced entry.
    mark_published(&mut answer);
    // A sait, its candidate's lagna and navamsa named.
    answer.observances[0].sait = Some(elected());
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
