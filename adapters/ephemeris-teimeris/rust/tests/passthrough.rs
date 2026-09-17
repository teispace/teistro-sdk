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
    clippy::indexing_slicing,
    reason = "a test fails by panicking: an index into an answer's array that is out of \
              bounds is a wrong answer, and one into a `serde_json::Value` answers `Null`"
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
            function["name"]
                .as_str()
                .is_some_and(|n| n.starts_with("tm_")),
            "every entry names an engine function: {function}"
        );
        for param in function["params"].as_array().expect("params") {
            let role = param["role"].as_str().expect("a role");
            assert!(
                matches!(role, "in" | "out"),
                "a parameter is the caller's or the engine's, not {role}"
            );
            assert!(
                param["type"].as_str().is_some_and(|t| t != "char"),
                "a string is declared a string and not the C it is made of: {param}"
            );
        }
    }
}

/// A string the engine **lends** arrives copied, and its way of saying
/// "there is no such name" arrives as null rather than as an empty
/// string — which is a different answer and the caller needs to tell
/// them apart.
#[test]
fn a_string_the_engine_lends_arrives_and_null_stays_null() {
    let provider = provider();
    let answer = provider
        .native_call("tm_status_name", r#"{"s": 0}"#)
        .expect("the engine answers");
    let answer: Value = serde_json::from_str(&answer).expect("it parses");
    assert!(
        answer["return"].as_str().is_some_and(|n| !n.is_empty()),
        "success has a name: {answer}"
    );

    let none = provider
        .native_call("tm_zodiac_sign_name", r#"{"sign": -1}"#)
        .expect("the engine answers");
    let none: Value = serde_json::from_str(&none).expect("it parses");
    assert!(
        none["return"].is_null(),
        "there is no sign before Aries, and that is not \"\": {none}"
    );
}

/// A string the engine **fills** comes back under its own parameter's
/// name, like every other out-parameter.
///
/// And the `size_t` the function returns does **not** come back: it is
/// the fill protocol's bookkeeping, the string it counts is already here
/// in full, and a `return` beside `buf` would invite a caller to believe
/// it meant something else.
#[test]
fn a_string_the_engine_fills_comes_back_under_its_parameter_name() {
    let provider = provider();
    let answer = provider
        .native_call("tm_body_name", r#"{"body": 0}"#)
        .expect("the engine answers");
    let answer: Value = serde_json::from_str(&answer).expect("it parses");
    assert_eq!(answer["buf"], "Sun");
    assert!(
        answer.get("return").is_none(),
        "the length is the protocol's, not an answer: {answer}"
    );

    // A formatted angle, because it is the fill whose answer is not a
    // table lookup and so is the one a marshalling bug would garble.
    let formatted = provider
        .native_call(
            "tm_angle_format",
            // Style 1 is the zodiacal one; the manifest names the
            // parameter and the engine's own enumeration names the value.
            r#"{"deg": 35.5, "style": 1, "decimals": 0}"#,
        )
        .expect("the engine answers");
    let formatted: Value = serde_json::from_str(&formatted).expect("it parses");
    let text = formatted["buf"].as_str().expect("buf");
    // The engine's header documents the zodiacal style as `12 Ari
    // 34'56"`: the sign is abbreviated.
    assert!(
        text.contains("5 Tau") && text.contains("30'"),
        "35.5 degrees is 5 Tau 30', not {text}"
    );
}

/// A string the **caller** passes reaches the engine, and the two ways
/// it can be wrong are told apart: a shape the marshaller refuses by
/// name, and a value the engine refuses with its own code.
#[test]
fn a_string_the_caller_passes_reaches_the_engine() {
    let provider = provider();
    let refused = provider
        .native_call(
            "tm_star_load_catalogue",
            r#"{"path": "/nowhere/no-such-catalogue.txt"}"#,
        )
        .expect_err("there is no such file");
    assert!(
        refused.to_string().contains("tm_star_load_catalogue"),
        "the engine refused the path it was given: {refused}"
    );

    let wrong_kind = provider
        .native_call("tm_star_load_catalogue", r#"{"path": 7}"#)
        .expect_err("a number is not a path");
    let said = wrong_kind.to_string();
    assert!(said.contains("path"), "{said}");
    assert!(said.contains("a number"), "{said}");

    let cut = provider
        .native_call("tm_set_jpl_file", "{\"filename\": \"de441\\u0000.eph\"}")
        .expect_err("C cannot carry a NUL");
    assert!(cut.to_string().contains("NUL"), "{cut}");
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
        .native_call(
            "tm_set_model",
            &format!(r#"{{"kind": 0, "model": {before}}}"#),
        )
        .expect("setting it to what it already is");
    assert_eq!(read(0), before, "the same context answered both times");
}

/// A struct crosses as an object keyed by the engine's own field names,
/// in both directions, and its extent crosses in neither.
///
/// A local time shifted to UTC is the call, because its answer is known
/// without an ephemeris: 06:30 at +05:45 is 00:45 the same day.
#[test]
fn a_struct_crosses_as_an_object_both_ways() {
    let provider = provider();
    let answer = provider
        .native_call(
            "tm_local_to_utc",
            r#"{"local": {"year": 2026, "month": 9, "day": 13, "hour": 6, "minute": 30,
                "second": 0}, "utc_offset_hours": 5.75, "cal": 1}"#,
        )
        .expect("the engine answers");
    let answer: Value = serde_json::from_str(&answer).expect("it parses");
    let utc = &answer["out_utc"];
    assert_eq!(
        (
            &utc["year"],
            &utc["month"],
            &utc["day"],
            &utc["hour"],
            &utc["minute"]
        ),
        (
            &Value::from(2026),
            &Value::from(9),
            &Value::from(13),
            &Value::from(0),
            &Value::from(45)
        ),
        "{answer}"
    );
    assert!(
        utc.get("struct_size").is_none(),
        "the extent is the arm's, and is not reported: {answer}"
    );
}

/// A field is refused by its whole path, because a call may take more
/// than one struct and `month` alone would not say whose.
#[test]
fn a_struct_field_is_refused_by_its_whole_path() {
    let provider = provider();
    let missing = provider
        .native_call(
            "tm_local_to_utc",
            r#"{"local": {"year": 2026, "day": 13, "hour": 6, "minute": 30, "second": 0},
                "utc_offset_hours": 5.75, "cal": 1}"#,
        )
        .expect_err("a struct with a field left out is refused");
    assert!(
        missing.to_string().contains("`local.month` is required"),
        "{missing}"
    );
    let fractional = provider
        .native_call(
            "tm_local_to_utc",
            r#"{"local": {"year": 2026.5, "month": 9, "day": 13, "hour": 6, "minute": 30,
                "second": 0}, "utc_offset_hours": 5.75, "cal": 1}"#,
        )
        .expect_err("a fractional year is refused rather than truncated");
    assert!(
        fractional.to_string().contains("`local.year`"),
        "{fractional}"
    );
}

