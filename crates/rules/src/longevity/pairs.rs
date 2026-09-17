//! The three pairs: a class of life from the signs six things stand in (BPHS
//! ch. 43 vv. 33 to 50).
//!
//! The verses pair the lagna lord with the eighth lord, Saturn with the Moon,
//! and the lagna with the hora lagna. Each pair gives a class by the
//! modalities of its two signs; the class two or three pairs agree on is the
//! chart's, and where all three differ the lagna pair decides, unless the Moon
//! stands in the lagna or the seventh, when Saturn and the Moon do. The class
//! and how many pairs gave it fix the years (vv. 41 to 44); the degrees the
//! contributing pairs stand at rectify them (vv. 45 to 46); and Saturn among
//! the contributors lowers the class while Jupiter well placed raises it
//! (vv. 47 to 50).
//!
//! It is a reading of an [`Evaluator`] and not a rule: it needs the chart's
//! hora lagna, so it is `None` on an evaluator given no points, and it reads
//! "joined or aspected only by malefics" through the kernel's own conditions,
//! so the words mean here what they mean in every rule.
//!
//! Three places the translation does not settle are knobs, [`ThreePairsRules`]
//! (crux C103): which way a contributor's degrees count, whether the years are
//! taken once or once per agreeing pair, and whether Saturn lowers or raises.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Modality, Rashi};

use crate::chart::PointAt;
use crate::eval::{Evaluator, Participants};
use crate::language::{Body, Condition, House};
use crate::reference::{BodyRef, BodySubject, SignRef, Subject};
use crate::rule::LifeClass;
use teistro_core::catalogue::Point;

/// One of the three pairs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Pair {
    /// The lagna lord and the eighth lord.
    LagnaAndEighthLords,
    /// Saturn and the Moon.
    SaturnAndMoon,
    /// The lagna and the hora lagna.
    LagnaAndHoraLagna,
}

impl Pair {
    /// The three, in the verses' order.
    pub const ALL: [Pair; 3] = [
        Pair::LagnaAndEighthLords,
        Pair::SaturnAndMoon,
        Pair::LagnaAndHoraLagna,
    ];
}

/// The class a pair's two signs give (vv. 33 to 40): both movable, or one
/// fixed and one dual, long; one movable and one fixed, or both dual, medium;
/// one movable and one dual, or both fixed, short. `None` for a modality the
/// verses do not name, which the catalogue could only add.
#[must_use]
pub fn class_of(one: Rashi, other: Rashi) -> Option<LifeClass> {
    use Modality::{Chara, Dwiswabhava, Sthira};
    Some(
        match (one.attributes().modality, other.attributes().modality) {
            (Chara, Chara) | (Sthira, Dwiswabhava) | (Dwiswabhava, Sthira) => LifeClass::Long,
            (Chara, Sthira) | (Sthira, Chara) | (Dwiswabhava, Dwiswabhava) => LifeClass::Medium,
            (Chara, Dwiswabhava) | (Dwiswabhava, Chara) | (Sthira, Sthira) => LifeClass::Short,
            _ => return None,
        },
    )
}

/// The years a class is given by how many pairs gave it (vv. 41 to 44): long
/// 96, 108 and 120 for one, two and three pairs; medium 64, 72 and 80; short
/// 40, 36 and 32. A class outside the three, which only a shift reaches, is
/// given its own span (vv. 53 to 54).
#[must_use]
pub fn years_of(class: LifeClass, pairs: u8) -> Option<f64> {
    let at = usize::from(pairs.clamp(1, 3) - 1);
    let row: [u16; 3] = match class {
        LifeClass::Long => [96, 108, 120],
        LifeClass::Medium => [64, 72, 80],
        LifeClass::Short => [40, 36, 32],
        other => return other.years().map(f64::from),
    };
    row.get(at).copied().map(f64::from)
}

