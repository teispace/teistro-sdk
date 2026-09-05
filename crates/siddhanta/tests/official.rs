//! The national panchanga committee's printed places
//! (`fixtures/official/npns-2082-2083.json`) against the text: the Sun at
//! the printed sunrise within a few arcseconds; the Moon within an
//! arcminute once the apsis carries the bija the committee's numbers
//! imply (four revolutions fewer in an age), at the printed sunrises and
//! at eight printed tithi ends; and the star planets, which the committee
//! does not take from the text, degrees away from it.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "tests fail by panicking and read a small fixture"
)]

use std::path::Path;

use serde_json::Value;
use teistro_calendar::gregorian::fixed_from_gregorian;
use teistro_core::catalogue::Graha;
use teistro_core::quantity::{JulianDay, Ut1};
use teistro_siddhanta::{Bija, Parameters, SuryaSiddhanta, Trig};

/// Nepal's clock, in days.
const NPT_DAYS: f64 = 5.75 / 24.0;

/// The bija the committee's Moon implies: its apsis makes four revolutions
/// fewer in an age than the text's 488 203 (`docs/calendars/bikram-sambat.md`,
/// R2). A measurement, not a citation: `Surya { bija: true }` stays refused.
const COMMITTEE_MOON_BIJA: Bija = Bija {
    moon_apsis: -4,
    moon: 0,
    moon_node: 0,
    mars: 0,
    mercury: 0,
    jupiter: 0,
    venus: 0,
    saturn: 0,
};

fn fixture() -> Value {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/official/npns-2082-2083.json");
    serde_json::from_str(&std::fs::read_to_string(path).expect("the committee fixture"))
        .expect("valid JSON")
}

/// `sign:degree:minute:second` to degrees.
fn printed_degrees(text: &str) -> f64 {
    let parts: Vec<f64> = text
        .split(':')
        .map(|p| p.parse::<f64>().expect("a number"))
        .collect();
    parts[0] * 30.0 + parts[1] + parts[2] / 60.0 + parts[3] / 3600.0
}

/// The UT Julian day of a Gregorian date and a Nepal clock time `HH:MM`.
fn instant(gregorian: &str, time: &str) -> JulianDay<Ut1> {
    let mut date = gregorian
        .split('-')
        .map(|p| p.parse::<i64>().expect("a date part"));
    let (year, month, day) = (
        date.next().unwrap() as i32,
        date.next().unwrap() as u8,
        date.next().unwrap() as u8,
    );
    let (h, m) = time.split_once(':').expect("HH:MM");
    let fraction = (h.parse::<f64>().unwrap() * 60.0 + m.parse::<f64>().unwrap()) / 1440.0;
    let midnight = fixed_from_gregorian(year, month, day)
        .jd_at_midnight()
        .expect("a Julian day")
        .get();
    JulianDay::<Ut1>::literal(midnight + fraction - NPT_DAYS)
}

fn arcminutes_apart(ours_deg: f64, theirs_deg: f64) -> f64 {
    ((ours_deg - theirs_deg + 540.0).rem_euclid(360.0) - 180.0) * 60.0
}

#[test]
fn the_committees_sun_is_the_texts_to_a_few_arcseconds() {
    let text = SuryaSiddhanta::text();
    let mut compared = 0;
    for row in fixture()["planets_at_sunrise"].as_array().unwrap() {
        let Some(sunrise) = row["sunrise"].as_str() else {
            continue;
        };
        let at = instant(row["gregorian"].as_str().unwrap(), sunrise);
        let ours = text.sun(at).longitude.get();
        let theirs = printed_degrees(row["positions"]["SUN"].as_str().unwrap());
        let arcsec = arcminutes_apart(ours, theirs) * 60.0;
        assert!(arcsec.abs() < 6.0, "{}: {arcsec:+.1}\"", row["gregorian"]);
        compared += 1;
    }
    assert_eq!(compared, 2);
}

#[test]
fn the_committees_moon_is_the_texts_with_four_revolutions_off_the_apsis() {
    let text = SuryaSiddhanta::text();
    let corrected = SuryaSiddhanta::new(
        Parameters::TEXT.with_bija(&COMMITTEE_MOON_BIJA),
        Trig::Table,
    );
    let fixture = fixture();
    // At the printed sunrises.
    for row in fixture["planets_at_sunrise"].as_array().unwrap() {
        let Some(sunrise) = row["sunrise"].as_str() else {
            continue;
        };
        let at = instant(row["gregorian"].as_str().unwrap(), sunrise);
        let theirs = printed_degrees(row["positions"]["MOON"].as_str().unwrap());
        let plain = arcminutes_apart(text.moon(at).longitude.get(), theirs);
        let with_bija = arcminutes_apart(corrected.moon(at).longitude.get(), theirs);
        assert!(
            with_bija.abs() < 1.0,
            "{}: {with_bija:+.2}'",
            row["gregorian"]
        );
        assert!(
            with_bija.abs() <= plain.abs(),
            "{}: the bija must not move the Moon away ({plain:+.2}' to {with_bija:+.2}')",
            row["gregorian"]
        );
    }
    // At the printed tithi ends: the Moon less the Sun is the tithi's
    // boundary within the arcminute a printed minute resolves.
    for end in fixture["tithi_ends"].as_array().unwrap() {
        let at = instant(
            end["gregorian"].as_str().unwrap(),
            end["time"].as_str().unwrap(),
        );
        let target = end["elongation_deg"].as_f64().unwrap();
        let elongation = (corrected.moon(at).longitude.get() - corrected.sun(at).longitude.get())
            .rem_euclid(360.0);
        let arcmin = arcminutes_apart(elongation, target);
        assert!(
            arcmin.abs() < 1.0,
            "{} {}: {arcmin:+.2}'",
            end["gregorian"],
            end["tithi"]
        );
    }
}

#[test]
fn the_committees_star_planets_are_not_the_texts() {
    // The committee prints modern positions for the five and the node
    // (`docs/calendars/bikram-sambat.md`, R2); the text's Saturn and node
    // are degrees away from them on every printed row.
    let text = SuryaSiddhanta::text();
    for row in fixture()["planets_at_sunrise"].as_array().unwrap() {
        let at = instant(
            row["gregorian"].as_str().unwrap(),
            row["sunrise"].as_str().unwrap_or("05:45"),
        );
        for (graha, key) in [(Graha::Saturn, "SATURN"), (Graha::Rahu, "RAHU")] {
            let ours = text
                .graha(graha, at)
                .expect("a graha the text models")
                .longitude
                .get();
            let theirs = printed_degrees(row["positions"][key].as_str().unwrap());
            let degrees = arcminutes_apart(ours, theirs) / 60.0;
            assert!(
                degrees.abs() > 3.0,
                "{} {key}: {degrees:+.2}°",
                row["gregorian"]
            );
        }
    }
}
