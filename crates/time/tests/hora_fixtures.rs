//! The planetary hours against the baseline's fixtures: the hora lord of
//! each chart's birth under the proportional reckoning, from the
//! panchanga day the baseline recorded (its sunrise, sunset and the next
//! sunrise), with the two charts the baseline's own conventions decide
//! named (`05-testing/01-golden-vectors.md`, conventions three and twelve).

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "a test fails by panicking and reads its fixtures by key"
)]

use std::path::Path;

use teistro_calendar::CalendarDate;
use teistro_core::catalogue::{Calendar, Graha, Vara};
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_core::settings::Sunrise;
use teistro_time::hora::{Reckoning, hora_at};
use teistro_time::local_day::{DayState, LocalDay};

/// The fixtures whose hora the baseline reckoned from a day of its own
/// making: c022 and c039, whose sunrise blocks are a day early so the
/// birth falls outside the day (convention twelve), and c028, a polar day
/// it synthesised (convention three).
const BASELINE_DECIDES: [&str; 3] = ["c022", "c028", "c039"];

fn charts() -> Vec<(String, LocalDay, JulianDay<Utc>, Graha)> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/baseline/charts");
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&dir).expect("the fixtures directory") {
        let path = entry.expect("an entry").path();
        if path.extension().is_none_or(|e| e != "json") {
            continue;
        }
        let value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).expect("readable")).expect("json");
        let foundation = &value["foundation"];
        let rise = foundation["lagna_sunrise_jd"].as_f64().expect("a sunrise");
        // The block whose sunrise anchors the panchanga day, and the one
        // after it for the next sunrise.
        let blocks = ["previous_day", "sunrise", "next_day"];
        let index = blocks
            .iter()
            .position(|b| (foundation[b]["sunrise_jd"].as_f64().unwrap() - rise).abs() < 1e-6)
            .expect("the anchoring block");
        let sunset = foundation[blocks[index]]["sunset_jd"].as_f64().unwrap();
        let next_sunrise = blocks.get(index + 1).map_or(rise + 1.0, |b| {
            foundation[b]["sunrise_jd"].as_f64().unwrap()
        });
        // The baseline's weekday numbers Monday 0 to Sunday 6.
        let swe = foundation["panchanga_day"]["weekday_swe"].as_u64().unwrap();
        let vara = Vara::from_id(u16::try_from((swe + 1) % 7).unwrap()).unwrap();
        let place = &value["input"]["place"];
        let day = LocalDay {
            place: Place::try_from_degrees(
                place["latitude"].as_f64().unwrap(),
                place["longitude"].as_f64().unwrap(),
                place["altitude_m"].as_f64().unwrap_or(0.0),
            )
            .unwrap(),
            date: CalendarDate::defined(Calendar::Gregorian, 2000, 1, 1),
            vara,
            sunrise: JulianDay::try_new(rise).unwrap(),
            sunset: JulianDay::try_new(sunset).unwrap(),
            next_sunrise: JulianDay::try_new(next_sunrise).unwrap(),
            state: DayState::Normal,
            convention: Sunrise::UpperLimbRefraction.into(),
            model: String::from("the baseline's fixture"),
        };
        let birth = JulianDay::try_new(foundation["jd_ut"].as_f64().unwrap()).unwrap();
        let lord: Graha = foundation["hora_lord"].as_str().unwrap().parse().unwrap();
        out.push((value["id"].as_str().unwrap().to_string(), day, birth, lord));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

#[test]
fn the_proportional_horas_reproduce_the_baselines_lords() {
    let charts = charts();
    assert_eq!(charts.len(), 55);
    let mut disagreed = Vec::new();
    let mut equal_agreed = 0;
    for (id, day, birth, expected) in &charts {
        // A birth outside the baseline's own day (its day-early block) is
        // one the baseline decided.
        match hora_at(day, *birth, Reckoning::Proportional) {
            Ok(hora) if hora.lord == *expected => {}
            _ => disagreed.push(id.clone()),
        }
        if hora_at(day, *birth, Reckoning::Equal).is_ok_and(|h| h.lord == *expected) {
            equal_agreed += 1;
        }
    }
    assert_eq!(disagreed, BASELINE_DECIDES, "{disagreed:?}");
    // The equal reckoning is not the baseline's: it agrees on fewer than
    // half the charts.
    assert!(equal_agreed < 30, "{equal_agreed}");
}
