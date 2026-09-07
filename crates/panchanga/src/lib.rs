//! The daily panchanga: the almanac of one day at one place.
//!
//! A chart is consulted once; a panchanga every morning. It is also
//! almost entirely convention — which arc a period divides, which instant
//! a limb is read at, which of two nights sizes a muhurta — and none of
//! that shows in a number while all of it changes the number. So it was
//! measured before it was designed: `cargo xtask panchanga` proposes a
//! rule for each of the conformance corpus's twenty-seven recorded fields
//! and tests it over all 55 days
//! (`03-design/panchanga-day-conventions.md`), and this crate is the
//! design written from the result (`03-design/panchanga-day.md`).
//!
//! Four things it settles, each measured:
//!
//! - **The day is a window, and the window is a [`LocalDay`].** Sunrise
//!   to the next sunrise, on every recorded day. Nothing here needs a day
//!   model of its own, and `day.day_boundary` — declared in Phase 1 and
//!   never read until now — is what moves it.
//! - **A limb is a [`span::Span`], and it carries its own bounds as well
//!   as the clipped ones.** The corpus records only the clipped view, so
//!   "the tithi began yesterday at 21:05" is a fact it cannot answer and
//!   this crate can.
//! - **Every period is one division of one of two arcs** ([`period`]),
//!   and a period whose arc does not exist is absent rather than empty.
//! - **The choghadiya is the hora's weekday walk**, not a grid of
//!   fifty-six names.
//!
//! ```no_run
//! # use teistro_core::quantity::Place;
//! # use teistro_core::settings::Resolved;
//! # fn example(almanac: &teistro_panchanga::Almanac<'_, teistro_port_ephemeris::TestProvider>, date: &teistro_calendar::CalendarDate, place: &Place) -> Result<(), teistro_core::error::Error> {
//! let day = almanac.day(date, place)?;
//! // What an almanac row prints: the tithi, and when it gives way.
//! for tithi in &day.value.limbs.tithi {
//!     println!("{:?} until {}", tithi.member, tithi.inside.to);
//! }
//! # Ok(())
//! # }
//! ```

pub mod almanac;
pub mod limb;
pub mod month;
pub mod omen;
pub mod period;
pub mod sky;
pub mod span;

pub use almanac::{Almanac, Panchanga};
pub use limb::{Limbs, karana_of};
pub use month::LunarMonth;
pub use omen::{MuhurtaYoga, Omens, YogaCause};
pub use period::{Kaalas, Muhurtas, Part};
pub use sky::{MoonDay, SunDay};
pub use span::Span;
pub use teistro_time::local_day::LocalDay;
