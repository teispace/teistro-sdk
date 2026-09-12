//! The Teistro SDK for Rust: the surface a Rust consumer reads.
//!
//! Node, Dart and Python read `sdk.<area>.<operation>` off a context.
//! This crate is that, in Rust, and it is designed rather than improvised
//! — `docs/03-design/rust-consumer-surface.md` decides its shape, from
//! the measurement in `rust-consumer-surface-measured.md`.
//!
//! What the measurement found, and why this crate exists: a Rust consumer
//! could build a context before this — `TsContext::build` is `pub` — and
//! could not *use* one, because converting a date meant
//! `ts_calendar_convert` with three raw pointers. So Rust had a context
//! it could build and could not use, and two of the nine crates such a
//! context needs were held by the C boundary and by nothing else.
//!
//! ```
//! # // The built-in ephemeris is a feature, and a doctest sees the
//! # // crate's features, so this one compiles away with it rather than
//! # // failing a `--no-default-features` run.
//! # #[cfg(feature = "builtin-ephemeris")] fn run() -> Result<(), teistro::Error> {
//! use teistro::catalogue::Calendar;
//! use teistro::{CalendarDate, Context, Ephemeris};
//!
//! let sdk = Context::builder()
//!     .profile("nepali-default")
//!     .locale("ne-Deva-NP")
//!     .ephemeris([Ephemeris::Builtin])
//!     .build()?;
//!
//! // 14 April 2015 is 1 Baisakh 2072 BS.
//! let day = CalendarDate::defined(Calendar::Gregorian, 2015, 4, 14);
//! let bs = sdk.calendar().convert(&day, Calendar::BikramSambat)?;
//! assert_eq!((bs.year, bs.month, bs.day), (2072, 1, 1));
//! # Ok(()) }
//! # #[cfg(feature = "builtin-ephemeris")] run().unwrap();
//! ```
//!
//! **It composes the crates; it does not call the SDK's own C ABI.** A
//! Rust consumer reaching itself through a C boundary would write a
//! request struct so the boundary could decode it, and decode a result
//! blob to read numbers the crates had already returned as `JulianDay`
//! and `Longitude`. Eight of the boundary's forty-six entry points are
//! that marshalling and nothing else.

mod area;
mod context;
mod ephemeris;
mod scale;

pub use area::{
    AlmanacArea, CalendarArea, ChartArea, EngineArea, FrameArea, IntlArea, KeysArea, TimeArea,
};
pub use context::{Context, ContextBuilder};
pub use ephemeris::Ephemeris;
pub use scale::{Conversion, Scale};

// The types an operation takes and answers with are the crates' own, and
// are re-exported so a consumer needs one dependency rather than five.
// Re-exported and **not** wrapped: a newtype over `JulianDay` would be a
// second type with the same invariant and no new one (ADR-0023).
//
// The rule this list keeps is checkable rather than a judgement:
// **every type an area's signature names is reachable from this crate
// root.** An operation answering a `CalendarResolution` a consumer
// cannot name is an operation whose answer cannot be matched on, and
// the examples found four of those before this list did —
// `CalendarResolution`, `Envelope`, `ChartFoundation` and `Panchanga`.
// `rust-consumer-surface-measured.md`'s *every type an area's signature
// names is reachable from the crate root* holds it now, born red on
// exactly those four.
pub use teistro_astro::DeltaTModel;
pub use teistro_astro::delta_t::DeltaT;
pub use teistro_calendar::{CalendarDate, FixedDay, Weekday};
pub use teistro_chart::foundation::{ChartFoundation, GrahaPosition};
pub use teistro_core::catalogue;
// `canonical_json` and `content_hash` with them: a stored chart keeps
// the bytes the provenance was hashed from, and without these a
// consumer could hold an `Envelope` and not reproduce its own hash.
pub use teistro_core::envelope::{
    CalendarResolution, Envelope, Hash, Provenance, canonical_json, content_hash,
};
pub use teistro_core::error::{Error, Status};
pub use teistro_core::interval::Interval;
pub use teistro_core::key::KeyId;
pub use teistro_core::quantity;
pub use teistro_core::settings;
pub use teistro_panchanga::almanac::Panchanga;
pub use teistro_port_ephemeris::native::{NativeFunction, NativeManifest};
// The typed accessor tree: every message of the SDK's locale as a value
// of its own parameters. A **module** tree, because that is what a
// namespace is in Rust — where Node writes
// `ctx.intl.messages.sdk.reason.grahaInBhava({ … })`.
pub use teistro_intl::messages;
// What the locale area takes and answers with.
pub use teistro_intl::source::Entity;
pub use teistro_intl::translit::Script;
pub use teistro_intl::{Intl, Loaded, Params, Rendered, TypedMessage, Value, params};
// What `positions` takes and answers with, and what an ephemeris of
// your own implements. Re-exported because a consumer needing five
// dependencies to call one operation is the thing this crate exists to
// stop -- and the test for `positions` reached past it before these
// were here, which is how the gap was noticed.
pub use teistro_astro::completion::Completed;
pub use teistro_core::time::UtcOffset;
// And what an ephemeris of your own needs to *implement* the port, not
// merely to call it: writing a provider is a first-class use of this
// crate (ADR-0029 hands a Rust consumer the adapters as `rlib`s), so a
// consumer writing one should need this dependency and no other.
pub use teistro_port_ephemeris::capabilities::{Astronomy, DistanceUnit, SpeedModel};
pub use teistro_port_ephemeris::columns::{EphemerisKind, Source};
pub use teistro_port_ephemeris::provider::validate;
pub use teistro_port_ephemeris::{
    Body, Capabilities, Cell, CellStatus, Centre, Coordinates, Corrections, EphemerisProvider,
    Equinox, Frame, Identity, Overrides, PositionColumns, PositionRequest, ProviderError,
    TimeScale, Zodiac,
};
pub use teistro_time::{CivilDateTime, CivilTime, Resolved, ZoneResolution, ZoneSpec};

/// The Julian day a fixed day begins at, in local time.
///
/// A **free function** and not an area's operation, as it is in the
/// other three bindings: it takes no context, because a fixed day and a
/// Julian day are two spellings of the same integer and nothing about a
/// profile or a locale can change the arithmetic.
#[must_use]
pub fn jd_of_fixed(fixed: FixedDay) -> f64 {
    #[expect(
        clippy::cast_precision_loss,
        reason = "a fixed day within any calendar's range is exact in an f64"
    )]
    let day = fixed.get() as f64;
    day + FixedDay::JD_EPOCH
}

/// The fixed day a Julian day falls in, and the fraction of that day
/// elapsed since its midnight.
#[must_use]
pub fn fixed_of_jd(jd: f64) -> (FixedDay, f64) {
    FixedDay::from_local_jd(jd)
}

// `BUNDLES`: the SDK's locales, built from `i18n/` by this crate's build
// script so a consumer needs no files to render its messages (ADR-0010).
include!(concat!(env!("OUT_DIR"), "/bundles.rs"));
