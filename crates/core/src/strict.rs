//! Reading JSON a consumer wrote, strictly: every key they named must be one
//! the reader took.
//!
//! `serde`'s `deny_unknown_fields` cannot say this for every shape. An
//! internally tagged enum hands its tag to the variant's struct, which then
//! refuses the tag, so a layout's `Shape` could not deny anything. Comparing
//! what was given against what was read back asks the same question of any
//! type that serialises every field it reads, which the SDK's records do.
//!
//! ```
//! use teistro_core::strict::read;
//!
//! #[derive(Debug, serde::Serialize, serde::Deserialize)]
//! struct Ring { inner: f64, outer: f64 }
//!
//! let ring: Ring = read(r#"{"inner": 0.2, "outer": 0.4}"#, "ring")?;
//! assert_eq!(ring.outer, 0.4);
//!
//! let typo = read::<Ring>(r#"{"inner": 0.2, "outer": 0.4, "outr": 0.5}"#, "ring").unwrap_err();
//! assert_eq!(typo.field(), Some("ring.outr"));
//!
//! // A value of the wrong kind is named where it stands, not by its record.
//! let wide = read::<Ring>(r#"{"inner": "wide", "outer": 0.4}"#, "ring").unwrap_err();
//! assert_eq!(wide.field(), Some("ring.inner"));
//! # Ok::<(), teistro_core::error::Error>(())
//! ```
//!
//! A value inside an internally tagged enum is buffered before it is read,
//! so a failure there is named down to the enum and no further: serde
//! reads the buffer, not the caller's keys.

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::error::Error;

/// Reads a value from JSON text, refusing JSON that is not one, and any key
/// the value does not read, by its path under `root`.
///
/// # Errors
///
/// Text that is not JSON, JSON that is not the type, or a key the type does
/// not read.
pub fn read<T: Serialize + DeserializeOwned>(text: &str, root: &str) -> Result<T, Error> {
    let given: Value = serde_json::from_str(text).map_err(|err| not_json(root, &err))?;
    read_value(&given, root)
}

/// As [`read`], from JSON already parsed.
///
/// # Errors
///
/// As [`read`].
pub fn read_value<T: Serialize + DeserializeOwned>(given: &Value, root: &str) -> Result<T, Error> {
    let value: T = deserialize(given, root)?;
    let read = serde_json::to_value(&value).map_err(|err| not_json(root, &err))?;
    match unread(given, &read, root) {
        None => Ok(value),
        Some(path) => Err(
            Error::invalid_arg(format!("`{path}` is not a field this reads"))
                .with_field(path)
                .with_hint(String::from(
                    "check the spelling against the record's fields",
                )),
        ),
    }
}

/// The path of the first key in `given` that `read` does not have, walking
/// objects and arrays together.
fn unread(given: &Value, read: &Value, path: &str) -> Option<String> {
    match (given, read) {
        (Value::Object(given), Value::Object(read)) => given.iter().find_map(|(key, inner)| {
            let at = format!("{path}.{key}");
            match read.get(key) {
                None => Some(at),
                Some(back) => unread(inner, back, &at),
            }
        }),
        (Value::Array(given), Value::Array(read)) => given
            .iter()
            .zip(read)
            .enumerate()
            .find_map(|(index, (inner, back))| unread(inner, back, &format!("{path}[{index}]"))),
        _ => None,
    }
}

/// Reads a value from JSON already parsed, naming a failure by the path to
/// the value that failed under `root` — without [`read`]'s key-for-key
/// check.
///
/// For a record whose serialised form is not what it reads, such as a
/// patch that leaves out what it does not change, and which refuses an
/// unknown key itself with `deny_unknown_fields`.
///
/// # Errors
///
/// JSON that is not the type, named where it failed.
pub fn deserialize<T: DeserializeOwned>(given: &Value, root: &str) -> Result<T, Error> {
    serde_path_to_error::deserialize(given).map_err(|err| {
        let at = under(root, &err.path().to_string());
        not_json(&at, err.inner())
    })
}

/// As [`deserialize`], from JSON text.
///
/// # Errors
///
/// Text that is not JSON, or JSON that is not the type, named where it
/// failed.
pub fn deserialize_str<T: DeserializeOwned>(text: &str, root: &str) -> Result<T, Error> {
    let mut reader = serde_json::Deserializer::from_str(text);
    let value = serde_path_to_error::deserialize(&mut reader).map_err(|err| {
        let at = under(root, &err.path().to_string());
        not_json(&at, err.inner())
    })?;
    reader.end().map_err(|err| not_json(root, &err))?;
    Ok(value)
}

