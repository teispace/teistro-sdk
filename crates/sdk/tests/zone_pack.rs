//! A signed zone pack replaces a context's database for every path that
//! reads a zone (`03-design/zone-packs.md` §6).

#![cfg(feature = "signed-packs")]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    missing_docs,
    reason = "a test fails by panicking"
)]

use std::sync::Arc;

use ed25519_dalek::{Signer as _, SigningKey};
use teistro::catalogue::Calendar;
use teistro::{CalendarDate, Context, ZoneDatabaseSource, ZonePack};
use teistro_core::quantity::{JulianDay, Utc};
use teistro_time::{CivilDateTime, CivilTime, ZoneSource, ZoneSpec};

/// Tokyo's rules under Kathmandu's name: a release that moved a clock.
fn moved(version: &str) -> Vec<u8> {
    let tokyo = jiff_tzdb::get("Asia/Tokyo").expect("a zone").1;
    ZonePack::write(version, [("Asia/Kathmandu", tokyo)])
}

fn noon() -> CivilDateTime {
    CivilDateTime::at(
        CalendarDate::defined(Calendar::Gregorian, 2026, 11, 1),
        CivilTime::NOON,
    )
}

#[test]
fn a_loaded_pack_answers_every_resolution_and_names_its_release() {
    let sdk = Context::builder().build().unwrap();
    let zone = ZoneSpec::iana("Asia/Kathmandu");
    let before = sdk.time().resolve(&noon(), &zone).unwrap();
    assert_eq!(before.zone.offset.seconds(), 20_700);

    let key = SigningKey::from_bytes(&[7; 32]);
    let bytes = moved("2099a");
    let signature = key.sign(&bytes).to_bytes();
    let public = key.verifying_key().to_bytes();
    let database = sdk
        .time()
        .load_zones(&bytes, &signature, &public, "2026a")
        .unwrap();
    assert_eq!(database.source, ZoneDatabaseSource::Pack);
    assert_eq!((database.version.as_str(), database.zones), ("2099a", 1));

    let after = sdk.time().resolve(&noon(), &zone).unwrap();
    assert_eq!(after.zone.offset.seconds(), 32_400);
    assert_eq!(after.zone.tzdb_version, "2099a");
    assert_eq!(after.zone.source, ZoneSource::Iana);
    let (_, read) = sdk
        .time()
        .civil_of(
            JulianDay::<Utc>::try_new(2_461_345.5).unwrap(),
            &zone,
            Calendar::Gregorian,
        )
        .unwrap();
    assert_eq!(read.offset.seconds(), 32_400);

    // What the pack left out is refused, not answered from the fallback.
    let other = ZoneSpec::iana("Europe/London");
    assert_eq!(
        sdk.time().resolve(&noon(), &other).unwrap_err().field(),
        Some("zone")
    );

    // A refused pack leaves the context as it was.
    let older = moved("2098a");
    let replayed = key.sign(&older).to_bytes();
    let refused = sdk
        .time()
        .load_zones(&older, &replayed, &public, "2099a")
        .unwrap_err();
    assert_eq!(refused.field(), Some("version"));
    assert_eq!(sdk.time().zone_database().version, "2099a");
}

#[test]
fn a_built_context_shares_a_pack_and_another_stays_on_the_fallback() {
    let pack = Arc::new(ZonePack::from_bytes(&moved("2099a")).unwrap());
    let shared = Context::builder().zones(Arc::clone(&pack)).build().unwrap();
    let worker = Context::builder().zones(pack).build().unwrap();
    let zone = ZoneSpec::iana("Asia/Kathmandu");
    for context in [&shared, &worker] {
        assert_eq!(
            context
                .time()
                .resolve(&noon(), &zone)
                .unwrap()
                .zone
                .offset
                .seconds(),
            32_400
        );
    }
    let plain = Context::builder().build().unwrap();
    let database = plain.time().zone_database();
    assert_eq!(database.source, ZoneDatabaseSource::Embedded);
    assert_eq!(
        plain
            .time()
            .resolve(&noon(), &zone)
            .unwrap()
            .zone
            .offset
            .seconds(),
        20_700
    );
}
