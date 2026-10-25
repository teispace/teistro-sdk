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
///
/// ```
/// use teistro_rules::longevity::{ChakraPortion, untraversed};
///
/// // Aries 15°: 100 of Bharani's 800 minutes gone, and 100 of its first pada's 200.
/// assert_eq!(untraversed(15.0, ChakraPortion::Star), 0.875);
/// assert!((untraversed(15.0, ChakraPortion::Pada) - 0.5).abs() < 1e-12);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum ChakraPortion {
    /// The whole nakshatra's, as the note's figure takes it.
    #[default]
    Star,
    /// The pada's, a quarter of it, as the verse names it.
    Pada,
}

/// The choices Chakrayus is read under.
///
/// ```
/// use teistro_rules::longevity::{ChakraPortion, ChakrayusRules};
///
/// let rules: ChakrayusRules = serde_json::from_str(r#"{"portion": "pada"}"#)?;
/// assert_eq!(rules.portion, ChakraPortion::Pada);
/// assert_eq!(serde_json::from_str::<ChakrayusRules>("{}")?, ChakrayusRules::default());
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct ChakrayusRules {
    /// Which untraversed part.
    pub portion: ChakraPortion,
}

/// One graha's part of Chakrayus.
///
/// ```
/// # use teistro_core::catalogue::{Dignity, Rashi};
/// # use teistro_rules::{Body, House, Placement, RuleChart};
/// #
/// # let mut chart = RuleChart {
/// #     placements: [Placement {
/// #         longitude: 15.0,
/// #         sign: Rashi::Aries,
/// #         house: House::try_new(1).unwrap(),
/// #         dignity: Dignity::Neutral,
/// #         retrograde: false,
/// #         combust: false,
/// #         karaka7: None,
/// #         karaka8: None,
/// #         navamsha: Rashi::Aries,
/// #     }; 10],
/// #     panchanga: None,
/// #     strengths: None,
/// # };
/// # let place = |chart: &mut RuleChart, body: Body, sign: Rashi| {
/// #     let at = &mut chart.placements[body.index()];
/// #     at.sign = sign;
/// #     at.longitude = f64::from(sign as u8) * 30.0 + 15.0;
/// #     at.house = House::between(Rashi::Aries, sign);
/// # };
///
/// # use teistro_core::catalogue::Graha;
/// # use teistro_rules::{Evaluator, Readings};
/// #
/// # let evaluator = Evaluator::new(&chart, Readings::TEXTS);
///
/// # use teistro_rules::longevity::ChakrayusRules;
/// #
/// # let span = evaluator.chakrayus(ChakrayusRules::default());
///
/// // The Moon at Aries 15° keeps seven eighths of her 21 years.
/// let moon = span.grahas[1];
/// assert_eq!((moon.graha, moon.untraversed), (Graha::Moon, 0.875));
/// assert!((moon.years - 18.375).abs() < 1e-12);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ChakraGiver {
    /// Which graha.
    pub graha: Graha,
    /// The fraction of its nakshatra or pada still untraversed, 0 to 1.
    pub untraversed: f64,
    /// Its years: its own ch. 17 years in that proportion.
    pub years: f64,
}

/// The seven grahas' years in the wheel of time and their sum.
///
/// ```
/// # use teistro_core::catalogue::{Dignity, Rashi};
/// # use teistro_rules::{Body, House, Placement, RuleChart};
/// #
/// # let mut chart = RuleChart {
/// #     placements: [Placement {
/// #         longitude: 15.0,
/// #         sign: Rashi::Aries,
/// #         house: House::try_new(1).unwrap(),
/// #         dignity: Dignity::Neutral,
/// #         retrograde: false,
/// #         combust: false,
/// #         karaka7: None,
/// #         karaka8: None,
/// #         navamsha: Rashi::Aries,
/// #     }; 10],
/// #     panchanga: None,
/// #     strengths: None,
/// # };
/// # let place = |chart: &mut RuleChart, body: Body, sign: Rashi| {
/// #     let at = &mut chart.placements[body.index()];
/// #     at.sign = sign;
/// #     at.longitude = f64::from(sign as u8) * 30.0 + 15.0;
/// #     at.house = House::between(Rashi::Aries, sign);
/// # };
///
/// # use teistro_core::catalogue::Graha;
/// # use teistro_rules::{Evaluator, Readings};
/// #
/// # let evaluator = Evaluator::new(&chart, Readings::TEXTS);
///
/// # use teistro_rules::longevity::ChakrayusRules;
/// #
/// # let span = evaluator.chakrayus(ChakrayusRules::default());
///
/// // Seven eighths of the seven's 72 years.
/// assert!((span.years - 63.0).abs() < 1e-12);
/// assert_eq!(span.grahas.map(|giver| giver.graha)[6], Graha::Saturn);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_rules::longevity::chakra_years;
///
/// // ch. 17 v. 6: the Moon 21, Saturn 4; the nodes own no sign.
/// assert_eq!(chakra_years(Graha::Moon), Some(21.0));
/// assert_eq!(chakra_years(Graha::Saturn), Some(4.0));
/// assert_eq!(chakra_years(Graha::Ketu), None);
/// ```
#[must_use]
pub fn chakra_years(graha: Graha) -> Option<f64> {
    let sign = graha.attributes().own.first()?;
    SIGN_YEARS.get(*sign as usize).copied().map(f64::from)
}

