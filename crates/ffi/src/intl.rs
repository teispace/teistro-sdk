//! The locale engine at the boundary: the SDK's bundles are embedded, a
//! consumer's packs load after construction, the locale is chosen
//! explicitly, and a render hands back its text, where it resolved from
//! and its warnings as a blob (`docs/03-design/intl-engine-and-packs.md`).

#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

use core::ffi::c_char;
use core::fmt::Write as _;

use teistro_core::Status;
use teistro_core::error::Error;
use teistro_idl::blob::{FixedValue, Writer};
use teistro_intl::Params;
use teistro_intl::translit::{Script, transliterate};

use crate::blob::TsBlob;
use crate::context::{TsContext, unknown_locale};
use crate::schemas;
use crate::string::{TsStr, TsString};
use crate::support::{bytes, c_struct, optional_text, text, with_context, write_out, write_plain};

/// What a loaded pack or bundle carried.
///
/// ```
/// use teistro_ffi::intl::TsIntlLoaded;
///
/// // What a caller allocates for `ts_intl_load_pack` to fill.
/// let loaded = TsIntlLoaded {
///     struct_size: u32::try_from(size_of::<TsIntlLoaded>())?,
///     entries: 0, replaced: 0, merged: 0,
///     locale: core::ptr::null(), sha256: core::ptr::null(),
/// };
/// assert_eq!(loaded.entries, 0);
/// # Ok::<(), core::num::TryFromIntError>(())
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TsIntlLoaded {
    /// `sizeof(ts_intl_loaded)` as the caller compiled it.
    pub struct_size: u32,
    /// The entries the file carried.
    pub entries: u32,
    /// The entries that stood where one already stood and kept nothing of
    /// it.
    pub replaced: u32,
    /// The entity records that stood where one already stood and kept a
    /// form, a gender or a glyph the file did not carry.
    pub merged: u32,
    /// The locale; lent until the next call on the context.
    pub locale: *const c_char,
    /// The file's SHA-256 as sixty-four hex digits; lent until the next
    /// call on the context.
    pub sha256: *const c_char,
}

c_struct!(TsIntlLoaded);

/// Loads a `.tpack` or `.tbundle` file: a locale it brings is added, and a
/// namespace it brings is laid over what was loaded under the same keys —
/// a message replaces, and two entity records merge their forms, so a pack
/// giving every nakshatra a `phala` leaves its `name` standing. A file that
/// does not verify is `PACK`.
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::Test)?;
/// use teistro_core::Status;
/// use teistro_ffi::intl::{TsIntlLoaded, ts_intl_load_pack};
///
/// let mut loaded = TsIntlLoaded {
///     struct_size: u32::try_from(size_of::<TsIntlLoaded>())?,
///     entries: 0, replaced: 0, merged: 0, locale: core::ptr::null(), sha256: core::ptr::null(),
/// };
/// // Bytes that are no pack are refused, and nothing loads.
/// let bytes = b"not a pack";
/// // SAFETY: a live context, readable bytes and a sized record.
/// let status = unsafe { ts_intl_load_pack(&raw const ctx, bytes.as_ptr(), bytes.len(), &raw mut loaded) };
/// assert_ne!(status, Status::Ok);
/// assert_eq!(loaded.entries, 0);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `bytes` valid for `bytes_len` reads;
/// `out_loaded` valid for a read of its `struct_size` and a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_intl_load_pack(
    context: *const TsContext,
    bytes: *const u8,
    bytes_len: usize,
    out_loaded: *mut TsIntlLoaded,
) -> Status {
    with_context(context, |ctx| {
        // SAFETY: the entry point's contract.
        let file = unsafe { self::bytes(bytes, bytes_len, "bytes") }?;
        let loaded = ctx
            .intl_mut()
            .load_pack(file)
            .map_err(|e| Error::new(Status::Pack, e.to_string()).with_field("bytes"))?;
        let out = TsIntlLoaded {
            struct_size: 0,
            entries: u32::try_from(loaded.entries).unwrap_or(u32::MAX),
            replaced: u32::try_from(loaded.replaced).unwrap_or(u32::MAX),
            merged: u32::try_from(loaded.merged).unwrap_or(u32::MAX),
            locale: ctx.lend(&loaded.locale).data,
            sha256: ctx.lend(&loaded.sha256).data,
        };
        // SAFETY: the entry point's contract.
        unsafe { write_out(out_loaded, "out_loaded", out) }
    })
}

