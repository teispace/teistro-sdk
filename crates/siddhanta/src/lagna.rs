//! The horoscope point (lagna): the times of rising of the signs at a
//! latitude (III.42 to 45), the point of the ecliptic on the eastern
//! horizon at a time counted from sunrise (III.46 to 48), the point on
//! the meridian from the Sun's hour angle (III.49), and the time at which
//! a given point rises (III.50 to 51).
//!
//! The text measures these in respirations (asu): a minute of arc of the
//! equator's rotation, 21 600 to the day, six to a vinadi and three
//! hundred and sixty to a nadi. Every arc within a sign is taken
//! proportional to the sign's rising time, as the text does (Burgess's
//! note under III.46 to 49 records that this is the text's own
//! approximation).

use core::fmt;

use teistro_core::quantity::Latitude;

use crate::model::SuryaSiddhanta;
use crate::params::Parameters;

/// Respirations in a day: the equator's rotation in minutes of arc.
pub const ASU_PER_DAY: f64 = 21_600.0;
/// The most signs a walk visits before it is a defect: a full circle
/// and the sign the walk began in, twice over for a time past a day.
const WALK_CAP: usize = 26;

/// The times of rising of the twelve signs, Mesha to Meena, in
/// respirations: at Lanka (the equator) the right ascensions of the
/// signs, elsewhere the oblique ascensions.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct RisingTimes {
    /// The twelve times, Mesha first.
    pub asu: [f64; 12],
}

impl RisingTimes {
    /// The times at Lanka (III.44): the three of the first quadrant, then
    /// the same three inverted for the second, and those six inverted for
    /// the other half.
    #[must_use]
    pub fn lanka(params: &Parameters) -> RisingTimes {
        let [first, second, third] = params.lanka_rising_asu.map(f64::from);
        RisingTimes {
            asu: [
                first, second, third, third, second, first, first, second, third, third, second,
                first,
            ],
        }
    }

    /// The times at a latitude (III.44 to 45): the three of the first
    /// quadrant each diminished by its portion of the ascensional
    /// difference (the difference at the end of the sign less that at the
    /// end of the sign before, from the model's declination and
    /// ascensional-difference rules), the second quadrant's the inverted
    /// three increased by the inverted portions, and the other half the
    /// same six inverted. `None` where the Sun neither rises nor sets at
    /// the end of a sign, which is where the text's rule has no answer.
    #[must_use]
    pub fn at(model: &SuryaSiddhanta, latitude: Latitude) -> Option<RisingTimes> {
        let cara = |tropical_deg: f64| -> Option<f64> {
            model.ascensional_difference_deg(latitude, model.declination_deg(tropical_deg))
        };
        let at_30 = cara(30.0)?;
        let at_60 = cara(60.0)?;
        let at_90 = cara(90.0)?;
        // Degrees of ascensional difference are minutes of arc of the
        // equator sixty times over: respirations.
        let portions = [at_30 * 60.0, (at_60 - at_30) * 60.0, (at_90 - at_60) * 60.0];
        let lanka = RisingTimes::lanka(model.parameters()).asu;
        let mut asu = [0.0; 12];
        for i in 0..3 {
            let (Some(base), Some(portion)) = (lanka.get(i), portions.get(i)) else {
                return None;
            };
            let first_quadrant = base - portion;
            let second_quadrant = base + portion;
            // Mesha, Vrishabha, Mithuna; Karka, Simha, Kanya as the
            // inverted three; Tula to Meena as the six inverted.
            for (index, value) in [
                (i, first_quadrant),
                (5 - i, second_quadrant),
                (6 + i, second_quadrant),
                (11 - i, first_quadrant),
            ] {
                if let Some(slot) = asu.get_mut(index) {
                    *slot = value;
                }
            }
        }
        Some(RisingTimes { asu })
    }

    /// The time of rising of a sign, 0 for Mesha.
    #[must_use]
    pub fn of_sign(&self, sign: usize) -> f64 {
        self.asu.get(sign % 12).copied().unwrap_or(0.0)
    }

