//! A missing input asked for rather than refused (the 2026-07-28
//! revision's multi round-trip requests, `03-design/mcp-server.md` D7).
//!
//! A record tool refused because its request lacks a field — a birth's
//! instant, a place's latitude — is answered, for a client whose
//! capabilities take a form, as an `InputRequiredResult`: one form asking
//! for every top-level field the refusal and the record's schema say is
//! missing, each a value a form can carry. The client's retry repeats the
//! call with `inputResponses`, whose answers are laid into the request
//! before it is read again; a declined or cancelled form gets the refusal
//! it would have had, and is not asked again. Nothing is kept between the
//! two requests: the retry carries the call and its answers, so the
//! server needs no `requestState`, and an answer is no more trusted than
//! the same value written in the call.

use std::borrow::Cow;

use serde_json::{Map, Value, json};

use crate::{CAPABILITIES_META, SERVER_INFO_META, schemas, server_info};

/// The one input request a call is asked: the form of what it lacks.
pub(crate) const KEY: &str = "teistro.missing";

/// What the retry's `inputResponses` said of the form.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Answer {
    /// No form was answered: a first call.
    Unasked,
    /// The form was filled in, its values laid into the request.
    Given,
    /// The user declined or dismissed it, so the call is not asked again.
    TurnedDown,
}

/// Whether the client declares form elicitation: `elicitation` naming
/// `form`, or naming no mode at all, which means a form.
pub(crate) fn takes_forms(params: &Value) -> bool {
    let Some(Value::Object(elicitation)) = params
        .get("_meta")
        .and_then(|meta| meta.get(CAPABILITIES_META))
        .and_then(|declared| declared.get("elicitation"))
    else {
        return false;
    };
    elicitation.contains_key("form") || !elicitation.contains_key("url")
}

/// The call's params with a retry's answers laid into its request, and
/// what the retry said. Only a top-level key of an accepted form's content
/// is laid in, over whatever the call itself wrote there.
pub(crate) fn answered(params: &Value) -> (Cow<'_, Value>, Answer) {
    let Some(response) = params
        .get("inputResponses")
        .and_then(|responses| responses.get(KEY))
    else {
        return (Cow::Borrowed(params), Answer::Unasked);
    };
    if response.get("action").and_then(Value::as_str) != Some("accept") {
        return (Cow::Borrowed(params), Answer::TurnedDown);
    }
    let Some(Value::Object(content)) = response.get("content") else {
        return (Cow::Borrowed(params), Answer::Given);
    };
    let mut params = params.clone();
    if let Some(Value::Object(arguments)) = params.get_mut("arguments") {
        let request = arguments
            .entry("request")
            .or_insert_with(|| Value::Object(Map::new()));
        if let Value::Object(request) = request {
            for (key, value) in content {
                request.insert(key.clone(), value.clone());
            }
        }
    }
    (Cow::Owned(params), Answer::Given)
}

/// The `InputRequiredResult` asking for what `result`, a record tool's
/// refusal, says the call lacks; `None` when the refusal is of anything
/// else, or names a field no form can carry.
pub(crate) fn ask(params: &Value, result: &Value) -> Option<Value> {
    if result.get("isError") != Some(&Value::Bool(true)) {
        return None;
    }
    let refusal = result.get("structuredContent")?;
    if refusal.get("status").and_then(Value::as_str) != Some("INVALID_ARG") {
        return None;
    }
    let refused = refusal.get("field").and_then(Value::as_str)?;
    let name = params.get("name").and_then(Value::as_str)?;
    let record = teistro::records::record(name)?;
    let given = params
        .get("arguments")
        .and_then(|arguments| arguments.get("request"));
    let lacks = |field: &str| given.and_then(|request| request.get(field)).is_none();
    // The refusal names its field from the request's root; one under it
    // is part of a field the call wrote, which a form cannot correct.
    let refused = refused.strip_prefix("request.").unwrap_or(refused);
    if refused.contains(['.', '[']) || !lacks(refused) {
        return None;
    }
    let input = schemas::input(&record);
    let request = resolved(&input, input.get("properties")?.get("request")?)?;
    let properties = request.get("properties")?;
    let required: Vec<&str> = request
        .get("required")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    // A required field's refusal says only that it is missing; one the
    // SDK refuses for itself says what to give, which the form repeats.
    let hint = refusal
        .get("hint")
        .and_then(Value::as_str)
        .filter(|_| !required.contains(&refused));
    let mut asked = Map::new();
    asked.insert(
        refused.to_owned(),
        primitive(&input, properties.get(refused)?, refused, hint)?,
    );
    // The schema's own required fields the call also lacks, so that one
    // form asks for all of them; one a form cannot carry is left to its
    // own refusal.
    let also: Vec<&str> = required
        .iter()
        .copied()
        .filter(|field| *field != refused && lacks(field))
        .collect();
    for field in also {
        if let Some(schema) = properties
            .get(field)
            .and_then(|schema| primitive(&input, schema, field, None))
        {
            asked.insert(field.to_owned(), schema);
        }
    }
    let asking: Vec<&String> = asked.keys().collect();
    let message = format!(
        "`{name}` needs {}: {}",
        listed(&asking),
        refusal
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("the request is incomplete")
    );
    let form = json!({
        "method": "elicitation/create",
        "params": {
            "mode": "form",
            "message": message,
            "requestedSchema": { "type": "object", "properties": asked.clone(), "required": asking },
        },
    });
    Some(json!({
        "resultType": "input_required",
        "inputRequests": { KEY: form },
        "_meta": { SERVER_INFO_META: server_info() },
    }))
}

