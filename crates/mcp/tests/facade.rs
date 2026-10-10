//! A tool answers as the façade does (`03-design/mcp-server.md` §5, the
//! second gate): every record tool is called with an example of its
//! record, and its structured content is the façade's own answer, value
//! and provenance, input hash included.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the replies they expect"
)]

use serde_json::{Value, json};
use teistro::{Context, Ephemeris};
use teistro_mcp::{Adapter, Detail, Engine, Server};

/// The engine passthrough's tools ask an engine with operations of its
/// own, which the built-in is not: they run over the test adapter.
fn asks_an_engine(name: &str) -> bool {
    name.starts_with("engine.")
}

/// The in-repo test adapter, loaded.
fn test_adapter() -> Adapter {
    let path = teistro_test_adapter::library().unwrap();
    // SAFETY: the library is this workspace's own test adapter.
    #[allow(unsafe_code, reason = "loading the workspace's own test adapter")]
    unsafe { Adapter::load(&path.to_string_lossy(), None) }.unwrap()
}

/// A server over the test adapter with the built-in behind it, and a
/// context built the same way for the façade's side.
fn over_the_test_adapter() -> (Server, Context) {
    let adapter = test_adapter();
    let context = Context::builder()
        .ephemeris([
            Ephemeris::Provider(Box::new(adapter.provider().unwrap())),
            Ephemeris::Builtin,
        ])
        .build()
        .unwrap();
    (
        Server::with_plugin(Engine::Builtin, adapter).unwrap(),
        context,
    )
}

/// That the tool `name` answers `request` as the façade answers it under
/// `context`: the value, and the provenance where it seals one.
fn answers_as_the_facade(name: &str, result: &Value, context: &Context, request: &Value) {
    assert_eq!(result["isError"], false, "{name}: {result}");
    let answered = teistro::records::record(name)
        .unwrap()
        .answer(context, &request.to_string())
        .unwrap();
    let structured = &result["structuredContent"];
    assert_eq!(structured["value"], answered.value, "{name}");
    match answered.provenance {
        Some(provenance) => {
            assert_eq!(
                structured["provenance"],
                serde_json::to_value(provenance).unwrap()
            );
        }
        None => assert!(structured.get("provenance").is_none(), "{name}"),
    }
}

/// One record of each tool's, the examples its area's own documentation
/// and tests send.
fn examples() -> Vec<(&'static str, Value)> {
    let birth = |instant: f64| {
        json!({"instant": instant, "latitudeDeg": 27.7172, "longitudeDeg": 85.324,
            "altitudeM": 1400, "utcOffsetSeconds": 20700, "uncertaintyMinutes": 0})
    };
    vec![
        (
            "chart.found",
            json!({"instants": [2_447_000.25, 2_447_365.75], "latitudeDeg": 27.7172,
                "longitudeDeg": 85.324, "altitudeM": 1400, "utcOffsetSeconds": 20700,
                "vargas": ["D9"], "shadbala": true, "fortitudes": {}, "lots": {},
                "kp": {"anyAyanamsha": true}, "antiscia": {}, "varsha": {"through": 1}}),
        ),
        (
            "almanac.days",
            json!({"first": {"year": 2026, "month": 10, "day": 10},
                "last": {"year": 2026, "month": 10, "day": 12},
                "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400,
                "utcOffsetSeconds": 20700, "muhurta": {"rules": "RAMAN_MARRIAGE"},
                "festivals": {"rules": "DHARMASINDHU"}, "years": true, "eclipses": true,
                "nepalSambat": true}),
        ),
        ("matching.naam", json!({"bride": "सीता", "groom": "राम"})),
        (
            "time.resolve",
            json!({"date": {"year": 1990, "month": 4, "day": 5},
                "time": {"hour": 4, "minute": 30, "second": 0},
                "zone": {"kind": "IANA", "zone": "Asia/Kathmandu"}}),
        ),
        (
            "time.civil",
            json!({"instant": 2_447_986.458_333_333_5, "zone": {"kind": "FIXED", "offset": 20700},
                "calendar": "BIKRAM_SAMBAT"}),
        ),
        (
            "time.convert",
            json!({"jd": 2_451_545.0, "from": "UTC", "to": "TT"}),
        ),
        (
            "calendar.convert",
            json!({"date": {"year": 2026, "month": 10, "day": 10}, "into": "calendar.BIKRAM_SAMBAT"}),
        ),
        (
            "engine.call",
            json!({"name": "tp_sum", "arguments": {"values": [1, 2.5]}}),
        ),
        ("engine.manifest", json!({})),
        (
            "numerology.profile",
            json!({"name": "Henry Elder", "date": {"year": 1872, "month": 1, "day": 17},
                "rules": {"masters": "NONE"}}),
        ),
        (
            "almanac.pakshi",
            json!({"first": {"year": 1984, "month": 10, "day": 30},
                "last": {"year": 1984, "month": 11, "day": 1},
                "latitudeDeg": 13.0827, "longitudeDeg": 80.2707, "altitudeM": 6,
                "utcOffsetSeconds": 19800, "native": {"bird": "COCK"}}),
        ),
        (
            "chart.rashifal",
            json!({"periods": [{"first": {"year": 2026, "month": 10, "day": 4},
                "last": {"year": 2026, "month": 10, "day": 10},
                "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400,
                "utcOffsetSeconds": 20700}], "baseline": "WEEKLY"}),
        ),
        (
            "research.study",
            json!({"study": "COUNTS", "rules": {"shipped": ["YOGAS"]}, "holds": "FORMED",
                "births": [birth(2_446_000.25), birth(2_447_100.5), birth(2_448_300.75)],
                "design": {"groups": [0, 1, 1]}}),
        ),
    ]
}

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

