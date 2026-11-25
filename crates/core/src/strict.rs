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
//! // An optional field left empty is left out, not written `null`.
//! #[derive(Debug, serde::Serialize, serde::Deserialize)]
//! struct Band {
//!     #[serde(default, skip_serializing_if = "Option::is_none")]
//!     width: Option<f64>,
//! }
//! let empty = read::<Band>(r#"{"width": null}"#, "band").unwrap_err();
//! assert_eq!(empty.field(), Some("band.width"));
//! assert!(empty.to_string().contains("is null"), "{empty}");
//! assert!(read::<Band>("{}", "band").is_ok());
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
use std::fmt::Write as _;

use serde::de::{
    DeserializeOwned, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess,
    Unexpected, VariantAccess, Visitor,
};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};

use crate::error::Error;

/// Reads a value from JSON text, refusing JSON that is not one, and any key
/// the value does not read, by its path under `root`.
///
/// ```
/// use teistro_core::strict::read;
///
/// #[derive(Debug, serde::Serialize, serde::Deserialize)]
/// struct Ring { inner: f64, outer: f64 }
///
/// let refused = read::<Ring>("[0.2, 0.4]", "ring").unwrap_err();
/// // serde would read a struct from a list; a consumer's record is an object.
/// assert_eq!(refused.field(), Some("ring"));
/// assert!(read::<Ring>("{\"inner\": 0.2, \"outer\": 0.4} x", "ring").is_err());
/// ```
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
/// ```
/// use teistro_core::strict::parse;
///
/// let value = parse(r#"{"rings": [{"inner": 1}]}"#, "layout")?;
/// assert_eq!(value["rings"][0]["inner"], 1);
/// let twice = parse(r#"{"rings": [{"inner": 1, "inner": 2}]}"#, "layout").unwrap_err();
/// assert_eq!(twice.field(), Some("layout.rings[0].inner"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
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
/// ```
/// use teistro_core::strict::{parse, read_value};
///
/// #[derive(Debug, serde::Serialize, serde::Deserialize)]
/// struct Ring { inner: f64, outer: f64 }
///
/// let given = parse(r#"[{"inner": 0.1, "outer": 0.2}, {"inner": 0.3, "outr": 0.4}]"#, "rings")?;
/// let refused = read_value::<Vec<Ring>>(&given, "rings").unwrap_err();
/// // Named down to the element that failed.
/// assert!(refused.field().is_some_and(|field| field.starts_with("rings[1]")));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`read`].
pub fn read_value<T: Serialize + DeserializeOwned>(given: &Value, root: &str) -> Result<T, Error> {
    let value: T = deserialize(given, root)?;
    let read = serde_json::to_value(&value).map_err(|err| not_json(root, &err))?;
    if let Some(path) = listed_record(given, &read, root) {
        // serde reads a struct from a list of its fields in order, so `[]`
        // would be a record of every default.
        return Err(Error::invalid_arg(format!(
            "`{path}` is a record, written as an object `{{…}}`, not a list"
        ))
        .with_field(path)
        .with_hint(String::from("write it with its field names")));
    }
    match unread(given, &read, root) {
        None => Ok(value),
        // An optional field left empty is not written back, so a `null`
        // given for it lands here; name that, not a misspelling.
        Some((path, true)) => Err(Error::invalid_arg(format!(
            "`{path}` is null, which is not a value; an empty field is left out"
        ))
        .with_field(path)
        .with_hint(String::from(
            "leave the key out for its default, or check its spelling",
        ))),
        Some((path, false)) => Err(Error::invalid_arg(format!(
            "`{path}` is not a field this reads"
        ))
        .with_field(path)
        .with_hint(String::from(
            "check the spelling against the record's fields",
        ))),
    }
}

/// The path of a record's member: `path.key`, or the key alone at a
/// record read with no root, as a request's own fields are named
/// (`horizonProfile`, not `.horizonProfile`).
fn member(path: &str, key: &str) -> String {
    if path.is_empty() {
        key.to_owned()
    } else {
        format!("{path}.{key}")
    }
}

