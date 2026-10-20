//! Where Nepal's official reckoning and the modern one disagree
//! (`03-design/setup-disagreements.md`): November 2025 at Kathmandu, the
//! month every binding's `nepal_official` example prints, read back
//! through `AlmanacArea::disagreements` and its festival twin.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "tests fail by panicking and index what they asked for; the measurement prints its figures"
)]

use teistro::catalogue::{Calendar, Masa, Tithi};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{
    CalendarDate, Context, DisagreementRequest, Ephemeris, FestivalPack, FestivalRequest,
    LimbDisagreement, UtcOffset,
};

fn kathmandu() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    )
}

fn contexts() -> (Context, Context) {
    let official = Context::builder().nepal_official([]).build().unwrap();
    let modern = Context::builder()
        .profile("nepali-default")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap();
    (official, modern)
}

fn november(day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, 2025, 11, day)
}

/// Each side's instant falls on its own side of its own sunrise, opposite
/// the other's: the one running ahead changed before its sunrise, the one
/// behind changes after it.
fn explained<T>(found: &LimbDisagreement<T>) {
    let (Some(a), Some(b)) = (found.official.change, found.other.change) else {
        panic!("adjacent members name the instant they meet");
    };
    assert!(
        a.minutes_from_sunrise * b.minutes_from_sunrise <= 0.0,
        "{} and {} minutes",
        a.minutes_from_sunrise,
        b.minutes_from_sunrise
    );
    let span = |from: f64, to: f64, at: f64| from <= at && at <= to;
    assert!(span(
        found.official.span.from.get(),
        found.official.span.to.get(),
        a.at.get()
    ));
    assert!(span(
        found.other.span.from.get(),
        found.other.span.to.get(),
        b.at.get()
    ));
}

#[test]
fn the_days_the_examples_print_are_the_tithi_disagreements() {
    let (official, modern) = contexts();
    let found = official
        .almanac()
        .disagreements(
            &modern,
            &november(1),
            &november(30),
            &kathmandu(),
            UtcOffset::literal(5, 45, 0),
        )
        .unwrap()
        .value;
    assert_eq!(found.compared, 30);
    assert_eq!(found.other_profile, "nepali-default");
    assert_eq!(found.other_settings_hash, modern.settings_hash());

    let tithi: Vec<_> = found
        .days
        .iter()
        .filter_map(|day| day.tithi.as_ref().map(|t| (day.date.day, t)))
        .collect();
    let days: Vec<u8> = tithi.iter().map(|(day, _)| *day).collect();
    assert_eq!(days, [1, 2, 9, 10, 11]);
    for (_, found) in &tithi {
        explained(found);
    }

    // Haribodhini Ekadashi: the text's 11th had begun before sunrise on
    // the 1st, the modern 10th runs on past it.
    let first = tithi[0].1;
    assert_eq!(first.official.member, Tithi::ShuklaEkadashi);
    assert_eq!(first.other.member, Tithi::ShuklaDashami);
    assert!(first.official.change.unwrap().minutes_from_sunrise < 0.0);
    assert!(first.other.change.unwrap().minutes_from_sunrise > 0.0);

    // Every day reported differs somewhere, and a day that agrees on
    // every limb is not reported.
    for day in &found.days {
        assert!(
            day.tithi.is_some()
                || day.nakshatra.is_some()
                || day.yoga.is_some()
                || day.karana.is_some()
        );
    }
}

#[test]
fn a_reckoning_agrees_with_itself() {
    let (official, _) = contexts();
    let found = official
        .almanac()
        .disagreements(
            &official,
            &november(1),
            &november(30),
            &kathmandu(),
            UtcOffset::literal(5, 45, 0),
        )
        .unwrap()
        .value;
    assert!(found.days.is_empty(), "{:?}", found.days);
    let festivals = official
        .almanac()
        .festival_disagreements(
            &official,
            &november(1),
            &november(30),
            &kathmandu(),
            UtcOffset::literal(5, 45, 0),
            &FestivalRequest::from(FestivalPack::Nepal),
        )
        .unwrap()
        .value;
    assert!(festivals.compared > 0);
    assert!(festivals.observances.is_empty() && festivals.ekadashis.is_empty());
}

#[test]
fn the_festival_twin_names_each_moved_occurrence_with_both_days() {
    let (official, modern) = contexts();
    let found = official
        .almanac()
        .festival_disagreements(
            &modern,
            &november(1),
            &november(30),
            &kathmandu(),
            UtcOffset::literal(5, 45, 0),
            &FestivalRequest::from(FestivalPack::Nepal),
        )
        .unwrap()
        .value;
    for moved in &found.observances {
        match (&moved.official, &moved.other) {
            (Some(a), Some(b)) => assert_ne!(a.day, b.day, "{}", moved.rule),
            (Some(_), None) | (None, Some(_)) => {}
            (None, None) => panic!("{} is on neither side", moved.rule),
        }
    }
    // Haribodhini Ekadashi falls on the 1st by the text and the 2nd by
    // the modern sky, the day the modern Ekadashi stands at sunrise.
    let haribodhini = found
        .ekadashis
        .iter()
        .find(|fast| fast.tithi == Tithi::ShuklaEkadashi)
        .expect("the bright 11th of Kartik moves");
    let (a, b) = (
        haribodhini.official.as_ref().unwrap(),
        haribodhini.other.as_ref().unwrap(),
    );
    assert_eq!((a.day.day, b.day.day), (1, 2));
}