#[test]
fn every_record_tool_has_an_example_and_every_example_a_tool() {
    let mut tools: Vec<&str> = teistro::records::records().iter().map(|r| r.name).collect();
    let mut examples: Vec<&str> = examples().iter().map(|(name, _)| *name).collect();
    tools.sort_unstable();
    examples.sort_unstable();
    assert_eq!(tools, examples);
}

#[test]
fn a_tool_answers_as_the_facade_does() {
    let mut server = Server::new(Engine::Builtin);
    for profile in [None, Some("conformance-baseline")] {
        let mut builder = Context::builder().ephemeris([Ephemeris::Builtin]);
        if let Some(profile) = profile {
            builder = builder.profile(profile);
        }
        let context = builder.build().unwrap();
        for (name, request) in examples() {
            if asks_an_engine(name) {
                continue;
            }
            let mut arguments = json!({"request": request});
            if let Some(profile) = profile {
                arguments["profile"] = json!(profile);
            }
            let result = call(&mut server, name, &arguments);
            answers_as_the_facade(name, &result, &context, &request);
        }
    }
}

#[test]
fn an_engine_tool_answers_as_the_facade_does_over_a_loaded_adapter() {
    let (mut server, context) = over_the_test_adapter();
    for (name, request) in examples() {
        if asks_an_engine(name) {
            let result = call(&mut server, name, &json!({"request": request}));
            answers_as_the_facade(name, &result, &context, &request);
        }
    }
    // An operation's own tool is `engine.call` with its arguments.
    let arguments = json!({"values": [1, 2.5]});
    let result = call(&mut server, "engine.tp_sum", &arguments);
    let request = json!({"name": "tp_sum", "arguments": arguments});
    answers_as_the_facade("engine.call", &result, &context, &request);
    assert_eq!(result["structuredContent"]["value"]["total"], 3.5);
    // And the SDK's own tools compute on the adapter's sky.
    let found = call(
        &mut server,
        "chart.found",
        &json!({"request": examples()[0].1}),
    );
    assert_eq!(found["isError"], false, "{found}");
}

#[test]
fn each_operation_of_a_loaded_engine_is_a_tool_with_its_parameters() {
    let (mut server, _) = over_the_test_adapter();
    let reply = server
        .handle(
            &json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {
                "_meta": {"io.modelcontextprotocol/protocolVersion": "2026-07-28", "io.modelcontextprotocol/clientCapabilities": {}}}})
            .to_string(),
        )
        .unwrap();
    let reply: Value = serde_json::from_str(&reply).unwrap();
    let tools = reply["result"]["tools"].as_array().unwrap();
    let sum = tools
        .iter()
        .find(|tool| tool["name"] == "engine.tp_sum")
        .unwrap();
    let properties = sum["inputSchema"]["properties"].as_object().unwrap();
    let mut names: Vec<&str> = properties.keys().map(String::as_str).collect();
    names.sort_unstable();
    assert_eq!(
        names,
        ["count", "values"],
        "the out parameter is the engine's to fill"
    );
    assert_eq!(properties["values"]["type"], "array");
    assert!(tools.iter().any(|tool| tool["name"] == "engine.tp_echo"));

    // Without a plugin, no engine operation is a tool.
    let reply = Server::new(Engine::Builtin)
        .handle(
            &json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {
                "_meta": {"io.modelcontextprotocol/protocolVersion": "2026-07-28", "io.modelcontextprotocol/clientCapabilities": {}}}})
            .to_string(),
        )
        .unwrap();
    assert!(!reply.contains("engine.tp_"));
}