/// The path of the first record in `given` written as a list, where `read`
/// writes it as an object, walking objects and arrays together.
fn listed_record(given: &Value, read: &Value, path: &str) -> Option<String> {
    match (given, read) {
        (Value::Array(_), Value::Object(_)) => Some(path.to_owned()),
        (Value::Object(given), Value::Object(read)) => given
            .iter()
            .find_map(|(key, inner)| listed_record(inner, read.get(key)?, &member(path, key))),
        (Value::Array(given), Value::Array(read)) => {
            given
                .iter()
                .zip(read)
                .enumerate()
                .find_map(|(index, (inner, back))| {
                    listed_record(inner, back, &format!("{path}[{index}]"))
                })
        }
        _ => None,
    }
}

/// The path of the first key in `given` that `read` does not have, walking
/// objects and arrays together, and whether its given value is `null`.
fn unread(given: &Value, read: &Value, path: &str) -> Option<(String, bool)> {
    match (given, read) {
        (Value::Object(given), Value::Object(read)) => given.iter().find_map(|(key, inner)| {
            let at = member(path, key);
            match read.get(key) {
                None => Some((at, inner.is_null())),
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
/// ```
/// use teistro_core::strict::{deserialize, parse};
///
/// /// A patch: what it leaves out it does not change.
/// #[derive(Debug, serde::Deserialize)]
/// #[serde(deny_unknown_fields)]
/// struct Patch { outer: Option<f64> }
///
/// let patch: Patch = deserialize(&parse("{}", "patch")?, "patch")?;
/// assert_eq!(patch.outer, None);
/// let refused = deserialize::<Patch>(&parse(r#"{"outer": "wide"}"#, "patch")?, "patch").unwrap_err();
/// assert_eq!(refused.field(), Some("patch.outer"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// JSON that is not the type, named where it failed.
pub fn deserialize<T: DeserializeOwned>(given: &Value, root: &str) -> Result<T, Error> {
    let track = Track::default();
    T::deserialize(Reader {
        value: given,
        step: &Step::Root,
        track: &track,
    })
    .map_err(|err| {
        let path = track.0.take().unwrap_or_else(|| String::from("."));
        not_json(&under(root, &path), &err)
    })
}

/// As [`deserialize`], from JSON text.
///
/// ```
/// use teistro_core::strict::deserialize_str;
///
/// let values: Vec<u8> = deserialize_str("[1, 2, 3]", "values")?;
/// assert_eq!(values, [1, 2, 3]);
/// assert_eq!(deserialize_str::<Vec<u8>>("[1, 300]", "values").unwrap_err().field(), Some("values[1]"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// Text that is not JSON, or JSON that is not the type, named where it
/// failed.
pub fn deserialize_str<T: DeserializeOwned>(text: &str, root: &str) -> Result<T, Error> {
    deserialize(&parse(text, root)?, root)
}

// Reading a type from a value, naming a failure by the value's path.
//
// A reader that wraps another deserialiser to watch the path instantiates
// every type's deserialiser twice, once over the wrapper and once over
// what it wraps; in the wasm module that was a tenth of the code (SR-159).
// This reader is the only deserialiser a type is read through: it walks
// the arrays, objects and enum bodies itself, keeping the path as a chain
// of borrowed steps, and hands every other value to `serde_json`'s own
// reader of a `&Value`, so a scalar reads, and is refused, exactly as
// `serde_json` reads it. The path is written out only when a read fails.

/// Where a value stands: a step from its container, or the root.
enum Step<'a> {
    Root,
    Index(&'a Step<'a>, usize),
    Key(&'a Step<'a>, &'a str),
}

impl Step<'_> {
    /// The path as `serde_path_to_error` wrote it: `.` for the root, keys
    /// joined by dots, an index in brackets with no dot before it.
    fn path(&self) -> String {
        let mut steps = Vec::new();
        let mut at = self;
        loop {
            match at {
                Step::Root => break,
                Step::Index(parent, _) | Step::Key(parent, _) => {
                    steps.push(at);
                    at = parent;
                }
            }
        }
        if steps.is_empty() {
            return String::from(".");
        }
        let mut path = String::new();
        for step in steps.into_iter().rev() {
            match step {
                Step::Index(_, index) => {
                    let _ = write!(path, "[{index}]");
                }
                Step::Key(_, key) => {
                    if !path.is_empty() {
                        path.push('.');
                    }
                    path.push_str(key);
                }
                Step::Root => {}
            }
        }
        path
    }
}

/// The path of the innermost value a read failed at: the first to fail
/// is the deepest, since a failure returns through every container.
#[derive(Default)]
struct Track(RefCell<Option<String>>);

impl Track {
    /// Records `step` as where the read failed, unless a deeper value
    /// already has, and hands the failure on.
    fn failed(&self, step: &Step<'_>, err: serde_json::Error) -> serde_json::Error {
        self.0.borrow_mut().get_or_insert_with(|| step.path());
        err
    }
}

/// A value read with its path.
struct Reader<'de, 'a> {
    value: &'de Value,
    step: &'a Step<'a>,
    track: &'a Track,
}

impl<'de> Reader<'de, '_> {
    /// An array, element by element, refused as `serde_json` refuses one
    /// with elements the type left unread.
    fn items<V: Visitor<'de>>(
        self,
        items: &'de [Value],
        visitor: V,
    ) -> Result<V::Value, serde_json::Error> {
        let mut access = Items {
            iter: items.iter(),
            index: 0,
            step: self.step,
            track: self.track,
        };
        let read = visitor.visit_seq(&mut access)?;
        if access.iter.len() == 0 {
            Ok(read)
        } else {
            Err(serde::de::Error::invalid_length(
                items.len(),
                &"fewer elements in array",
            ))
        }
    }

    /// An object, entry by entry, refused as `serde_json` refuses one
    /// with entries the type left unread.
    fn entries<V: Visitor<'de>>(
        self,
        entries: &'de Map<String, Value>,
        visitor: V,
    ) -> Result<V::Value, serde_json::Error> {
        let mut access = Entries {
            iter: entries.iter(),
            value: None,
            step: self.step,
            track: self.track,
        };
        let read = visitor.visit_map(&mut access)?;
        if access.iter.len() == 0 {
            Ok(read)
        } else {
            Err(serde::de::Error::invalid_length(
                entries.len(),
                &"fewer elements in map",
            ))
        }
    }
}

/// Hands a method to `serde_json`'s reader of the value itself.
macro_rules! as_serde_json {
    ($($method:ident)*) => {
        $(
            fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, serde_json::Error> {
                self.value.$method(visitor)
            }
        )*
    };
}

impl<'de> Deserializer<'de> for Reader<'de, '_> {
    type Error = serde_json::Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, serde_json::Error> {
        match self.value {
            Value::Array(items) => self.items(items, visitor),
            Value::Object(entries) => self.entries(entries, visitor),
            scalar => scalar.deserialize_any(visitor),
        }
    }

    as_serde_json! {
        deserialize_bool deserialize_i8 deserialize_i16 deserialize_i32 deserialize_i64
        deserialize_i128 deserialize_u8 deserialize_u16 deserialize_u32 deserialize_u64
        deserialize_u128 deserialize_f32 deserialize_f64 deserialize_char deserialize_str
        deserialize_string deserialize_bytes deserialize_byte_buf deserialize_unit
        deserialize_identifier deserialize_ignored_any
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, serde_json::Error> {
        self.value.deserialize_unit_struct(name, visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, serde_json::Error> {
        match self.value {
            Value::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, serde_json::Error> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, serde_json::Error> {
        match self.value {
            Value::Array(items) => self.items(items, visitor),
            other => other.deserialize_seq(visitor),
        }
    }

    fn deserialize_tuple<V: Visitor<'de>>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, serde_json::Error> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, serde_json::Error> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, serde_json::Error> {
        match self.value {
            Value::Object(entries) => self.entries(entries, visitor),
            other => other.deserialize_map(visitor),
        }
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, serde_json::Error> {
        match self.value {
            Value::Array(items) => self.items(items, visitor),
            Value::Object(entries) => self.entries(entries, visitor),
            other => other.deserialize_struct(name, fields, visitor),
        }
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, serde_json::Error> {
        let Value::Object(entries) = self.value else {
            // A unit variant by name, or a value no enum is.
            return self.value.deserialize_enum(name, variants, visitor);
        };
        let mut iter = entries.iter();
        match (iter.next(), iter.next()) {
            (Some((variant, value)), None) => visitor.visit_enum(Variant {
                name: variant,
                value,
                step: self.step,
                track: self.track,
            }),
            _ => Err(serde::de::Error::invalid_value(
                Unexpected::Map,
                &"map with a single key",
            )),
        }
    }
}

/// An array's elements, each read with its index.
struct Items<'de, 'a> {
    iter: std::slice::Iter<'de, Value>,
    index: usize,
    step: &'a Step<'a>,
    track: &'a Track,
}

impl<'de> SeqAccess<'de> for Items<'de, '_> {
    type Error = serde_json::Error;

    fn next_element_seed<S: DeserializeSeed<'de>>(
        &mut self,
        seed: S,
    ) -> Result<Option<S::Value>, serde_json::Error> {
        let Some(value) = self.iter.next() else {
            return Ok(None);
        };
        let step = Step::Index(self.step, self.index);
        self.index += 1;
        seed.deserialize(Reader {
            value,
            step: &step,
            track: self.track,
        })
        .map(Some)
        .map_err(|err| self.track.failed(&step, err))
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.len())
    }
}

/// An object's entries, each value read with its key.
struct Entries<'de, 'a> {
    iter: serde_json::map::Iter<'de>,
    value: Option<(&'de str, &'de Value)>,
    step: &'a Step<'a>,
    track: &'a Track,
}

impl<'de> MapAccess<'de> for Entries<'de, '_> {
    type Error = serde_json::Error;

    fn next_key_seed<S: DeserializeSeed<'de>>(
        &mut self,
        seed: S,
    ) -> Result<Option<S::Value>, serde_json::Error> {
        let Some((key, value)) = self.iter.next() else {
            return Ok(None);
        };
        self.value = Some((key, value));
        // A key refused (a field the record does not have) is named by
        // itself, as `serde_path_to_error` named it.
        seed.deserialize(Key(key))
            .map(Some)
            .map_err(|err| self.track.failed(&Step::Key(self.step, key), err))
    }

    fn next_value_seed<S: DeserializeSeed<'de>>(
        &mut self,
        seed: S,
    ) -> Result<S::Value, serde_json::Error> {
        let Some((key, value)) = self.value.take() else {
            return Err(serde::de::Error::custom("value is missing"));
        };
        let step = Step::Key(self.step, key);
        seed.deserialize(Reader {
            value,
            step: &step,
            track: self.track,
        })
        .map_err(|err| self.track.failed(&step, err))
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.len())
    }
}

/// An enum written as an object of one key, its variant, read with the
/// variant as the step to its body.
struct Variant<'de, 'a> {
    name: &'de str,
    value: &'de Value,
    step: &'a Step<'a>,
    track: &'a Track,
}

impl<'de, 'a> EnumAccess<'de> for Variant<'de, 'a> {
    type Error = serde_json::Error;
    type Variant = Body<'de, 'a>;

    fn variant_seed<S: DeserializeSeed<'de>>(
        self,
        seed: S,
    ) -> Result<(S::Value, Body<'de, 'a>), serde_json::Error> {
        let named = seed.deserialize(serde::de::value::BorrowedStrDeserializer::<
            serde_json::Error,
        >::new(self.name))?;
        Ok((
            named,
            Body {
                variant: self.name,
                value: self.value,
                step: self.step,
                track: self.track,
            },
        ))
    }
}

