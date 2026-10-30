//! Naming a child at the C boundary (nāmakaraṇa): the syllables a pada
//! names a child by, names read and checked, the names a birth gives, and
//! a consumer's name book loaded and suggested from, each answered as
//! canonical JSON (`03-design/namakarana.md` §4.2).
//!
//! The answers are nested records of text — spellings, a syllable's
//! initials, a reading's clause rows, a refusal a name, a book's own
//! tags — so they cross as the JSON the façade's types serialise to, as
//! numerology's profile does, rather than as columns beside a text
//! section: every binding parses it into its own types, and the agent
//! server answers the same bytes.
#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

use core::ffi::c_char;

#[cfg(feature = "chart")]
use teistro_core::error::Error;
use teistro_core::error::Status;

use crate::context::TsContext;
use crate::family::in_family;
use crate::string::TsString;
use crate::support::with_context;

/// `request_json` read as text, answered by `answer` and written to
/// `out_json` as canonical JSON: every naming entry point is this with
/// its own request.
///
/// # Safety
///
/// As the entry points': `request_json` NUL-terminated, `out_json` valid
/// for a write.
#[cfg(feature = "chart")]
unsafe fn answered<T: serde::Serialize>(
    request_json: *const c_char,
    out_json: *mut TsString,
    answer: impl FnOnce(&str) -> Result<T, Error>,
) -> Result<(), Error> {
    // SAFETY: the caller's contract.
    let text = unsafe { crate::support::text(request_json, "request_json") }?;
    let json = TsString::from_string(teistro_core::envelope::canonical_json(&answer(text)?));
    // SAFETY: the caller's contract.
    unsafe { crate::support::write_plain(out_json, "out_json", json) }
}

/// The syllables a pada, a star or a sign names a child by, in the
/// śatapada cakra's order, each with every initial that reaches it: its
/// own letter, a paired letter (C294) and the patro's substitute for a
/// nasal no name begins with (C770). A sign's nine padas, and Makara's
/// four more of Abhijit's (C771), whose `pada` is null.
///
/// `request_json` is `{"of", "rules"}`: `of` is `{"of": "PADA",
/// "nakshatra", "pada"}`, `{"of": "NAKSHATRA", "nakshatra"}` or `{"of":
/// "RASHI", "rashi"}`, and `rules` the `NamingRules` with every field
/// optional. The answer is a list of `PadaSyllable`. A pada outside 1 to
/// 4 is `INVALID_ARG`, named `naming.of.pada`; an unknown key is refused
/// by its path under `naming`.
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::None)?;
/// # fn owned(mut out: teistro_ffi::string::TsString) -> String {
/// #     // SAFETY: the library wrote `len` bytes of UTF-8 at `data`.
/// #     let text = String::from_utf8_lossy(unsafe { core::slice::from_raw_parts(out.data, out.len) }).into_owned();
/// #     // SAFETY: a descriptor the library wrote, freed once.
/// #     unsafe { teistro_ffi::string::ts_string_free(&raw mut out) };
/// #     text
/// # }
/// use teistro_core::Status;
/// use teistro_ffi::naming::ts_naming_syllables;
/// use teistro_ffi::string::TsString;
///
/// let request = c"{\"of\": {\"of\": \"PADA\", \"nakshatra\": \"ASHWINI\", \"pada\": 1}}";
/// let mut out = TsString::empty();
/// // SAFETY: a live context, a NUL-terminated request and a writable slot.
/// let status = unsafe { ts_naming_syllables(&raw const ctx, request.as_ptr(), &raw mut out) };
/// assert_eq!(status, Status::Ok);
/// let answer: serde_json::Value = serde_json::from_str(&owned(out))?;
/// assert_eq!(answer[0]["devanagari"], "चू");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `request_json` NUL-terminated;
/// `out_json` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_naming_syllables(
    context: *const TsContext,
    request_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        in_family!("chart", [request_json, out_json, ctx], {
            // SAFETY: the entry point's contract, passed on.
            unsafe {
                answered(request_json, out_json, |text| {
                    teistro::NamingSyllablesRequest::from_json(text)?.answer(ctx.sdk())
                })
            }
        })
    })
}

