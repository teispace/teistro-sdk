//! Prompts as worked requests (`03-design/mcp-server.md` §7, P4): a
//! request the SDK answers well, written out from what a person supplies
//! (a date, a time, a zone, a place). The civil time is resolved through
//! `time.resolve` and the record read by its tool's own reader before the
//! prompt is answered, so a model is handed a call that works and the
//! record it learns from is the one the reader takes.

use std::collections::BTreeMap;

use serde_json::{Map, Value, json};
use teistro::Error;
use teistro::quantity::Place;

use crate::{Limits, resources};

/// The arguments a person gave, each trimmed.
type Given = BTreeMap<String, String>;

/// Asks a record tool by name, answering its value.
pub(crate) type Ask<'a> = dyn FnMut(&str, &Value) -> Result<Value, Error> + 'a;

/// Why a prompt is refused: the message, and the argument it names.
pub(crate) struct Refused {
    pub(crate) message: String,
    pub(crate) argument: Option<String>,
}

impl Refused {
    fn of(argument: impl Into<String>, message: impl Into<String>) -> Refused {
        Refused {
            message: message.into(),
            argument: Some(argument.into()),
        }
    }
}

/// The values an argument takes, read from the table that holds them.
type Values = fn() -> Vec<&'static str>;

/// One argument a person supplies, as text.
struct Argument {
    name: &'static str,
    description: &'static str,
    required: bool,
    /// The values it takes where a table holds them, which a completion
    /// offers, and whether it takes several, comma-separated.
    values: Option<(Values, bool)>,
}

const fn required(name: &'static str, description: &'static str) -> Argument {
    Argument {
        name,
        description,
        required: true,
        values: None,
    }
}

const fn optional(name: &'static str, description: &'static str) -> Argument {
    Argument {
        name,
        description,
        required: false,
        values: None,
    }
}

impl Argument {
    /// The same argument, taking the values `values` gives; several,
    /// comma-separated, where `list`.
    const fn from(self, values: Values, list: bool) -> Argument {
        Argument {
            values: Some((values, list)),
            ..self
        }
    }
}

fn sections() -> Vec<&'static str> {
    teistro::FoundRequest::sections().to_vec()
}

fn festival_packs() -> Vec<&'static str> {
    teistro::FestivalPack::ALL
        .iter()
        .map(|pack| pack.key())
        .collect()
}

/// What a prompt's argument takes, for a completion.
pub(crate) enum Takes {
    /// No prompt has the name.
    NoPrompt,
    /// The prompt reads no such argument.
    NoArgument,
    /// A person's own text, which no table holds.
    Text,
    /// One of these, or several comma-separated where `list`.
    Values {
        values: Vec<&'static str>,
        list: bool,
    },
}

/// What the prompt `name`'s argument `argument` takes.
pub(crate) fn takes(name: &str, argument: &str) -> Takes {
    let Some(prompt) = PROMPTS.iter().find(|prompt| prompt.name == name) else {
        return Takes::NoPrompt;
    };
    match prompt.arguments.iter().find(|known| known.name == argument) {
        None => Takes::NoArgument,
        Some(Argument {
            values: Some((values, list)),
            ..
        }) => Takes::Values {
            values: values(),
            list: *list,
        },
        Some(_) => Takes::Text,
    }
}

const DATE: &str = "The civil date, YYYY-MM-DD, in the Gregorian calendar.";
const TIME: &str = "The civil time of day, HH:MM or HH:MM:SS, on the zone's clock.";
const ZONE: &str = "The zone the time is read in: an IANA name (Asia/Kathmandu), a fixed \
                    offset (+05:45, UTC), or LMT for local mean time at the longitude.";
const LATITUDE: &str = "Degrees, north positive.";
const LONGITUDE: &str = "Degrees, east positive.";
const ALTITUDE: &str = "Metres above sea level; 0 when left out.";