#[test]
fn a_tool_reads_the_settings_and_the_locale_it_names() {
    let mut server = Server::new(Engine::Builtin);
    let (name, request) = examples()
        .into_iter()
        .find(|(name, _)| *name == "almanac.pakshi")
        .unwrap();
    let centred = |server: &mut Server, centre: &str| {
        let result = call(
            server,
            name,
            &json!({"request": request, "settings": {"frame": {"centre": centre}}}),
        );
        assert_eq!(result["isError"], false, "{result}");
        result["structuredContent"]["provenance"]["settings_hash"].clone()
    };
    let geocentric = centred(&mut server, "GEOCENTRIC");
    assert_ne!(
        geocentric,
        centred(&mut server, "TOPOCENTRIC"),
        "the patch is applied"
    );
    assert_eq!(
        geocentric,
        centred(&mut server, "GEOCENTRIC"),
        "and the context kept"
    );
    let unknown = call(
        &mut server,
        name,
        &json!({"request": request, "locale": "xx-Zzzz"}),
    );
    assert_eq!(unknown["isError"], true);
    assert_eq!(unknown["structuredContent"]["field"], "locale");
}

#[test]
fn every_record_tool_is_described_by_its_boundary_function() {
    let mut server = Server::new(Engine::None);
    let reply = server
        .handle(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{}}}}"#)
        .unwrap();
    let reply: Value = serde_json::from_str(&reply).unwrap();
    for tool in reply["result"]["tools"].as_array().unwrap() {
        let description = tool["description"].as_str().unwrap();
        assert_eq!(tool["annotations"]["readOnlyHint"], true);
        assert_eq!(tool["inputSchema"]["type"], "object");
        if !tool["name"].as_str().unwrap().ends_with(".describe") {
            assert!(
                description.contains("`request` is"),
                "{}: {description}",
                tool["name"]
            );
            assert!(!description.contains("request_json"), "{}", tool["name"]);
        }
    }
}

#[test]
fn settings_describe_answers_the_profiles_and_the_patch_schema() {
    let mut server = Server::new(Engine::None);
    let result = call(&mut server, "settings.describe", &json!({}));
    assert_eq!(result["isError"], false);
    let described = &result["structuredContent"];
    assert_eq!(
        described["defaultProfile"],
        teistro::settings::DEFAULT_PROFILE
    );
    let ids: Vec<&str> = described["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|profile| profile["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, teistro::settings::SHIPPED_PROFILES);
    let schema = &described["settings"];
    assert!(schema["$schema"].as_str().unwrap().contains("2020-12"));
    assert!(schema["properties"]["frame"].is_object(), "{schema}");
    let refused = call(
        &mut server,
        "settings.describe",
        &json!({"profile": "kp-default"}),
    );
    assert_eq!(refused["isError"], true);
    assert_eq!(refused["structuredContent"]["field"], "arguments.profile");
}

#[test]
fn a_chart_request_answers_its_charts_and_a_row_a_chart_of_each_table_asked() {
    let mut server = Server::new(Engine::Builtin);
    let (name, request) = examples()
        .into_iter()
        .find(|(name, _)| *name == "chart.found")
        .unwrap();
    let result = call(&mut server, name, &json!({"request": request}));
    let value = &result["structuredContent"]["value"];
    assert_eq!(value["charts"].as_array().unwrap().len(), 2);
    for asked in ["fortitudes", "lots", "kp", "varsha"] {
        assert_eq!(value[asked].as_array().unwrap().len(), 2, "{asked}");
    }
    assert_eq!(value["western"]["antiscia"].as_array().unwrap().len(), 2);
    assert_eq!(value["gochar"], json!([]));
    let input_hash = &result["structuredContent"]["provenance"]["input_hash"];
    assert!(input_hash.is_string(), "{input_hash}");
}

