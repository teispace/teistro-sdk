//! The canonical form: bytes two bindings agree on.
//!
//! Two jobs pull apart here, so there are two functions rather than one
//! with a flag.
//!
//! [`to_hash_form`] is **fixed and settings-independent**: keys in
//! code-point order, no whitespace, and every number in the grammar
//! below. It is what [`hash_of`] is taken over, and it cannot honour a
//! display setting, because a hash that moves with one is a worse cache
//! key — a reader would find the same answer under two hashes and
//! recompute.
//!
//! [`to_rendered`] honours `output.precision`, which says how many
//! decimals an angle, an instant and a score are written to. It is a
//! rendering and is never hashed.
//!
//! # The number grammar
//!
//! A decimal with an optional leading `-`, at least one digit before the
//! point, and up to [`DECIMALS`] digits after it with trailing zeros
//! removed. **Never an exponent**, never a bare `.5`, never `-0`.
//!
//! That last rule is the reason this module exists. Rust's JSON layer
//! writes `1e-6` where JavaScript's writes `0.000001`, so two bindings
//! that agree about the number disagree about the bytes and therefore
//! about the hash (`03-design/serial-measured.md` §4). A binding
//! implementing this grammar needs no float printer of its own.
//!
//! ```
//! use teistro_serial::canonical::{to_hash_form, DECIMALS};
//!
//! // Never an exponent, whichever side of the threshold.
//! assert_eq!(to_hash_form(&1e-6_f64), "0.000001");
//! assert_eq!(to_hash_form(&1e-7_f64), "0.0000001");
//! // Trailing zeros go, and so does a negative nought.
//! assert_eq!(to_hash_form(&1.5_f64), "1.5");
//! assert_eq!(to_hash_form(&2.0_f64), "2");
//! assert_eq!(to_hash_form(&-0.0_f64), "0");
//! assert_eq!(DECIMALS, 12);
//! ```

use serde::Serialize;
use teistro_core::envelope::{
    CANONICAL_DECIMALS, Hash, canonical_json, canonical_json_at, decimal as core_decimal,
};
use teistro_core::error::Error;
use teistro_core::settings::Precision;

/// The decimals the hash form writes a number to, which is `core`'s own
/// [`CANONICAL_DECIMALS`]: the grammar lives with `content_hash`,
/// because there is one canonical form in the SDK and not two.
pub const DECIMALS: u8 = CANONICAL_DECIMALS;

/// The most decimals a rendering may ask for, which is the grammar's own.
pub const MOST_DECIMALS: u8 = DECIMALS;

/// The canonical bytes of a value, which [`hash_of`] is taken over.
///
/// Keys in code-point order at every depth, no whitespace, and every
/// number in the grammar this module documents.
#[must_use]
pub fn to_hash_form<T: Serialize + ?Sized>(value: &T) -> String {
    canonical_json(value)
}

/// The hash of a value's canonical bytes.
#[must_use]
pub fn hash_of<T: Serialize + ?Sized>(value: &T) -> Hash {
    Hash::of(to_hash_form(value).as_bytes())
}

/// A double as a plain decimal in the canonical grammar, which is
/// `core`'s own.
#[must_use]
pub fn decimal(value: f64, decimals: u8) -> String {
    core_decimal(value, decimals)
}

/// The same bytes with every number written to the precision the
/// settings ask for.
///
/// This is a **rendering**: it is what a consumer reads and stores, and
/// it is never hashed, because a hash that moved with a display setting
/// would make the cache key useless.
///
/// # Errors
///
/// `OUT_OF_RANGE` for a precision beyond the grammar's own
/// [`MOST_DECIMALS`], which is what a caller asking for more digits than
/// a double carries has done.
pub fn to_rendered<T: Serialize + ?Sized>(
    value: &T,
    precision: &Precision,
) -> Result<String, Error> {
    let decimals = widest(precision);
    if decimals > MOST_DECIMALS {
        return Err(Error::new(
            teistro_core::error::Status::OutOfRange,
            format!(
                "a rendering asks for {decimals} decimals and the canonical grammar \
                 writes at most {MOST_DECIMALS}"
            ),
        )
        .with_field("output.precision"));
    }
    Ok(canonical_json_at(value, decimals))
}

