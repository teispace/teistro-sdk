//! What the generated dispatch is built from: reading an argument,
//! checking a status, and nothing else.
//!
//! Hand-written on purpose, and small on purpose. `src/dispatch.rs` is
//! one arm per engine function and is regenerated whenever the engine
//! changes; anything it needs that is the *same* for every arm belongs
//! here, where it is written once and read by a person.

use core::ffi::{CStr, c_char};
use std::ffi::CString;

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

/// An argument that must be a string, as the C string the engine takes.
///
/// Returned rather than passed straight through, because the buffer has
/// to outlive the call and a temporary would not: the generated code
/// binds it and then lends its pointer.
///
/// # Errors
///
/// `Invalid` naming the argument when it is missing, is not a string, or
/// contains a NUL — which C has no way to carry and which would
/// otherwise hand the engine a silently shortened path.
pub(crate) fn text(args: &Map<String, Value>, name: &str) -> Result<CString, ProviderError> {
    match args.get(name) {
        Some(Value::String(found)) => CString::new(found.as_str()).map_err(|_| {
            ProviderError::invalid(format!("`{name}` contains a NUL, which C cannot carry"))
        }),
        Some(other) => Err(ProviderError::invalid(format!(
            "`{name}` must be a string; it is {}",
            kind(other)
        ))),
        None => Err(ProviderError::invalid(format!("`{name}` is required"))),
    }
}

/// A string the engine lent: copied at once, because what it points at
/// belongs to the engine and may not outlive the next call.
///
/// `None` for null, which is how the engine says "no such thing" from a
/// function that returns a name.
#[allow(
    unsafe_code,
    reason = "reading a string the engine returned; the generated dispatch \
              is the only caller and hands it exactly what the engine gave"
)]
pub(crate) fn borrowed(pointer: *const c_char) -> Option<String> {
    if pointer.is_null() {
        return None;
    }
    // SAFETY: the caller passes a pointer the engine returned from a
    // function declared to return a NUL-terminated string.
    Some(unsafe { CStr::from_ptr(pointer) }.to_string_lossy().into_owned())
}

/// A string the engine fills a buffer with, by the engine's own
/// documented protocol: the call answers **the length it wanted**, and a
/// buffer too short is filled to its limit rather than refused.
///
/// So the first call is made into a buffer generous enough for every
/// answer these functions have — which is one call, no allocation, and
/// no probe — and only an answer that did not fit is asked for again
/// into a buffer sized to what it said it wanted. `call` is invoked at
/// most twice and must be the same call both times.
///
/// The `size_t` a function like this returns is therefore **the
/// protocol's bookkeeping and not an answer**: the string it describes
/// has already arrived in full, which is why the generated dispatch
/// reports the string alone.
///
/// # Errors
///
/// `Refused` when the second call wanted more than the first — a
/// provider whose answer changed underneath the fill, which a caller
/// must be told about rather than handed a truncation.
pub(crate) fn fill<F>(function: &str, mut call: F) -> Result<String, ProviderError>
where
    F: FnMut(*mut c_char, usize) -> usize,
{
    /// Room for every string these functions answer with, a body name and
    /// a formatted angle among them, on the stack and in one call.
    ///
    /// Not a limit: an answer longer than this is asked for again into
    /// exactly the room it wanted. It is the point at which one call
    /// stops being enough, and it is generous because the engine's own
    /// fast path for a formatted angle needs sixty-four bytes to avoid a
    /// copy.
    const ROOMY: usize = 128;

    let mut buffer = [0_u8; ROOMY];
    let wanted = call(buffer.as_mut_ptr().cast(), buffer.len());
    if let Some(answer) = fitted(&buffer, wanted) {
        return Ok(answer);
    }
    let mut grown = vec![0_u8; wanted.saturating_add(1)];
    let again = call(grown.as_mut_ptr().cast(), grown.len());
    fitted(&grown, again).ok_or_else(|| ProviderError::Refused {
        detail: format!(
            "`{function}` wanted {wanted} bytes and then {again}; its answer changed between \
             the two calls"
        ),
    })
}

/// What the engine wrote, if what it wanted fit in what it was given.
///
/// `wanted == len` is a truncation and not a fit: the engine keeps a byte
/// of the buffer for the NUL, so a buffer exactly as long as the answer
/// came back one character short.
fn fitted(buffer: &[u8], wanted: usize) -> Option<String> {
    if wanted >= buffer.len() {
        return None;
    }
    buffer
        .get(..wanted)
        .map(|written| String::from_utf8_lossy(written).into_owned())
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

    use super::{borrowed, fill, integer, number, text};
    use core::ffi::c_char;
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

    /// A NUL inside a string is refused rather than carried, because C
    /// stops there: a path with one would reach the engine silently
    /// shortened, and the engine would answer about a different file.
    #[test]
    fn a_nul_inside_a_string_is_refused() {
        let map = args(json!({ "path": "de441.eph", "cut": "de441\u{0}.eph" }));
        assert_eq!(text(&map, "path").unwrap().to_str().unwrap(), "de441.eph");
        let error = text(&map, "cut").unwrap_err();
        assert!(error.to_string().contains("cut"), "{error}");
        assert!(error.to_string().contains("NUL"), "{error}");
    }

    /// The engine says "there is no such name" by answering null, and
    /// JSON has a word for that which is not the empty string.
    #[test]
    fn a_null_string_is_none_and_not_empty() {
        assert_eq!(borrowed(core::ptr::null()), None);
        let word = c"Aries";
        assert_eq!(
            borrowed(word.as_ptr().cast::<c_char>()),
            Some("Aries".to_string())
        );
    }

    /// The fill protocol in all three of its outcomes, against a fake
    /// engine: this is the part the generator cannot test, because no
    /// function this adapter offers answers more than a line and the
    /// second call would never run against the real one.
    #[test]
    fn a_fill_asks_twice_only_when_the_first_answer_did_not_fit() {
        /// Writes `answer` into whatever room it is given, truncating to
        /// fit, and returns the length it wanted — the engine's own
        /// documented contract.
        fn engine(answer: &str, buffer: *mut c_char, capacity: usize) -> usize {
            if capacity > 0 {
                let room = capacity - 1;
                let take = room.min(answer.len());
                // SAFETY: the caller is `fill`, which passes a buffer of
                // the capacity it says; `take` is inside it.
                #[allow(unsafe_code, reason = "a fake engine, filling as C does")]
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        answer.as_ptr(),
                        buffer.cast::<u8>(),
                        take,
                    );
                    buffer.add(take).write(0);
                }
            }
            answer.len()
        }

        let mut calls = 0_u32;
        let short = fill("short", |buffer, capacity| {
            calls += 1;
            engine("Sun", buffer, capacity)
        })
        .unwrap();
        assert_eq!(short, "Sun");
        assert_eq!(calls, 1, "an answer that fits costs one call");

        let long = "x".repeat(400);
        let mut calls = 0_u32;
        let grown = fill("long", |buffer, capacity| {
            calls += 1;
            engine(&long, buffer, capacity)
        })
        .unwrap();
        assert_eq!(grown, long, "the whole answer, not the first bufferful");
        assert_eq!(calls, 2, "one to find out, one to fill");

        // A provider whose answer changed underneath the fill. Told
        // about, rather than papered over with a truncation.
        let mut calls = 0_u32;
        let Err(error) = fill("growing", |buffer, capacity| {
            calls += 1;
            engine(&"y".repeat(200 * calls as usize), buffer, capacity)
        }) else {
            panic!("an answer that keeps growing must be refused");
        };
        assert!(error.to_string().contains("growing"), "{error}");
    }
}
