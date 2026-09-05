//! The planetary hours (horas): twenty-four in a sunrise-anchored day,
//! each ruled by a graha, the first by the day's lord and each next by
//! the lord of the sixth weekday from it, which runs the seven in the
//! order Sun, Venus, Mercury, Moon, Saturn, Jupiter, Mars (the order of
//! decreasing orbital period the Chaldean tradition names). Under the
//! proportional reckoning twelve horas span the daylight and twelve the
//! night, as the muhurta texts define them; under the equal reckoning a
//! hora is sixty minutes from sunrise (`docs/03-design/time-and-timezone.md`,
//! §3.3 and §4). The baseline engine's fixtures follow the proportional
//! reckoning (53 of 55 charts; the two others are its own day-early and
//! polar cases).

use core::fmt;

use teistro_core::catalogue::{Graha, Vara};
use teistro_core::error::{Error, Status};
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::settings::HoraReckoning;

use crate::ghati::unknown_knob;
use crate::local_day::LocalDay;

/// Horas in a day.
pub const HORAS_PER_DAY: u8 = 24;
/// Horas in each half of the day.
const HORAS_PER_HALF: u8 = 12;
/// The lord of a hora is the lord of the weekday this many days on from
/// the previous hora's: the sixth from it, counting the day itself.
const WEEKDAY_STEP: u16 = 5;

/// How the horas are counted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Reckoning {
    /// Twelve horas over the daylight and twelve over the night.
    Proportional,
    /// Twenty-four horas of sixty minutes from sunrise.
    Equal,
}

impl TryFrom<HoraReckoning> for Reckoning {
    type Error = Error;

    /// The reckoning a settings knob names.
    ///
    /// # Errors
    ///
    /// A knob value core added before this crate learnt it.
    fn try_from(knob: HoraReckoning) -> Result<Reckoning, Error> {
        match knob {
            HoraReckoning::Proportional => Ok(Reckoning::Proportional),
            HoraReckoning::Equal => Ok(Reckoning::Equal),
            other => Err(unknown_knob("day.hora_reckoning", &format!("{other:?}"))),
        }
    }
}

/// One hora of a day.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct Hora {
    /// The hora's number, 1 to 24 from sunrise.
    pub number: u8,
    /// The graha that rules it.
    pub lord: Graha,
    /// When it begins.
    pub start: JulianDay<Utc>,
    /// When it ends: the next hora's start.
    pub end: JulianDay<Utc>,
}

impl Hora {
    /// Whether the hora is one of the twelve of the daylight.
    #[must_use]
    pub const fn is_daytime(&self) -> bool {
        self.number <= HORAS_PER_HALF
    }
}

impl fmt::Display for Hora {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "hora {} of {} from {} to {}",
            self.number,
            self.lord.key(),
            self.start,
            self.end
        )
    }
}

/// The lord of a day's hora by its number, from the day's vara: the
/// first hora's lord is the day's, each next the lord of the sixth
/// weekday on.
#[must_use]
pub fn lord_of(vara: Vara, number: u8) -> Graha {
    let steps = u16::from(number.saturating_sub(1)) * WEEKDAY_STEP;
    let weekday = (u16::from(vara.attributes().weekday) + steps) % 7;
    Vara::ALL
        .iter()
        .find(|v| u16::from(v.attributes().weekday) == weekday)
        .map_or(vara.attributes().lord, |v| v.attributes().lord)
}

/// The boundaries of the horas of a day: twenty-five instants from the
/// sunrise to the next sunrise.
fn boundaries(day: &LocalDay, reckoning: Reckoning) -> [f64; 25] {
    let mut bounds = [0.0; 25];
    let sunrise = day.sunrise.get();
    for (index, bound) in bounds.iter_mut().enumerate() {
        #[allow(clippy::cast_precision_loss, reason = "an index below twenty-five")]
        let n = index as f64;
        *bound = match reckoning {
            Reckoning::Equal => sunrise + n / 24.0,
            Reckoning::Proportional => {
                let half = f64::from(HORAS_PER_HALF);
                if n <= half {
                    sunrise + (day.sunset.get() - sunrise) * n / half
                } else {
                    day.sunset.get()
                        + (day.next_sunrise.get() - day.sunset.get()) * (n - half) / half
                }
            }
        };
    }
    bounds
}

/// The twenty-four horas of a local day.
///
/// # Errors
///
/// A day whose boundaries are not instants (never for a day the local
/// day computed).
pub fn horas(day: &LocalDay, reckoning: Reckoning) -> Result<Vec<Hora>, Error> {
    let vara = day_vara(day);
    let bounds = boundaries(day, reckoning);
    bounds
        .windows(2)
        .enumerate()
        .map(|(index, pair)| {
            let (Some(start), Some(end)) = (pair.first(), pair.get(1)) else {
                return Err(Error::internal("a window of two has two ends"));
            };
            let number = u8::try_from(index + 1).unwrap_or(HORAS_PER_DAY);
            Ok(Hora {
                number,
                lord: lord_of(vara, number),
                start: JulianDay::try_new(*start)?,
                end: JulianDay::try_new(*end)?,
            })
        })
        .collect()
}

