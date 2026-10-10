//! A range of days read from JSON (`DaysRequest`, the agent server's
//! `almanac.days`): it asks what the builder asks, it refuses by the
//! field written, and each section beside the days is sealed over what
//! a binding reads (`AlmanacAnswer::sections`).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests fail by panicking"
)]

use teistro::catalogue::Calendar;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{
    AlmanacRequest, CalendarDate, Context, DaysRequest, Ephemeris, FestivalPack, FestivalRequest,
    UtcOffset, content_hash,
};

fn context() -> Context {
    Context::builder()
        .profile("nepali-default")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("a context")
}

const KATHMANDU: &str = r#""latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400,
    "utcOffsetSeconds": 20700"#;

#[test]
fn a_json_days_request_is_the_request_it_spells() {
    let read = DaysRequest::from_json(&format!(
        r#"{{"calendar": "calendar.BIKRAM_SAMBAT", "first": {{"year": 2083, "month": 6, "day": 24}},
            "last": {{"year": 2083, "month": 6, "day": 26}}, {KATHMANDU},
            "festivals": {{"rules": "DHARMASINDHU"}}, "years": true, "nepalSambat": true}}"#
    ))
    .unwrap();
    let place = Place::new(
        Latitude::try_new(27.7172).unwrap(),
        Longitude::try_new(85.324).unwrap(),
        Altitude::try_new(1400.0).unwrap(),
    );
    assert_eq!(
        read.first,
        CalendarDate::defined(Calendar::BikramSambat, 2083, 6, 24)
    );
    assert_eq!(
        read.last,
        CalendarDate::defined(Calendar::BikramSambat, 2083, 6, 26)
    );
    assert_eq!(read.place, place);
    assert_eq!(read.offset, UtcOffset::try_from_seconds(20700).unwrap());
    let built = AlmanacRequest::new()
        .with_festivals(FestivalRequest::new(Vec::new()).with_pack(FestivalPack::Dharmasindhu))
        .with_years()
        .with_nepal_sambat();
    assert_eq!(read.beside, built);
}

#[test]
fn a_json_days_request_refuses_by_the_field_written() {
    let refused = |text: String| DaysRequest::from_json(&text).unwrap_err();
    let first = r#""first": {"year": 2026, "month": 10, "day": 10}"#;
    for (text, field) in [
        (
            format!("{{{first}, {KATHMANDU}, \"yeers\": true}}"),
            "yeers",
        ),
        (format!("{{{first}, {KATHMANDU}, \"years\": 1}}"), "years"),
        (
            format!("{{{first}, {KATHMANDU}, \"festivals\": {{\"rules\": 7}}}}"),
            "festivals.rules",
        ),
        (
            format!(
                "{{{first}, {KATHMANDU}, \"muhurta\": {{\"rules\": \"RAMAN_MARRIAGE\", \"rulez\": 1}}}}"
            ),
            "muhurta.rulez",
        ),
        (
            format!(
                "{{{first}, \"latitudeDeg\": 91, \"longitudeDeg\": 85.3, \"utcOffsetSeconds\": 0}}"
            ),
            "latitudeDeg",
        ),
        (format!("{{{KATHMANDU}}}"), "first"),
    ] {
        let error = refused(text.clone());
        assert!(
            error.field().is_some_and(|named| named.starts_with(field)),
            "{text}: {error} names {:?}",
            error.field()
        );
    }
}

#[test]
fn each_section_is_sealed_over_the_value_a_binding_reads() {
    let days = DaysRequest::from_json(&format!(
        r#"{{"first": {{"year": 2026, "month": 10, "day": 10}},
            "last": {{"year": 2026, "month": 10, "day": 11}}, {KATHMANDU},
            "muhurta": {{"rules": "RAMAN_MARRIAGE"}}, "festivals": {{"rules": "DHARMASINDHU"}},
            "years": true, "eclipses": true, "nepalSambat": true}}"#
    ))
    .unwrap();
    let answer = context()
        .almanac()
        .asked(
            &days.first,
            &days.last,
            &days.place,
            days.offset,
            &days.beside,
        )
        .unwrap();
    let sections = answer.sections().unwrap();
    for (name, section) in [
        ("muhurta", &sections.muhurta),
        ("festivals", &sections.festivals),
        ("years", &sections.years),
        ("eclipses", &sections.eclipses),
        ("nepalSambat", &sections.nepal_sambat),
    ] {
        let section = section.as_ref().expect(name);
        assert_eq!(
            section.provenance.content_hash,
            content_hash(&section.value),
            "{name}"
        );
    }
    let years = answer.years.as_ref().unwrap();
    assert_eq!(
        sections.years.unwrap().provenance.input_hash,
        years.provenance.input_hash,
        "a section keeps the stamp it was answered with"
    );
    assert_eq!(
        context()
            .almanac()
            .asked(
                &days.first,
                &days.last,
                &days.place,
                days.offset,
                &AlmanacRequest::new()
            )
            .unwrap()
            .sections()
            .unwrap(),
        teistro::AlmanacSections::default(),
        "nothing asked, nothing written"
    );
}
