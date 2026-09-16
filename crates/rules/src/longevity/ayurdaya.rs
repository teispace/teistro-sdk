//! Pindayu, Nisargayu and Amsayu: a span of life summed from what each graha
//! and the lagna give (BPHS ch. 43 vv. 4 to 32).
//!
//! **Pindayu** (vv. 4 to 8): each graha gives its full years at its deep
//! exaltation and half at its deep debilitation, in proportion between.
//! **Nisargayu** (vv. 16 to 17) lists years of its own for each graha.
//! **Amsayu** (vv. 18 to 19) gives each graha a year for every navamsha it has
//! gone from Aries, less whole twelves. All three lose the same reductions
//! (vv. 9 to 13, 20): half for combustion (never Venus or Saturn), a third in
//! an enemy's sign (never retrograde), a share in the visible half from the
//! whole in the twelfth to a sixth in the seventh (a benefic half that), and
//! a malefic rising in proportion to the lagna's degrees. The lagna gives its
//! own (vv. 14 to 15). Which span is the native's is the one whose ruler —
//! the lagna for Amsayu, the Sun for Pindayu, the Moon for Nisargayu — is
//! strongest, and the mean on a tie (vv. 30 to 32).
//!
//! What the translation does not settle is a knob, [`AyurdayaRules`] (crux
//! C104). Years are the texts' own, of 360 days.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi};

use crate::chart::{Placement, Strengths};
use crate::eval::{Evaluator, Participants};
use crate::language::{Body, Condition, House};
use crate::reference::{BodyRef, BodySubject};

/// The grahas the spans are summed over, the Sun to Saturn.
const GRAHAS: [Graha; 7] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];

/// The benefics for these reductions: the natural four, the Moon a benefic
/// waxing or waning (v. 11) and Mercury whatever his company.
const BENEFICS: [Graha; 4] = [Graha::Moon, Graha::Mercury, Graha::Jupiter, Graha::Venus];

/// Which span.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Method {
    /// From each graha's distance to its deep exaltation.
    Pindayu,
    /// From each graha's own listed years.
    Nisargayu,
    /// From the navamshas each has gone.
    Amsayu,
}

/// How several reductions on one graha combine.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Combine {
    /// Only the largest is taken, as the translator's notes to vv. 4 to 15
    /// say; the verses themselves are silent.
    #[default]
    Largest,
    /// Each is taken in turn from what the one before left.
    Each,
}

/// How Nisargayu's listed years are given.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Nisarga {
    /// As Pindayu gives its years: full at deep exaltation, half at deep
    /// debilitation, with the same reductions and the lagna's own years, as
    /// the Thakur Prasad edition's verse adds.
    #[default]
    LikePindayu,
    /// The listed years alone, as the verses the translation follows give
    /// them: always 120.
    Listed,
}

/// The choices the spans are read under.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AyurdayaRules {
    /// How reductions combine.
    pub combine: Combine,
    /// How Nisargayu's years are given.
    pub nisarga: Nisarga,
    /// Whether Amsayu triples a graha exalted or in its own sign and doubles
    /// one in its own navamsha or drekkana, as "some scholars suggest"
    /// (vv. 20 to 22).
    pub amsayu_multiplied: bool,
}

/// Who gives years.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Giver {
    /// A graha.
    Graha(Graha),
    /// The lagna.
    Lagna,
}

/// What each reduction takes from a graha, in years.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct Reductions {
    /// Astangata harana: half, combust.
    pub combustion: f64,
    /// Shatrukshetra harana: a third, in an enemy's sign.
    pub enemy_sign: f64,
    /// Vyayadi harana: in the visible half, the twelfth to the seventh.
    pub visible_half: f64,
    /// Kroorodaya harana: a malefic rising.
    pub rising: f64,
}

/// What one giver gives.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Contribution {
    /// Who.
    pub giver: Giver,
    /// The years before any reduction.
    pub basic: f64,
    /// What each reduction would take.
    pub reductions: Reductions,
    /// The years after the reductions, combined as the rules say.
    pub net: f64,
}

/// One span.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Span {
    /// Which method.
    pub method: Method,
    /// The seven grahas' contributions, the Sun to Saturn, then the lagna's.
    pub contributions: [Contribution; 8],
    /// Their sum, in years of 360 days.
    pub years: f64,
}

/// The three spans of a chart.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Ayurdaya {
    /// Pindayu.
    pub pindayu: Span,
    /// Nisargayu.
    pub nisargayu: Span,
    /// Amsayu.
    pub amsayu: Span,
    /// The choices it was read under.
    pub rules: AyurdayaRules,
}

