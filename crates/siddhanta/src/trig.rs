//! The sine table (II.15 to 27), its interpolation rule (II.31 to 33) and
//! the inverse; and the exact-trigonometry alternative for comparison.
//! The table works on a radius of 3438, the number of minutes in a
//! radian, in steps of 225 minutes (3°45′).

use teistro_core::math;
/// The radius the table's sines are measured on (3438′).
pub const RADIUS: f64 = 3438.0;
/// The step between table entries in minutes of arc (225′ = 3°45′).
pub const STEP_ARCMIN: f64 = 225.0;
/// The twenty-four sines of II.15 to 27, with the zero prepended.
pub const SINES: [u16; 25] = [
    0, 225, 449, 671, 890, 1105, 1315, 1520, 1719, 1910, 2093, 2267, 2431, 2585, 2728, 2859, 2978,
    3084, 3177, 3256, 3321, 3372, 3409, 3431, 3438,
];

/// Which trigonometry the model uses.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Trig {
    /// The text's table with linear interpolation: the classical path,
    /// which uses no platform mathematics and is bit-identical everywhere.
    #[default]
    Table,
    /// The platform's sine on the same radius: what a modern reformulation
    /// of the text computes; not bit-identical across platforms.
    Exact,
}

/// An arc reduced to the first quadrant (II.30): the arc itself, and the
/// signs its sine and cosine carry in the quadrant it came from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bhuja {
    /// The reduced arc, degrees in `[0, 90]`.
    pub arc_deg: f64,
    /// Whether the sine of the original arc is non-negative (the first
    /// half of the circle).
    pub sine_positive: bool,
    /// Whether the cosine of the original arc is non-negative (the half
    /// from Capricorn through Gemini).
    pub cosine_positive: bool,
}

impl Bhuja {
    /// The reduction of any arc in degrees.
    #[must_use]
    pub fn of(arc_deg: f64) -> Bhuja {
        let k = arc_deg.rem_euclid(360.0);
        if k <= 90.0 {
            Bhuja {
                arc_deg: k,
                sine_positive: true,
                cosine_positive: true,
            }
        } else if k <= 180.0 {
            Bhuja {
                arc_deg: 180.0 - k,
                sine_positive: true,
                cosine_positive: false,
            }
        } else if k <= 270.0 {
            Bhuja {
                arc_deg: k - 180.0,
                sine_positive: false,
                cosine_positive: false,
            }
        } else {
            Bhuja {
                arc_deg: 360.0 - k,
                sine_positive: false,
                cosine_positive: true,
            }
        }
    }

    /// The reduced arc's complement, for the cosine (kotijya).
    #[must_use]
    pub fn complement_deg(self) -> f64 {
        90.0 - self.arc_deg
    }
}