/// The years rectified by the contributors' degrees (vv. 45 to 46): the mean
/// of the degrees, counted as the rules count them, over thirty, times the
/// years or the years once per pair.
///
/// ```
/// use teistro_rules::longevity::{rectify, ThreePairsRules};
///
/// // The translator's example: short life from two pairs, 36 years; four
/// // contributors at 51° 58′ 26″ between them.
/// let sum = 51.0 + 58.0 / 60.0 + 26.0 / 3600.0;
/// let years = rectify(36.0, 2, [sum / 4.0; 4], ThreePairsRules::TRANSLATOR);
/// assert!((years - 31.18).abs() < 0.005, "{years}");
/// ```
#[must_use]
pub fn rectify(
    years: f64,
    pairs: u8,
    degrees: impl IntoIterator<Item = f64>,
    rules: ThreePairsRules,
) -> f64 {
    let (mut sum, mut count) = (0.0, 0_u32);
    for degrees in degrees {
        sum += match rules.rectification {
            Rectification::Remaining => 30.0 - degrees,
            Rectification::Elapsed => degrees,
        };
        count += 1;
    }
    let basis = match rules.basis {
        Basis::ClassYears => years,
        Basis::PerPair => years * f64::from(pairs),
    };
    basis * sum / f64::from(count.max(1)) / 30.0
}

/// Which way a contributor's degrees rectify the years (v. 45).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Rectification {
    /// As the verse says it: a contributor at the start of a sign gives the
    /// whole, one at its end nothing, so the degrees still to run count.
    #[default]
    Remaining,
    /// As the translator's arithmetic runs: the degrees gone count.
    Elapsed,
}

/// What the rectification multiplies (v. 46).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Basis {
    /// The class's years once, as vv. 41 to 44 give them.
    #[default]
    ClassYears,
    /// The class's years once for each pair that gave it, as the translator's
    /// example takes 36 years for two pairs as 72.
    PerPair,
}

/// What Saturn among the contributors does to the class (v. 47).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SaturnAmongThem {
    /// Lowers it a class, as the verse says first.
    #[default]
    Lowers,
    /// Raises it, as "some advocate".
    Raises,
}

/// The choices the three pairs are read under.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ThreePairsRules {
    /// Which way the degrees count.
    pub rectification: Rectification,
    /// What the degrees multiply.
    pub basis: Basis,
    /// What Saturn does.
    pub saturn: SaturnAmongThem,
}

impl ThreePairsRules {
    /// The verses as they read: the degrees to run, the class's years once,
    /// Saturn lowering.
    pub const VERSE: ThreePairsRules = ThreePairsRules {
        rectification: Rectification::Remaining,
        basis: Basis::ClassYears,
        saturn: SaturnAmongThem::Lowers,
    };

    /// The translator's worked example: the degrees gone, the years once per
    /// agreeing pair.
    pub const TRANSLATOR: ThreePairsRules = ThreePairsRules {
        rectification: Rectification::Elapsed,
        basis: Basis::PerPair,
        saturn: SaturnAmongThem::Lowers,
    };
}

/// What one pair stands at.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct PairReading {
    /// Which pair.
    pub pair: Pair,
    /// The two signs, in the verses' order within the pair.
    pub signs: [Rashi; 2],
    /// How far into its sign each stands, degrees.
    pub degrees: [f64; 2],
    /// The class the two signs give.
    pub class: LifeClass,
}

/// How the class was decided.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(tag = "by", rename_all = "kebab-case")]
pub enum Decided {
    /// Two or three pairs agreed.
    Agreement {
        /// How many.
        pairs: u8,
    },
    /// All three differed, and the lagna pair decided (v. 40).
    LagnaPair,
    /// All three differed and the Moon stood in the lagna or the seventh, so
    /// Saturn and the Moon decided (v. 40).
    SaturnAndMoon,
}

/// A shift of class (vv. 47 to 50).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Shift {
    /// Saturn among the contributors, neither in his own sign nor exalted,
    /// nor joined or aspected by malefics alone.
    Saturn,
    /// Jupiter in the lagna or the seventh, joined or aspected by benefics
    /// alone.
    Jupiter,
}

/// The three pairs' reading of a chart.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct ThreePairs {
    /// Each pair, in the verses' order.
    pub pairs: [PairReading; 3],
    /// How the class was decided.
    pub decided: Decided,
    /// The class decided, before any shift.
    pub decided_class: LifeClass,
    /// Which shifts applied, Saturn's first.
    pub shifts: [Option<Shift>; 2],
    /// The class after the shifts.
    pub class: LifeClass,
    /// The years that class is given by the pairs that gave it, or its own
    /// span where a shift left the three classes; none for the illimitable.
    pub years: Option<f64>,
    /// The years rectified by the contributors' degrees, under the rules.
    pub rectified: Option<f64>,
    /// The choices it was read under.
    pub rules: ThreePairsRules,
}

