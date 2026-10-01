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
    Case, Choice, DayPart, Decided, Edge, EkadashiRule, FestivalDay, FestivalRule, FollowingRule,
    Guard, Observances, Predicate, Unjudged, Which, Window, ekadashis, following, observances,
    yugma,
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
        month: Some(Masa::Shravana),
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
    let (text, nepal) = (FestivalRule::dharmasindhu(), FestivalRule::nepal());
    // The text's rules, then the monthly full-moon fast no text states.
    assert_eq!(text.len() + 1, nepal.len());
    assert_eq!(nepal.last().map(|rule| rule.key.as_str()), Some("PURNIMA_VRATA"));
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
        full_moon(ghati(1, 20.0), ghati(2, 18.0), Masa::Kartika, MonthKind::Nija),
        (Case::EarlierOnly, 1, Masa::Kartika, false)
    );
    assert_eq!(
        full_moon(ghati(1, 20.0), ghati(2, 18.0), Masa::Jyeshtha, MonthKind::Adhika),
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
    assert!(observances(&[kartika_only], &days).unwrap().observances.is_empty());
}

/// A rule spelt out without a month is kept every month, and one written
/// back out leaves the month out rather than writing a null.
#[test]
fn a_rule_without_a_month_reads_and_writes_without_one() {
    let rule = purnima_vrata();
    let written = serde_json::to_value(&rule).unwrap();
    assert!(written.get("month").is_none(), "{written}");
    assert_eq!(serde_json::from_value::<FestivalRule>(written).unwrap(), rule);
    let sunset = serde_json::to_value(Window::Sunset).unwrap();
    assert_eq!(sunset, serde_json::json!({ "window": "SUNSET" }));
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
    const NOT_MEMBERS: [&str; 15] = [
        "observances.rule",
        "observances.decidedBy.rule",
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
