//! The day a chart belongs to, against the 55 recorded charts.
//!
//! The corpus records three days' arcs for every chart — the civil date's,
//! the one before and the one after — along with which arc holds the birth
//! (`foundation.panchanga_day`), whether the birth is by day
//! (`is_day_birth`), the sunrise that anchors the lagna
//! (`lagna_sunrise_jd`) and the ishtakaal in ghati and pala under both
//! reckonings. Nothing has read any of it until now.
//!
//! The solar model here is a double over those recorded arcs, so what is
//! tested is the day *selection* and the reckoning over it, not the rise
//! and set solver — which `crates/astro` measures separately. The arcs are
//! keyed by the fixed day their own sunrise falls on rather than by the
//! label the engine gave them, because on three charts that label is a day
//! early (entry 12 of the deliberate-difference registry) and a test that
//! trusted it would be testing the mislabelling.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "tests fail by panicking, index JSON by key and print the measurement under --nocapture"
)]

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::Value;
use teistro_calendar::Gregorian;
use teistro_calendar::fixed::FixedDay;
use teistro_calendar::solar::{DayArc, DayLight, SolarModel};
use teistro_chart::day::{DayPart, chart_day};
use teistro_core::error::Error;
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro_core::settings::{PolarDayPolicy, Sunrise, SunriseConvention};
use teistro_core::time::UtcOffset;
use teistro_time::ghati::{Reckoning, ghati_pala};

/// A ghati is twenty-four minutes and a pala is two fifths of one, so a
/// civil ishtakaal that agrees to the pala agrees to 24 seconds.
const CIVIL_PALA_TOLERANCE: i64 = 1;

/// The proportional reckoning divides the night, and the engine's night
/// is `24h - daylight` rather than the interval from sunset to the next
/// sunrise (entry 15 of the deliberate-difference registry, measured at
/// up to 1.8 minutes over the corpus's 110 nights). Thirty ghatis spread
/// over a night that is 1.8 minutes out moves the count by about five
/// palas at the end of it; the band is that, and the test reports what it
/// actually measured so the number stays honest.
const PROPORTIONAL_PALA_TOLERANCE: i64 = 8;

/// The engine's `foundation.sunrise` block is the previous day's on these
/// three, and its `next_day` block the civil date's own (entry 12 of the
/// deliberate-difference registry, crux C35). The arcs here are keyed by
/// the day their sunrise actually falls on, so the mislabelling does not
/// reach the test; what these charts cannot be compared on is the engine's
/// own `lagna_sunrise_jd` and `panchanga_day`, which follow its labels.
const DAY_EARLY: [&str; 3] = ["c022", "c025", "c039"];

/// The two polar charts, where the engine synthesises an arc the sky does
/// not have (entry 3).
const SYNTHESISED: [&str; 2] = ["c028", "c029"];

/// A Sun that answers from what the fixture recorded.
#[derive(Debug)]
struct Recorded {
    arcs: BTreeMap<i64, DayArc>,
}

impl SolarModel for Recorded {
    fn sidereal_sun_deg(&self, _jd_ut: f64) -> Result<f64, Error> {
        Ok(0.0)
    }

    fn day_light(&self, day: FixedDay, _place: &Place) -> Result<DayLight, Error> {
        self.arcs
            .get(&day.get())
            .map(|arc| DayLight::Arc(*arc))
            .ok_or_else(|| Error::unsupported(format!("no recorded arc for {day}")))
    }

    fn describe(&self) -> String {
        String::from("the corpus's recorded arcs")
    }

    fn convention(&self) -> SunriseConvention {
        Sunrise::CentreNoRefraction.into()
    }
}

fn charts() -> Vec<Value> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/baseline/charts");
    let mut charts: Vec<Value> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| {
            panic!(
                "{}: {e}. The corpus is a submodule; `git submodule update --init`",
                dir.display()
            )
        })
        .map(|entry| entry.expect("an entry").path())
        .filter(|path| path.extension().is_some_and(|e| e == "json"))
        .map(|path| {
            serde_json::from_str(&std::fs::read_to_string(path).expect("a fixture"))
                .expect("valid JSON")
        })
        .collect();
    charts.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    charts
}

/// The fixture's own clock: the offset it resolved the birth under.
fn clock(chart: &Value) -> UtcOffset {
    let minutes = chart["foundation"]["tz_offset_min"]
        .as_i64()
        .expect("an offset");
    let minutes = i32::try_from(minutes).expect("an offset in minutes");
    UtcOffset::literal(minutes.div_euclid(60), minutes.rem_euclid(60), 0)
}