/// A prompt: its name, what it is, the arguments it reads, and how it
/// writes its messages.
struct Prompt {
    name: &'static str,
    title: &'static str,
    description: &'static str,
    arguments: &'static [Argument],
    write: fn(&Given, &mut Ask<'_>) -> Result<Written, Refused>,
}

/// What a prompt writes: the tool, its record, and the sentences around
/// the call.
struct Written {
    tool: &'static str,
    request: Value,
    /// The argument a refusal under the server's limits names.
    limited: &'static str,
    asks: String,
    reads: &'static str,
}

const PROMPTS: [Prompt; 3] = [
    Prompt {
        name: "birth-chart",
        title: "A birth chart",
        description: "A birth chart founded at a civil date, time and place, with the \
                      navamsha, the Vimshottari dasha and the sections asked for.",
        arguments: &[
            required("date", DATE),
            required("time", TIME),
            required("zone", ZONE),
            required("latitude", LATITUDE),
            required("longitude", LONGITUDE),
            optional("altitude", ALTITUDE),
            optional(
                "sections",
                "Sections beside the foundation, comma-separated (shadbala, jaimini, …); a \
                 completion offers each.",
            )
            .from(sections, true),
        ],
        write: birth_chart,
    },
    Prompt {
        name: "day-panchanga",
        title: "A day's panchanga",
        description: "The panchanga of a civil day, or a run of days, at a place: each \
                      day's limbs, sunrise and sunset, and the festivals it carries.",
        arguments: &[
            required("date", DATE),
            required("zone", ZONE),
            required("latitude", LATITUDE),
            required("longitude", LONGITUDE),
            optional("altitude", ALTITUDE),
            optional(
                "last",
                "The last day of a run, YYYY-MM-DD; the one day when left out.",
            ),
            optional(
                "festivals",
                "The festival rules the days are reckoned by, a pack the SDK ships; \
                 DHARMASINDHU when left out.",
            )
            .from(festival_packs, false),
        ],
        write: day_panchanga,
    },
    Prompt {
        name: "match",
        title: "A match between two births",
        description: "The groom's chart founded with the bride's as its partner, and the \
                      match between them read as the matching record reads it.",
        arguments: &[
            required("groomDate", DATE),
            required("groomTime", TIME),
            required("groomZone", ZONE),
            required("groomLatitude", LATITUDE),
            required("groomLongitude", LONGITUDE),
            required("brideDate", DATE),
            required("brideTime", TIME),
            required("brideZone", ZONE),
            required("brideLatitude", LATITUDE),
            required("brideLongitude", LONGITUDE),
        ],
        write: matching,
    },
];

/// `prompts/list`'s prompts.
pub(crate) fn list() -> Map<String, Value> {
    let prompts: Vec<Value> = PROMPTS
        .iter()
        .map(|prompt| {
            let arguments: Vec<Value> = prompt
                .arguments
                .iter()
                .map(|argument| {
                    json!({
                        "name": argument.name,
                        "description": argument.description,
                        "required": argument.required,
                    })
                })
                .collect();
            json!({
                "name": prompt.name,
                "title": prompt.title,
                "description": prompt.description,
                "arguments": arguments,
            })
        })
        .collect();
    let mut list = Map::new();
    list.insert(String::from("prompts"), Value::Array(prompts));
    list
}

/// `prompts/get`'s answer: the prompt `params` names, written from its
/// arguments, each record asked through `ask`.
pub(crate) fn get(
    params: &Value,
    ask: &mut Ask<'_>,
    limits: &Limits,
) -> Result<Map<String, Value>, Refused> {
    let name = params.get("name").and_then(Value::as_str).ok_or(Refused {
        message: String::from("`prompts/get` names its prompt in `name`"),
        argument: None,
    })?;
    let prompt = PROMPTS
        .iter()
        .find(|prompt| prompt.name == name)
        .ok_or_else(|| Refused {
            message: format!("no prompt `{name}`: `prompts/list` names every one"),
            argument: None,
        })?;
    let mut given = Given::new();
    match params.get("arguments") {
        None | Some(Value::Null) => {}
        Some(Value::Object(arguments)) => {
            for (key, value) in arguments {
                if !prompt.arguments.iter().any(|argument| argument.name == key) {
                    return Err(Refused::of(
                        key.clone(),
                        format!("`{name}` reads no argument `{key}`"),
                    ));
                }
                let Value::String(text) = value else {
                    return Err(Refused::of(key.clone(), format!("`{key}` is text")));
                };
                given.insert(key.clone(), text.trim().to_owned());
            }
        }
        Some(_) => {
            return Err(Refused {
                message: String::from("`arguments` maps each argument to its text"),
                argument: None,
            });
        }
    }
    if let Some(missing) = prompt
        .arguments
        .iter()
        .find(|argument| argument.required && given.get(argument.name).is_none_or(String::is_empty))
    {
        return Err(Refused::of(
            missing.name,
            format!("`{name}` needs `{}`: {}", missing.name, missing.description),
        ));
    }
    let written = (prompt.write)(&given, ask)?;
    limits
        .check(&written.request)
        .map_err(|why| Refused::of(written.limited, why.to_string()))?;
    let call = json!({ "request": written.request });
    let text = format!(
        "{} Call the tool `{}` with these arguments; its own reader has read and checked \
         them:\n\n```json\n{}\n```\n\n{} Every answer carries its provenance: quote the \
         settings hash and the conventions it names beside any number taken from it.",
        written.asks,
        written.tool,
        serde_json::to_string_pretty(&call).unwrap_or_else(|_| call.to_string()),
        written.reads,
    );
    let schema = format!("teistro://tools/{}/schema", written.tool);
    let mut result = Map::new();
    result.insert(String::from("description"), json!(prompt.description));
    result.insert(
        String::from("messages"),
        json!([
            { "role": "user", "content": { "type": "text", "text": text } },
            { "role": "user", "content": {
                "type": "resource_link",
                "uri": schema,
                "name": "tool-schema",
                "description": format!("`{}`'s input and output schemas in full.", written.tool),
                "mimeType": resources::SCHEMA,
            } },
        ]),
    );
    Ok(result)
}

/// The text of argument `name`, which the required check has seen.
fn text<'a>(given: &'a Given, name: &str) -> &'a str {
    given.get(name).map_or("", String::as_str)
}

