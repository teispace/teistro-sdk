//! A vrata's parana (`festival-rules.md` §8.7) over synthetic days: day
//! `k` rises at `k` and sets at `k + 0.5`, so three yamas end at ghati
//! 22.5 and the night's middle stands at ghati 45.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro_calendar::lunisolar::MonthKind;
use teistro_core::JulianDay;
use teistro_core::catalogue::{Masa, Nakshatra, Tithi};
use teistro_panchanga::festival::{
    DayPart, FestivalDay, FestivalRule, FollowingRule, MoonArcs, Observance, VrataEdge,
    VrataParana, VrataParanaTime, Window, following, observances,
};

mod common;

use common::{days_of, ghati};

fn shipped(key: &str) -> FestivalRule {
    FestivalRule::dharmasindhu()
        .into_iter()
        .find(|rule| rule.key == key)
        .unwrap()
}

/// The one observance of `rule`, over these tithis and nakshatras.
fn kept(
    rule: &FestivalRule,
    tithis: &[(Tithi, f64)],
    nakshatras: &[(Nakshatra, f64)],
    month: Masa,
) -> Observance {
    let days = days_of(tithis, nakshatras, month, MonthKind::Nija);
    let answer = observances(std::slice::from_ref(rule), &days).unwrap();
    assert!(answer.unjudged.is_empty(), "{:?}", answer.unjudged);
    assert_eq!(answer.observances.len(), 1, "{:?}", answer.observances);
    let observance = answer.observances[0].clone();
    assert_eq!(observance.parana.len(), rule.parana.len());
    observance
}

/// The one observance's paranas, each falling on the day after it.
fn paranas(
    rule: &FestivalRule,
    tithis: &[(Tithi, f64)],
    nakshatras: &[(Nakshatra, f64)],
    month: Masa,
) -> Vec<VrataParanaTime> {
    let observance = kept(rule, tithis, nakshatras, month);
    for time in &observance.parana {
        assert_eq!(time.day.day, observance.day.day + 1, "the day after");
    }
    observance.parana
}