/// The three recorded arcs, keyed by the fixed day their sunrise falls on
/// rather than by the label the engine gave the block.
fn model(chart: &Value, clock: UtcOffset) -> Recorded {
    let mut arcs = BTreeMap::new();
    for block in ["previous_day", "sunrise", "next_day"] {
        let section = &chart["foundation"][block];
        let (Some(sunrise), Some(sunset)) = (
            section["sunrise_jd"].as_f64(),
            section["sunset_jd"].as_f64(),
        ) else {
            continue;
        };
        let sunrise = JulianDay::<Utc>::literal(sunrise);
        let day = teistro_calendar::solar::rule::local_day(&clock, sunrise).0;
        arcs.insert(
            day.get(),
            DayArc {
                sunrise,
                sunset: JulianDay::<Utc>::literal(sunset),
            },
        );
    }
    Recorded { arcs }
}

fn place(chart: &Value) -> Place {
    Place::new(
        Latitude::literal(
            chart["input"]["place"]["latitude"]
                .as_f64()
                .expect("a latitude"),
        ),
        Longitude::literal(
            chart["input"]["place"]["longitude"]
                .as_f64()
                .expect("a longitude"),
        ),
        Altitude::literal(
            chart["input"]["place"]["altitude_m"]
                .as_f64()
                .unwrap_or(0.0),
        ),
    )
}

#[test]
fn every_chart_lands_in_the_arc_the_engine_recorded() {
    let charts = charts();
    assert_eq!(charts.len(), 55);
    let mut compared = 0;
    let mut pre_sunrise = 0;
    for chart in &charts {
        let id = chart["id"].as_str().unwrap();
        if SYNTHESISED.contains(&id) || DAY_EARLY.contains(&id) {
            continue;
        }
        let clock = clock(chart);
        let model = model(chart, clock);
        let place = place(chart);
        let instant =
            JulianDay::<Utc>::literal(chart["foundation"]["jd_ut"].as_f64().expect("an instant"));
        let day = chart_day(
            &model,
            &Gregorian,
            &clock,
            &place,
            instant,
            PolarDayPolicy::Undefined,
        )
        .unwrap_or_else(|e| panic!("{id}: {e}"));

        let recorded = &chart["foundation"]["panchanga_day"];
        let (from, to) = day.part_bounds();
        for (name, mine, theirs) in [
            (
                "arc_start_jd",
                from.get(),
                recorded["arc_start_jd"].as_f64(),
            ),
            ("arc_end_jd", to.get(), recorded["arc_end_jd"].as_f64()),
        ] {
            let theirs = theirs.unwrap_or_else(|| panic!("{id}: no {name}"));
            assert!(
                (mine - theirs).abs() < 1e-9,
                "{id} {name}: {mine} against {theirs}"
            );
        }

        // The sunrise that anchors the lagna, which for a birth before
        // dawn is the previous morning's.
        let anchor = chart["foundation"]["lagna_sunrise_jd"]
            .as_f64()
            .unwrap_or_else(|| panic!("{id}: no lagna_sunrise_jd"));
        assert!(
            (day.lagna_sunrise().get() - anchor).abs() < 1e-9,
            "{id}: the lagna anchor is {} against {anchor}",
            day.lagna_sunrise().get()
        );

        // Day or night, as the engine reports it.
        let by_day = chart["foundation"]["is_day_birth"]
            .as_bool()
            .unwrap_or_else(|| panic!("{id}: no is_day_birth"));
        assert_eq!(
            day.part.is_daylight(),
            by_day,
            "{id}: {:?} against is_day_birth {by_day}",
            day.part
        );

        if recorded["kind"].as_str() == Some("pre-sunrise") {
            pre_sunrise += 1;
            assert_eq!(
                day.part,
                DayPart::Night,
                "{id}: a birth before sunrise is the previous day's night"
            );
        }
        compared += 1;
    }
    println!("{compared} charts, {pre_sunrise} of them before sunrise");
    assert!(
        pre_sunrise >= 3,
        "the corpus was recorded with pre-sunrise births in it"
    );
}