/// Selects the locale every render resolves from; an unknown one is
/// `UNSUPPORTED` naming the loaded locales.
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::Test)?;
/// use teistro_core::Status;
/// use teistro_ffi::intl::ts_intl_set_locale;
///
/// // SAFETY: a live context and NUL-terminated tags.
/// assert_eq!(unsafe { ts_intl_set_locale(&raw const ctx, c"ne-Deva-NP".as_ptr()) }, Status::Ok);
/// assert_eq!(ctx.intl().locale(), "ne-Deva-NP");
/// // A locale with no source is refused.
/// // SAFETY: as above.
/// assert_ne!(unsafe { ts_intl_set_locale(&raw const ctx, c"xx-Nope".as_ptr()) }, Status::Ok);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `locale` a NUL-terminated string.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_intl_set_locale(
    context: *const TsContext,
    locale: *const c_char,
) -> Status {
    with_context(context, |ctx| {
        // SAFETY: the entry point's contract.
        let tag = unsafe { text(locale, "locale") }?;
        let mut intl = ctx.intl_mut();
        intl.set_locale(tag)
            .map(drop)
            .map_err(|e| unknown_locale(&intl, tag, &e.to_string()))
    })
}

/// The loaded locale a tag means, by RFC 4647 lookup, lent until the next
/// call on the context: `ne` is `ne-Deva-NP`. Nothing changes, and
/// `ts_intl_set_locale` selects the same one.
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::Test)?;
/// # fn lent(out: teistro_ffi::string::TsStr) -> String {
/// #     // SAFETY: the library lent `len` bytes of UTF-8 at `data`.
/// #     String::from_utf8_lossy(unsafe { core::slice::from_raw_parts(out.data.cast::<u8>(), out.len) }).into_owned()
/// # }
/// # let mut out = teistro_ffi::string::TsStr { data: core::ptr::null(), len: 0 };
/// use teistro_core::Status;
/// use teistro_ffi::intl::ts_intl_lookup_locale;
///
/// // SAFETY: a live context, a NUL-terminated tag and a writable slot.
/// assert_eq!(unsafe { ts_intl_lookup_locale(&raw const ctx, c"sa".as_ptr(), &raw mut out) }, Status::Ok);
/// assert_eq!(lent(out), "sa-Deva");
/// // SAFETY: as above.
/// assert_ne!(unsafe { ts_intl_lookup_locale(&raw const ctx, c"hi-Latn".as_ptr(), &raw mut out) }, Status::Ok);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// `UNSUPPORTED` naming the loaded locales, as `ts_intl_set_locale`.
///
/// # Safety
///
/// `context` must be a live handle; `locale` a NUL-terminated string;
/// `out_locale` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_intl_lookup_locale(
    context: *const TsContext,
    locale: *const c_char,
    out_locale: *mut TsStr,
) -> Status {
    with_context(context, |ctx| {
        // SAFETY: the entry point's contract.
        let tag = unsafe { text(locale, "locale") }?;
        let intl = ctx.intl();
        let chosen = intl
            .lookup_locale(tag)
            .map_err(|e| unknown_locale(&intl, tag, &e.to_string()))?;
        let lent = ctx.lend(chosen);
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_locale, "out_locale", lent) }
    })
}

/// The locale every render resolves from, lent until the next call on the
/// context.
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts { locale: Some("ne-Deva-NP"), ..OptionTexts::default() }, teistro::Ephemeris::Test)?;
/// # fn lent(out: teistro_ffi::string::TsStr) -> String {
/// #     // SAFETY: the library lent `len` bytes of UTF-8 at `data`.
/// #     String::from_utf8_lossy(unsafe { core::slice::from_raw_parts(out.data.cast::<u8>(), out.len) }).into_owned()
/// # }
/// # let mut out = teistro_ffi::string::TsStr { data: core::ptr::null(), len: 0 };
/// use teistro_core::Status;
/// use teistro_ffi::intl::ts_intl_locale;
///
/// // SAFETY: a live context and a writable slot.
/// assert_eq!(unsafe { ts_intl_locale(&raw const ctx, &raw mut out) }, Status::Ok);
/// assert_eq!(lent(out), "ne-Deva-NP");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `out_locale` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_intl_locale(
    context: *const TsContext,
    out_locale: *mut TsStr,
) -> Status {
    with_context(context, |ctx| {
        let lent = ctx.lend(ctx.intl().locale());
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_locale, "out_locale", lent) }
    })
}

