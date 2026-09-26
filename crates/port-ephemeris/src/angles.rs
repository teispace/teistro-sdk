//! The angles of a chart from a provider's own reckoning (the angles
//! override): the ascendant and the midheaven at an instant and a place.
//!
//! The SDK builds the angles from the sphere — the sidereal time, the
//! latitude and the obliquity — and a modern provider's would be the same
//! sky. A classical text reckons them its own way: the Surya Siddhanta
//! carries the Sun at its sunrise through the signs' rising times in
//! proportion within each sign (III.46 to 49), and no choice of obliquity
//! reproduces that (`03-design/classical-chart-measured.md`). So a
//! provider that **defines** the angles answers them here, and the chart
//! builds the house systems that need only the angles from them.

use serde::{Deserialize, Serialize};
use teistro_core::quantity::Place;

use crate::body::TimeScale;

/// What the angles are asked for: an instant and a place.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AnglesRequest {
    /// The instant, a Julian day.
    pub jd: f64,
    /// The scale the instant is in.
    pub scale: TimeScale,
    /// The place.
    pub place: Place,
}

/// A provider's angles, **tropical** degrees of date, with the obliquity
/// its own reckoning takes: the chart measures them in its zodiac, as it
/// does every other longitude.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Angles {
    /// The ascendant: the point of the ecliptic rising in the east.
    pub ascendant_deg: f64,
    /// The midheaven: the point of the ecliptic on the meridian.
    pub midheaven_deg: f64,
    /// The obliquity the angles were reckoned with.
    pub obliquity_deg: f64,
}
