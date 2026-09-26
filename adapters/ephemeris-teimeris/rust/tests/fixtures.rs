//! The rise and set solver against the baseline's fixtures: for each of
//! the 55 charts, the sunrise and sunset of the local civil day at the
//! place, from the SDK's solver over Teimeris's positions under the
//! almanac's convention (the upper limb with refraction, the baseline's
//! `apparent-refraction`) and from Teimeris's own search, compared with
//! what the baseline recorded (`fixtures/README.md`), which reckons at sea
//! level; and at each chart's own height, the almanac's fixed 34′ against
//! the engine's standard air and then the same air named on both sides
//! (`docs/03-design/horizon-atmosphere.md`). Run by hand with the engine
//! present; the worst differences are published in
//! `docs/03-design/astro-events-and-crossings.md`.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::print_stdout,
    clippy::indexing_slicing,
    reason = "a test fails by panicking, reads its fixtures by key and reports its measurement"
)]

use std::path::Path;

use teistro_astro::rise_set::Solver;
use teistro_astro::{Completion, DeltaTModel};
use teistro_core::quantity::{Altitude, JulianDay, Place, Ut1};
use teistro_core::settings::{Atmosphere, OverridePolicy, Sunrise, SunriseConvention};
use teistro_ephemeris_teimeris::{TeimerisProvider, data_dir_from_env};
use teistro_port_ephemeris::{Body, EphemerisProvider, Horizon, HorizonEventKind, HorizonRequest};

/// One chart's sunrise facts as the baseline recorded them.
struct Chart {
    id: String,
    /// At sea level, where the baseline reckons its sunrise.
    place: Place,
    altitude_m: f64,
    /// The local civil day's midnight, UT.
    midnight_jd: f64,
    sunrise_jd: f64,
    sunset_jd: f64,
    next_day_sunrise_jd: f64,
    next_day_sunset_jd: f64,
    polar: bool,
}

fn charts() -> Vec<Chart> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../fixtures/baseline/charts");
    let mut charts = Vec::new();
    for entry in std::fs::read_dir(&dir).expect("the fixtures directory") {
        let path = entry.expect("an entry").path();
        if path.extension().is_none_or(|e| e != "json") {
            continue;
        }
        let value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).expect("readable")).expect("json");
        let place = &value["input"]["place"];
        let altitude_m = place["altitude_m"].as_f64().unwrap_or(0.0);
        let foundation = &value["foundation"];
        let date = foundation["local_civil_date"].as_str().expect("a date");
        let mut parts = date
            .split('-')
            .map(|p| p.parse::<i32>().expect("a date part"));
        let (y, m, d) = (
            parts.next().unwrap(),
            parts.next().unwrap(),
            parts.next().unwrap(),
        );
        let offset_min = foundation["tz_offset_min"].as_f64().expect("an offset");
        let midnight_jd = teistro_port_ephemeris::sefile::julian_day(
            y,
            u32::try_from(m).unwrap(),
            u32::try_from(d).unwrap(),
        ) - offset_min / 1440.0;
        let tags: Vec<&str> = value["tags"]
            .as_array()
            .map(|a| a.iter().filter_map(|t| t.as_str()).collect())
            .unwrap_or_default();
        charts.push(Chart {
            id: value["id"].as_str().expect("an id").to_string(),
            place: Place::try_from_degrees(
                place["latitude"].as_f64().expect("a latitude"),
                place["longitude"].as_f64().expect("a longitude"),
                0.0,
            )
            .expect("a place"),
            altitude_m,
            midnight_jd,
            sunrise_jd: foundation["sunrise"]["sunrise_jd"]
                .as_f64()
                .expect("a sunrise"),
            sunset_jd: foundation["sunrise"]["sunset_jd"]
                .as_f64()
                .expect("a sunset"),
            next_day_sunrise_jd: foundation["next_day"]["sunrise_jd"]
                .as_f64()
                .expect("a sunrise"),
            next_day_sunset_jd: foundation["next_day"]["sunset_jd"]
                .as_f64()
                .expect("a sunset"),
            polar: tags.iter().any(|t| t.contains("polar")),
        });
    }
    charts.sort_by(|a, b| a.id.cmp(&b.id));
    charts
}

/// The largest gap seen, in seconds, and where.
#[derive(Default)]
struct Worst {
    seconds: f64,
    at: String,
}

impl Worst {
    fn offer(&mut self, signed_seconds: f64, label: &str) {
        if signed_seconds.abs() > self.seconds {
            self.seconds = signed_seconds.abs();
            label.clone_into(&mut self.at);
        }
    }
}