/// `YYYY-MM-DD` as the record's date.
fn date(given: &Given, name: &str) -> Result<Value, Refused> {
    let written = text(given, name);
    let refused = || Refused::of(name, format!("`{name}` is a date, YYYY-MM-DD: `{written}`"));
    let (sign, rest) = match written.strip_prefix('-') {
        Some(rest) => (-1, rest),
        None => (1, written),
    };
    let mut parts = rest.split('-');
    let mut number = || -> Result<i64, Refused> {
        parts
            .next()
            .and_then(|part| part.parse::<i64>().ok())
            .ok_or_else(refused)
    };
    let (year, month, day) = (number()? * sign, number()?, number()?);
    if parts.next().is_some() {
        return Err(refused());
    }
    Ok(json!({ "year": year, "month": month, "day": day }))
}

/// `HH:MM` or `HH:MM:SS` as the record's time of day.
fn time(given: &Given, name: &str) -> Result<Value, Refused> {
    let written = text(given, name);
    let refused = || {
        Refused::of(
            name,
            format!("`{name}` is a time, HH:MM or HH:MM:SS: `{written}`"),
        )
    };
    let parts: Vec<u64> = written
        .split(':')
        .map(|part| part.parse::<u64>().map_err(|_| refused()))
        .collect::<Result<_, _>>()?;
    match parts.as_slice() {
        [hour, minute] => Ok(json!({ "hour": hour, "minute": minute, "second": 0 })),
        [hour, minute, second] => Ok(json!({ "hour": hour, "minute": minute, "second": second })),
        _ => Err(refused()),
    }
}

/// A zone as the record spells it: an IANA name, a fixed offset, or local
/// mean time at `longitude`.
fn zone(given: &Given, name: &str, longitude: f64) -> Result<Value, Refused> {
    let written = text(given, name);
    if written.eq_ignore_ascii_case("LMT") {
        return Ok(json!({ "kind": "LOCAL_MEAN", "longitude": longitude }));
    }
    if written.eq_ignore_ascii_case("UTC") || written.eq_ignore_ascii_case("Z") {
        return Ok(json!({ "kind": "FIXED", "offset": 0 }));
    }
    let signed = match written.as_bytes().first() {
        Some(b'+') => Some(1),
        Some(b'-') => Some(-1),
        _ => None,
    };
    let Some(sign) = signed else {
        return Ok(json!({ "kind": "IANA", "zone": written }));
    };
    let refused = || Refused::of(name, format!("`{name}` is an offset, ±HH:MM: `{written}`"));
    let (hours, minutes) = written
        .get(1..)
        .and_then(|rest| rest.split_once(':'))
        .ok_or_else(refused)?;
    let hours: i64 = hours.parse().map_err(|_| refused())?;
    let minutes: i64 = minutes.parse().map_err(|_| refused())?;
    Ok(json!({ "kind": "FIXED", "offset": sign * (hours * 3600 + minutes * 60) }))
}

/// A number of degrees or metres.
fn number(given: &Given, name: &str) -> Result<f64, Refused> {
    match given.get(name).map(String::as_str) {
        None | Some("") => Ok(0.0),
        Some(written) => written
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
            .ok_or_else(|| Refused::of(name, format!("`{name}` is a number: `{written}`"))),
    }
}