#[test]
fn a_days_request_answers_its_days_and_each_section_sealed_as_the_boundary_seals_it() {
    let mut server = Server::new(Engine::Builtin);
    let (name, request) = examples()
        .into_iter()
        .find(|(name, _)| *name == "almanac.days")
        .unwrap();
    let result = call(&mut server, name, &json!({"request": request}));
    let value = &result["structuredContent"]["value"];
    assert_eq!(value["days"].as_array().unwrap().len(), 3);
    assert_eq!(value["dayHashes"].as_array().unwrap().len(), 3);
    for section in ["muhurta", "festivals", "years", "eclipses", "nepalSambat"] {
        let hash = &value[section]["provenance"]["content_hash"];
        assert!(hash.is_string(), "{section}: {hash}");
    }
    // A section not asked is left out rather than answered empty.
    let mut plain = request.clone();
    for asked in ["muhurta", "festivals", "years", "eclipses", "nepalSambat"] {
        plain.as_object_mut().unwrap().remove(asked);
    }
    let result = call(&mut server, name, &json!({"request": plain}));
    let keys: Vec<&String> = result["structuredContent"]["value"]
        .as_object()
        .unwrap()
        .keys()
        .collect();
    assert_eq!(keys, ["days", "dayHashes"]);
}

#[test]
fn a_civil_time_resolves_and_reads_back_on_the_same_clock() {
    let mut server = Server::new(Engine::None);
    let resolved = call(
        &mut server,
        "time.resolve",
        &json!({"request": {"date": {"year": 1990, "month": 4, "day": 5},
            "time": {"hour": 4, "minute": 30, "second": 0},
            "zone": {"kind": "IANA", "zone": "Asia/Kathmandu"}}}),
    );
    let resolved = &resolved["structuredContent"]["value"];
    assert_eq!(resolved["zone"]["offset"], json!(20700));
    let back = call(
        &mut server,
        "time.civil",
        &json!({"request": {"instant": resolved["instant"],
            "zone": {"kind": "IANA", "zone": "Asia/Kathmandu"}}}),
    );
    // The date and the time read back; the era is a view the reading
    // adds, which a date as given does not carry.
    let civil = &back["structuredContent"]["value"]["civil"];
    for part in ["calendar", "year", "month", "day"] {
        assert_eq!(
            civil["date"][part], resolved["civil"]["date"][part],
            "{part}"
        );
    }
    assert_eq!(civil["time"], resolved["civil"]["time"]);
    let converted = call(
        &mut server,
        "calendar.convert",
        &json!({"request": {"date": {"year": 2026, "month": 10, "day": 10},
            "into": "BIKRAM_SAMBAT"}}),
    );
    assert_eq!(
        converted["structuredContent"]["value"]["weekday"],
        json!("SATURDAY")
    );
}

/// Every tool the server lists, by name.
fn listed(server: &mut Server) -> Vec<Value> {
    let reply = server
        .handle(
            &json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {
                "_meta": {"io.modelcontextprotocol/protocolVersion": "2026-07-28", "io.modelcontextprotocol/clientCapabilities": {}}}})
            .to_string(),
        )
        .unwrap();
    let reply: Value = serde_json::from_str(&reply).unwrap();
    reply["result"]["tools"].as_array().unwrap().clone()
}

/// That `instance` holds to `schema`, a whole JSON Schema 2020-12.
fn holds(schema: &Value, instance: &Value) -> Result<(), String> {
    let mut schemas = boon::Schemas::new();
    let mut compiler = boon::Compiler::new();
    compiler
        .add_resource("urn:teistro:tool", schema.clone())
        .map_err(|error| format!("{error:#}"))?;
    let index = compiler
        .compile("urn:teistro:tool", &mut schemas)
        .map_err(|error| format!("the schema does not compile: {error:#}"))?;
    schemas
        .validate(instance, index)
        .map_err(|error| format!("{error:#}"))
}

