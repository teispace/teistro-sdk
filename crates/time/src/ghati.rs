//! Ghati-pala: the Vedic day's sixty ghatis of sixty palas of sixty
//! vipalas, counted from sunrise, as exact integer arithmetic on
//! microseconds (`docs/03-design/time-and-timezone.md`, §3.3 and §4).

use core::fmt;

use teistro_core::error::{Error, Status};
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::settings::GhatiReckoning;

use crate::local_day::LocalDay;

/// Microseconds in a day.
const MICROS_PER_DAY: i128 = 86_400_000_000;
/// Microseconds in a civil vipala (0.4 s).
const MICROS_PER_VIPALA: i128 = 400_000;
/// Vipalas in a ghati.
const VIPALAS_PER_GHATI: u32 = 3600;
/// Vipalas in half a day of thirty ghatis.
const VIPALAS_PER_HALF: i128 = 108_000;

/// A ghati-pala count from sunrise.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct GhatiPala {
    /// Ghatis, 0 to 59 (60 when the sunrise-to-sunrise day exceeds
    /// twenty-four hours under civil reckoning).
    pub ghati: u8,
    /// Palas, 0 to 59.
    pub pala: u8,
    /// Vipalas, 0 to 59.
    pub vipala: u8,
}

impl GhatiPala {
    /// The count as whole vipalas.
    #[must_use]
    pub const fn total_vipalas(self) -> u32 {
        self.ghati as u32 * VIPALAS_PER_GHATI + self.pala as u32 * 60 + self.vipala as u32
    }

    /// A count from whole vipalas.
    #[must_use]
    pub fn from_vipalas(vipalas: u32) -> GhatiPala {
        GhatiPala {
            ghati: u8::try_from(vipalas / VIPALAS_PER_GHATI).unwrap_or(u8::MAX),
            pala: u8::try_from(vipalas / 60 % 60).unwrap_or(59),
            vipala: u8::try_from(vipalas % 60).unwrap_or(59),
        }
    }
}

impl fmt::Display for GhatiPala {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{:02}-{:02}", self.ghati, self.pala, self.vipala)
    }
}

/// How ghatis are counted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Reckoning {
    /// A ghati is twenty-four minutes, counted from sunrise.
    Civil,
    /// Thirty ghatis span the daylight and thirty the night.
    Proportional,
}

impl TryFrom<GhatiReckoning> for Reckoning {
    type Error = Error;

    /// The reckoning a settings knob names.
    ///
    /// # Errors
    ///
    /// A knob value core added before this crate learnt it.
    fn try_from(knob: GhatiReckoning) -> Result<Reckoning, Error> {
        match knob {
            GhatiReckoning::Civil => Ok(Reckoning::Civil),
            GhatiReckoning::Proportional => Ok(Reckoning::Proportional),
            other => Err(unknown_knob("day.ghati_reckoning", &format!("{other:?}"))),
        }
    }
}

/// The error for a settings value this crate does not yet know.
pub(crate) fn unknown_knob(field: &str, value: &str) -> Error {
    Error::unsupported(format!(
        "the time layer does not know the {field} value {value} yet"
    ))
    .with_field(field)
}

/// The ceiling of a division of non-negative counts.
const fn ceil_div(numerator: i128, denominator: i128) -> i128 {
    (numerator + denominator - 1) / denominator
}

/// The grid instants are snapped to, in microseconds: a tenth of a
/// millisecond, twice the resolution of an `f64` Julian day in the
/// present era, so that an instant a vipala after sunrise counts as one
/// vipala and not as 399 960 microseconds.
#[allow(clippy::cast_precision_loss, reason = "a small constant")]
const GRID_MICROS: f64 = GRID as f64;
/// The grid in microseconds, as a count.
const GRID: i128 = 100;