/// Names read into the śatapada cakra, each on its own: every first
/// syllable its spelling reaches (one, unless `CASUAL` reads it more
/// ways), each with the Devanagari it was read as, its sign, its
/// syllable count and the clauses of a name in daily use (C291 to C294,
/// C772, C774).
///
/// `request_json` is `{"names", "rules"}`: the names, in Devanagari or
/// in the Latin scheme `rules.name.latin` declares, and the
/// `NamingRules` with every field optional. The answer is a list with one
/// entry a name, in the request's order, each `{"reading", "refused"}`
/// with one of the two null: a name the cakra does not read is refused
/// alone, named `naming.names[i]`, and the batch never fails as a whole.
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::None)?;
/// # fn owned(mut out: teistro_ffi::string::TsString) -> String {
/// #     // SAFETY: the library wrote `len` bytes of UTF-8 at `data`.
/// #     let text = String::from_utf8_lossy(unsafe { core::slice::from_raw_parts(out.data, out.len) }).into_owned();
/// #     // SAFETY: a descriptor the library wrote, freed once.
/// #     unsafe { teistro_ffi::string::ts_string_free(&raw mut out) };
/// #     text
/// # }
/// use teistro_core::Status;
/// use teistro_ffi::naming::ts_naming_read;
/// use teistro_ffi::string::TsString;
///
/// let request = c"{\"names\": [\"राम\", \"ऐश्वर्य\"]}";
/// let mut out = TsString::empty();
/// // SAFETY: a live context, a NUL-terminated request and a writable slot.
/// let status = unsafe { ts_naming_read(&raw const ctx, request.as_ptr(), &raw mut out) };
/// assert_eq!(status, Status::Ok);
/// let answer: serde_json::Value = serde_json::from_str(&owned(out))?;
/// assert_eq!(answer[0]["reading"]["readings"][0]["clauses"]["syllables"], 2);
/// assert_eq!(answer[1]["refused"]["field"], "naming.names[1]");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `request_json` NUL-terminated;
/// `out_json` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_naming_read(
    context: *const TsContext,
    request_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        in_family!("chart", [request_json, out_json, ctx], {
            // SAFETY: the entry point's contract, passed on.
            unsafe {
                answered(request_json, out_json, |text| {
                    Ok(teistro::NamingReadRequest::from_json(text)?.answer(ctx.sdk()))
                })
            }
        })
    })
}

/// Names checked against a birth's pada, each on its own: where each
/// begins, the pada's own syllable with its initials, whether the two
/// share a star or a pada, whether the name took the pada's substitute,
/// and its clauses (C334, C770, C774). Facts, not a score.
///
/// `request_json` is `{"names", "birth", "rules"}`: the names, the birth
/// as `{"nakshatra", "pada"}`, and the `NamingRules` with every field
/// optional. The answer is a list with one entry a name, each `{"check",
/// "refused"}` with one of the two null; a name refused alone is named
/// `naming.names[i]`. A pada outside 1 to 4 refuses the call, named
/// `naming.birth.pada`.
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::None)?;
/// # fn owned(mut out: teistro_ffi::string::TsString) -> String {
/// #     // SAFETY: the library wrote `len` bytes of UTF-8 at `data`.
/// #     let text = String::from_utf8_lossy(unsafe { core::slice::from_raw_parts(out.data, out.len) }).into_owned();
/// #     // SAFETY: a descriptor the library wrote, freed once.
/// #     unsafe { teistro_ffi::string::ts_string_free(&raw mut out) };
/// #     text
/// # }
/// use teistro_core::Status;
/// use teistro_ffi::naming::ts_naming_check;
/// use teistro_ffi::string::TsString;
///
/// // Dharmasindhu's Cūḍāmaṇi for an Ashvini first-pada birth.
/// let request = c"{\"names\": [\"चूडामणि\"], \"birth\": {\"nakshatra\": \"ASHWINI\", \"pada\": 1}}";
/// let mut out = TsString::empty();
/// // SAFETY: a live context, a NUL-terminated request and a writable slot.
/// let status = unsafe { ts_naming_check(&raw const ctx, request.as_ptr(), &raw mut out) };
/// assert_eq!(status, Status::Ok);
/// let answer: serde_json::Value = serde_json::from_str(&owned(out))?;
/// assert_eq!(answer[0]["check"]["samePada"], true);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `request_json` NUL-terminated;
/// `out_json` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_naming_check(
    context: *const TsContext,
    request_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        in_family!("chart", [request_json, out_json, ctx], {
            // SAFETY: the entry point's contract, passed on.
            unsafe {
                answered(request_json, out_json, |text| {
                    teistro::NamingCheckRequest::from_json(text)?.answer(ctx.sdk())
                })
            }
        })
    })
}

