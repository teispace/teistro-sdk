//! The tool list: every record entry point of the façade, described by
//! its boundary function's documentation, `settings.describe` and
//! `schema.describe`.

use serde_json::{Map, Value, json};
use teistro::Error;
use teistro::records::Record;
use teistro_core::envelope::canonical_json;
use teistro_core::settings::{DEFAULT_PROFILE, Profile, SHIPPED_PROFILES, SettingsPatch};
use teistro_port_ephemeris::native::{NativeFunction, NativeParam, Role};

use crate::{Detail, schemas};

include!(concat!(env!("OUT_DIR"), "/boundary_docs.rs"));

/// The tool answering the profiles and the settings patch's schema.
pub(crate) const DESCRIBE: &str = "settings.describe";

/// What an engine operation's tool is called: this, then its name.
pub(crate) const ENGINE_PREFIX: &str = "engine.";

/// The arguments a record tool reads beside its record.
const ARGUMENTS: [&str; 4] = ["request", "profile", "settings", "locale"];

/// Every tool, in name order: the records, `settings.describe`,
/// `schema.describe`, and the engine's own `operations`, each record's
/// schemas stated in `detail`.
pub(crate) fn list(operations: &[NativeFunction], detail: Detail) -> Map<String, Value> {
    let mut tools: Vec<(String, Value)> = teistro::records::records()
        .iter()
        .map(|record| (record.name.to_owned(), record_tool(record, detail)))
        .collect();
    tools.push((DESCRIBE.to_owned(), describe_tool()));
    tools.push((schemas::DESCRIBE.to_owned(), schemas::describe_tool()));
    tools.extend(operations.iter().map(|operation| {
        let tool = engine_tool(operation);
        (format!("{ENGINE_PREFIX}{}", operation.name), tool)
    }));
    tools.sort_by(|(a, _), (b, _)| a.cmp(b));
    let mut list = Map::new();
    list.insert(
        String::from("tools"),
        Value::Array(tools.into_iter().map(|(_, tool)| tool).collect()),
    );
    list
}

/// Every tool computes and changes nothing, so a host may call one
/// without asking and again with the same answer.
pub(crate) fn annotations(title: &str) -> Value {
    json!({
        "title": title,
        "readOnlyHint": true,
        "destructiveHint": false,
        "idempotentHint": true,
        "openWorldHint": false,
    })
}

fn record_tool(record: &Record, detail: Detail) -> Value {
    let input = schemas::input(record);
    let tool = json!({
        "name": record.name,
        "title": record.title,
        "description": description(record),
        "inputSchema": match detail {
            Detail::Full => input,
            Detail::Lean => schemas::lean(input, record),
        },
        "annotations": annotations(record.title),
    });
    match (detail, schemas::output(record)) {
        (Detail::Full, Some(output)) => schemas::with(tool, "outputSchema", output),
        _ => tool,
    }
}

/// The operations that can be tools of their own: each whose tool name
/// no SDK tool holds, so `engine.call` and `engine.manifest` stay the
/// SDK's whatever an engine calls its functions.
pub(crate) fn reachable(operations: Vec<NativeFunction>) -> Vec<NativeFunction> {
    let records = teistro::records::records();
    operations
        .into_iter()
        .filter(|operation| {
            let name = format!("{ENGINE_PREFIX}{}", operation.name);
            name != DESCRIBE && records.iter().all(|record| record.name != name)
        })
        .collect()
}

/// An engine operation as a tool: its parameters the engine's own, as
/// its manifest describes them, and its answer the engine's JSON sealed
/// with a provenance naming the engine.
fn engine_tool(operation: &NativeFunction) -> Value {
    let title = format!("{} (the engine's own)", operation.name);
    let properties: Map<String, Value> = operation
        .supplied()
        .map(|param| (param.name.clone(), param_schema(param)))
        .collect();
    let doc = operation
        .doc
        .as_deref()
        .unwrap_or("An operation the engine offers.");
    json!({
        "name": format!("{ENGINE_PREFIX}{}", operation.name),
        "title": title,
        "description": format!(
            "{doc} The engine's own operation, beyond what the SDK computes: the arguments \
             are its parameters by the names its manifest gives, and the answer is \
             `{{value, provenance}}`, the value the engine's JSON unread by the SDK. \
             `engine.manifest` describes every operation; `engine.call` calls one under a \
             profile or settings of the caller's."
        ),
        "inputSchema": {
            "type": "object",
            "properties": properties,
            "additionalProperties": false,
        },
        "annotations": annotations(&title),
    })
}

/// One parameter's schema: the JSON type its role implies where it
/// implies one, and the engine's spelling of its type and its meaning as
/// the description.
fn param_schema(param: &NativeParam) -> Value {
    let mut schema = Map::new();
    let kind = match param.role {
        Role::ArrayIn => Some("array"),
        Role::StringIn => Some("string"),
        Role::StructIn => Some("object"),
        _ => None,
    };
    if let Some(kind) = kind {
        schema.insert(String::from("type"), json!(kind));
    }
    let role = serde_json::to_value(&param.role)
        .ok()
        .and_then(|role| role.as_str().map(str::to_owned))
        .unwrap_or_default();
    let spelt = param
        .kind
        .as_deref()
        .map_or(String::new(), |kind| format!("`{kind}`, "));
    let doc = param.doc.as_deref().unwrap_or("");
    schema.insert(
        String::from("description"),
        json!(format!("{spelt}{role}. {doc}").trim().to_owned()),
    );
    Value::Object(schema)
}

/// A record's description: its title, then what its boundary function's
/// documentation says of the record it reads, from the paragraph naming
/// `request_json` on, the argument here being `request`; or the record's
/// own description, where the boundary function takes a C request.
///
/// The paragraphs before it describe the answer as the boundary hands it
/// over, a blob in some areas; the answer here is the envelope, which the
/// tool's result says.
pub(crate) fn description(record: &Record) -> String {
    let doc = boundary_doc(record.boundary).unwrap_or_default();
    let read = match record.reads {
        Some(reads) => reads.to_owned(),
        None => doc
            .find("`request_json` is")
            .and_then(|at| doc.get(at..))
            .unwrap_or(doc)
            .replace("`request_json`", "`request`"),
    };
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
    let schema = schemas::generator().into_root_schema_for::<SettingsPatch>();
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
