//! How far a day's times may move with Delta T (SR-104,
//! `docs/03-design/time-and-timezone.md` §3.5).
//!
//! A limb's end, a sankranti and a new Moon are fixed in Terrestrial Time:
//! the Sun and the Moon reach a longitude at one TT instant, and its civil
//! time is that instant less Delta T. So a limb's time moves second for
//! second with Delta T, and carries Delta T's whole uncertainty:
//! `|∂t/∂ΔT| = 1`. A rise is fixed by the Earth's turning, in UT; Delta T
//! reaches it only through where the body has moved to by the TT instant
//! the sky is read at, so a rise moves by `n / (ω − n)` of Delta T, with
//! `n` the body's mean motion and `ω` the Earth's sidereal rotation: about
//! 1/365 for the Sun and 1/26 for the Moon. That is why, against another
//! library's Delta T in 2100, every limb moves by the same minute and a
//! half while the sunrise does not.

use serde::{Deserialize, Serialize};
use teistro_astro::delta_t::DeltaT;
use teistro_port_ephemeris::DeltaTBand;

/// The Earth's sidereal rotation, degrees a UT1 day: the IAU 2000 Earth
/// rotation angle's rate, 1.002 737 811 911 354 48 turns a day (IERS
/// Conventions 2010, eq. 5.15).
const EARTH_ROTATION_DEG_PER_DAY: f64 = 360.0 * 1.002_737_811_911_354_5;
/// The Sun's mean motion in longitude, degrees a day (Meeus,
/// *Astronomical Algorithms*, eq. 25.2: 36 000.769 83° a Julian century).
const SUN_MEAN_MOTION_DEG_PER_DAY: f64 = 36_000.769_83 / 36_525.0;
/// The Moon's mean motion in longitude, degrees a day (Meeus, eq. 47.1:
/// 481 267.881 342 3° a Julian century).
const MOON_MEAN_MOTION_DEG_PER_DAY: f64 = 481_267.881_342_3 / 36_525.0;

/// How much of a change in Delta T a body's rise takes on, from its mean
/// motion: `n / (ω − n)`.
///
/// ```
/// use teistro_panchanga::uncertainty::{moonrise_share, sunrise_share};
///
/// // A sunrise takes on about 1/365 of Delta T, a moonrise about 1/26.
/// assert!((1.0 / sunrise_share() - 365.2).abs() < 0.5);
/// assert!((1.0 / moonrise_share() - 26.4).abs() < 0.1);
/// ```
#[must_use]
pub fn sunrise_share() -> f64 {
    SUN_MEAN_MOTION_DEG_PER_DAY / (EARTH_ROTATION_DEG_PER_DAY - SUN_MEAN_MOTION_DEG_PER_DAY)
}

/// [`sunrise_share`] for the Moon.
#[must_use]
pub fn moonrise_share() -> f64 {
    MOON_MEAN_MOTION_DEG_PER_DAY / (EARTH_ROTATION_DEG_PER_DAY - MOON_MEAN_MOTION_DEG_PER_DAY)
}

/// A day's times with a ± band from Delta T: the Delta T the day was
/// reckoned under, where its uncertainty comes from, and the one-sigma
/// band each kind of time takes on from it. **Left out unless asked for**
/// (the façade's `AlmanacRequest::with_uncertainty`).
///
/// ```
/// use teistro_astro::delta_t::{DeltaTModel, delta_t};
/// use teistro_core::quantity::{JulianDay, Ut1};
/// use teistro_panchanga::uncertainty::TimesUncertainty;
/// use teistro_port_ephemeris::DeltaTBand;
///
/// // 2100: Delta T is extrapolated, with Huber's band.
/// let applied = delta_t(JulianDay::<Ut1>::literal(2_488_069.5), DeltaTModel::TableThenModel)?;
/// let band = TimesUncertainty::of(&applied);
/// assert_eq!(band.band, DeltaTBand::Huber2000);
/// // A limb takes on the whole of it, a sunrise a 365th part.
/// let limbs = band.limbs_seconds.ok_or("stated")?;
/// assert_eq!(Some(limbs), applied.uncertainty_seconds);
/// assert!(band.sunrise_seconds.ok_or("stated")? < limbs / 300.0);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TimesUncertainty {
    /// Delta T at the day's window's middle, TT less UT1, seconds.
    pub delta_t_seconds: f64,
    /// Where its uncertainty comes from.
    pub band: DeltaTBand,
    /// The one-sigma band of each limb's end (tithi, nakshatra, yoga,
    /// karana), of a sankranti and of a lunation, seconds: Delta T's own,
    /// since they are fixed in TT. `None` when Delta T states none (a
    /// custom value, or an engine's that gives none).
    pub limbs_seconds: Option<f64>,
    /// The one-sigma band of the sunrise and sunset, and of every period
    /// divided from them, seconds: [`sunrise_share`] of Delta T's.
    pub sunrise_seconds: Option<f64>,
    /// The one-sigma band of the moonrise and moonset, seconds:
    /// [`moonrise_share`] of Delta T's.
    pub moonrise_seconds: Option<f64>,
}

impl TimesUncertainty {
    /// The bands a Delta T gives a day's times.
    #[must_use]
    pub fn of(delta_t: &DeltaT) -> TimesUncertainty {
        let sigma = delta_t.uncertainty_seconds;
        TimesUncertainty {
            delta_t_seconds: delta_t.seconds,
            band: delta_t.band(),
            limbs_seconds: sigma,
            sunrise_seconds: sigma.map(|s| s * sunrise_share()),
            moonrise_seconds: sigma.map(|s| s * moonrise_share()),
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_astro::delta_t::{DeltaTModel, delta_t};
    use teistro_core::quantity::{JulianDay, Ut1};

    use super::*;

    #[test]
    fn a_custom_delta_t_states_no_band() {
        let custom = delta_t(
            JulianDay::<Ut1>::literal(2_451_545.0),
            DeltaTModel::Custom { seconds: 64.0 },
        )
        .unwrap();
        let band = TimesUncertainty::of(&custom);
        assert_eq!(
            (band.delta_t_seconds, band.band),
            (64.0, DeltaTBand::Unstated)
        );
        assert_eq!(
            (
                band.limbs_seconds,
                band.sunrise_seconds,
                band.moonrise_seconds
            ),
            (None, None, None)
        );
        let json = serde_json::to_value(band).unwrap();
        assert_eq!(json["band"], "UNSTATED");
    }

    #[test]
    fn a_measured_day_has_a_band_of_milliseconds() {
        let measured = delta_t(
            JulianDay::<Ut1>::literal(2_451_545.0),
            DeltaTModel::TableThenModel,
        )
        .unwrap();
        let band = TimesUncertainty::of(&measured);
        assert_eq!(band.band, DeltaTBand::Measured);
        assert!(band.limbs_seconds.unwrap() < 0.01);
        let moon = band.moonrise_seconds.unwrap();
        assert!(moon > band.sunrise_seconds.unwrap() && moon < band.limbs_seconds.unwrap());
    }
}
