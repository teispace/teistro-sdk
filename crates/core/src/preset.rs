//! A rule set a request may name instead of spelling it out.
//!
//! A rule set the SDK ships whole (the baseline engine's readings, every
//! one of them) is a Rust constant, and a binding that had to write it out
//! value by value would keep a copy of it. So a request field holding such
//! a rule set reads either the record or one of its [`Presets`] by name,
//! `"BASELINE"`, as `MuhurtaRequest.rules` reads an activity's name, and
//! states both in its schema as an or-named union, which every binding's
//! generated request types then offer
//! (`docs/03-design/generated-requests.md`).
//!
//! ```
//! use serde::{Deserialize, Serialize};
//! use teistro_core::preset::{self, Presets};
//!
//! #[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
//! #[serde(default, deny_unknown_fields)]
//! struct Rules { strict: bool }
//!
//! impl Presets for Rules {
//!     const PRESETS: &'static [(&'static str, Rules)] = &[("BASELINE", Rules { strict: true })];
//! }
//!
//! #[derive(Debug, Serialize, Deserialize)]
//! struct Request {
//!     #[serde(default, deserialize_with = "preset::read")]
//!     rules: Rules,
//! }
//!
//! let named: Request = teistro_core::strict::read(r#"{"rules": "BASELINE"}"#, "request")?;
//! let spelt: Request = teistro_core::strict::read(r#"{"rules": {"strict": true}}"#, "request")?;
//! assert_eq!(named.rules, spelt.rules);
//! let wrong = teistro_core::strict::read::<Request>(r#"{"rules": "BASELIN"}"#, "request").unwrap_err();
//! assert_eq!(wrong.field(), Some("request.rules"));
//! # Ok::<(), teistro_core::Error>(())
//! ```

use core::fmt;
use core::marker::PhantomData;

use serde::de::{Deserialize, Deserializer, MapAccess, Visitor};

/// A rule set the SDK ships under names: each preset's name, as a request
/// writes it, and its value.
///
/// ```
/// use teistro_core::preset::Presets;
///
/// #[derive(Clone, Copy)]
/// struct Rules { strict: bool }
///
/// impl Presets for Rules {
///     const PRESETS: &'static [(&'static str, Rules)] = &[("BASELINE", Rules { strict: true })];
/// }
///
/// assert_eq!(Rules::named("BASELINE").map(|rules| rules.strict), Some(true));
/// assert!(Rules::named("baseline").is_none(), "a name is matched exactly");
/// ```
pub trait Presets: Sized + Copy + 'static {
    /// The presets, by the name a request writes.
    const PRESETS: &'static [(&'static str, Self)];

    /// The preset a name names, if one does: the name exactly as a request
    /// writes it.
    ///
    /// ```
    /// use teistro_core::preset::Presets;
    ///
    /// #[derive(Clone, Copy, Debug, PartialEq)]
    /// struct Orb(u8);
    ///
    /// impl Presets for Orb {
    ///     const PRESETS: &'static [(&'static str, Orb)] = &[("WIDE", Orb(8)), ("TIGHT", Orb(3))];
    /// }
    ///
    /// assert_eq!(Orb::named("TIGHT"), Some(Orb(3)));
    /// assert_eq!(Orb::named("NARROW"), None);
    /// ```
    #[must_use]
    fn named(name: &str) -> Option<Self> {
        Self::PRESETS
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, preset)| *preset)
    }
}

/// The names [`Presets::PRESETS`] gives, comma-separated, for a refusal.
fn names<T: Presets>() -> String {
    T::PRESETS
        .iter()
        .map(|(name, _)| *name)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Reads a rule set a request names (`"BASELINE"`) or spells out, as a
/// field's `#[serde(deserialize_with = "teistro_core::preset::read")]`.
///
/// A name that is none of the presets is refused naming them; a record is
/// read as the type reads it, so its own refusals keep their paths.
///
/// ```
/// use teistro_core::preset::{self, Presets};
///
/// #[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize)]
/// struct Rules { strict: bool }
///
/// impl Presets for Rules {
///     const PRESETS: &'static [(&'static str, Rules)] = &[("BASELINE", Rules { strict: true })];
/// }
///
/// let mut json = serde_json::Deserializer::from_str(r#""BASELINE""#);
/// assert_eq!(preset::read::<_, Rules>(&mut json)?, Rules { strict: true });
/// let mut wrong = serde_json::Deserializer::from_str(r#""STRICT""#);
/// let refused = preset::read::<_, Rules>(&mut wrong).unwrap_err().to_string();
/// assert!(refused.contains("it names BASELINE"), "{refused}");
/// # Ok::<(), serde_json::Error>(())
/// ```
///
/// # Errors
///
/// A string that names no preset, or a value the type does not read.
pub fn read<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Presets + Deserialize<'de>,
{
    /// The two shapes: a preset's name, or the record.
    struct Shapes<T>(PhantomData<T>);

    impl<'de, T: Presets + Deserialize<'de>> Visitor<'de> for Shapes<T> {
        type Value = T;

        fn expecting(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(out, "one of {}, or the rules spelt out", names::<T>())
        }

        fn visit_str<E: serde::de::Error>(self, name: &str) -> Result<T, E> {
            T::named(name).ok_or_else(|| {
                E::custom(format!(
                    "`{name}` is not one the SDK names; it names {}, or the rules may be \
                     spelt out",
                    names::<T>()
                ))
            })
        }

        fn visit_map<M: MapAccess<'de>>(self, map: M) -> Result<T, M::Error> {
            T::deserialize(serde::de::value::MapAccessDeserializer::new(map))
        }
    }

    deserializer.deserialize_any(Shapes(PhantomData))
}