/// A variant's body.
struct Body<'de, 'a> {
    variant: &'de str,
    value: &'de Value,
    step: &'a Step<'a>,
    track: &'a Track,
}

impl<'de> Body<'de, '_> {
    /// Reads the body under the variant's step, recording a failure there.
    fn read<R>(
        self,
        read: impl FnOnce(Reader<'de, '_>) -> Result<R, serde_json::Error>,
    ) -> Result<R, serde_json::Error> {
        let step = Step::Key(self.step, self.variant);
        read(Reader {
            value: self.value,
            step: &step,
            track: self.track,
        })
        .map_err(|err| self.track.failed(&step, err))
    }
}

impl<'de> VariantAccess<'de> for Body<'de, '_> {
    type Error = serde_json::Error;

    fn unit_variant(self) -> Result<(), serde_json::Error> {
        <()>::deserialize(self.value)
    }

    fn newtype_variant_seed<S: DeserializeSeed<'de>>(
        self,
        seed: S,
    ) -> Result<S::Value, serde_json::Error> {
        self.read(|body| seed.deserialize(body))
    }

    fn tuple_variant<V: Visitor<'de>>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, serde_json::Error> {
        self.read(|body| match body.value {
            Value::Array(items) if items.is_empty() => visitor.visit_unit(),
            Value::Array(items) => body.items(items, visitor),
            other => Err(serde::de::Error::invalid_type(
                unexpected(other),
                &"tuple variant",
            )),
        })
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, serde_json::Error> {
        self.read(|body| match body.value {
            Value::Object(entries) => body.entries(entries, visitor),
            other => Err(serde::de::Error::invalid_type(
                unexpected(other),
                &"struct variant",
            )),
        })
    }
}

