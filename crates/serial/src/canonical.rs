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
//! point, and the **fewest** digits after it that still read back as the
//! same double, with trailing zeros removed. **Never an exponent**,
//! never a bare `.5`, never `-0`.
//!
//! Two rules, and each is here for a measured reason.
//!
//! *Never an exponent*, because Rust's JSON layer writes `1e-6` where
//! JavaScript's writes `0.000001`: two bindings that agree about the
//! number would disagree about the bytes and therefore about the hash
//! (`03-design/serial-measured.md` §4).
//!
//! *The fewest digits*, because a fixed count is not a fixed point. A
//! double's resolution depends on its magnitude: at a Julian day's
//! 2.46e6 one unit in the last place is about 5e-10, so writing twelve
//! decimals there writes three digits that are the decimal expansion of
//! a binary value rather than information, and they do not survive a
//! parse. A consumer that stored a document and hashed it again did not
//! get the producer's hash (`03-design/schema-measured.md` §9).
//!
//! A binding implementing this grammar needs no float printer of its
//! own: Rust's `Display` for a double is already the shortest form and
//! never writes an exponent, and JavaScript's `toString` is the same
//! shortest form outside 1e-6 to 1e21, where it must be expanded.
//!
//! ```
//! use teistro_serial::canonical::{to_hash_form, MOST_DECIMALS};
//!
//! // Never an exponent, whichever side of the threshold.
//! assert_eq!(to_hash_form(&1e-6_f64), "0.000001");
//! assert_eq!(to_hash_form(&1e-7_f64), "0.0000001");
//! // Trailing zeros go, and so does a negative nought.
//! assert_eq!(to_hash_form(&1.5_f64), "1.5");
//! assert_eq!(to_hash_form(&2.0_f64), "2");
//! assert_eq!(to_hash_form(&-0.0_f64), "0");
//! // The fewest digits that read back: writing what it wrote gives the
//! // same bytes, at any magnitude.
//! let instant = 2_460_483.108_666_389_249_f64;
//! assert_eq!(to_hash_form(&instant), "2460483.1086663892");
//! assert_eq!(MOST_DECIMALS, 12);
//! ```

use serde::Serialize;
use teistro_core::envelope::{
    CANONICAL_DECIMALS, Digits, Hash, canonical_json, canonical_json_at, decimal as core_decimal,
};
use teistro_core::error::Error;
use teistro_core::settings::Precision;

/// The most decimals a rendering may ask a number to be written to,
/// which is `core`'s own [`CANONICAL_DECIMALS`]: the grammar lives with
/// `content_hash`, because there is one canonical form in the SDK and
/// not two.
///
/// The **hash** form asks for no count at all — it writes the shortest
/// decimal that reads back as the same double ([`Digits::Shortest`]),
/// which is what makes it a fixed point.
pub const MOST_DECIMALS: u8 = CANONICAL_DECIMALS;

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
/// `core`'s own: [`Digits::Shortest`] for the bytes a hash is taken
/// over, [`Digits::Rounded`] for a rendering.
#[must_use]
pub fn decimal(value: f64, digits: Digits) -> String {
    core_decimal(value, digits)
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

    use super::{Digits, MOST_DECIMALS, decimal, hash_of, to_hash_form, to_rendered, widest};
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
        // However small, and however large: the form writes the value it
        // was given rather than rounding it away, because a hash form
        // that rounds is not a fixed point.
        assert_eq!(to_hash_form(&1e-15_f64), "0.000000000000001");
        assert_eq!(to_hash_form(&1e16_f64), "10000000000000000");
        assert_eq!(to_hash_form(&1e21_f64), "1000000000000000000000");
    }

    #[test]
    fn the_hash_form_is_a_fixed_point_at_any_magnitude() {
        // Writing what it wrote gives the same bytes. This is what the
        // content hash rests on: a consumer that stores a document and
        // hashes it again gets the producer's hash.
        //
        // A fixed count of decimals could not do it. At a Julian day's
        // magnitude an `f64` resolves about nine decimals, so the twelve
        // this grammar used to write were three digits of a binary
        // value's decimal expansion, and a parse did not return them
        // (`03-design/schema-measured.md` §9).
        for value in [
            // A Julian day, written at the precision a double carries:
            // the literal the doc comment above uses has three digits
            // more, and is the very same value.
            2_460_483.108_666_389_2_f64,
            2_460_482.5,
            0.1 + 0.2,
            1e-15,
            1e16,
            -85.324,
            0.0,
        ] {
            let once = to_hash_form(&value);
            let read: serde_json::Value =
                serde_json::from_str(&once).expect("the form reads back as JSON");
            assert_eq!(to_hash_form(&read), once, "{value} is not a fixed point");
            assert!(!once.contains('e'), "{value} wrote as {once}");
        }
    }

    #[test]
    fn trailing_zeros_and_a_negative_nought_have_one_spelling() {
        assert_eq!(to_hash_form(&2.0_f64), "2");
        assert_eq!(to_hash_form(&2.5000_f64), "2.5");
        assert_eq!(to_hash_form(&-0.0_f64), "0");
        assert_eq!(to_hash_form(&0.0_f64), "0");
        // A rendering rounds; the hash form does not, because rounding
        // is what stops it being a fixed point.
        assert_eq!(
            decimal(-0.000_000_000_000_1, Digits::Rounded(MOST_DECIMALS)),
            "0",
            "a rendering rounds to it"
        );
        assert_eq!(
            decimal(-0.000_000_000_000_1, Digits::Shortest),
            "-0.0000000000001",
            "the hash form keeps the value it was given"
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
        // The hash form ignores both, and rounds nothing: it writes the
        // shortest decimal that reads back as the same double.
        assert_eq!(to_hash_form(&value), r#"{"angle":1.2345678901234}"#);
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
        // Two doubles that differ at all hash differently. The form used
        // to round them together below twelve decimals, which made it
        // lossy in the one place that cannot afford to be: a document a
        // consumer stores and hashes again.
        assert_ne!(hash_of(&json!(1.0)), hash_of(&json!(1.000_000_000_000_01)));
        // Two spellings of one double are still one answer.
        assert_eq!(hash_of(&json!(1.0)), hash_of(&json!(1.000_f64)));
    }
}