#[test]
fn the_ishtakaal_is_the_one_the_engine_recorded() {
    let charts = charts();
    let mut compared = 0;
    let mut worst = (0_i64, String::new());
    for chart in &charts {
        let id = chart["id"].as_str().unwrap();
        if SYNTHESISED.contains(&id) || DAY_EARLY.contains(&id) {
            continue;
        }
        let clock = clock(chart);
        let model = model(chart, clock);
        let instant =
            JulianDay::<Utc>::literal(chart["foundation"]["jd_ut"].as_f64().expect("an instant"));
        let day = chart_day(
            &model,
            &Gregorian,
            &clock,
            &place(chart),
            instant,
            PolarDayPolicy::Undefined,
        )
        .unwrap_or_else(|e| panic!("{id}: {e}"));

        let recorded = &chart["foundation"]["birth_timing"]["ishtakaal"];
        for (name, reckoning, tolerance) in [
            ("civil", Reckoning::Civil, CIVIL_PALA_TOLERANCE),
            (
                "proportional",
                Reckoning::Proportional,
                PROPORTIONAL_PALA_TOLERANCE,
            ),
        ] {
            let (Some(ghati), Some(pala)) = (
                recorded[name]["ghati"].as_i64(),
                recorded[name]["pala"].as_i64(),
            ) else {
                continue;
            };
            let theirs = ghati * 60 + pala;
            let mine = ghati_pala(&day.day, instant, reckoning)
                .unwrap_or_else(|e| panic!("{id} {name}: {e}"));
            let ours = i64::from(mine.ghati) * 60 + i64::from(mine.pala);
            let apart = (ours - theirs).abs();
            if apart > worst.0 {
                worst = (apart, format!("{id} {name}"));
            }
            assert!(
                apart <= tolerance,
                "{id} {name}: {}g {}p against {ghati}g {pala}p, {apart} pala apart",
                mine.ghati,
                mine.pala
            );
            compared += 1;
        }
    }
    println!(
        "{compared} ishtakaal readings, worst {} pala at {}",
        worst.0, worst.1
    );
    assert!(compared > 90, "both reckonings on every comparable chart");
}

#[test]
fn bhayat_and_bhabhoga_are_the_moons_nakshatra_and_not_the_days_arc() {
    // Measured, because the design page said otherwise until this test
    // was written. `bhabhoga` is the *duration of the Moon's traversal of
    // its nakshatra* and `bhayat` the elapsed part of it at birth — not
    // the length of the day's part and the elapsed part of that. The
    // engine's own `dashas.methods.temporal.nakshatra_span` is where they
    // come from, which is why they belong to `dasha` and not here
    // (`03-design/chart-foundation.md` §7).
    let charts = charts();
    let mut compared = 0;
    let mut worst_minutes = 0.0_f64;
    for chart in &charts {
        let id = chart["id"].as_str().unwrap();
        let timing = &chart["foundation"]["birth_timing"];
        let span = &chart["dashas"]["methods"]["temporal"]["nakshatra_span"];
        let (Some(entry), Some(exit)) = (span["entry_jd"].as_f64(), span["exit_jd"].as_f64())
        else {
            continue;
        };
        let birth = chart["foundation"]["jd_ut"].as_f64().expect("an instant");
        let ghatis = |section: &Value| {
            section["ghati"].as_f64().unwrap_or(f64::NAN)
                + section["pala"].as_f64().unwrap_or(f64::NAN) / 60.0
        };
        // A ghati is twenty-four minutes.
        for (name, recorded, days) in [
            ("bhabhoga", ghatis(&timing["bhabhoga"]), exit - entry),
            ("bhayat", ghatis(&timing["bhayat"]), birth - entry),
        ] {
            let apart = (recorded * 24.0 - days * 24.0 * 60.0).abs();
            worst_minutes = worst_minutes.max(apart);
            assert!(
                apart < 1.0,
                "{id} {name}: {recorded} ghati against {} minutes of nakshatra",
                days * 24.0 * 60.0
            );
            compared += 1;
        }

        // And the day's own part is a different length, which is the
        // point: were they the same the mistake would not matter.
        let part_minutes = chart["foundation"]["birth_timing"]["night_duration_hours"]
            .as_f64()
            .unwrap_or(f64::NAN)
            * 60.0;
        let bhabhoga_minutes = ghatis(&timing["bhabhoga"]) * 24.0;
        assert!(
            (part_minutes - bhabhoga_minutes).abs() > 1.0,
            "{id}: the nakshatra and the night are not the same length"
        );
    }
    println!("{compared} bhayat and bhabhoga readings, worst {worst_minutes:.2} minutes");
    assert!(compared >= 100, "both on every chart with a temporal span");
}

#[test]
fn the_recorded_arc_does_not_hold_the_birth_on_the_registered_charts() {
    // Entry 12: on three charts the engine's `sunrise` block is the
    // previous day's, and its `panchanga_day` follows those labels. On
    // two of them the arc it recorded does not contain the birth it
    // belongs to — a statement about the corpus that needs no model to
    // check, and the reason those charts are held out of the comparisons
    // above rather than quietly passing.
    let mut differing = Vec::new();
    for chart in &charts() {
        let id = chart["id"].as_str().unwrap();
        let foundation = &chart["foundation"];
        let birth = foundation["jd_ut"].as_f64().expect("an instant");
        let arc = &foundation["panchanga_day"];
        let (Some(start), Some(end)) = (arc["arc_start_jd"].as_f64(), arc["arc_end_jd"].as_f64())
        else {
            continue;
        };
        if !(start..=end).contains(&birth) {
            differing.push(id.to_string());
        }
    }
    differing.sort();
    assert_eq!(
        differing,
        ["c022", "c039"],
        "the charts whose own recorded arc does not contain their birth"
    );
}