    /// The point of the ecliptic on the eastern horizon when the given
    /// point has been up for `elapsed_asu` respirations (III.46 to 48):
    /// forward through the parts to come when the time is positive,
    /// backward through the parts past when it is negative. Degrees in
    /// `[0, 360)`, in the frame the point was given in.
    #[must_use]
    pub fn point_after(&self, from_deg: f64, elapsed_asu: f64) -> f64 {
        let from = from_deg.rem_euclid(360.0);
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a sign index in 0 to 11"
        )]
        let mut sign = (from / 30.0).floor() as usize % 12;
        let within = from - 30.0 * (from / 30.0).floor();
        if elapsed_asu >= 0.0 {
            // The equivalent of the part of the sign to come, then the
            // following signs in succession.
            let mut remaining = elapsed_asu;
            let mut available = (30.0 - within) * self.of_sign(sign) / 30.0;
            for _ in 0..WALK_CAP {
                if remaining <= available {
                    let done = 30.0 - available * 30.0 / self.of_sign(sign).max(f64::MIN_POSITIVE);
                    return (30.0 * sign_f64(sign) + done + remaining * 30.0 / self.of_sign(sign))
                        .rem_euclid(360.0);
                }
                remaining -= available;
                sign = (sign + 1) % 12;
                available = self.of_sign(sign);
            }
        } else {
            // The equivalent of the part past, then the signs past in
            // inverse order.
            let mut remaining = -elapsed_asu;
            let mut available = within * self.of_sign(sign) / 30.0;
            for _ in 0..WALK_CAP {
                if remaining <= available {
                    let start = 30.0 * sign_f64(sign)
                        + available * 30.0 / self.of_sign(sign).max(f64::MIN_POSITIVE);
                    return (start - remaining * 30.0 / self.of_sign(sign)).rem_euclid(360.0);
                }
                remaining -= available;
                sign = (sign + 11) % 12;
                available = self.of_sign(sign);
            }
        }
        from
    }

    /// The respirations from the rising of one point of the ecliptic to
    /// the rising of a later one (III.50): the part of the first point's
    /// sign to come, the part of the second's sign past, and the signs
    /// between. In `[0, 21 600)`.
    #[must_use]
    pub fn asu_between(&self, from_deg: f64, to_deg: f64) -> f64 {
        let from = from_deg.rem_euclid(360.0);
        let to = to_deg.rem_euclid(360.0);
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "sign indices in 0 to 11"
        )]
        let (from_sign, to_sign) = (
            (from / 30.0).floor() as usize % 12,
            (to / 30.0).floor() as usize % 12,
        );
        let from_within = from - 30.0 * (from / 30.0).floor();
        let to_within = to - 30.0 * (to / 30.0).floor();
        if from_sign == to_sign && to >= from {
            return (to - from) * self.of_sign(from_sign) / 30.0;
        }
        let mut total = (30.0 - from_within) * self.of_sign(from_sign) / 30.0;
        let mut sign = (from_sign + 1) % 12;
        for _ in 0..12 {
            if sign == to_sign {
                return total + to_within * self.of_sign(sign) / 30.0;
            }
            total += self.of_sign(sign);
            sign = (sign + 1) % 12;
        }
        total
    }
}

#[allow(clippy::cast_precision_loss, reason = "a sign index below twelve")]
const fn sign_f64(sign: usize) -> f64 {
    sign as f64
}

impl fmt::Display for RisingTimes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts: Vec<String> = self.asu.iter().map(|a| format!("{a:.0}")).collect();
        write!(f, "[{}] asu", parts.join(", "))
    }
}

