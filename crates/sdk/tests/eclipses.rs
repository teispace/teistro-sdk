//! Eclipses through the façade (`03-design/eclipses.md`): a year's at
//! Kathmandu, what the place sees of each, the shadow knob, and the
//! classical sky's refusal.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use std::collections::BTreeSet;

use teistro::catalogue::Calendar;
use teistro::eclipse::{LunarEclipseKind, ShadowRule, SolarEclipseKind};
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
    let lunar: Vec<LunarEclipseKind> = found.lunar.iter().map(|e| e.eclipse.kind).collect();
    let solar: Vec<SolarEclipseKind> = found.solar.iter().map(|e| e.eclipse.kind).collect();
    assert_eq!(lunar, [LunarEclipseKind::Total, LunarEclipseKind::Total]);
    assert_eq!(
        solar,
        [SolarEclipseKind::Partial, SolarEclipseKind::Partial]
    );
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

/// Every string leaf's dotted path, a list reaching each element.
fn string_paths(value: &serde_json::Value, path: &str, out: &mut BTreeSet<String>) {
    match value {
        serde_json::Value::String(_) => {
            out.insert(path.to_owned());
        }
        serde_json::Value::Array(list) => {
            for item in list {
                string_paths(item, path, out);
            }
        }
        serde_json::Value::Object(fields) => {
            for (field, inner) in fields {
                let joined = if path.is_empty() {
                    field.clone()
                } else {
                    format!("{path}.{field}")
                };
                string_paths(inner, &joined, out);
            }
        }
        _ => {}
    }
}

/// The section a boundary carries names each kind in full, where
/// `EclipsesHere::MEMBERS` says and nowhere else: every string path of a
/// year whose views are all present is a listed member or the shadow
/// rule, and every listed path is reached.
#[test]
fn the_eclipses_name_their_kinds_where_their_table_says() {
    // The shadow rule, which is a setting's value.
    const NOT_MEMBERS: [&str; 1] = ["lunar.eclipse.shadow"];
    // London saw March 2025's partial solar eclipse, so a view of each
    // kind is there to read.
    let london = Place::new(
        Latitude::literal(51.5),
        Longitude::literal(-0.13),
        Altitude::literal(20.0),
    );
    let found = context("nepali-default", None)
        .almanac()
        .eclipses(
            &CalendarDate::defined(Calendar::Gregorian, 2025, 3, 1),
            &CalendarDate::defined(Calendar::Gregorian, 2025, 3, 31),
            &london,
            UtcOffset::literal(0, 0, 0),
        )
        .unwrap()
        .value;
    assert!(found.solar.iter().any(|e| e.here.is_some()));
    assert!(!found.lunar.is_empty(), "`found.lunar` is empty");

    let mut found_paths = BTreeSet::new();
    string_paths(&serde_json::to_value(&found).unwrap(), "", &mut found_paths);
    let listed: BTreeSet<String> = EclipsesHere::MEMBERS
        .iter()
        .map(|(path, _)| (*path).to_owned())
        .chain(NOT_MEMBERS.iter().map(|path| (*path).to_owned()))
        .collect();
    assert_eq!(
        found_paths, listed,
        "a string path is unlisted, or a listed one is gone"
    );

    let full = found.in_full().unwrap();
    assert_eq!(
        full["lunar"][0]["eclipse"]["kind"],
        "lunar_eclipse_kind.TOTAL"
    );
    assert_eq!(
        full["solar"][0]["eclipse"]["kind"],
        "solar_eclipse_kind.PARTIAL"
    );
    assert_eq!(
        full["solar"][0]["here"]["kind"],
        "solar_eclipse_kind.PARTIAL"
    );
    assert_eq!(full["lunar"][0]["eclipse"]["shadow"], "DANJON");
}