impl Ayurdaya {
    /// The span the native is given (vv. 30 to 32): Amsayu where the lagna is
    /// strongest, Pindayu the Sun, Nisargayu the Moon, and the mean of those
    /// tied for strongest. The strengths are the caller's, in one measure; the
    /// kernel carries none for the lagna.
    #[must_use]
    pub fn chosen(&self, lagna: f64, sun: f64, moon: f64) -> f64 {
        let candidates = [
            (lagna, self.amsayu.years),
            (sun, self.pindayu.years),
            (moon, self.nisargayu.years),
        ];
        let strongest = candidates
            .iter()
            .map(|(strength, _)| *strength)
            .fold(f64::NEG_INFINITY, f64::max);
        let (sum, count) = candidates
            .iter()
            .filter(|(strength, _)| (*strength - strongest).abs() < f64::EPSILON)
            .fold((0.0, 0.0), |(sum, count), (_, years)| {
                (sum + years, count + 1.0)
            });
        sum / f64::max(count, 1.0)
    }
}

/// Each graha's full years at deep exaltation in Pindayu (vv. 4 to 8), the Sun
/// to Saturn.
const PINDAYU_YEARS: [f64; 7] = [19.0, 25.0, 15.0, 12.0, 15.0, 21.0, 20.0];

/// Each graha's listed years in Nisargayu (vv. 16 to 17), the Sun to Saturn;
/// the verse lists them from the Moon, and they sum to 120.
const NISARGAYU_YEARS: [f64; 7] = [20.0, 1.0, 2.0, 9.0, 18.0, 20.0, 50.0];

/// A graha's full years in Pindayu or its listed years in Nisargayu; none in
/// Amsayu, which counts navamshas, or for a node.
#[must_use]
pub fn full_years(method: Method, graha: Graha) -> Option<f64> {
    let table = match method {
        Method::Pindayu => &PINDAYU_YEARS,
        Method::Nisargayu => &NISARGAYU_YEARS,
        Method::Amsayu => return None,
    };
    let at = GRAHAS.iter().position(|seven| *seven == graha)?;
    table.get(at).copied()
}

/// The years a graha gives by its distance from its deep exaltation (vv. 4 to
/// 8): the whole there, half at the point opposite, in proportion between.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_rules::longevity::by_exaltation;
///
/// // The translator's example: Saturn at 3° 9′ 41″ Gemini is 223.1613° from
/// // 20° Libra, and gives 12.3979 of his 20 years.
/// let saturn = 60.0 + 3.0 + 9.0 / 60.0 + 41.0 / 3600.0;
/// assert!((by_exaltation(Graha::Saturn, saturn, 20.0).unwrap() - 12.3979).abs() < 1e-4);
/// ```
#[must_use]
pub fn by_exaltation(graha: Graha, longitude: f64, full: f64) -> Option<f64> {
    let exaltation = graha.attributes().exaltation?;
    let point = f64::from(exaltation.sign as u8) * 30.0 + f64::from(exaltation.degree);
    let arc = (point - longitude).rem_euclid(360.0);
    let arc = if arc < 180.0 { 360.0 - arc } else { arc };
    Some(full * arc / 360.0)
}

/// The years a longitude gives in Amsayu (vv. 18 to 19): a year a navamsha
/// from Aries, less whole twelves.
#[must_use]
pub fn by_navamsha(longitude: f64) -> f64 {
    (longitude.rem_euclid(360.0) * 108.0 / 360.0).rem_euclid(12.0)
}

/// The share of its years a graha loses in a house of the visible half (vv. 10
/// to 11): the whole in the twelfth, then a half, a third, a quarter, a fifth
/// and a sixth down to the seventh; a benefic half of that.
#[must_use]
pub fn visible_half_share(house: House, benefic: bool) -> f64 {
    let share = match house.get() {
        12 => 1.0,
        11 => 1.0 / 2.0,
        10 => 1.0 / 3.0,
        9 => 1.0 / 4.0,
        8 => 1.0 / 5.0,
        7 => 1.0 / 6.0,
        _ => 0.0,
    };
    if benefic { share / 2.0 } else { share }
}