/// What a value is, as a refusal names it.
fn unexpected(value: &Value) -> Unexpected<'_> {
    match value {
        Value::Null => Unexpected::Unit,
        Value::Bool(flag) => Unexpected::Bool(*flag),
        Value::Number(number) => number
            .as_u64()
            .map(Unexpected::Unsigned)
            .or_else(|| number.as_i64().map(Unexpected::Signed))
            .unwrap_or_else(|| Unexpected::Float(number.as_f64().unwrap_or(f64::NAN))),
        Value::String(text) => Unexpected::Str(text),
        Value::Array(_) => Unexpected::Seq,
        Value::Object(_) => Unexpected::Map,
    }
}

/// An object's key, read as `serde_json` reads one: a string, or a number
/// or a flag written as one.
struct Key<'de>(&'de str);

/// Reads a key as a number of the type asked, as `serde_json` does.
macro_rules! numeric_key {
    ($($method:ident $visit:ident $number:ty),*) => {
        $(
            fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, serde_json::Error> {
                match self.0.parse::<$number>() {
                    Ok(number) => visitor.$visit(number),
                    Err(_) => Err(serde::de::Error::invalid_type(Unexpected::Str(self.0), &visitor)),
                }
            }
        )*
    };
}

impl<'de> Deserializer<'de> for Key<'de> {
    type Error = serde_json::Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, serde_json::Error> {
        visitor.visit_borrowed_str(self.0)
    }

    numeric_key! {
        deserialize_i8 visit_i8 i8, deserialize_i16 visit_i16 i16,
        deserialize_i32 visit_i32 i32, deserialize_i64 visit_i64 i64,
        deserialize_i128 visit_i128 i128, deserialize_u8 visit_u8 u8,
        deserialize_u16 visit_u16 u16, deserialize_u32 visit_u32 u32,
        deserialize_u64 visit_u64 u64, deserialize_u128 visit_u128 u128,
        deserialize_f32 visit_f32 f32, deserialize_f64 visit_f64 f64
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, serde_json::Error> {
        match self.0 {
            "true" => visitor.visit_bool(true),
            "false" => visitor.visit_bool(false),
            other => Err(serde::de::Error::invalid_type(
                Unexpected::Str(other),
                &visitor,
            )),
        }
    }

    fn deserialize_option<V: Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, serde_json::Error> {
        // A key is never null.
        visitor.visit_some(self)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, serde_json::Error> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, serde_json::Error> {
        self.0
            .into_deserializer()
            .deserialize_enum(name, variants, visitor)
    }

    serde::forward_to_deserialize_any! {
        char str string bytes byte_buf unit unit_struct seq tuple tuple_struct
        map struct identifier ignored_any
    }
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
#[cfg_attr(coverage_nightly, coverage(off))]
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
    fn a_field_of_a_record_with_no_root_is_named_without_a_dot() {
        let unknown = read::<Ring>(r#"{"inner": 0, "outr": 2}"#, "").unwrap_err();
        assert_eq!(unknown.field(), Some("outr"));
        // A `null` the read-back does not hold is named by the walk too. (A
        // `null` given for `outer`, an `f64`, fails serde itself inside the
        // tagged enum, which buffers it and so names no path: `deserialize`'s
        // refusal, not this walk's.)
        let null = read::<Shape>(
            r#"{"kind": "square", "rings": [{"inner": 0, "outr": null}]}"#,
            "",
        )
        .unwrap_err();
        assert_eq!(null.field(), Some("rings[0].outr"));
        assert!(null.to_string().contains("is null"), "{null}");
        let listed = read::<Shape>(r#"{"kind": "square", "rings": [[1]]}"#, "").unwrap_err();
        assert_eq!(listed.field(), Some("rings[0]"));
    }

    #[test]
    fn a_record_written_as_a_list_is_refused_where_it_stands() {
        // serde would read `[0.5]` as `{"inner": 0.5}` and `[]` as nothing.
        let err = read::<Ring>("[0.5]", "ring").unwrap_err();
        assert_eq!(err.field(), Some("ring"));
        let err = read::<Shape>(
            r#"{"kind": "square", "rings": [{"inner": 0}, [1]]}"#,
            "shape",
        )
        .unwrap_err();
        assert_eq!(err.field(), Some("shape.rings[1]"));
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

    /// An enum written as an object of one key, as serde writes one by
    /// default, with a body of each kind.
    #[derive(Debug, PartialEq, Serialize, serde::Deserialize)]
    enum Mark {
        Dot,
        At(u8),
        Pair(u8, u8),
        Box { side: u8 },
    }

    /// The paths a refusal names, each as the wrapping reader named it
    /// before SR-159: an enum's body under its variant, an element by its
    /// index, a record by its own path when a field is missing.
    #[test]
    fn every_failure_is_named_where_it_stands() {
        let field = |text: &str| {
            deserialize_str::<Vec<Mark>>(text, "marks")
                .unwrap_err()
                .field()
                .map(String::from)
        };
        assert_eq!(field(r#"[{"At": 300}]"#).as_deref(), Some("marks[0].At"));
        assert_eq!(
            field(r#"[{"Pair": [1, "x"]}]"#).as_deref(),
            Some("marks[0].Pair[1]")
        );
        assert_eq!(
            field(r#"[{"Box": {"side": -1}}]"#).as_deref(),
            Some("marks[0].Box.side")
        );
        assert_eq!(field(r#"[{"Box": {}}]"#).as_deref(), Some("marks[0].Box"));
        assert_eq!(field(r#"["Dot", "Line"]"#).as_deref(), Some("marks[1]"));
        assert_eq!(
            field(r#"[{"At": 1, "Dot": null}]"#).as_deref(),
            Some("marks[0]")
        );
        assert_eq!(field(r#"[{"Pair": 1}]"#).as_deref(), Some("marks[0].Pair"));
        assert_eq!(field(r#"{"Dot": null}"#).as_deref(), Some("marks"));
        let marks: Vec<Mark> = deserialize_str(
            r#"["Dot", {"Dot": null}, {"At": 2}, {"Pair": [1, 2]}, {"Box": {"side": 3}}]"#,
            "marks",
        )
        .unwrap();
        assert_eq!(
            marks,
            [
                Mark::Dot,
                Mark::Dot,
                Mark::At(2),
                Mark::Pair(1, 2),
                Mark::Box { side: 3 }
            ]
        );
    }

    /// A key read as a number or a flag, as `serde_json` reads one; an
    /// array longer than a tuple; an option.
    #[test]
    fn keys_tuples_and_options_read_as_serde_json_reads_them() {
        use std::collections::BTreeMap;
        let years: BTreeMap<i32, Option<u8>> =
            deserialize_str(r#"{"-2": 1, "2026": null}"#, "years").unwrap();
        assert_eq!(years, BTreeMap::from([(-2, Some(1)), (2026, None)]));
        let refused = deserialize_str::<BTreeMap<u8, u8>>(r#"{"x": 1}"#, "years").unwrap_err();
        // A key that is not a number is named by itself, as every refused
        // key is.
        assert_eq!(refused.field(), Some("years.x"));
        let flags: BTreeMap<bool, u8> = deserialize_str(r#"{"true": 1}"#, "flags").unwrap();
        assert_eq!(flags.get(&true), Some(&1));
        let long = deserialize_str::<(u8, u8)>("[1, 2, 3]", "pair").unwrap_err();
        assert_eq!(long.field(), Some("pair"));
        assert!(
            long.to_string().contains("fewer elements in array"),
            "{long}"
        );
        let nested = deserialize_str::<Vec<Vec<u8>>>("[[1], [2, 256]]", "grid").unwrap_err();
        assert_eq!(nested.field(), Some("grid[1][1]"));
    }
}
