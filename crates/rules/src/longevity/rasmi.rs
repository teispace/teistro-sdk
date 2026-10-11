//! The rays of the seven grahas, the class of life their sum gives, and the
//! years they give (*Jataka Parijata* ch. 5 vv. 22 to 25, C305 to C307).
//!
//! Each graha has its rays at deep exaltation (v. 22): the Sun 10, the Moon
//! 9, Mars 5, Mercury 5, Jupiter 7, Venus 8 and Saturn 5. The chapter counts
//! two things from them on facing pages (C305). The translator's note
//! (pp. 250 to 254), after Mahendra, Manittha, Maya, Yavana and Badarayana,
//! counts **rays**, nothing at deep debilitation and in proportion between,
//! and works a full figure. v. 23 counts **years**, Rasmija ayurdaya, half
//! at deep debilitation as Pindayu's are, and works the Sun of the same
//! figure to 9.363 years. Both are reported, each in its own unit.
//!
//! Both are doubled in a graha's own place, its exaltation or a great
//! friend's, or when it is retrograde, and lose a share in an enemy's; the
//! note reads the place by the dwadasamsa, v. 24 by the sign (C306). The
//! rays' total grades a life by Jatakadesa's bands, which the note quotes:
//! more than 25 rays long, 15 to 25 medium, fewer short (C307).
//!
//! Not read: the eighth both lose "when the retrograde motion of a planet is
//! about to cease", which needs a graha's speed, and a rule chart carries
//! only whether it is retrograde.
//!
//! Every step is reported, so a reader sees why a graha has the rays it
//! has, and the class is a clause of the total, never a verdict on the
//! chart.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi, Relationship};
use teistro_state::dignity::{compound, natural, temporary_from};

use super::ayurdaya::{by_exaltation, deep_exaltation};
use crate::chart::{Placement, RuleChart};
use crate::eval::Evaluator;
use crate::language::Body;
use crate::rule::LifeClass;

/// Each graha's rays at deep exaltation (v. 22), the Sun to Saturn.
const RAYS: [(Graha, f64); 7] = [
    (Graha::Sun, 10.0),
    (Graha::Moon, 9.0),
    (Graha::Mars, 5.0),
    (Graha::Mercury, 5.0),
    (Graha::Jupiter, 7.0),
    (Graha::Venus, 8.0),
    (Graha::Saturn, 5.0),
];

/// Which place doubles a graha's rays or takes a share of them (C306).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum RayPlace {
    /// The dwadasamsa, as the note reads it: its own, its exaltation or a
    /// great friend's doubles, and an enemy's or its debilitation takes a
    /// sixteenth.
    #[default]
    Dwadasamsa,
    /// The sign, as v. 24 reads it: its own, its exaltation or a very
    /// friendly one doubles, and an enemy's takes a twelfth.
    Sign,
}

/// The choices the rays are read under. Every field is optional where a
/// request reads it, and the defaults are the note's, whose figure the
/// kernel reproduces.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct RasmiRules {
    /// Which place doubles or reduces them.
    pub place: RayPlace,
}

impl RasmiRules {
    /// v. 24 as the verse reads: the place by the sign.
    pub const VERSE: RasmiRules = RasmiRules {
        place: RayPlace::Sign,
    };
}

/// Whether a graha is rising towards its exaltation or falling from it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum Facing {
    /// Abhimukha: risen from its debilitation and going towards its
    /// exaltation, whose rays the note calls beneficent.
    Towards,
    /// Parangmukha: fallen from its exaltation and going towards its
    /// debilitation.
    Away,
}

/// One graha's rays and years, step by step.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GrahaRays {
    /// Which graha.
    pub graha: Graha,
    /// Its rays by its distance from deep debilitation, before its place:
    /// nothing there.
    pub basic: f64,
    /// Its years by the same distance, before its place: half there (v. 23).
    pub basic_years: f64,
    /// Whether it rises towards its exaltation or falls from it.
    pub facing: Facing,
    /// Whether its place or its retrogression doubled them; only once,
    /// however many reasons.
    pub doubled: bool,
    /// The share an enemy's place took: a sixteenth by the dwadasamsa, a
    /// twelfth by the sign, or none.
    pub enemy_share: f64,
    /// Whether the Sun's burning halved them; never Venus or Saturn (v. 25).
    pub eclipsed: bool,
    /// The rays it has.
    pub rays: f64,
    /// The years it gives, under the same doubling and shares.
    pub years: f64,
}