/// A path the reader reported, under the caller's own root: `.` is the
/// root itself, and an index joins without a dot. An empty root names
/// from the value's own root, for a caller that adds its own prefix.
fn under(root: &str, path: &str) -> String {
    match path {
        "." | "" => root.to_owned(),
        _ if root.is_empty() || path.starts_with('[') => format!("{root}{path}"),
        _ => format!("{root}.{path}"),
    }
}

/// The refusal of a value that is not what `at` reads; at the root of an
/// empty root it names no field, since there is none to name.
fn not_json(at: &str, err: &serde_json::Error) -> Error {
    if at.is_empty() {
        return Error::invalid_arg(format!("it is not one: {err}"));
    }
    Error::invalid_arg(format!("`{at}` is not one: {err}")).with_field(at.to_owned())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    #[derive(Debug, PartialEq, Serialize, serde::Deserialize)]
    #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
    #[serde(tag = "kind", rename_all = "lowercase")]
    enum Shape {
        Circle { radius: f64 },
        Square { rings: Vec<Ring> },
    }

    #[derive(Debug, PartialEq, Serialize, serde::Deserialize)]
    #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
    struct Ring {
        inner: f64,
        #[serde(default)]
        outer: f64,
    }

    #[test]
    fn a_tagged_enum_reads_and_an_extra_key_is_refused_by_its_path() {
        let shape: Shape = read(r#"{"kind": "circle", "radius": 1}"#, "shape").unwrap();
        assert_eq!(shape, Shape::Circle { radius: 1.0 });
        let err = read::<Shape>(
            r#"{"kind": "square", "rings": [{"inner": 0}, {"inner": 1, "outr": 2}]}"#,
            "layouts[0].shape",
        )
        .unwrap_err();
        assert_eq!(err.field(), Some("layouts[0].shape.rings[1].outr"));
    }

    #[test]
    fn a_default_field_may_be_left_out() {
        let ring: Ring = read(r#"{"inner": 0.5}"#, "ring").unwrap();
        assert_eq!(
            ring,
            Ring {
                inner: 0.5,
                outer: 0.0
            }
        );
    }

    #[test]
    fn text_that_is_not_json_is_refused_at_the_root() {
        assert_eq!(read::<Ring>("[", "ring").unwrap_err().field(), Some("ring"));
        // A record where a list belongs is the root's own fault.
        assert_eq!(
            read::<Vec<Ring>>("{}", "rings").unwrap_err().field(),
            Some("rings")
        );
    }

    #[test]
    fn a_value_of_the_wrong_kind_is_refused_where_it_stands() {
        let wide = read::<Ring>(r#"{"inner": "wide"}"#, "ring").unwrap_err();
        assert_eq!(wide.field(), Some("ring.inner"));
        assert!(wide.message.contains("`ring.inner`"), "{}", wide.message);
        // Down through a list, joined without a dot at the index.
        let deep =
            read::<Vec<Ring>>(r#"[{"inner": 0}, {"inner": 0, "outer": []}]"#, "rings").unwrap_err();
        assert_eq!(deep.field(), Some("rings[1].outer"));
        // A missing field is its record's fault: the key is not there to name.
        assert_eq!(
            read::<Ring>("{}", "ring").unwrap_err().field(),
            Some("ring")
        );
    }

    #[test]
    fn the_lenient_readers_name_the_value_and_take_what_the_type_takes() {
        // No key-for-key check: an extra key is the type's own business.
        let ring: Ring = deserialize_str(r#"{"inner": 1, "extra": 2}"#, "ring").unwrap();
        assert_eq!(ring.inner.to_bits(), 1.0_f64.to_bits());
        let wide = deserialize_str::<Ring>(r#"{"inner": 1, "outer": "far"}"#, "ring").unwrap_err();
        assert_eq!(wide.field(), Some("ring.outer"));
        let parsed: Value = serde_json::from_str(r#"[{"inner": true}]"#).unwrap();
        let deep = deserialize::<Vec<Ring>>(&parsed, "rings").unwrap_err();
        assert_eq!(deep.field(), Some("rings[0].inner"));
        // An empty root names from the value's own root, and a failure at
        // that root names nothing.
        let own = deserialize_str::<Ring>(r#"{"inner": []}"#, "").unwrap_err();
        assert_eq!(own.field(), Some("inner"));
        assert_eq!(deserialize_str::<Ring>("[]", "").unwrap_err().field(), None);
        // Text after the value is refused, as `serde_json::from_str` does.
        assert_eq!(
            deserialize_str::<Ring>(r#"{"inner": 1} {}"#, "ring")
                .unwrap_err()
                .field(),
            Some("ring")
        );
    }

    #[test]
    fn inside_a_tagged_enum_the_name_stops_at_the_enum() {
        let err =
            read::<Vec<Shape>>(r#"[{"kind": "circle", "radius": "big"}]"#, "shapes").unwrap_err();
        assert_eq!(err.field(), Some("shapes[0]"));
    }
}
