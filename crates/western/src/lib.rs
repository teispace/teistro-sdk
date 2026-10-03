//! Modern Western doctrine: progressions and directions
//! (`03-design/western-progressions.md`), and the aspects a chart holds
//! under a model of orbs (`03-design/western-aspects.md`).
//!
//! Everything here is the arithmetic of a measure, so there is no ephemeris
//! and no chart: a progression matches an instant of the sky to an instant
//! of the life it is read for, and an arc measure matches degrees to years.
//! Founding the chart at the instant a progression answers is the SDK's
//! work. Every measure is held to Alan Leo's *The Progressed Horoscope*
//! (1906), whose own nativity is the fixture.
//!
//! ```
//! use teistro_western::{ArcMeasure, Progression};
//! use teistro_core::quantity::{JulianDay, Utc};
//!
//! // Leo's birth, 7 August 1860 at 5.49 a.m. in London, and his 47th year.
//! let birth = JulianDay::<Utc>::try_new(2_400_629.742_361)?;
//! let life = birth.plus_days(46.0 * 365.242_189)?;
//!
//! // A day for a year: the sky forty-six days after birth.
//! let sky = Progression::SECONDARY.sky_at(birth, life)?;
//! assert!((sky.get() - birth.get() - 46.0).abs() < 1e-9);
//! assert!((Progression::SECONDARY.life_at(birth, sky)?.get() - life.get()).abs() < 1e-6);
//!
//! // Naibod's measure: 20° 15′ of arc is about 20 years and 199 days.
//! let years = ArcMeasure::Naibod.years(20.25)?;
//! assert!((years - 20.545).abs() < 1e-3);
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

#![doc(html_no_source)]

mod angles;
mod antiscia;
mod arc;
mod aspects;
mod declination;
mod midpoint;
mod progression;
mod synastry;

pub use angles::{AngleMethod, Meridian, SunAt, progressed_armc};
pub use antiscia::{
    Antiscia, AntisciaRequest, Antiscion, AntiscionRow, antiscia, antiscion_deg,
    contrantiscion_deg, synastry_antiscia,
};
pub use arc::ArcMeasure;
pub use aspects::{
    AspectOrb, AspectRequest, BodyOrb, OrbModel, Placed, PlanetAt, WesternAspect, WesternAspectRow,
    aspects,
};
pub use declination::{
    Declined, LEO_PARALLEL_ORB_DEG, MAX_PARALLEL_ORB_DEG, ParallelRequest, ParallelRow, parallels,
};
pub use midpoint::{
    DEFAULT_MIDPOINT_ORB_DEG, MAX_MIDPOINT_ORB_DEG, MidpointRequest, MidpointRow, midpoints,
};
pub use progression::{
    Progression, Rate, SIDEREAL_MONTH_DAYS, SYNODIC_MONTH_DAYS, Span, TROPICAL_YEAR_DAYS,
    YearMeasure,
};
pub use synastry::{
    DeclinedPoint, SynastryParallelRow, SynastryPoint, SynastryRequest, SynastryRow,
    SynastryZodiac, synastry, synastry_parallels,
};