/// The fields as a sentence names them.
fn listed(fields: &[&String]) -> String {
    let quoted: Vec<String> = fields.iter().map(|field| format!("`{field}`")).collect();
    match quoted.split_last() {
        Some((last, [])) => last.clone(),
        Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
        None => String::new(),
    }
}

/// A schema with its `$ref` into the root's `$defs` followed, and a
/// nullable `anyOf` or type list read as the type it allows.
fn resolved<'a>(root: &'a Value, schema: &'a Value) -> Option<&'a Value> {
    let mut schema = schema;
    for _ in 0..8 {
        if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
            let name = reference.strip_prefix("#/$defs/")?;
            schema = root.get("$defs")?.get(name)?;
        } else if let Some(Value::Array(choices)) = schema.get("anyOf") {
            let mut present = choices
                .iter()
                .filter(|choice| choice.get("type").and_then(Value::as_str) != Some("null"));
            match (present.next(), present.next()) {
                (Some(only), None) => schema = only,
                // A key catalogue's bare or full spelling: the bare one,
                // the spelling a person writes, is the first.
                (Some(first), Some(_)) if first.get("enum").is_some() => schema = first,
                _ => return Some(schema),
            }
        } else {
            return Some(schema);
        }
    }
    None
}

/// The field's schema as a form's primitive: a string (its enum, length
/// and a format a form names), a number or integer (its bounds) or a
/// boolean, described from the schema or the refusal's hint. `None` for
/// anything a form cannot carry, an object or a list.
fn primitive(root: &Value, schema: &Value, field: &str, hint: Option<&str>) -> Option<Value> {
    let resolved = resolved(root, schema)?;
    let kind = match resolved.get("type") {
        Some(Value::String(kind)) => kind.as_str(),
        Some(Value::Array(kinds)) => {
            let mut present = kinds
                .iter()
                .filter_map(Value::as_str)
                .filter(|kind| *kind != "null");
            match (present.next(), present.next()) {
                (Some(only), None) => only,
                _ => return None,
            }
        }
        _ => return None,
    };
    let kept: &[&str] = match kind {
        "string" => &["enum", "minLength", "maxLength"],
        "number" | "integer" => &["minimum", "maximum"],
        "boolean" => &[],
        _ => return None,
    };
    let mut primitive = Map::new();
    primitive.insert(String::from("type"), json!(kind));
    primitive.insert(String::from("title"), json!(field));
    let description = [schema, resolved]
        .iter()
        .find_map(|schema| schema.get("description").and_then(Value::as_str))
        .or(hint);
    if let Some(description) = description {
        primitive.insert(String::from("description"), json!(description));
    }
    for key in kept {
        if let Some(value) = resolved.get(*key) {
            primitive.insert((*key).to_owned(), value.clone());
        }
    }
    if let Some(format) = resolved
        .get("format")
        .and_then(Value::as_str)
        .filter(|format| {
            kind == "string" && matches!(*format, "uri" | "email" | "date" | "date-time")
        })
    {
        primitive.insert(String::from("format"), json!(format));
    }
    Some(Value::Object(primitive))
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
#[allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a test fails by panicking and indexes the JSON it built"
)]
mod tests {
    use super::*;

