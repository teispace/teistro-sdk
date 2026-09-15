//! The Bhava bala: each house's strength, from its lord, its direction and the
//! drishtis it receives (`03-design/bhava-bala-measured.md`).
//!
//! BPHS ch. 27 vv. 26 to 31 give it, and three readings of them are published
//! or recorded, parting at three forks, each a setting in [`BhavaBalaRules`]:
//!
//! - [`BhavaBalaRules::BPHS`], the verses as translated, the default: the Dig
//!   as an arc from the bhava madhyas, a quarter of it for a benefic's or a
//!   malefic's drishti, and the special rules of occupants and rising signs;
//! - [`BhavaBalaRules::SRIPATI`], Sripati's as B.V. Raman works it: the Dig in
//!   ten-virupa house steps and the sphuta drishti on each bhava madhya;
//! - [`BhavaBalaRules::RECORDING_ENGINE`], the conformance corpus's engine,
//!   which this module reproduces on every recorded house, the engine rounding
//!   its answers to hundredths and this module not.
//!
//! The lord's strength is the Shadbala the caller computed, under whatever
//! reading it chose.

use serde::{Deserialize, Serialize};
use teistro_aspect::drishti::quarters;
use teistro_aspect::sphuta;
use teistro_core::catalogue::{Graha, Rashi, Rising};
use teistro_core::settings::{Benefics, BhavaDig, BhavaDrishti, BhavaSpecialRules, Settings};

use crate::ShadbalaReading;
use crate::shadbala::{apart, sign_of};

/// The grahas a bhava's drishtis and occupants are read from: Sun to Saturn,
/// then Rahu and Ketu.
pub const NINE: [Graha; 9] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
    Graha::Rahu,
    Graha::Ketu,
];

/// The ghatis before sunrise and after sunset a twilight birth falls in: two,
/// a muhurta, the sandhya (crux C75).
pub const TWILIGHT_GHATIS: f64 = 2.0;

/// One graha as the Bhava bala reads it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BhavaGraha {
    /// Its sidereal longitude, degrees.
    pub longitude: f64,
    /// The bhava it stands in, 1 to 12.
    pub house: u8,
}

/// What a Bhava bala reads of a chart.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BhavaBalaChart {
    /// The twelve bhava madhyas, sidereal degrees, from the first.
    pub madhya: [f64; 12],
    /// The nine grahas, Sun to Saturn, Rahu and Ketu.
    pub grahas: [BhavaGraha; 9],
    /// Each of Sun to Saturn's Shadbala in virupas, and its Drik bala.
    pub shadbala: [(f64, f64); 7],
    /// The instant, a Julian day (UT).
    pub instant: f64,
    /// The Hindu day's sunrise, sunset and the next sunrise, Julian days (UT),
    /// the instant between the first and the last.
    pub day: (f64, f64, f64),
}

impl BhavaBalaChart {
    /// The strengths a chart's Shadbala gives each graha: its virupas and its
    /// Drik bala, Sun to Saturn.
    #[must_use]
    pub fn strengths(reading: &ShadbalaReading) -> [(f64, f64); 7] {
        let mut out = [(0.0, 0.0); 7];
        for (slot, graha) in out.iter_mut().zip(&reading.grahas) {
            *slot = (graha.virupas, graha.drik);
        }
        out
    }
}

/// The choices a Bhava bala is computed under (cruxes C73 to C75).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct BhavaBalaRules {
    /// How the Dig bala reads a bhava's sign class.
    pub dig: BhavaDig,
    /// How the drishtis a bhava receives are weighed.
    pub drishti: BhavaDrishti,
    /// Whether the special rules of occupants and rising signs are added.
    pub special_rules: BhavaSpecialRules,
    /// Which grahas are benefics, for the Moon's drishti.
    pub benefics: Benefics,
}

impl BhavaBalaRules {
    /// BPHS ch. 27 vv. 26 to 31 as translated.
    pub const BPHS: BhavaBalaRules = BhavaBalaRules {
        dig: BhavaDig::Bphs,
        drishti: BhavaDrishti::QuarterOfDig,
        special_rules: BhavaSpecialRules::Bphs,
        benefics: Benefics::Conditional,
    };

