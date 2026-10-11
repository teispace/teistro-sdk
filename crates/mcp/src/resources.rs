//! Resources: what an agent reads rather than computes
//! (`03-design/mcp-server.md` §7, P2). The catalogue a kind at a time,
//! each shipped profile, the settings patch's schema, the chart
//! document's schema, and a tool's schemas. None varies by caller, and
//! every listed one is written once per process.

use std::sync::{LazyLock, OnceLock};

use serde_json::{Map, Value, json};
use teistro::Error;
use teistro_core::settings::SHIPPED_PROFILES;

use crate::{schemas, tools};

include!(concat!(env!("OUT_DIR"), "/catalogue_kinds.rs"));

/// What every resource's URI starts with.
const SCHEME: &str = "teistro://";

const JSON: &str = "application/json";
pub(crate) const SCHEMA: &str = "application/schema+json";

/// One resource a list names, and its text.
struct Listed {
    path: String,
    name: String,
    title: String,
    description: String,
    mime: &'static str,
    text: String,
}

/// Every listed resource, in path order, written on first use.
static LISTED: LazyLock<Vec<Listed>> = LazyLock::new(listed);

fn listed() -> Vec<Listed> {
    let mut listed = vec![
        Listed {
            path: String::from("settings/schema"),
            name: String::from("settings-schema"),
            title: String::from("The settings patch's JSON Schema"),
            description: String::from(
                "Every group and knob a tool's `settings` takes, each documented; \
                 the same schema `settings.describe` answers.",
            ),
            mime: SCHEMA,
            text: tools::settings_schema().map_or_else(|why| refusal(&why), |s| s.to_string()),
        },
        Listed {
            path: String::from("document/schema"),
            name: String::from("document-schema"),
            title: String::from("The sealed chart document's JSON Schema"),
            description: String::from(
                "The shape of every chart `chart.found` answers, as its reader accepts it.",
            ),
            mime: SCHEMA,
            text: teistro::schema::DOCUMENT.to_owned(),
        },
        Listed {
            path: String::from("catalogue"),
            name: String::from("catalogue"),
            title: String::from("The catalogue's kinds"),
            description: String::from(
                "Each kind of entity the SDK keys (graha, rashi, nakshatra, …) with its \
                 number and documentation; `teistro://catalogue/{kind}` reads one whole.",
            ),
            mime: JSON,
            text: catalogue_index().to_string(),
        },
    ];
    listed.extend(SHIPPED_PROFILES.iter().map(|id| {
        Listed {
            path: format!("profiles/{id}"),
            name: format!("profile-{id}"),
            title: format!("The `{id}` profile"),
            description: String::from("A shipped profile: its version, its base and what it sets."),
            mime: JSON,
            text: tools::profile(id)
                .unwrap_or_else(|| Err(Error::internal(format!("no shipped profile `{id}`"))))
                .map_or_else(|why| refusal(&why), |profile| profile.to_string()),
        }
    }));
    listed.extend(
        CATALOGUE_KINDS
            .iter()
            .map(|(kind, _, doc, open, text)| Listed {
                path: format!("catalogue/{kind}"),
                name: format!("catalogue-{kind}"),
                title: format!("The catalogue's `{kind}` kind"),
                description: if *open {
                    format!("{doc} Its members are registered at run time.")
                } else {
                    (*doc).to_owned()
                },
                mime: JSON,
                text: (*text).to_owned(),
            }),
    );
    listed.sort_by(|a, b| a.path.cmp(&b.path));
    listed
}

/// What a resource says where writing it failed, which no shipped data
/// does: the refusal itself, so the failure is read rather than hidden.
fn refusal(why: &Error) -> String {
    json!({ "error": why.to_string() }).to_string()
}

fn catalogue_index() -> Value {
    let kinds: Vec<Value> = CATALOGUE_KINDS
        .iter()
        .map(|(kind, number, doc, open, _)| {
            json!({
                "kind": kind,
                "number": number,
                "doc": doc,
                "open": open,
                "uri": format!("{SCHEME}catalogue/{kind}"),
            })
        })
        .collect();
    json!({ "schema": CATALOGUE_SCHEMA, "kinds": kinds })
}

