//! The rise and set solver against the baseline's fixtures: for each of
//! the 55 charts, the sunrise and sunset of the local civil day at the
//! place, from the SDK's solver over Teimeris's positions under the
//! almanac's convention (the upper limb with refraction, the baseline's
//! `apparent-refraction`) and from Teimeris's own search, compared with
//! what the baseline recorded (`fixtures/README.md`). Run by hand with the
//! engine present; the worst differences are published in
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
use teistro_core::quantity::{JulianDay, Place, Ut1};
use teistro_core::settings::OverridePolicy;
use teistro_ephemeris_teimeris::{TeimerisProvider, data_dir_from_env};
use teistro_port_ephemeris::{Body, EphemerisProvider, Horizon, HorizonEventKind, HorizonRequest};

/// One chart's sunrise facts as the baseline recorded them.
struct Chart {
    id: String,
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

#[test]
fn the_solver_and_the_native_search_reproduce_the_baselines_sunrise_at_sea_level() {
    let provider = TeimerisProvider::open(&data_dir_from_env()).unwrap_or_else(|e| panic!("{e}"));
    let completion = Completion::new(
        &provider,
        OverridePolicy::PreferNative,
        DeltaTModel::TableThenModel,
    );
    let charts = charts();
    assert_eq!(charts.len(), 55);
    let mut worst_sdk = (0.0f64, String::new());
    let mut worst_native = (0.0f64, String::new());
    let mut worst_sdk_vs_native = (0.0f64, String::new());
    let mut compared = 0;
    let mut skipped = Vec::new();
    let mut day_early = Vec::new();
    for chart in &charts {
        let solver = Solver::new(
            &completion,
            Body::Sun,
            chart.place,
            Horizon::UPPER_LIMB_REFRACTION,
            DeltaTModel::TableThenModel,
        );
        let midnight = JulianDay::<Ut1>::try_new(chart.midnight_jd).unwrap();
        // The baseline's sunrise block for three charts holds the previous
        // day's events (`fixtures/README.md`, convention twelve): compare
        // with its next-day block there and count them.
        let (mut expected_rise, mut expected_set) = (chart.sunrise_jd, chart.sunset_jd);
        if (chart.next_day_sunrise_jd - chart.midnight_jd) < 1.0
            && (chart.sunrise_jd - chart.midnight_jd) < 0.0
        {
            day_early.push(chart.id.clone());
            expected_rise = chart.next_day_sunrise_jd;
            expected_set = chart.next_day_sunset_jd;
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
                    place: chart.place,
                    from,
                    window_days: 1.0,
                    horizon: Horizon::UPPER_LIMB_REFRACTION,
                })
                .unwrap_or_else(|e| panic!("{}: {e}", chart.id));
            let (Some(sdk), Some(native)) = (sdk, native) else {
                found = false;
                continue;
            };
            from = sdk.instant;
            compared += 1;
            let label = format!("{} {kind} (altitude {} m)", chart.id, chart.altitude_m);
            let gap_sdk = (sdk.instant.get() - expected).abs() * 86_400.0;
            let gap_native = (native.get() - expected).abs() * 86_400.0;
            let gap_between = (sdk.instant.get() - native.get()).abs() * 86_400.0;
            println!(
                "{label}: SDK - baseline {:+.2} s, native - baseline {:+.2} s, SDK - native {:+.2} s",
                (sdk.instant.get() - expected) * 86_400.0,
                (native.get() - expected) * 86_400.0,
                (sdk.instant.get() - native.get()) * 86_400.0
            );
            if gap_sdk > worst_sdk.0 {
                worst_sdk = (gap_sdk, label.clone());
            }
            if gap_native > worst_native.0 {
                worst_native = (gap_native, label.clone());
            }
            if gap_between > worst_sdk_vs_native.0 {
                worst_sdk_vs_native = (gap_between, label);
            }
        }
        if !found {
            skipped.push(format!(
                "{}{}",
                chart.id,
                if chart.polar { " (polar)" } else { "" }
            ));
        }
    }
    println!(
        "compared {compared} events over {} charts; skipped {skipped:?}; the baseline's sunrise block a day early for {day_early:?}",
        charts.len()
    );
    assert_eq!(day_early, vec!["c022", "c025", "c039"]);
    println!(
        "worst SDK solver against the baseline: {:.2} s at {}",
        worst_sdk.0, worst_sdk.1
    );
    println!(
        "worst native search against the baseline: {:.2} s at {}",
        worst_native.0, worst_native.1
    );
    println!(
        "worst SDK solver against the native search: {:.2} s at {}",
        worst_sdk_vs_native.0, worst_sdk_vs_native.1
    );
    assert!(worst_sdk_vs_native.0 < 30.0, "{}", worst_sdk_vs_native.1);
    assert!(worst_sdk.0 < 12.0, "{}", worst_sdk.1);
}