    #[test]
    fn a_client_takes_forms_by_naming_them_or_no_mode() {
        let declared = |elicitation: Value| json!({ "_meta": { CAPABILITIES_META: { "elicitation": elicitation } } });
        assert!(takes_forms(&declared(json!({}))));
        assert!(takes_forms(&declared(json!({ "form": {} }))));
        assert!(takes_forms(&declared(json!({ "form": {}, "url": {} }))));
        assert!(!takes_forms(&declared(json!({ "url": {} }))));
        assert!(!takes_forms(&declared(json!(true))));
        assert!(!takes_forms(&json!({ "_meta": { CAPABILITIES_META: {} } })));
    }

    #[test]
    fn an_accepted_form_is_laid_into_the_request_and_a_declined_one_is_not() {
        let call = |response: Value| {
            json!({ "name": "chart.found", "arguments": { "request": { "latitudeDeg": 1.0 } },
                    "inputResponses": { KEY: response } })
        };
        let accepted = call(json!({ "action": "accept", "content": { "instant": 2.5 } }));
        let (params, answer) = answered(&accepted);
        assert_eq!(answer, Answer::Given);
        assert_eq!(
            params["arguments"]["request"],
            json!({ "latitudeDeg": 1.0, "instant": 2.5 })
        );
        let declined = call(json!({ "action": "decline" }));
        let (params, answer) = answered(&declined);
        assert_eq!(answer, Answer::TurnedDown);
        assert_eq!(
            params["arguments"]["request"],
            json!({ "latitudeDeg": 1.0 })
        );
        assert_eq!(
            answered(&call(json!({ "action": "cancel" }))).1,
            Answer::TurnedDown
        );
        assert_eq!(
            answered(&call(json!({ "action": "accept" }))).1,
            Answer::Given
        );
        let unasked = json!({ "name": "chart.found" });
        assert_eq!(answered(&unasked).1, Answer::Unasked);
        // A call without a request is given one.
        let bare = json!({ "name": "x", "arguments": {}, "inputResponses": { KEY:
            { "action": "accept", "content": { "a": 1 } } } });
        assert_eq!(answered(&bare).0["arguments"]["request"], json!({ "a": 1 }));
    }

    #[test]
    fn a_schema_is_read_through_its_references_and_nullable_choices() {
        let root = json!({ "$defs": {
            "Kind": { "anyOf": [ { "type": "string", "enum": ["A", "B"] },
                                 { "type": "string", "pattern": "x" } ] },
            "Loop": { "$ref": "#/$defs/Loop" },
        } });
        let kind = primitive(
            &root,
            &json!({ "$ref": "#/$defs/Kind" }),
            "kind",
            Some("a hint"),
        )
        .unwrap();
        assert_eq!(
            kind,
            json!({ "type": "string", "title": "kind", "description": "a hint", "enum": ["A", "B"] })
        );
        let nullable = json!({ "anyOf": [ { "$ref": "#/$defs/Kind" }, { "type": "null" } ] });
        assert!(primitive(&root, &nullable, "kind", None).is_some());
        let number = json!({ "type": ["number", "null"], "format": "double", "minimum": 0, "description": "d" });
        assert_eq!(
            primitive(&root, &number, "n", None).unwrap(),
            json!({ "type": "number", "title": "n", "description": "d", "minimum": 0 })
        );
        let date = json!({ "type": "string", "format": "date" });
        assert_eq!(
            primitive(&root, &date, "d", None).unwrap()["format"],
            "date"
        );
        let odd = json!({ "type": "string", "format": "regex" });
        assert!(
            primitive(&root, &odd, "o", None)
                .unwrap()
                .get("format")
                .is_none()
        );
        assert!(primitive(&root, &json!({ "type": "boolean" }), "b", None).is_some());
        for unformable in [
            json!({ "type": "object" }),
            json!({ "type": "array" }),
            json!({ "type": ["number", "string"] }),
            json!({ "anyOf": [ { "type": "number" }, { "type": "string" } ] }),
            json!({ "$ref": "#/$defs/Loop" }),
            json!({ "$ref": "#/elsewhere" }),
            json!({ "$ref": "#/$defs/Gone" }),
            json!({}),
        ] {
            assert!(
                primitive(&root, &unformable, "f", None).is_none(),
                "{unformable}"
            );
        }
    }

    #[test]
    fn fields_are_listed_as_a_sentence_names_them() {
        let (a, b, c) = (String::from("a"), String::from("b"), String::from("c"));
        assert_eq!(listed(&[]), "");
        assert_eq!(listed(&[&a]), "`a`");
        assert_eq!(listed(&[&a, &b, &c]), "`a`, `b` and `c`");
    }
}