/// Microseconds from one instant to another, snapped to the grid.
#[allow(
    clippy::cast_possible_truncation,
    reason = "a rounded microsecond count of at most a few days"
)]
fn micros_between(from: JulianDay<Utc>, to: JulianDay<Utc>) -> i128 {
    (((to.get() - from.get()) * 86_400e6 / GRID_MICROS).round() * GRID_MICROS) as i128
}

/// The ghati-pala of an instant in a local day.
///
/// # Errors
///
/// `OUT_OF_RANGE` for an instant outside the day (before its sunrise or
/// from the next sunrise on).
pub fn ghati_pala(
    day: &LocalDay,
    instant: JulianDay<Utc>,
    reckoning: Reckoning,
) -> Result<GhatiPala, Error> {
    if !day.contains(instant) {
        return Err(Error::new(
            Status::OutOfRange,
            format!(
                "{instant} is not in the local day from {} to {}",
                day.sunrise, day.next_sunrise
            ),
        )
        .with_field("instant"));
    }
    let elapsed = micros_between(day.sunrise, instant);
    let vipalas = match reckoning {
        Reckoning::Civil => elapsed / MICROS_PER_VIPALA,
        Reckoning::Proportional => {
            let daylight = micros_between(day.sunrise, day.sunset).max(1);
            let night = micros_between(day.sunset, day.next_sunrise).max(1);
            if instant.get() < day.sunset.get() {
                elapsed * VIPALAS_PER_HALF / daylight
            } else {
                VIPALAS_PER_HALF + micros_between(day.sunset, instant) * VIPALAS_PER_HALF / night
            }
        }
    };
    Ok(GhatiPala::from_vipalas(u32::try_from(vipalas).unwrap_or(0)))
}

