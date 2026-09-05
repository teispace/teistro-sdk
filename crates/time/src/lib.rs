//! The time layer of the Teistro SDK (`docs/03-design/time-and-timezone.md`).
//!
//! Every chart begins with "when": a civil date and time at a place,
//! resolved to an instant on a named time scale with enough recorded to
//! reproduce the resolution after the zone database changes. This crate
//! holds:
//!
//! - [`scale`]: the conversions between UTC, UT1 and TT, explicit
//!   functions that stamp what they applied, over the astronomy layer's
//!   Delta T ([`delta_t`], re-exported from `teistro_astro`: the IERS
//!   table where measured and a cited model either side, with an
//!   uncertainty on every value);
//! - [`leap`]: the leap-second table and what it allows;
//! - [`civil`]: a civil time and a civil date-time in any calendar;
//! - [`zone`]: zone specifications, resolution under the daylight-saving
//!   policies, and the metadata a stored chart keeps ([`ZoneResolution`]);
//!   the embedded database ([`EmbeddedTzdb`]) implements the time-zone
//!   port and never reads the host;
//! - [`local_day`]: the sunrise-anchored day at a place from a solar
//!   model, with the polar policies;
//! - [`ghati`]: ghati-pala as exact integer arithmetic on microseconds;
//! - [`hora`]: the twenty-four planetary hours of a day and their lords.
//!
//! Instants are `f64` Julian days, which resolve to about fifty
//! microseconds in the present era; the ghati-pala arithmetic is exact
//! on the microsecond count it derives from them.
//!
//! ```
//! use teistro_calendar::CalendarDate;
//! use teistro_core::catalogue::Calendar;
//! use teistro_time::{CivilDateTime, CivilTime, EmbeddedTzdb, Policy, ZoneEra, ZoneSpec, resolve};
//!
//! // A birth in Kathmandu twenty minutes into 1986, after the clocks moved
//! // from +05:30 to +05:45 at midnight.
//! let civil = CivilDateTime::at(
//!     CalendarDate::defined(Calendar::Gregorian, 1986, 1, 1),
//!     CivilTime::new(0, 20, 0).expect("a real time"),
//! );
//! let resolved = resolve(&civil, &ZoneSpec::iana("Asia/Kathmandu"), &Policy::default(), EmbeddedTzdb::shared())
//!     .expect("a zone the database knows");
//! assert_eq!(resolved.zone.offset.to_string(), "+05:45");
//! assert_eq!(resolved.zone.era, ZoneEra::Current);
//! assert!((resolved.instant.get() - 2_446_431.274_305_6).abs() < 1e-6);
//! ```

pub mod civil;
pub mod ghati;
pub mod hora;
pub mod leap;
pub mod local_day;
pub mod scale;
pub mod zone;
pub mod zones;

#[rustfmt::skip]
mod generated;

pub use civil::{CivilDateTime, CivilTime};
pub use ghati::{GhatiPala, Reckoning, ghati_pala, instant_of};
pub use hora::{Hora, hora_at, horas};
pub use local_day::{DayState, LocalDay, PolarKind, local_day};
pub use scale::{
    TimeBasis, TtConversion, tt_from_ut1, tt_from_utc, ut1_from_tt, ut1_from_utc, utc_from_tt,
    utc_from_ut1,
};
pub use teistro_astro::delta_t::{self, DeltaT, DeltaTModel, DeltaTSource, delta_t};
pub use teistro_core::time::{LocalClock, LocalMeanTime, UtcOffset};
pub use teistro_port_timezone::{LocalCandidates, LocalSeconds, OffsetInfo, TimeZoneProvider};
pub use zone::embedded::{EmbeddedTzdb, ZoneClock};
pub use zone::{
    Chosen, DayContext, DstOutcome, Policy, Resolved, Warning, ZoneEra, ZoneResolution, ZoneSource,
    ZoneSpec, civil_of, civil_of_with, resolve, resolve_at_place, resolve_at_place_with,
    resolve_with,
};
