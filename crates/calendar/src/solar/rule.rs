//! The month-start rules: which civil day a sankranti begins the month
//! on. Each variant is a row with the tradition that follows it; the
//! fitted threshold is the row a measurement chooses.

use core::fmt;

use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_core::time::LocalClock;

use crate::fixed::FixedDay;
use crate::solar::{DayArc, SolarModel};

/// Which civil day a sankranti begins the month on.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MonthStartRule {
    /// The civil day, midnight to midnight, in which the sankranti falls
    /// (the Orissa rule: Sewell and Dikshit, *The Indian Calendar*, 1896,
    /// §28; the Calendar Reform Committee's report, 1955, part C).
    SankrantiDay,
    /// The civil day after the one in which the sankranti falls (the
    /// Bengal rule, same sources: from sunrise to midnight the following
    /// day, after midnight the day after next, which is the same civil
    /// day after the sankranti's).
    FollowingDay,
    /// The civil day in which the sankranti moved by so many days falls:
    /// no tradition's rule but the family every uniform convention
    /// belongs to (a cutoff at a fraction `f` of the day is a shift of
    /// `1 - f`), the row a measurement against a published table scans.
    Shifted {
        /// The shift in days, later when positive.
        days: f64,
    },
    /// The day whose span from its sunrise to the next sunrise holds the
    /// sankranti: the almanac day, so a sankranti before dawn belongs to
    /// the day before.
    SunriseToSunrise,
    /// The sankranti's day when it falls before sunset, the next day after
    /// sunset (the Tamil rule, same sources).
    BeforeSunset,
    /// The sankranti's day when it falls before three fifths of the
    /// daylight have passed (the start of aparahna), the next day after
    /// (the Malabar rule, same sources).
    BeforeAparahna,
    /// The sankranti's day by its punya-kala, the Dharmasindhu's rule for
    /// observing a sankranti (first pariccheda, the sankranti section): a
    /// sankranti in the first half of the night belongs to the day that
    /// ended, in the second half to the day beginning, which is the civil
    /// day; except the two ayana sankrantis, whose punya-kala falls on one
    /// side only: a Karka sankranti at night is observed on the preceding
    /// day (its afternoon), a Makara sankranti at night on the following
    /// day (its forenoon). So Karka follows the sunrise-to-sunrise day,
    /// Makara the before-sunset rule, and the ten others the civil day.
    /// Nepal's official calendar follows this rule
    /// (`docs/calendars/bikram-sambat.md`).
    Punyakala,
}

impl MonthStartRule {
    /// The rows a tradition names, for a measurement to run over.
    pub const NAMED: [MonthStartRule; 6] = [
        MonthStartRule::SankrantiDay,
        MonthStartRule::FollowingDay,
        MonthStartRule::SunriseToSunrise,
        MonthStartRule::BeforeSunset,
        MonthStartRule::BeforeAparahna,
        MonthStartRule::Punyakala,
    ];

    /// The sign of the Karka sankranti, 0 for Mesha.
    pub const KARKA: u8 = 3;
    /// The sign of the Makara sankranti.
    pub const MAKARA: u8 = 9;

    /// Whether the rule needs sunrise or sunset.
    #[must_use]
    pub const fn needs_day_arc(self) -> bool {
        matches!(
            self,
            MonthStartRule::SunriseToSunrise
                | MonthStartRule::BeforeSunset
                | MonthStartRule::BeforeAparahna
                | MonthStartRule::Punyakala
        )
    }

    /// The row's key for stamps and reports.
    #[must_use]
    pub fn key(self) -> String {
        match self {
            MonthStartRule::SankrantiDay => String::from("SANKRANTI_DAY"),
            MonthStartRule::FollowingDay => String::from("FOLLOWING_DAY"),
            MonthStartRule::Shifted { days } => format!("SHIFTED {days:+.3}"),
            MonthStartRule::SunriseToSunrise => String::from("SUNRISE_TO_SUNRISE"),
            MonthStartRule::BeforeSunset => String::from("BEFORE_SUNSET"),
            MonthStartRule::BeforeAparahna => String::from("BEFORE_APARAHNA"),
            MonthStartRule::Punyakala => String::from("PUNYAKALA"),
        }
    }