/// Whether the current locale or its fallbacks have a message: `1` or `0`.
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::Test)?;
/// use teistro_core::Status;
/// use teistro_ffi::intl::ts_intl_has;
///
/// let has = |key: &core::ffi::CStr| -> Result<u8, Status> {
///     let mut has = 9_u8;
///     // SAFETY: a live context, a NUL-terminated key and a writable slot.
///     match unsafe { ts_intl_has(&raw const ctx, key.as_ptr(), &raw mut has) } {
///         Status::Ok => Ok(has),
///         refused => Err(refused),
///     }
/// };
/// assert_eq!(has(c"sdk.reason.grahaInBhava"), Ok(1));
/// assert_eq!(has(c"sdk.nope.missing"), Ok(0));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `key` a NUL-terminated string;
/// `out_has` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_intl_has(
    context: *const TsContext,
    key: *const c_char,
    out_has: *mut u8,
) -> Status {
    with_context(context, |ctx| {
        // SAFETY: the entry point's contract.
        let key = unsafe { text(key, "key") }?;
        let has = ctx.intl().has(key);
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_has, "out_has", u8::from(has)) }
    })
}

/// A number or an amount of money in words and in figures, by CLDR's
/// rule-based number formats (`03-design/amounts-in-words.md`), answered
/// as canonical JSON: `{text, figure, rules, locale, amount}`, `amount`
/// for an amount the major and minor counts, each in words with its unit
/// word, and whether a decimal was rounded.
///
/// `request_json` is `{"number"}` (an integer, a negative spelt with the
/// language's minus) or `{"amount": {"currency", "value", "minorDigits",
/// "rounding", "only"}}`, never both, with `"locale"` (the context's when
/// left out), `"grouping"` (`INDIAN`, `INTERNATIONAL`), `"digits"`
/// (`LATN`, `DEVA`) and `"style"` (`PLAIN`, `VERBOSE`). An amount's
/// value is `{"minor": units}` or `{"decimal": "1234.56"}`, never a float.
/// A key it does not read, both or neither of `number` and `amount`, or
/// a malformed decimal or code is `INVALID_ARG`, named under `inWords`;
/// Sanskrit (not yet reviewed), a grouping or style the language's rules
/// lack, or a currency the locale names no units for is `UNSUPPORTED`.
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
/// use teistro_ffi::intl::ts_intl_in_words;
/// use teistro_ffi::string::TsString;
///
/// let request = c"{\"number\": 1500000, \"locale\": \"ne-Deva-NP\"}";
/// let mut out = TsString::empty();
/// // SAFETY: a live context, a NUL-terminated request and a writable slot.
/// let status = unsafe { ts_intl_in_words(&raw const ctx, request.as_ptr(), &raw mut out) };
/// assert_eq!(status, Status::Ok);
/// let answer: serde_json::Value = serde_json::from_str(&owned(out))?;
/// assert_eq!(answer["text"], "पन्ध्र लाख");
/// assert_eq!(answer["figure"], "१५,००,०००");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `request_json` NUL-terminated;
/// `out_json` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_intl_in_words(
    context: *const TsContext,
    request_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        // SAFETY: the entry point's contract.
        let asked =
            teistro::InWordsRequest::from_json(unsafe { text(request_json, "request_json") }?)?;
        let answer = ctx.intl().in_words(&asked)?;
        let json = TsString::from_string(teistro_core::envelope::canonical_json(&answer));
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_json, "out_json", json) }
    })
}

/// A time of day as text and parts, through the locale's pattern and its
/// parts of the day (`03-design/intl-engine-and-packs.md`), answered as
/// canonical JSON: `{text, parts, locale, resolvedFrom, isFallback,
/// isDraft, warnings}`, each part `{kind, text}` and the parts joined being
/// `text`.
///
/// `request_json` is `{"at": {"hour", "minute", "second"}}`, with `"style"`
/// (`NUMERIC`, `LONG`), `"hour12"`, `"locale"` (the context's when left
/// out) and `"digits"` (`LATN`, `DEVA`). A key it does not read or a part
/// of the time out of range is `INVALID_ARG`, named under `time`; a
/// locale not loaded is `UNSUPPORTED`.
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::None)?;
/// use teistro_core::Status;
/// use teistro_ffi::intl::ts_intl_time;
/// use teistro_ffi::string::{TsString, ts_string_free};
///
/// let request = c"{\"at\": {\"hour\": 6, \"minute\": 5}, \"hour12\": true, \"locale\": \"ne-Deva-NP\"}";
/// let mut out = TsString::empty();
/// // SAFETY: a live context, a NUL-terminated request and a writable slot.
/// let status = unsafe { ts_intl_time(&raw const ctx, request.as_ptr(), &raw mut out) };
/// assert_eq!(status, Status::Ok);
/// // SAFETY: the library wrote `len` bytes of UTF-8 at `data`.
/// let bytes = unsafe { core::slice::from_raw_parts(out.data, out.len) };
/// let answer: serde_json::Value = serde_json::from_slice(bytes)?;
/// assert_eq!(answer["text"], "बिहान ६:०५");
/// // SAFETY: a descriptor the library wrote, freed once.
/// unsafe { ts_string_free(&raw mut out) };
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `request_json` NUL-terminated;
/// `out_json` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_intl_time(
    context: *const TsContext,
    request_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        // SAFETY: the entry point's contract.
        let asked =
            teistro::TimeTextRequest::from_json(unsafe { text(request_json, "request_json") }?)?;
        let answer = ctx.intl().time(&asked)?;
        let json = TsString::from_string(teistro_core::envelope::canonical_json(&answer));
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_json, "out_json", json) }
    })
}

