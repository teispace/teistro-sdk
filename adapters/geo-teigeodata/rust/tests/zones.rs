//! The packs' zone rules as the SDK's time-zone port: Kathmandu's offsets
//! across its history, New York's gap and overlap, names matched as the
//! SDK's own packs match them, an unknown name refused with suggestions,
//! and a context built over them reporting the provider as its database.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::print_stderr,
    reason = "tests fail by panicking and say on stderr what they skip"
)]

mod common;

use std::sync::Arc;

use teigeodata_engine::Civil;
use teistro::{Context, ZoneDatabaseSource};
use teistro_core::error::Status;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::time::UtcOffset;
use teistro_port_geo::GeoProvider;
use teistro_port_timezone::{LocalCandidates, LocalSeconds, TimeZoneProvider};

/// Kathmandu's rules begin when it left local mean time (+05:41:16) for
/// +05:30, at 1920-01-01 00:00 local mean time.
const KATHMANDU_RULES_START: i64 = -1_577_943_676;

/// The packs' rules: Nepal's country pack carries its own zone.
fn zones(test: &str) -> Option<Arc<dyn TimeZoneProvider>> {
    let geo = common::provider(test)?;
    Some(geo.time_zones().expect("Nepal's pack carries zone rules"))
}

/// The packs' rules when they hold the world's zones, as the baseline
/// pack (`baseline-lite.tgd`) does beside Nepal's; `None`, said on stderr,
/// when they hold only a country's (a failure under the required
/// variable, which CI sets with both packs named).
fn world(test: &str) -> Option<Arc<dyn TimeZoneProvider>> {
    let zones = zones(test)?;
    if zones.has_zone("America/New_York") {
        return Some(zones);
    }
    assert!(
        std::env::var_os(common::REQUIRED_VARIABLE).is_none(),
        "{test}: the packs hold {} zone(s), not the world's: name `baseline-lite.tgd` too",
        zones.zones().len()
    );
    eprintln!("SKIP  {test}: the packs hold no world zone rules (`baseline-lite.tgd`)");
    None
}

fn instant(unix_seconds: i64) -> JulianDay<Utc> {
    #[allow(
        clippy::cast_precision_loss,
        reason = "second counts in these tests are far below 2^53"
    )]
    let days = unix_seconds as f64 / 86_400.0;
    JulianDay::try_new(JulianDay::<Utc>::UNIX_EPOCH.get() + days).unwrap()
}

fn civil(year: i32, month: u8, day: u8, hour: u8, minute: u8) -> i64 {
    Civil {
        year,
        month,
        day,
        hour,
        minute,
        second: 0,
    }
    .seconds()
    .unwrap()
}

fn local(year: i32, month: u8, day: u8, hour: u8, minute: u8) -> LocalSeconds {
    LocalSeconds {
        seconds: civil(year, month, day, hour, minute),
        nanos: 0,
    }
}

#[test]
fn the_rules_list_every_zone_with_the_packs_release() {
    let Some(zones) = world("the_rules_list_every_zone_with_the_packs_release") else {
        return;
    };
    let names = zones.zones();
    assert!(names.len() > 300, "{} zones", names.len());
    assert!(names.is_sorted_by(|a, b| a < b), "byte order, each once");
    assert!(names.iter().any(|name| name == "Asia/Kathmandu"));
    assert!(
        zones.version().starts_with("20"),
        "a tzdb release, not {}",
        zones.version()
    );
}

#[test]
fn kathmandu_keeps_its_history() {
    let Some(zones) = zones("kathmandu_keeps_its_history") else {
        return;
    };
    let today = zones
        .offset_at("Asia/Kathmandu", instant(civil(2026, 11, 13, 0, 0)))
        .unwrap();
    assert_eq!(today.offset, UtcOffset::literal(5, 45, 0));
    assert!(!today.is_dst && !today.before_rules, "{today:?}");

    let indian = zones
        .offset_at("Asia/Kathmandu", instant(civil(1970, 1, 1, 0, 0)))
        .unwrap();
    assert_eq!(indian.offset, UtcOffset::literal(5, 30, 0));

    let first = zones
        .offset_at("Asia/Kathmandu", instant(KATHMANDU_RULES_START))
        .unwrap();
    assert!(!first.before_rules, "{first:?}");
    let mean = zones
        .offset_at("Asia/Kathmandu", instant(KATHMANDU_RULES_START - 1))
        .unwrap();
    assert!(mean.before_rules, "{mean:?}");
    assert_eq!(mean.offset, UtcOffset::literal(5, 41, 16));
}

