//! Every date of the coverage is answered (SR-136, `03-design/coverage-edges.md`
//! §7, C2242 to C2250): every call of the brief's table, under the Nepal
//! official and drik profiles over the built-in ephemeris, at Kathmandu and
//! New York, on every day of the first and last forty days of 1800-01-01 to
//! 2399-12-31 and on a sparse sample between.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "a test fails by panicking and indexes what it asked for"
)]

use teistro::catalogue::Calendar;
use teistro::day_sky::SkyStep;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{
    AlmanacRequest, CalendarDate, Clipped, Context, Ephemeris, FestivalRequest, SkyViewRequest,
    ZoneSpec,
};

/// The days read whole at each end.
const EDGE_DAYS: i64 = 40;

fn date(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, year, month, day)
}

fn places() -> [(&'static str, Place, ZoneSpec); 2] {
    [
        (
            "Kathmandu",
            Place::new(
                Latitude::literal(27.7172),
                Longitude::literal(85.324),
                Altitude::literal(1400.0),
            ),
            ZoneSpec::iana("Asia/Kathmandu"),
        ),
        (
            "New York",
            Place::new(
                Latitude::literal(40.7128),
                Longitude::literal(-74.006),
                Altitude::literal(10.0),
            ),
            ZoneSpec::iana("America/New_York"),
        ),
    ]
}

fn profiles() -> [(&'static str, Context); 2] {
    [
        (
            "official",
            Context::builder()
                .nepal_official([Ephemeris::Builtin])
                .build()
                .unwrap(),
        ),
        (
            "drik",
            Context::builder()
                .profile("nepali-default")
                .ephemeris([Ephemeris::Builtin])
                .build()
                .unwrap(),
        ),
    ]
}

/// Everything the brief's table asks over a run of days, at once.
fn everything() -> AlmanacRequest {
    let festivals = FestivalRequest::from_json(r#"{"rules": "NEPAL_PANCHANGA"}"#).unwrap();
    AlmanacRequest::new()
        .with_years()
        .with_festivals(festivals)
        .with_sky(SkyStep::of_minutes(60).unwrap())
}

#[test]
fn the_range_is_data() {
    let (_, sdk) = &profiles()[1];
    let coverage = sdk.coverage().unwrap();
    let span = coverage.ephemeris.unwrap();
    assert_eq!(
        (span.first.year, span.first.month, span.first.day),
        (1800, 1, 1)
    );
    assert_eq!(
        (span.last.year, span.last.month, span.last.day),
        (2399, 12, 31)
    );
    assert!(span.from_jd < 2_378_496.6 && span.to_jd > 2_597_641.4);
    let bs = coverage.bikram_sambat;
    assert!(bs.first_year <= bs.official_first_year && bs.official_last_year <= bs.last_year);
    assert_eq!(
        coverage.holidays[0].years,
        (2076..=2083).collect::<Vec<i32>>()
    );
}

#[test]
fn every_day_at_the_edges_is_answered_and_marked() {
    for (who, sdk) in &profiles() {
        for (town, place, zone) in &places() {
            for (first, last) in [
                (date(1800, 1, 1), date(1800, 2, 9)),
                (date(2399, 11, 22), date(2399, 12, 31)),
            ] {
                let answer = sdk
                    .almanac()
                    .in_zone(&first, &last, place, zone, &everything())
                    .unwrap_or_else(|why| panic!("{who} at {town} from {first}: {why}"));
                let days = &answer.days.value;
                assert_eq!(days.len(), usize::try_from(EDGE_DAYS).unwrap());
                let years = answer.years.as_ref().unwrap();
                assert!(answer.festivals.is_some() && answer.sky.is_some());
                if first.year == 1800 {
                    // The lunation and the year that held 1 January 1800
                    // opened in 1799, before the coverage.
                    assert_eq!(days[0].month.clipped, Some(Clipped::Start), "{who} {town}");
                    assert!(days[0].month.margin.is_none());
                    assert_eq!(years.value[0].clipped, Some(Clipped::Start), "{who} {town}");
                    assert_eq!(years.value[0].vikrama, 1856, "{who} {town}");
                } else {
                    let last_day = days.last().unwrap();
                    assert_eq!(last_day.month.clipped, Some(Clipped::End), "{who} {town}");
                    assert_eq!(last_day.day.clipped, Some(Clipped::End), "{who} {town}");
                    assert_eq!(
                        years.value.last().unwrap().clipped,
                        Some(Clipped::End),
                        "{who} {town}"
                    );
                }
            }
        }
    }
}

#[test]
fn the_middle_is_answered_and_unmarked() {
    let request = AlmanacRequest::new().with_years();
    for (who, sdk) in &profiles() {
        for (town, place, zone) in &places() {
            // A day every seven years and a few months, 1802 to 2397.
            for year in (1802..2398).step_by(7) {
                let day = date(year, u8::try_from(1 + year % 12).unwrap(), 15);
                let answer = sdk
                    .almanac()
                    .in_zone(&day, &day, place, zone, &request)
                    .unwrap_or_else(|why| panic!("{who} at {town} on {day}: {why}"));
                let today = &answer.days.value[0];
                assert_eq!(today.month.clipped, None, "{who} {town} {day}");
                assert!(today.limbs.tithi.iter().all(|span| span.clipped.is_none()));
                assert!(
                    answer
                        .years
                        .unwrap()
                        .value
                        .iter()
                        .all(|y| y.clipped.is_none())
                );
            }
        }
    }
}

#[test]
fn the_sky_view_answers_at_both_ends() {
    let (_, sdk) = &profiles()[1];
    for jd in [2_378_496.6, 2_378_497.2, 2_597_640.9, 2_597_641.4] {
        let asked = SkyViewRequest::from_json(&format!(
            r#"{{"latitudeDeg": 27.7172, "longitudeDeg": 85.324, "at": {jd}}}"#
        ))
        .unwrap();
        sdk.almanac()
            .sky_view(&asked)
            .unwrap_or_else(|why| panic!("the sky at JD {jd}: {why}"));
    }
}