impl Evaluator<'_> {
    /// The three spans of this chart under `rules`. A graha's house is whole
    /// signs from the lagna, and "the strongest of several in one house" reads
    /// the chart's strengths; where they cannot say, each of the several loses.
    #[must_use]
    pub fn ayurdaya(&self, rules: AyurdayaRules) -> Ayurdaya {
        Ayurdaya {
            pindayu: self.span(Method::Pindayu, rules),
            nisargayu: self.span(Method::Nisargayu, rules),
            amsayu: self.span(Method::Amsayu, rules),
            rules,
        }
    }

    fn span(&self, method: Method, rules: AyurdayaRules) -> Span {
        let chart = self.chart();
        let lagna = chart.placement(Body::Lagna);
        let strengths = self.strengths();
        let benefic_aspects_lagna = self.benefic_aspects_lagna();
        let contributions = GRAHAS.map(|graha| {
            let body = Body::Graha(graha);
            let at = chart.placement(body);
            let house = House::between(lagna.sign, at.sign);
            let full = full_years(method, graha).unwrap_or(0.0);
            let basic = match (method, rules.nisarga) {
                (Method::Nisargayu, Nisarga::Listed) => full,
                (Method::Pindayu | Method::Nisargayu, _) => {
                    by_exaltation(graha, at.longitude, full).unwrap_or(full)
                }
                (Method::Amsayu, _) => {
                    by_navamsha(at.longitude) * amsayu_multiple(rules, graha, at)
                }
            };
            if method == Method::Nisargayu && rules.nisarga == Nisarga::Listed {
                return plain(Giver::Graha(graha), basic);
            }
            let benefic = BENEFICS.contains(&graha);
            // Only the strongest of several in one house of the visible half
            // loses, where the strengths can say which that is.
            let outdone = GRAHAS.iter().any(|other| {
                *other != graha
                    && chart.placement(Body::Graha(*other)).sign == at.sign
                    && strengths_say(&strengths, *other, graha)
            });
            let reductions = Reductions {
                combustion: if at.combust && !matches!(graha, Graha::Venus | Graha::Saturn) {
                    basic / 2.0
                } else {
                    0.0
                },
                // The sign's lord a natural enemy of the graha's, as the
                // catalogue lists them: the compound relationship a chart's
                // dignity carries turns on where the two stand, which the
                // verse does not ask.
                enemy_sign: if !at.retrograde
                    && graha
                        .attributes()
                        .enemies
                        .contains(&at.sign.attributes().lord)
                {
                    basic / 3.0
                } else {
                    0.0
                },
                visible_half: if outdone {
                    0.0
                } else {
                    basic * visible_half_share(house, benefic)
                },
                rising: if !benefic && house.get() == 1 {
                    let loss = basic * lagna.longitude.rem_euclid(360.0) / 360.0;
                    if benefic_aspects_lagna {
                        loss / 2.0
                    } else {
                        loss
                    }
                } else {
                    0.0
                },
            };
            Contribution {
                giver: Giver::Graha(graha),
                basic,
                reductions,
                net: combined(basic, reductions, rules.combine),
            }
        });
        let lagna_years = match (method, rules.nisarga) {
            (Method::Nisargayu, Nisarga::Listed) => 0.0,
            (Method::Amsayu, _) => by_navamsha(lagna.longitude),
            _ => self.lagna_years(),
        };
        let [sun, moon, mars, mercury, jupiter, venus, saturn] = contributions;
        let contributions = [
            sun,
            moon,
            mars,
            mercury,
            jupiter,
            venus,
            saturn,
            plain(Giver::Lagna, lagna_years),
        ];
        Span {
            method,
            years: contributions
                .iter()
                .map(|contribution| contribution.net)
                .sum(),
            contributions,
        }
    }

    /// The lagna's own years (vv. 14 to 15): a year a sign from Aries and its
    /// degrees in proportion, or, where the navamsha lagna's lord is stronger
    /// than the lagna's, a year a navamsha.
    fn lagna_years(&self) -> f64 {
        let chart = self.chart();
        let lagna = chart.placement(Body::Lagna);
        let lord = lagna.sign.attributes().lord;
        let navamsha_lord = lagna.navamsha.attributes().lord;
        if navamsha_lord != lord && strengths_say(&self.strengths(), navamsha_lord, lord) {
            by_navamsha(lagna.longitude)
        } else {
            lagna.longitude.rem_euclid(360.0) / 30.0
        }
    }

    /// Whether a natural benefic aspects the lagna, which halves a rising
    /// malefic's loss (v. 13).
    fn benefic_aspects_lagna(&self) -> bool {
        let Ok(first) = House::try_new(1) else {
            return false;
        };
        BENEFICS.iter().any(|graha| {
            self.holds(
                &Condition::PlanetAspectsHouse {
                    from: BodySubject::Ref(BodyRef::Body(Body::Graha(*graha))),
                    house_ruled: first,
                },
                &mut Participants::default(),
            )
        })
    }
}

