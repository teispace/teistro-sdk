//! The day count from the epoch (ahargana) and the mean places: a body's
//! revolutions in its cycle times the days elapsed, the whole days in
//! exact integer arithmetic (I.29 to 34, I.41 to 44, I.48 to 53).

use core::fmt;

use crate::params::Parameters;

/// The cycle a revolution count is given for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Cycle {
    /// An age of 4 320 000 years.
    Yuga,
    /// An aeon of a thousand ages.
    Kalpa,
}

/// A mean motion: whole revolutions in a cycle, direct or retrograde.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Motion {
    /// Whole revolutions in the cycle.
    pub revolutions: u64,
    /// The cycle.
    pub cycle: Cycle,
    /// Whether the motion runs against the order of the signs.
    pub retrograde: bool,
}

impl Motion {
    /// A direct motion.
    #[must_use]
    pub const fn direct(revolutions: u64, cycle: Cycle) -> Motion {
        Motion {
            revolutions,
            cycle,
            retrograde: false,
        }
    }

    /// A retrograde motion (the nodes).
    #[must_use]
    pub const fn retrograde(revolutions: u64, cycle: Cycle) -> Motion {
        Motion {
            revolutions,
            cycle,
            retrograde: true,
        }
    }

    /// The mean motion in degrees per civil day, signed.
    #[must_use]
    #[allow(clippy::cast_precision_loss, reason = "counts far below 2^53")]
    pub fn degrees_per_day(self, params: &Parameters) -> f64 {
        let rate = self.revolutions as f64 * 360.0 / params.cycle_days(self.cycle) as f64;
        if self.retrograde { -rate } else { rate }
    }
}

/// The days elapsed since the epoch: the whole days, which the mean-place
/// arithmetic handles exactly, and the fraction of the current day.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ahargana {
    /// Whole civil days since the epoch; negative before it.
    pub days: i64,
    /// The fraction of the day elapsed, in `[0, 1)`.
    pub fraction: f64,
}

impl Ahargana {
    /// The count at a Universal Time instant.
    #[must_use]
    #[allow(
        clippy::cast_possible_truncation,
        reason = "the floor of a day count inside i64"
    )]
    pub fn at(jd_ut: f64, params: &Parameters) -> Ahargana {
        let elapsed = jd_ut - params.epoch_jd_ut;
        let days = elapsed.floor();
        Ahargana {
            days: days as i64,
            fraction: elapsed - days,
        }
    }

    /// The count as one number of days.
    #[must_use]
    #[allow(clippy::cast_precision_loss, reason = "day counts far below 2^53")]
    pub fn total(self) -> f64 {
        self.days as f64 + self.fraction
    }

    /// The count `days` later.
    #[must_use]
    pub fn plus(self, days: f64) -> Ahargana {
        let total = self.fraction + days;
        let whole = total.floor();
        #[allow(clippy::cast_possible_truncation, reason = "a floored day count")]
        let shift = whole as i64;
        Ahargana {
            days: self.days.saturating_add(shift),
            fraction: total - whole,
        }
    }

    /// The mean place of a motion at this count, degrees in `[0, 360)`:
    /// the days since the aeon began (the text's elapsed days plus this
    /// count) times the revolutions, modulo the cycle, the whole days in
    /// integer arithmetic and only the current day's fraction in floating
    /// point.
    #[must_use]
    #[allow(
        clippy::cast_precision_loss,
        reason = "a residue below the cycle's days"
    )]
    pub fn mean_degrees(self, motion: Motion, params: &Parameters) -> f64 {
        let cycle = i128::from(params.cycle_days(motion.cycle));
        let elapsed = i128::from(params.elapsed_days_at_kali) + i128::from(self.days);
        let whole = (elapsed * i128::from(motion.revolutions)).rem_euclid(cycle);
        let turns = (whole as f64 + self.fraction * motion.revolutions as f64) / cycle as f64;
        let turns = turns.rem_euclid(1.0);
        let turns = if motion.retrograde {
            (1.0 - turns).rem_euclid(1.0)
        } else {
            turns
        };
        (turns * 360.0).rem_euclid(360.0)
    }
}