/// A date as text and parts, in its own calendar, through the locale's
/// pattern (`03-design/intl-engine-and-packs.md`), answered as `ts_intl_time`
/// answers.
///
/// `request_json` is `{"at": {"calendar", "year", "month", "day", "era",
/// "leap"}}`, with `"style"` (`NUMERIC`, `LONG`, `FULL`), `"locale"` (the
/// context's when left out) and `"digits"` (`LATN`, `DEVA`). A key it does
/// not read or a date its calendar does not hold is `INVALID_ARG`, named
/// under `date`; a locale not loaded, a calendar not shipped or one with no
/// pattern is `UNSUPPORTED`.
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::None)?;
/// use teistro_core::Status;
/// use teistro_ffi::intl::ts_intl_date;
/// use teistro_ffi::string::{TsString, ts_string_free};
///
/// let request = c"{\"at\": {\"calendar\": \"calendar.GREGORIAN\", \"year\": 2026, \"month\": 10, \"day\": 31}, \"style\": \"LONG\"}";
/// let mut out = TsString::empty();
/// // SAFETY: a live context, a NUL-terminated request and a writable slot.
/// let status = unsafe { ts_intl_date(&raw const ctx, request.as_ptr(), &raw mut out) };
/// assert_eq!(status, Status::Ok);
/// // SAFETY: the library wrote `len` bytes of UTF-8 at `data`.
/// let bytes = unsafe { core::slice::from_raw_parts(out.data, out.len) };
/// let answer: serde_json::Value = serde_json::from_slice(bytes)?;
/// assert_eq!(answer["text"], "31 October 2026");
/// // SAFETY: a descriptor the library wrote, freed once.
/// unsafe { ts_string_free(&raw mut out) };
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `request_json` NUL-terminated;
/// `out_json` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_intl_date(
    context: *const TsContext,
    request_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        // SAFETY: the entry point's contract.
        let asked =
            teistro::DateTextRequest::from_json(unsafe { text(request_json, "request_json") }?)?;
        let answer = ctx.intl().date(&asked)?;
        let json = TsString::from_string(teistro_core::envelope::canonical_json(&answer));
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_json, "out_json", json) }
    })
}

/// Every catalogue member's names in a locale, romanised forms too
/// (`03-design/lexicon.md` §1), answered as canonical JSON: `{locale,
/// entries}`, each entry `{key, forms}` and each form `{form, text,
/// locale}`, the form `name`, `prose`, `short`, `iast`, any the locale adds,
/// or `romanised`, and `locale` the one in the chain that gave it.
///
/// `request_json` is `{}`, with `"locale"` (a loaded tag or one that means
/// one, `ne`; the context's when left out) and `"kinds"` (`["tithi",
/// "nakshatra"]`; every kind when left out). A key it does not read or a
/// kind no member's name has is `INVALID_ARG`, named under `lexicon`; a
/// locale no loaded one answers is `UNSUPPORTED`. The lexicon is built the
/// first time a locale is asked for and kept on the context.
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::None)?;
/// use teistro_core::Status;
/// use teistro_ffi::intl::ts_intl_lexicon;
/// use teistro_ffi::string::{TsString, ts_string_free};
///
/// let request = c"{\"locale\": \"ne\", \"kinds\": [\"masa\"]}";
/// let mut out = TsString::empty();
/// // SAFETY: a live context, a NUL-terminated request and a writable slot.
/// let status = unsafe { ts_intl_lexicon(&raw const ctx, request.as_ptr(), &raw mut out) };
/// assert_eq!(status, Status::Ok);
/// // SAFETY: the library wrote `len` bytes of UTF-8 at `data`.
/// let bytes = unsafe { core::slice::from_raw_parts(out.data, out.len) };
/// let answer: serde_json::Value = serde_json::from_slice(bytes)?;
/// assert_eq!(answer["locale"], "ne-Deva-NP");
/// assert_eq!(answer["entries"][0]["key"], "masa.CHAITRA");
/// // SAFETY: a descriptor the library wrote, freed once.
/// unsafe { ts_string_free(&raw mut out) };
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `request_json` NUL-terminated;
/// `out_json` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_intl_lexicon(
    context: *const TsContext,
    request_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        // SAFETY: the entry point's contract.
        let asked =
            teistro::LexiconRequest::from_json(unsafe { text(request_json, "request_json") }?)?;
        let answer = ctx.intl().lexicon(&asked)?;
        let json = TsString::from_string(teistro_core::envelope::canonical_json(&answer));
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_json, "out_json", json) }
    })
}