/// The seven grahas' rays, the class of life their sum gives, and the years
/// they give.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Rasmi {
    /// The seven, the Sun to Saturn.
    pub grahas: [GrahaRays; 7],
    /// The sum of their rays.
    pub total: f64,
    /// Jatakadesa's class for the total: long above 25, medium from 15 to
    /// 25, short below 15.
    pub class: LifeClass,
    /// The sum of their years, Rasmija ayurdaya, in years of 360 days.
    pub years: f64,
    /// The choices they were read under.
    pub rules: RasmiRules,
}

/// The class of life a total of rays gives (Jatakadesa, quoted p. 251):
/// "exceed 25" long, "between 15 and 25" medium, "short of 15" short.
#[must_use]
pub fn class_of_rays(total: f64) -> LifeClass {
    if total > 25.0 {
        LifeClass::Long
    } else if total >= 15.0 {
        LifeClass::Medium
    } else {
        LifeClass::Short
    }
}

/// A graha's rays by its place in the zodiac alone, before its place's
/// lord or its motion: `full` at deep exaltation and nothing at deep
/// debilitation, in proportion between. Its years by the same place are
/// [`by_exaltation`]'s, half at debilitation (v. 23).
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_rules::longevity::{by_exaltation, rays_by_distance};
///
/// // The note's Sun at 1s 2° 55′ 30″ is 5s 7° 4′ 30″ from his
/// // debilitation: 8.7264 of his 10 rays (p. 252), and 9.363 years by
/// // v. 23 (p. 254).
/// let sun = 32.0 + 55.0 / 60.0 + 30.0 / 3600.0;
/// let rays = rays_by_distance(Graha::Sun, sun, 10.0).unwrap();
/// let years = by_exaltation(Graha::Sun, sun, 10.0).unwrap();
/// assert!((rays - 8.7264).abs() < 1e-4 && (years - 9.363).abs() < 1e-3);
/// ```
#[must_use]
pub fn rays_by_distance(graha: Graha, longitude: f64, full: f64) -> Option<f64> {
    // Pindayu's proportion keeps half at debilitation; the rays stretch the
    // same proportion to nothing there.
    Some(2.0 * by_exaltation(graha, longitude, full)? - full)
}

/// The dwadasamsa sign of a longitude: the sign itself for its first
/// twelfth, and each twelfth after it the next sign.
#[must_use]
pub fn dwadasamsa_of(longitude: f64) -> Rashi {
    let longitude = longitude.rem_euclid(360.0);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a longitude under 360 over thirty is 0 to 11, and its degrees over 2.5 are 0 to 11"
    )]
    let (sign, part) = ((longitude / 30.0) as u16, ((longitude % 30.0) / 2.5) as u16);
    Rashi::from_id((sign.min(11) + part.min(11)) % 12).unwrap_or(Rashi::Aries)
}

/// How a graha stands to the place its rays are read by.
enum Standing {
    /// Its own, its exaltation or a great friend's: doubled.
    Doubles,
    /// An enemy's, or by the dwadasamsa its debilitation: a share taken.
    Reduces,
    /// Neither.
    Neither,
}

/// How `graha`, seated in the sign `seat`, stands to the place `place`,
/// by the compound friendship with its lord; `sign_of` places that lord
/// for the temporary friendship, counted from the graha's seat.
///
/// Only a great friend doubles. v. 24 says "a very friendly house", and the
/// note, though it writes "a friendly planet", calls the Sun's dwadasamsa
/// of Mercury, a compound friend, "a neutral planet's" and Venus's of
/// Saturn "a very friendly planet's" (pp. 252 to 253, C306).
fn standing(
    graha: Graha,
    seat: Rashi,
    place: Rashi,
    by: RayPlace,
    sign_of: impl Fn(Graha) -> Option<Rashi>,
) -> Standing {
    let attributes = graha.attributes();
    if attributes.own.contains(&place)
        || attributes
            .exaltation
            .is_some_and(|point| point.sign == place)
    {
        return Standing::Doubles;
    }
    if by == RayPlace::Dwadasamsa
        && attributes
            .debilitation
            .is_some_and(|point| point.sign == place)
    {
        return Standing::Reduces;
    }
    let lord = place.attributes().lord;
    let Some(temporary) = temporary_from(graha, lord, seat, sign_of) else {
        return Standing::Neither;
    };
    match compound(natural(graha, place), temporary) {
        Relationship::GreatFriend => Standing::Doubles,
        Relationship::Enemy | Relationship::GreatEnemy => Standing::Reduces,
        _ => Standing::Neither,
    }
}