impl Trig {
    /// The sine of an arc in the first quadrant, on the radius: the
    /// table's interpolation (II.31 to 33) or the exact value.
    #[must_use]
    pub fn sine(self, arc_deg: f64) -> f64 {
        let arc = arc_deg.clamp(0.0, 90.0);
        match self {
            Trig::Table => {
                let minutes = arc * 60.0;
                let position = minutes / STEP_ARCMIN;
                let index = position.floor();
                #[allow(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "a floored value in 0 to 24"
                )]
                let i = index as usize;
                let Some((&lower, &upper)) = SINES.get(i).zip(SINES.get(i + 1)) else {
                    return RADIUS;
                };
                let remainder = position - index;
                f64::from(lower) + remainder * f64::from(upper - lower)
            }
            Trig::Exact => RADIUS * math::sin(arc.to_radians()),
        }
    }

    /// The arc in degrees, in the first quadrant, whose sine on the radius
    /// is `sine`: the inverse of the interpolation (II.33) or the exact
    /// value. A sine beyond the radius is the quadrant's end.
    #[must_use]
    pub fn arc(self, sine: f64) -> f64 {
        let sine = sine.clamp(0.0, RADIUS);
        match self {
            Trig::Table => {
                // The interval whose upper entry first reaches the sine.
                let i = SINES
                    .iter()
                    .skip(1)
                    .take_while(|&&entry| f64::from(entry) < sine)
                    .count()
                    .min(SINES.len() - 2);
                let Some((&lower, &upper)) = SINES.get(i).zip(SINES.get(i + 1)) else {
                    return 90.0;
                };
                let span = f64::from(upper - lower);
                let index = f64::from(u8::try_from(i).unwrap_or(u8::MAX));
                let position = index + (sine - f64::from(lower)) / span;
                position * STEP_ARCMIN / 60.0
            }
            Trig::Exact => math::asin(sine / RADIUS).to_degrees(),
        }
    }

    /// The cosine of an arc in the first quadrant as a ratio: the table's
    /// slope in its current interval (the "difference of sines" of II.47,
    /// which divided by the step is the cosine because the radius is the
    /// minutes in a radian) or the exact value.
    #[must_use]
    pub fn cosine_ratio(self, arc_deg: f64) -> f64 {
        let arc = arc_deg.clamp(0.0, 90.0);
        match self {
            Trig::Table => {
                let position = arc * 60.0 / STEP_ARCMIN;
                #[allow(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "a floored value in 0 to 24"
                )]
                let i = (position.floor() as usize).min(SINES.len() - 2);
                let Some((&lower, &upper)) = SINES.get(i).zip(SINES.get(i + 1)) else {
                    return 0.0;
                };
                f64::from(upper - lower) / STEP_ARCMIN
            }
            Trig::Exact => math::cos(arc.to_radians()),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use proptest::prelude::*;

    use super::*;

    #[test]
    fn the_table_is_the_texts_and_is_exact_at_its_nodes() {
        assert_eq!(SINES.len(), 25);
        assert_eq!(SINES[8], 1719); // sin 30° on 3438
        assert_eq!(SINES[24], 3438);
        assert!(SINES.windows(2).all(|w| w.first() < w.last()));
        for (i, &s) in SINES.iter().enumerate() {
            #[allow(clippy::cast_precision_loss, reason = "small indices")]
            let arc = i as f64 * 3.75;
            assert!((Trig::Table.sine(arc) - f64::from(s)).abs() < 1e-9, "{arc}");
            assert!((Trig::Table.arc(f64::from(s)) - arc).abs() < 1e-9, "{arc}");
            // The table is within 1′ of the true sine on the same radius.
            assert!((f64::from(s) - Trig::Exact.sine(arc)).abs() < 1.0, "{arc}");
        }
        // II.31 to 33 worked: 61°31′9″ = 3691.15′; 16 steps and 91.15′
        // over, a difference of 106: 2978 + 91.15 × 106 / 225 = 3020.94.
        let arc = 61.0 + 31.0 / 60.0 + 9.0 / 3600.0;
        assert!((Trig::Table.sine(arc) - 3020.94).abs() < 0.01);
        assert!((Trig::Table.sine(-5.0)).abs() < 1e-12);
        assert!((Trig::Table.sine(95.0) - RADIUS).abs() < 1e-12);
        assert!((Trig::Table.arc(-1.0)).abs() < 1e-12);
        assert!((Trig::Table.arc(5000.0) - 90.0).abs() < 1e-12);
        assert!((Trig::Exact.arc(RADIUS) - 90.0).abs() < 1e-9);
    }

    #[test]
    fn the_slope_is_the_cosine() {
        // In the first interval the slope is exactly 1, the cosine of 0.
        assert!((Trig::Table.cosine_ratio(1.0) - 1.0).abs() < 1e-12);
        // Near 60° the slope is 106/225 = 0.4711, the cosine 0.5.
        assert!((Trig::Table.cosine_ratio(61.0) - 106.0 / 225.0).abs() < 1e-12);
        assert!((Trig::Table.cosine_ratio(90.0) - 7.0 / 225.0).abs() < 1e-12);
        assert!((Trig::Exact.cosine_ratio(60.0) - 0.5).abs() < 1e-12);
        assert!((Trig::Table.cosine_ratio(500.0) - 7.0 / 225.0).abs() < 1e-12);
    }

    #[test]
    fn quadrants_reduce_with_their_signs() {
        let q = Bhuja::of(45.0);
        assert!((q.arc_deg - 45.0).abs() < 1e-12 && q.sine_positive && q.cosine_positive);
        let q = Bhuja::of(150.0);
        assert!((q.arc_deg - 30.0).abs() < 1e-12 && q.sine_positive && !q.cosine_positive);
        let q = Bhuja::of(210.0);
        assert!((q.arc_deg - 30.0).abs() < 1e-12 && !q.sine_positive && !q.cosine_positive);
        let q = Bhuja::of(330.0);
        assert!((q.arc_deg - 30.0).abs() < 1e-12 && !q.sine_positive && q.cosine_positive);
        assert!((Bhuja::of(-30.0).arc_deg - 30.0).abs() < 1e-12);
        assert!((Bhuja::of(720.0).arc_deg).abs() < 1e-12);
        assert!((Bhuja::of(20.0).complement_deg() - 70.0).abs() < 1e-12);
    }

    /// How much an arc recovered from its own sine may miss by.
    ///
    /// Reading an arc back out of a sine is **ill conditioned near the
    /// quadrant's end**: the sine is flat there, so an error of one ulp in
    /// it spreads over `1/cos(arc)` of arc. At 89.999998° the cosine is
    /// 3.3e-8 and a single ulp of the radius is already 2e-7 degrees out —
    /// which is arithmetic and not a defect, and a fixed bound of 1e-9
    /// asserts something no `f64` can do.
    ///
    /// **The table's own inverse is exempt**: it interpolates 24 linear
    /// segments, so its arc is the exact inverse of its sine whatever the
    /// conditioning of the true one.
    fn arc_tolerance(arc_deg: f64) -> f64 {
        const ULP_OF_RADIUS: f64 = RADIUS * f64::EPSILON;
        1e-9 + (ULP_OF_RADIUS / RADIUS / math::cos(arc_deg.to_radians())).to_degrees()
    }

    proptest! {
        #[test]
        fn the_table_inverts_and_tracks_the_true_sine(arc in 0.0f64..90.0) {
            let sine = Trig::Table.sine(arc);
            prop_assert!((Trig::Table.arc(sine) - arc).abs() < 1e-9);
            prop_assert!((sine - Trig::Exact.sine(arc)).abs() < 2.0);
            let read_back = Trig::Exact.arc(Trig::Exact.sine(arc));
            prop_assert!(
                (read_back - arc).abs() < arc_tolerance(arc),
                "{arc} read back as {read_back}, past {}",
                arc_tolerance(arc)
            );
            prop_assert!((Trig::Table.cosine_ratio(arc) - Trig::Exact.cosine_ratio(arc)).abs() < 0.04);
        }
    }

    /// The bound above is the conditioning and not a licence: away from the
    /// quadrant's end the exact reading still inverts to a nanodegree, and
    /// it is only the last fraction of a degree that needs more.
    #[test]
    fn the_exact_reading_inverts_tightly_away_from_the_quadrants_end() {
        for arc in [0.0, 1.0, 30.0, 45.0, 60.0, 80.0, 89.0] {
            assert!(arc_tolerance(arc) < 1.1e-9, "{arc}: {}", arc_tolerance(arc));
            let read_back = Trig::Exact.arc(Trig::Exact.sine(arc));
            assert!(
                (read_back - arc).abs() < 1e-9,
                "{arc} read back as {read_back}"
            );
        }
        // And the end itself is where it has to give: a hundredth of a
        // degree short of 90 already needs more than a nanodegree.
        assert!(arc_tolerance(89.999_999) > 1e-8);
    }
}
