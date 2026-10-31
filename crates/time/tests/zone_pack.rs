//! A zone pack answers as the database it was written from, and is refused
//! by name for everything else (`docs/03-design/zone-packs.md` §6).

#![cfg(feature = "signed-packs")]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    missing_docs,
    reason = "a test fails by panicking"
)]

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use ed25519_dalek::{Signer as _, SigningKey};
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::time::LocalClock as _;
use teistro_time::{EmbeddedTzdb, LocalSeconds, TimeZoneProvider, ZONE_PACK_FORMAT, ZonePack};

/// The embedded data, as a pack.
fn whole() -> Vec<u8> {
    let zones: Vec<(&str, &[u8])> = jiff_tzdb::available().filter_map(jiff_tzdb::get).collect();
    ZonePack::write(EmbeddedTzdb::bundled_version(), zones)
}

fn tzif(name: &str) -> &'static [u8] {
    jiff_tzdb::get(name).expect("a zone").1
}

fn one(version: &str) -> Vec<u8> {
    ZonePack::write(version, [("Asia/Kathmandu", tzif("Asia/Kathmandu"))])
}

fn pack_of(json: &str) -> Result<ZonePack, teistro_core::error::Error> {
    ZonePack::from_bytes(json.as_bytes())
}

fn field(json: &str) -> Option<String> {
    pack_of(json).unwrap_err().field().map(str::to_owned)
}

#[test]
fn the_embedded_data_as_a_pack_answers_as_the_embedded_database() {
    let embedded = EmbeddedTzdb::shared();
    let pack = ZonePack::from_bytes(&whole()).unwrap();
    assert_eq!(pack.version(), embedded.version());
    let (mut ours, mut theirs) = (pack.zones(), embedded.zones());
    ours.sort();
    theirs.sort();
    assert_eq!(ours, theirs);
    for zone in &theirs {
        // 1 January of every 25th year from 1800 to 2100, and mid-year.
        for year in (0..=12).map(|step| 1800.0 + 25.0 * f64::from(step)) {
            for shift in [0.0, 182.0] {
                let at = (year - 2000.0).mul_add(365.2425, 2_451_544.5) + shift;
                let instant = JulianDay::<Utc>::try_new(at).unwrap();
                assert_eq!(
                    pack.offset_at(zone, instant).unwrap(),
                    embedded.offset_at(zone, instant).unwrap(),
                    "{zone} at {at}"
                );
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "whole seconds of a date near the present"
                )]
                let local = LocalSeconds {
                    seconds: ((at - JulianDay::<Utc>::UNIX_EPOCH.get()) * 86_400.0) as i64,
                    nanos: 0,
                };
                assert_eq!(
                    pack.candidates(zone, local).unwrap(),
                    embedded.candidates(zone, local).unwrap(),
                    "{zone} at {at}"
                );
            }
        }
        assert_eq!(
            pack.current_offsets(zone).unwrap(),
            embedded.current_offsets(zone).unwrap(),
            "{zone}"
        );
    }
}

#[test]
fn a_name_reads_as_the_embedded_database_reads_it() {
    let pack = ZonePack::from_bytes(&one("2026f")).unwrap();
    assert!(pack.has_zone("asia/kathmandu"));
    let unknown = pack
        .offset_at("Asia/Kathmandoo", JulianDay::J2000)
        .unwrap_err();
    assert_eq!(unknown.hint(), Some("did you mean Asia/Kathmandu?"));
    assert_eq!(unknown.field(), Some("zone"));
    assert!(pack.clock("Asia/Tokyo").is_err());
    let clock = pack.clock("Asia/Kathmandu").unwrap();
    assert_eq!(clock.describe(), "Asia/Kathmandu (tzdb 2026f)");
    assert_eq!(clock.offset_at(JulianDay::J2000).seconds(), 20_700);
}

