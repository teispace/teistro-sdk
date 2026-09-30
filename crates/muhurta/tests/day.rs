//! A day's clauses over an assembled almanac day.
//!
//! The provider is the analytic test sky, so nothing here compares an
//! instant with a published almanac. What is held is that the clauses
//! agree with the day they read: each lies inside it, each rejected limb
//! is one the rules name and the day ran, and the native's two balas
//! follow the spans they count.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index fixed lists"
)]

use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::precession::PrecessionModel;
use teistro_calendar::solar::drik::DrikSun;
use teistro_calendar::{CalendarDate, Gregorian};
use teistro_core::catalogue::{Ayanamsha, Calendar, Nakshatra, Rashi, Vara};
use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
use teistro_core::settings::{OverridePolicy, Profile, SettingsPatch, Sunrise};
use teistro_core::time::UtcOffset;
use teistro_muhurta::{Clause, ClauseKind, DayRules, Native, clauses};
use teistro_panchanga::{Almanac, Panchanga};
use teistro_port_ephemeris::test_provider::TestProvider;

/// A day of the plausible sky at Kathmandu.
fn day(year: i32, month: u8, day: u8) -> Panchanga {
    let provider = TestProvider;
    let resolved = Profile::shipped(teistro_core::settings::DEFAULT_PROFILE)
        .unwrap()
        .resolve(&SettingsPatch::default())
        .unwrap();
    let model = DrikSun::new(
        &provider,
        Ayanamsha::Lahiri,
        Sunrise::CentreNoRefraction.into(),
        OverridePolicy::PreferNative,
        DeltaTModel::TableThenModel,
    );
    let clock = UtcOffset::literal(5, 45, 0);
    let almanac = Almanac::new(
        &provider,
        &resolved,
        &model,
        &Gregorian,
        &clock,
        PrecessionModel::Vondrak2011,
        DeltaTModel::TableThenModel,
    );
    let place = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.3240),
        Altitude::literal(1400.0),
    );
    almanac
        .day(
            &CalendarDate::defined(Calendar::Gregorian, year, month, day),
            &place,
        )
        .unwrap()
        .value
}

fn native() -> Native {
    Native {
        star: Nakshatra::Pushya,
        moon_sign: Rashi::Cancer,
    }
}

#[test]
fn every_clause_lies_inside_the_day_it_reads() {
    for d in 15..=21 {
        let day = day(2024, 6, d);
        for clause in clauses(&day, Some(&native()), &DayRules::raman()) {
            assert!(
                clause.at.clipped_to(day.window) == Some(clause.at),
                "{clause:?} outside {:?}",
                day.window
            );
        }
    }
}

#[test]
fn a_rejected_limb_is_one_the_rules_name_and_the_day_ran() {
    let rules = DayRules::raman();
    for d in 15..=21 {
        let day = day(2024, 6, d);
        let found = clauses(&day, None, &rules);
        let expected = day
            .limbs
            .tithi
            .iter()
            .filter(|s| rules.tithis.contains(&s.member))
            .count()
            + day
                .limbs
                .nakshatra
                .iter()
                .filter(|s| rules.nakshatras.contains(&s.member))
                .count()
            + day
                .limbs
                .yoga
                .iter()
                .filter(|s| rules.yogas.contains(&s.member))
                .count()
            + day
                .limbs
                .karana
                .iter()
                .filter(|s| rules.karanas.contains(&s.member))
                .count();
        let limbs = found
            .iter()
            .filter(|c| {
                matches!(
                    c.kind,
                    ClauseKind::Tithi { .. }
                        | ClauseKind::Nakshatra { .. }
                        | ClauseKind::Yoga { .. }
                        | ClauseKind::Karana { .. }
                )
            })
            .count();
        assert_eq!(limbs, expected, "2024-06-{d}");
        assert!(
            found
                .iter()
                .all(|c| !c.favourable() || matches!(c.kind, ClauseKind::MuhurtaYoga { .. }))
        );
    }
}

#[test]
fn tuesday_is_rejected_for_the_whole_day_and_wednesday_is_not() {
    let rules = DayRules::raman();
    // 18 June 2024 was a Tuesday.
    let tuesday = day(2024, 6, 18);
    assert_eq!(tuesday.day.vara, Vara::Mangalavara);
    let vara: Vec<Clause> = clauses(&tuesday, None, &rules)
        .into_iter()
        .filter(|c| matches!(c.kind, ClauseKind::Vara { .. }))
        .collect();
    assert_eq!(vara.len(), 1);
    assert_eq!(vara[0].at, tuesday.window);
    let wednesday = day(2024, 6, 19);
    assert!(
        clauses(&wednesday, None, &rules)
            .iter()
            .all(|c| !matches!(c.kind, ClauseKind::Vara { .. }))
    );
}

#[test]
fn the_natives_balas_follow_the_spans_they_count() {
    let rules = DayRules::raman();
    let day = day(2024, 6, 20);
    let with = clauses(&day, Some(&native()), &rules);
    let without = clauses(&day, None, &rules);
    let tara = with
        .iter()
        .filter(|c| matches!(c.kind, ClauseKind::Tarabala { .. }))
        .count();
    let chandra = with
        .iter()
        .filter(|c| matches!(c.kind, ClauseKind::Chandrabala { .. }))
        .count();
    assert_eq!(tara, day.limbs.nakshatra.len());
    assert_eq!(chandra, day.moon.signs.len());
    assert_eq!(
        with.len(),
        without.len() + tara + chandra,
        "the native adds only the two balas"
    );
}

#[test]
fn a_clause_reads_back_from_its_json() {
    let day = day(2024, 6, 18);
    let found = clauses(&day, Some(&native()), &DayRules::raman());
    let text = serde_json::to_string(&found).unwrap();
    assert!(text.contains(r#""clause":"VARA""#), "{text}");
    let back: Vec<Clause> = serde_json::from_str(&text).unwrap();
    assert_eq!(back, found);
}