    /// Sripati's, as B.V. Raman works it.
    pub const SRIPATI: BhavaBalaRules = BhavaBalaRules {
        dig: BhavaDig::Sripati,
        drishti: BhavaDrishti::Sphuta,
        special_rules: BhavaSpecialRules::None,
        benefics: Benefics::Conditional,
    };

    /// The conformance corpus's recording engine's.
    pub const RECORDING_ENGINE: BhavaBalaRules = BhavaBalaRules {
        dig: BhavaDig::WholeSign,
        drishti: BhavaDrishti::QuarterOfDig,
        special_rules: BhavaSpecialRules::None,
        benefics: Benefics::Fixed,
    };

    /// The rules a context's settings name.
    #[must_use]
    pub const fn of(settings: &Settings) -> BhavaBalaRules {
        BhavaBalaRules {
            dig: settings.strength.bhava_dig,
            drishti: settings.strength.bhava_drishti,
            special_rules: settings.strength.bhava_special_rules,
            benefics: settings.strength.benefics,
        }
    }
}

/// One bhava's strength, in virupas.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct BhavaStrength {
    /// Which bhava, 1 to 12.
    pub bhava: u8,
    /// The lord of the sign its madhya falls in.
    pub lord: Graha,
    /// The lord's Shadbala.
    pub adhipati: f64,
    /// From its direction, 0 to 60.
    pub dig: f64,
    /// From the drishtis it receives, which may be negative.
    pub drishti: f64,
    /// From its occupants and its sign's rising, under the special rules.
    pub special: f64,
    /// The four together.
    pub virupas: f64,
}

/// A chart's Bhava bala.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct BhavaBalaReading {
    /// The choices it was computed under.
    pub rules: BhavaBalaRules,
    /// Each bhava's, the first to the twelfth.
    pub bhavas: Vec<BhavaStrength>,
}

/// A sign class's weakest house: Nara the seventh, Chatushpada the fourth,
/// Jalachara the tenth and Keeta the first.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Class {
    Nara = 7,
    Chatushpada = 4,
    Jalachara = 10,
    Keeta = 1,
}

/// The class of the sign a madhya falls in, under `rule`.
fn class(madhya: f64, rule: BhavaDig) -> Class {
    let sign = sign_of(madhya);
    let first_half = madhya.rem_euclid(30.0) < 15.0;
    match (sign, rule) {
        (Rashi::Gemini | Rashi::Virgo | Rashi::Libra | Rashi::Aquarius, _)
        | (Rashi::Sagittarius, BhavaDig::WholeSign) => Class::Nara,
        (Rashi::Sagittarius, _) if first_half => Class::Nara,
        (Rashi::Scorpio, _) | (Rashi::Cancer, BhavaDig::Bphs | BhavaDig::WholeSign) => Class::Keeta,
        (Rashi::Pisces | Rashi::Cancer, _) => Class::Jalachara,
        (Rashi::Capricorn, BhavaDig::WholeSign) => Class::Chatushpada,
        (Rashi::Capricorn, _) if !first_half => Class::Jalachara,
        _ => Class::Chatushpada,
    }
}

/// A bhava's Dig bala (vv. 26 to 28).
fn dig(chart: &BhavaBalaChart, bhava: usize, rule: BhavaDig) -> f64 {
    let madhya = chart.madhya.get(bhava).copied().unwrap_or_default();
    let weakest = class(madhya, rule) as usize;
    if rule == BhavaDig::Bphs {
        let from = chart.madhya.get(weakest - 1).copied().unwrap_or_default();
        return apart(madhya, from) / 3.0;
    }
    let steps = (bhava + 1).abs_diff(weakest);
    #[allow(clippy::cast_precision_loss, reason = "0 to 6")]
    let steps = steps.min(12 - steps) as f64;
    steps * 10.0
}

/// Whether a graha's drishti counts as a benefic's for a bhava.
fn benefic(graha: Graha, chart: &BhavaBalaChart, rules: BhavaBalaRules) -> bool {
    match graha {
        Graha::Jupiter | Graha::Venus | Graha::Mercury => true,
        Graha::Moon => {
            rules.benefics == Benefics::Fixed || {
                let [sun, moon, ..] = chart.grahas;
                (84.0..276.0).contains(&(moon.longitude - sun.longitude).rem_euclid(360.0))
            }
        }
        _ => false,
    }
}