impl std::fmt::Display for Worst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2} s at {}", self.seconds, self.at)
    }
}

/// What one horizon convention gave over every chart: the worst gaps of
/// the SDK's solver and of Teimeris's own search from the baseline, and
/// from each other.
struct Measured {
    sdk: Worst,
    native: Worst,
    between: Worst,
    /// Every event's chart height, metres, and its SDK - native gap,
    /// seconds.
    gaps: Vec<(f64, f64)>,
    compared: usize,
    skipped: Vec<String>,
    day_early: Vec<String>,
}

impl Chart {
    /// The place a measurement puts the chart at.
    fn at(&self, at: Where) -> Place {
        match at {
            Where::SeaLevel => self.place,
            Where::AtHeight => Place {
                altitude: Altitude::try_new(self.altitude_m).unwrap(),
                ..self.place
            },
        }
    }

    /// The sunrise and sunset the baseline recorded for the civil day, and
    /// whether its sunrise block was a day early. For three charts that
    /// block holds the previous day's events (`fixtures/README.md`,
    /// convention twelve), so its next-day block is the one compared.
    fn expected_arc(&self) -> (f64, f64, bool) {
        if (self.next_day_sunrise_jd - self.midnight_jd) < 1.0
            && (self.sunrise_jd - self.midnight_jd) < 0.0
        {
            (self.next_day_sunrise_jd, self.next_day_sunset_jd, true)
        } else {
            (self.sunrise_jd, self.sunset_jd, false)
        }
    }
}

/// Where a measurement puts each chart: at sea level, as the baseline
/// reckons, or at its recorded height, where the air thins.
#[derive(Clone, Copy, PartialEq)]
enum Where {
    SeaLevel,
    AtHeight,
}

impl Measured {
    /// What one convention gave, printed for the measured pages.
    fn report(&self, horizon: Horizon, charts: usize) {
        println!(
            "{horizon}: compared {} events over {} charts; skipped {:?}; the baseline's sunrise block a day early for {:?}",
            self.compared, charts, self.skipped, self.day_early
        );
        println!(
            "{horizon}: worst SDK solver against the baseline: {}",
            self.sdk
        );
        println!(
            "{horizon}: worst native search against the baseline: {}",
            self.native
        );
        println!(
            "{horizon}: worst SDK solver against the native search: {}",
            self.between
        );
    }

    /// The median and the largest size of the SDK - native gaps over the
    /// charts at or above a height, seconds.
    fn between_from(&self, metres: f64) -> (f64, f64) {
        let mut sizes: Vec<f64> = self
            .gaps
            .iter()
            .filter(|(altitude, _)| *altitude >= metres)
            .map(|(_, gap)| gap.abs())
            .collect();
        sizes.sort_by(f64::total_cmp);
        (sizes[sizes.len() / 2], sizes[sizes.len() - 1])
    }
}

/// Every chart's day arc under a horizon, from the SDK's solver and from
/// Teimeris's own search, against what the baseline recorded (at sea
/// level) and against each other.
fn measure(provider: &TeimerisProvider, horizon: Horizon, at: Where) -> Measured {
    let completion = Completion::new(
        provider,
        OverridePolicy::PreferNative,
        DeltaTModel::TableThenModel,
    );
    let charts = charts();
    assert_eq!(charts.len(), 55);
    let mut measured = Measured {
        sdk: Worst::default(),
        native: Worst::default(),
        between: Worst::default(),
        gaps: Vec::new(),
        compared: 0,
        skipped: Vec::new(),
        day_early: Vec::new(),
    };
    for chart in &charts {
        let place = chart.at(at);
        let solver = Solver::new(
            &completion,
            Body::Sun,
            place,
            horizon,
            DeltaTModel::TableThenModel,
        );
        let midnight = JulianDay::<Ut1>::try_new(chart.midnight_jd).unwrap();
        let (expected_rise, expected_set, day_early) = chart.expected_arc();
        if day_early {
            measured.day_early.push(chart.id.clone());
        }
        let mut found = true;
        // The day's arc: the sunrise of the civil day, then the sunset that
        // follows it, as the baseline and the panchanga reckon.
        let mut from = midnight;
        for (kind, expected) in [
            (HorizonEventKind::Rise, expected_rise),
            (HorizonEventKind::Set, expected_set),
        ] {
            let sdk = solver
                .event(kind, from, 1.0)
                .unwrap_or_else(|e| panic!("{}: {e}", chart.id));
            let native = provider
                .horizon_event(&HorizonRequest {
                    body: Body::Sun,
                    kind,
                    place,
                    from,
                    window_days: 1.0,
                    horizon,
                })
                .unwrap_or_else(|e| panic!("{}: {e}", chart.id));
            let (Some(sdk), Some(native)) = (sdk, native) else {
                found = false;
                continue;
            };
            from = sdk.instant;
            measured.compared += 1;
            let label = format!(
                "{} {kind} ({} m{})",
                chart.id,
                chart.altitude_m,
                if at == Where::SeaLevel {
                    ", reckoned at sea level"
                } else {
                    ""
                }
            );
            let gap_sdk = (sdk.instant.get() - expected) * 86_400.0;
            let gap_native = (native.get() - expected) * 86_400.0;
            let gap_between = (sdk.instant.get() - native.get()) * 86_400.0;
            println!(
                "{horizon} {label}: SDK - baseline {gap_sdk:+.2} s, native - baseline {gap_native:+.2} s, SDK - native {gap_between:+.2} s"
            );
            measured.sdk.offer(gap_sdk, &label);
            measured.native.offer(gap_native, &label);
            measured.between.offer(gap_between, &label);
            measured.gaps.push((chart.altitude_m, gap_between));
        }
        if !found {
            measured.skipped.push(format!(
                "{}{}",
                chart.id,
                if chart.polar { " (polar)" } else { "" }
            ));
        }
    }
    measured.report(horizon, charts.len());
    measured
}