/// The widest of the three precisions a rendering may use.
#[must_use]
pub fn widest(precision: &Precision) -> u8 {
    precision
        .angle_decimals
        .max(precision.instant_decimals)
        .max(precision.score_decimals)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index their own JSON literals"
    )]

    use super::{DECIMALS, MOST_DECIMALS, decimal, hash_of, to_hash_form, to_rendered, widest};
    use serde_json::json;
    use teistro_core::settings::Precision;

    fn precision(angle: u8, instant: u8, score: u8) -> Precision {
        Precision {
            angle_decimals: angle,
            instant_decimals: instant,
            score_decimals: score,
        }
    }

    #[test]
    fn a_number_is_never_written_with_an_exponent() {
        // The two sides of the threshold the pass found: Rust's own JSON
        // layer writes `1e-6` here and JavaScript's writes `0.000001`.
        assert_eq!(to_hash_form(&1e-6_f64), "0.000001");
        assert_eq!(to_hash_form(&1e-7_f64), "0.0000001");
        assert_eq!(to_hash_form(&1e-12_f64), "0.000000000001");
        // And below the grammar's resolution a value rounds to nought.
        assert_eq!(to_hash_form(&1e-15_f64), "0");
        assert_eq!(to_hash_form(&1e16_f64), "10000000000000000");
    }

    #[test]
    fn trailing_zeros_and_a_negative_nought_have_one_spelling() {
        assert_eq!(to_hash_form(&2.0_f64), "2");
        assert_eq!(to_hash_form(&2.5000_f64), "2.5");
        assert_eq!(to_hash_form(&-0.0_f64), "0");
        assert_eq!(to_hash_form(&0.0_f64), "0");
        assert_eq!(
            decimal(-0.000_000_000_000_1, DECIMALS),
            "0",
            "it rounds to it"
        );
        assert_eq!(to_hash_form(&-1.5_f64), "-1.5");
    }

    #[test]
    fn an_object_comes_out_in_code_point_order_at_every_depth() {
        let value = json!({"b": 1, "a": {"z": 1, "y": [ {"n": 1, "m": 2} ]}});
        assert_eq!(
            to_hash_form(&value),
            r#"{"a":{"y":[{"m":2,"n":1}],"z":1},"b":1}"#
        );
        // And the same value built the other way about gives the bytes.
        let other = json!({"a": {"y": [ {"m": 2, "n": 1} ], "z": 1}, "b": 1});
        assert_eq!(to_hash_form(&value), to_hash_form(&other));
        assert_eq!(hash_of(&value), hash_of(&other));
    }

    #[test]
    fn a_string_is_escaped_the_one_way() {
        assert_eq!(to_hash_form(&"plain"), r#""plain""#);
        assert_eq!(to_hash_form(&"a\"b"), r#""a\"b""#);
        assert_eq!(to_hash_form(&"a\\b"), r#""a\\b""#);
        assert_eq!(to_hash_form(&"a\nb"), r#""a\nb""#);
        assert_eq!(
            to_hash_form(&"a\u{1}b"),
            r#""a\u0001b""#,
            "a control character is escaped, and the one way"
        );
        // A non-ASCII character is itself, not an escape: two writers
        // that escape differently would disagree about the bytes.
        assert_eq!(to_hash_form(&"नेपाल"), "\"नेपाल\"");
    }

    #[test]
    fn a_rendering_honours_the_precision_and_the_hash_form_does_not() {
        let value = json!({"angle": 1.234_567_890_123_4_f64});
        let coarse = to_rendered(&value, &precision(3, 3, 3)).unwrap();
        assert_eq!(coarse, r#"{"angle":1.235}"#);
        let fine = to_rendered(&value, &precision(9, 8, 3)).unwrap();
        assert_eq!(fine, r#"{"angle":1.23456789}"#);
        // The hash form ignores both.
        assert_eq!(to_hash_form(&value), r#"{"angle":1.234567890123}"#);
        assert_eq!(hash_of(&value), hash_of(&value));
        assert_ne!(coarse, fine, "the knob does something");
    }

    #[test]
    fn a_precision_beyond_the_grammar_is_refused_by_field() {
        let error = to_rendered(&json!(1.0), &precision(MOST_DECIMALS + 1, 0, 0))
            .expect_err("too many decimals");
        assert_eq!(error.field(), Some("output.precision"));
        assert!(error.message.contains("12"), "{error}");
        assert!(to_rendered(&json!(1.0), &precision(MOST_DECIMALS, 0, 0)).is_ok());
        assert_eq!(widest(&precision(9, 8, 3)), 9);
    }

    #[test]
    fn the_form_is_stable_and_a_change_moves_the_hash() {
        let value = json!({"a": 1, "b": [1.5, "x"]});
        assert_eq!(to_hash_form(&value), to_hash_form(&value));
        let mut moved = value.clone();
        moved["a"] = json!(2);
        assert_ne!(hash_of(&value), hash_of(&moved));
        // Two values differing below the grammar's resolution are one
        // answer, deliberately.
        assert_eq!(hash_of(&json!(1.0)), hash_of(&json!(1.000_000_000_000_01)));
    }
}
