//! BPHS ch. 85's inauspicious births read from charts the SDK computes: an
//! hour at a time through twenty days, in which a Krishna Chaturdashi passes
//! through its six parts and the Sun enters Capricorn.
//!
//! The rules crate's corpus records no limb's ghatikas and no sankranti, so
//! the parts and the sankranti days are held here, where the chart carries
//! both.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use std::collections::BTreeSet;

use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::rules::shipped;
use teistro::{ChartRequest, Context, Ephemeris, RuleRequest, UtcOffset};

/// The pack's rules that a moment's limbs decide, read at each hour of the
/// twenty days from JD 2 451 545 (2000 January 1, noon) at Kathmandu: the
/// keys present at each.
fn hours() -> Vec<BTreeSet<String>> {
    let sdk = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap();
    let pack: Vec<_> = shipped::nabhasas()
        .iter()
        .filter(|rule| rule.category == "ashubha-janma")
        .cloned()
        .collect();
    assert_eq!(pack.len(), 22);
    let set = RuleRequest::shipped([])
        .with_rules(pack)
        .rule_set()
        .unwrap();
    let place = Place::new(
        Latitude::try_new(27.7172).unwrap(),
        Longitude::try_new(85.324).unwrap(),
        Altitude::try_new(0.0).unwrap(),
    );
    let instants: Vec<JulianDay<Utc>> = (0..20 * 24)
        .map(|hour| JulianDay::literal(2_451_545.0 + f64::from(hour) / 24.0))
        .collect();
    sdk.chart()
        .readings_with_rules(&instants, &ChartRequest::at(place, UtcOffset::UTC), &set)
        .unwrap()
        .value
        .iter()
        .map(|(_, reading)| {
            reading
                .present
                .iter()
                .map(|present| present.rule.key.clone())
                .collect()
        })
        .collect()
}

#[test]
fn chaturdashi_s_parts_follow_one_another_and_a_sankranti_names_its_day() {
    let hours = hours();
    let parts: Vec<u8> = hours
        .iter()
        .map(|present| {
            let held: Vec<u8> = (1..=6)
                .filter(|part| present.contains(&format!("BPHS_KRISHNA_CHATURDASHI_PART_{part}")))
                .collect();
            // Ch. 87's six parts divide the tithi: at most one holds.
            assert!(held.len() <= 1, "{held:?}");
            held.first().copied().unwrap_or(0)
        })
        .collect();
    // The Krishna Chaturdashi of 4 January walks its parts from the first
    // to the sixth without skipping or returning.
    let mut runs: Vec<Vec<u8>> = Vec::new();
    for (at, part) in parts.iter().enumerate() {
        if *part == 0 {
            continue;
        }
        if at == 0 || parts[at - 1] == 0 {
            runs.push(Vec::new());
        }
        let run = runs.last_mut().unwrap();
        if run.last() != Some(part) {
            run.push(*part);
        }
    }
    assert_eq!(runs.len(), 1, "{runs:?}");
    for run in &runs {
        assert_eq!(run.as_slice(), [1, 2, 3, 4, 5, 6]);
    }

    // The Sun enters Capricorn: the sankranti day holds one named
    // sankranti, the one its weekday names, and no other day does.
    let sankranti: Vec<&str> = hours
        .iter()
        .flat_map(|present| present.iter())
        .filter(|key| key.ends_with("_SANKRANTI"))
        .map(String::as_str)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    // The Sun entered sidereal Capricorn on Friday 14 January 2000 at
    // Kathmandu: ch. 90 v. 1 names a Friday's sankranti Mishra.
    assert_eq!(sankranti, ["BPHS_BIRTH_ON_THE_MISHRA_SANKRANTI"]);
    let days = hours
        .iter()
        .filter(|present| present.iter().any(|key| key.ends_with("_SANKRANTI")))
        .count();
    assert!(
        (20..=28).contains(&days),
        "{days} hours on a sankranti's day"
    );
    for present in &hours {
        assert!(
            present
                .iter()
                .filter(|key| key.ends_with("_SANKRANTI"))
                .count()
                <= 1
        );
    }

    // The Amavasya of 6 January passes too, about a day long.
    let amavasya = hours
        .iter()
        .filter(|present| present.contains("BPHS_BIRTH_ON_AMAVASYA"))
        .count();
    assert!(
        (18..=28).contains(&amavasya),
        "{amavasya} hours of Amavasya"
    );
}
