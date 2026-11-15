//! Each lunar month's decision margin (SR-92, C1902): how far the Sun
//! stood from a sign boundary at its two new moons, in degrees and in
//! minutes of its motion, and whether that is inside
//! `calendars.marginal_month_minutes`.
//!
//! The acceptance values are the consumer's (addendum 8, SR-92): AD
//! 2028's adhika Kartika and kshaya Margashirsha under `nepali-committee`
//! turn on 0.044° and 0.017° of the Sun, and 2029's Chaitra under
//! `nepali-drik` on 44 minutes. The precedents are tithipatra's
//! `sdk-gap-research.md` §3.11 table, every year of AD 1900–2100 the
//! committee and a drik almanac decide a month differently, reproduced
//! here from the field alone.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    missing_docs,
    reason = "a test fails by panicking"
)]

use std::collections::{BTreeMap, BTreeSet};

use teistro::catalogue::{Calendar, Masa};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{
    CalendarDate, CellsRequest, Context, DayClock, Ephemeris, MonthKind, MonthMargin, UtcOffset,
};

fn context(profile: &str, settings: Option<&str>) -> Context {
    // The committee's sky is the text's, opened over its own provider.
    let sky = if profile == "nepali-committee" {
        Ephemeris::SuryaSiddhanta
    } else {
        Ephemeris::Builtin
    };
    let builder = Context::builder().profile(profile).ephemeris([sky]);
    match settings {
        Some(json) => builder.settings_json(json),
        None => builder,
    }
    .build()
    .expect("a context")
}

fn kathmandu() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    )
}

/// The Gregorian date holding a Julian day (UTC), by the days since the
/// Unix epoch (Hinnant's `civil_from_days`).
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a Julian day of the precedents, far inside i64 and u8"
)]
fn date_of(jd: f64) -> CalendarDate {
    let z = (jd + 0.5).floor() as i64 - 2_440_588 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u8;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u8;
    let year = yoe + era * 400 + i64::from(month <= 2);
    CalendarDate::defined(Calendar::Gregorian, year as i32, month, day)
}

/// The Gregorian year a Julian day falls in.
fn year_of(jd: f64) -> i32 {
    date_of(jd).year
}

/// One lunar month as the field says it.
#[derive(Clone, Copy, Debug)]
struct Month {
    amanta: Masa,
    kind: MonthKind,
    margin: MonthMargin,
}

/// The month a date's sunrise falls in, at Kathmandu on Nepal's clock.
fn month_on(sdk: &Context, date: &CalendarDate) -> Month {
    let asked = CellsRequest {
        first: date.clone(),
        last: date.clone(),
        place: kathmandu(),
        clock: DayClock::from(UtcOffset::literal(5, 45, 0)),
        festivals: None,
    };
    let cell = sdk
        .almanac()
        .cells(&asked)
        .expect("a cell")
        .value
        .cells
        .remove(0);
    Month {
        amanta: cell.month.amanta,
        kind: cell.month.kind,
        margin: cell
            .month
            .margin
            .expect("an almanac month carries its margin"),
    }
}

/// Every lunar month whose opening new moon falls in `years`, in order:
/// each found two days after the last one's closing new moon.
fn months(sdk: &Context, years: std::ops::RangeInclusive<i32>) -> Vec<Month> {
    let mut found = Vec::new();
    let mut month = month_on(
        sdk,
        &CalendarDate::defined(Calendar::Gregorian, *years.start(), 1, 3),
    );
    while year_of(month.margin.opening.new_moon.get()) <= *years.end() {
        if years.contains(&year_of(month.margin.opening.new_moon.get())) {
            found.push(month);
        }
        let next = month_on(sdk, &date_of(month.margin.closing.new_moon.get() + 2.0));
        assert_eq!(
            next.margin.opening.new_moon, month.margin.closing.new_moon,
            "one month's closing new moon opens the next"
        );
        month = next;
    }
    found
}

/// The new moons of the year's months that are not nija, with the Sun's
/// distance from the boundary in degrees.
fn uncommon_degrees(months: &[Month]) -> Vec<f64> {
    months
        .iter()
        .filter(|month| month.kind != MonthKind::Nija)
        .flat_map(|month| [month.margin.opening.degrees, month.margin.closing.degrees])
        .collect()
}

/// AD 2028 under the committee: an adhika Kartika then a kshaya
/// Margashirsha, decided by the Sun 0.044° and 0.017° from a boundary.
#[test]
fn the_committee_s_2028_turns_on_hundredths_of_a_degree() {
    let sdk = context("nepali-committee", None);
    let year = months(&sdk, 2028..=2028);
    let uncommon: Vec<_> = year
        .iter()
        .filter(|month| month.kind != MonthKind::Nija)
        .map(|month| (month.amanta, month.kind))
        .collect();
    assert_eq!(
        uncommon,
        [
            (Masa::Kartika, MonthKind::Adhika),
            (Masa::Margashirsha, MonthKind::Kshaya)
        ]
    );
    let degrees = uncommon_degrees(&year);
    for wanted in [0.044, 0.017] {
        assert!(
            degrees
                .iter()
                .any(|got| (got.abs() - wanted).abs() <= 0.001),
            "{wanted}° among {degrees:?}"
        );
    }
    for month in year.iter().filter(|month| month.kind != MonthKind::Nija) {
        assert!(month.margin.marginal, "{month:?}");
    }
}

