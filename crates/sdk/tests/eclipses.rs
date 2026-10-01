//! Eclipses through the façade (`03-design/eclipses.md`): a year's at
//! Kathmandu, what the place sees of each, the shadow knob, and the
//! classical sky's refusal.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::Calendar;
use teistro::eclipse::{LunarKind, ShadowRule, SolarKind};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{
    AlmanacRequest, CalendarDate, Context, EclipsesHere, Envelope, Ephemeris, Status, UtcOffset,
};

fn context(profile: &str, patch: Option<&str>) -> Context {
    // The committee's sky is the text's, opened over its own provider.
    let sky = if profile == "nepali-committee" {
        Ephemeris::SuryaSiddhanta
    } else {
        Ephemeris::Builtin
    };
    let builder = Context::builder().profile(profile).ephemeris([sky]);
    match patch {
        Some(json) => builder.settings_json(json),
        None => builder,
    }
    .build()
    .unwrap()
}

fn kathmandu() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    )
}

/// 2025's eclipses at Kathmandu, on Nepal's clock.
fn year(sdk: &Context) -> Result<Envelope<EclipsesHere>, teistro::Error> {
    sdk.almanac().eclipses(
        &CalendarDate::defined(Calendar::Gregorian, 2025, 1, 1),
        &CalendarDate::defined(Calendar::Gregorian, 2025, 12, 31),
        &kathmandu(),
        UtcOffset::literal(5, 45, 0),
    )
}

#[test]
fn kathmandu_saw_one_of_2025s_four_eclipses() {
    let found = year(&context("nepali-default", None)).unwrap().value;
    // Two total lunar eclipses, March's and September's, and two partial
    // solar ones, March's over the North Atlantic and September's over
    // the South Pacific.
    let lunar: Vec<LunarKind> = found.lunar.iter().map(|e| e.eclipse.kind).collect();
    let solar: Vec<SolarKind> = found.solar.iter().map(|e| e.eclipse.kind).collect();
    assert_eq!(lunar, [LunarKind::Total, LunarKind::Total]);
    assert_eq!(solar, [SolarKind::Partial, SolarKind::Partial]);
    // March's lunar eclipse fell in Kathmandu's afternoon, the Moon below
    // the horizon; September's near midnight, seen whole.
    assert!(found.lunar[0].here.seen.is_none());
    let september = &found.lunar[1];
    let seen = september.here.seen.unwrap();
    assert_eq!(
        (seen.from, seen.to),
        (september.here.p1.at, september.here.p4.at)
    );
    assert_eq!(september.eclipse.shadow, ShadowRule::Danjon);
    // Neither solar eclipse is seen from Nepal.
    assert!(
        found
            .solar
            .iter()
            .all(|e| e.here.is_none_or(|here| here.seen.is_none()))
    );
    assert!(found.any_seen());
}

#[test]
fn the_answer_says_its_window_and_the_knob_moves_the_shadow() {
    let danjon = year(&context("nepali-default", None)).unwrap();
    assert!(
        danjon
            .provenance
            .applied_conventions
            .iter()
            .any(|c| c.knob == "eclipse.window")
    );
    let chauvenet = year(&context(
        "nepali-default",
        Some(r#"{"panchanga": {"eclipse_shadow": "CHAUVENET"}}"#),
    ))
    .unwrap();
    let (a, b) = (
        &danjon.value.lunar[1].eclipse,
        &chauvenet.value.lunar[1].eclipse,
    );
    assert_eq!(b.shadow, ShadowRule::Chauvenet);
    assert!(b.umbral_magnitude > a.umbral_magnitude);
    // A different setting is a different answer, and says so.
    assert_ne!(
        danjon.provenance.settings_hash,
        chauvenet.provenance.settings_hash
    );
}

#[test]
fn the_committees_sky_refuses_with_the_crux_it_waits_on() {
    let refused = year(&context("nepali-committee", None)).unwrap_err();
    assert_eq!(refused.status, Status::Unsupported);
    assert!(refused.message.contains("C188"), "{}", refused.message);
}

#[test]
fn the_eclipses_asked_beside_the_days_are_the_eclipses() {
    let sdk = context("nepali-default", None);
    let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2025, month, day);
    let (from, to) = (date(9, 1), date(9, 30));
    let clock = UtcOffset::literal(5, 45, 0);
    let ask = |request: &AlmanacRequest| {
        sdk.almanac()
            .asked(&from, &to, &kathmandu(), clock, request)
            .unwrap()
    };
    let answer = ask(&AlmanacRequest::new().with_eclipses());
    let alone = sdk
        .almanac()
        .eclipses(&from, &to, &kathmandu(), clock)
        .unwrap();
    assert_eq!(answer.eclipses.unwrap().value, alone.value);
    assert_eq!((alone.value.lunar.len(), alone.value.solar.len()), (1, 1));
    assert!(ask(&AlmanacRequest::new()).eclipses.is_none());
}