/// The instant a ghati-pala names in a local day: the first instant with
/// that count, so that [`ghati_pala`] of it is the count again.
///
/// # Errors
///
/// A count beyond the day.
pub fn instant_of(
    day: &LocalDay,
    count: GhatiPala,
    reckoning: Reckoning,
) -> Result<JulianDay<Utc>, Error> {
    let vipalas = i128::from(count.total_vipalas());
    let exact = match reckoning {
        Reckoning::Civil => vipalas * MICROS_PER_VIPALA,
        Reckoning::Proportional => {
            let daylight = micros_between(day.sunrise, day.sunset).max(1);
            let night = micros_between(day.sunset, day.next_sunrise).max(1);
            if vipalas < VIPALAS_PER_HALF {
                ceil_div(vipalas * daylight, VIPALAS_PER_HALF)
            } else {
                daylight + ceil_div((vipalas - VIPALAS_PER_HALF) * night, VIPALAS_PER_HALF)
            }
        }
    };
    // The first grid instant at or after the count's start, so that the
    // forward conversion, which snaps to the grid, lands on the count.
    let micros = ceil_div(exact, GRID) * GRID;
    let whole_day = micros_between(day.sunrise, day.next_sunrise);
    if micros >= whole_day || micros > MICROS_PER_DAY * 2 {
        return Err(Error::new(
            Status::OutOfRange,
            format!("{count} is beyond the local day of {}", day.date),
        )
        .with_field("count"));
    }
    #[allow(
        clippy::cast_precision_loss,
        reason = "microseconds of at most two days"
    )]
    let days = micros as f64 / 86_400e6;
    Ok(day.sunrise.plus_days(days)?)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_calendar::CalendarDate;
    use teistro_core::catalogue::Calendar;
    use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};

    use super::*;
    use crate::local_day::DayState;

    fn day() -> LocalDay {
        let sunrise = JulianDay::try_new(2_460_000.75).unwrap();
        LocalDay {
            place: Place::new(
                Latitude::literal(27.7),
                Longitude::literal(85.3),
                Altitude::literal(0.0),
            ),
            date: CalendarDate::defined(Calendar::Gregorian, 2023, 2, 24),
            sunrise,
            sunset: sunrise.plus_days(11.5 / 24.0).unwrap(),
            next_sunrise: sunrise.plus_days(1.0 - 20.0 / 86_400.0).unwrap(),
            state: DayState::Normal,
            convention: teistro_core::settings::Sunrise::CentreNoRefraction.into(),
            model: String::from("test"),
        }
    }

    #[test]
    fn sunrise_is_zero_and_the_count_renders() {
        let d = day();
        let at_sunrise = ghati_pala(&d, d.sunrise, Reckoning::Civil).unwrap();
        assert_eq!(at_sunrise, GhatiPala::default());
        assert_eq!(at_sunrise.to_string(), "0-00-00");
        let one_vipala = d.sunrise.plus_days(0.4 / 86_400.0).unwrap();
        assert_eq!(
            ghati_pala(&d, one_vipala, Reckoning::Civil)
                .unwrap()
                .to_string(),
            "0-00-01"
        );
        // Six hours after sunrise: fifteen ghatis.
        let six_hours = d.sunrise.plus_days(0.25).unwrap();
        assert_eq!(
            ghati_pala(&d, six_hours, Reckoning::Civil).unwrap(),
            GhatiPala {
                ghati: 15,
                pala: 0,
                vipala: 0
            }
        );
        // Proportionally, sunset is thirty ghatis whatever the daylight.
        assert_eq!(
            ghati_pala(&d, d.sunset, Reckoning::Proportional).unwrap(),
            GhatiPala {
                ghati: 30,
                pala: 0,
                vipala: 0
            }
        );
        assert_eq!(
            Reckoning::try_from(GhatiReckoning::Proportional).unwrap(),
            Reckoning::Proportional
        );
        assert!(ghati_pala(&d, d.next_sunrise, Reckoning::Civil).is_err());
        assert!(ghati_pala(&d, d.sunrise.plus_days(-1.0).unwrap(), Reckoning::Civil).is_err());
        assert!(
            instant_of(
                &d,
                GhatiPala {
                    ghati: 60,
                    pala: 0,
                    vipala: 0
                },
                Reckoning::Civil
            )
            .is_err()
        );
        let json = serde_json::to_string(&at_sunrise).unwrap();
        assert_eq!(
            serde_json::from_str::<GhatiPala>(&json).unwrap(),
            at_sunrise
        );
    }

    #[test]
    fn every_vipala_of_the_day_round_trips_in_both_reckonings() {
        let d = day();
        for reckoning in [Reckoning::Civil, Reckoning::Proportional] {
            let mut checked = 0;
            for vipalas in 0..216_000u32 {
                let count = GhatiPala::from_vipalas(vipalas);
                assert_eq!(count.total_vipalas(), vipalas);
                let Ok(instant) = instant_of(&d, count, reckoning) else {
                    // Civil counts beyond the day's length are refused.
                    assert!(
                        reckoning == Reckoning::Civil && vipalas > 215_900,
                        "{count} {reckoning:?}"
                    );
                    continue;
                };
                assert_eq!(
                    ghati_pala(&d, instant, reckoning).unwrap(),
                    count,
                    "{reckoning:?} {count}"
                );
                checked += 1;
            }
            assert!(checked >= 215_900, "{reckoning:?}: {checked}");
        }
    }

    #[test]
    fn instants_map_to_the_count_that_started_before_them() {
        let d = day();
        for step in 0..8_640u32 {
            let instant = d
                .sunrise
                .plus_days(f64::from(step) * 10.0 / 86_400.0)
                .unwrap();
            if !d.contains(instant) {
                break;
            }
            for reckoning in [Reckoning::Civil, Reckoning::Proportional] {
                let count = ghati_pala(&d, instant, reckoning).unwrap();
                let start = instant_of(&d, count, reckoning).unwrap();
                assert!(start.get() <= instant.get() + 1e-9, "{reckoning:?} {count}");
                let next = GhatiPala::from_vipalas(count.total_vipalas() + 1);
                if let Ok(after) = instant_of(&d, next, reckoning) {
                    assert!(after.get() > instant.get() - 1e-9, "{reckoning:?} {count}");
                }
            }
        }
    }
}
