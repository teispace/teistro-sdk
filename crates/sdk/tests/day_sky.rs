//! The day's sky through the façade (`03-design/day-sky.md`): every
//! angle against JPL Horizons' observer tables, which read DE441 and
//! share nothing with the SDK, and the section against the day it sits
//! beside: the same rises, the same paksha, the same days and hashes.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "tests fail by panicking, index what they asked for and print the measurement under --nocapture"
)]

use std::collections::BTreeMap;

use serde_json::Value;
use teistro::catalogue::{Calendar, Paksha};
use teistro::day_sky::{DaySky, SkyPoint, SkyPointKind, SkyState, SkyStep, day_sky};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Ut1};
use teistro::{AlmanacRequest, CalendarDate, Context, DaysRequest, Ephemeris, UtcOffset};
use teistro_astro::{Completion, DeltaTModel};
use teistro_core::angle::difference_deg;
use teistro_core::settings::OverridePolicy;
use teistro_port_ephemeris::Horizon;

/// The worst differences from Horizons, measured here and published on the
/// design page. Horizons' clock is UTC and the SDK reads it as UT1, up to
/// 0.4 s of the Earth's turn; its Moon is DE441's and the built-in's is
/// another fit.
const ANGLE_BOUND_DEG: f64 = 0.001;
/// The refracted altitude, above the horizon, where both models are fits
/// to one atmosphere.
const REFRACTED_BOUND_DEG: f64 = 0.001;
/// The fraction lit.
const FRACTION_BOUND: f64 = 1e-4;
/// The phase angle and the parallactic angle; the bright limb and the
/// zenith are ill-conditioned near new or full and overhead, so rows
/// further from those are compared.
const SHAPE_BOUND_DEG: f64 = 0.01;

fn context() -> Context {
    Context::builder()
        .profile("nepali-default")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

fn place(latitude: f64, longitude: f64, altitude: f64) -> Place {
    Place::new(
        Latitude::literal(latitude),
        Longitude::literal(longitude),
        Altitude::literal(altitude),
    )
}

fn kathmandu() -> Place {
    place(27.7172, 85.324, 1400.0)
}

fn nepal() -> UtcOffset {
    UtcOffset::literal(5, 45, 0)
}

fn date(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, year, month, day)
}

/// The sky of a short window opening at `jd`, whose first point is `jd`.
fn at(sdk: &Context, spot: Place, jd: f64) -> DaySky {
    let delta_t = DeltaTModel::TableThenModel;
    let source = Completion::new(sdk.ephemeris().unwrap(), OverridePolicy::SdkOnly, delta_t);
    let from = JulianDay::<Ut1>::literal(jd);
    let to = JulianDay::<Ut1>::literal(jd + 0.01);
    day_sky(
        &source,
        spot,
        Horizon::UPPER_LIMB_REFRACTION,
        delta_t,
        (from, to),
        SkyStep::of_minutes(240).unwrap(),
    )
    .unwrap()
}

