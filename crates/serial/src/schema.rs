//! The JSON Schema of a sealed chart document.
//!
//! [`DOCUMENT`] is the schema as `cargo xtask document-schema` last wrote
//! it, and `check-document-schema` fails the build when the types have
//! moved since. It costs nothing to reach: a constant, with no feature and
//! no dependency. Only *generating* it needs the `schema` feature, which
//! derives `schemars::JsonSchema` on every type the document carries.
//!
//! The schema is taken from serde's own reading of those types, so it
//! cannot disagree with the writer about an attribute; the few types that
//! serialise themselves by hand carry a schema written beside their
//! reader (`docs/03-design/document-schema.md`).
//!
//! What it describes is what the **reader** accepts, which is slightly
//! wider than what this writer produces: an `Option` serde reads back as
//! `None` when its key is absent is not required, and a key the document
//! does not know is not refused, because `Document` does not refuse one.
//!
//! ```
//! let schema: serde_json::Value = serde_json::from_str(teistro_serial::schema::DOCUMENT).unwrap();
//! assert_eq!(schema["$schema"], "https://json-schema.org/draft/2020-12/schema");
//! assert!(schema["$id"].as_str().unwrap().starts_with("urn:teistro:schema:sealed-document:"));
//! ```

/// The JSON Schema (draft 2020-12) of a [`Sealed`](crate::Sealed)
/// [`Document`](crate::Document), as generated for this release.
pub const DOCUMENT: &str = include_str!("../schema/document.schema.json");

/// The schema's identifier: a name rather than a URL, because nothing
/// serves the schema yet, and one per release, because the shape is a
/// property of a release.
pub const DOCUMENT_ID: &str = concat!(
    "urn:teistro:schema:sealed-document:",
    env!("CARGO_PKG_VERSION")
);

/// Generates the schema from the types, as the text [`DOCUMENT`] holds.
///
/// Every object's keys are written in code-point order and arrays in the
/// order the types give them (a struct's `required` list keeps its field
/// order), so the text is the same whichever map order the JSON layer was
/// built with — a workspace build and a crate built alone produce the
/// same bytes. Each description is the doc comment's first paragraph,
/// which is the summary an editor shows; the examples and the reasoning
/// after it belong to the Rust documentation.
///
/// # Errors
///
/// Two types that would share a name in `$defs`. The generator would
/// otherwise number the second (`Span2`), a name that says nothing and
/// changes when a type is added; each such type is given its own name
/// with `#[schemars(rename = ...)]` instead.
#[cfg(feature = "schema")]
pub fn generate() -> Result<String, String> {
    use crate::{Document, Sealed};

    let generator = schemars::generate::SchemaSettings::draft2020_12().into_generator();
    let mut schema = generator.into_root_schema_for::<Sealed<Document>>();
    schema.insert("$id".to_owned(), DOCUMENT_ID.into());
    schema.insert("title".to_owned(), "A sealed chart document".into());
    let value = schema.to_value();
    let numbered = numbered_names(&value);
    if !numbered.is_empty() {
        return Err(format!(
            "these types share a schema name and were numbered apart: {}; give each its own \
             with #[cfg_attr(feature = \"schema\", schemars(rename = \"...\"))]",
            numbered.join(", ")
        ));
    }
    let mut text = serde_json::to_string_pretty(&tidied(value)).map_err(|e| e.to_string())?;
    text.push('\n');
    Ok(text)
}

/// The `$defs` names the generator made unique by numbering them.
#[cfg(feature = "schema")]
fn numbered_names(schema: &serde_json::Value) -> Vec<String> {
    let Some(defs) = schema.get("$defs").and_then(serde_json::Value::as_object) else {
        return Vec::new();
    };
    defs.keys()
        .filter(|name| {
            let base = name.trim_end_matches(|c: char| c.is_ascii_digit());
            base.len() < name.len() && defs.contains_key(base)
        })
        .cloned()
        .collect()
}

/// A value with every object's keys in code-point order and every
/// description cut to its first paragraph.
#[cfg(feature = "schema")]
fn tidied(value: serde_json::Value) -> serde_json::Value {
    use serde_json::Value;

    match value {
        Value::Object(map) => {
            let mut entries: Vec<(String, Value)> = map.into_iter().collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            Value::Object(
                entries
                    .into_iter()
                    .map(|(key, inner)| match (key.as_str(), inner) {
                        ("description", Value::String(text)) => {
                            (key, Value::String(summary(&text)))
                        }
                        (_, inner) => (key, tidied(inner)),
                    })
                    .collect(),
            )
        }
        Value::Array(items) => Value::Array(items.into_iter().map(tidied).collect()),
        other => other,
    }
}

/// A doc comment's first paragraph on one line, with rustdoc's link
/// brackets taken off the code spans they wrap.
#[cfg(feature = "schema")]
fn summary(doc: &str) -> String {
    let paragraph = doc.split("\n\n").next().unwrap_or_default();
    paragraph
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace("[`", "`")
        .replace("`]", "`")
}
