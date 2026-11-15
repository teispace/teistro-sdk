//! How close a lunar month came to another kind (SR-92, C1902).
//!
//! A month is adhika, nija or kshaya by the count of sankrantis between
//! its two new moons ([`super::kind_of`]). The count changes only when a
//! sankranti crosses one of the two new moons, so how far the month was
//! from another kind is how far the Sun stood from a sign boundary at
//! each of them: in degrees of the Sun's sidereal longitude, and in the
//! time the Sun takes to cover them, which is the time between the new
//! moon and the nearest sankranti. A reckoning whose Sun differs by more
//! than that (another siddhanta, another ayanamsha) may decide the month
//! otherwise: Nepal's committee and a drik almanac part on 2028's adhika
//! Kartika and kshaya Margashirsha by 0.044° and 0.017° of the Sun.

use serde::{Deserialize, Serialize};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};

use crate::solar::{SolarModel, find_sankranti};

/// Minutes in a day.
const MINUTES_PER_DAY: f64 = 1440.0;

/// Degrees in a sign.
const SIGN_DEG: f64 = 30.0;

/// How far before a new moon the sankranti that opened the Sun's sign is
/// looked for, days: the Sun is under half a sign past it, which it
/// covers in under 15.8 days at its slowest, so this starts the search
/// inside the sign before.
const OPENED_WITHIN_DAYS: f64 = 17.0;

/// The Sun at one of a month's two new moons, against the nearest sign
/// boundary.
///
/// ```
/// use teistro_calendar::lunisolar::NewMoonMargin;
/// use teistro_core::quantity::JulianDay;
///
/// // Short of the boundary: the sankranti is still to come.
/// let short = NewMoonMargin { new_moon: JulianDay::literal(2_462_000.0), degrees: -0.017, minutes: -24.5 };
/// assert!(short.degrees < 0.0 && short.minutes < 0.0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct NewMoonMargin {
    /// The new moon.
    pub new_moon: JulianDay<Utc>,
    /// The Sun's sidereal longitude less the nearest sign boundary,
    /// degrees, −15 to 15: positive when the Sun had passed it (the
    /// sankranti came before the new moon), negative when it was short of
    /// it (the sankranti came after).
    pub degrees: f64,
    /// The new moon less the nearest sankranti, minutes: the Sun's own
    /// time over `degrees`, with the same sign.
    pub minutes: f64,
}

/// How close a month came to another kind: the Sun against the nearest
/// sign boundary at each of its two new moons, and whether the nearer is
/// inside the threshold `calendars.marginal_month_minutes` names.
///
/// ```
/// use teistro_calendar::lunisolar::{MonthMargin, NewMoonMargin};
/// use teistro_core::quantity::JulianDay;
///
/// let at = |jd, degrees, minutes| NewMoonMargin { new_moon: JulianDay::literal(jd), degrees, minutes };
/// let margin = MonthMargin::of(at(2_462_000.0, 3.2, 4_700.0), at(2_462_029.5, -0.017, -24.5), 360);
/// assert!(margin.marginal);
/// assert_eq!(margin.least_minutes(), 24.5);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MonthMargin {
    /// At the new moon that opened the month.
    pub opening: NewMoonMargin,
    /// At the new moon that closes it.
    pub closing: NewMoonMargin,
    /// Whether either new moon fell within the threshold of a sankranti,
    /// so that a reckoning whose Sun differs by that much may decide the
    /// month's kind otherwise.
    pub marginal: bool,
}

impl MonthMargin {
    /// The margin of two new moons under a threshold in minutes.
    #[must_use]
    pub fn of(opening: NewMoonMargin, closing: NewMoonMargin, threshold_minutes: u32) -> Self {
        let least = opening.minutes.abs().min(closing.minutes.abs());
        MonthMargin {
            opening,
            closing,
            marginal: least < f64::from(threshold_minutes),
        }
    }

    /// The nearer new moon's distance from its sankranti, minutes.
    #[must_use]
    pub fn least_minutes(&self) -> f64 {
        self.opening.minutes.abs().min(self.closing.minutes.abs())
    }