#[test]
fn every_angle_agrees_with_jpl_horizons() {
    let table: Value =
        serde_json::from_str(include_str!("horizons-sky.json")).expect("the recorded table");
    let sdk = context();
    let mut worst: BTreeMap<&str, f64> = BTreeMap::new();
    let mut note = |name, value: f64| {
        let held = worst.entry(name).or_insert(0.0);
        *held = held.max(value.abs());
    };
    let rows = table["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 120);
    for row in rows {
        let site = &table["sites"][row["site"].as_str().unwrap()];
        let spot = place(
            site[0].as_f64().unwrap(),
            site[1].as_f64().unwrap(),
            site[2].as_f64().unwrap(),
        );
        let field = |name: &str| row[name].as_f64().unwrap();
        let sky = at(&sdk, spot, field("jd"));
        let moon = row["body"] == "moon";
        let point: &SkyPoint = if moon {
            &sky.moon.track[0]
        } else {
            &sky.sun.track[0]
        };
        note("altitude", point.altitude_deg - field("altitude"));
        // An azimuth runs fast near the zenith, and is held in the sky's
        // own arc, its cosine of altitude.
        note(
            "azimuth",
            difference_deg(point.azimuth_deg, field("azimuth"))
                * point.altitude_deg.to_radians().cos(),
        );
        if field("altitude") > 2.0 {
            note(
                "refracted",
                point.refracted_altitude_deg - field("refracted"),
            );
        }
        if let Some(shape) = point.moon {
            note(
                "fraction",
                shape.illuminated_fraction - field("illuminated"),
            );
            note("phase", shape.phase_angle_deg - field("phase"));
            // Away from new and full, where the limb's direction is sharp.
            if (20.0..160.0).contains(&field("phase")) {
                // `PsAng` is measured from the ICRF's north and the limb from
                // the true equator's, and the pole moves 20.04″ a year
                // towards 0h (Meeus, ch. 21).
                let years = (field("jd") - 2_451_545.0) / 365.25;
                let north = 20.04 / 3600.0 * years * field("ra").to_radians().sin()
                    / field("dec").to_radians().cos();
                note(
                    "limb",
                    difference_deg(shape.bright_limb_deg, field("radius_pa") + 180.0 + north),
                );
            }
            // The zenith and the pole are both well away from the Moon.
            let (hour_angle, dec, latitude) = (
                field("hour_angle_hours") * 15.0,
                field("dec"),
                site[0].as_f64().unwrap(),
            );
            let want = teistro_astro::phenomena::parallactic_angle_deg(hour_angle, dec, latitude);
            if point.altitude_deg < 80.0 {
                note(
                    "parallactic",
                    difference_deg(shape.parallactic_angle_deg, want),
                );
            }
        }
    }
    println!("{worst:?}");
    for (name, bound) in [
        ("altitude", ANGLE_BOUND_DEG),
        ("azimuth", ANGLE_BOUND_DEG),
        ("refracted", REFRACTED_BOUND_DEG),
        ("fraction", FRACTION_BOUND),
        ("phase", SHAPE_BOUND_DEG),
        ("limb", SHAPE_BOUND_DEG),
        ("parallactic", SHAPE_BOUND_DEG),
    ] {
        assert!(worst[name] < bound, "{name}: {}", worst[name]);
    }
}

/// Each day's section, over a range.
fn asked(
    sdk: &Context,
    from: &CalendarDate,
    to: &CalendarDate,
    spot: &Place,
) -> teistro::AlmanacAnswer {
    let request = AlmanacRequest::new().with_sky(SkyStep::default());
    sdk.almanac()
        .asked(from, to, spot, nepal(), &request)
        .unwrap()
}

#[test]
fn a_days_rises_and_sets_are_the_ones_the_day_found() {
    let sdk = context();
    let answer = asked(&sdk, &date(2026, 10, 1), &date(2026, 10, 31), &kathmandu());
    let days = &answer.days.value;
    let skies = &answer.sky.as_ref().unwrap().value;
    assert_eq!(days.len(), skies.len());
    let mut worst: f64 = 0.0;
    let near = |points: Vec<f64>, want: f64| {
        points
            .iter()
            .map(|at| (at - want).abs() * 86_400.0)
            .fold(f64::INFINITY, f64::min)
    };
    for (day, sky) in days.iter().zip(skies) {
        assert_eq!(sky.from.get(), day.window.from.get());
        assert_eq!(sky.to.get(), day.window.to.get());
        let times = |body: &teistro::day_sky::BodySky, kind| -> Vec<f64> {
            body.points(kind).map(|point| point.at.get()).collect()
        };
        worst = worst.max(near(
            times(&sky.sun, SkyPointKind::Rise),
            day.day.sunrise.get(),
        ));
        worst = worst.max(near(
            times(&sky.sun, SkyPointKind::Set),
            day.day.sunset.get(),
        ));
        for rise in &day.moon.rises {
            if (day.window.from.get()..day.window.to.get()).contains(&rise.get()) {
                worst = worst.max(near(times(&sky.moon, SkyPointKind::Rise), rise.get()));
            }
        }
        for set in &day.moon.sets {
            if (day.window.from.get()..day.window.to.get()).contains(&set.get()) {
                worst = worst.max(near(times(&sky.moon, SkyPointKind::Set), set.get()));
            }
        }
        // The Sun transits once a day, and high here.
        assert_eq!(sky.sun.points(SkyPointKind::Transit).count(), 1);
    }
    println!("worst event difference {worst} s");
    assert!(worst < 0.01, "{worst} s");
}

/// Horizons' own meridian passages at Kathmandu, where its hour angle at
/// one-minute steps crosses zero, interpolated: parallax has no part in
/// right ascension there, so the geocentric search is the place's
/// (crux C970), and the Moon stands 85° up, where a wrong one would show.
#[test]
fn a_transit_is_where_horizons_has_its_hour_angle_zero() {
    let sdk = context();
    let answer = asked(&sdk, &date(2026, 10, 31), &date(2026, 10, 31), &kathmandu());
    let sky = &answer.sky.unwrap().value[0];
    let only = |body: &teistro::day_sky::BodySky| {
        let mut transits = body.points(SkyPointKind::Transit);
        let found = transits.next().unwrap().at.get();
        assert!(transits.next().is_none());
        found
    };
    // 0.1 s: Horizons' clock is UTC and the SDK reads it as UT1.
    assert!((only(&sky.sun) - 2_461_344.751_613_6).abs() * 86_400.0 < 0.15);
    assert!((only(&sky.moon) - 2_461_345.483_957_5).abs() * 86_400.0 < 0.15);
}

#[test]
fn waxing_agrees_with_the_paksha_at_sunrise() {
    let sdk = context();
    let answer = asked(&sdk, &date(2026, 3, 1), &date(2026, 4, 30), &kathmandu());
    for (day, sky) in answer.days.value.iter().zip(&answer.sky.unwrap().value) {
        let waxing = sky.moon.track[0].moon.unwrap().waxing;
        // The window opens at the sunrise whose tithi names the paksha.
        let first = day.limbs.tithi[0].member.id();
        assert_eq!(waxing, first < 15, "{:?}", day.day.date);
        let _: Paksha = day.month.paksha;
    }
}

#[test]
fn a_full_moon_is_lit_and_a_new_one_dark() {
    let sdk = context();
    // Full on 2026-10-26 and new on 2026-11-09, near enough.
    let answer = asked(&sdk, &date(2026, 10, 26), &date(2026, 10, 26), &kathmandu());
    let full = answer.sky.unwrap().value[0].moon.track[0].moon.unwrap();
    assert!(full.illuminated_fraction > 0.97, "{full:?}");
    let answer = asked(&sdk, &date(2026, 11, 9), &date(2026, 11, 9), &kathmandu());
    let new = answer.sky.unwrap().value[0].moon.track[0].moon.unwrap();
    assert!(new.illuminated_fraction < 0.03, "{new:?}");
}

#[test]
fn a_polar_day_says_so_and_keeps_its_noon() {
    let sdk = context();
    let tromso = place(69.65, 18.96, 0.0);
    let answer = sdk
        .almanac()
        .asked(
            &date(2024, 6, 21),
            &date(2024, 6, 21),
            &tromso,
            UtcOffset::literal(2, 0, 0),
            &AlmanacRequest::new().with_sky(SkyStep::of_minutes(60).unwrap()),
        )
        .unwrap();
    let sky = &answer.sky.unwrap().value[0];
    assert_eq!(sky.sun.state, SkyState::AlwaysAbove);
    assert_eq!(sky.sun.points(SkyPointKind::Rise).count(), 0);
    assert_eq!(sky.sun.points(SkyPointKind::Set).count(), 0);
    let noon = sky.sun.points(SkyPointKind::Transit).next().unwrap();
    // Horizons: 43.79° at 10:47 UT.
    assert!((noon.altitude_deg - 43.79).abs() < 0.05, "{noon:?}");
    // The civil day asked, midnight to midnight on the offset, not the
    // nearest sunrise's the policy moved the day's own window to.
    assert!((sky.from.get() - (2_460_482.5 - 2.0 / 24.0)).abs() < 1e-9);
    assert!((sky.to.get() - sky.from.get() - 1.0).abs() < 1e-9);
    assert!(sky.sun.track.iter().all(|point| point.altitude_deg > 0.0));
}

#[test]
fn asking_leaves_the_days_and_their_hashes_alone() {
    let sdk = context();
    let (first, last) = (date(2026, 10, 1), date(2026, 10, 3));
    let plain = sdk
        .almanac()
        .asked(&first, &last, &kathmandu(), nepal(), &AlmanacRequest::new())
        .unwrap();
    let with = asked(&sdk, &first, &last, &kathmandu());
    assert_eq!(plain.days.value, with.days.value);
    assert_eq!(plain.day_hashes, with.day_hashes);
    assert!(plain.sky.is_none());
    assert!(plain.sections().unwrap().sky.is_none());
    let sky = with.sky.as_ref().unwrap();
    assert!(
        sky.provenance
            .applied_conventions
            .iter()
            .any(|applied| applied.knob == "sky.window")
    );
    assert!(with.sections().unwrap().sky.is_some());
}

#[test]
fn a_sky_asked_without_days_still_answers_and_a_zone_too() {
    let sdk = context();
    let day = date(2026, 10, 31);
    let request = AlmanacRequest::new()
        .with_sky(SkyStep::default())
        .without_days();
    let alone = sdk
        .almanac()
        .asked(&day, &day, &kathmandu(), nepal(), &request)
        .unwrap();
    assert_eq!(alone.days.value.len(), 0);
    assert_eq!(alone.sky.unwrap().value.len(), 1);
}

#[test]
fn a_context_without_an_ephemeris_is_refused() {
    let sdk = Context::builder().build().unwrap();
    let day = date(2026, 10, 31);
    assert!(
        sdk.almanac()
            .asked(
                &day,
                &day,
                &kathmandu(),
                nepal(),
                &AlmanacRequest::new().with_sky(SkyStep::default()),
            )
            .is_err()
    );
}

#[test]
fn the_json_request_reads_the_flag_and_its_step() {
    let base = r#""first": {"year": 2026, "month": 10, "day": 10}, "latitudeDeg": 27.7, "longitudeDeg": 85.3, "utcOffsetSeconds": 20700"#;
    let read = |rest: &str| DaysRequest::from_json(&format!("{{{base}{rest}}}"));
    assert_eq!(read("").unwrap().beside.sky(), None);
    assert_eq!(
        read(r#", "sky": true"#).unwrap().beside.sky(),
        Some(SkyStep::default())
    );
    assert_eq!(
        read(r#", "sky": true, "skyStepMinutes": 5"#)
            .unwrap()
            .beside
            .sky(),
        Some(SkyStep::of_minutes(5).unwrap())
    );
    let refused = read(r#", "skyStepMinutes": 5"#).unwrap_err();
    assert_eq!(refused.field(), Some("skyStepMinutes"));
    assert!(read(r#", "sky": true, "skyStepMinutes": 0"#).is_err());
}

/// What the flag costs: timed, not gated, since a test's clock is no gate.
/// `cargo test --release --test day_sky price -- --ignored --nocapture`.
#[test]
#[ignore = "a measurement, in release"]
fn price() {
    let sdk = context();
    let (first, last) = (date(2026, 1, 1), date(2026, 1, 31));
    let plain = AlmanacRequest::new();
    let with = plain.clone().with_sky(SkyStep::default());
    let time = |request: &AlmanacRequest| {
        let mut best = f64::INFINITY;
        for _ in 0..5 {
            let start = std::time::Instant::now();
            sdk.almanac()
                .asked(&first, &last, &kathmandu(), nepal(), request)
                .unwrap();
            best = best.min(start.elapsed().as_secs_f64());
        }
        best
    };
    let (off, on) = (time(&plain), time(&with));
    println!(
        "31 days: off {off:.4} s, on {on:.4} s, {:.1}%",
        (on / off - 1.0) * 100.0
    );
}