/// The record's refusal named by the argument it came from: the first
/// whose spelling begins its field, else the prompt's own.
fn named(why: &Error, fields: &[(&str, &str)], otherwise: &str) -> Refused {
    let field = why.field().unwrap_or("");
    let argument = fields
        .iter()
        .find(|(prefix, _)| field.starts_with(prefix))
        .map_or(otherwise, |(_, argument)| argument);
    Refused::of(argument, why.to_string())
}

/// An instant and the offset in force there, from a civil date, time
/// and zone resolved through `time.resolve`; `prefix` names the
/// arguments (`groom` for `groomDate`).
fn resolved(
    given: &Given,
    prefix: &str,
    at: Option<Value>,
    longitude: f64,
    ask: &mut Ask<'_>,
) -> Result<(f64, i64), Refused> {
    let named_as = |part: &str| {
        if prefix.is_empty() {
            part.to_owned()
        } else {
            let mut letters = part.chars();
            let first: String = letters
                .next()
                .map(|c| c.to_ascii_uppercase())
                .into_iter()
                .collect();
            format!("{prefix}{first}{}", letters.as_str())
        }
    };
    let (date_name, time_name, zone_name) = (named_as("date"), named_as("time"), named_as("zone"));
    let clock = match at {
        Some(clock) => clock,
        None => time(given, &time_name)?,
    };
    let request = json!({
        "date": date(given, &date_name)?,
        "time": clock,
        "zone": zone(given, &zone_name, longitude)?,
    });
    let answer = ask("time.resolve", &request).map_err(|why| {
        named(
            &why,
            &[
                ("date", &date_name),
                ("year", &date_name),
                ("month", &date_name),
                ("day", &date_name),
                ("time", &time_name),
                ("hour", &time_name),
                ("minute", &time_name),
                ("second", &time_name),
                ("zone", &zone_name),
            ],
            &zone_name,
        )
    })?;
    let instant = answer.get("instant").and_then(Value::as_f64);
    let offset = answer
        .get("zone")
        .and_then(|zone| zone.get("offset"))
        .and_then(Value::as_i64);
    match (instant, offset) {
        (Some(instant), Some(offset)) => Ok((instant, offset)),
        _ => Err(Refused {
            message: format!("`time.resolve` answered no instant and offset: {answer}"),
            argument: None,
        }),
    }
}

/// The place fields every chart and day record carries.
fn place_fields(request: &mut Map<String, Value>, latitude: f64, longitude: f64, altitude: f64) {
    request.insert(String::from("latitudeDeg"), json!(latitude));
    request.insert(String::from("longitudeDeg"), json!(longitude));
    request.insert(String::from("altitudeM"), json!(altitude));
}

const PLACE_FIELDS: [(&str, &str); 3] = [
    ("latitudeDeg", "latitude"),
    ("longitudeDeg", "longitude"),
    ("altitudeM", "altitude"),
];

/// `request` read by the chart request's own reader.
fn found(request: Value, fields: &[(&str, &str)], otherwise: &str) -> Result<Value, Refused> {
    teistro::FoundRequest::from_json(&request.to_string())
        .map_err(|why| named(&why, fields, otherwise))?;
    Ok(request)
}

fn birth_chart(given: &Given, ask: &mut Ask<'_>) -> Result<Written, Refused> {
    let (latitude, longitude) = (number(given, "latitude")?, number(given, "longitude")?);
    let altitude = number(given, "altitude")?;
    let (instant, offset) = resolved(given, "", None, longitude, ask)?;
    let mut request = Map::new();
    request.insert(String::from("instant"), json!(instant));
    place_fields(&mut request, latitude, longitude, altitude);
    request.insert(String::from("utcOffsetSeconds"), json!(offset));
    request.insert(String::from("vargas"), json!(["D9"]));
    request.insert(String::from("dashas"), json!(["VIMSHOTTARI"]));
    let known = teistro::FoundRequest::sections();
    for section in text(given, "sections")
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        if !known.contains(&section) {
            return Err(Refused::of(
                "sections",
                format!(
                    "no section `{section}`; the sections are {}",
                    known.join(", ")
                ),
            ));
        }
        request.insert(section.to_owned(), json!(true));
    }
    let request = found(Value::Object(request), &PLACE_FIELDS, "sections")?;
    Ok(Written {
        tool: "chart.found",
        request,
        limited: "sections",
        asks: format!(
            "Found the birth chart for {} at {} ({}), at {latitude}° {longitude}°. The civil \
             time resolved through `time.resolve` to the UTC Julian day {instant}, the clock \
             {offset} seconds from UTC.",
            text(given, "date"),
            text(given, "time"),
            text(given, "zone"),
        ),
        reads: "The answer's `charts` holds the chart document: the lagna, each graha's sign, \
                nakshatra and house, the navamsha and the dasha periods.",
    })
}