    /// The nearer new moon's distance from its sign boundary, degrees.
    ///
    /// ```
    /// use teistro_calendar::lunisolar::{MonthMargin, NewMoonMargin};
    /// use teistro_core::quantity::JulianDay;
    ///
    /// let at = |degrees: f64| NewMoonMargin { new_moon: JulianDay::literal(2_462_000.0), degrees, minutes: degrees * 1440.0 };
    /// assert_eq!(MonthMargin::of(at(0.044), at(-2.0), 360).least_degrees(), 0.044);
    /// ```
    #[must_use]
    pub fn least_degrees(&self) -> f64 {
        self.opening.degrees.abs().min(self.closing.degrees.abs())
    }
}

/// The Sun against the nearest sign boundary at a new moon.
///
/// ```
/// use teistro_siddhanta::SuryaSiddhanta;
/// use teistro_calendar::lunisolar::{month_at, new_moon_margin};
/// use teistro_core::quantity::{JulianDay, Utc};
///
/// let text = SuryaSiddhanta::text();
/// let month = month_at(&text, &text, JulianDay::<Utc>::literal(2_451_545.0))?;
/// let margin = new_moon_margin(&text, month.from)?;
/// assert!(margin.degrees.abs() <= 15.0);
/// // The time to the sankranti is the Sun's, about a degree a day.
/// assert!((margin.minutes / 1440.0 - margin.degrees).abs() < 0.1 * margin.degrees.abs() + 1e-6);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// The model's own refusal, or a sankranti search that does not
/// converge.
pub fn new_moon_margin(
    sun: &dyn SolarModel,
    new_moon: JulianDay<Utc>,
) -> Result<NewMoonMargin, Error> {
    let at = new_moon.get();
    let longitude = sun.sidereal_sun_deg(at)?.rem_euclid(360.0);
    let within = longitude.rem_euclid(SIGN_DEG);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a normalised longitude over thirty is 0 to 11"
    )]
    let sign = ((longitude / SIGN_DEG) as u8).min(11);
    let (degrees, sankranti) = if within < SIGN_DEG / 2.0 {
        // Past the boundary: the sankranti that opened this sign.
        let from = JulianDay::try_new(at - OPENED_WITHIN_DAYS)?;
        (within, find_sankranti(sun, sign, from)?)
    } else {
        // Short of it: the sankranti into the next sign.
        (
            within - SIGN_DEG,
            find_sankranti(sun, (sign + 1) % 12, new_moon)?,
        )
    };
    Ok(NewMoonMargin {
        new_moon,
        degrees,
        minutes: (at - sankranti.instant.get()) * MINUTES_PER_DAY,
    })
}

/// A month's margin from its two new moons ([`new_moon_margin`]) under a
/// threshold in minutes.
///
/// ```
/// use teistro_siddhanta::SuryaSiddhanta;
/// use teistro_calendar::lunisolar::{MonthKind, month_at, month_margin};
/// use teistro_core::quantity::{JulianDay, Utc};
///
/// // January 1964 held a kshaya month: one sankranti fell just after its
/// // opening new moon and another just before its closing one.
/// let text = SuryaSiddhanta::text();
/// let month = month_at(&text, &text, JulianDay::<Utc>::literal(2_438_395.0))?;
/// assert_eq!(month.kind, MonthKind::Kshaya);
/// let margin = month_margin(&text, month.from, month.to, 360)?;
/// assert!(margin.opening.degrees < 0.0 && margin.closing.degrees > 0.0);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`new_moon_margin`].
pub fn month_margin(
    sun: &dyn SolarModel,
    from: JulianDay<Utc>,
    to: JulianDay<Utc>,
    threshold_minutes: u32,
) -> Result<MonthMargin, Error> {
    Ok(MonthMargin::of(
        new_moon_margin(sun, from)?,
        new_moon_margin(sun, to)?,
        threshold_minutes,
    ))
}