/// The names a birth gives, of *Dharmasindhu*'s four that the SDK
/// computes (C773): the birth pada's syllable with its initials, and the
/// star's deity, whose name the Śāṅkhāyanas give.
///
/// `request_json` is `{"birth", "rules"}`. The answer is `Names`. A pada
/// outside 1 to 4 is `INVALID_ARG`, named `naming.birth.pada`.
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::None)?;
/// # fn owned(mut out: teistro_ffi::string::TsString) -> String {
/// #     // SAFETY: the library wrote `len` bytes of UTF-8 at `data`.
/// #     let text = String::from_utf8_lossy(unsafe { core::slice::from_raw_parts(out.data, out.len) }).into_owned();
/// #     // SAFETY: a descriptor the library wrote, freed once.
/// #     unsafe { teistro_ffi::string::ts_string_free(&raw mut out) };
/// #     text
/// # }
/// use teistro_core::Status;
/// use teistro_ffi::naming::ts_naming_of;
/// use teistro_ffi::string::TsString;
///
/// // Dharmasindhu's Āgniśarman: a Krittika birth named after Agni.
/// let request = c"{\"birth\": {\"nakshatra\": \"KRITTIKA\", \"pada\": 1}}";
/// let mut out = TsString::empty();
/// // SAFETY: a live context, a NUL-terminated request and a writable slot.
/// let status = unsafe { ts_naming_of(&raw const ctx, request.as_ptr(), &raw mut out) };
/// assert_eq!(status, Status::Ok);
/// let answer: serde_json::Value = serde_json::from_str(&owned(out))?;
/// assert_eq!(answer["deity"], "AGNI");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `request_json` NUL-terminated;
/// `out_json` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_naming_of(
    context: *const TsContext,
    request_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        in_family!("chart", [request_json, out_json, ctx], {
            // SAFETY: the entry point's contract, passed on.
            unsafe {
                answered(request_json, out_json, |text| {
                    teistro::NamingOfRequest::from_json(text)?.answer(ctx.sdk())
                })
            }
        })
    })
}