/// A struct input left out crosses as null, and the engine — not the
/// marshaller — decides whether it takes one: a datetime it does not,
/// and refuses with its own status.
#[test]
fn a_struct_left_out_is_null_and_the_engine_decides() {
    let provider = provider();
    let refused = provider
        .native_call("tm_local_to_utc", r#"{"utc_offset_hours": 5.75, "cal": 1}"#)
        .expect_err("the engine takes no null datetime");
    assert!(
        refused.to_string().contains("tm_local_to_utc refused")
            || refused.to_string().contains("`tm_local_to_utc` refused"),
        "the engine's own refusal, not the marshaller's: {refused}"
    );
}

/// A sized struct's defaults come back with every field and without the
/// extent, and an extent a caller passes is not read: the arm measures
/// the struct it declared.
#[test]
fn an_extent_is_the_arms_and_never_the_callers() {
    let provider = provider();
    let answer = provider
        .native_call("tm_crossing_request_init_sized", r#"{"struct_size": 1}"#)
        .expect("the engine answers, whatever extent the caller named");
    let answer: Value = serde_json::from_str(&answer).expect("it parses");
    let request = answer["req"].as_object().expect("an object");
    assert!(request.contains_key("jd_start"), "{answer}");
    assert!(!request.contains_key("struct_size"), "{answer}");
}

/// A struct nested in another comes back nested: the four points of an
/// orbit are each a position.
#[test]
fn a_nested_struct_comes_back_nested() {
    let provider = provider();
    // Mars (4), UT1 (1), no flags, the mean method and the aphelion; no
    // observer, because the answer is not topocentric.
    let answer = provider
        .native_call(
            "tm_nodes_apsides_calc",
            r#"{"jd": 2461296.5, "scale": 1, "body": 4, "flags": 0, "method": 0, "apsis": 0}"#,
        )
        .expect("the engine answers");
    let answer: Value = serde_json::from_str(&answer).expect("it parses");
    let perihelion = &answer["out"]["perihelion"];
    let lon = perihelion["lon"].as_f64().expect("a longitude");
    assert!((0.0..360.0).contains(&lon), "{answer}");
    assert!(perihelion.get("struct_size").is_none(), "{answer}");
}

/// Calls a function by name with JSON written inline, and parses what
/// comes back.
fn called(provider: &TeimerisProvider, function: &str, args: &Value) -> Value {
    let answer = provider
        .native_call(function, &args.to_string())
        .unwrap_or_else(|error| panic!("{function}: {error}"));
    serde_json::from_str(&answer).expect("it parses")
}

/// An output as long as its input: one answer per instant, in order, and
/// none for none.
#[test]
fn an_array_answers_one_value_per_input() {
    let provider = provider();
    let jds = [2_451_545.0, 2_461_296.5, 2_378_497.0];
    let answer = called(
        &provider,
        "tm_delta_t_many",
        &serde_json::json!({ "jds_ut1": jds }),
    );
    let batch = answer["out_seconds"].as_array().expect("an array");
    assert_eq!(batch.len(), jds.len(), "{answer}");
    for (at, jd) in jds.iter().enumerate() {
        let one = called(
            &provider,
            "tm_delta_t",
            &serde_json::json!({ "jd_ut1": jd }),
        );
        assert_eq!(batch.get(at), one.get("out_seconds"), "instant {at}");
    }
    let none = called(
        &provider,
        "tm_delta_t_many",
        &serde_json::json!({ "jds_ut1": [] }),
    );
    assert_eq!(none["out_seconds"], serde_json::json!([]));
    assert!(
        answer.get("count").is_none() && answer.get("out_capacity").is_none(),
        "a length and a capacity are the marshaller's own: {answer}"
    );
}

/// An array of structs crosses both ways, and a bad element is refused
/// by its index and field.
#[test]
fn an_array_of_structs_crosses_both_ways() {
    let provider = provider();
    let dates = called(
        &provider,
        "tm_calendar_date_many",
        &serde_json::json!({ "jds": [2_451_545.0, 2_461_296.5], "cal": 1 }),
    );
    let dates = dates["out"].as_array().expect("an array").clone();
    assert_eq!(dates.len(), 2);
    assert_eq!(dates[0]["year"], 2000);
    assert!(dates[0].get("struct_size").is_none(), "{dates:?}");

    let back = called(
        &provider,
        "tm_julian_day_many",
        &serde_json::json!({ "dts": dates, "cal": 1 }),
    );
    assert_eq!(
        back["out_jd"],
        serde_json::json!([2_451_545.0, 2_461_296.5])
    );

    let refused = provider
        .native_call(
            "tm_julian_day_many",
            &serde_json::json!({
                "dts": [
                    { "year": 2000, "month": 1, "day": 1, "hour": 12, "minute": 0, "second": 0 },
                    { "year": 2000, "day": 1, "hour": 12, "minute": 0, "second": 0 },
                ],
                "cal": 1,
            })
            .to_string(),
        )
        .expect_err("an element with a field left out is refused");
    assert!(refused.to_string().contains("`dts[1].month`"), "{refused}");
}

/// An output laid out over two inputs is as long as their product, in
/// the layout the header states: bodies outermost.
#[test]
fn a_grid_is_as_long_as_its_inputs_multiplied() {
    let provider = provider();
    let answer = called(
        &provider,
        "tm_position_calc_grid",
        &serde_json::json!({
            "bodies": [0, 1],
            "jds": [2_451_545.0, 2_451_546.0, 2_451_547.0],
            "scale": 1,
            "flags": 0,
        }),
    );
    let grid = answer["out"].as_array().expect("an array");
    assert_eq!(grid.len(), 6, "two bodies by three epochs");
    // The Sun moves about a degree a day and the Moon about thirteen, so
    // body-major means the first three are close together.
    let lon = |at: usize| grid[at]["lon"].as_f64().expect("a longitude");
    assert!((lon(1) - lon(0)).rem_euclid(360.0) < 2.0, "{answer}");
    assert!((lon(4) - lon(3)).rem_euclid(360.0) > 10.0, "{answer}");
}

/// An output the engine counts: asked once into room that fits, and the
/// count is the array's length rather than a second key.
#[test]
fn an_output_the_engine_counts_is_gathered() {
    let provider = provider();
    let answer = called(&provider, "tm_chart_default_bodies", &serde_json::json!({}));
    let bodies = answer["out_bodies"].as_array().expect("an array");
    assert!(bodies.len() >= 10, "{answer}");
    assert_eq!(bodies[0], 0, "the Sun first: {answer}");
    assert!(answer.get("return").is_none(), "{answer}");
}

/// A search answers as many as the caller asks for, no more, and the
/// count it was told is the array's length.
#[test]
fn a_search_answers_as_many_as_asked() {
    let provider = provider();
    let mut request = called(
        &provider,
        "tm_crossing_request_init_sized",
        &serde_json::json!({}),
    )["req"]
        .clone();
    // The Sun reaching 0° from the start of 2026: each March equinox.
    request["body"] = 0.into();
    request["jd_start"] = 2_461_041.5.into();
    request["target_deg"] = 0.0.into();
    request["jd_end"] = (2_461_041.5 + 3.0 * 365.25).into();
    let answer = called(
        &provider,
        "tm_crossing_search",
        &serde_json::json!({ "req": request, "out_capacity": 2 }),
    );
    let found = answer["out"].as_array().expect("an array");
    assert_eq!(found.len(), 2, "{answer}");
    let jd = |at: usize| found[at]["jd"].as_f64().expect("an instant");
    assert!(
        (jd(1) - jd(0) - 365.24).abs() < 1.0,
        "a year apart: {answer}"
    );
    assert!(answer.get("out_count").is_none(), "{answer}");
}

/// A refusal carries the engine's own words for it, which is the
/// difference between "status -1" and knowing what to change.
#[test]
fn a_refusal_carries_the_engine_s_own_message() {
    let provider = provider();
    let refused = provider
        .native_call(
            "tm_set_ayanamsha",
            r#"{"mode": 9999, "t0": 2451545.0, "ayan_t0": 0.0}"#,
        )
        .expect_err("no ayanamsha is numbered 9999");
    let said = refused.to_string();
    assert!(said.contains("invalid argument"), "{said}");
    assert!(said.contains("no such ayanamsha"), "{said}");
}

/// A failure is never given an earlier failure's message (findings
/// register D3). An engine from before its fix leaves the previous record
/// in place, and the passthrough says it recorded nothing; one after the
/// fix writes its own, naming the null argument. Either is true; the star's
/// message never is.
#[test]
fn an_earlier_failure_s_message_is_not_repeated_as_this_one_s() {
    let provider = provider();
    let first = provider
        .native_call("tm_star_find", r#"{"name": "no such star"}"#)
        .expect_err("no star has that name");
    assert!(first.to_string().contains("no star named"), "{first}");
    // A null datetime, which the engine refuses with a bare return and
    // no message of its own.
    let second = provider
        .native_call("tm_local_to_utc", r#"{"utc_offset_hours": 5.75, "cal": 1}"#)
        .expect_err("the engine takes no null datetime");
    let said = second.to_string();
    assert!(!said.contains("no star named"), "a stale message: {said}");
    assert!(
        said.contains("is null") || said.contains("recorded no message"),
        "this failure's own words, or none: {said}"
    );
}

/// A string the engine writes into a struct it fills comes back as a
/// string, copied before anything else can move it.
#[test]
fn a_string_inside_an_answer_arrives() {
    let provider = provider();
    let answer = called(
        &provider,
        "tm_model_info",
        &serde_json::json!({ "kind": 0, "index": 0 }),
    );
    let name = answer["out"]["name"].as_str().expect("a name");
    assert!(!name.is_empty(), "{answer}");
    assert!(answer["out"].get("struct_size").is_none(), "{answer}");
}

/// A request pointing at an observer crosses as a nested object, and
/// null says "no observer"; a pointer left out is refused by its path,
/// because a key forgotten is a mistake and null is a choice.
#[test]
fn a_pointer_inside_a_request_is_an_object_or_null() {
    let provider = provider();
    let request = |flags: u32, observer: Value| {
        serde_json::json!({ "req": {
            "jd": 2_451_545.0, "scale": 1, "body": 1, "flags": flags, "observer": observer,
            "center": 0, "ayanamsha": 0, "ayanamsha_set": 0,
        }})
    };
    let geocentric = called(&provider, "tm_position_calc", &request(0, Value::Null));
    // TM_TOPOCENTRIC, from Kathmandu: the Moon moves by up to its
    // parallax, about a degree, so the two must differ.
    let topocentric = called(
        &provider,
        "tm_position_calc",
        &request(
            1 << 5,
            serde_json::json!({ "longitude_deg": 85.324, "latitude_deg": 27.7172, "altitude_m": 1400.0 }),
        ),
    );
    let lon = |answer: &Value| answer["out"]["lon"].as_f64().expect("a longitude");
    let apart = (lon(&geocentric) - lon(&topocentric)).abs();
    assert!(
        apart > 0.01 && apart < 2.0,
        "{apart}: {geocentric} {topocentric}"
    );

    let mut forgotten = request(0, Value::Null);
    forgotten["req"]
        .as_object_mut()
        .expect("an object")
        .remove("observer");
    let refused = provider
        .native_call("tm_position_calc", &forgotten.to_string())
        .expect_err("a pointer field left out is refused");
    assert!(
        refused
            .to_string()
            .contains("`req.observer` is required; pass null"),
        "{refused}"
    );
}

/// A string inside a request reaches the engine, and a default request
/// with null pointers round-trips: asked for, changed, passed back.
#[test]
fn a_default_request_round_trips_with_a_string_in_it() {
    let provider = provider();
    let mut query = called(
        &provider,
        "tm_star_query_init_sized",
        &serde_json::json!({}),
    )["q"]
        .clone();
    assert!(
        query["name_contains"].is_null(),
        "no name by default: {query}"
    );
    query["name_contains"] = "Aldeb".into();
    let found = called(
        &provider,
        "tm_star_search",
        &serde_json::json!({ "query": query }),
    );
    let stars = found["out"].as_array().expect("an array");
    assert!(!stars.is_empty(), "Aldebaran contains Aldeb: {found}");
}

/// An array of requests each pointing at its own observer: every
/// pointer stays valid until the call returns, however many there are.
#[test]
fn an_array_of_requests_keeps_every_pointer_alive() {
    let provider = provider();
    let reqs: Vec<Value> = (0..40)
        .map(|at| {
            serde_json::json!({
                "jd": 2_451_545.0, "scale": 1, "body": 1, "flags": 1 << 5,
                "observer": { "longitude_deg": f64::from(at) * 9.0, "latitude_deg": 0.0, "altitude_m": 0.0 },
                "center": 0, "ayanamsha": 0, "ayanamsha_set": 0,
            })
        })
        .collect();
    let answer = called(
        &provider,
        "tm_position_calc_many",
        &serde_json::json!({ "reqs": reqs }),
    );
    let out = answer["out"].as_array().expect("an array");
    assert_eq!(out.len(), 40);
    for (at, position) in out.iter().enumerate() {
        assert_eq!(position["status"], 0, "element {at}: {position}");
    }
    let first = out[0]["lon"].as_f64().expect("a longitude");
    assert!(
        out.iter()
            .any(|position| (position["lon"].as_f64().unwrap_or(first) - first).abs() > 0.1),
        "forty observers round the equator see the Moon in different places"
    );
}

/// A batch with one bad element answers every element, each with its own
/// status, instead of discarding the ones that succeeded.
#[test]
fn a_batch_with_one_bad_element_answers_the_rest() {
    let provider = provider();
    let request = |body: i32| {
        serde_json::json!({
            "jd": 2_451_545.0, "scale": 1, "body": body, "flags": 0, "observer": null,
            "center": 0, "ayanamsha": 0, "ayanamsha_set": 0,
        })
    };
    let answer = called(
        &provider,
        "tm_position_calc_many",
        &serde_json::json!({ "reqs": [request(0), request(9999), request(1)] }),
    );
    let out = answer["out"].as_array().expect("an array");
    assert_eq!(out.len(), 3, "{answer}");
    assert_eq!(out[0]["status"], 0, "{answer}");
    assert_ne!(out[1]["status"], 0, "no body is numbered 9999: {answer}");
    assert_eq!(out[2]["status"], 0, "{answer}");
    assert!(
        out[2]["lon"].as_f64().is_some_and(|lon| lon > 0.0),
        "{answer}"
    );
}

/// A batch that succeeds never hands a caller the mark the marshaller
/// used to see which elements were written.
#[test]
fn a_successful_batch_carries_no_mark() {
    let provider = provider();
    let position = serde_json::json!({
        "lon": 10.0, "lat": 1.0, "dist": 1.0, "lon_speed": 0.0, "lat_speed": 0.0,
        "dist_speed": 0.0, "flags_used": 0, "status": 0,
    });
    let answer = called(
        &provider,
        "tm_coord_rotate",
        &serde_json::json!({ "direction": 0, "obliquity_deg": 23.44, "positions": [position] }),
    );
    let status = &answer["out"][0]["status"];
    assert_eq!(status, 0, "{answer}");
}

/// House cusps are as long as the requested system has cusps — asked of
/// `tm_house_cusp_count` before the call — and their speeds as long as
/// the cusps: twelve for Placidus, thirty-six for Gauquelin's sectors.
#[test]
fn a_house_system_decides_how_many_cusps_come_back() {
    let provider = provider();
    let houses = |system: i32| {
        called(
            &provider,
            "tm_houses_calc",
            &serde_json::json!({ "req": {
                "jd_ut1": 2_451_545.0, "geo_lat_deg": 27.7172, "geo_lon_deg": 85.324,
                "system": system, "flags": 0,
            }}),
        )
    };
    for (system, cusps) in [(0, 12), (12, 36)] {
        let answer = houses(system);
        let length = |key: &str| answer[key].as_array().map_or(0, Vec::len);
        assert_eq!(length("cusps"), cusps, "system {system}: {answer}");
        assert_eq!(length("cusp_speeds"), cusps, "system {system}: {answer}");
        let used = answer["out_angles"]["system_used"].as_i64();
        assert_eq!(used, Some(i64::from(system)), "no substitution: {answer}");
    }
}

/// A chart's positions are as long as its bodies and its cusps as long as
/// its house system says, in one call.
#[test]
fn a_chart_is_sized_by_its_bodies_and_its_system() {
    let provider = provider();
    let answer = called(
        &provider,
        "tm_chart_calc",
        &serde_json::json!({
            "req": {
                "jd": 2_451_545.0, "scale": 1, "parts": 0, "flags": 0, "system": 0,
                "house_flags": 0,
                "place": { "longitude_deg": 85.324, "latitude_deg": 27.7172, "altitude_m": 1400.0 },
            },
            "bodies": [0, 1],
        }),
    );
    let length = |key: &str| answer[key].as_array().map_or(0, Vec::len);
    assert_eq!(length("out_positions"), 2, "{answer}");
    assert_eq!(length("out_cusps"), 12, "{answer}");
    assert_eq!(length("out_cusp_speeds"), 12, "{answer}");
}

/// A calendar grid is as long as a field of its request says: a day per
/// `day_count`, and a position per day per body.
#[test]
fn a_calendar_is_sized_by_a_field_of_its_request() {
    let provider = provider();
    let mut req = called(
        &provider,
        "tm_calendar_request_init_sized",
        &serde_json::json!({}),
    )["req"]
        .clone();
    req["jd_start"] = 2_461_296.5.into();
    req["day_count"] = 3.into();
    req["observer"] = serde_json::json!({ "longitude_deg": 85.324, "latitude_deg": 27.7172, "altitude_m": 1400.0 });
    let answer = called(
        &provider,
        "tm_calendar_grid",
        &serde_json::json!({ "req": req, "bodies": [0, 1] }),
    );
    let length = |key: &str| answer[key].as_array().map_or(0, Vec::len);
    assert_eq!(length("out_days"), 3, "{answer}");
    assert_eq!(
        length("out_positions"),
        6,
        "three days by two bodies: {answer}"
    );
    let found = answer["out_days"][0]["found"].as_i64().unwrap_or_default();
    assert_eq!(found, 1, "the Sun rises in Kathmandu: {answer}");
}

/// A batch of house requests is laid out at its widest system's stride:
/// two Placidus charts hold twenty-four cusps, and one Placidus beside one
/// Gauquelin holds seventy-two, the Placidus chart's twelve at the start
/// of its thirty-six-wide slot.
#[test]
fn a_house_batch_is_laid_out_at_its_widest_stride() {
    let provider = provider();
    let request = |system: i32| {
        serde_json::json!({
            "jd_ut1": 2_451_545.0, "geo_lat_deg": 27.7172, "geo_lon_deg": 85.324,
            "system": system, "flags": 0,
        })
    };
    let batch = |systems: &[i32]| {
        let reqs: Vec<Value> = systems.iter().copied().map(request).collect();
        called(
            &provider,
            "tm_houses_calc_many",
            &serde_json::json!({ "reqs": reqs }),
        )
    };
    let length = |answer: &Value, key: &str| answer[key].as_array().map_or(0, Vec::len);

    let placidus = batch(&[0, 0]);
    assert_eq!(length(&placidus, "cusps"), 24, "{placidus}");
    assert_eq!(length(&placidus, "out_angles"), 2, "{placidus}");

    let mixed = batch(&[0, 12]);
    assert_eq!(length(&mixed, "cusps"), 72, "{mixed}");
    let single = called(
        &provider,
        "tm_houses_calc",
        &serde_json::json!({ "req": request(0) }),
    );
    assert_eq!(
        mixed["cusps"]
            .as_array()
            .and_then(|cusps| cusps.get(..12))
            .map(<[Value]>::to_vec),
        single["cusps"].as_array().cloned(),
        "the Placidus chart's cusps open its slot"
    );
}