fn day_panchanga(given: &Given, ask: &mut Ask<'_>) -> Result<Written, Refused> {
    let (latitude, longitude) = (number(given, "latitude")?, number(given, "longitude")?);
    let altitude = number(given, "altitude")?;
    let noon = json!({ "hour": 12, "minute": 0, "second": 0 });
    let (_, offset) = resolved(given, "", Some(noon), longitude, ask)?;
    let mut request = Map::new();
    request.insert(String::from("first"), date(given, "date")?);
    if given.get("last").is_some_and(|last| !last.is_empty()) {
        request.insert(String::from("last"), date(given, "last")?);
    }
    place_fields(&mut request, latitude, longitude, altitude);
    request.insert(String::from("utcOffsetSeconds"), json!(offset));
    let pack = given
        .get("festivals")
        .filter(|pack| !pack.is_empty())
        .map_or("DHARMASINDHU", String::as_str);
    request.insert(String::from("festivals"), json!({ "rules": pack }));
    let request = Value::Object(request);
    let mut fields = PLACE_FIELDS.to_vec();
    fields.extend([
        ("first", "date"),
        ("last", "last"),
        ("festivals", "festivals"),
    ]);
    teistro::DaysRequest::from_json(&request.to_string())
        .map_err(|why| named(&why, &fields, "date"))?;
    Ok(Written {
        tool: "almanac.days",
        request,
        limited: "last",
        asks: format!(
            "Read the panchanga from {}{} at {latitude}° {longitude}°, on a clock {offset} \
             seconds from UTC (the offset `time.resolve` gives {} at noon).",
            text(given, "date"),
            given
                .get("last")
                .filter(|last| !last.is_empty())
                .map_or_else(String::new, |last| format!(" to {last}")),
            text(given, "zone"),
        ),
        reads: "Each of the answer's `days` gives the tithi, nakshatra, yoga, karana and vara \
                with the instants each ends, and sunrise and sunset; `festivals` names the \
                festivals the days carry.",
    })
}

fn matching(given: &Given, ask: &mut Ask<'_>) -> Result<Written, Refused> {
    let mut births = Vec::new();
    for prefix in ["groom", "bride"] {
        let latitude = number(given, &format!("{prefix}Latitude"))?;
        let longitude = number(given, &format!("{prefix}Longitude"))?;
        let (instant, offset) = resolved(given, prefix, None, longitude, ask)?;
        births.push((prefix, latitude, longitude, instant, offset));
    }
    let [groom, bride] = births.as_slice() else {
        return Err(Refused {
            message: String::from("a match is two births"),
            argument: None,
        });
    };
    let place = Place::try_from_degrees(bride.1, bride.2, 0.0).map_err(|why| {
        let argument = if why.quantity == "latitude" {
            "brideLatitude"
        } else {
            "brideLongitude"
        };
        Refused::of(argument, why.to_string())
    })?;
    let partner = json!({
        "instant": bride.3,
        "place": serde_json::to_value(place).map_err(|why| Refused {
            message: format!("serde cannot write the place: {why}"),
            argument: None,
        })?,
        "utcOffsetSeconds": bride.4,
    });
    let mut request = Map::new();
    request.insert(String::from("instant"), json!(groom.3));
    place_fields(&mut request, groom.1, groom.2, 0.0);
    request.insert(String::from("utcOffsetSeconds"), json!(groom.4));
    request.insert(
        String::from("matching"),
        json!({ "partner": partner, "partnerRole": "BRIDE" }),
    );
    let request = found(
        Value::Object(request),
        &[
            ("latitudeDeg", "groomLatitude"),
            ("longitudeDeg", "groomLongitude"),
            ("matching", "brideDate"),
        ],
        "groomDate",
    )?;
    Ok(Written {
        tool: "chart.found",
        request,
        limited: "brideDate",
        asks: String::from(
            "Read the match between these two births: the groom's chart founded with the \
             bride's as its partner, each civil time resolved through `time.resolve`.",
        ),
        reads: "The answer's `matching` row holds each koota with its points and the total, \
                and the doshas the matching record reads.",
    })
}
