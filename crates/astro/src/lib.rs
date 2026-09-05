//! The astronomy layer of the Teistro SDK: everything above raw positions
//! (`docs/02-architecture/01-module-catalog.md`,
//! `01-research/platform/13-astronomy-layer.md`).
//!
//! - [`delta_t`] and [`scale`]: Delta T as the IERS table where measured
//!   and a cited model either side, with an uncertainty on every value,
//!   and the conversions between UT1 and TT;
//! - [`iau`]: the IAU routines ported from ERFA with a provenance table
//!   (ADR-0021): the Earth rotation angle, mean and apparent sidereal
//!   time, the obliquity, the IAU 2000B nutation, the refraction
//!   constants;
//! - [`sky`]: the obliquity record, the rotation between the ecliptic and
//!   the equator, and apparent sidereal time at a place;
//! - [`completion`]: frame completion over the ephemeris port, from the
//!   frame a provider returns to the frame a caller asks for, with the
//!   override policy deciding who does each step and every step stamped;
//! - [`precession`]: precession as a catalogue of models (Vondrák 2011,
//!   IAU 2006, IAU 1976, Newcomb) over the ported routines, with the mean
//!   obliquity each is consistent with;
//! - [`ayanamsha`]: the ayanamsha catalogue, every epoch-defined member
//!   computed by the SDK from its published epoch and value carried by
//!   precession, mean or with nutation, so a sidereal zodiac needs no
//!   provider override;
//! - [`houses`]: the twenty-two catalogued house systems from the meridian,
//!   the latitude and the obliquity, with the auxiliary points and the
//!   polar policy;
//! - [`solve`]: the shared boundary solver, one root finder every event
//!   search in the SDK goes through;
//! - [`rise_set`]: the rise and set solver under a horizon convention,
//!   with polar days and nights as reported states;
//! - [`stars`]: the star table's places of date from the catalogue's ICRS
//!   astrometry: proper motion, parallax, deflection and aberration over
//!   the SDK's own Earth ephemeris, then the frame bias, precession and
//!   nutation; what the star-anchored ayanamshas read.
//! - [`phenomena`]: what a body looks like: elongation, phase, the disc,
//!   the horizontal parallax and the visual magnitude under the Almanac's
//!   models; and the equation of time beside the sidereal time in [`sky`].
//! - [`events`]: crossings of a longitude, a composite angle or a speed
//!   over a lattice of boundaries, and stations, one kernel over the
//!   boundary solver.
//!
//! ```
//! use teistro_astro::delta_t::{DeltaTModel, delta_t};
//! use teistro_core::quantity::{JulianDay, Ut1};
//!
//! let at = JulianDay::<Ut1>::literal(2_451_544.5);
//! let dt = delta_t(at, DeltaTModel::TableThenModel).expect("inside the table");
//! assert!((dt.seconds - 63.83).abs() < 0.02);
//! ```

pub mod ayanamsha;
pub mod completion;
pub mod delta_t;
pub mod events;
pub mod houses;
pub mod iau;
pub mod phenomena;
pub mod precession;
pub mod rise_set;
pub mod scale;
pub mod sky;
pub mod solve;
pub mod stars;
pub mod visibility;

#[rustfmt::skip]
mod generated;

pub use completion::{Completed, Completion, CompletionError, Implementation, Step};
pub use delta_t::{DeltaT, DeltaTModel, DeltaTSource, delta_t};
pub use rise_set::{DayEvents, HorizonEvent, Method, Outcome};
pub use scale::{tt_from_ut1, tt_of, ut1_from_tt};
pub use sky::{
    Apparent, ApparentPositions, Spherical, equation_of_time_seconds, obliquity, sidereal_time_deg,
};
pub use solve::{Caps, Crossing, SolveError, first_zero, next_crossing, refine};
