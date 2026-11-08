//! The tool list: every record entry point of the façade, described by
//! its boundary function's documentation, `settings.describe` and
//! `schema.describe`.

use serde_json::{Map, Value, json};
use teistro::records::Record;
use teistro::{Error, Loaded};
use teistro_core::envelope::canonical_json;
use teistro_core::settings::{
    DEFAULT_PROFILE, PROFILE_ALIASES, Profile, SHIPPED_PROFILES, SettingsPatch,
};
use teistro_port_ephemeris::native::{NativeFunction, NativeParam, Role};

use crate::{AnswerDetail, Detail, schemas};

include!(concat!(env!("OUT_DIR"), "/boundary_docs.rs"));

/// The tool answering the profiles and the settings patch's schema.
pub(crate) const DESCRIBE: &str = "settings.describe";

/// What an engine operation's tool is called: this, then its name.
pub(crate) const ENGINE_PREFIX: &str = "engine.";

/// The arguments a record tool reads beside its record.
const ARGUMENTS: [&str; 5] = ["request", "profile", "settings", "locale", DETAIL];

/// The argument naming how much of its answer a record tool answers.
pub(crate) const DETAIL: &str = "detail";

/// The argument naming the context a comparing record's call is
/// compared with.
pub(crate) const OTHER: &str = "other";

/// The arguments naming the context a call computes under.
pub(crate) const SCOPE: [&str; 3] = ["profile", "settings", "locale"];

/// Every tool, in name order: the records, `settings.describe`,
/// `schema.describe`, and the engine's own `operations`, each record's
/// schemas stated in `detail`.
pub(crate) fn list(
    operations: &[NativeFunction],
    own: &[crate::Tool],
    detail: Detail,
) -> Map<String, Value> {
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
    tools.extend(
        own.iter()
            .map(|tool| (tool.name.clone(), tool.listed(detail))),
    );
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
    annotations_reading(title, true)
}

/// The same, for a tool that may change state where `read_only` is false:
/// then it is neither read-only nor idempotent, and still not destructive.
fn annotations_reading(title: &str, read_only: bool) -> Value {
    json!({
        "title": title,
        "readOnlyHint": read_only,
        "destructiveHint": false,
        "idempotentHint": read_only,
        "openWorldHint": false,
    })
}

fn record_tool(record: &Record, detail: Detail) -> Value {
    let input = schemas::input(record);
    let tool = json!({
        "name": record.name,
        "title": record.title,
        "description": description(record),
        "inputSchema": schemas::with_examples(
            match detail {
                Detail::Full => input,
                Detail::Lean => schemas::lean(input, record),
            },
            record,
        ),
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
            named_as_the_protocol_allows(&name)
                && name != DESCRIBE
                && records.iter().all(|record| record.name != name)
        })
        .collect()
}

/// Whether `name` is a tool name as the revision spells one: at most 128
/// letters, digits, `_`, `-` and `.`. An operation the engine names
/// otherwise is still reached through `engine.call`.
pub(crate) fn named_as_the_protocol_allows(name: &str) -> bool {
    (1..=128).contains(&name.len())
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
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
    let required: Vec<&str> = operation
        .supplied()
        .filter(|param| !param.optional)
        .map(|param| param.name.as_str())
        .collect();
    // An operation that may change the engine's state is not read-only,
    // and the engine is the one to say it does not.
    let annotations = annotations_reading(&title, operation.mutates_engine_state == Some(false));
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
            "required": required,
            "additionalProperties": false,
        },
        "annotations": annotations,
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
        // A bare direction says nothing of the type; the engine's spelling
        // of it does, for the plain C types. An engine type (`tm_body`, an
        // enum or a struct) is left untyped rather than guessed.
        Role::In => param.kind.as_deref().and_then(|kind| match kind {
            "double" | "float" => Some("number"),
            "int32_t" | "int64_t" | "uint32_t" | "uint64_t" | "size_t" | "int" => Some("integer"),
            "string" => Some("string"),
            array if array.ends_with("[]") => Some("array"),
            _ => None,
        }),
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
/// What the operator added to every context, for `settings.describe`.
pub(crate) struct Added<'a> {
    pub(crate) packs: Vec<&'a Loaded>,
    pub(crate) layouts: Vec<&'a str>,
    pub(crate) dashas: Vec<&'a str>,
}

