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
//! # Ok::<(), teistro::Error>(())
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

pub use area::{CalendarArea, FrameArea, IntlArea, KeysArea, TimeArea};
pub use context::{Context, ContextBuilder};
pub use ephemeris::Ephemeris;
pub use scale::{Conversion, Scale};

// The types an operation takes and answers with are the crates' own, and
// are re-exported so a consumer needs one dependency rather than five.
// Re-exported and **not** wrapped: a newtype over `JulianDay` would be a
// second type with the same invariant and no new one (ADR-0023).
pub use teistro_calendar::{CalendarDate, FixedDay, Weekday};
pub use teistro_core::catalogue;
pub use teistro_core::envelope::Hash;
pub use teistro_core::error::{Error, Status};
pub use teistro_core::quantity;
pub use teistro_core::settings;
// The typed accessor tree: every message of the SDK's locale as a value
// of its own parameters. A **module** tree, because that is what a
// namespace is in Rust — where Node writes
// `ctx.intl.messages.sdk.reason.grahaInBhava({ … })`.
pub use teistro_intl::messages;
// What `positions` takes and answers with, and what an ephemeris of
// your own implements. Re-exported because a consumer needing five
// dependencies to call one operation is the thing this crate exists to
// stop -- and the test for `positions` reached past it before these
// were here, which is how the gap was noticed.
pub use teistro_astro::completion::Completed;
pub use teistro_port_ephemeris::{
    Body, Capabilities, Cell, CellStatus, EphemerisProvider, Frame, PositionColumns,
    PositionRequest, TimeScale,
};
pub use teistro_time::{CivilDateTime, CivilTime, ZoneSpec};

// `BUNDLES`: the SDK's locales, built from `i18n/` by this crate's build
// script so a consumer needs no files to render its messages (ADR-0010).
include!(concat!(env!("OUT_DIR"), "/bundles.rs"));