/// `resources/list`'s resources.
pub(crate) fn list() -> Map<String, Value> {
    let resources: Vec<Value> = LISTED
        .iter()
        .map(|resource| {
            json!({
                "uri": format!("{SCHEME}{}", resource.path),
                "name": resource.name,
                "title": resource.title,
                "description": resource.description,
                "mimeType": resource.mime,
                "size": resource.text.len(),
            })
        })
        .collect();
    let mut list = Map::new();
    list.insert(String::from("resources"), Value::Array(resources));
    list
}

/// A parameterised resource: its URI with one `{argument}`, and the
/// values that argument takes, which `completion/complete` offers.
pub(crate) struct Template {
    /// The URI after the scheme, as `catalogue/{kind}`.
    pub(crate) path: &'static str,
    /// The one argument inside it.
    pub(crate) argument: &'static str,
    name: &'static str,
    title: &'static str,
    description: &'static str,
    mime: &'static str,
    /// Every value the argument takes, in the order a person reads them.
    pub(crate) values: fn() -> Vec<&'static str>,
}

/// Every template, each argument's values read from the table it names.
pub(crate) const TEMPLATES: [Template; 3] = [
    Template {
        path: "catalogue/{kind}",
        argument: "kind",
        name: "catalogue-kind",
        title: "A catalogue kind",
        description: "One kind of the catalogue whole: its documentation, its attributes' \
                      types, and every member with its key, id, sources and mark.",
        mime: JSON,
        values: || CATALOGUE_KINDS.iter().map(|(kind, ..)| *kind).collect(),
    },
    Template {
        path: "profiles/{id}",
        argument: "id",
        name: "profile",
        title: "A shipped profile",
        description: "A shipped profile: its version, its base and what it sets.",
        mime: JSON,
        values: || SHIPPED_PROFILES.to_vec(),
    },
    Template {
        path: "tools/{tool}/schema",
        argument: "tool",
        name: "tool-schema",
        title: "A record tool's schemas",
        description: "A record tool's input and output schemas in full, as \
                      `schema.describe` answers them.",
        mime: SCHEMA,
        values: || {
            teistro::records::records()
                .iter()
                .map(|record| record.name)
                .collect()
        },
    },
];

/// The template whose URI is `uri`, as a list writes it.
pub(crate) fn template(uri: &str) -> Option<&'static Template> {
    let path = uri.strip_prefix(SCHEME)?;
    TEMPLATES.iter().find(|template| template.path == path)
}

/// `resources/templates/list`'s templates.
pub(crate) fn templates() -> Map<String, Value> {
    let templates: Vec<Value> = TEMPLATES
        .iter()
        .map(|template| {
            json!({
                "uriTemplate": format!("{SCHEME}{}", template.path),
                "name": template.name,
                "title": template.title,
                "description": template.description,
                "mimeType": template.mime,
            })
        })
        .collect();
    let mut list = Map::new();
    list.insert(String::from("resourceTemplates"), Value::Array(templates));
    list
}

/// Each record tool's schemas as text, by the tool's place in
/// [`teistro::records::records`], written on its first read.
static TOOL_SCHEMAS: LazyLock<Vec<OnceLock<Option<String>>>> = LazyLock::new(|| {
    teistro::records::records()
        .iter()
        .map(|_| OnceLock::new())
        .collect()
});

/// The record tool `tool`'s schemas, as `schema.describe` answers them.
fn tool_schemas(tool: &str) -> Option<&'static str> {
    let at = teistro::records::records()
        .iter()
        .position(|record| record.name == tool)?;
    TOOL_SCHEMAS
        .get(at)?
        .get_or_init(|| {
            let mut arguments = Map::new();
            arguments.insert(String::from("tool"), json!(tool));
            schemas::describe(&arguments)
                .ok()
                .map(|schemas| schemas.to_string())
        })
        .as_deref()
}

/// `resources/read`'s contents for `uri`, or None where no resource has
/// it.
pub(crate) fn read(uri: &str) -> Option<Map<String, Value>> {
    let path = uri.strip_prefix(SCHEME)?;
    let listed = LISTED
        .binary_search_by(|resource| resource.path.as_str().cmp(path))
        .ok()
        .and_then(|at| LISTED.get(at));
    let (mime, text) = if let Some(resource) = listed {
        (resource.mime, resource.text.as_str())
    } else {
        let tool = path.strip_prefix("tools/")?.strip_suffix("/schema")?;
        (SCHEMA, tool_schemas(tool)?)
    };
    let mut contents = Map::new();
    contents.insert(
        String::from("contents"),
        json!([{ "uri": uri, "mimeType": mime, "text": text }]),
    );
    Some(contents)
}