/// A bhava's drishti bala (v. 29).
fn drishti(chart: &BhavaBalaChart, bhava: usize, dig: f64, rules: BhavaBalaRules) -> f64 {
    let drik_of = |graha: Graha| {
        crate::ashtakavarga::GRAHAS
            .iter()
            .zip(chart.shadbala)
            .find_map(|(g, (_, drik))| (*g == graha).then_some(drik))
            .unwrap_or_default()
    };
    if rules.drishti == BhavaDrishti::Sphuta {
        let madhya = chart.madhya.get(bhava).copied().unwrap_or_default();
        return crate::ashtakavarga::GRAHAS
            .iter()
            .zip(chart.grahas)
            .map(|(graha, at)| {
                let virupas = sphuta::drishti(*graha, at.longitude, madhya);
                let weight = if matches!(graha, Graha::Jupiter | Graha::Mercury) {
                    1.0
                } else {
                    0.25
                };
                let sign = if benefic(*graha, chart, rules) {
                    1.0
                } else {
                    -1.0
                };
                sign * weight * virupas
            })
            .sum();
    }
    let house = u8::try_from(bhava + 1).unwrap_or(1);
    let aspecting: Vec<Graha> = NINE
        .iter()
        .zip(chart.grahas)
        .filter(|(graha, at)| quarters(**graha, (house + 12 - at.house) % 12 + 1).is_full())
        .map(|(graha, _)| *graha)
        .collect();
    let any_benefic = aspecting.iter().any(|g| benefic(*g, chart, rules));
    let any_malefic = aspecting.iter().any(|g| !benefic(*g, chart, rules));
    let quarter = match (any_benefic, any_malefic) {
        (true, false) => dig / 4.0,
        (false, true) => -dig / 4.0,
        _ => 0.0,
    };
    let full: f64 = aspecting
        .iter()
        .filter(|g| matches!(g, Graha::Jupiter | Graha::Mercury))
        .map(|g| drik_of(*g))
        .sum();
    quarter + full
}

/// A bhava's special rules (vv. 30 and 31).
fn special(chart: &BhavaBalaChart, bhava: usize) -> f64 {
    let house = u8::try_from(bhava + 1).unwrap_or(1);
    let occupants: f64 = NINE
        .iter()
        .zip(chart.grahas)
        .filter(|(_, at)| at.house == house)
        .map(|(graha, _)| match graha {
            Graha::Jupiter | Graha::Mercury => 60.0,
            Graha::Sun | Graha::Mars | Graha::Saturn => -60.0,
            _ => 0.0,
        })
        .sum();
    let (_, sunset, next_sunrise) = chart.day;
    let twilight = TWILIGHT_GHATIS / 60.0;
    let rising = sign_of(chart.madhya.get(bhava).copied().unwrap_or_default())
        .attributes()
        .rising;
    // The Hindu day holds the instant: dusk is the first two ghatis after
    // its sunset, and dawn the last two before the next sunrise.
    let in_twilight = (chart.instant >= sunset && chart.instant < sunset + twilight)
        || chart.instant >= next_sunrise - twilight;
    let by_day = !in_twilight && chart.instant < sunset;
    let fits = match rising {
        Rising::Sirshodaya => by_day,
        Rising::Prishtodaya => !by_day && !in_twilight,
        _ => in_twilight,
    };
    occupants + if fits { 15.0 } else { 0.0 }
}

