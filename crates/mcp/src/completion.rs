//! `completion/complete` (`03-design/mcp-server.md` §7, P3): an argument
//! of a resource template or a prompt completed from the table the reader
//! takes it from, so a client offers keys the server accepts.

use serde_json::{Map, Value, json};

use crate::{prompts, resources};

/// The most values one answer carries, as the revision bounds it.
const MOST: usize = 100;

/// Why a completion request is refused, as the field it names.
pub(crate) struct Refused {
    pub(crate) message: String,
    pub(crate) field: &'static str,
}

impl Refused {
    fn new(field: &'static str, message: impl Into<String>) -> Refused {
        Refused {
            message: message.into(),
            field,
        }
    }
}

/// The values `params` asks for: its `ref`'s argument, completed from
/// what the argument takes.
pub(crate) fn complete(params: &Value) -> Result<Map<String, Value>, Refused> {
    let reference = params
        .get("ref")
        .and_then(Value::as_object)
        .ok_or_else(|| Refused::new("ref", "name what is completed in `ref`"))?;
    let argument = params
        .get("argument")
        .and_then(Value::as_object)
        .ok_or_else(|| Refused::new("argument", "name the argument in `argument`"))?;
    let name = argument
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| Refused::new("argument.name", "`argument.name` is a string"))?;
    let typed = match argument.get("value") {
        None | Some(Value::Null) => "",
        Some(Value::String(typed)) => typed.as_str(),
        Some(_) => {
            return Err(Refused::new(
                "argument.value",
                "`argument.value` is a string",
            ));
        }
    };
    let values = match reference.get("type").and_then(Value::as_str) {
        Some("ref/resource") => {
            let uri = reference
                .get("uri")
                .and_then(Value::as_str)
                .ok_or_else(|| Refused::new("ref.uri", "a resource reference names its `uri`"))?;
            let template = resources::template(uri).ok_or_else(|| {
                Refused::new(
                    "ref.uri",
                    format!("no resource template `{uri}`: `resources/templates/list` names each"),
                )
            })?;
            if name != template.argument {
                return Err(Refused::new(
                    "argument.name",
                    format!("`{uri}` takes `{}`, not `{name}`", template.argument),
                ));
            }
            (template.values)()
        }
        Some("ref/prompt") => {
            let prompt = reference
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| Refused::new("ref.name", "a prompt reference names its `name`"))?;
            match prompts::takes(prompt, name) {
                prompts::Takes::NoPrompt => {
                    return Err(Refused::new(
                        "ref.name",
                        format!("no prompt `{prompt}`: `prompts/list` names every one"),
                    ));
                }
                prompts::Takes::NoArgument => {
                    return Err(Refused::new(
                        "argument.name",
                        format!("`{prompt}` reads no argument `{name}`"),
                    ));
                }
                // A date, a time, a place: a person's own text, which no
                // table holds, so none is offered.
                prompts::Takes::Text => Vec::new(),
                prompts::Takes::Values {
                    values,
                    list: false,
                } => values,
                prompts::Takes::Values { values, list: true } => {
                    return Ok(completion(&listed(values, typed)));
                }
            }
        }
        Some(other) => {
            return Err(Refused::new(
                "ref.type",
                format!(
                    "no reference type `{other}`: the server completes `ref/resource` and \
                     `ref/prompt`"
                ),
            ));
        }
        None => return Err(Refused::new("ref.type", "a reference names its `type`")),
    };
    Ok(completion(&ranked(values, typed)))
}

/// A comma-separated list's completions: what was typed before its last
/// comma kept, the last item completed from the values not yet named.
fn listed(values: Vec<&'static str>, typed: &str) -> Vec<String> {
    let (head, last) = typed.rsplit_once(',').unwrap_or(("", typed));
    let named: Vec<&str> = head.split(',').map(str::trim).collect();
    let fresh: Vec<&'static str> = values
        .into_iter()
        .filter(|value| !named.contains(value))
        .collect();
    let head = head.trim();
    ranked(fresh, last.trim())
        .into_iter()
        .map(|value| {
            if head.is_empty() {
                value.to_owned()
            } else {
                format!("{head}, {value}")
            }
        })
        .collect()
}

/// The values matching `typed`, best first: those it begins, those it is
/// inside, then those holding its letters in order, each group in the
/// table's own order; case is ignored.
fn ranked(values: Vec<&'static str>, typed: &str) -> Vec<&'static str> {
    let typed = typed.to_lowercase();
    let mut tiers: [Vec<&'static str>; 3] = Default::default();
    for value in values {
        let lower = value.to_lowercase();
        let tier = if lower.starts_with(&typed) {
            0
        } else if lower.contains(&typed) {
            1
        } else if in_order(&typed, &lower) {
            2
        } else {
            continue;
        };
        if let Some(group) = tiers.get_mut(tier) {
            group.push(value);
        }
    }
    tiers.into_iter().flatten().collect()
}

/// Whether every character of `typed` appears in `value`, in order.
fn in_order(typed: &str, value: &str) -> bool {
    let mut rest = value.chars();
    typed.chars().all(|wanted| rest.any(|c| c == wanted))
}

fn completion<T: AsRef<str>>(matched: &[T]) -> Map<String, Value> {
    let shown: Vec<&str> = matched.iter().take(MOST).map(AsRef::as_ref).collect();
    let mut result = Map::new();
    result.insert(
        String::from("completion"),
        json!({
            "values": shown,
            "total": matched.len(),
            "hasMore": matched.len() > MOST,
        }),
    );
    result
}