/// The catalogue members a text may name (`03-design/lexicon.md` §1.3),
/// answered as canonical JSON: `{matches}`, each match `{start, end, text,
/// candidates}` in the text's order, `start` and `end` counted in Unicode
/// scalar values, and each candidate `{key, via}`, `via` the forms whose
/// fold the span's is. A span that may name several members answers every
/// one of them.
///
/// `request_json` is `{"text"}`, with `"locale"` (whose chain's names are
/// searched; every loaded locale's when left out) and `"kinds"` (every kind
/// when left out). A key it does not read or a kind no member's name has is
/// `INVALID_ARG`, named under `lookup`; a locale no loaded one answers is
/// `UNSUPPORTED`.
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::None)?;
/// use teistro_core::Status;
/// use teistro_ffi::intl::ts_intl_lookup;
/// use teistro_ffi::string::{TsString, ts_string_free};
///
/// let request = c"{\"text\": \"आज एकादशी हो\", \"kinds\": [\"tithi\"]}";
/// let mut out = TsString::empty();
/// // SAFETY: a live context, a NUL-terminated request and a writable slot.
/// let status = unsafe { ts_intl_lookup(&raw const ctx, request.as_ptr(), &raw mut out) };
/// assert_eq!(status, Status::Ok);
/// // SAFETY: the library wrote `len` bytes of UTF-8 at `data`.
/// let bytes = unsafe { core::slice::from_raw_parts(out.data, out.len) };
/// let answer: serde_json::Value = serde_json::from_slice(bytes)?;
/// assert_eq!(answer["matches"][0]["text"], "एकादशी");
/// assert_eq!(answer["matches"][0]["candidates"].as_array().map(Vec::len), Some(2));
/// // SAFETY: a descriptor the library wrote, freed once.
/// unsafe { ts_string_free(&raw mut out) };
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `request_json` NUL-terminated;
/// `out_json` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_intl_lookup(
    context: *const TsContext,
    request_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        // SAFETY: the entry point's contract.
        let asked =
            teistro::LookupRequest::from_json(unsafe { text(request_json, "request_json") }?)?;
        let answer = ctx.intl().lookup(&asked)?;
        let json = TsString::from_string(teistro_core::envelope::canonical_json(&answer));
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_json, "out_json", json) }
    })
}

/// Every Bikram Sambat and Gregorian date a text names
/// (`03-design/lexicon.md` §2), answered as canonical JSON: `{dates}`, each
/// found date `{start, end, text, readings}` in the text's order, `start`
/// and `end` counted in Unicode scalar values, and each reading `{date,
/// order, gregorian}`: the date in its calendar (`{calendar, year, month,
/// day, era, leap}`), how its figures were ordered (`YMD`, `DMY` or `MDY`),
/// and the Gregorian day it is, on a Bikram Sambat reading. A span that may
/// be read several ways answers every reading.
///
/// `request_json` is `{"text"}`, with `"calendars"` (`["calendar.BIKRAM_SAMBAT",
/// "calendar.GREGORIAN"]` when left out). A key it does not read is `INVALID_ARG`,
/// named under `dates`; a calendar other than those two is `UNSUPPORTED`,
/// named `dates.calendars`.
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::None)?;
/// use teistro_core::Status;
/// use teistro_ffi::intl::ts_intl_dates;
/// use teistro_ffi::string::{TsString, ts_string_free};
///
/// let request = c"{\"text\": \"२०८२ कात्तिक १५ गते\"}";
/// let mut out = TsString::empty();
/// // SAFETY: a live context, a NUL-terminated request and a writable slot.
/// let status = unsafe { ts_intl_dates(&raw const ctx, request.as_ptr(), &raw mut out) };
/// assert_eq!(status, Status::Ok);
/// // SAFETY: the library wrote `len` bytes of UTF-8 at `data`.
/// let bytes = unsafe { core::slice::from_raw_parts(out.data, out.len) };
/// let answer: serde_json::Value = serde_json::from_slice(bytes)?;
/// let reading = &answer["dates"][0]["readings"][0];
/// assert_eq!(reading["date"]["calendar"], "calendar.BIKRAM_SAMBAT");
/// assert_eq!(reading["gregorian"]["day"], 1);
/// // SAFETY: a descriptor the library wrote, freed once.
/// unsafe { ts_string_free(&raw mut out) };
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `request_json` NUL-terminated;
/// `out_json` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_intl_dates(
    context: *const TsContext,
    request_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        // SAFETY: the entry point's contract.
        let asked =
            teistro::DatesRequest::from_json(unsafe { text(request_json, "request_json") }?)?;
        let answer = ctx.intl().dates(&asked)?.in_full()?;
        let json = TsString::from_string(teistro_core::envelope::canonical_json(&answer));
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_json, "out_json", json) }
    })
}

