//! The module families at the boundary (`03-design/wasm-profiles.md`).
//!
//! A family is a feature of the façade, forwarded here. Every entry point
//! exists in every build, so the generated glue is one file for every
//! profile; a call into a family the build left out answers
//! `CAPABILITY`, naming the family, rather than a missing symbol.

use core::ffi::c_char;

use teistro_core::error::{Error, Status};

pub(crate) mod kp;
pub(crate) mod muhurta;
pub(crate) mod prashna;
pub(crate) mod remedies;
pub(crate) mod svg;
pub(crate) mod tajika;
pub(crate) mod western;

/// A left-out family's record and answer types: nothing can be one, so a
/// request holding one is always `None` and the code reading its answers
/// type-checks without the family's crate.
#[derive(Debug)]
#[allow(dead_code, reason = "a build with every family names none")]
#[allow(
    unreachable_pub,
    reason = "public because `chart::Composed`, a public struct, names it in a build without Tajika"
)]
pub enum Absent {}

/// The refusal a call into `family` answers in a build without it.
#[cold]
#[must_use]
#[allow(dead_code, reason = "a build with every family refuses none")]
pub(crate) fn left_out(family: &str) -> Error {
    Error::new(
        Status::Capability,
        format!("this build leaves out the `{family}` module family"),
    )
    .with_hint(format!(
        "use a build with `{family}`: the default `full` build has every family, and the wasm module's is `@teistro/sdk-wasm` rather than a profile's subpath"
    ))
}

/// The section a left-out family writes: nothing, since no request can
/// hold its record.
#[allow(dead_code, reason = "a build with every family has none left out")]
pub(crate) const fn unasked(asked: Option<&Absent>) -> String {
    match asked {
        None => String::new(),
        Some(never) => match *never {},
    }
}

/// A left-out family's record read from a request: none when the field is
/// null, and the family's refusal naming `record`, the field every binding
/// writes, when it was sent.
#[allow(dead_code, reason = "a build with every family refuses none")]
pub(crate) fn refused_if_sent(
    text: *const c_char,
    record: &str,
    family: &str,
) -> Result<Option<Absent>, Error> {
    if text.is_null() {
        Ok(None)
    } else {
        Err(left_out(family).with_field(record))
    }
}

/// Every chart's answer to a record, as the canonical JSON its section
/// carries: an array with one answer a chart, or nothing at all when the
/// record was not sent.
#[allow(dead_code, reason = "a build with no chart-record family answers none")]
pub(crate) fn each_json<A, T: serde::Serialize>(
    documents: &[teistro_serial::Document],
    asked: Option<&A>,
    answer: impl Fn(&teistro_serial::Document, &A) -> Result<T, Error>,
) -> Result<String, Error> {
    let Some(asked) = asked else {
        return Ok(String::new());
    };
    let answers = documents
        .iter()
        .map(|document| answer(document, asked))
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(teistro_core::envelope::canonical_json(&answers))
}

/// `$body` in a build with the feature `$family`, and the family's
/// refusal in one without it. The variables in brackets are the ones
/// `$body` reads, marked used where the body is compiled out.
macro_rules! in_family {
    ($family:literal, [$($used:ident),*], $body:block) => {{
        #[cfg(feature = $family)]
        let answer = $body;
        #[cfg(not(feature = $family))]
        let answer = {
            $(let _ = &$used;)*
            Err($crate::family::left_out($family))
        };
        answer
    }};
}

/// A family's answer type, `$alias`: the façade's `$ty`, or [`Absent`] in
/// a build without `$family`, where no request can ask for one.
macro_rules! answer {
    ($family:literal, $alias:ident, $ty:ty) => {
        #[doc = concat!("`", stringify!($ty), "`, which a build without `", $family, "` cannot hold.")]
        #[cfg(feature = $family)]
        pub(crate) type $alias = $ty;
        #[doc = concat!("`", stringify!($ty), "`, which a build without `", $family, "` cannot hold.")]
        #[cfg(not(feature = $family))]
        pub(crate) type $alias = $crate::family::Absent;
    };
}

/// A family's record in a chart request: the type `$alias`, and `$reader`
/// reading the C field `$field` with the façade's own reader, each with the
/// twin a build without `$family` compiles in its place, which refuses a
/// record that was sent, naming `$record` as every binding writes it.
macro_rules! record {
    ($family:literal, $field:literal, $record:literal, $alias:ident, $reader:ident, $request:ty) => {
        #[doc = concat!("The record `", $field, "` carries.")]
        #[cfg(feature = $family)]
        pub(crate) type $alias = $request;
        #[doc = concat!("The record `", $field, "` carries, which this build cannot hold.")]
        #[cfg(not(feature = $family))]
        pub(crate) type $alias = $crate::family::Absent;

        #[doc = concat!("The record a request's `", $field, "` sends, none for null, read by the façade's own reader, which names a refusal from the record's root.")]
        ///
        /// # Safety
        ///
        /// `text` null or a NUL-terminated string.
        #[cfg(feature = $family)]
        pub(crate) unsafe fn $reader(
            text: *const core::ffi::c_char,
        ) -> Result<Option<$alias>, teistro_core::error::Error> {
            // SAFETY: the caller's contract.
            unsafe { $crate::support::optional_text(text, $field) }?
                .map(<$request>::from_json)
                .transpose()
        }

        #[doc = concat!("A `", $field, "` record, refused in a build without `", $family, "`.")]
        ///
        /// # Safety
        ///
        /// None needed: the pointer is only compared with null.
        #[cfg(not(feature = $family))]
        pub(crate) unsafe fn $reader(
            text: *const core::ffi::c_char,
        ) -> Result<Option<$alias>, teistro_core::error::Error> {
            $crate::family::refused_if_sent(text, $record, $family)
        }
    };
}

/// A family a chart request asks for by one JSON record and the façade
/// answers a chart at a time: the [`record!`] items, and `json` writing
/// the section, with its twin.
macro_rules! chart_record {
    ($family:literal, $field:literal, $record:literal, $request:ty, $answer:ident) => {
        $crate::family::record!($family, $field, $record, Request, request_of, $request);

        #[doc = concat!("Every chart's answer to the `", $record, "` record, as the canonical JSON its section carries.")]
        #[cfg(feature = $family)]
        pub(crate) fn json(
            sdk: &teistro::Context,
            documents: &[teistro_serial::Document],
            asked: Option<&Request>,
        ) -> Result<String, teistro_core::error::Error> {
            $crate::family::each_json(documents, asked, |document, asked| {
                sdk.chart().$answer(document, asked)
            })
        }

        #[doc = concat!("The `", $record, "` section of a build without `", $family, "`, which no request can ask for.")]
        #[cfg(not(feature = $family))]
        #[allow(clippy::unnecessary_wraps, reason = "the signature of the build with the family")]
        pub(crate) fn json(
            _sdk: &teistro::Context,
            _documents: &[teistro_serial::Document],
            asked: Option<&Request>,
        ) -> Result<String, teistro_core::error::Error> {
            Ok($crate::family::unasked(asked))
        }
    };
}

pub(crate) use {answer, chart_record, in_family, record};