/// The hora an instant falls in.
///
/// # Errors
///
/// `OUT_OF_RANGE` for an instant outside the day (before its sunrise or
/// from the next sunrise on).
pub fn hora_at(
    day: &LocalDay,
    instant: JulianDay<Utc>,
    reckoning: Reckoning,
) -> Result<Hora, Error> {
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
    let bounds = boundaries(day, reckoning);
    let t = instant.get();
    // The last boundary at or before the instant; the day contains it, so
    // one exists and it is not the final one.
    let index = bounds
        .iter()
        .take(usize::from(HORAS_PER_DAY))
        .rposition(|bound| *bound <= t)
        .unwrap_or(0);
    let number = u8::try_from(index + 1).unwrap_or(HORAS_PER_DAY);
    let (Some(start), Some(end)) = (bounds.get(index), bounds.get(index + 1)) else {
        return Err(Error::internal(
            "a hora index inside the day has both bounds",
        ));
    };
    Ok(Hora {
        number,
        lord: lord_of(day_vara(day), number),
        start: JulianDay::try_new(*start)?,
        end: JulianDay::try_new(*end)?,
    })
}

/// The vara of a local day: the weekday of the civil date the day is
/// named by, which the sunrise-anchored reckoning keeps.
fn day_vara(day: &LocalDay) -> Vara {
    day.vara
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and read fixed tables"
    )]

    use teistro_calendar::solar::DayArc;
    use teistro_calendar::{CalendarDate, FixedDay, Gregorian};
    use teistro_core::catalogue::Calendar;
    use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
    use teistro_core::settings::{PolarDayPolicy, Sunrise};
    use teistro_core::time::UtcOffset;

    use super::*;
    use crate::local_day::{DayState, local_day};

    #[test]
    fn the_lords_run_the_chaldean_order_from_the_days_lord() {
        // A Sunday: Sun, Venus, Mercury, Moon, Saturn, Jupiter, Mars, then
        // the Sun again; the twenty-fifth would be the Moon, Monday's lord,
        // which is what makes the next day's first hora its own lord's.
        let sunday = [
            Graha::Sun,
            Graha::Venus,
            Graha::Mercury,
            Graha::Moon,
            Graha::Saturn,
            Graha::Jupiter,
            Graha::Mars,
        ];
        for number in 1..=HORAS_PER_DAY {
            let expected = sunday[usize::from((number - 1) % 7)];
            assert_eq!(lord_of(Vara::Ravivara, number), expected, "hora {number}");
        }
        assert_eq!(lord_of(Vara::Ravivara, 25), Graha::Moon);
        for vara in Vara::ALL {
            assert_eq!(lord_of(vara, 1), vara.attributes().lord);
            let next = Vara::ALL[(usize::from(vara.attributes().weekday) + 1) % 7];
            assert_eq!(lord_of(vara, 25), next.attributes().lord, "{vara}");
        }
    }

    /// A model whose Sun rises at 06:00 and sets at 18:00 UTC of any day.
    struct SixToSix;

    impl teistro_calendar::solar::SolarModel for SixToSix {
        fn sidereal_sun_deg(&self, _jd_ut: f64) -> Result<f64, Error> {
            Ok(0.0)
        }

        fn day_light(
            &self,
            day: FixedDay,
            _place: &Place,
        ) -> Result<teistro_calendar::solar::DayLight, Error> {
            let midnight = day.jd_at_midnight()?;
            Ok(teistro_calendar::solar::DayLight::Arc(DayArc {
                sunrise: midnight.plus_days(0.25)?,
                sunset: midnight.plus_days(0.75)?,
            }))
        }

        fn describe(&self) -> String {
            String::from("six to six")
        }

        fn convention(&self) -> teistro_core::settings::SunriseConvention {
            Sunrise::CentreNoRefraction.into()
        }
    }

    #[test]
    fn horas_tile_the_day_and_answer_for_an_instant() {
        let place = Place::new(
            Latitude::literal(0.0),
            Longitude::literal(0.0),
            Altitude::literal(0.0),
        );
        // 2024-06-23 is a Sunday.
        let date = CalendarDate::defined(Calendar::Gregorian, 2024, 6, 23);
        let day = local_day(
            &SixToSix,
            &Gregorian,
            &UtcOffset::UTC,
            &place,
            &date,
            PolarDayPolicy::Undefined,
        )
        .unwrap();
        assert_eq!(day.state, DayState::Normal);
        assert_eq!(day.vara, Vara::Ravivara);
        for reckoning in [Reckoning::Proportional, Reckoning::Equal] {
            let all = horas(&day, reckoning).unwrap();
            assert_eq!(all.len(), 24);
            assert_eq!(all[0].start, day.sunrise);
            assert_eq!(all[23].end, day.next_sunrise);
            assert!(all.windows(2).all(|w| w[0].end == w[1].start));
            assert_eq!(all[0].lord, Graha::Sun);
            assert_eq!(all[1].lord, Graha::Venus);
            assert!(all[11].is_daytime() && !all[12].is_daytime());
            // A twelve-hour day makes the two reckonings agree.
            let at = hora_at(&day, day.sunrise.plus_days(13.5 / 24.0).unwrap(), reckoning).unwrap();
            assert_eq!(at.number, 14);
            assert_eq!(at.lord, lord_of(Vara::Ravivara, 14));
            assert_eq!(at, all[13]);
            assert!(at.to_string().starts_with("hora 14 of"));
        }
        let first = hora_at(&day, day.sunrise, Reckoning::Equal).unwrap();
        assert_eq!(first.number, 1);
        assert!(hora_at(&day, day.next_sunrise, Reckoning::Equal).is_err());
        assert!(hora_at(&day, day.sunrise.plus_days(-0.1).unwrap(), Reckoning::Equal).is_err());
        assert_eq!(
            Reckoning::try_from(HoraReckoning::Equal).unwrap(),
            Reckoning::Equal
        );
        assert_eq!(
            Reckoning::try_from(HoraReckoning::Proportional).unwrap(),
            Reckoning::Proportional
        );
    }
}