pub(crate) fn describe(arguments: &Map<String, Value>, added: &Added<'_>) -> Result<Value, Error> {
    if let Some(key) = arguments.keys().next() {
        return Err(
            Error::invalid_arg(format!("`{DESCRIBE}` reads no argument"))
                .with_field(format!("arguments.{key}")),
        );
    }
    let profiles = SHIPPED_PROFILES
        .iter()
        .filter_map(|id| profile(id))
        .collect::<Result<Vec<Value>, Error>>()?;
    // What the operator loaded into every context, so a model can tell a
    // reading a pack supplies from the SDK's own.
    let packs: Vec<Value> = added
        .packs
        .iter()
        .map(|loaded| {
            json!({
                "locale": loaded.locale,
                "namespaces": loaded.namespaces,
                "entries": loaded.entries,
                "sha256": loaded.sha256,
            })
        })
        .collect();
    Ok(json!({
        "defaultProfile": DEFAULT_PROFILE,
        "profiles": profiles,
        "settings": settings_schema()?,
        "packs": packs,
        // The members a request may name beside the catalogue's.
        "registered": {
            "dashaSystems": added.dashas,
            "chartLayouts": added.layouts,
        },
    }))
}

/// The shipped profile `id`: its version, its base, its older names and
/// what it sets, or None where no shipped profile has the id.
pub(crate) fn profile(id: &str) -> Option<Result<Value, Error>> {
    let profile = Profile::shipped(id)?;
    let aliases: Vec<&str> = PROFILE_ALIASES
        .iter()
        .filter(|(_, named)| *named == profile.id.as_str())
        .map(|(alias, _)| *alias)
        .collect();
    Some(written(&profile.patch).map(|patch| {
        json!({
            "id": profile.id.as_str(),
            "aliases": aliases,
            "version": profile.version,
            "base": profile.base.as_ref().map(teistro_core::settings::ProfileId::as_str),
            "patch": patch,
        })
    }))
}

/// The JSON Schema of the settings patch every tool's `settings` takes.
pub(crate) fn settings_schema() -> Result<Value, Error> {
    written(&schemas::generator().into_root_schema_for::<SettingsPatch>())
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
    /// The detail the call asks for; the server's when it names none.
    pub(crate) detail: Option<AnswerDetail>,
}

impl Arguments {
    /// A tool's own arguments with the context they name split off: the
    /// scope as a record tool's would read, and the rest.
    pub(crate) fn scope(
        arguments: &Map<String, Value>,
    ) -> Result<(Arguments, Map<String, Value>), Error> {
        let scope = Arguments {
            request: String::new(),
            profile: text(arguments, "profile")?,
            settings: arguments.get("settings").map(canonical_json),
            locale: text(arguments, "locale")?,
            detail: None,
        };
        let rest = arguments
            .iter()
            .filter(|(key, _)| !SCOPE.contains(&key.as_str()))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        Ok((scope, rest))
    }

    /// A comparing record's arguments with `other` split off, read as the
    /// scope of the context compared with; none where it was not named,
    /// which compares the call's context with itself. A record that
    /// compares nothing keeps every argument, and refuses `other` when
    /// read.
    pub(crate) fn other(
        record: Record,
        arguments: &Map<String, Value>,
    ) -> Result<(Map<String, Value>, Option<Arguments>), Error> {
        if !record.compares() {
            return Ok((arguments.clone(), None));
        }
        let mut rest = arguments.clone();
        let Some(other) = rest.remove(OTHER) else {
            return Ok((rest, None));
        };
        let Value::Object(scope) = other else {
            return Err(Error::invalid_arg(
                "`other` is the context compared with, as `{profile, settings, locale}`",
            )
            .with_field(format!("arguments.{OTHER}")));
        };
        if let Some(key) = scope.keys().find(|key| !SCOPE.contains(&key.as_str())) {
            return Err(Error::invalid_arg(format!("`{OTHER}` has no `{key}`"))
                .with_field(format!("arguments.{OTHER}.{key}"))
                .with_hint(format!("it names {}", SCOPE.join(", "))));
        }
        let (scope, _) = Arguments::scope(&scope).map_err(of_other)?;
        Ok((rest, Some(scope)))
    }

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
        let detail = text(arguments, DETAIL)?
            .map(|name| {
                AnswerDetail::read(&name)
                    .map_err(|why| why.with_field(format!("arguments.{DETAIL}")))
            })
            .transpose()?;
        Ok(Arguments {
            request,
            profile: text(arguments, "profile")?,
            settings: arguments.get("settings").map(canonical_json),
            locale: text(arguments, "locale")?,
            detail,
        })
    }
}

/// A refusal of the context compared with, named under `other`: its
/// `arguments.profile` is `arguments.other.profile`.
pub(crate) fn of_other(why: Error) -> Error {
    let field = why
        .field()
        .unwrap_or_default()
        .trim_start_matches("arguments.")
        .to_owned();
    let root = format!("arguments.{OTHER}");
    if field.is_empty() {
        why.with_field(root)
    } else {
        why.with_field(format!("{root}.{field}"))
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