    /// The civil day the month begins on for the sankranti into a sign
    /// (0 for Mesha), under a clock, with the model and the place for the
    /// rules that need the day's arc.
    ///
    /// # Errors
    ///
    /// The model's refusal, or a rule that needs a sunrise on a day
    /// without one.
    pub fn month_start(
        self,
        sign: u8,
        sankranti: JulianDay<Utc>,
        clock: &dyn LocalClock,
        model: &dyn SolarModel,
        place: &Place,
    ) -> Result<FixedDay, Error> {
        let (day, _) = local_day(clock, sankranti);
        Ok(match self {
            MonthStartRule::SankrantiDay => day,
            MonthStartRule::FollowingDay => day.plus_days(1),
            MonthStartRule::Shifted { days } => local_day(clock, sankranti.plus_days(days)?).0,
            MonthStartRule::Punyakala => {
                let row = match sign {
                    MonthStartRule::KARKA => MonthStartRule::SunriseToSunrise,
                    MonthStartRule::MAKARA => MonthStartRule::BeforeSunset,
                    _ => MonthStartRule::SankrantiDay,
                };
                return row.month_start(sign, sankranti, clock, model, place);
            }
            MonthStartRule::SunriseToSunrise => {
                let arc = arc_of(model, day, place)?;
                if sankranti.get() < arc.sunrise.get() {
                    day.plus_days(-1)
                } else {
                    day
                }
            }
            MonthStartRule::BeforeSunset => {
                let arc = arc_of(model, day, place)?;
                if sankranti.get() < arc.sunset.get() {
                    day
                } else {
                    day.plus_days(1)
                }
            }
            MonthStartRule::BeforeAparahna => {
                let arc = arc_of(model, day, place)?;
                if sankranti.get() < arc.at_fraction(0.6) {
                    day
                } else {
                    day.plus_days(1)
                }
            }
        })
    }
}

impl fmt::Display for MonthStartRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.key())
    }
}

/// The civil day an instant falls in under a clock, and the fraction of
/// that day elapsed.
#[must_use]
pub fn local_day(clock: &dyn LocalClock, instant: JulianDay<Utc>) -> (FixedDay, f64) {
    FixedDay::from_local_jd(clock.local_jd(instant))
}