/// The schema [`read`] reads: an or-named union of the presets' names and
/// the rule set, as a field's
/// `#[schemars(schema_with = "teistro_core::preset::schema::<T>")]`.
///
/// ```
/// use teistro_core::preset::{self, Presets};
///
/// #[derive(Clone, Copy, schemars::JsonSchema)]
/// struct Rules { strict: bool }
///
/// impl Presets for Rules {
///     const PRESETS: &'static [(&'static str, Rules)] = &[("BASELINE", Rules { strict: true })];
/// }
///
/// let mut generator = schemars::SchemaGenerator::default();
/// let schema = preset::schema::<Rules>(&mut generator);
/// assert_eq!(schema.get("anyOf").and_then(|options| options.get(0)),
///     Some(&serde_json::json!({"type": "string", "enum": ["BASELINE"]})));
/// ```
#[cfg(feature = "schema")]
pub fn schema<T: Presets + schemars::JsonSchema>(
    generator: &mut schemars::SchemaGenerator,
) -> schemars::Schema {
    let names: Vec<&str> = T::PRESETS.iter().map(|(name, _)| *name).collect();
    schemars::json_schema!({
        "anyOf": [
            { "type": "string", "enum": names },
            generator.subschema_for::<T>(),
        ],
    })
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize)]
    #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
    #[serde(deny_unknown_fields)]
    struct Orb {
        degrees: u8,
    }

    impl Presets for Orb {
        const PRESETS: &'static [(&'static str, Orb)] =
            &[("WIDE", Orb { degrees: 8 }), ("TIGHT", Orb { degrees: 3 })];
    }

    fn read_json(json: &str) -> Result<Orb, serde_json::Error> {
        read(&mut serde_json::Deserializer::from_str(json))
    }

    /// A name is matched exactly, and every preset is reachable by its own.
    #[test]
    fn every_preset_is_named_and_nothing_else_is() {
        for (name, value) in Orb::PRESETS {
            assert_eq!(Orb::named(name), Some(*value));
            assert_eq!(read_json(&format!("\"{name}\"")).unwrap(), *value);
        }
        assert_eq!(Orb::named("tight"), None);
        assert_eq!(names::<Orb>(), "WIDE, TIGHT");
    }

    /// A wrong name is refused naming every preset, and that a record may
    /// be spelt out instead.
    #[test]
    fn a_wrong_name_is_refused_naming_the_presets() {
        let refused = read_json(r#""NARROW""#).unwrap_err().to_string();
        assert!(
            refused.contains("`NARROW` is not one the SDK names"),
            "{refused}"
        );
        assert!(refused.contains("it names WIDE, TIGHT"), "{refused}");
        assert!(refused.contains("spelt out"), "{refused}");
    }

    /// A record is read as the type reads it: its own refusals stay its own.
    #[test]
    fn a_record_is_read_as_its_type_reads_it() {
        assert_eq!(read_json(r#"{"degrees": 5}"#).unwrap(), Orb { degrees: 5 });
        let unknown = read_json(r#"{"degrees": 5, "width": 1}"#)
            .unwrap_err()
            .to_string();
        assert!(unknown.contains("width"), "{unknown}");
        let missing = read_json("{}").unwrap_err().to_string();
        assert!(missing.contains("degrees"), "{missing}");
    }

    /// Neither a name nor a record is refused saying what was expected.
    #[test]
    fn neither_shape_is_refused_saying_what_was_expected() {
        for json in ["5", "true", "null", "[]"] {
            let refused = read_json(json).unwrap_err().to_string();
            assert!(
                refused.contains("one of WIDE, TIGHT, or the rules spelt out"),
                "{json}: {refused}"
            );
        }
    }

    /// The schema lists every name, then the record by its reference.
    #[cfg(feature = "schema")]
    #[test]
    fn the_schema_is_the_names_or_the_record() {
        let mut generator = schemars::SchemaGenerator::default();
        let union = schema::<Orb>(&mut generator);
        let options = union
            .get("anyOf")
            .and_then(serde_json::Value::as_array)
            .unwrap();
        assert_eq!(options.len(), 2);
        assert_eq!(
            options.first(),
            Some(&serde_json::json!({"type": "string", "enum": ["WIDE", "TIGHT"]}))
        );
        assert!(
            options
                .get(1)
                .and_then(|record| record.get("$ref"))
                .is_some()
        );
    }
}