impl Evaluator<'_> {
    /// The three pairs' reading of this chart, or `None` when the evaluator was
    /// given no hora lagna among its points.
    #[must_use]
    pub fn three_pairs(&self, rules: ThreePairsRules) -> Option<ThreePairs> {
        let chart = self.chart();
        let at = |body: Body| {
            let placement = chart.placement(body);
            (placement.sign, placement.longitude)
        };
        let lord = |house: u8| {
            House::try_new(house)
                .ok()
                .map(|house| Body::Graha(self.house_sign(house).attributes().lord))
        };
        let hora: PointAt = self.point(Point::HoraLagna)?;
        let place = |pair: Pair| -> Option<[(Rashi, f64); 2]> {
            Some(match pair {
                Pair::LagnaAndEighthLords => [at(lord(1)?), at(lord(8)?)],
                Pair::SaturnAndMoon => [at(graha(Graha::Saturn)), at(graha(Graha::Moon))],
                Pair::LagnaAndHoraLagna => [at(Body::Lagna), (hora.sign, hora.longitude)],
            })
        };
        let mut readings = [None; 3];
        for (slot, pair) in readings.iter_mut().zip(Pair::ALL) {
            let [(one, first), (other, second)] = place(pair)?;
            *slot = Some(PairReading {
                pair,
                signs: [one, other],
                degrees: [first.rem_euclid(30.0), second.rem_euclid(30.0)],
                class: class_of(one, other)?,
            });
        }
        let [Some(first), Some(second), Some(third)] = readings else {
            return None;
        };
        let pairs = [first, second, third];

        let agreeing = |class: LifeClass| pairs.iter().filter(|pair| pair.class == class).count();
        let (decided, decided_class) = match pairs
            .iter()
            .map(|pair| (pair.class, agreeing(pair.class)))
            .find(|(_, count)| *count >= 2)
        {
            Some((class, count)) => (
                Decided::Agreement {
                    pairs: u8::try_from(count).unwrap_or(3),
                },
                class,
            ),
            None if self.moon_in_lagna_or_seventh() => (Decided::SaturnAndMoon, second.class),
            None => (Decided::LagnaPair, third.class),
        };
        // Which pairs gave the class: the agreeing ones, or the one that
        // decided where all three differed.
        let contributing: [bool; 3] = match decided {
            Decided::Agreement { .. } => pairs.map(|pair| pair.class == decided_class),
            Decided::SaturnAndMoon => [false, true, false],
            Decided::LagnaPair => [false, false, true],
        };
        let counted = contributing.iter().filter(|gave| **gave).count();
        let givers = || {
            pairs
                .iter()
                .zip(contributing)
                .filter_map(|(pair, gave)| gave.then_some(pair))
        };

        let saturn_contributes = givers().any(|pair| match pair.pair {
            Pair::SaturnAndMoon => true,
            Pair::LagnaAndEighthLords => [lord(1), lord(8)].contains(&Some(graha(Graha::Saturn))),
            Pair::LagnaAndHoraLagna => false,
        });
        let mut class = decided_class;
        let mut shifts = [None; 2];
        if saturn_contributes && !self.saturn_exempt() {
            class = match rules.saturn {
                SaturnAmongThem::Lowers => class.lowered(),
                SaturnAmongThem::Raises => class.raised(),
            };
            shifts[0] = Some(Shift::Saturn);
        }
        if self.jupiter_raises() {
            class = class.raised();
            shifts[1] = Some(Shift::Jupiter);
        }

        let counted = u8::try_from(counted).unwrap_or(3);
        let years = years_of(class, counted);
        let rectified = years.map(|years| {
            rectify(
                years,
                counted,
                givers().flat_map(|pair| pair.degrees),
                rules,
            )
        });
        Some(ThreePairs {
            pairs,
            decided,
            decided_class,
            shifts,
            class,
            years,
            rectified,
            rules,
        })
    }

    /// A point the evaluator was given.
    fn point(&self, point: Point) -> Option<PointAt> {
        self.points().iter().find(|at| at.point == point).copied()
    }

    fn moon_in_lagna_or_seventh(&self) -> bool {
        self.holds_now(&in_houses(Graha::Moon, &[1, 7]))
    }

    /// Saturn keeps the class in his own sign or exaltation, or joined or
    /// aspected by malefics alone (v. 47).
    fn saturn_exempt(&self) -> bool {
        let saturn = BodyRef::Body(graha(Graha::Saturn));
        let own = Condition::SameBody {
            of: BodyRef::lord_of(SignRef::Of(saturn.clone())),
            as_body: saturn.clone(),
        };
        let exalted = Condition::SameSign {
            of: SignRef::Of(saturn.clone()),
            as_sign: SignRef::Exaltation(saturn),
        };
        self.holds_now(&own)
            || self.holds_now(&exalted)
            || (self.holds_now(&touched(Graha::Saturn, BodySubject::AnyMalefic))
                && !self.holds_now(&touched(Graha::Saturn, BodySubject::AnyBenefic)))
    }

    /// Jupiter in the lagna or the seventh, joined or aspected by benefics
    /// alone, raises the class (v. 48).
    fn jupiter_raises(&self) -> bool {
        self.holds_now(&in_houses(Graha::Jupiter, &[1, 7]))
            && self.holds_now(&touched(Graha::Jupiter, BodySubject::AnyBenefic))
            && !self.holds_now(&touched(Graha::Jupiter, BodySubject::AnyMalefic))
    }

    fn holds_now(&self, condition: &Condition) -> bool {
        self.holds(condition, &mut Participants::default())
    }
}

