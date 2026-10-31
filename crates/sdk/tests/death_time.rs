//! The death-time check on the sky at Kathmandu (`03-design/death-time.md`):
//! January 2026 read every three hours, so every reading below is one the
//! Moon made, and the texts' clauses are held to it.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the readings they took"
)]

use teistro::catalogue::{Nakshatra, Rashi};
use teistro::death_time::{DeathInstant, DeathRiteKind, DeathTime, DeathTimeTable, NakshatraFeet};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{Context, DeathTimeQuery, Ephemeris, Status, UtcOffset};

/// 2026 January 1, 00:00 UTC.
const JANUARY: f64 = 2_461_041.5;

fn context() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

fn kathmandu() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1337.0),
    )
}

fn check(sdk: &Context, at: f64, cremated_at: Option<f64>) -> DeathTime {
    sdk.almanac()
        .death_time(
            &DeathTimeTable::dharmasindhu(),
            JulianDay::<Utc>::try_new(at).unwrap(),
            cremated_at.map(|at| JulianDay::try_new(at).unwrap()),
            &kathmandu(),
            UtcOffset::literal(5, 45, 0),
        )
        .unwrap()
        .value
}

/// The month's readings, every three hours.
fn month(sdk: &Context) -> Vec<DeathInstant> {
    (0..31 * 8)
        .map(|step| check(sdk, JANUARY + f64::from(step) / 8.0, None).death)
        .collect()
}

/// The signs a nakshatra's thirteen and a third degrees lie in.
fn signs_of(nakshatra: Nakshatra) -> (Rashi, Rashi) {
    let index = Nakshatra::ALL
        .iter()
        .position(|&one| one == nakshatra)
        .unwrap();
    let start = 360.0 / 27.0 * f64::from(u8::try_from(index).unwrap());
    (
        Rashi::of_longitude(start),
        Rashi::of_longitude(start + 360.0 / 27.0 - 1e-6),
    )
}

#[test]
fn ds_334_the_panchaka_is_the_moons_stay_in_aquarius_and_pisces() {
    let sdk = context();
    let read = month(&sdk);
    let table = DeathTimeTable::dharmasindhu();
    for one in &read {
        // The nakshatra and the sign are one Moon.
        let (first, last) = signs_of(one.nakshatra);
        assert!(one.moon == first || one.moon == last, "{one:?}");
        assert_eq!(
            one.panchaka,
            matches!(one.moon, Rashi::Aquarius | Rashi::Pisces),
            "{one:?}"
        );
        assert_eq!(one.feet, table.feet(one.nakshatra), "{one:?}");
    }
    // A lunar month holds one panchaka: it opens in Dhanishtha's second
    // half (DS p. 334), never in its first, and closes with Revati.
    let inside: Vec<&DeathInstant> = read.iter().filter(|one| one.panchaka).collect();
    assert!(inside.len() >= 8 * 4, "about four and a half days");
    assert!(
        read.iter()
            .any(|one| one.nakshatra == Nakshatra::Dhanishtha && !one.panchaka),
        "Dhanishtha's first half is outside"
    );
    assert!(matches!(
        inside[0].nakshatra,
        Nakshatra::Dhanishtha | Nakshatra::Shatabhisha
    ));
    let opened = read.iter().position(|one| one.panchaka).unwrap();
    let closed = opened + read[opened..].iter().position(|one| !one.panchaka).unwrap();
    assert_eq!(read[closed - 1].nakshatra, Nakshatra::Revati);
    assert_eq!(read[closed].nakshatra, Nakshatra::Ashwini);
}

#[test]
fn ds_335_the_burning_decides_the_figures_and_the_death_the_shanti() {
    use DeathRiteKind::{ClosingShanti, PanchakaFigures};
    let sdk = context();
    let read = month(&sdk);
    let opened = read.iter().position(|one| one.panchaka).unwrap();
    let closed = opened + read[opened..].iter().position(|one| !one.panchaka).unwrap();
    let (inside, after) = (read[opened + 4].at.get(), read[closed + 1].at.get());
    // Before it, in a nakshatra that owes nothing of its own: no pushkara
    // and not tripada, which owe the shanti by themselves.
    let before = read[..opened]
        .iter()
        .rev()
        .find(|one| one.pushkara.is_none() && one.feet != Some(NakshatraFeet::Tripada))
        .unwrap()
        .at
        .get();
    let rites = |at, cremated| {
        check(&sdk, at, cremated)
            .rites
            .iter()
            .map(|one| one.rite)
            .filter(|&rite| rite != DeathRiteKind::PushkaraFigures)
            .collect::<Vec<_>>()
    };
    // Died in it and burnt in it: both.
    assert_eq!(rites(inside, None), [PanchakaFigures, ClosingShanti]);
    // Died in it and burnt in Ashwini: the shanti and no figures.
    assert_eq!(rites(inside, Some(after)), [ClosingShanti]);
    // Died before it and burnt in it: the figures and no shanti.
    let found = check(&sdk, before, Some(inside));
    assert!(!found.death.panchaka && found.cremation.unwrap().panchaka);
    assert_eq!(rites(before, Some(inside)), [PanchakaFigures]);
}

#[test]
fn ds_335_a_tripada_death_owes_the_shanti_and_a_dvipada_one_nothing() {
    let sdk = context();
    let read = month(&sdk);
    let outside = |feet| {
        read.iter()
            .find(|one| one.feet == Some(feet) && !one.panchaka && one.pushkara.is_none())
            .unwrap()
            .at
            .get()
    };
    let tripada = check(&sdk, outside(NakshatraFeet::Tripada), None);
    assert_eq!(
        tripada.rites.iter().map(|one| one.rite).collect::<Vec<_>>(),
        [DeathRiteKind::ClosingShanti]
    );
    let dvipada = check(&sdk, outside(NakshatraFeet::Dvipada), None);
    assert!(dvipada.rites.is_empty(), "{dvipada:?}");
}

#[test]
fn a_binding_query_is_the_same_answer_and_names_a_refusal_under_its_record() {
    let sdk = context();
    let at = JANUARY + 10.25;
    let asked = DeathTimeQuery::from_json(&format!(
        r#"{{"latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1337,
            "utcOffsetSeconds": 20700, "at": {at}}}"#
    ))
    .unwrap();
    let found = sdk.almanac().death_time_query(&asked).unwrap();
    assert_eq!(found.value, check(&sdk, at, None).in_full().unwrap());
    assert!(
        found.value["death"]["moon"]
            .as_str()
            .unwrap()
            .starts_with("rashi.")
    );
    let refused = |text: &str| DeathTimeQuery::from_json(text).unwrap_err();
    let early = refused(&format!(
        r#"{{"latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700,
            "at": {at}, "crematedAt": {}}}"#,
        at - 1.0
    ));
    assert_eq!(early.status, Status::InvalidArg);
    assert_eq!(early.field(), Some("deathTime.crematedAt"), "{early}");
    let unknown = refused(&format!(
        r#"{{"latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700,
            "at": {at}, "burial": true}}"#
    ));
    assert_eq!(unknown.status, Status::InvalidArg);
    let north = refused(&format!(
        r#"{{"latitudeDeg": 97, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700, "at": {at}}}"#
    ));
    assert_eq!(north.status, Status::InvalidArg);
}