impl BhavaBalaReading {
    /// A chart's Bhava bala under `rules`.
    #[must_use]
    pub fn of(chart: &BhavaBalaChart, rules: BhavaBalaRules) -> BhavaBalaReading {
        let bhavas = (0..12)
            .map(|bhava| {
                let madhya = chart.madhya.get(bhava).copied().unwrap_or_default();
                let lord = sign_of(madhya).attributes().lord;
                let adhipati = crate::ashtakavarga::GRAHAS
                    .iter()
                    .zip(chart.shadbala)
                    .find_map(|(g, (virupas, _))| (*g == lord).then_some(virupas))
                    .unwrap_or_default();
                let dig = dig(chart, bhava, rules.dig);
                // Adding a positive zero keeps a malefic's quarter of no Dig, or a
                // sum of nothing, from reading as a negative zero.
                let drishti = drishti(chart, bhava, dig, rules) + 0.0;
                let special = if rules.special_rules == BhavaSpecialRules::Bphs {
                    special(chart, bhava)
                } else {
                    0.0
                };
                BhavaStrength {
                    bhava: u8::try_from(bhava + 1).unwrap_or(1),
                    lord,
                    adhipati,
                    dig,
                    drishti,
                    special,
                    virupas: adhipati + dig + drishti + special,
                }
            })
            .collect();
        BhavaBalaReading { rules, bhavas }
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::float_cmp,
        clippy::indexing_slicing,
        reason = "tests compare exact values and index what they built"
    )]

    use super::*;

    /// A Mesha-rising whole-sign chart, every graha in the first house, at
    /// noon, each graha's Shadbala 300 with no Drik.
    fn chart() -> BhavaBalaChart {
        BhavaBalaChart {
            madhya: std::array::from_fn(|i| 15.0 + 30.0 * f64::from(u8::try_from(i).unwrap_or(0))),
            grahas: [BhavaGraha {
                longitude: 10.0,
                house: 1,
            }; 9],
            shadbala: [(300.0, 0.0); 7],
            instant: 2_451_545.0,
            day: (2_451_544.75, 2_451_545.25, 2_451_545.75),
        }
    }

    #[test]
    fn the_dig_follows_each_reading_s_classes() {
        let chart = chart();
        // The fourth bhava is Cancer: an insect for BPHS and the engine, its
        // weakest house the first; watery for Sripati, the tenth.
        assert_eq!(dig(&chart, 3, BhavaDig::WholeSign), 30.0);
        assert_eq!(dig(&chart, 3, BhavaDig::Sripati), 60.0);
        assert_eq!(dig(&chart, 3, BhavaDig::Bphs), 30.0);
        // The ninth is Sagittarius at 15°, its second half: a quadruped's
        // weakest house the fourth, five houses away; the engine's human.
        assert_eq!(dig(&chart, 8, BhavaDig::Sripati), 50.0);
        assert_eq!(dig(&chart, 8, BhavaDig::WholeSign), 20.0);
    }

    #[test]
    fn a_quarter_of_the_dig_follows_a_one_sided_drishti() {
        let mut chart = chart();
        let rules = BhavaBalaRules::RECORDING_ENGINE;
        // Nothing aspects the seventh from the first but everything there.
        let dig = super::dig(&chart, 6, rules.dig);
        let mixed = drishti(&chart, 6, dig, rules);
        assert_eq!(mixed, 0.0);
        // Jupiter alone in the first: a benefic's quarter and its Drik bala.
        for at in &mut chart.grahas {
            at.house = 2;
        }
        chart.grahas[4].house = 1;
        chart.shadbala[4].1 = 12.0;
        assert_eq!(drishti(&chart, 6, 40.0, rules), 10.0 + 12.0);
    }

    #[test]
    fn the_special_rules_add_occupants_and_rising() {
        let mut chart = chart();
        for at in &mut chart.grahas {
            at.house = 2;
        }
        chart.grahas[4].house = 1;
        chart.grahas[0].house = 1;
        // Aries is prishtodaya and the birth is at noon: the occupants alone.
        assert_eq!(special(&chart, 0), 0.0);
        // Gemini is sirshodaya: 15 by day.
        assert_eq!(special(&chart, 2), 15.0);
        // Pisces is ubhayodaya: 15 just after sunset.
        chart.instant = chart.day.1 + 0.01;
        assert_eq!(special(&chart, 11), 15.0);
    }

    #[test]
    fn every_bhava_s_total_is_its_components() {
        for rules in [
            BhavaBalaRules::BPHS,
            BhavaBalaRules::SRIPATI,
            BhavaBalaRules::RECORDING_ENGINE,
        ] {
            let reading = BhavaBalaReading::of(&chart(), rules);
            assert_eq!(reading.bhavas.len(), 12);
            for bhava in reading.bhavas {
                assert_eq!(
                    bhava.virupas,
                    bhava.adhipati + bhava.dig + bhava.drishti + bhava.special
                );
            }
        }
    }
}
