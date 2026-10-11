//! Prompts as worked requests (`03-design/mcp-server.md` §7, P4): the
//! call a prompt writes is made and answers, each required argument is
//! refused by name when left out, a bad one is named as the argument it
//! came from, and a prompt's argument completes from its table.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the replies they expect"
)]

use serde_json::{Map, Value, json};
use teistro_mcp::{Engine, Server};

fn ask(server: &mut Server, method: &str, mut params: Value) -> Value {
    params["_meta"] = json!({ "io.modelcontextprotocol/protocolVersion": "2026-07-28",
        "io.modelcontextprotocol/clientCapabilities": {} });
    let message = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
    serde_json::from_str(&server.handle(&message.to_string()).unwrap()).unwrap()
}

fn get(server: &mut Server, name: &str, arguments: &Value) -> Value {
    ask(
        server,
        "prompts/get",
        json!({ "name": name, "arguments": arguments }),
    )
}

/// An example of every prompt's arguments, each required one given.
fn examples() -> Vec<(&'static str, Value)> {
    vec![
        (
            "birth-chart",
            json!({ "date": "1990-04-05", "time": "04:30", "zone": "Asia/Kathmandu",
                    "latitude": "27.7172", "longitude": "85.324", "altitude": "1400",
                    "sections": "shadbala, jaimini" }),
        ),
        (
            "day-panchanga",
            json!({ "date": "2026-10-10", "last": "2026-10-12", "zone": "+05:45",
                    "latitude": "27.7172", "longitude": "85.324", "festivals": "NEPAL" }),
        ),
        (
            "match",
            json!({ "groomDate": "1990-04-05", "groomTime": "04:30",
                    "groomZone": "Asia/Kathmandu", "groomLatitude": "27.7172",
                    "groomLongitude": "85.324", "brideDate": "1992-01-20",
                    "brideTime": "18:10", "brideZone": "Asia/Kolkata",
                    "brideLatitude": "28.6139", "brideLongitude": "77.209" }),
        ),
    ]
}

/// The tool and arguments a prompt's first message writes.
fn written_call(prompt: &Value) -> (String, Value) {
    let text = prompt["result"]["messages"][0]["content"]["text"]
        .as_str()
        .unwrap_or_else(|| panic!("{prompt}"));
    let tool = text
        .split("Call the tool `")
        .nth(1)
        .and_then(|rest| rest.split('`').next())
        .unwrap();
    let block = text
        .split("```json\n")
        .nth(1)
        .unwrap()
        .split("\n```")
        .next()
        .unwrap();
    (tool.to_owned(), serde_json::from_str(block).unwrap())
}