/// Whether one graha is stronger than another by the chart's strengths.
fn strengths_say(strengths: &Strengths, one: Graha, other: Graha) -> bool {
    strengths.exceeds(Body::Graha(one), Body::Graha(other))
}

/// A contribution no reduction touches.
const fn plain(giver: Giver, years: f64) -> Contribution {
    Contribution {
        giver,
        basic: years,
        reductions: Reductions {
            combustion: 0.0,
            enemy_sign: 0.0,
            visible_half: 0.0,
            rising: 0.0,
        },
        net: years,
    }
}

/// The years left after the reductions, combined as the rules say.
fn combined(basic: f64, reductions: Reductions, combine: Combine) -> f64 {
    let shares = [
        reductions.combustion,
        reductions.enemy_sign,
        reductions.visible_half,
        reductions.rising,
    ];
    match combine {
        Combine::Largest => basic - shares.iter().copied().fold(0.0, f64::max),
        Combine::Each => shares.iter().fold(basic, |left, share| {
            // Each share was measured against the basic years; taken in turn,
            // it takes the same fraction of what is left.
            if basic > 0.0 {
                left - left * share / basic
            } else {
                left
            }
        }),
    }
}

/// Amsayu's multiple for a graha under the rules (vv. 20 to 22): three
/// exalted or in its own sign, two in its own navamsha or drekkana, the
/// larger only; one otherwise, or when the rules do not multiply.
fn amsayu_multiple(rules: AyurdayaRules, graha: Graha, at: &Placement) -> f64 {
    if !rules.amsayu_multiplied {
        return 1.0;
    }
    let attributes = graha.attributes();
    let exalted = attributes
        .exaltation
        .is_some_and(|point| point.sign == at.sign);
    let own = attributes.own.contains(&at.sign);
    if exalted || own {
        return 3.0;
    }
    let own_navamsha = attributes.own.contains(&at.navamsha);
    if own_navamsha || attributes.own.contains(&drekkana_of(at.longitude)) {
        2.0
    } else {
        1.0
    }
}