/// The record a binding sends answers what the two Rust calls do, each
/// side written in full and sealed as the boundary carries it.
#[test]
fn a_binding_s_request_answers_both_comparisons_in_full() {
    let (official, modern) = contexts();
    let asked = DisagreementRequest::from_json(
        r#"{"first": {"year": 2025, "month": 11, "day": 1},
            "last": {"year": 2025, "month": 11, "day": 30},
            "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400,
            "utcOffsetSeconds": 20700, "festivals": {"rules": "NEPAL"}}"#,
    )
    .unwrap();
    let answer = official
        .almanac()
        .disagreements_request(&modern, &asked)
        .unwrap();
    let days = answer
        .days
        .expect("the days are compared unless a request says not");
    let direct = official
        .almanac()
        .disagreements(
            &modern,
            &november(1),
            &november(30),
            &kathmandu(),
            UtcOffset::literal(5, 45, 0),
        )
        .unwrap();
    assert_eq!(days.value, direct.value.in_full().unwrap());
    assert_eq!(
        days.provenance.settings_hash,
        direct.provenance.settings_hash
    );
    let first = &days.value["days"][0];
    assert_eq!(first["date"]["calendar"], "calendar.GREGORIAN");
    assert_eq!(
        first["tithi"]["official"]["member"],
        "tithi.SHUKLA_EKADASHI"
    );
    assert_eq!(first["tithi"]["other"]["member"], "tithi.SHUKLA_DASHAMI");

    let festivals = answer.festivals.expect("a festival request was sent");
    let fast = festivals.value["ekadashis"]
        .as_array()
        .unwrap()
        .iter()
        .find(|fast| fast["tithi"] == "tithi.SHUKLA_EKADASHI")
        .expect("Haribodhini Ekadashi moves");
    assert!(fast["month"].as_str().unwrap().starts_with("masa."));
    assert_eq!(fast["official"]["day"]["day"], 1);
    assert_eq!(fast["other"]["day"]["day"], 2);
    assert_eq!(fast["official"]["tithi"], "tithi.SHUKLA_EKADASHI");
    assert_eq!(fast["official"]["day"]["calendar"], "calendar.GREGORIAN");

    let lean = DisagreementRequest::from_json(
        r#"{"first": {"year": 2025, "month": 11, "day": 1},
            "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700,
            "days": false, "festivals": {"rules": "NEPAL"}}"#,
    )
    .unwrap();
    let answer = official
        .almanac()
        .disagreements_request(&modern, &lean)
        .unwrap();
    assert!(answer.days.is_none() && answer.festivals.is_some());

    let refused = DisagreementRequest::from_json(
        r#"{"first": {"year": 2025, "month": 11, "day": 1},
            "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700,
            "festivals": {"rules": "NOT_A_PACK"}}"#,
    )
    .unwrap_err();
    assert!(
        refused
            .field()
            .is_some_and(|field| field.starts_with("disagreements.festivals")),
        "{:?}",
        refused.field()
    );
}

/// In 2028 the text has an adhika Kartika the modern sky lacks, and every
/// month to January 2029 is named one apart. One fast is still one
/// occurrence: paired by its tithi, reported for its month, never split
/// into two one-sided records.
#[test]
fn a_month_named_apart_is_one_occurrence_with_two_names() {
    let (official, modern) = contexts();
    let found = official
        .almanac()
        .festival_disagreements(
            &modern,
            &CalendarDate::defined(Calendar::Gregorian, 2028, 10, 1),
            &CalendarDate::defined(Calendar::Gregorian, 2029, 1, 31),
            &kathmandu(),
            UtcOffset::literal(5, 45, 0),
            &FestivalRequest::from(FestivalPack::Nepal),
        )
        .unwrap()
        .value;
    for fast in &found.ekadashis {
        let (Some(a), Some(b)) = (&fast.official, &fast.other) else {
            panic!("{} {:?} is on one side only", fast.rule, fast.tithi);
        };
        assert!(a.day != b.day || a.month != b.month || a.adhika != b.adhika);
    }
    let renamed = found
        .ekadashis
        .iter()
        .find(|fast| {
            let day = &fast.official.as_ref().unwrap().day;
            (day.year, day.month, day.day) == (2028, 10, 29)
        })
        .expect("the bright 11th of the text's adhika Kartika");
    let (a, b) = (
        renamed.official.as_ref().unwrap(),
        renamed.other.as_ref().unwrap(),
    );
    assert_eq!((a.month, a.adhika), (Masa::Kartika, true));
    assert_eq!((b.month, b.adhika), (Masa::Kartika, false));
    assert_eq!(a.day, b.day);
}

