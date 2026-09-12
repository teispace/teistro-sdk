//! The engine's own functions, reached by name (ADR-0030).
//!
//! The route is generated, so what these check is not the arithmetic of
//! any one function — the engine has its own tests for that — but the
//! three things a generator cannot prove about itself: that the manifest
//! describes what `native_call` will actually answer, that an answer
//! comes back through the same context every chart is cast on, and that
//! everything the adapter declined to offer stays declined.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test fails by panicking"
)]

use serde_json::Value;
use teistro_ephemeris_teimeris::{TeimerisProvider, data_dir_from_env};
use teistro_port_ephemeris::EphemerisProvider;

fn provider() -> TeimerisProvider {
    TeimerisProvider::open(&data_dir_from_env()).unwrap_or_else(|error| panic!("{error}"))
}

fn manifest(provider: &TeimerisProvider) -> Value {
    serde_json::from_str(&provider.native_manifest().expect("a manifest")).expect("it parses")
}

/// The capability says the route is there, and the manifest is what it
/// promises: a name for every function, the role of every parameter.
///
/// A provider that declared `native` and answered nothing would be the
/// worst of both — a consumer would ask once, be told yes, and fail per
/// call.
#[test]
fn the_manifest_describes_what_the_adapter_will_answer() {
    let provider = provider();
    assert!(
        provider.capabilities().native,
        "the adapter declares the route"
    );
    let manifest = manifest(&provider);
    assert_eq!(manifest["engine"], "teimeris");
    let functions = manifest["functions"].as_array().expect("functions");
    assert!(
        functions.len() > 30,
        "the manifest lists {} functions",
        functions.len()
    );
    for function in functions {
        assert!(
            function["name"].as_str().is_some_and(|n| n.starts_with("tm_")),
            "every entry names an engine function: {function}"
        );
        for param in function["params"].as_array().expect("params") {
            let role = param["role"].as_str().expect("a role");
            assert!(
                matches!(role, "in" | "out"),
                "a parameter is the caller's or the engine's, not {role}"
            );
        }
    }
}

/// A real call, through the adapter's own context, answering the value
/// the engine answers.
///
/// Delta T at J2000 is about 64 seconds — a figure any reference gives —
/// and the point is not the number but that the number arrived: the
/// arguments were read from JSON, the engine was called, and its
/// out-parameter came back under its own name.
#[test]
fn a_function_is_called_by_name_and_answers() {
    let provider = provider();
    let answer = provider
        .native_call("tm_delta_t", r#"{"jd_ut1": 2451545.0}"#)
        .expect("the engine answers");
    let answer: Value = serde_json::from_str(&answer).expect("it parses");
    let seconds = answer["out_seconds"].as_f64().expect("out_seconds");
    assert!(
        (60.0..70.0).contains(&seconds),
        "Delta T at J2000 is about 64 seconds, not {seconds}"
    );
}

/// A function that returns a value rather than filling a parameter comes
/// back under `return`, which is the manifest's word for it.
#[test]
fn a_returned_value_comes_back_as_return() {
    let provider = provider();
    let answer = provider
        .native_call("tm_day_of_week", r#"{"jd": 2451545.0}"#)
        .expect("the engine answers");
    let answer: Value = serde_json::from_str(&answer).expect("it parses");
    let day = answer["return"].as_i64().expect("return");
    assert!(
        (0..7).contains(&day),
        "a weekday is one of seven, not {day}"
    );
}

/// **The adapter's own context is never handed over.** These close it,
/// or rebind the data it holds, and a consumer who reached one through
/// this route would take the ground from under every other call.
///
/// They are refused for what they touch and not for what they cost, so
/// teaching the marshaller more shapes must never bring them in — which
/// is what this test is for.
#[test]
fn what_the_adapter_owns_is_refused_by_name() {
    let provider = provider();
    let manifest = manifest(&provider);
    let offered: Vec<&str> = manifest["functions"]
        .as_array()
        .expect("functions")
        .iter()
        .filter_map(|f| f["name"].as_str())
        .collect();
    for name in [
        "tm_context_close",
        "tm_context_open",
        "tm_data_source_from_path",
        "tm_context_set_fetch",
    ] {
        assert!(
            !offered.contains(&name),
            "`{name}` must not be in the manifest"
        );
        let error = provider
            .native_call(name, "{}")
            .expect_err("and must not be callable");
        assert!(
            error.to_string().contains(name),
            "the refusal names what was asked: {error}"
        );
    }
}

/// A name the engine does not have is refused the same way, and says
/// where to look.
#[test]
fn an_unknown_function_is_refused_and_says_where_to_look() {
    let provider = provider();
    let error = provider
        .native_call("tm_not_a_function", "{}")
        .expect_err("there is no such function");
    assert!(
        error.to_string().contains("native_manifest"),
        "the refusal points at the list: {error}"
    );
}

/// An argument of the wrong kind is refused **by name**, because a caller
/// writing against a manifest has usually passed the right name and the
/// wrong shape, and a bare "invalid" would leave them reading all of it.
#[test]
fn a_bad_argument_is_refused_by_name() {
    let provider = provider();
    let error = provider
        .native_call("tm_delta_t", r#"{"jd_ut1": "noon"}"#)
        .expect_err("a string is not an instant");
    let said = error.to_string();
    assert!(said.contains("jd_ut1"), "{said}");
    assert!(said.contains("a string"), "{said}");

    let missing = provider
        .native_call("tm_delta_t", "{}")
        .expect_err("the argument is required");
    assert!(missing.to_string().contains("jd_ut1"), "{missing}");
}

/// The route answers from the **same** context the charts are cast on.
///
/// This is the property that makes the namespace worth having and the one
/// a second context would quietly break: a consumer reading the engine's
/// model settings must read the settings their charts were computed
/// under, not another engine's defaults.
#[test]
fn the_route_shares_the_context_that_computes() {
    let provider = provider();
    let read = |kind: i64| -> i64 {
        let answer = provider
            .native_call("tm_get_model", &format!(r#"{{"kind": {kind}}}"#))
            .expect("the engine answers");
        let answer: Value = serde_json::from_str(&answer).expect("it parses");
        answer["out_model"].as_i64().expect("out_model")
    };
    let before = read(0);
    provider
        .native_call("tm_set_model", &format!(r#"{{"kind": 0, "model": {before}}}"#))
        .expect("setting it to what it already is");
    assert_eq!(read(0), before, "the same context answered both times");
}