/// Text from one script into another (`deva`, `iast`), as a string lent
/// until the next call on the context. A pair the build has no table for
/// is `UNSUPPORTED` naming both scripts; anything the table does not know
/// passes through, so a name written in two scripts is transliterated in
/// the half that needs it.
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::Test)?;
/// # fn lent(out: teistro_ffi::string::TsStr) -> String {
/// #     // SAFETY: the library lent `len` bytes of UTF-8 at `data`.
/// #     String::from_utf8_lossy(unsafe { core::slice::from_raw_parts(out.data.cast::<u8>(), out.len) }).into_owned()
/// # }
/// # let mut out = teistro_ffi::string::TsStr { data: core::ptr::null(), len: 0 };
/// use teistro_core::Status;
/// use teistro_ffi::intl::ts_intl_transliterate;
///
/// // Devanagari into IAST.
/// // SAFETY: a live context, NUL-terminated strings and a writable slot.
/// let status = unsafe { ts_intl_transliterate(&raw const ctx, c"सूर्य".as_ptr(), c"deva".as_ptr(), c"iast".as_ptr(), &raw mut out) };
/// assert_eq!(status, Status::Ok);
/// assert_eq!(lent(out), "sūrya");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `text`, `from` and `to`
/// NUL-terminated strings; `out_text` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_intl_transliterate(
    context: *const TsContext,
    text: *const c_char,
    from: *const c_char,
    to: *const c_char,
    out_text: *mut TsStr,
) -> Status {
    with_context(context, |ctx| {
        // SAFETY: the entry point's contract.
        let (source, from_key, to_key) = unsafe {
            (
                crate::support::text(text, "text")?,
                crate::support::text(from, "from")?,
                crate::support::text(to, "to")?,
            )
        };
        let script = |key: &str, field: &'static str| {
            Script::from_key(key).ok_or_else(|| {
                Error::unsupported(format!("no script `{key}`"))
                    .with_field(field)
                    .with_hint(String::from("the scripts are `deva` and `iast`"))
            })
        };
        let (from, to) = (script(from_key, "from")?, script(to_key, "to")?);
        let written = transliterate(source, from, to)
            .map_err(|e| Error::unsupported(e.to_string()).with_field("to"))?;
        let lent = ctx.lend(&written);
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_text, "out_text", lent) }
    })
}

/// An entity's forms in the current locale or its fallbacks, as a JSON
/// object lent until the next call on the context: every form the locale
/// gives (`name`, `prose`, `iast`, `short`, and any it adds), the
/// `glyph` when it has one, and the `gender` when the locale marks one.
/// A key the locale chain does not carry is `UNSUPPORTED`, naming the
/// locale that was asked.
///
/// The typed accessors each binding generates read entities through this,
/// so an application spells `graha.SUN` once and never a name.
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::Test)?;
/// # fn lent(out: teistro_ffi::string::TsStr) -> String {
/// #     // SAFETY: the library lent `len` bytes of UTF-8 at `data`.
/// #     String::from_utf8_lossy(unsafe { core::slice::from_raw_parts(out.data.cast::<u8>(), out.len) }).into_owned()
/// # }
/// # let mut out = teistro_ffi::string::TsStr { data: core::ptr::null(), len: 0 };
/// use teistro_core::Status;
/// use teistro_ffi::intl::ts_intl_entity;
///
/// // The Sun's record in the context's locale, as JSON.
/// // SAFETY: a live context, a NUL-terminated key and a writable slot.
/// assert_eq!(unsafe { ts_intl_entity(&raw const ctx, c"graha.SUN".as_ptr(), &raw mut out) }, Status::Ok);
/// let record: serde_json::Value = serde_json::from_str(&lent(out))?;
/// assert!(record.is_object());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `key` a NUL-terminated string;
/// `out_json` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_intl_entity(
    context: *const TsContext,
    key: *const c_char,
    out_json: *mut TsStr,
) -> Status {
    with_context(context, |ctx| {
        // SAFETY: the entry point's contract.
        let key = unsafe { text(key, "key") }?;
        let intl = ctx.intl();
        let locale = intl.locale();
        let entity = intl.entity_from(locale, key).ok_or_else(|| {
            Error::unsupported(format!("no entity `{key}` in `{locale}`"))
                .with_field("key")
                .with_hint(format!(
                    "the catalogue's key, as `graha.SUN`; `{locale}` and its fallbacks carry none"
                ))
        })?;
        let json = entity_json(entity);
        let lent = ctx.lend(&json);
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_json, "out_json", lent) }
    })
}