const fn graha(graha: Graha) -> Body {
    Body::Graha(graha)
}

/// A graha in one of these houses from the lagna.
fn in_houses(of: Graha, houses: &[u8]) -> Condition {
    Condition::PlanetInHouse {
        planet: Subject::Ref(SignRef::Of(BodyRef::Body(graha(of)))),
        houses: houses
            .iter()
            .filter_map(|house| House::try_new(*house).ok())
            .collect(),
    }
}

/// A graha joined by another of a class, or aspected by one.
fn touched(of: Graha, by: BodySubject) -> Condition {
    let body = BodyRef::Body(graha(of));
    Condition::Or {
        conditions: vec![
            Condition::CountInHouses {
                planets: by.clone(),
                houses: House::try_new(1).into_iter().collect(),
                from: SignRef::Of(body.clone()),
                at_least: 1,
                except: vec![body.clone()],
            },
            Condition::PlanetAspectsPlanet {
                from: by,
                target: SignRef::Of(body),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests unwrap what they built and index their own tallies"
    )]

    use super::*;
    use crate::chart::{Readings, RuleChart};
    use crate::test_chart::{chart, place};

    const MARS: Body = Body::Graha(Graha::Mars);
    const SATURN: Body = Body::Graha(Graha::Saturn);
    const MOON: Body = Body::Graha(Graha::Moon);
    const JUPITER: Body = Body::Graha(Graha::Jupiter);
    const RAHU: Body = Body::Graha(Graha::Rahu);
    const KETU: Body = Body::Graha(Graha::Ketu);

    fn hora(sign: Rashi) -> [PointAt; 1] {
        [PointAt {
            point: Point::HoraLagna,
            longitude: f64::from(sign as u8) * 30.0 + 10.0,
            sign,
        }]
    }

    fn read(chart: &RuleChart, points: &[PointAt], rules: ThreePairsRules) -> Option<ThreePairs> {
        Evaluator::new(chart, Readings::TEXTS)
            .with_points(points)
            .three_pairs(rules)
    }

    #[test]
    fn the_modalities_split_every_pair_of_signs_into_three_equal_classes() {
        let mut counts = [0; 3];
        for one in Rashi::ALL {
            for other in Rashi::ALL {
                let class = class_of(one, other).unwrap();
                assert_eq!(class_of(other, one), Some(class), "{one:?} {other:?}");
                let at = match class {
                    LifeClass::Long => 0,
                    LifeClass::Medium => 1,
                    _ => 2,
                };
                counts[at] += 1;
            }
        }
        assert_eq!(counts, [48, 48, 48]);
        assert_eq!(years_of(LifeClass::Long, 3), Some(120.0));
        assert_eq!(years_of(LifeClass::Short, 1), Some(40.0));
        assert_eq!(years_of(LifeClass::Divine, 2), Some(1000.0));
        assert_eq!(years_of(LifeClass::Unlimited, 1), None);
    }

    #[test]
    fn two_agreeing_pairs_decide_and_their_degrees_rectify_the_years() {
        // Aries rising: Mars lords the lagna and the eighth.
        let mut c = chart();
        place(&mut c, MARS, Rashi::Cancer); // both lords movable: long
        place(&mut c, SATURN, Rashi::Gemini); // dual with a movable Moon: short
        place(&mut c, MOON, Rashi::Cancer);
        place(&mut c, JUPITER, Rashi::Taurus);
        let reading = read(&c, &hora(Rashi::Libra), ThreePairsRules::VERSE).unwrap();
        let classes = reading.pairs.map(|pair| pair.class);
        assert_eq!(
            classes,
            [LifeClass::Long, LifeClass::Short, LifeClass::Long]
        );
        assert_eq!(reading.decided, Decided::Agreement { pairs: 2 });
        assert_eq!(
            (reading.class, reading.shifts),
            (LifeClass::Long, [None, None])
        );
        assert_eq!(reading.years, Some(108.0));
        // Mars at 15° twice, the lagna at 15°, the hora lagna at 10°: 15, 15,
        // 15 and 20 still to run, a mean of 16.25.
        assert!((reading.rectified.unwrap() - 108.0 * 16.25 / 30.0).abs() < 1e-9);
        let elapsed = read(
            &c,
            &hora(Rashi::Libra),
            ThreePairsRules {
                rectification: Rectification::Elapsed,
                ..ThreePairsRules::VERSE
            },
        )
        .unwrap();
        assert!((elapsed.rectified.unwrap() - 108.0 * 13.75 / 30.0).abs() < 1e-9);
    }

    #[test]
    fn with_all_three_apart_the_moon_in_the_seventh_hands_it_to_saturn_who_lowers_it() {
        let mut c = chart();
        place(&mut c, MARS, Rashi::Cancer); // long
        place(&mut c, SATURN, Rashi::Leo); // fixed with a movable Moon: medium
        place(&mut c, MOON, Rashi::Libra); // the seventh from Aries
        place(&mut c, RAHU, Rashi::Taurus);
        place(&mut c, KETU, Rashi::Scorpio);
        // The lagna movable and the hora lagna dual: short. Jupiter in Aries
        // aspects Saturn in Leo, so Saturn is not joined by malefics alone.
        let reading = read(&c, &hora(Rashi::Pisces), ThreePairsRules::VERSE).unwrap();
        assert_eq!(reading.decided, Decided::SaturnAndMoon);
        assert_eq!(reading.decided_class, LifeClass::Medium);
        assert_eq!(reading.shifts, [Some(Shift::Saturn), None]);
        assert_eq!(
            (reading.class, reading.years),
            (LifeClass::Short, Some(40.0))
        );
        // As some advocate, Saturn raises it instead.
        let raised = read(
            &c,
            &hora(Rashi::Pisces),
            ThreePairsRules {
                saturn: SaturnAmongThem::Raises,
                ..ThreePairsRules::VERSE
            },
        )
        .unwrap();
        assert_eq!((raised.class, raised.years), (LifeClass::Long, Some(96.0)));
        // Without the Moon in the lagna or the seventh, the lagna pair decides.
        place(&mut c, MOON, Rashi::Capricorn);
        let lagna = read(&c, &hora(Rashi::Pisces), ThreePairsRules::VERSE).unwrap();
        assert_eq!(lagna.decided, Decided::LagnaPair);
        assert_eq!(lagna.class, LifeClass::Short);
    }

    #[test]
    fn a_chart_without_its_hora_lagna_has_no_three_pairs() {
        assert!(read(&chart(), &[], ThreePairsRules::VERSE).is_none());
    }
}
