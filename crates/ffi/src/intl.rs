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

use teistro_calendar::CalendarDate;
use teistro_core::Status;
use teistro_core::catalogue::Calendar;
use teistro_core::error::Error;
use teistro_idl::blob::{FixedValue, Writer};
use teistro_intl::{ClockTime, Ghati, Params, Value};

use crate::blob::TsBlob;
use crate::context::{TsContext, unknown_locale};
use crate::schemas;
use crate::strings::TsStr;
use crate::support::{bytes, c_struct, optional_text, text, with_context, write_out, write_plain};

/// What a loaded pack or bundle carried.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TsIntlLoaded {
    /// `sizeof(ts_intl_loaded)` as the caller compiled it.
    pub struct_size: u32,
    /// The entries the file carried.
    pub entries: u32,
    /// The entries that replaced ones already loaded.
    pub replaced: u32,
    /// Reserved, zero.
    pub reserved: u32,
    /// The locale; lent until the next call on the context.
    pub locale: *const c_char,
    /// The file's SHA-256 as sixty-four hex digits; lent until the next
    /// call on the context.
    pub sha256: *const c_char,
}

c_struct!(TsIntlLoaded);

/// Loads a `.tpack` or `.tbundle` file: a locale it brings is added, a
/// namespace it brings replaces what was loaded under the same keys. A
/// file that does not verify is `PACK`.
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
            reserved: 0,
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

/// Parameters from their JSON form.
///
/// # Errors
///
/// Text that is not a JSON object, or a value no parameter type takes.
pub fn params_from_json(text: &str) -> Result<Params, Error> {
    let value: serde_json::Value = serde_json::from_str(text).map_err(|e| {
        Error::invalid_arg(format!("`params_json` does not parse: {e}")).with_field("params_json")
    })?;
    let serde_json::Value::Object(map) = value else {
        return Err(
            Error::invalid_arg("`params_json` must be a JSON object").with_field("params_json")
        );
    };
    map.into_iter()
        .map(|(name, v)| value_from_json(&name, &v).map(|value| (name, value)))
        .collect()
}

fn refuse(name: &str, what: &str) -> Error {
    Error::invalid_arg(format!("parameter `{name}`: {what}")).with_field(name)
}

fn value_from_json(name: &str, value: &serde_json::Value) -> Result<Value, Error> {
    use serde_json::Value as Json;
    match value {
        Json::String(s) => Ok(Value::Str(s.clone())),
        Json::Number(n) => n
            .as_i64()
            .map(Value::Int)
            .or_else(|| n.as_f64().map(Value::Num))
            .ok_or_else(|| refuse(name, "a number outside the range of an integer or a double")),
        Json::Array(items) => items
            .iter()
            .map(|item| value_from_json(name, item))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::List),
        Json::Object(map) => {
            let mut entries = map.iter();
            let (Some((tag, inner)), None) = (entries.next(), entries.next()) else {
                return Err(refuse(
                    name,
                    "an object must have exactly one `$`-tagged key",
                ));
            };
            match tag.as_str() {
                "$entity" => inner.as_str().map(Value::entity).ok_or_else(|| {
                    refuse(name, "`$entity` takes a catalogue key such as `graha.SUN`")
                }),
                "$date" => date_from_json(name, inner).map(Value::Date),
                "$time" => time_from_json(name, inner).map(Value::Time),
                "$datetime" => Ok(Value::DateTime(
                    date_from_json(name, inner.get("date").unwrap_or(&Json::Null))?,
                    time_from_json(name, inner.get("time").unwrap_or(&Json::Null))?,
                )),
                "$ghati" => {
                    let field = |key: &str| small(name, inner, key);
                    Ok(Value::Ghati(Ghati::new(
                        field("ghati")?,
                        field("pala")?,
                        field("vipala")?,
                    )))
                }
                other => Err(refuse(
                    name,
                    &format!(
                        "`{other}` is not a tag; the tags are $entity, $date, $time, $datetime, $ghati"
                    ),
                )),
            }
        }
        Json::Bool(_) | Json::Null => {
            Err(refuse(name, "booleans and null are not parameter values"))
        }
    }
}

fn small(name: &str, object: &serde_json::Value, key: &str) -> Result<u8, Error> {
    object
        .get(key)
        .and_then(serde_json::Value::as_u64)
        .and_then(|v| u8::try_from(v).ok())
        .ok_or_else(|| refuse(name, &format!("`{key}` must be an integer from 0 to 255")))
}

fn date_from_json(name: &str, object: &serde_json::Value) -> Result<CalendarDate, Error> {
    let calendar = object
        .get("calendar")
        .and_then(serde_json::Value::as_str)
        .and_then(Calendar::from_key)
        .ok_or_else(|| {
            refuse(
                name,
                "`calendar` must be a calendar key such as `GREGORIAN`",
            )
        })?;
    let year = object
        .get("year")
        .and_then(serde_json::Value::as_i64)
        .and_then(|v| i32::try_from(v).ok())
        .ok_or_else(|| refuse(name, "`year` must be an integer"))?;
    Ok(CalendarDate::defined(
        calendar,
        year,
        small(name, object, "month")?,
        small(name, object, "day")?,
    ))
}

fn time_from_json(name: &str, object: &serde_json::Value) -> Result<ClockTime, Error> {
    Ok(ClockTime::new(
        small(name, object, "hour")?,
        small(name, object, "minute")?,
        small(name, object, "second")?,
    ))
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;

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