fn arc_of(model: &dyn SolarModel, day: FixedDay, place: &Place) -> Result<DayArc, Error> {
    model.day_arc(day, place)?.ok_or_else(|| {
        Error::unsupported(format!(
            "the month-start rule needs a sunrise on {day} at {place}, where the Sun does not rise"
        ))
        .with_hint("use a rule that does not depend on the day's arc")
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_core::quantity::{Altitude, Latitude, Longitude};
    use teistro_core::time::UtcOffset;

    use super::*;

    /// A model whose Sun rises at 06:00 and sets at 18:00 UTC of any day.
    struct SixToSix;

    impl SolarModel for SixToSix {
        fn sidereal_sun_deg(&self, _jd_ut: f64) -> Result<f64, Error> {
            Ok(0.0)
        }

        fn day_arc(&self, day: FixedDay, _place: &Place) -> Result<Option<DayArc>, Error> {
            let midnight = day.jd_at_midnight()?;
            Ok(Some(DayArc {
                sunrise: midnight.plus_days(0.25)?,
                sunset: midnight.plus_days(0.75)?,
            }))
        }

        fn describe(&self) -> String {
            String::from("six to six")
        }
    }

    fn at(day: FixedDay, hours: f64) -> JulianDay<Utc> {
        day.jd_at_midnight()
            .unwrap()
            .plus_days(hours / 24.0)
            .unwrap()
    }

    #[test]
    fn every_rule_places_a_sankranti_as_its_tradition_says() {
        use MonthStartRule as R;
        let day = FixedDay::new(738_000);
        let place = Place::new(
            Latitude::literal(0.0),
            Longitude::literal(0.0),
            Altitude::literal(0.0),
        );
        let utc = UtcOffset::UTC;
        let start_of = |rule: MonthStartRule, sign: u8, hours: f64| {
            day.days_until(
                rule.month_start(sign, at(day, hours), &utc, &SixToSix, &place)
                    .unwrap(),
            )
        };
        let start = |rule: MonthStartRule, hours: f64| start_of(rule, 0, hours);
        // Morning (03:00), afternoon (15:00), evening (20:00).
        assert_eq!(
            [
                start(R::SankrantiDay, 3.0),
                start(R::SankrantiDay, 15.0),
                start(R::SankrantiDay, 20.0)
            ],
            [0, 0, 0]
        );
        assert_eq!(
            [start(R::FollowingDay, 3.0), start(R::FollowingDay, 20.0)],
            [1, 1]
        );
        // A cutoff at 0.705 of the day is a shift of 0.295 days.
        let cutoff = R::Shifted { days: 0.295 };
        assert_eq!(
            [start(cutoff, 3.0), start(cutoff, 16.0), start(cutoff, 17.0)],
            [0, 0, 1]
        );
        assert_eq!(
            [
                start(R::Shifted { days: -0.25 }, 3.0),
                start(R::Shifted { days: -0.25 }, 9.0)
            ],
            [-1, 0]
        );
        assert_eq!(
            [
                start(R::SunriseToSunrise, 3.0),
                start(R::SunriseToSunrise, 15.0),
                start(R::SunriseToSunrise, 20.0)
            ],
            [-1, 0, 0]
        );
        assert_eq!(
            [
                start(R::BeforeSunset, 3.0),
                start(R::BeforeSunset, 15.0),
                start(R::BeforeSunset, 20.0)
            ],
            [0, 0, 1]
        );
        // Aparahna begins at three fifths of the daylight: 13:12.
        assert_eq!(
            [
                start(R::BeforeAparahna, 13.0),
                start(R::BeforeAparahna, 13.5)
            ],
            [0, 1]
        );
        // The punya-kala rule: Karka at night to the day before, Makara
        // after sunset to the day after, the rest the civil day.
        assert_eq!(
            [
                start_of(R::Punyakala, R::KARKA, 3.0),
                start_of(R::Punyakala, R::KARKA, 20.0)
            ],
            [-1, 0]
        );
        assert_eq!(
            [
                start_of(R::Punyakala, R::MAKARA, 3.0),
                start_of(R::Punyakala, R::MAKARA, 20.0)
            ],
            [0, 1]
        );
        assert_eq!(
            [
                start_of(R::Punyakala, 0, 3.0),
                start_of(R::Punyakala, 6, 20.0)
            ],
            [0, 0]
        );
        assert!(
            R::BeforeSunset.needs_day_arc()
                && R::Punyakala.needs_day_arc()
                && !R::FollowingDay.needs_day_arc()
        );
        assert_eq!(cutoff.to_string(), "SHIFTED +0.295");
        assert_eq!(R::NAMED.len(), 6);
        let json = serde_json::to_string(&cutoff).unwrap();
        assert_eq!(json, "{\"kind\":\"SHIFTED\",\"days\":0.295}");
        assert_eq!(
            serde_json::from_str::<MonthStartRule>(&json).unwrap(),
            cutoff
        );
        assert_eq!(
            serde_json::to_string(&R::Punyakala).unwrap(),
            "{\"kind\":\"PUNYAKALA\"}"
        );
    }

    #[test]
    fn the_clock_decides_the_civil_day() {
        // 23:00 UTC is the next day at +05:45.
        let day = FixedDay::new(738_000);
        let instant = at(day, 23.0);
        let (utc_day, utc_fraction) = local_day(&UtcOffset::UTC, instant);
        assert_eq!(utc_day, day);
        assert!((utc_fraction - 23.0 / 24.0).abs() < 1e-9);
        let (nepal_day, nepal_fraction) = local_day(&UtcOffset::literal(5, 45, 0), instant);
        assert_eq!(nepal_day, day.plus_days(1));
        assert!((nepal_fraction - 4.75 / 24.0).abs() < 1e-9);
    }

    #[test]
    fn a_rule_that_needs_a_sunrise_refuses_a_polar_day() {
        struct Polar;
        impl SolarModel for Polar {
            fn sidereal_sun_deg(&self, _jd_ut: f64) -> Result<f64, Error> {
                Ok(0.0)
            }
            fn day_arc(&self, _day: FixedDay, _place: &Place) -> Result<Option<DayArc>, Error> {
                Ok(None)
            }
            fn describe(&self) -> String {
                String::from("polar")
            }
        }
        let day = FixedDay::new(738_000);
        let place = Place::new(
            Latitude::literal(80.0),
            Longitude::literal(0.0),
            Altitude::literal(0.0),
        );
        let error = MonthStartRule::BeforeSunset
            .month_start(0, at(day, 12.0), &UtcOffset::UTC, &Polar, &place)
            .unwrap_err();
        assert!(error.message.contains("does not rise"));
        assert!(
            MonthStartRule::SankrantiDay
                .month_start(0, at(day, 12.0), &UtcOffset::UTC, &Polar, &place)
                .is_ok()
        );
        // The punya-kala rule needs the arc only at the ayana sankrantis.
        assert!(
            MonthStartRule::Punyakala
                .month_start(0, at(day, 12.0), &UtcOffset::UTC, &Polar, &place)
                .is_ok()
        );
        assert!(
            MonthStartRule::Punyakala
                .month_start(
                    MonthStartRule::MAKARA,
                    at(day, 12.0),
                    &UtcOffset::UTC,
                    &Polar,
                    &place
                )
                .is_err()
        );
    }
}
