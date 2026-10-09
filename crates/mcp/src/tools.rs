//! The tool list: every record entry point of the façade, described by
//! its boundary function's documentation, and `settings.describe`.

use serde_json::{Map, Value, json};
use teistro::Error;
use teistro::records::Record;
use teistro_core::envelope::canonical_json;
use teistro_core::settings::{DEFAULT_PROFILE, Profile, SHIPPED_PROFILES, SettingsPatch};

include!(concat!(env!("OUT_DIR"), "/boundary_docs.rs"));

/// The tool answering the profiles and the settings patch's schema.
pub(crate) const DESCRIBE: &str = "settings.describe";

/// The arguments a record tool reads beside its record.
const ARGUMENTS: [&str; 4] = ["request", "profile", "settings", "locale"];

/// Every tool, in name order.
pub(crate) fn list() -> Map<String, Value> {
    let mut tools: Vec<(&str, Value)> = teistro::records::records()
        .iter()
        .map(|record| (record.name, record_tool(record)))
        .collect();
    tools.push((DESCRIBE, describe_tool()));
    tools.sort_by_key(|(name, _)| *name);
    let mut list = Map::new();
    list.insert(
        String::from("tools"),
        Value::Array(tools.into_iter().map(|(_, tool)| tool).collect()),
    );
    list
}

/// Every tool computes and changes nothing, so a host may call one
/// without asking and again with the same answer.
fn annotations(title: &str) -> Value {
    json!({
        "title": title,
        "readOnlyHint": true,
        "destructiveHint": false,
        "idempotentHint": true,
        "openWorldHint": false,
    })
}

fn record_tool(record: &Record) -> Value {
    json!({
        "name": record.name,
        "title": record.title,
        "description": description(record),
        "inputSchema": {
            "type": "object",
            "properties": {
                "request": {
                    "type": "object",
                    "description": format!(
                        "The record `{}` reads, as the tool's description gives it.",
                        record.boundary
                    ),
                },
                "profile": {
                    "type": "string",
                    "enum": SHIPPED_PROFILES,
                    "default": DEFAULT_PROFILE,
                    "description": "The shipped profile the settings resolve from.",
                },
                "settings": {
                    "type": "object",
                    "description": "A patch over the profile, every group and knob optional; \
                                    `settings.describe` answers its JSON Schema.",
                },
                "locale": {
                    "type": "string",
                    "description": "The locale a message renders in, as a BCP 47 tag.",
                },
            },
            "required": ["request"],
            "additionalProperties": false,
        },
        "annotations": annotations(record.title),
    })
}

/// A record's description: its title, then what its boundary function's
/// documentation says of the record it reads, from the paragraph naming
/// `request_json` on, the argument here being `request`.
///
/// The paragraphs before it describe the answer as the boundary hands it
/// over, a blob in some areas; the answer here is the envelope, which the
/// tool's result says.
pub(crate) fn description(record: &Record) -> String {
    let doc = boundary_doc(record.boundary).unwrap_or_default();
    let read = doc
        .find("`request_json` is")
        .and_then(|at| doc.get(at..))
        .unwrap_or(doc)
        .replace("`request_json`", "`request`");
    format!(
        "{}. Answers `{{value, provenance}}`.\n\n{}",
        record.title,
        read.trim()
    )
}

/// The documentation `idl/api.json` holds for the boundary function
/// `name`.
pub(crate) fn boundary_doc(name: &str) -> Option<&'static str> {
    BOUNDARY_DOCS
        .iter()
        .find(|(function, _)| *function == name)
        .map(|(_, doc)| *doc)
}

fn describe_tool() -> Value {
    let title = "The shipped profiles and every setting";
    json!({
        "name": DESCRIBE,
        "title": title,
        "description": "The shipped profiles, each with its version, its base and what it \
                        sets, the profile a call without one uses, and the JSON Schema of the \
                        settings patch every tool's `settings` takes, each knob documented.",
        "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false },
        "annotations": annotations(title),
    })
}

/// `settings.describe`'s answer.
pub(crate) fn describe(arguments: &Map<String, Value>) -> Result<Value, Error> {
    if let Some(key) = arguments.keys().next() {
        return Err(
            Error::invalid_arg(format!("`{DESCRIBE}` reads no argument"))
                .with_field(format!("arguments.{key}")),
        );
    }
    let profiles = SHIPPED_PROFILES
        .iter()
        .filter_map(|id| Profile::shipped(id))
        .map(|profile| {
            Ok(json!({
                "id": profile.id.as_str(),
                "version": profile.version,
                "base": profile.base.as_ref().map(teistro_core::settings::ProfileId::as_str),
                "patch": written(&profile.patch)?,
            }))
        })
        .collect::<Result<Vec<Value>, Error>>()?;
    let schema = schemars::generate::SchemaSettings::draft2020_12()
        .into_generator()
        .into_root_schema_for::<SettingsPatch>();
    Ok(json!({
        "defaultProfile": DEFAULT_PROFILE,
        "profiles": profiles,
        "settings": written(&schema)?,
    }))
}

fn written<T: serde::Serialize>(value: &T) -> Result<Value, Error> {
    serde_json::to_value(value)
        .map_err(|why| Error::internal(format!("serde cannot write the answer: {why}")))
}

/// A record tool's arguments, read and refused by name.
#[derive(Debug)]
pub(crate) struct Arguments {
    /// The record, as the text its reader takes.
    pub(crate) request: String,
    pub(crate) profile: Option<String>,
    /// The patch as canonical JSON, so two spellings of one patch share a
    /// context.
    pub(crate) settings: Option<String>,
    pub(crate) locale: Option<String>,
}

impl Arguments {
    pub(crate) fn read(arguments: &Map<String, Value>) -> Result<Arguments, Error> {
        if let Some(key) = arguments
            .keys()
            .find(|key| !ARGUMENTS.contains(&key.as_str()))
        {
            return Err(Error::invalid_arg(format!("no argument `{key}`"))
                .with_field(format!("arguments.{key}"))
                .with_hint(format!("a record tool reads {}", ARGUMENTS.join(", "))));
        }
        let request = match arguments.get("request") {
            Some(request @ Value::Object(_)) => request.to_string(),
            Some(_) => {
                return Err(Error::invalid_arg("the record is a JSON object")
                    .with_field("arguments.request"));
            }
            None => {
                return Err(Error::invalid_arg("the call carries no record")
                    .with_field("arguments.request")
                    .with_hint("the tool's description gives the record it reads"));
            }
        };
        Ok(Arguments {
            request,
            profile: text(arguments, "profile")?,
            settings: arguments.get("settings").map(canonical_json),
            locale: text(arguments, "locale")?,
        })
    }
}

fn text(arguments: &Map<String, Value>, key: &str) -> Result<Option<String>, Error> {
    match arguments.get(key) {
        None => Ok(None),
        Some(Value::String(text)) => Ok(Some(text.clone())),
        Some(_) => Err(Error::invalid_arg(format!("`{key}` is a string"))
            .with_field(format!("arguments.{key}"))),
    }
}
