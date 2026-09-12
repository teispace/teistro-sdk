//! What the generated dispatch is built from: reading an argument,
//! checking a status, and nothing else.
//!
//! Hand-written on purpose, and small on purpose. `src/dispatch.rs` is
//! one arm per engine function and is regenerated whenever the engine
//! changes; anything it needs that is the *same* for every arm belongs
//! here, where it is written once and read by a person.

use serde_json::{Map, Value};
use teistro_port_ephemeris::ProviderError;

/// An argument that must be a number, as a `f64`.
///
/// # Errors
///
/// `Invalid` naming the argument when it is missing or is not a number.
/// Naming it matters: a call through this route is written by hand
/// against a manifest, so the mistake is nearly always a name.
pub(crate) fn number(args: &Map<String, Value>, name: &str) -> Result<f64, ProviderError> {
    match args.get(name) {
        Some(Value::Number(found)) => found.as_f64().ok_or_else(|| {
            ProviderError::invalid(format!("`{name}` is a number this engine cannot take"))
        }),
        Some(other) => Err(ProviderError::invalid(format!(
            "`{name}` must be a number; it is {}",
            kind(other)
        ))),
        None => Err(ProviderError::invalid(format!("`{name}` is required"))),
    }
}

/// An argument that must be a whole number, as an `i64`.
///
/// A JSON number with a fractional part is **refused** rather than
/// truncated. The engine's integer parameters are counts, ids and enum
/// members, and silently turning 2.7 into 2 would answer a question the
/// caller did not ask.
///
/// `true` and `false` are accepted for an integer, because C has no
/// separate boolean at this boundary and a caller writing JSON will reach
/// for one where the engine declares a flag.
///
/// # Errors
///
/// `Invalid` naming the argument when it is missing, not a number, or not
/// whole.
pub(crate) fn integer(args: &Map<String, Value>, name: &str) -> Result<i64, ProviderError> {
    match args.get(name) {
        Some(Value::Bool(flag)) => Ok(i64::from(*flag)),
        Some(Value::Number(found)) => {
            if let Some(whole) = found.as_i64() {
                return Ok(whole);
            }
            if let Some(unsigned) = found.as_u64() {
                return i64::try_from(unsigned).map_err(|_| {
                    ProviderError::invalid(format!("`{name}` is larger than this engine takes"))
                });
            }
            Err(ProviderError::invalid(format!(
                "`{name}` must be a whole number; it is {found}"
            )))
        }
        Some(other) => Err(ProviderError::invalid(format!(
            "`{name}` must be a whole number; it is {}",
            kind(other)
        ))),
        None => Err(ProviderError::invalid(format!("`{name}` is required"))),
    }
}

/// A whole-number argument narrowed to the width the engine declares.
///
/// **Refused rather than truncated.** The engine's integer parameters are
/// counts, ids and enum members, and a caller who passes a number too
/// large for one has made a mistake that silently keeping the low bits
/// would turn into a wrong answer instead of a message.
///
/// # Errors
///
/// `Invalid` naming the argument and the width it did not fit.
pub(crate) fn narrow<T>(args: &Map<String, Value>, name: &str) -> Result<T, ProviderError>
where
    T: TryFrom<i64>,
{
    let whole = integer(args, name)?;
    T::try_from(whole).map_err(|_| {
        ProviderError::invalid(format!(
            "`{name}` is {whole}, which does not fit the {} this engine declares",
            core::any::type_name::<T>()
        ))
    })
}

/// The engine's own status, as the port's error.
///
/// # Errors
///
/// Anything but success, carrying the engine's numeric code so a caller
/// can match on it, and the function's name so they know which call it
/// came from — the pass-through calls by name, so a bare code would leave
/// them guessing.
pub(crate) fn status(code: teimeris::sys::tm_status, function: &str) -> Result<(), ProviderError> {
    if code == 0 {
        return Ok(());
    }
    Err(ProviderError::Provider {
        code: crate::CODE_BASE + code,
        detail: format!("`{function}` refused with status {code}"),
    })
}

/// What a JSON value is, for a message that has to say so.
fn kind(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a string",
        Value::Array(_) => "an array",
        Value::Object(_) => "an object",
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        reason = "a test fails by panicking"
    )]

    use super::{integer, number};
    use serde_json::json;

    fn args(value: serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
        value.as_object().expect("an object").clone()
    }

    #[test]
    fn a_number_is_read_and_a_missing_one_is_named() {
        let map = args(json!({ "jd": 2_451_545.5 }));
        assert!((number(&map, "jd").unwrap() - 2_451_545.5).abs() < 1e-9);
        let error = number(&map, "jd_ut1").unwrap_err();
        assert!(
            error.to_string().contains("jd_ut1"),
            "the refusal must name the argument: {error}"
        );
    }

    /// A fractional value for a whole-number parameter is refused rather
    /// than truncated: the engine's integers are counts, ids and enum
    /// members, and 2.7 becoming 2 answers a question nobody asked.
    #[test]
    fn a_fraction_is_refused_where_a_whole_number_is_wanted() {
        let map = args(json!({ "system": 2.7, "count": 3, "flag": true }));
        let error = integer(&map, "system").unwrap_err();
        assert!(error.to_string().contains("whole"), "{error}");
        assert_eq!(integer(&map, "count").unwrap(), 3);
        assert_eq!(
            integer(&map, "flag").unwrap(),
            1,
            "a JSON boolean is a C flag"
        );
    }

    /// The wrong kind is named as what it is, because a caller writing
    /// against a manifest has usually passed the right name and the wrong
    /// shape.
    #[test]
    fn the_wrong_kind_is_named() {
        let map = args(json!({ "jd": "2451545.5" }));
        let error = number(&map, "jd").unwrap_err();
        assert!(error.to_string().contains("a string"), "{error}");
    }
}
