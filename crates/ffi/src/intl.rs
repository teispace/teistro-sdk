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
use crate::string::TsStr;
use crate::support::{bytes, c_struct, optional_text, text, with_context, write_out, write_plain};

/// What a loaded pack or bundle carried.
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
/// # Safety
///
/// `context` must be a live handle; `bytes` valid for `bytes_len` reads;
/// `out_loaded` valid for a read of its `struct_size` and a write.
#[unsafe(no_mangle)]
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
/// # Safety
///
/// `context` must be a live handle; `locale` a NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_intl_set_locale(
    context: *const TsContext,
    locale: *const c_char,
) -> Status {
    with_context(context, |ctx| {
        // SAFETY: the entry point's contract.
        let tag = unsafe { text(locale, "locale") }?;
        let mut intl = ctx.intl_mut();
        intl.set_locale(tag)
            .map_err(|e| unknown_locale(&intl, tag, &e.to_string()))
    })
}

/// The locale every render resolves from, lent until the next call on the
/// context.
///
/// # Safety
///
/// `context` must be a live handle; `out_locale` valid for a write.
#[unsafe(no_mangle)]
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
/// # Safety
///
/// `context` must be a live handle; `key` a NUL-terminated string;
/// `out_has` valid for a write.
#[unsafe(no_mangle)]
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

/// Text from one script into another (`deva`, `iast`), as a string lent
/// until the next call on the context. A pair the build has no table for
/// is `UNSUPPORTED` naming both scripts; anything the table does not know
/// passes through, so a name written in two scripts is transliterated in
/// the half that needs it.
///
/// # Safety
///
/// `context` must be a live handle; `text`, `from` and `to`
/// NUL-terminated strings; `out_text` valid for a write.
#[unsafe(no_mangle)]
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
/// # Safety
///
/// `context` must be a live handle; `key` a NUL-terminated string;
/// `out_json` valid for a write.
#[unsafe(no_mangle)]
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
/// The result blob carries the text, where it resolved from and the
/// warnings; a missing message renders as its key with a warning, never
/// an error.
///
/// `api: blob=intl_render`
/// `api: params_json: nullable`
///
/// # Safety
///
/// `context` must be a live handle; `key` a NUL-terminated string;
/// `params_json` null or one; `out_blob` valid for a write.
#[unsafe(no_mangle)]
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