/// An entity's forms as the JSON object the bindings read: every form,
/// then `glyph` and `gender` when the locale gives them. The keys are
/// sorted, because the map they come from is.
fn entity_json(entity: &teistro_intl::source::Entity) -> String {
    let mut fields: Vec<(&str, &str)> = entity
        .forms
        .iter()
        .map(|(form, text)| (form.as_str(), text.as_str()))
        .collect();
    if let Some(glyph) = entity.glyph.as_deref() {
        fields.push(("glyph", glyph));
    }
    if let Some(gender) = entity.gender.as_deref() {
        fields.push(("gender", gender));
    }
    let body: Vec<String> = fields
        .iter()
        .map(|(name, value)| format!("{}:{}", json_string(name), json_string(value)))
        .collect();
    format!("{{{}}}", body.join(","))
}

/// A JSON string, escaped as the grammar requires.
fn json_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Renders a message with parameters given as a JSON object: a string, an
/// integer, a number or an array is itself; an entity is
/// `{"$entity": "graha.SUN"}`; a date `{"$date": {"calendar": "GREGORIAN",
/// "year": 2026, "month": 9, "day": 6}}`; a time `{"$time": {"hour": 6,
/// "minute": 15, "second": 0}}`; a date and time `{"$datetime": {"date":
/// {...}, "time": {...}}}`; a ghati count `{"$ghati": {"ghati": 12,
/// "pala": 30, "vipala": 0}}`. A null `params_json` renders with none.
/// The result blob carries the text, its parts, where it resolved from
/// and the warnings; a missing message renders as its key with a
/// warning, never an error.
///
/// `api: blob=intl_render`
/// `api: params_json: nullable`
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts { locale: Some("ne-Deva-NP"), ..OptionTexts::default() }, teistro::Ephemeris::Test)?;
/// # fn bytes(mut blob: teistro_ffi::blob::TsBlob) -> Vec<u8> {
/// #     // SAFETY: the library wrote `len` bytes at `data`.
/// #     let bytes = unsafe { core::slice::from_raw_parts(blob.data, blob.len) }.to_vec();
/// #     // SAFETY: a descriptor the library wrote, freed once.
/// #     unsafe { teistro_ffi::blob::ts_blob_free(&raw mut blob) };
/// #     bytes
/// # }
/// use teistro_core::Status;
/// use teistro_ffi::blob::TsBlob;
/// use teistro_ffi::intl::ts_intl_render;
/// use teistro_idl::blob::Reader;
///
/// let params = c"{\"graha\": {\"$entity\": \"graha.JUPITER\"}, \"bhava\": 7}";
/// let mut blob = TsBlob::empty();
/// // SAFETY: a live context, NUL-terminated strings and a writable slot.
/// let status = unsafe { ts_intl_render(&raw const ctx, c"sdk.reason.grahaInBhava".as_ptr(), params.as_ptr(), &raw mut blob) };
/// assert_eq!(status, Status::Ok);
/// let bytes = bytes(blob);
/// let schema = teistro_ffi::schemas::intl_render();
/// let reader = Reader::parse(&bytes, &schema)?;
/// // In Nepali, its digits Devanagari.
/// assert_eq!(reader.text("resolved_from")?, "ne-Deva-NP");
/// assert!(reader.text("text")?.contains('७'));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `key` a NUL-terminated string;
/// `params_json` null or one; `out_blob` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_intl_render(
    context: *const TsContext,
    key: *const c_char,
    params_json: *const c_char,
    out_blob: *mut TsBlob,
) -> Status {
    with_context(context, |ctx| {
        // SAFETY: the entry point's contract.
        let (key, params) = unsafe {
            (
                text(key, "key")?,
                optional_text(params_json, "params_json")?,
            )
        };
        let params = params
            .map(params_from_json)
            .transpose()?
            .unwrap_or_default();
        let rendered = ctx.intl().render(key, &params);
        let schema = schemas::intl_render();
        let mut writer = Writer::new(&schema);
        let warning_count = u32::try_from(rendered.warnings.len()).unwrap_or(u32::MAX);
        let encoded = (|| -> Result<Vec<u8>, teistro_idl::blob::BlobError> {
            writer.fixed(
                "flags",
                &[
                    FixedValue::from(u8::from(rendered.is_fallback)),
                    FixedValue::from(u8::from(rendered.is_override)),
                    FixedValue::from(warning_count),
                ],
            )?;
            writer.bytes("text", rendered.text.as_bytes())?;
            writer.bytes(
                "resolved_from",
                rendered.resolved_from.as_deref().unwrap_or("").as_bytes(),
            )?;
            let warnings =
                serde_json::to_string(&rendered.warnings).unwrap_or_else(|_| String::from("[]"));
            writer.bytes("warnings", warnings.as_bytes())?;
            writer.bytes(
                "parts",
                teistro_intl::wire::parts_json(&rendered).as_bytes(),
            )?;
            writer.fixed("review", &[FixedValue::from(u8::from(rendered.is_draft))])?;
            writer.finish()
        })()
        .map_err(|e| Error::internal(format!("the render blob did not encode: {e}")))?;
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_blob, "out_blob", TsBlob::from_vec(encoded)) }
    })
}
/// The parameters of a render, as the JSON `ts_intl_render` documents.
///
/// One parser, and it is the locale engine's own
/// ([`teistro_intl::wire`]): a consumer writing parameters and a composer
/// writing a narrative plan therefore cannot be told apart by the reader,
/// and a shape added there is taken here without a second reading of it.
///
/// ```
/// use teistro_core::Status;
/// use teistro_ffi::intl::params_from_json;
///
/// assert!(params_from_json(r#"{"count": 3, "name": "Ram"}"#).is_ok());
/// // Anything but an object of values is refused, naming the field.
/// let refused = params_from_json("[1, 2]").unwrap_err();
/// assert_eq!((refused.status, refused.field()), (Status::InvalidArg, Some("params_json")));
/// ```
///
/// # Errors
///
/// JSON that is not an object of values, naming the parameter it stopped
/// at.
pub fn params_from_json(text: &str) -> Result<Params, Error> {
    serde_json::from_str::<Params>(text).map_err(|err| {
        Error::invalid_arg(format!("`params_json` does not read: {err}")).with_field("params_json")
    })
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;
    use teistro_core::catalogue::Calendar;
    use teistro_intl::{ClockTime, Ghati, Value};

    #[test]
    fn parameters_come_from_json_with_their_types() {
        let params = params_from_json(
            r#"{"n": 7, "x": 1.5, "s": "text", "l": [1, "a"], "g": {"$entity": "graha.SUN"}, "d": {"$date": {"calendar": "GREGORIAN", "year": 2026, "month": 9, "day": 6}}, "t": {"$time": {"hour": 6, "minute": 15, "second": 0}}, "dt": {"$datetime": {"date": {"calendar": "BIKRAM_SAMBAT", "year": 2083, "month": 5, "day": 21}, "time": {"hour": 1, "minute": 2, "second": 3}}}, "gh": {"$ghati": {"ghati": 12, "pala": 30, "vipala": 0}}}"#,
        )
        .unwrap();
        assert_eq!(params["n"], Value::Int(7));
        assert_eq!(params["x"], Value::Num(1.5));
        assert_eq!(params["s"], Value::Str("text".into()));
        assert_eq!(
            params["l"],
            Value::List(vec![Value::Int(1), Value::Str("a".into())])
        );
        assert_eq!(params["g"], Value::entity("graha.SUN"));
        assert!(
            matches!(&params["d"], Value::Date(d) if d.year == 2026 && d.calendar == Calendar::Gregorian)
        );
        assert_eq!(params["t"], Value::Time(ClockTime::new(6, 15, 0)));
        assert!(
            matches!(&params["dt"], Value::DateTime(d, t) if d.calendar == Calendar::BikramSambat && *t == ClockTime::new(1, 2, 3))
        );
        assert_eq!(params["gh"], Value::Ghati(Ghati::new(12, 30, 0)));
        for bad in [
            "[]",
            r#"{"b": true}"#,
            r#"{"o": {"a": 1, "b": 2}}"#,
            r#"{"o": {"$nope": 1}}"#,
            r#"{"d": {"$date": {"calendar": "MAYAN", "year": 1, "month": 1, "day": 1}}}"#,
            r#"{"t": {"$time": {"hour": 300}}}"#,
            "not json",
        ] {
            let error = params_from_json(bad).unwrap_err();
            assert_eq!(error.status, Status::InvalidArg, "{bad}");
        }
    }
}