/// **What a schema says, the tool takes and answers** (P1): every
/// example's arguments hold to its tool's `inputSchema`, and its answer
/// to the `outputSchema`, over the built-in and over a loaded adapter,
/// in full and in the lean list a model reads.
#[test]
fn every_example_and_its_answer_hold_to_the_tools_schemas() {
    let (plugged, _) = over_the_test_adapter();
    let mut plugged = plugged.with_detail(Detail::Full);
    let mut builtin = Server::new(Engine::Builtin).with_detail(Detail::Full);
    let lean = listed(&mut Server::new(Engine::Builtin));
    for (name, request) in examples() {
        let server = if asks_an_engine(name) {
            &mut plugged
        } else {
            &mut builtin
        };
        let tool = listed(server)
            .into_iter()
            .find(|tool| tool["name"] == name)
            .unwrap();
        let arguments = json!({"request": request});
        holds(&tool["inputSchema"], &arguments)
            .unwrap_or_else(|why| panic!("{name}'s arguments: {why}"));
        let leaner = lean.iter().find(|tool| tool["name"] == name).unwrap();
        holds(&leaner["inputSchema"], &arguments)
            .unwrap_or_else(|why| panic!("{name}'s arguments, lean: {why}"));
        let result = call(server, name, &arguments);
        assert_eq!(result["isError"], false, "{name}: {result}");
        if let Some(output) = tool.get("outputSchema") {
            holds(output, &result["structuredContent"])
                .unwrap_or_else(|why| panic!("{name}'s answer: {why}"));
        }
    }
}

/// Every tool states its answer and its record, every part of the record
/// included: a part whose reader reads no single type says so in its
/// schema, and none is left.
#[test]
fn every_tool_states_its_schemas_and_the_unstated_parts_are_exactly_these() {
    let tools = listed(&mut Server::new(Engine::Builtin).with_detail(Detail::Full));
    let mut unstated: Vec<String> = Vec::new();
    for tool in &tools {
        let name = tool["name"].as_str().unwrap();
        if name.ends_with(".describe") {
            continue;
        }
        assert!(
            tool.get("outputSchema").is_some(),
            "{name} states no answer"
        );
        let request = &tool["inputSchema"]["properties"]["request"];
        assert!(
            request.get("$ref").is_some() || request.get("properties").is_some(),
            "{name} states no record: {request}"
        );
        let text = tool["inputSchema"].to_string();
        for (at, _) in text.match_indices(": its schema is not stated yet") {
            let named = &text[..at];
            let start = named.rfind("The `").unwrap() + "The `".len();
            let end = named[start..].find('`').unwrap() + start;
            unstated.push(format!("{name}:{}", &named[start..end]));
        }
    }
    assert_eq!(unstated, Vec::<String>::new());
}

/// **What a chart record's reader takes, `chart.found`'s schema takes**
/// (P1, "never stricter than the reader"): each record by its name, at
/// the least its reader needs and, where the record has more to say, in
/// full, read by the record's own reader and held to the tool's schema.
#[test]
fn every_chart_record_the_reader_takes_the_schema_takes() {
    use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
    use teistro::{ChartRecords, Partner, UtcOffset};

    let partner = serde_json::to_value(Partner {
        instant: JulianDay::<Utc>::try_new(2_447_892.5).unwrap(),
        place: Place::new(
            Latitude::literal(27.7172),
            Longitude::literal(85.324),
            Altitude::literal(1400.0),
        ),
        utc_offset: UtcOffset::try_from_seconds(20_700).unwrap(),
    })
    .unwrap();
    let least = |name: &str| match name {
        "gochar" => json!({"instants": [2_460_676.5]}),
        "hits" | "sadeSati" => json!({"from": 2_460_676.5, "to": 2_460_680.5}),
        "perfection" => json!({"querent": "VENUS", "quesited": "MARS"}),
        "synastry" => json!({"partner": partner}),
        "matching" => json!({"partner": partner, "partnerRole": "BRIDE"}),
        "varsha" => json!({"through": 1}),
        "progressions" => json!({"at": 2_460_676.5}),
        "harmonic" => json!({"number": 5}),
        _ => json!({}),
    };
    let full = [
        (
            "synastry",
            json!({"partner": partner, "davison": true, "lagna": false}),
        ),
        (
            "matching",
            json!({"partner": partner, "partnerRole": "GROOM"}),
        ),
        (
            "varsha",
            json!({"through": 2, "place": "birth", "matters": [1, 7], "sahams": "all"}),
        ),
        ("theme", json!({"extends": "DARK", "style": {}})),
        ("interpret", json!({"placements": true})),
        ("kp", json!({"clock": 20_700})),
    ];
    let mut server = Server::new(Engine::Builtin).with_detail(Detail::Full);
    let tool = listed(&mut server)
        .into_iter()
        .find(|tool| tool["name"] == "chart.found")
        .unwrap();
    let records = ChartRecords::NAMES
        .iter()
        .map(|name| (*name, least(name)))
        .chain(full);
    for (name, record) in records {
        ChartRecords::default()
            .read(name, &record.to_string())
            .unwrap_or_else(|why| panic!("{name}'s reader refuses {record}: {why}"));
        let mut request = examples()[0].1.clone();
        request[name] = record.clone();
        holds(&tool["inputSchema"], &json!({"request": request}))
            .unwrap_or_else(|why| panic!("{name}: the reader takes {record}, the schema: {why}"));
        let part = call(
            &mut server,
            "schema.describe",
            &json!({"tool": "chart.found", "part": name}),
        );
        holds(&part["structuredContent"]["schema"], &record)
            .unwrap_or_else(|why| panic!("{name}: the reader takes {record}, its part: {why}"));
    }
}