/// 2029's adhika Chaitra under drik: 44 minutes of the Sun from another
/// kind, on the true basis the consumer measured and SR-100 made the
/// default (on the mean basis the nutation in longitude moves the Sun by
/// about 0.005° and the margin to about 37 minutes).
#[test]
fn drik_s_2029_chaitra_turns_on_forty_four_minutes() {
    let chaitra = months(&context("nepali-drik", None), 2029..=2029)
        .into_iter()
        .find(|month| month.amanta == Masa::Chaitra && month.kind == MonthKind::Adhika)
        .expect("an adhika Chaitra in 2029");
    let least = chaitra.margin.least_minutes();
    assert!((least - 44.0).abs() <= 1.0, "{least} min: {chaitra:?}");
    assert!(chaitra.margin.marginal);
}

/// The threshold is a setting: nought calls no month marginal, and a
/// call's own settings move it.
#[test]
fn the_threshold_is_a_setting() {
    let date = CalendarDate::defined(Calendar::Gregorian, 2029, 4, 1);
    let default = month_on(&context("nepali-drik", None), &date);
    assert!(default.margin.marginal);
    let none = month_on(
        &context(
            "nepali-drik",
            Some(r#"{"calendars": {"marginal_month_minutes": 0}}"#),
        ),
        &date,
    );
    assert!(!none.margin.marginal);
    assert_eq!(
        none.margin.opening, default.margin.opening,
        "the margin is the sky's"
    );
}

/// The degrees and the minutes agree: the Sun's own motion over the gap.
#[test]
fn the_minutes_are_the_sun_s_time_over_the_degrees() {
    for month in months(&context("nepali-drik", None), 2026..=2026) {
        for side in [month.margin.opening, month.margin.closing] {
            assert!(side.degrees.abs() <= 15.0, "{side:?}");
            assert_eq!(side.degrees.signum(), side.minutes.signum(), "{side:?}");
            // The Sun moves 0.95° to 1.02° a day.
            let rate = side.degrees / (side.minutes / 1440.0);
            assert!((0.94..=1.03).contains(&rate), "{rate}°/day: {side:?}");
        }
    }
}

/// Each year of AD 1900–2100 the committee and drik decide a month
/// differently, with the least margin among its uncommon months, hours to
/// a tenth, as the field gives them: the golden.
const PRECEDENTS: &[(i32, f64)] = &[
    (1901, 0.4),
    (1907, 0.9),
    (1917, 4.1),
    (1926, 0.6),
    (1963, 0.8),
    (1977, 3.8),
    (2028, 0.4),
    (2042, 4.4),
    (2047, 0.3),
    (2048, 0.3),
    (2050, 5.7),
    (2058, 3.5),
    (2061, 6.3),
    (2067, 0.9),
    (2077, 1.6),
    (2080, 3.8),
    (2085, 0.4),
    (2086, 1.8),
    (2099, 2.4),
];

/// The same years in tithipatra's `sdk-gap-research.md` §3.11, measured
/// independently (Python, Swiss Ephemeris and the committee's bija), with
/// its smallest margins: the field's are within a quarter hour of them.
const RESEARCH: &[(i32, f64)] = &[
    (1901, 0.4),
    (1907, 0.9),
    (1917, 4.0),
    (1926, 0.6),
    (1963, 0.9),
    (1977, 3.7),
    (2028, 0.4),
    (2042, 4.3),
    (2047, 0.3),
    (2048, 0.7),
    (2050, 5.5),
    (2058, 3.5),
    (2061, 6.1),
    (2067, 0.9),
    (2077, 1.6),
    (2080, 3.7),
    (2085, 0.4),
    (2086, 1.8),
    (2099, 2.3),
];

/// The §3.11 table from the field: every year the two decide differently,
/// each flagged under the default threshold by its nearest new moon.
#[test]
fn the_precedents_of_1900_to_2100_are_the_field_s() {
    let committee = months(&context("nepali-committee", None), 1900..=2100);
    let drik = months(&context("nepali-drik", None), 1900..=2100);
    let uncommon = |months: &[Month]| {
        let mut by_year: BTreeMap<i32, BTreeSet<String>> = BTreeMap::new();
        for month in months.iter().filter(|month| month.kind != MonthKind::Nija) {
            by_year
                .entry(year_of(month.margin.opening.new_moon.get() + 15.0))
                .or_default()
                .insert(format!("{:?} {:?}", month.kind, month.amanta));
        }
        by_year
    };
    let (by_committee, by_drik) = (uncommon(&committee), uncommon(&drik));
    let years: BTreeSet<i32> = by_committee.keys().chain(by_drik.keys()).copied().collect();
    let mut found = Vec::new();
    for year in years {
        if by_committee.get(&year) == by_drik.get(&year) {
            continue;
        }
        let least = committee
            .iter()
            .chain(&drik)
            .filter(|month| {
                month.kind != MonthKind::Nija
                    && year_of(month.margin.opening.new_moon.get() + 15.0) == year
            })
            .map(|month| month.margin.least_minutes() / 60.0)
            .fold(f64::INFINITY, f64::min);
        assert!(
            least * 60.0 < 420.0,
            "{year} turns on {least} h, past the default threshold"
        );
        found.push((year, (least * 10.0).round() / 10.0));
    }
    assert_eq!(found, PRECEDENTS);
    for (&(year, field), &(noted, research)) in PRECEDENTS.iter().zip(RESEARCH) {
        assert_eq!(year, noted);
        // 2048's research row is its adhika Phalguna alone; the field
        // counts the kshaya Margashirsha whose new moon falls in it too.
        if year != 2048 {
            assert!(
                (field - research).abs() <= 0.25,
                "{year}: {field} h against {research} h"
            );
        }
    }
}