impl fmt::Display for Ahargana {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} days + {:.6}", self.days, self.fraction)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;
    use crate::params::Planet;

    const TEXT: &Parameters = &Parameters::TEXT;

    #[test]
    fn at_the_epoch_the_planets_are_at_the_first_of_mesha() {
        // I.57: at the start of the Kali age the mean planets are at the
        // first of Mesha; their apsides and nodes are not.
        let epoch = Ahargana::at(TEXT.epoch_jd_ut, TEXT);
        assert_eq!(epoch.days, 0);
        assert!(epoch.fraction.abs() < 1e-9);
        for planet in Planet::ALL {
            assert!(
                epoch.mean_degrees(TEXT.motion(planet), TEXT).abs() < 1e-9,
                "{planet}"
            );
        }
        // Burgess's notes on I.41 to 44: the Sun's apsis at 77°7′48″, the
        // Moon's apsis at 90°, the Moon's node at 180°.
        let sun_apsis = epoch.mean_degrees(TEXT.apsis(Planet::Sun), TEXT);
        assert!((sun_apsis - (77.0 + 7.0 / 60.0 + 48.0 / 3600.0)).abs() < 1e-9);
        assert!((epoch.mean_degrees(TEXT.moon_apsis, TEXT) - 90.0).abs() < 1e-9);
        assert!((epoch.mean_degrees(TEXT.moon_node, TEXT) - 180.0).abs() < 1e-9);
    }

    #[test]
    fn a_whole_age_later_everything_returns() {
        #[allow(clippy::cast_precision_loss, reason = "test arithmetic")]
        let later = Ahargana::at(TEXT.epoch_jd_ut + TEXT.yuga_civil_days as f64, TEXT);
        for planet in Planet::ALL {
            let deg = later.mean_degrees(TEXT.motion(planet), TEXT);
            assert!(deg < 1e-6 || deg > 360.0 - 1e-6, "{planet}: {deg}");
        }
        assert_eq!(later.days, 1_577_917_828);
    }

    #[test]
    fn mean_motion_and_the_thesis_ahargana() {
        // 4 320 000 revolutions in 1 577 917 828 days: 59′8″ a day.
        let sun = TEXT.motion(Planet::Sun).degrees_per_day(TEXT);
        assert!((sun * 60.0 - (59.0 + 8.0 / 60.0)).abs() < 0.01);
        let node = TEXT.moon_node.degrees_per_day(TEXT);
        assert!(node < 0.0 && (node * 60.0 + 3.18).abs() < 0.01);
        // A hand computation of the tradition reaches the mean Sun at
        // 6 signs 15°46′30″ for what it calls a count of 1 861 191 days;
        // the text's arithmetic gives that place one day later, at
        // 1 861 192: the tradition counts the current day as elapsed.
        let count = Ahargana {
            days: 1_861_191,
            fraction: 0.0,
        };
        let mean_sun = count.mean_degrees(TEXT.motion(Planet::Sun), TEXT);
        assert!((mean_sun - 194.790).abs() < 0.002, "{mean_sun}");
        let next = count.plus(1.0).mean_degrees(TEXT.motion(Planet::Sun), TEXT);
        assert!((next - 195.775_9).abs() < 0.001, "{next}");
        assert_eq!(
            count.plus(1.75),
            Ahargana {
                days: 1_861_192,
                fraction: 0.75
            }
        );
        assert!((count.plus(-0.25).total() - 1_861_190.75).abs() < 1e-9);
        assert_eq!(count.to_string(), "1861191 days + 0.000000");
    }

    #[test]
    fn before_the_epoch_the_count_is_negative_and_places_still_wrap() {
        let before = Ahargana::at(TEXT.epoch_jd_ut - 10.5, TEXT);
        assert_eq!(before.days, -11);
        assert!((before.fraction - 0.5).abs() < 1e-9);
        let sun = before.mean_degrees(TEXT.motion(Planet::Sun), TEXT);
        assert!((sun - (360.0 - 10.5 * 0.985_602_6)).abs() < 1e-4, "{sun}");
    }
}
