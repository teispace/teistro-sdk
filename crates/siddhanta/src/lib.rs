//! The Surya Siddhanta as a computation model (`docs/03-design/siddhanta.md`).
//!
//! The text (cited throughout by chapter and verse in the translation of
//! Ebenezer Burgess, 1860) gives mean motions as whole revolutions per
//! age, apsides and nodes as revolutions per aeon, epicycles as
//! circumferences at the ends of the even and odd quadrants, a
//! twenty-four-entry sine table with an interpolation rule, and the
//! procedure that turns them into true longitudes. This crate implements
//! exactly that, with nothing added:
//!
//! - [`params`]: every number the text supplies, as [`Parameters::TEXT`],
//!   and the overlay a tradition's bija corrections take;
//! - [`trig`]: the sine table, its interpolation and inverse, and the
//!   exact-trigonometry alternative for comparison;
//! - [`mean`]: the day count from the epoch and the mean longitudes,
//!   apsides and nodes, in exact integer arithmetic for the whole days;
//! - [`equation`]: the corrected epicycle, the manda and sighra
//!   equations, the four-step procedure and the true daily motion;
//! - [`model`]: [`SuryaSiddhanta`], which answers for a graha at an
//!   instant, and the text's declination, ascensional difference and
//!   precession, which give sunrise and sunset the way an almanac
//!   computes them.
//!
//! The classical path ([`Trig::Table`]) uses only the table and
//! elementary arithmetic, so its results are bit-identical on every
//! platform; [`Trig::Exact`] substitutes the platform's sine for
//! comparison and is not bit-identical.
//!
//! ```
//! use teistro_core::catalogue::Graha;
//! use teistro_core::quantity::{JulianDay, Ut1};
//! use teistro_siddhanta::{SuryaSiddhanta, Trig};
//!
//! let text = SuryaSiddhanta::text();
//! // 13 April 2024, 0h UT: the day the Nepali calendar begins 2081 BS.
//! let instant = JulianDay::<Ut1>::try_new(2_460_413.5).expect("finite");
//! let sun = text.sun(instant);
//! assert!(sun.longitude.get() < 1.0 || sun.longitude.get() > 359.0);
//! let saturn = text.graha(Graha::Saturn, instant).expect("a body the text models");
//! assert!(saturn.speed_deg_per_day.abs() < 0.2);
//! assert_eq!(text.trig(), Trig::Table);
//! ```

pub mod equation;
pub mod mean;
pub mod model;
pub mod params;
pub mod trig;

pub use equation::{Epicycle, SighraEquation};
pub use mean::{Ahargana, Cycle, Motion};
pub use model::{DayArc, Position, SuryaSiddhanta, Trace};
pub use params::{Bija, Parameters, Planet};
pub use trig::Trig;
