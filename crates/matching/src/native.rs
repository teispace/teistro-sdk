//! A native as matching reads one: the Moon at birth.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Nakshatra, Rashi};
use teistro_core::error::Error;

/// The arc of one nakshatra, degrees.
const NAKSHATRA_DEG: f64 = 360.0 / 27.0;

/// One native's Moon: her nakshatra, its pada, her sign and her navamsha's
/// sign, all in the chart's own zodiac.
///
/// ```
/// use teistro_core::catalogue::{Nakshatra, Rashi};
/// use teistro_matching::Native;
///
/// // 10° Aries: Ashvini's fourth pada, the navamsha of Cancer.
/// let moon = Native::of_moon(10.0)?;
/// assert_eq!((moon.nakshatra, moon.pada), (Nakshatra::Ashwini, 4));
/// assert_eq!((moon.rashi, moon.navamsha), (Rashi::Aries, Rashi::Cancer));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Native {
    /// The Moon's nakshatra.
    pub nakshatra: Nakshatra,
    /// Its pada, 1 to 4.
    pub pada: u8,
    /// The Moon's sign.
    pub rashi: Rashi,
    /// The sign of the Moon's navamsha.
    pub navamsha: Rashi,
}

impl Native {
    /// The native whose Moon stands at a longitude, in degrees of the
    /// chart's own zodiac.
    ///
    /// # Errors
    ///
    /// A longitude that is not a finite number, named `moon`.
    pub fn of_moon(longitude_deg: f64) -> Result<Native, Error> {
        if !longitude_deg.is_finite() {
            return Err(Error::invalid_arg(format!(
                "the Moon's longitude is a finite number of degrees, not {longitude_deg}"
            ))
            .with_field("moon"));
        }
        let longitude = longitude_deg.rem_euclid(360.0);
        // The 108 padas of the circle, of which nine fill a sign: the pada's
        // place in the circle gives the nakshatra, the pada and the navamsha.
        let quarter = whole(longitude / (NAKSHATRA_DEG / 4.0), 108);
        let nakshatra = Nakshatra::ALL
            .get(usize::from(quarter / 4))
            .copied()
            .unwrap_or(Nakshatra::Ashwini);
        let navamsha = Rashi::ALL
            .get(usize::from(quarter % 12))
            .copied()
            .unwrap_or(Rashi::Aries);
        Ok(Native {
            nakshatra,
            pada: u8::try_from(quarter % 4 + 1).unwrap_or(1),
            rashi: Rashi::of_longitude(longitude),
            navamsha,
        })
    }

    /// The nakshatra's place in the circle, 0 for Ashvini.
    pub(crate) fn star(self) -> u16 {
        self.nakshatra.id()
    }

    /// The sign's place in the zodiac, 0 for Aries.
    pub(crate) fn sign(self) -> u16 {
        self.rashi.id()
    }
}

/// The whole part of a non-negative ratio, held below `count`, which a
/// longitude a hair under 360° can otherwise round onto.
fn whole(ratio: f64, count: u16) -> u16 {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a non-negative ratio below a few hundred, clamped below"
    )]
    let floor = ratio.floor().max(0.0) as u16;
    floor.min(count - 1)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::Native;
    use teistro_core::catalogue::{Nakshatra, Rashi};

    #[test]
    fn every_pada_holds_its_arc_and_the_navamsha_follows() {
        for quarter in 0..108_u16 {
            let start = f64::from(quarter) * 360.0 / 108.0;
            for at in [start + 1e-9, start + 360.0 / 108.0 - 1e-9] {
                let moon = Native::of_moon(at).unwrap();
                assert_eq!(moon.nakshatra.id(), quarter / 4, "{at}");
                assert_eq!(u16::from(moon.pada), quarter % 4 + 1, "{at}");
                assert_eq!(moon.navamsha.id(), quarter % 12, "{at}");
            }
        }
        let last = Native::of_moon(360.0 - 1e-12).unwrap();
        assert_eq!(
            (last.nakshatra, last.pada, last.rashi, last.navamsha),
            (Nakshatra::Revati, 4, Rashi::Pisces, Rashi::Pisces)
        );
        let refused = Native::of_moon(f64::NAN).unwrap_err();
        assert_eq!(refused.field(), Some("moon"));
    }
}