#[test]
fn names_match_as_spelled_or_without_case() {
    let Some(zones) = zones("names_match_as_spelled_or_without_case") else {
        return;
    };
    assert!(zones.has_zone("Asia/Kathmandu"));
    assert!(zones.has_zone("asia/KATHMANDU"));
    let folded = zones
        .offset_at("asia/kathmandu", instant(civil(2026, 1, 1, 0, 0)))
        .unwrap();
    assert_eq!(folded.offset, UtcOffset::literal(5, 45, 0));
    assert_eq!(
        zones.current_offsets("ASIA/KATHMANDU").unwrap(),
        [UtcOffset::literal(5, 45, 0)]
    );
}

#[test]
fn an_unknown_zone_is_refused_with_the_nearest_names() {
    let Some(zones) = zones("an_unknown_zone_is_refused_with_the_nearest_names") else {
        return;
    };
    assert!(!zones.has_zone("Asia/Kathmando"));
    let why = zones.offset_at("Asia/Kathmando", instant(0)).unwrap_err();
    assert_eq!(why.status, Status::InvalidArg, "{why:?}");
    assert!(
        why.hint()
            .is_some_and(|hint| hint.contains("Asia/Kathmandu")),
        "{why:?}"
    );

    // A tzdb link is not a zone the packs hold.
    assert!(!zones.has_zone("Etc/UTC"));
    assert!(
        zones
            .candidates("Etc/UTC", local(2026, 1, 1, 0, 0))
            .is_err()
    );
}

#[test]
fn new_york_springs_forward_and_falls_back() {
    let Some(zones) = world("new_york_springs_forward_and_falls_back") else {
        return;
    };
    let east = UtcOffset::literal(-5, 0, 0);
    let summer = UtcOffset::literal(-4, 0, 0);
    let ny = "America/New_York";

    assert_eq!(
        zones.candidates(ny, local(2026, 3, 8, 2, 30)).unwrap(),
        LocalCandidates::Gap {
            before: east,
            after: summer
        }
    );
    assert_eq!(
        zones.candidates(ny, local(2026, 11, 1, 1, 30)).unwrap(),
        LocalCandidates::Overlap {
            earlier: summer,
            later: east
        }
    );
    assert_eq!(
        zones.candidates(ny, local(2026, 6, 1, 12, 0)).unwrap(),
        LocalCandidates::Unambiguous { offset: summer }
    );

    let july = zones
        .offset_at(ny, instant(civil(2026, 7, 1, 12, 0)))
        .unwrap();
    assert!(july.is_dst, "{july:?}");
    assert_eq!(july.abbreviation, "EDT");
    assert_eq!(zones.current_offsets(ny).unwrap(), [east, summer]);
}

#[test]
fn a_far_instant_is_out_of_the_rules_range() {
    let Some(zones) = zones("a_far_instant_is_out_of_the_rules_range") else {
        return;
    };
    let far = JulianDay::<Utc>::try_new(1.0e12).unwrap();
    let why = zones.offset_at("Asia/Kathmandu", far).unwrap_err();
    assert_eq!(why.status, Status::OutOfRange, "{why:?}");
}

#[test]
fn a_context_reckons_by_the_packs_rules() {
    let Some(zones) = zones("a_context_reckons_by_the_packs_rules") else {
        return;
    };
    let version = zones.version().to_owned();
    let count = zones.zones().len();
    let sdk = Context::builder().time_zones(zones).build().unwrap();
    let database = sdk.time().zone_database();
    assert_eq!(database.source, ZoneDatabaseSource::Provider);
    assert_eq!(database.version, version);
    assert_eq!(database.zones, count);
}