/// One graha's rays and years under the rules.
fn graha_rays(
    graha: Graha,
    full: f64,
    at: &Placement,
    chart: &RuleChart,
    rules: RasmiRules,
) -> GrahaRays {
    let basic = rays_by_distance(graha, at.longitude, full).unwrap_or(full);
    let basic_years = by_exaltation(graha, at.longitude, full).unwrap_or(full);
    let sign_of = |other: Graha| Some(chart.placement(Body::Graha(other)).sign);
    let (place, share) = match rules.place {
        RayPlace::Dwadasamsa => (dwadasamsa_of(at.longitude), 1.0 / 16.0),
        RayPlace::Sign => (at.sign, 1.0 / 12.0),
    };
    let (by_place, enemy_share) = match standing(graha, at.sign, place, rules.place, sign_of) {
        Standing::Doubles => (true, 0.0),
        Standing::Reduces => (false, share),
        Standing::Neither => (false, 0.0),
    };
    let doubled = by_place || at.retrograde;
    let eclipsed = at.combust && !matches!(graha, Graha::Venus | Graha::Saturn);
    let factor =
        if doubled { 2.0 } else { 1.0 } * (1.0 - enemy_share) * if eclipsed { 0.5 } else { 1.0 };
    GrahaRays {
        graha,
        basic,
        basic_years,
        facing: facing(graha, at.longitude),
        doubled,
        enemy_share,
        eclipsed,
        rays: basic * factor,
        years: basic_years * factor,
    }
}

/// Whether a longitude lies on the half of the zodiac a graha climbs from
/// its debilitation to its exaltation, the zodiac's order taken as its way.
fn facing(graha: Graha, longitude: f64) -> Facing {
    match deep_exaltation(graha) {
        Some(point) if (point - longitude).rem_euclid(360.0) < 180.0 => Facing::Towards,
        _ => Facing::Away,
    }
}

impl Evaluator<'_> {
    /// The seven grahas' rays, the class of life their total gives and the
    /// years they give, under `rules` (*Jataka Parijata* ch. 5 vv. 22 to
    /// 25).
    #[must_use]
    pub fn rasmi(&self, rules: RasmiRules) -> Rasmi {
        let chart = self.chart();
        let grahas = RAYS.map(|(graha, full)| {
            graha_rays(
                graha,
                full,
                chart.placement(Body::Graha(graha)),
                chart,
                rules,
            )
        });
        let total = grahas.iter().map(|graha| graha.rays).sum();
        Rasmi {
            grahas,
            total,
            class: class_of_rays(total),
            years: grahas.iter().map(|graha| graha.years).sum(),
            rules,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bands_are_jatakadesa_s_and_rays_and_years_meet_at_exaltation() {
        assert_eq!(class_of_rays(25.0001), LifeClass::Long);
        assert_eq!(class_of_rays(25.0), LifeClass::Medium);
        assert_eq!(class_of_rays(15.0), LifeClass::Medium);
        assert_eq!(class_of_rays(14.9999), LifeClass::Short);
        let near = |value: Option<f64>, to: f64| value.is_some_and(|got| (got - to).abs() < 1e-12);
        assert!(near(rays_by_distance(Graha::Sun, 10.0, 10.0), 10.0));
        assert!(near(by_exaltation(Graha::Sun, 10.0, 10.0), 10.0));
        assert!(near(rays_by_distance(Graha::Sun, 190.0, 10.0), 0.0));
        assert!(near(by_exaltation(Graha::Sun, 190.0, 10.0), 5.0));
        // A dwadasamsa runs on from the sign: 13° 10′ Aries is Virgo's.
        assert_eq!(dwadasamsa_of(13.0 + 10.0 / 60.0), Rashi::Virgo);
        assert_eq!(dwadasamsa_of(359.9), Rashi::Aquarius);
    }
}