/// When the meal may begin, and what set it.
fn begins(time: &VrataParanaTime) -> (f64, VrataEdge) {
    (time.from.get(), time.from_by)
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// The 8th from day 1's ghati 40 to `ashtami` on day 2, and Rohini from
/// day 1's sunset to `rohini` (none when absent): kept on day 1, so its
/// parana falls on day 2.
fn janmashtami(ashtami: f64, rohini: Option<f64>) -> Vec<VrataParanaTime> {
    let tithis = [
        (Tithi::KrishnaShashthi, ghati(0, 30.0)),
        (Tithi::KrishnaSaptami, ghati(1, 40.0)),
        (Tithi::KrishnaAshtami, ashtami),
        (Tithi::KrishnaNavami, ghati(3, 50.0)),
        (Tithi::KrishnaDashami, ghati(5, 0.0)),
    ];
    let nakshatras: Vec<_> = rohini
        .map(|end| {
            vec![
                (Nakshatra::Krittika, ghati(1, 30.0)),
                (Nakshatra::Rohini, end),
            ]
        })
        .unwrap_or_default();
    paranas(
        &shipped("JANMASHTAMI"),
        &tithis,
        &nakshatras,
        Masa::Shravana,
    )
}

#[test]
fn janmashtami_on_the_tithi_alone_is_broken_at_its_end_and_at_sunrise_for_one_who_cannot_wait() {
    // The 8th ends at night: none by day, so its end, as on 16 August
    // 2025 at Delhi.
    let [main, sunrise] = janmashtami(ghati(2, 42.0), None).try_into().unwrap();
    assert!(matches!(main.reading, VrataParana::TithiEnd { .. }));
    let (from, by) = begins(&main);
    assert!(close(from, ghati(2, 42.0)) && by == VrataEdge::TithiEnd);
    assert!(main.to.is_none());
    assert_eq!(begins(&sunrise), (ghati(2, 0.0), VrataEdge::Sunrise));
}

#[test]
fn janmashtami_joined_to_rohini_waits_for_both_by_day_and_for_one_at_night() {
    // Both by day: the later of them.
    let both = janmashtami(ghati(2, 10.0), Some(ghati(2, 20.0)));
    let (from, by) = begins(&both[0]);
    assert!(close(from, ghati(2, 20.0)) && by == VrataEdge::NakshatraEnd);
    // One by day, the other at night: the one by day.
    let one = janmashtami(ghati(2, 10.0), Some(ghati(2, 40.0)));
    let (from, by) = begins(&one[0]);
    assert!(close(from, ghati(2, 10.0)) && by == VrataEdge::TithiEnd);
    // Neither by day: the first end.
    let neither = janmashtami(ghati(2, 50.0), Some(ghati(2, 42.0)));
    let (from, by) = begins(&neither[0]);
    assert!(close(from, ghati(2, 42.0)) && by == VrataEdge::NakshatraEnd);
    // Both over by sunrise: the sunrise, as on 5 September 2026.
    let over = janmashtami(ghati(1, 55.0), Some(ghati(1, 57.0)));
    assert_eq!(begins(&over[0]), (ghati(2, 0.0), VrataEdge::Sunrise));
}

#[test]
fn a_nakshatra_that_does_not_join_the_tithi_is_not_waited_for() {
    // Rohini ends at day 1's ghati 35, before the 8th begins at 40.
    let tithis = [
        (Tithi::KrishnaShashthi, ghati(0, 30.0)),
        (Tithi::KrishnaSaptami, ghati(1, 40.0)),
        (Tithi::KrishnaAshtami, ghati(2, 10.0)),
        (Tithi::KrishnaNavami, ghati(3, 50.0)),
        (Tithi::KrishnaDashami, ghati(5, 0.0)),
    ];
    let nakshatras = [
        (Nakshatra::Rohini, ghati(1, 35.0)),
        (Nakshatra::Mrigashira, ghati(2, 50.0)),
    ];
    let main = &paranas(
        &shipped("JANMASHTAMI"),
        &tithis,
        &nakshatras,
        Masa::Shravana,
    )[0];
    let (from, by) = begins(main);
    assert!(close(from, ghati(2, 10.0)) && by == VrataEdge::TithiEnd);
}

/// The 14th from day 1's ghati 40 to `end` on day 2: kept on day 1.
fn shivaratri(end: f64) -> Vec<VrataParanaTime> {
    let tithis = [
        (Tithi::KrishnaDwadashi, ghati(0, 30.0)),
        (Tithi::KrishnaTrayodashi, ghati(1, 40.0)),
        (Tithi::KrishnaChaturdashi, end),
        (Tithi::Amavasya, ghati(3, 50.0)),
        (Tithi::ShuklaPratipada, ghati(5, 0.0)),
    ];
    paranas(&shipped("SHIVARATRI"), &tithis, &[], Masa::Magha)
}

#[test]
fn shivaratri_is_broken_inside_the_14th_within_three_yamas_or_at_its_end() {
    // The 14th ends within three yamas: sunrise to its end, as on 27
    // February 2025.
    let [within, at_end] = shivaratri(ghati(2, 10.0)).try_into().unwrap();
    assert_eq!(begins(&within), (ghati(2, 0.0), VrataEdge::Sunrise));
    assert!(close(within.to.unwrap().get(), ghati(2, 10.0)));
    assert_eq!(within.to_by, Some(VrataEdge::TithiEnd));
    let (from, by) = begins(&at_end);
    assert!(close(from, ghati(2, 10.0)) && by == VrataEdge::TithiEnd);
    // Past three yamas: sunrise to their end, as on 16 February 2026.
    let within = &shivaratri(ghati(2, 25.0))[0];
    assert!(close(within.to.unwrap().get(), ghati(2, 22.5)));
    assert_eq!(within.to_by, Some(VrataEdge::Yamas));
}

#[test]
fn the_general_rule_s_yamas_keep_a_late_end_in_the_morning() {
    // A rule of the consumer's: the 9th at sunset, broken at its end
    // unless that runs past two yamas.
    let mut rule = shipped("RAMA_NAVAMI");
    rule.at = Window::Sunset;
    rule.parana = vec![VrataParana::TithiEnd {
        nakshatra: None,
        yamas: Some(2),
    }];
    let tithis = |end| {
        [
            (Tithi::ShuklaSaptami, ghati(0, 30.0)),
            (Tithi::ShuklaAshtami, ghati(1, 10.0)),
            (Tithi::ShuklaNavami, end),
            (Tithi::ShuklaDashami, ghati(3, 50.0)),
            (Tithi::ShuklaEkadashi, ghati(5, 0.0)),
        ]
    };
    // Kept on day 1, whose sunset the 9th alone holds; it ends at day
    // 2's ghati 10, inside two yamas (15).
    let early = &paranas(&rule, &tithis(ghati(2, 10.0)), &[], Masa::Chaitra)[0];
    let (from, by) = begins(early);
    assert!(close(from, ghati(2, 10.0)) && by == VrataEdge::TithiEnd);
    // At ghati 20, past them: the morning, sunrise to ghati 15.
    let late = &paranas(&rule, &tithis(ghati(2, 20.0)), &[], Masa::Chaitra)[0];
    assert_eq!(begins(late), (ghati(2, 0.0), VrataEdge::Sunrise));
    assert!(close(late.to.unwrap().get(), ghati(2, 15.0)));
    assert_eq!(late.to_by, Some(VrataEdge::Yamas));
}

#[test]
fn after_a_window_is_its_end_on_the_fast_day() {
    let mut rule = shipped("JANMASHTAMI");
    rule.parana = vec![VrataParana::AfterWindow {
        at: Window::NightMuhurta { muhurta: 8 },
    }];
    let tithis = [
        (Tithi::KrishnaShashthi, ghati(0, 30.0)),
        (Tithi::KrishnaSaptami, ghati(1, 40.0)),
        (Tithi::KrishnaAshtami, ghati(2, 42.0)),
        (Tithi::KrishnaNavami, ghati(3, 50.0)),
        (Tithi::KrishnaDashami, ghati(5, 0.0)),
    ];
    let observance = kept(&rule, &tithis, &[], Masa::Shravana);
    let after = &observance.parana[0];
    // The night of day 1 is ghatis 30 to 60, the 8th muhurta its 15th
    // sixtieth to its 16th: ending at ghati 46, in the fast's own day.
    assert_eq!(after.day, observance.day);
    let (from, by) = begins(after);
    assert!(
        close(from, ghati(1, 46.0)) && by == VrataEdge::WindowEnd,
        "{from}"
    );
}

#[test]
fn a_reading_out_of_range_is_refused_by_its_field() {
    let mut rule = shipped("SHIVARATRI");
    rule.parana = vec![VrataParana::Sunrise, VrataParana::Within { yamas: 5 }];
    let refused = rule.check().unwrap_err();
    assert_eq!(refused.field(), Some("parana[1].yamas"));
    rule.parana = vec![VrataParana::AfterWindow {
        at: Window::Part {
            part: DayPart::Pratah,
        },
    }];
    assert!(rule.check().is_ok());
    rule.parana = vec![VrataParana::AfterWindow {
        at: Window::NightMuhurta { muhurta: 16 },
    }];
    assert_eq!(
        rule.check().unwrap_err().field(),
        Some("parana[0].at.muhurta")
    );
}

/// A rule counted from a vrata's day names it, or the day after; the fast
/// is the vrata's, broken once, so the counted observance carries no
/// parana (Shila Charhe, Shivaratri's night by its Newar name, C614).
#[test]
fn a_rule_counted_from_a_vrata_does_not_break_its_fast_again() {
    let tithis = [
        (Tithi::KrishnaDwadashi, ghati(0, 30.0)),
        (Tithi::KrishnaTrayodashi, ghati(1, 40.0)),
        (Tithi::KrishnaChaturdashi, ghati(2, 10.0)),
        (Tithi::Amavasya, ghati(3, 50.0)),
        (Tithi::ShuklaPratipada, ghati(5, 0.0)),
    ];
    let days = days_of(&tithis, &[], Masa::Magha, MonthKind::Nija);
    let found = observances(&[shipped("SHIVARATRI")], &days).unwrap();
    assert_eq!(found.observances[0].parana.len(), 2);
    let rules = [0, 1].map(|days| FollowingRule {
        key: format!("COUNTED_{days}"),
        source: "a test".to_owned(),
        after: "SHIVARATRI".to_owned(),
        days,
        meta: teistro_panchanga::festival::FestivalMeta::default(),
    });
    let counted = following(&rules, &found.observances, &days).unwrap();
    assert_eq!(counted.observances.len(), 2);
    for observance in &counted.observances {
        assert!(observance.parana.is_empty(), "{}", observance.rule);
    }
}

/// The Navaratri's close over days whose Moon arcs were searched and hold
/// no share of Shravana: the 9th kept on day 1 when it holds that
/// evening's last three muhurtas, its 10th's visarjana, and both
/// paranas (`festival-rules.md` §8.7, *Dharmasindhu* pp. 69–71). Day
/// `k` is dated the `k + 1`th.
fn navaratri(ninth_ends: f64, tenth_ends: f64) -> (Observance, Observance) {
    let tithis = [
        (Tithi::ShuklaAshtami, ghati(0, 50.0)),
        (Tithi::ShuklaNavami, ninth_ends),
        (Tithi::ShuklaDashami, tenth_ends),
        (Tithi::ShuklaEkadashi, ghati(5, 0.0)),
    ];
    let mut days: Vec<FestivalDay> = days_of(&tithis, &[], Masa::Ashwina, MonthKind::Nija);
    for day in &mut days {
        day.moon_arcs = MoonArcs::Found(Vec::new());
    }
    let rules = [shipped("MAHA_NAVAMI"), shipped("DEVI_VISARJAN")];
    let answer = observances(&rules, &days).unwrap();
    assert!(answer.unjudged.is_empty(), "{:?}", answer.unjudged);
    let of = |key: &str| {
        answer
            .observances
            .iter()
            .find(|kept| kept.rule == key)
            .unwrap()
            .clone()
    };
    (of("MAHA_NAVAMI"), of("DEVI_VISARJAN"))
}

/// DS p. 71's remaining 9th: the 10th holds both days' mornings, so the
/// visarjana is on the later, day 3, and the meal is kept inside the 9th
/// left at day 2's sunrise; Nirnayasindhu's at the 9th's end that day.
#[test]
fn the_navaratri_meal_is_kept_in_the_ninth_left_when_the_visarjana_waits() {
    let (navami, visarjana) = navaratri(ghati(2, 10.0), ghati(3, 5.0));
    assert_eq!((navami.day.day, visarjana.day.day), (2, 4));
    let [by_the_text, on_the_tenth]: &[VrataParanaTime; 2] = navami.parana[..].try_into().unwrap();
    assert_eq!(by_the_text.day.day, 3);
    assert_eq!(
        (
            begins(by_the_text),
            by_the_text.to.map(JulianDay::get),
            by_the_text.to_by
        ),
        (
            (2.0, VrataEdge::Sunrise),
            Some(ghati(2, 10.0)),
            Some(VrataEdge::TithiEnd)
        )
    );
    assert_eq!(on_the_tenth.day.day, 3);
    assert!(close(on_the_tenth.from.get(), ghati(2, 10.0)));
    assert_eq!(on_the_tenth.from_by, VrataEdge::TithiEnd);
}

/// The visarjana on the day after the fast: the meal follows it there,
/// from that day's sunrise, the 10th having begun in the night.
#[test]
fn the_navaratri_meal_follows_the_visarjana_on_its_day() {
    let (navami, visarjana) = navaratri(ghati(1, 55.0), ghati(2, 50.0));
    assert_eq!((navami.day.day, visarjana.day.day), (2, 3));
    let by_the_text = &navami.parana[0];
    assert_eq!(by_the_text.day.day, 3);
    assert_eq!(begins(by_the_text), (2.0, VrataEdge::Rite));
    // The 9th ended in day 1's night, so the 10th's first daylight is
    // day 2's sunrise.
    let on_the_tenth = &navami.parana[1];
    assert_eq!(
        (on_the_tenth.day.day, begins(on_the_tenth)),
        (3, (2.0, VrataEdge::Sunrise))
    );
}

/// Nirnayasindhu's reading on the fast day itself: a 9th ending by day
/// there is broken at its end, the same day.
#[test]
fn the_tenth_s_first_daylight_may_be_the_fast_day() {
    let mut rule = shipped("MAHA_NAVAMI");
    rule.parana = vec![VrataParana::NextTithi];
    let tithis = [
        (Tithi::ShuklaAshtami, ghati(0, 50.0)),
        (Tithi::ShuklaNavami, ghati(1, 28.0)),
        (Tithi::ShuklaDashami, ghati(2, 50.0)),
    ];
    // Day 0's evening holds no 9th, so the fast is day 1's, the later.
    let observance = kept(&rule, &tithis, &[], Masa::Ashwina);
    let time = &observance.parana[0];
    assert_eq!(observance.day.day, 2);
    assert_eq!(time.day, observance.day);
    assert!(close(time.from.get(), ghati(1, 28.0)) && time.from_by == VrataEdge::TithiEnd);
}

/// A rite no rule of the request keeps, the rule's own key, and a remnant
/// past a day's sixty ghatis are each refused by their field.
#[test]
fn a_parana_after_a_rite_is_refused_by_its_field() {
    let days = days_of(
        &[(Tithi::ShuklaNavami, ghati(1, 40.0))],
        &[],
        Masa::Ashwina,
        MonthKind::Nija,
    );
    let alone = observances(&[shipped("MAHA_NAVAMI")], &days).unwrap_err();
    assert_eq!(alone.field(), Some("parana[0].rite"));
    let mut rule = shipped("MAHA_NAVAMI");
    rule.parana = vec![VrataParana::AfterRite {
        rite: "DEVI_VISARJAN".to_owned(),
        remnant: Some(61),
    }];
    assert_eq!(rule.check().unwrap_err().field(), Some("parana[0].remnant"));
    rule.parana = vec![VrataParana::AfterRite {
        rite: "MAHA_NAVAMI".to_owned(),
        remnant: None,
    }];
    assert_eq!(rule.check().unwrap_err().field(), Some("parana[0].rite"));
}
