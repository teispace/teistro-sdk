//! The provider over a real teigeodata pack (Nepal's country pack, named
//! by `TEISTRO_TEIGEODATA_PACKS`): Kathmandu resolves to its zone, its
//! province and a plausible height; a point no pack covers answers
//! nothing; the answers pass the port's own check.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]

mod common;

use teistro_core::error::Status;
use teistro_geo_teigeodata::{ENGINE_VERSION, NAME, TeigeodataGeo};
use teistro_port_geo::{GeoPoint, GeoProvider, resolve};

fn at(lat: f64, lon: f64) -> GeoPoint {
    GeoPoint::try_from_degrees(lat, lon).unwrap()
}

#[test]
fn kathmandu_resolves_to_its_zone_province_and_height() {
    let Some(geo) = common::provider("kathmandu") else {
        return;
    };
    let place = resolve(&geo, at(27.7172, 85.324)).unwrap();
    assert_eq!(place.zone.as_deref(), Some("Asia/Kathmandu"));

    // Coarse to fine: Nepal, then Bagmati Province, by their ISO codes.
    let codes: Vec<Option<&str>> = place.admin.iter().map(|u| u.code.as_deref()).collect();
    assert_eq!(codes.first(), Some(&Some("NP")), "{:?}", place.admin);
    assert_eq!(place.admin[0].level, 0);
    assert!(codes.contains(&Some("NP-P3")), "{:?}", place.admin);
    assert!(place.admin.iter().all(|unit| unit.name.is_some()));

    // The valley floor stands about 1,300 m above EGM2008.
    let height = place.height.as_ref().unwrap();
    assert!((1_200.0..1_450.0).contains(&height.metres), "{height:?}");
    assert_eq!(height.datum, "EGM2008");
    assert!(
        height.resolution_m > 0.0 && height.resolution_m < 1_000.0,
        "{height:?}"
    );
    assert_ne!(height.source, "");

    // Who answered, and from which data.
    assert_eq!(place.source.name, NAME);
    assert_eq!(place.source.version, ENGINE_VERSION);
    assert_eq!(&place.source, geo.source());
    assert_eq!(
        place.source.tzdb.as_deref(),
        geo.engine().tz_version(),
        "the tzdb release is the one the packs' rules state"
    );
    let layers = geo.engine().layers().len();
    assert_eq!(place.source.data_version.split(" + ").count(), layers);
    assert!(place.source.data_version.contains("np."));

    // Each question asked alone answers as the resolution did.
    let kathmandu = at(27.7172, 85.324);
    assert_eq!(geo.zone_at(kathmandu).unwrap(), place.zone);
    assert_eq!(geo.admin_at(kathmandu).unwrap(), place.admin);
    assert_eq!(geo.height_at(kathmandu).unwrap(), place.height);
}

#[test]
fn a_point_no_pack_covers_answers_nothing() {
    let Some(geo) = common::provider("nowhere") else {
        return;
    };
    // The middle of the Atlantic: no pack, no answer, and no refusal.
    let atlantic = at(0.0, -30.0);
    assert_eq!(geo.zone_at(atlantic).unwrap(), None);
    assert_eq!(geo.height_at(atlantic).unwrap(), None);
    assert_eq!(geo.admin_at(atlantic).unwrap(), Vec::new());
    let place = resolve(&geo, atlantic).unwrap();
    assert!(place.zone.is_none() && place.height.is_none() && place.admin.is_empty());
    assert_eq!(place.source, *geo.source());
}

#[test]
fn a_pack_that_does_not_open_is_named_by_its_place_in_the_list() {
    let missing = TeigeodataGeo::open(["/no/such/pack.tgd"]).unwrap_err();
    assert_eq!(
        (missing.status, missing.field()),
        (Status::Pack, Some("packs[0]"))
    );
    let garbage = TeigeodataGeo::from_bytes([vec![0_u8; 64]]).unwrap_err();
    assert_eq!(
        (garbage.status, garbage.field()),
        (Status::Pack, Some("packs[0]"))
    );
    let none = TeigeodataGeo::from_bytes(Vec::<Vec<u8>>::new()).unwrap_err();
    assert_eq!(
        (none.status, none.field()),
        (Status::InvalidArg, Some("packs"))
    );
    // The second of two is named as the second.
    if let Some(mut packs) = common::packs("second pack") {
        packs.truncate(1);
        packs.push("/no/such/pack.tgd".into());
        let second = TeigeodataGeo::open(&packs).unwrap_err();
        assert_eq!(second.field(), Some("packs[1]"));
    }
}

/// The version the source states is the engine's the lock resolved: a
/// tag moved in `Cargo.toml` without [`ENGINE_VERSION`] fails here.
#[test]
fn the_stated_version_is_the_locked_engines() {
    let lock = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.lock")).unwrap();
    let entry = lock
        .split("[[package]]")
        .find(|entry| entry.contains("name = \"teigeodata-engine\""))
        .unwrap();
    assert!(
        entry.contains(&format!("version = \"{ENGINE_VERSION}\"")),
        "{entry}"
    );
    assert!(
        entry.contains(&format!("?tag=v{ENGINE_VERSION}#")),
        "{entry}"
    );
}
