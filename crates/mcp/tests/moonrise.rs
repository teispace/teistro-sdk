//! The moonrise convention through the server (SR-102, SR-113,
//! `03-design/moonrise-convention.md`): `settings.describe` names the knob
//! and the six named horizons, a call's patch moves the Moon and not the
//! day, the horizon model's dip lowers both, and a convention the
//! settings refuse is refused by its knob.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the replies they expect"
)]

use serde_json::{Value, json};
use teistro_mcp::{Engine, Server};

fn call(server: &mut Server, name: &str, arguments: &Value) -> Value {
    let reply = server
        .handle(
            &json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {
                "_meta": {"io.modelcontextprotocol/protocolVersion": "2026-07-28", "io.modelcontextprotocol/clientCapabilities": {}},
                "name": name, "arguments": arguments}})
            .to_string(),
        )
        .unwrap();
    let reply: Value = serde_json::from_str(&reply).unwrap();
    reply["result"].clone()
}

/// One day at Kathmandu, 5 January 2026, with nothing beside it.
fn one_day() -> Value {
    json!({"first": {"year": 2026, "month": 1, "day": 5}, "last": {"year": 2026, "month": 1, "day": 5},
        "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400, "utcOffsetSeconds": 20700})
}

/// The day's sunrise and its first moonrise, as Julian days.
fn sunrise_and_moonrise(answer: &Value) -> (f64, f64) {
    let day = &answer["structuredContent"]["value"]["days"][0];
    let sunrise = day["day"]["sunrise"].as_f64().expect("a sunrise");
    let rise = day["moon"]["rises"][0].as_f64().expect("a moonrise");
    (sunrise, rise)
}

#[test]
fn settings_describe_names_the_moonrise_and_every_named_horizon() {
    let mut server = Server::new(Engine::None);
    let described = call(&mut server, "settings.describe", &json!({}));
    assert_eq!(described["isError"], false);
    let schema = described["structuredContent"]["settings"].to_string();
    for named in [
        "moonrise",
        "UPPER_LIMB_NO_REFRACTION",
        "LOWER_LIMB_NO_REFRACTION",
        "REFRACTED",
    ] {
        assert!(schema.contains(named), "{named} is not described");
    }
}

#[test]
fn the_moon_rises_by_its_own_convention_and_the_day_by_its_sunrise() {
    let mut server = Server::new(Engine::Builtin);
    let asked = |server: &mut Server, settings: Value| {
        let answer = call(
            server,
            "almanac.days",
            &json!({"request": one_day(), "detail": "full", "profile": "nepali-drik", "settings": settings}),
        );
        assert_eq!(answer["isError"], false, "{answer}");
        sunrise_and_moonrise(&answer)
    };
    let own = asked(&mut server, json!({}));
    let centre = asked(
        &mut server,
        json!({"day": {"moonrise": {"kind": "NAMED", "which": "CENTRE_REFRACTION"}}}),
    );
    // The same day, and a Moon whose upper limb clears the horizon some
    // eighty seconds before her centre does.
    assert_eq!(own.0, centre.0);
    let earlier = (centre.1 - own.1) * 86_400.0;
    assert!((60.0..110.0).contains(&earlier), "{earlier} s");
    // The horizon model's refracted dip from Kathmandu's 1400 m lowers
    // the horizon some 1.05 degrees for both bodies.
    let dipped = asked(
        &mut server,
        json!({"day": {"horizon": {"dip": "REFRACTED"}}}),
    );
    let sun_sooner = (own.0 - dipped.0) * 86_400.0;
    assert!((280.0..350.0).contains(&sun_sooner), "{sun_sooner} s");
    let moon_sooner = (own.1 - dipped.1) * 86_400.0;
    assert!((240.0..420.0).contains(&moon_sooner), "{moon_sooner} s");
}

#[test]
fn a_moonrise_the_settings_refuse_is_refused_by_its_knob() {
    let mut server = Server::new(Engine::Builtin);
    let refused = call(
        &mut server,
        "almanac.days",
        &json!({"request": one_day(), "settings": {"day": {"moonrise":
            {"kind": "CUSTOM", "altitude_deg": 91}}}}),
    );
    assert_eq!(refused["isError"], true, "{refused}");
    assert_eq!(
        refused["structuredContent"]["field"],
        "settings.day.moonrise"
    );
}