/// The figures of `setup-disagreements.md` §4: Kathmandu, 2020 to 2030,
/// the official reckoning against the modern one over the built-in sky.
/// A measurement, not a check, and slow (the built-in ephemeris reckons a
/// modern year in about 14 s):
///
/// ```text
/// cargo test --release -p teistro --test disagreements -- --ignored --nocapture
/// ```
#[test]
#[ignore = "a measurement for the design page, several minutes"]
fn measure_kathmandu_2020_to_2030() {
    let (official, modern) = contexts();
    let offset = UtcOffset::literal(5, 45, 0);
    let year = |y| {
        (
            CalendarDate::defined(Calendar::Gregorian, y, 1, 1),
            CalendarDate::defined(Calendar::Gregorian, y, 12, 31),
        )
    };
    let spread = |name: &str, mut v: Vec<f64>| {
        v.sort_by(f64::total_cmp);
        let at = |p: f64| {
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::cast_precision_loss,
                reason = "an index into a list of a few hundred"
            )]
            let i = ((v.len() - 1) as f64 * p).round() as usize;
            v[i]
        };
        println!(
            "{name}: median {:.0}, 90th percentile {:.0}, largest {:.0} minutes",
            at(0.5),
            at(0.9),
            at(1.0)
        );
    };
    let (mut compared, mut days) = (0, Vec::new());
    for y in 2020..=2030 {
        let (from, to) = year(y);
        let found = official
            .almanac()
            .disagreements(&modern, &from, &to, &kathmandu(), offset)
            .unwrap()
            .value;
        compared += found.compared;
        days.extend(found.days);
    }
    let count =
        |f: &dyn Fn(&teistro::DayDisagreement) -> bool| days.iter().filter(|d| f(d)).count();
    println!(
        "days compared {compared}, any limb differs on {}",
        days.len()
    );
    println!(
        "tithi {}, nakshatra {}, yoga {}, karana {}",
        count(&|d| d.tithi.is_some()),
        count(&|d| d.nakshatra.is_some()),
        count(&|d| d.yoga.is_some()),
        count(&|d| d.karana.is_some())
    );
    let tithi: Vec<_> = days.iter().filter_map(|d| d.tithi.as_ref()).collect();
    let explained: Vec<_> = tithi
        .iter()
        .filter_map(|t| Some((t.official.change?, t.other.change?)))
        .collect();
    println!(
        "tithi disagreements with one instant on each side: {} of {}",
        explained.len(),
        tithi.len()
    );
    spread(
        "official, from sunrise",
        explained
            .iter()
            .map(|(a, _)| a.minutes_from_sunrise.abs())
            .collect(),
    );
    spread(
        "modern, from sunrise",
        explained
            .iter()
            .map(|(_, b)| b.minutes_from_sunrise.abs())
            .collect(),
    );
    for pack in [FestivalPack::Nepal, FestivalPack::NepalPanchanga] {
        measure_festivals(&official, &modern, pack);
    }
}

/// The festival half of [`measure_kathmandu_2020_to_2030`], for one pack.
fn measure_festivals(official: &Context, modern: &Context, pack: FestivalPack) {
    let offset = UtcOffset::literal(5, 45, 0);
    let year = |y| {
        (
            CalendarDate::defined(Calendar::Gregorian, y, 1, 1),
            CalendarDate::defined(Calendar::Gregorian, y, 12, 31),
        )
    };
    let (mut compared, mut moved, mut renamed, mut one_sided) = (0, 0, 0, 0);
    let (mut fasts, mut fasts_moved, mut fasts_renamed, mut fasts_one_sided) = (0, 0, 0, 0);
    for y in 2020..=2030 {
        let (from, to) = year(y);
        let ours = official
            .almanac()
            .festivals(
                &from,
                &to,
                &kathmandu(),
                offset,
                &FestivalRequest::from(pack),
            )
            .unwrap()
            .value;
        let found = official
            .almanac()
            .festival_disagreements(
                modern,
                &from,
                &to,
                &kathmandu(),
                offset,
                &FestivalRequest::from(pack),
            )
            .unwrap()
            .value;
        compared += ours.observances.len();
        fasts += ours.ekadashis.len();
        for o in &found.observances {
            match (&o.official, &o.other) {
                (Some(a), Some(b)) if a.day != b.day => moved += 1,
                (Some(_), Some(_)) => renamed += 1,
                _ => one_sided += 1,
            }
        }
        for f in &found.ekadashis {
            match (&f.official, &f.other) {
                (Some(a), Some(b)) if a.day != b.day => fasts_moved += 1,
                (Some(_), Some(_)) => fasts_renamed += 1,
                _ => fasts_one_sided += 1,
            }
        }
    }
    println!(
        "{pack:?}: {compared} observances, {moved} on another day, {renamed} under another month, {one_sided} on one side only; \
         {fasts} Ekadashi fasts, {fasts_moved} on another day, {fasts_renamed} under another month, {fasts_one_sided} on one side only"
    );
}
