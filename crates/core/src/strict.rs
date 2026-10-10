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
//!
//! **Text is parsed once, into a [`Value`], and every type is read from
//! that.** Reading a type from text and from a value are two
//! instantiations of every deserialiser it has, and in the wasm module the
//! text-reading copies were an eighth of the code. A value keeps the last
//! of a key given twice, so the parse refuses a duplicate by its path:
//!
//! ```
//! let twice = teistro_core::strict::parse(r#"{"inner": 1, "inner": 2}"#, "ring").unwrap_err();
//! assert_eq!(twice.field(), Some("ring.inner"));
//! ```

use std::cell::RefCell;
use std::fmt;

use serde::Serialize;
use serde::de::{DeserializeOwned, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value};

use crate::error::Error;

/// Reads a value from JSON text, refusing JSON that is not one, and any key
/// the value does not read, by its path under `root`.
///
/// # Errors
///
/// Text that is not JSON, JSON that is not the type, or a key the type does
/// not read.
pub fn read<T: Serialize + DeserializeOwned>(text: &str, root: &str) -> Result<T, Error> {
    read_value(&parse(text, root)?, root)
}

/// Parses JSON text into a value, refusing text that is not one JSON value
/// and a key given twice in one object, by its path under `root`.
///
/// # Errors
///
/// Text that is not JSON, text after the value, or a duplicate key.
pub fn parse(text: &str, root: &str) -> Result<Value, Error> {
    let twice = RefCell::new(None);
    let mut reader = serde_json::Deserializer::from_str(text);
    let parsed = Unique {
        path: root.to_owned(),
        twice: &twice,
    }
    .deserialize(&mut reader)
    .and_then(|value| reader.end().map(|()| value));
    match (parsed, twice.into_inner()) {
        (_, Some(path)) => Err(Error::invalid_arg(format!("`{path}` is given twice"))
            .with_field(path)
            .with_hint(String::from("give each key once"))),
        (Ok(value), None) => Ok(value),
        (Err(err), None) => Err(not_json(root, &err)),
    }
}

/// A JSON value read with its path, which records the first key it finds
/// given twice and stops there.
struct Unique<'a> {
    path: String,
    twice: &'a RefCell<Option<String>>,
}

impl<'de> DeserializeSeed<'de> for Unique<'_> {
    type Value = Value;

    fn deserialize<D: serde::Deserializer<'de>>(self, reader: D) -> Result<Value, D::Error> {
        reader.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for Unique<'_> {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a JSON value")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Value, E> {
        Ok(Value::from(value))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Value, E> {
        Ok(Value::from(value))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Value, E> {
        // JSON text has no non-finite number, so this is never `Null`.
        Ok(Value::from(value))
    }

    fn visit_str<E>(self, value: &str) -> Result<Value, E> {
        Ok(Value::String(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Value, E> {
        Ok(Value::String(value))
    }

    fn visit_unit<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut items: A) -> Result<Value, A::Error> {
        let mut read = Vec::new();
        while let Some(item) = items.next_element_seed(Unique {
            path: under(&self.path, &format!("[{}]", read.len())),
            twice: self.twice,
        })? {
            read.push(item);
        }
        Ok(Value::Array(read))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut entries: A) -> Result<Value, A::Error> {
        let mut read = Map::new();
        while let Some(key) = entries.next_key::<String>()? {
            let path = under(&self.path, &key);
            if read.contains_key(&key) {
                self.twice.replace(Some(path));
                return Err(serde::de::Error::custom("a key is given twice"));
            }
            let value = entries.next_value_seed(Unique {
                path,
                twice: self.twice,
            })?;
            read.insert(key, value);
        }
        Ok(Value::Object(read))
    }
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
    deserialize(&parse(text, root)?, root)
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

/// The refusal of a value that is not what `at` reads. At an empty root a
/// missing field is named by its own key, the one thing a caller writing
/// the record can add; anything else there names no field, since there is
/// none to name.
fn not_json(at: &str, err: &serde_json::Error) -> Error {
    if at.is_empty() {
        let message = err.to_string();
        return match missing(&message) {
            Some(key) => Error::invalid_arg(format!("`{key}` is missing"))
                .with_field(key.to_owned())
                .with_hint(String::from("the record needs this field")),
            None => Error::invalid_arg(format!("it is not one: {message}")),
        };
    }
    Error::invalid_arg(format!("`{at}` is not one: {err}")).with_field(at.to_owned())
}

/// The key serde's "missing field `key`" names, if that is the message.
fn missing(message: &str) -> Option<&str> {
    message.strip_prefix("missing field `")?.split('`').next()
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
        // At the empty root a missing field is named by its own key.
        assert_eq!(
            deserialize_str::<Ring>("{}", "").unwrap_err().field(),
            Some("inner")
        );
        // Text after the value is refused, as `serde_json::from_str` does.
        assert_eq!(
            deserialize_str::<Ring>(r#"{"inner": 1} {}"#, "ring")
                .unwrap_err()
                .field(),
            Some("ring")
        );
    }

    #[test]
    fn a_key_given_twice_is_refused_by_its_path_wherever_it_stands() {
        let twice = read::<Ring>(r#"{"inner": 1, "inner": 2}"#, "ring").unwrap_err();
        assert_eq!(twice.field(), Some("ring.inner"));
        assert!(twice.message.contains("given twice"), "{}", twice.message);
        let deep = deserialize_str::<Vec<Ring>>(
            r#"[{"inner": 1}, {"inner": 1, "outer": 2, "outer": 3}]"#,
            "rings",
        )
        .unwrap_err();
        assert_eq!(deep.field(), Some("rings[1].outer"));
        // From the value's own root, as the settings read it.
        let own = deserialize_str::<Ring>(r#"{"inner": 1, "inner": 1}"#, "").unwrap_err();
        assert_eq!(own.field(), Some("inner"));
        // The same key in two objects is two keys.
        assert!(read::<Vec<Ring>>(r#"[{"inner": 1}, {"inner": 2}]"#, "rings").is_ok());
    }

    #[test]
    fn a_parsed_value_keeps_every_number_to_the_bit() {
        let text = "[0.1, -0, 1e-320, 18446744073709551615, -9223372036854775808, 2.5e300]";
        assert_eq!(
            parse(text, "n").unwrap(),
            serde_json::from_str::<Value>(text).unwrap()
        );
        let read = parse("[0.1, -0.0]", "n").unwrap();
        let bits = |at: usize| read.get(at).and_then(Value::as_f64).map(f64::to_bits);
        assert_eq!(bits(0), Some(0.1_f64.to_bits()));
        assert_eq!(bits(1), Some((-0.0_f64).to_bits()));
    }

    #[test]
    fn inside_a_tagged_enum_the_name_stops_at_the_enum() {
        let err =
            read::<Vec<Shape>>(r#"[{"kind": "circle", "radius": "big"}]"#, "shapes").unwrap_err();
        assert_eq!(err.field(), Some("shapes[0]"));
    }
}