/// The horoscope point at an instant, with the figures behind it.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct Lagna {
    /// The point on the eastern horizon, sidereal, degrees in `[0, 360)`.
    pub sidereal_deg: f64,
    /// The same with the text's precession applied, degrees.
    pub tropical_deg: f64,
    /// The point on the meridian (madhya lagna), sidereal, degrees.
    pub meridian_sidereal_deg: f64,
    /// The Sun's tropical longitude at sunrise, the walk's start.
    pub sun_tropical_deg: f64,
    /// The time since sunrise the walk covered, respirations; negative
    /// before sunrise.
    pub elapsed_asu: f64,
    /// The rising times used.
    pub rising: RisingTimes,
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and read the twelve signs' asu"
    )]

    use super::*;

    #[test]
    fn lanka_rising_times_mirror_the_quadrants_and_sum_to_a_day() {
        let lanka = RisingTimes::lanka(&Parameters::TEXT);
        let whole = |index: usize| lanka.asu[index].round() as u32;
        assert_eq!([whole(0), whole(5), whole(6), whole(11)], [1670; 4]);
        assert_eq!([whole(1), whole(4), whole(7), whole(10)], [1795; 4]);
        assert_eq!([whole(2), whole(3), whole(8), whole(9)], [1935; 4]);
        assert!((lanka.asu.iter().sum::<f64>() - ASU_PER_DAY).abs() < 1e-9);
        assert!(lanka.to_string().starts_with("[1670, 1795, 1935"));
    }

    #[test]
    fn washington_rising_times_follow_burgess() {
        // Burgess under III.42 to 45 for Washington, 38°54′: ascensional
        // differences 578′, 1061′ and 1263′ at the ends of the three
        // signs, oblique ascensions 1312½ for Vrishabha, 1733½ for
        // Mithuna, 2137½ for Karka and 2278½ for Simha.
        let washington = Latitude::literal(38.9);
        let text = SuryaSiddhanta::text();
        let rising = RisingTimes::at(&text, washington).unwrap();
        for (sign, expected) in [(1, 1312.5), (2, 1733.5), (3, 2137.5), (4, 2278.5)] {
            assert!(
                (rising.of_sign(sign) - expected).abs() < 2.0,
                "sign {sign}: {} against {expected}",
                rising.of_sign(sign)
            );
        }
        // The twelve still make a day, and the opposite quadrants mirror.
        assert!((rising.asu.iter().sum::<f64>() - ASU_PER_DAY).abs() < 1e-6);
        for i in 0..6 {
            assert!((rising.asu[i] - rising.asu[11 - i]).abs() < 1e-9);
        }
        // At the equator the oblique times are the right ascensions; above
        // the polar circle the rule has no answer.
        let equator = RisingTimes::at(&text, Latitude::literal(0.0)).unwrap();
        assert_eq!(equator, RisingTimes::lanka(&Parameters::TEXT));
        assert!(RisingTimes::at(&text, Latitude::literal(70.0)).is_none());
    }

    #[test]
    fn burgess_lagna_example_is_reproduced_and_the_walk_inverts() {
        // Burgess under III.46 to 49: at Washington, the Sun at 1s 12°
        // (42°) and 18 nadis 12 vinadis 3 palas (6555 respirations) after
        // sunrise, the horoscope point is 4s 25°.
        let rising = RisingTimes::at(&SuryaSiddhanta::text(), Latitude::literal(38.9)).unwrap();
        let lagna = rising.point_after(42.0, 6555.0);
        assert!((lagna - 145.0).abs() < 0.15, "{lagna}");
        // The inverse rule gives the elapsed time back.
        let back = rising.asu_between(42.0, lagna);
        assert!((back - 6555.0).abs() < 1e-6, "{back}");
        // No time elapsed: the point itself; a whole day: the point again;
        // a negative time walks back.
        assert!((rising.point_after(42.0, 0.0) - 42.0).abs() < 1e-9);
        assert!((rising.point_after(42.0, ASU_PER_DAY) - 42.0).abs() < 1e-6);
        // Back through Mesha, which rises slowly at Washington: 525 asu
        // through the part of Vrishabha past, the rest through Mesha.
        let before = rising.point_after(42.0, -1000.0);
        assert!(before < 30.0 && before > 10.0, "{before}");
        assert!((rising.asu_between(before, 42.0) - 1000.0).abs() < 1e-6);
        // Across the end of the zodiac: 364 asu through the rest of Meena
        // (which rises as Mesha does), 1092 through Mesha, 1312 through
        // Vrishabha and 232 of Mithuna's 1733½, four degrees.
        let wrapped = rising.point_after(350.0, 3000.0);
        assert!((wrapped - 64.015).abs() < 0.05, "{wrapped}");
        assert!((rising.asu_between(350.0, wrapped) - 3000.0).abs() < 1e-6);
        assert!((rising.asu_between(100.0, 100.0)).abs() < 1e-9);
    }
}
