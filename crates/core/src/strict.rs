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
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

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
    let value = T::deserialize(given).map_err(|err| not_json(root, &err))?;
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

fn not_json(root: &str, err: &serde_json::Error) -> Error {
    Error::invalid_arg(format!("`{root}` is not one: {err}")).with_field(root.to_owned())
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
    fn json_that_is_not_the_type_is_refused_at_the_root() {
        assert_eq!(read::<Ring>("[", "ring").unwrap_err().field(), Some("ring"));
        assert_eq!(
            read::<Ring>(r#"{"inner": "wide"}"#, "ring")
                .unwrap_err()
                .field(),
            Some("ring")
        );
    }
}