/// The drekkana sign of a longitude, Parashara's: the sign itself, the fifth
/// from it and the ninth, a third of the sign each.
fn drekkana_of(longitude: f64) -> Rashi {
    let longitude = longitude.rem_euclid(360.0);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a longitude under 360 over thirty is 0 to 11, and a third of a sign 0 to 2"
    )]
    let (sign, part) = (
        (longitude / 30.0) as u16,
        ((longitude % 30.0) / 10.0) as u16,
    );
    Rashi::from_id((sign.min(11) + 4 * part.min(2)) % 12).unwrap_or(Rashi::Aries)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests unwrap what they built and index their own arrays"
    )]

    use super::*;
    use crate::chart::Readings;
    use crate::test_chart::chart;

    /// Degrees, minutes and seconds into a sign, as a longitude.
    fn at(sign: Rashi, degrees: f64, minutes: f64, seconds: f64) -> f64 {
        f64::from(sign as u8) * 30.0 + degrees + minutes / 60.0 + seconds / 3600.0
    }

    /// The translator's worked example, his longitudes and his basic years for
    /// each graha. His Venus gives 19.2327 years only at 27° 17′ 50″ Aries.
    #[test]
    fn pindayu_gives_the_translator_s_basic_years_from_his_longitudes() {
        let example = [
            (Graha::Sun, at(Rashi::Taurus, 7.0, 12.0, 18.0), 17.5642),
            (Graha::Moon, at(Rashi::Aries, 27.0, 35.0, 46.0), 24.6247),
            (Graha::Mars, at(Rashi::Cancer, 6.0, 18.0, 46.0), 8.4036),
            (Graha::Mercury, at(Rashi::Aries, 14.0, 54.0, 13.0), 6.9968),
            (Graha::Jupiter, at(Rashi::Cancer, 26.0, 7.0, 13.0), 14.1200),
            (Graha::Venus, at(Rashi::Aries, 27.0, 17.0, 50.0), 19.2327),
            (Graha::Saturn, at(Rashi::Gemini, 3.0, 9.0, 41.0), 12.3979),
        ];
        for (graha, longitude, years) in example {
            let full = full_years(Method::Pindayu, graha).unwrap();
            let basic = by_exaltation(graha, longitude, full).unwrap();
            assert!((basic - years).abs() < 1e-3, "{graha:?}: {basic}");
        }
        // Full at deep exaltation, half at deep debilitation.
        assert!((by_exaltation(Graha::Sun, 10.0, 19.0).unwrap() - 19.0).abs() < 1e-9);
        assert!((by_exaltation(Graha::Sun, 190.0, 19.0).unwrap() - 9.5).abs() < 1e-9);
        // The Sun in an enemy's sign keeps two thirds, the Moon combust half.
        let third = Reductions {
            enemy_sign: 17.5642 / 3.0,
            ..Reductions::default()
        };
        assert!((combined(17.5642, third, Combine::Largest) - 11.7095).abs() < 1e-4);
        let half = Reductions {
            combustion: 24.6247 / 2.0,
            ..Reductions::default()
        };
        assert!((combined(24.6247, half, Combine::Largest) - 12.3124).abs() < 1e-4);
        // The lagna at 0° 48′ 34″ Scorpio, its navamsha lord the stronger, gives
        // 3.2428 years by navamsha.
        assert!((by_navamsha(at(Rashi::Scorpio, 0.0, 48.0, 34.0)) - 3.2428).abs() < 1e-4);
        // Nisargayu's listed years are the full span of a man.
        assert!((NISARGAYU_YEARS.iter().sum::<f64>() - 120.0).abs() < 1e-9);
    }

    #[test]
    fn the_visible_half_takes_less_the_further_from_the_twelfth_and_half_from_a_benefic() {
        let shares: Vec<f64> = (1..=12)
            .map(|house| visible_half_share(House::try_new(house).unwrap(), false))
            .collect();
        let expected = [
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0 / 6.0,
            1.0 / 5.0,
            1.0 / 4.0,
            1.0 / 3.0,
            1.0 / 2.0,
            1.0,
        ];
        for (share, expected) in shares.iter().zip(expected) {
            assert!((share - expected).abs() < 1e-12);
        }
        assert!((visible_half_share(House::try_new(12).unwrap(), false) - 1.0).abs() < 1e-9);
        assert!((visible_half_share(House::try_new(7).unwrap(), true) - 1.0 / 12.0).abs() < 1e-9);
        assert!(visible_half_share(House::try_new(6).unwrap(), false).abs() < 1e-9);
        // Largest takes the one; each takes its fraction of what is left.
        let two = Reductions {
            combustion: 5.0,
            enemy_sign: 10.0 / 3.0,
            ..Reductions::default()
        };
        assert!((combined(10.0, two, Combine::Largest) - 5.0).abs() < 1e-9);
        assert!((combined(10.0, two, Combine::Each) - 10.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn a_chart_s_spans_are_summed_and_chosen_by_the_strongest_with_ties_averaged() {
        let c = chart();
        let evaluator = Evaluator::new(&c, Readings::TEXTS);
        let listed = evaluator.ayurdaya(AyurdayaRules {
            nisarga: Nisarga::Listed,
            ..AyurdayaRules::default()
        });
        assert!((listed.nisargayu.years - 120.0).abs() < 1e-9);
        let mut spans = evaluator.ayurdaya(AyurdayaRules::default());
        for span in [&spans.pindayu, &spans.nisargayu, &spans.amsayu] {
            let sum: f64 = span.contributions.iter().map(|c| c.net).sum();
            assert!((sum - span.years).abs() < 1e-9);
            for contribution in &span.contributions {
                assert!(contribution.net >= 0.0 && contribution.net <= contribution.basic + 1e-9);
            }
        }
        // The translator's notes to v. 32: the Sun and the Moon equally strong,
        // 52.5 years by Pindayu and 40.7 by Nisargayu. He writes their mean as
        // 46.35, a slip for 46.6; his three-way example, 62.9, 25.5 and 12.8,
        // is 33.73 as he says.
        spans.pindayu.years = 52.5;
        spans.nisargayu.years = 40.7;
        assert!((spans.chosen(1.0, 2.0, 2.0) - 46.6).abs() < 1e-9);
        spans.pindayu.years = 62.9;
        spans.nisargayu.years = 25.5;
        spans.amsayu.years = 12.8;
        assert!((spans.chosen(2.0, 2.0, 2.0) - 33.733_333_333).abs() < 1e-6);
        assert!((spans.chosen(3.0, 2.0, 1.0) - 12.8).abs() < 1e-9);
    }
}