#[test]
fn identical_files_are_held_once() {
    let utc = tzif("UTC");
    let bytes = ZonePack::write("2026f", [("UTC", utc), ("Etc/UTC", utc), ("Zulu", utc)]);
    let text = String::from_utf8(bytes.clone()).unwrap();
    assert!(
        text.contains(r#""links":{"UTC":"Etc/UTC","Zulu":"Etc/UTC"}"#),
        "{text}"
    );
    assert_eq!(ZonePack::from_bytes(&bytes).unwrap().zones().len(), 3);
}

#[test]
fn each_part_of_a_pack_is_refused_by_name() {
    let utc = STANDARD.encode(tzif("UTC"));
    let f = ZONE_PACK_FORMAT;
    let zone = format!(r#""zones":{{"UTC":"{utc}"}}"#);
    assert_eq!(
        field(&format!(r#"{{"format":"x","version":"2026f",{zone}}}"#)).as_deref(),
        Some("format")
    );
    for version in ["2026", "26f", "2026F", "20a6f", "", "2026f-1"] {
        assert_eq!(
            field(&format!(
                r#"{{"format":"{f}","version":"{version}",{zone}}}"#
            ))
            .as_deref(),
            Some("version"),
            "{version}"
        );
    }
    assert_eq!(
        field(&format!(
            r#"{{"format":"{f}","version":"2026f","zones":{{}}}}"#
        ))
        .as_deref(),
        Some("zones")
    );
    for name in ["", "/etc", "a/", "a//b", "a b", "../x"] {
        assert_eq!(
            field(&format!(
                r#"{{"format":"{f}","version":"2026f","zones":{{"{name}":"{utc}"}}}}"#
            ))
            .as_deref(),
            Some(format!("zones.{name}").as_str()),
            "{name}"
        );
    }
    let bad = |value: &str| {
        format!(r#"{{"format":"{f}","version":"2026f","zones":{{"UTC":"{value}"}}}}"#)
    };
    assert_eq!(field(&bad("not base64!")).as_deref(), Some("zones.UTC"));
    assert_eq!(field(&bad("AAAA")).as_deref(), Some("zones.UTC"));
    let twice =
        format!(r#"{{"format":"{f}","version":"2026f","zones":{{"UTC":"{utc}","utc":"{utc}"}}}}"#);
    assert_eq!(field(&twice).as_deref(), Some("zones.utc"));
    let links =
        |body: &str| format!(r#"{{"format":"{f}","version":"2026f",{zone},"links":{{{body}}}}}"#);
    assert_eq!(
        field(&links(r#""A":"Nowhere""#)).as_deref(),
        Some("links.A")
    );
    assert_eq!(
        field(&links(r#""A":"UTC","a":"UTC""#)).as_deref(),
        Some("links.a")
    );
    assert_eq!(
        field(&links(r#""A b":"UTC""#)).as_deref(),
        Some("links.A b")
    );
    assert_eq!(
        field(&links(r#""B":"UTC","Z":"B""#)).as_deref(),
        Some("links.Z")
    );
    assert_eq!(
        ZonePack::from_bytes(&[0xff]).unwrap_err().field(),
        Some("pack")
    );
    assert_eq!(field("[]").as_deref(), Some("pack"));
    let vast = vec![b' '; 16 * 1024 * 1024 + 1];
    let limit = ZonePack::from_bytes(&vast).unwrap_err();
    assert_eq!(
        (limit.field(), limit.status),
        (Some("pack"), teistro_core::Status::Limit)
    );
}

#[test]
fn a_signed_pack_is_read_and_a_bad_one_refused_by_name() {
    let key = SigningKey::from_bytes(&[7; 32]);
    let bytes = one("2026f");
    let signature = key.sign(&bytes).to_bytes();
    let public = key.verifying_key().to_bytes();
    let read = ZonePack::from_signed(&bytes, &signature, &public, "2026f").unwrap();
    assert_eq!(read.version(), "2026f");
    assert!(ZonePack::from_signed(&bytes, &signature, &public, "").is_ok());
    let refused = |bytes: &[u8], signature: &[u8], key: &[u8], at_least: &str| {
        ZonePack::from_signed(bytes, signature, key, at_least)
            .unwrap_err()
            .field()
            .map(str::to_owned)
    };
    let mut altered = bytes.clone();
    altered[20] ^= 1;
    assert_eq!(
        refused(&altered, &signature, &public, "").as_deref(),
        Some("signature")
    );
    let other = SigningKey::from_bytes(&[8; 32]).verifying_key().to_bytes();
    assert_eq!(
        refused(&bytes, &signature, &other, "").as_deref(),
        Some("signature")
    );
    assert_eq!(
        refused(&bytes, &[0; 3], &public, "").as_deref(),
        Some("signature")
    );
    assert_eq!(
        refused(&bytes, &signature, &[0; 3], "").as_deref(),
        Some("key")
    );
    assert_eq!(
        refused(&bytes, &signature, &public, "2026g").as_deref(),
        Some("version")
    );
    assert_eq!(
        refused(&bytes, &signature, &public, "2026za").as_deref(),
        Some("version")
    );
    assert_eq!(
        refused(&bytes, &signature, &public, "newest").as_deref(),
        Some("at_least")
    );
    // A pack that verifies and is no pack is still refused.
    let junk = b"{}";
    assert_eq!(
        refused(junk, &key.sign(junk).to_bytes(), &public, "").as_deref(),
        Some("pack")
    );
}