/// Loads a consumer's name book into the context: every name read once
/// into the cakra and indexed by its cell, kept for the context's life
/// under the book's id, the SHA-256 of its canonical JSON, so the same
/// book loaded twice is one book (`03-design/namakarana.md` §3).
///
/// `request_json` is `{"book"}`, the book `{"names": [{"devanagari",
/// "latin", "sex", "tags"}]}` with only `devanagari` required. The answer
/// is `{"book", "names", "refused"}`: the id a suggestion names it by,
/// how many names it holds, and every name the cakra refused (an ai or
/// au initial), listed and not dropped. A name with no Devanagari letter
/// is `INVALID_ARG`, named `naming.book.names[i].devanagari`; a `sex`
/// that is not a key, `naming.book.names[i].sex`.
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::None)?;
/// # fn owned(mut out: teistro_ffi::string::TsString) -> String {
/// #     // SAFETY: the library wrote `len` bytes of UTF-8 at `data`.
/// #     let text = String::from_utf8_lossy(unsafe { core::slice::from_raw_parts(out.data, out.len) }).into_owned();
/// #     // SAFETY: a descriptor the library wrote, freed once.
/// #     unsafe { teistro_ffi::string::ts_string_free(&raw mut out) };
/// #     text
/// # }
/// use teistro_core::Status;
/// use teistro_ffi::naming::ts_naming_load;
/// use teistro_ffi::string::TsString;
///
/// let request = c"{\"book\": {\"names\": [{\"devanagari\": \"चूडामणि\"}, {\"devanagari\": \"औषधि\"}]}}";
/// let mut out = TsString::empty();
/// // SAFETY: a live context, a NUL-terminated request and a writable slot.
/// let status = unsafe { ts_naming_load(&raw const ctx, request.as_ptr(), &raw mut out) };
/// assert_eq!(status, Status::Ok);
/// let answer: serde_json::Value = serde_json::from_str(&owned(out))?;
/// assert_eq!((answer["names"].as_u64(), answer["refused"][0]["index"].as_u64()), (Some(2), Some(1)));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `request_json` NUL-terminated;
/// `out_json` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_naming_load(
    context: *const TsContext,
    request_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        in_family!("chart", [request_json, out_json, ctx], {
            // SAFETY: the entry point's contract, passed on.
            unsafe {
                answered(request_json, out_json, |text| {
                    teistro::NamingLoadRequest::from_json(text)?.answer(ctx.sdk())
                })
            }
        })
    })
}

/// The names a book suggests for a birth: those whose first syllable is
/// the birth pada's, or reaches it by a paired letter or the pada's
/// substitute, in the cakra's order and then the book's, each with its
/// clauses under the rules; and every name the book holds that the cakra
/// refused. A filter keeps some and never ranks.
///
/// `request_json` is `{"bookId" | "book", "birth", "filter", "rules"}`:
/// a book this context loaded, by the id its load answered, or a book
/// carried for this one call; the birth; the `SuggestFilter` (`sex`,
/// `syllables`, `clauses`); and the `NamingRules`. The answer is
/// `Suggestions`. Neither or both of `bookId` and `book`, a book not
/// loaded in this context, a pada outside 1 to 4 or a filter clause the
/// sutra does not report is `INVALID_ARG`, named under `naming`.
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::None)?;
/// # fn owned(mut out: teistro_ffi::string::TsString) -> String {
/// #     // SAFETY: the library wrote `len` bytes of UTF-8 at `data`.
/// #     let text = String::from_utf8_lossy(unsafe { core::slice::from_raw_parts(out.data, out.len) }).into_owned();
/// #     // SAFETY: a descriptor the library wrote, freed once.
/// #     unsafe { teistro_ffi::string::ts_string_free(&raw mut out) };
/// #     text
/// # }
/// use teistro_core::Status;
/// use teistro_ffi::naming::ts_naming_suggest;
/// use teistro_ffi::string::TsString;
///
/// // Ardra's third pada is ङ, which the patro reads as ग.
/// let request = c"{\"book\": {\"names\": [{\"devanagari\": \"गणेश\"}, {\"devanagari\": \"राम\"}]}, \"birth\": {\"nakshatra\": \"ARDRA\", \"pada\": 3}}";
/// let mut out = TsString::empty();
/// // SAFETY: a live context, a NUL-terminated request and a writable slot.
/// let status = unsafe { ts_naming_suggest(&raw const ctx, request.as_ptr(), &raw mut out) };
/// assert_eq!(status, Status::Ok);
/// let answer: serde_json::Value = serde_json::from_str(&owned(out))?;
/// assert_eq!(answer["names"][0]["by"], "SUBSTITUTE");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `request_json` NUL-terminated;
/// `out_json` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_naming_suggest(
    context: *const TsContext,
    request_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        in_family!("chart", [request_json, out_json, ctx], {
            // SAFETY: the entry point's contract, passed on.
            unsafe {
                answered(request_json, out_json, |text| {
                    teistro::NamingSuggestRequest::from_json(text)?.answer(ctx.sdk())
                })
            }
        })
    })
}