#[test]
fn the_solver_and_the_native_search_reproduce_the_baselines_sunrise() {
    let provider = TeimerisProvider::open(&data_dir_from_env()).unwrap_or_else(|e| panic!("{e}"));
    // The baseline reckons its sunrise at sea level under the almanac's
    // convention, which is what the SDK's solver reproduces.
    let almanac = measure(&provider, Horizon::UPPER_LIMB_REFRACTION, Where::SeaLevel);
    assert_eq!(almanac.day_early, vec!["c022", "c025", "c039"]);
    assert!(almanac.between.seconds < 30.0, "{}", almanac.between);
    assert!(almanac.sdk.seconds < 12.0, "{}", almanac.sdk);
    // Why a chart over a modern engine keeps the SDK's day (QUESTIONS.md
    // Q40): against the Swiss-based recording the solver is the closer of
    // the two, 9.77 s at worst against the engine's own 32.39 s (measured
    // 2026-09-26). Should the engine come to lead, the decision reopens.
    assert!(
        almanac.sdk.seconds < almanac.native.seconds,
        "the engine's own sunrise now reproduces the recording better than the SDK's solver \
         ({} against {}): reopen QUESTIONS.md Q40",
        almanac.native,
        almanac.sdk
    );
}

/// At each chart's own height the engine's standard air thins and the
/// almanac's fixed 34′ does not, so the two part by up to a minute; the
/// same air named on both sides brings them back to their models'
/// difference (`03-design/horizon-atmosphere.md`). The baseline, reckoned
/// at sea level, is not the reference here: the engine is.
#[test]
fn naming_the_air_brings_the_solver_to_the_engine_at_every_height() {
    let provider = TeimerisProvider::open(&data_dir_from_env()).unwrap_or_else(|e| panic!("{e}"));
    let fixed = measure(&provider, Horizon::UPPER_LIMB_REFRACTION, Where::AtHeight);
    let named = measure(
        &provider,
        Horizon::from_convention(SunriseConvention::Atmospheric {
            which: Sunrise::UpperLimbRefraction,
            air: Atmosphere::STANDARD,
        }),
        Where::AtHeight,
    );
    assert_eq!(named.compared, fixed.compared);
    // Measured 2026-09-26 over the 40 events of the 20 charts at or above
    // 1000 m: the fixed 34′ is 29.9 s from the engine in the median and
    // 61.8 s at worst (La Paz, 3640 m); the named air 3.7 s and 5.8 s,
    // which is Bennett's scaling against Sinclair's, growing as the air
    // thins. Both bounds are held, so neither can pass by the other
    // failing to move.
    let (fixed_median, fixed_worst) = fixed.between_from(1000.0);
    let (named_median, named_worst) = named.between_from(1000.0);
    println!(
        "SDK - native at or above 1000 m: the almanac's 34′ {fixed_median:.2} s median, {fixed_worst:.2} s worst; the air named {named_median:.2} s, {named_worst:.2} s"
    );
    assert!(
        fixed_median > 20.0,
        "the fixed 34′ no longer parts from the engine at height: {fixed_median:.2} s"
    );
    assert!(
        named_worst < 8.0,
        "the named air parts from the engine by {named_worst:.2} s at height"
    );
}