/// Every prompt is listed with an example here, and every example is a
/// listed prompt.
#[test]
fn every_prompt_has_an_example_and_every_example_a_prompt() {
    let mut server = Server::new(Engine::None);
    let listed = ask(&mut server, "prompts/list", json!({}));
    assert_eq!(listed["result"]["cacheScope"], "public");
    let mut names: Vec<&str> = listed["result"]["prompts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|prompt| prompt["name"].as_str().unwrap())
        .collect();
    names.sort_unstable();
    let mut examples: Vec<&str> = examples().iter().map(|(name, _)| *name).collect();
    examples.sort_unstable();
    assert_eq!(names, examples);
}

/// **A prompt's call works** (P4): the call each prompt writes, made as
/// written, answers without a refusal, and links the tool's schemas.
#[test]
fn the_call_a_prompt_writes_answers() {
    let mut server = Server::new(Engine::Builtin);
    for (name, arguments) in examples() {
        let prompt = get(&mut server, name, &arguments);
        let link = &prompt["result"]["messages"][1]["content"];
        assert_eq!(link["type"], "resource_link", "{name}: {prompt}");
        let (tool, call) = written_call(&prompt);
        assert_eq!(link["uri"], format!("teistro://tools/{tool}/schema"));
        let reply = ask(
            &mut server,
            "tools/call",
            json!({ "name": tool, "arguments": call }),
        );
        assert_eq!(reply["result"]["isError"], false, "{name}: {reply}");
    }
}

/// **The instant is `time.resolve`'s** : a birth chart's instant and
/// clock are the ones `time.resolve` answers for its date, time and zone.
#[test]
fn a_birth_charts_instant_is_the_one_time_resolve_answers() {
    let mut server = Server::new(Engine::None);
    let (_, arguments) = examples().remove(0);
    let (_, call) = written_call(&get(&mut server, "birth-chart", &arguments));
    let resolved = ask(
        &mut server,
        "tools/call",
        json!({ "name": "time.resolve", "arguments": { "request": {
            "date": { "year": 1990, "month": 4, "day": 5 },
            "time": { "hour": 4, "minute": 30, "second": 0 },
            "zone": { "kind": "IANA", "zone": "Asia/Kathmandu" } } } }),
    );
    let value = &resolved["result"]["structuredContent"]["value"];
    assert_eq!(call["request"]["instant"], value["instant"]);
    assert_eq!(call["request"]["utcOffsetSeconds"], value["zone"]["offset"]);
    assert_eq!(call["request"]["shadbala"], true);
    assert_eq!(call["request"]["altitudeM"], 1400.0);
}

/// Each required argument left out is refused by its name, and an
/// argument no prompt reads is refused by its own.
#[test]
fn a_missing_or_unknown_argument_is_refused_by_name() {
    let mut server = Server::new(Engine::None);
    let listed = ask(&mut server, "prompts/list", json!({}));
    for prompt in listed["result"]["prompts"].as_array().unwrap() {
        let name = prompt["name"].as_str().unwrap();
        let example = examples()
            .into_iter()
            .find(|(example, _)| *example == name)
            .unwrap()
            .1;
        for argument in prompt["arguments"].as_array().unwrap() {
            let argument_name = argument["name"].as_str().unwrap();
            let mut fewer: Map<String, Value> = example.as_object().unwrap().clone();
            fewer.remove(argument_name);
            let reply = get(&mut server, name, &Value::Object(fewer));
            if argument["required"] == true {
                assert_eq!(
                    reply["error"]["code"], -32_602,
                    "{name} without {argument_name}"
                );
                assert_eq!(reply["error"]["data"]["argument"], argument_name, "{reply}");
            } else {
                assert!(
                    reply.get("result").is_some(),
                    "{name} without {argument_name}: {reply}"
                );
            }
        }
        let mut more = example.as_object().unwrap().clone();
        more.insert(String::from("colour"), json!("red"));
        let reply = get(&mut server, name, &Value::Object(more));
        assert_eq!(reply["error"]["data"]["argument"], "colour", "{reply}");
    }
    let reply = get(&mut server, "horoscope", &json!({}));
    assert_eq!(reply["error"]["code"], -32_602, "{reply}");
}

/// **A bad argument is named as written** (P4): whichever reader refuses
/// it, `time.resolve`'s or the chart request's, the refusal names the
/// prompt's argument rather than the record's field.
#[test]
fn a_bad_argument_is_named_as_the_prompt_spells_it() {
    let mut server = Server::new(Engine::None);
    let (_, birth) = examples().remove(0);
    let (_, couple) = examples().remove(2);
    for (name, base, argument, value) in [
        ("birth-chart", &birth, "date", "1990-13-05"),
        ("birth-chart", &birth, "date", "5 April 1990"),
        ("birth-chart", &birth, "time", "25:00"),
        ("birth-chart", &birth, "time", "half past four"),
        ("birth-chart", &birth, "zone", "Mars/Olympus"),
        ("birth-chart", &birth, "zone", "+5"),
        ("birth-chart", &birth, "latitude", "97"),
        ("birth-chart", &birth, "longitude", "east"),
        ("birth-chart", &birth, "sections", "shadbala, horoscope"),
        ("match", &couple, "brideDate", "1992-02-30"),
        ("match", &couple, "brideTime", "18:61"),
        ("match", &couple, "groomZone", "Nowhere/City"),
        ("match", &couple, "brideLatitude", "-91"),
    ] {
        let mut arguments = base.clone();
        arguments[argument] = json!(value);
        let reply = get(&mut server, name, &arguments);
        assert_eq!(
            reply["error"]["code"], -32_602,
            "{argument} = {value}: {reply}"
        );
        assert_eq!(
            reply["error"]["data"]["argument"], argument,
            "{argument} = {value}: {reply}"
        );
    }
}

/// A prompt's argument completes from its table: a list's last item from
/// those not yet named, a pack from the shipped ones, and a person's own
/// text not at all.
#[test]
fn a_prompts_argument_completes_from_its_table() {
    let mut server = Server::new(Engine::None);
    let mut complete = |prompt: &str, argument: &str, typed: &str| -> Value {
        ask(
            &mut server,
            "completion/complete",
            json!({ "ref": { "type": "ref/prompt", "name": prompt },
                    "argument": { "name": argument, "value": typed } }),
        )
    };
    let sections = complete("birth-chart", "sections", "shadbala, jai");
    assert_eq!(
        sections["result"]["completion"]["values"],
        json!(["shadbala, jaimini"]),
        "{sections}"
    );
    let named = complete("birth-chart", "sections", "shadbala, ");
    let values = named["result"]["completion"]["values"].as_array().unwrap();
    assert!(!values.contains(&json!("shadbala, shadbala")), "{named}");
    assert_eq!(values.len(), teistro::FoundRequest::sections().len() - 1);
    let packs = complete("day-panchanga", "festivals", "");
    assert_eq!(
        packs["result"]["completion"]["values"],
        json!(["DHARMASINDHU", "NEPAL"])
    );
    let date = complete("birth-chart", "date", "1990");
    assert_eq!(date["result"]["completion"]["values"], json!([]), "{date}");
    let unknown = complete("birth-chart", "colour", "");
    assert_eq!(
        unknown["error"]["data"]["field"], "argument.name",
        "{unknown}"
    );
    let nothing = complete("horoscope", "date", "");
    assert_eq!(nothing["error"]["data"]["field"], "ref.name", "{nothing}");
}

/// Every pack a completion offers is one the prompt takes.
#[test]
fn every_offered_pack_makes_a_prompt() {
    let mut server = Server::new(Engine::None);
    let offered = ask(
        &mut server,
        "completion/complete",
        json!({ "ref": { "type": "ref/prompt", "name": "day-panchanga" },
                "argument": { "name": "festivals", "value": "" } }),
    );
    let (_, mut arguments) = examples().remove(1);
    for pack in offered["result"]["completion"]["values"]
        .as_array()
        .unwrap()
    {
        arguments["festivals"] = pack.clone();
        let reply = get(&mut server, "day-panchanga", &arguments);
        assert!(reply.get("result").is_some(), "{pack}: {reply}");
    }
}
