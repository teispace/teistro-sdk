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

use teistro_calendar::CalendarDate;
use teistro_calendar::lunisolar::MonthKind;
use teistro_core::catalogue::{Calendar, Masa, Nakshatra, Tithi};
use teistro_core::interval::Interval;
use teistro_core::quantity::JulianDay;
use teistro_core::settings::LunarMonth as Convention;
use teistro_panchanga::festival::{
    Case, Choice, DayPart, Decided, FestivalDay, FestivalRule, Guard, Predicate, Which, Window,
    observances, yugma,
};
use teistro_panchanga::month;
use teistro_panchanga::span::Span;

const DAYS: usize = 5;

fn ghati(day: usize, ghatis: f64) -> f64 {
    #[allow(clippy::cast_precision_loss, reason = "a handful of days")]
    let day = day as f64;
    day + ghatis / 60.0
}

/// Five synthetic days over a run of tithis, each `(tithi, ends)` ending
/// at a Julian day, in a month named `amanta`, of `kind`.
fn days_of(
    tithis: &[(Tithi, f64)],
    nakshatras: &[(Nakshatra, f64)],
    amanta: Masa,
    kind: MonthKind,
) -> Vec<FestivalDay> {
    fn spans<T: Copy>(runs: &[(T, f64)], window: Interval) -> Vec<Span<T>> {
        let mut from = 0.0;
        let mut out = Vec::new();
        for &(member, to) in runs {
            if let Some(span) = Span::new(member, Interval::literal(from, to), window) {
                out.push(span);
            }
            from = to;
        }
        out
    }
    (0..DAYS)
        .map(|k| {
            let rise = ghati(k, 0.0);
            let window = Interval::literal(rise, rise + 1.0);
            let tithi = spans(tithis, window);
            let at_sunrise = tithi
                .first()
                .map_or(Tithi::ShuklaPratipada, |span| span.member);
            FestivalDay {
                date: CalendarDate::defined(
                    Calendar::Gregorian,
                    2024,
                    1,
                    u8::try_from(k + 1).unwrap(),
                ),
                sunrise: JulianDay::literal(rise),
                sunset: JulianDay::literal(rise + 0.5),
                next_sunrise: JulianDay::literal(rise + 1.0),
                normal: true,
                month: month::of(amanta, at_sunrise, Convention::Amanta, kind),
                tithi,
                nakshatra: spans(nakshatras, window),
            }
        })
        .collect()
}

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
    let sixteen = a_rule(
        Tithi::ShuklaDvitiya,
        vec![Guard::new(
            [Predicate::Lasts {
                day: Which::Later,
                muhurtas: 16,
            }],
            Choice::Later,
        )],
        Choice::Earlier,
    );
    assert_eq!(
        sixteen.check().unwrap_err().field(),
        Some("decide.when.muhurtas")
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

/// Vijaya Dashami (p. 72): the earlier day holds aparahna alone, but the
/// later day's 10th lasts three muhurtas and has Shravana, so the later.
#[test]
fn vijaya_dashami_yields_to_a_later_day_with_three_muhurtas_and_shravana() {
    let rule = FestivalRule::dharmasindhu()
        .into_iter()
        .find(|rule| rule.key == "VIJAYA_DASHAMI")
        .unwrap();
    // Aparahna is the fourth fifth of the daylight: ghatis 18 to 24.
    // The 10th from ghati 10 of day 1 to ghati 7 of day 2; a daylight
    // muhurta is two ghatis, so three of them end at ghati 6.
    let tithis = [
        (Tithi::ShuklaNavami, ghati(1, 10.0)),
        (Tithi::ShuklaDashami, ghati(2, 7.0)),
        (Tithi::ShuklaDwadashi, ghati(5, 0.0)),
    ];
    let shravana_later = [
        (Nakshatra::UttaraAshadha, ghati(2, 1.0)),
        (Nakshatra::Shravana, ghati(3, 0.0)),
    ];
    let days = days_of(&tithis, &shravana_later, Masa::Ashwina, MonthKind::Nija);
    let observance = &observances(std::slice::from_ref(&rule), &days)
        .unwrap()
        .observances[0];
    assert_eq!(
        (observance.case, observance.day.day - 1),
        (Case::EarlierOnly, 2)
    );
    assert_eq!(observance.decided_by, Decided::Guard { index: 1 });
    // Ending at ghati 5, short of three muhurtas: the earlier.
    let short = [
        (Tithi::ShuklaNavami, ghati(1, 10.0)),
        (Tithi::ShuklaDashami, ghati(2, 5.0)),
        (Tithi::ShuklaDwadashi, ghati(5, 0.0)),
    ];
    let days = days_of(&short, &shravana_later, Masa::Ashwina, MonthKind::Nija);
    let observance = &observances(&[rule], &days).unwrap().observances[0];
    assert_eq!(
        (observance.case, observance.day.day - 1),
        (Case::EarlierOnly, 1)
    );
    assert_eq!(observance.decided_by, Decided::Guard { index: 2 });
}

/// The new moon ends its month, so its month is read at a sunrise inside
/// it even when the next sunrise is already in the next month.
#[test]
fn a_new_moon_is_read_in_the_month_it_ends() {
    let rule = FestivalRule::dharmasindhu()
        .into_iter()
        .find(|rule| rule.key == "LAKSHMI_PUJA")
        .unwrap();
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
