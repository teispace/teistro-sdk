//! The span of life the wheel of time gives, Chakrayus (*Jataka Parijata*
//! ch. 5 v. 26, C310).
//!
//! "The Ayus consisting of the aggregate of the several periods belonging to
//! the untraversed portions of the nakshatrapadas or navamsas occupied by
//! the Sun and other planets is said to be the Chakrayus reckoned from the
//! seven planets" (p. 256), in ch. 17 v. 6's years: the Sun 5, the Moon 21,
//! Mars 7, Mercury 9, Jupiter 10, Venus 16 and Saturn 4. The note works the
//! Sun at 1s 2° 55′ 30″: 375.5 of Krittika's 800 minutes gone, so
//! 424.5 × 5 / 800 = 2.653 years. It takes the whole star's untraversed
//! part and the graha's own years; the verse names the pada, which is a
//! knob. "There is no harana in this Ayurdaya."

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Graha;
use teistro_dasha::kalachakra::SIGN_YEARS;

use crate::eval::Evaluator;
use crate::language::Body;

/// The seven, the Sun to Saturn.
const GRAHAS: [Graha; 7] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];

/// A nakshatra's arc, 13° 20′.
const STAR: f64 = 360.0 / 27.0;

/// Which untraversed part a graha's years are taken in proportion to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChakraPortion {
    /// The whole nakshatra's, as the note's figure takes it.
    #[default]
    Star,
    /// The pada's, a quarter of it, as the verse names it.
    Pada,
}

/// The choices Chakrayus is read under.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ChakrayusRules {
    /// Which untraversed part.
    pub portion: ChakraPortion,
}

/// One graha's part of Chakrayus.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct ChakraGiver {
    /// Which graha.
    pub graha: Graha,
    /// The fraction of its nakshatra or pada still untraversed, 0 to 1.
    pub untraversed: f64,
    /// Its years: its own ch. 17 years in that proportion.
    pub years: f64,
}

/// The seven grahas' years in the wheel of time and their sum.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Chakrayus {
    /// The seven, the Sun to Saturn.
    pub grahas: [ChakraGiver; 7],
    /// Their sum, in years of 360 days.
    pub years: f64,
    /// The choices it was read under.
    pub rules: ChakrayusRules,
}

/// A graha's years in the wheel of time (ch. 17 v. 6): its own signs'
/// years, which v. 6 says "correspond to the years of their lords"; none
/// for a node, which owns no sign.
#[must_use]
pub fn chakra_years(graha: Graha) -> Option<f64> {
    let sign = graha.attributes().own.first()?;
    SIGN_YEARS.get(*sign as usize).copied().map(f64::from)
}

/// The untraversed fraction of the nakshatra or pada a longitude is in.
#[must_use]
pub fn untraversed(longitude: f64, portion: ChakraPortion) -> f64 {
    let arc = match portion {
        ChakraPortion::Star => STAR,
        ChakraPortion::Pada => STAR / 4.0,
    };
    1.0 - longitude.rem_euclid(360.0).rem_euclid(arc) / arc
}

impl Evaluator<'_> {
    /// Chakrayus under `rules` (*Jataka Parijata* ch. 5 v. 26).
    #[must_use]
    pub fn chakrayus(&self, rules: ChakrayusRules) -> Chakrayus {
        let chart = self.chart();
        let grahas = GRAHAS.map(|graha| {
            let full = chakra_years(graha).unwrap_or(0.0);
            let left = untraversed(chart.placement(Body::Graha(graha)).longitude, rules.portion);
            ChakraGiver {
                graha,
                untraversed: left,
                years: full * left,
            }
        });
        Chakrayus {
            grahas,
            years: grahas.iter().map(|giver| giver.years).sum(),
            rules,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_years_are_ch_17_s_and_the_sun_of_the_figure_gives_2_653() {
        // ch. 17 v. 6: the Sun 5, the Moon 21, Mars 7, Mercury 9, Jupiter
        // 10, Venus 16 and Saturn 4; the nodes own no sign.
        let years = GRAHAS.map(|graha| chakra_years(graha).unwrap_or(-1.0));
        assert_eq!(years, [5.0, 21.0, 7.0, 9.0, 10.0, 16.0, 4.0]);
        assert_eq!(chakra_years(Graha::Rahu), None);
        // 1s 2° 55′ 30″: 375.5 of Krittika's 800 minutes gone (p. 256).
        let sun = 32.0 + 55.0 / 60.0 + 30.0 / 3600.0;
        let left = untraversed(sun, ChakraPortion::Star);
        assert!((left - 424.5 / 800.0).abs() < 1e-12);
        assert!((5.0 * left - 2.653).abs() < 1e-3);
        // The pada's own quarter: 24.5 of its 200 minutes left.
        assert!((untraversed(sun, ChakraPortion::Pada) - 24.5 / 200.0).abs() < 1e-12);
    }
}
