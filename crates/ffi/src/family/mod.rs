//! The module families at the boundary (`03-design/wasm-profiles.md`).
//!
//! A family is a feature of the façade, forwarded here. Every entry point
//! exists in every build, so the generated glue is one file for every
//! profile; a call into a family the build left out answers
//! `CAPABILITY`, naming the family, rather than a missing symbol.

use core::ffi::c_char;

use teistro_core::error::{Error, Status};

// The families read off a chart, which a build without the chart area
// has no chart to read them off.
#[cfg(not(feature = "chart"))]
pub mod chart;
#[cfg(feature = "chart")]
pub(crate) mod kp;
#[cfg(feature = "chart")]
pub(crate) mod lalkitab;
pub(crate) mod muhurta;
#[cfg(feature = "chart")]
pub(crate) mod prashna;
#[cfg(feature = "chart")]
pub(crate) mod rectification;
#[cfg(feature = "chart")]
pub(crate) mod remedies;
#[cfg(feature = "chart")]
pub(crate) mod svg;
#[cfg(feature = "chart")]
pub(crate) mod tajika;
#[cfg(feature = "chart")]
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

#[cfg(feature = "chart")]
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

#[cfg(feature = "chart")]
/// A family a chart request asks for by one JSON record and the façade
/// answers a chart at a time ([`teistro::ChartArea::compose`]): the
/// [`record!`] items, under the names every such family uses.
macro_rules! chart_record {
    ($family:literal, $field:literal, $record:literal, $request:ty) => {
        $crate::family::record!($family, $field, $record, Request, request_of, $request);
    };
}

#[cfg(feature = "chart")]
pub(crate) use {answer, chart_record};
pub(crate) use {in_family, record};