/// **A lean list is one a model can read** (P1): no answer schema, each
/// record a request carries by name one line naming `schema.describe`,
/// every definition kept reached, and the whole a small fraction of the
/// full list.
#[test]
fn the_lean_list_names_each_part_and_keeps_only_what_it_reaches() {
    let lean = listed(&mut Server::new(Engine::Builtin));
    let full = listed(&mut Server::new(Engine::Builtin).with_detail(Detail::Full));
    for tool in &lean {
        let name = tool["name"].as_str().unwrap();
        assert!(
            tool.get("outputSchema").is_none(),
            "{name} lists its answer"
        );
        let schema = &tool["inputSchema"];
        let text = schema.to_string();
        for (defined, _) in schema["$defs"].as_object().into_iter().flatten() {
            assert!(
                text.contains(&format!("\"#/$defs/{defined}\"")),
                "{name} keeps `{defined}`, which nothing reaches"
            );
        }
    }
    let found = lean
        .iter()
        .find(|tool| tool["name"] == "chart.found")
        .unwrap();
    let kp = &found["inputSchema"]["properties"]["request"]["properties"]["kp"];
    assert!(
        kp["description"]
            .as_str()
            .unwrap()
            .contains("schema.describe"),
        "{kp}"
    );
    let size = |tools: &[Value]| Value::Array(tools.to_vec()).to_string().len();
    let (lean, full) = (size(&lean), size(&full));
    assert!(lean * 4 < full, "lean {lean} bytes, full {full}");
}

/// **`schema.describe` answers what the full list states** (P1), and
/// refuses a tool or a part by the field that names it.
#[test]
fn schema_describe_answers_the_full_schemas_and_refuses_by_field() {
    let mut server = Server::new(Engine::Builtin);
    let full = listed(&mut Server::new(Engine::Builtin).with_detail(Detail::Full));
    for tool in full
        .iter()
        .filter(|tool| !tool["name"].as_str().unwrap().ends_with(".describe"))
    {
        let name = &tool["name"];
        let described = call(&mut server, "schema.describe", &json!({"tool": name}));
        assert_eq!(described["isError"], false, "{name}: {described}");
        let described = &described["structuredContent"];
        assert_eq!(described["input"], tool["inputSchema"], "{name}");
        assert_eq!(described["output"], tool["outputSchema"], "{name}");
    }
    for (arguments, field) in [
        (json!({}), "arguments.tool"),
        (json!({"tool": "chart.nothing"}), "arguments.tool"),
        (
            json!({"tool": "chart.found", "part": "nothing"}),
            "arguments.part",
        ),
        (
            json!({"tool": "matching.naam", "part": "kp"}),
            "arguments.part",
        ),
        (json!({"tool": "chart.found", "part": 1}), "arguments.part"),
        (json!({"tool": "chart.found", "also": 1}), "arguments.also"),
    ] {
        let refused = call(&mut server, "schema.describe", &arguments);
        assert_eq!(refused["isError"], true, "{arguments}: {refused}");
        assert_eq!(
            refused["structuredContent"]["field"], field,
            "{arguments}: {refused}"
        );
    }
}