/// The untraversed fraction of the nakshatra or pada a longitude is in.
///
/// ```
/// use teistro_rules::longevity::{ChakraPortion, untraversed};
///
/// // The note's Sun at 1s 2° 55′ 30″: 424.5 of Krittika's 800 minutes left (p. 256).
/// let sun = 32.0 + 55.0 / 60.0 + 30.0 / 3600.0;
/// assert!((untraversed(sun, ChakraPortion::Star) - 424.5 / 800.0).abs() < 1e-12);
/// // A star's first instant leaves the whole.
/// assert_eq!(untraversed(0.0, ChakraPortion::Star), 1.0);
/// ```
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
    ///
    /// ```
    /// # use teistro_core::catalogue::{Dignity, Rashi};
    /// # use teistro_rules::{Body, House, Placement, RuleChart};
    /// #
    /// # let mut chart = RuleChart {
    /// #     placements: [Placement {
    /// #         longitude: 15.0,
    /// #         sign: Rashi::Aries,
    /// #         house: House::try_new(1).unwrap(),
    /// #         dignity: Dignity::Neutral,
    /// #         retrograde: false,
    /// #         combust: false,
    /// #         karaka7: None,
    /// #         karaka8: None,
    /// #         navamsha: Rashi::Aries,
    /// #     }; 10],
    /// #     panchanga: None,
    /// #     strengths: None,
    /// # };
    /// # let place = |chart: &mut RuleChart, body: Body, sign: Rashi| {
    /// #     let at = &mut chart.placements[body.index()];
    /// #     at.sign = sign;
    /// #     at.longitude = f64::from(sign as u8) * 30.0 + 15.0;
    /// #     at.house = House::between(Rashi::Aries, sign);
    /// # };
    ///
    /// # use teistro_core::catalogue::Graha;
    /// # use teistro_rules::{Evaluator, Readings};
    /// #
    /// # let evaluator = Evaluator::new(&chart, Readings::TEXTS);
    ///
    /// # use teistro_rules::longevity::ChakrayusRules;
    /// #
    /// # let span = evaluator.chakrayus(ChakrayusRules::default());
    ///
    /// use teistro_rules::longevity::ChakraPortion;
    ///
    /// // Read by the pada, each keeps half of its years.
    /// let by_pada = evaluator.chakrayus(ChakrayusRules { portion: ChakraPortion::Pada });
    /// assert!((by_pada.years - 36.0).abs() < 1e-12);
    /// assert_eq!(span.rules, ChakrayusRules::default());
    /// ```
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
    #![allow(clippy::unwrap_used, reason = "tests unwrap what they built")]

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

    /// Every graha at Aries 15°, 100 of Bharani's 800 minutes gone, gives
    /// seven eighths of its years: 63 of the 72.
    #[test]
    fn each_graha_gives_its_years_by_what_is_left_of_its_star() {
        let chart = crate::test_chart::chart();
        let span = crate::eval::Evaluator::new(&chart, crate::chart::Readings::TEXTS)
            .chakrayus(ChakrayusRules::default());
        for giver in &span.grahas {
            let full = chakra_years(giver.graha).unwrap();
            assert!((giver.untraversed - 0.875).abs() < 1e-12, "{giver:?}");
            assert!((giver.years - full * 0.875).abs() < 1e-12, "{giver:?}");
        }
        assert!((span.years - 63.0).abs() < 1e-12);
    }
}
