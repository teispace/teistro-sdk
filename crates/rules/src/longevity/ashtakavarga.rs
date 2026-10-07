//! The spans of life the ashtakavargas give (*Jataka Parijata* ch. 10,
//! C311 to C313): each graha's years from its own reduced ashtakavarga
//! (vv. 44 to 46), whose sum v. 33 calls Bhinnashtakavargaja, and the
//! gathered ashtakavarga's, Samudaya (vv. 70 and 71).
//!
//! In a graha's own ashtakavarga after the trine and single-lord reductions
//! (vv. 39 to 42), each sign's bindus are multiplied by the sign's measure
//! and the bindus in the signs the seven occupy by the occupant's; the sum
//! over 30 is the graha's years, twelves cast out (vv. 44 and 45). The
//! figure (p. 690) gives the Sun 162 and 90, 8.4 years. The translator's
//! note gives Balabhadra's and Mantreswara's divisor, 7 over 27 with
//! twenty-sevens cast out (C312). v. 46 doubles them at exaltation and
//! halves them at debilitation, in proportion between, halves them when
//! eclipsed and doubles retrograde Mars (C313). The note's own reductions,
//! the greatest of a half or a third, are the other choice (C317); 7 over
//! 27 counts nakshatra years, converted to solar ones by 324 over 365.
//!
//! Samudaya reduces the gathered ashtakavarga the same two ways, casts the
//! twelves out of each sign keeping a 12 (v. 70), and takes the same
//! products times 7 over 27, less 100 years above 100 (v. 71), once: the
//! verse says nothing of a product above 200 (C314). The verse counts it
//! in nakshatra years of 324 days and gives it in solar years by 324 over
//! 365, both reported.
//!
//! The sign measures are *Jataka Parijata*'s, Virgo 5, where BPHS's
//! translator reads 6 (C311). The lagna's own years, which "some" add
//! (v. 48), come from its own ashtakavarga, Parasara's as the note gives it
//! (p. 693), reported beside the seven's and never added to them. v. 50's
//! other span counts each graha's bindus in the seven's signs, reduced by
//! vv. 39 to 42 and its years by v. 46 or the note (C318), and v. 49's two
//! clauses for when the ashtakavarga span applies are reported.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::settings::Ekadhipatya;
use teistro_strength::ashtakavarga::{AshtakavargaReading, GRAHA_MEASURES, RASHI_MEASURES, reduce};

use super::ayurdaya::{Enmity, by_exaltation, in_enemy_sign};
use crate::eval::Evaluator;
use crate::language::{Body, House};
use crate::reference::Class;

/// The seven, the Sun to Saturn, as an ashtakavarga lists them.
const GRAHAS: [Graha; 7] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];

/// Each sign's measure as *Jataka Parijata* ch. 10 v. 44 prints it, Aries
/// to Pisces: Virgo 5.
pub const PARIJATA_RASHI_MEASURES: [u16; 12] = [7, 10, 8, 4, 10, 5, 7, 8, 9, 5, 11, 12];

/// The nine, whose company v. 49 and Balabhadra's reductions count.
const NINE: [Graha; 9] = [
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

/// A nakshatra year's days, twelve months of 27, in which v. 71 counts
/// Samudaya.
const NAKSHATRA_YEAR: f64 = 324.0;
/// A year of 360 days, in which v. 34 counts a span.
const SAVANA_YEAR: f64 = 360.0;
/// A solar year's days as v. 71 converts to them.
const SOLAR_YEAR: f64 = 365.0;

/// Which table the signs are measured by (C311).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SignMeasures {
    /// *Jataka Parijata*'s, Virgo 5.
    #[default]
    Parijata,
    /// BPHS's translator's, Virgo 6, which the SDK's pindas use.
    Bphs,
}

impl SignMeasures {
    /// The twelve measures, Aries to Pisces.
    #[must_use]
    pub const fn table(self) -> [u16; 12] {
        match self {
            SignMeasures::Parijata => PARIJATA_RASHI_MEASURES,
            SignMeasures::Bphs => RASHI_MEASURES,
        }
    }
}

/// How a graha's pinda becomes years (C312).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Divisor {
    /// Over 30, twelves cast out (vv. 44 and 45).
    #[default]
    Thirty,
    /// Times 7 over 27, twenty-sevens cast out (Balabhadra and Mantreswara,
    /// in the note).
    SevenOverTwentySeven,
}

/// How each graha's ashtakavarga years are reduced (C317).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AshtakaReductions {
    /// v. 46: twice at exaltation, half at debilitation or eclipsed, in
    /// proportion between, and retrograde Mars doubled.
    #[default]
    Verse,
    /// Balabhadra's and Mantreswara's, in the note (p. 691): half for
    /// another graha in the bhava, at debilitation or combust; a third in a
    /// natural enemy's sign or the visible half, and for the Sun or the
    /// Moon in a node's sign; the greatest only.
    Balabhadra,
}

/// Which bindus v. 50's span counts in the seven's signs (C318).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OccupiedBindus {
    /// After the trine and single-lord reductions (vv. 39 to 42), which
    /// "the reductions mentioned already" name first.
    #[default]
    Reduced,
    /// As the ashtakavarga gives them.
    Raw,
}

/// The lagna's own ashtakavarga, Parasara's as the note to v. 48 gives it
/// (p. 693): the houses from the Sun, the Moon, Mars, Mercury, Jupiter,
/// Venus, Saturn and the lagna itself in which each gives the lagna a
/// bindu, 49 in all.
pub const LAGNA_ASHTAKAVARGA: [&[u8]; 8] = [
    &[3, 4, 6, 10, 11, 12],
    &[3, 6, 10, 11, 12],
    &[1, 3, 6, 10, 11],
    &[1, 2, 4, 6, 8, 10, 11],
    &[1, 2, 4, 5, 6, 7, 9, 10, 11],
    &[1, 2, 3, 4, 5, 8, 9],
    &[1, 3, 4, 6, 10, 11],
    &[3, 6, 10, 11],
];

/// The lagna's ashtakavarga for the seven's `signs`, the Sun to Saturn, and
/// the `lagna`'s, Aries to Pisces.
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_rules::longevity::lagna_bindus;
///
/// let bindus = lagna_bindus(&[Rashi::Aries; 7], Rashi::Aries);
/// assert_eq!(bindus.iter().map(|b| u32::from(*b)).sum::<u32>(), 49);
/// // Every contributor in Aries: the table's own column of sums.
/// assert_eq!(bindus, [5, 3, 6, 5, 2, 7, 1, 2, 2, 7, 7, 2]);
/// ```
#[must_use]
pub fn lagna_bindus(signs: &[Rashi; 7], lagna: Rashi) -> [u8; 12] {
    let mut bindus = [0_u8; 12];
    for (from, houses) in signs.iter().chain([&lagna]).zip(LAGNA_ASHTAKAVARGA) {
        for house in houses {
            let sign = (*from as usize + usize::from(*house) - 1) % 12;
            if let Some(cell) = bindus.get_mut(sign) {
                *cell += 1;
            }
        }
    }
    bindus
}

/// The choices the ashtakavarga spans are read under.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AshtakavargaAyusRules {
    /// Which sign measures.
    pub measures: SignMeasures,
    /// How a graha's pinda becomes years.
    pub divisor: Divisor,
    /// How each graha's years are reduced.
    pub reductions: AshtakaReductions,
    /// Which bindus v. 50's span counts.
    pub occupied: OccupiedBindus,
}

/// A chart's raw ashtakavargas and how they are reduced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bindus {
    /// Each graha's bindus by sign, the Sun to Saturn, Aries to Pisces.
    pub grahas: [[u8; 12]; 7],
    /// How a co-ruled sign beside an occupied one is reduced.
    pub ekadhipatya: Ekadhipatya,
}

impl From<&AshtakavargaReading> for Bindus {
    /// A chart's computed ashtakavarga's raw bindus, under the Ekadhipatya
    /// rule it was reduced by; a graha it does not list has none.
    fn from(reading: &AshtakavargaReading) -> Bindus {
        Bindus {
            grahas: GRAHAS.map(|graha| {
                reading
                    .grahas
                    .iter()
                    .find(|row| row.graha == graha)
                    .map_or([0; 12], |row| row.bindus)
            }),
            ekadhipatya: reading.rules.ekadhipatya,
        }
    }
}

/// One graha's years from its ashtakavarga, step by step.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct AshtakaGiver {
    /// Which graha.
    pub graha: Graha,
    /// Its reduced bindus times the sign measures.
    pub rashi_pinda: u32,
    /// Its reduced bindus in the seven's signs times their measures.
    pub graha_pinda: u32,
    /// The two as years, before v. 46.
    pub basic: f64,
    /// v. 46's factor: 2 at exaltation, ½ at debilitation or eclipsed, 2
    /// for retrograde Mars.
    pub factor: f64,
    /// Its years.
    pub years: f64,
    /// Its bindus in the seven's signs (v. 50).
    pub occupied_bindus: u32,
    /// Those as years under the same factor (v. 50).
    pub occupied_years: f64,
}

/// The lagna's years from its own ashtakavarga (v. 48), as a graha's are.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct LagnaAshtaka {
    /// Its bindus by sign, Aries to Pisces, before the reductions.
    pub bindus: [u8; 12],
    /// Its reduced bindus times the sign measures.
    pub rashi_pinda: u32,
    /// Its reduced bindus in the seven's signs times their measures.
    pub graha_pinda: u32,
    /// The two as years, which "some" add to the seven's (v. 48).
    pub years: f64,
}

/// When v. 49 calls for the ashtakavarga span, clause by clause.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CalledFor {
    /// The Moon outside the kendras with another graha in her sign.
    pub moon_joined_outside_a_kendra: bool,
    /// The tenth occupied by a benefic and a malefic, by the readings'
    /// classes.
    pub tenth_benefic_and_malefic: bool,
}

/// The ashtakavarga spans of a chart.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct AshtakavargaAyus {
    /// The seven, the Sun to Saturn.
    pub grahas: [AshtakaGiver; 7],
    /// Their sum, Bhinnashtakavargaja (v. 48).
    pub bhinna: f64,
    /// That sum in solar years: years of 360 days over 30, by v. 34, and
    /// nakshatra years of 324 over 7 by 27, as the note converts them.
    pub bhinna_solar: f64,
    /// v. 50's span: the seven's bindus in their own signs, as years, under
    /// the same factors.
    pub occupied: f64,
    /// The lagna's years (v. 48), which "some" add to the seven's.
    pub lagna: LagnaAshtaka,
    /// When v. 49 calls for the ashtakavarga span.
    pub called_for: CalledFor,
    /// The gathered ashtakavarga's span, Samudaya (v. 71): the product
    /// below, less a hundred years once when over a hundred (C314).
    pub samudaya: f64,
    /// The gathered product times 7 over 27, before v. 71 takes a hundred
    /// off it.
    pub samudaya_product: f64,
    /// Samudaya in solar years: v. 71 calls it Nakshatra Ayus, in years of
    /// 324 days, "which when multiplied by 324 and divided by 365 will give
    /// the period of life correctly in solar years" (p. 706).
    pub samudaya_solar: f64,
    /// The choices they were read under.
    pub rules: AshtakavargaAyusRules,
}

/// The two pindas of a reduced row (v. 44): each sign's figure times its
/// measure, and the figures in the seven's `signs`, the Sun to Saturn, times
/// theirs.
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_rules::longevity::{Divisor, SignMeasures, pinda_years, pindas};
///
/// // The figure (p. 690): the Sun's reduced ashtakavarga, Aries to Pisces,
/// // and the Sun in Cancer, the Moon in Pisces, Mars in Scorpio, Mercury
/// // and Venus in Gemini, Jupiter in Libra and Saturn in Aquarius.
/// let sun = [2, 0, 2, 2, 3, 0, 2, 2, 2, 0, 2, 2];
/// let signs = [Rashi::Cancer, Rashi::Pisces, Rashi::Scorpio, Rashi::Gemini,
///              Rashi::Libra, Rashi::Gemini, Rashi::Aquarius];
/// let (rashi, graha) = pindas(&sun, &signs, SignMeasures::Parijata);
/// assert_eq!((rashi, graha), (162, 90));
/// assert!((pinda_years(rashi + graha, Divisor::Thirty) - 8.4).abs() < 1e-12);
/// ```
#[must_use]
pub fn pindas(row: &[u16; 12], signs: &[Rashi; 7], measures: SignMeasures) -> (u32, u32) {
    let rashi = row
        .iter()
        .zip(measures.table())
        .map(|(figure, measure)| u32::from(*figure) * u32::from(measure))
        .sum();
    let graha = signs
        .iter()
        .zip(GRAHA_MEASURES)
        .map(|(sign, measure)| {
            row.get(*sign as usize % 12)
                .map_or(0, |figure| u32::from(*figure) * u32::from(measure))
        })
        .sum();
    (rashi, graha)
}

/// A pinda as years under a divisor, its cycles cast out.
#[must_use]
pub fn pinda_years(pinda: u32, divisor: Divisor) -> f64 {
    let pinda = f64::from(pinda);
    match divisor {
        Divisor::Thirty => cast_out(pinda / 30.0, 12.0),
        Divisor::SevenOverTwentySeven => cast_out(pinda * 7.0 / 27.0, 27.0),
    }
}

/// A number of years less whole cycles, when over one.
fn cast_out(years: f64, cycle: f64) -> f64 {
    if years > cycle {
        years.rem_euclid(cycle)
    } else {
        years
    }
}

impl Evaluator<'_> {
    /// The ashtakavarga spans of the chart from its raw `bindus`, under
    /// `rules` (*Jataka Parijata* ch. 10 vv. 44 to 46, 70 and 71).
    #[must_use]
    pub fn ashtakavarga_ayus(
        &self,
        bindus: &Bindus,
        rules: AshtakavargaAyusRules,
    ) -> AshtakavargaAyus {
        let chart = self.chart();
        let signs = GRAHAS.map(|graha| chart.placement(Body::Graha(graha)).sign);
        let mut occupied = [false; 12];
        for sign in signs {
            if let Some(cell) = occupied.get_mut(sign as usize % 12) {
                *cell = true;
            }
        }
        let grahas = std::array::from_fn(|at| {
            let graha = GRAHAS.get(at).copied().unwrap_or(Graha::Sun);
            let raw = bindus
                .grahas
                .get(at)
                .copied()
                .unwrap_or([0; 12])
                .map(u16::from);
            let reduced = reduce(raw, &occupied, bindus.ekadhipatya);
            let (rashi_pinda, graha_pinda) = pindas(&reduced, &signs, rules.measures);
            let basic = pinda_years(rashi_pinda + graha_pinda, rules.divisor);
            let at = chart.placement(Body::Graha(graha));
            let factor = match rules.reductions {
                AshtakaReductions::Verse => factor(graha, at.longitude, at.retrograde, at.combust),
                AshtakaReductions::Balabhadra => self.balabhadra_factor(graha),
            };
            let counted = match rules.occupied {
                OccupiedBindus::Reduced => reduced,
                OccupiedBindus::Raw => raw,
            };
            let occupied_bindus = signs
                .iter()
                .filter_map(|sign| counted.get(*sign as usize % 12))
                .map(|figure| u32::from(*figure))
                .sum();
            AshtakaGiver {
                graha,
                rashi_pinda,
                graha_pinda,
                basic,
                factor,
                years: basic * factor,
                occupied_bindus,
                occupied_years: f64::from(occupied_bindus) * factor,
            }
        });
        let mut sarva = [0_u16; 12];
        for row in &bindus.grahas {
            for (cell, b) in sarva.iter_mut().zip(row) {
                *cell += u16::from(*b);
            }
        }
        let net = reduce(sarva, &occupied, bindus.ekadhipatya).map(|figure| match figure % 12 {
            0 if figure > 0 => 12,
            rest => rest,
        });
        let (rashi, graha) = pindas(&net, &signs, rules.measures);
        let product = f64::from(rashi + graha) * 7.0 / 27.0;
        let samudaya = if product > 100.0 {
            product - 100.0
        } else {
            product
        };
        let bhinna = grahas.iter().map(|giver| giver.years).sum::<f64>();
        // Seven over 27 counts in nakshatra years, as v. 71 and the note
        // convert them; over 30 in the years of 360 days v. 34 converts.
        let year = match rules.divisor {
            Divisor::Thirty => SAVANA_YEAR,
            Divisor::SevenOverTwentySeven => NAKSHATRA_YEAR,
        };
        let lagna_sign = chart.placement(Body::Lagna).sign;
        let lagna_raw = lagna_bindus(&signs, lagna_sign);
        let (rashi_pinda, graha_pinda) = pindas(
            &reduce(lagna_raw.map(u16::from), &occupied, bindus.ekadhipatya),
            &signs,
            rules.measures,
        );
        let lagna = LagnaAshtaka {
            bindus: lagna_raw,
            rashi_pinda,
            graha_pinda,
            years: pinda_years(rashi_pinda + graha_pinda, rules.divisor),
        };
        AshtakavargaAyus {
            grahas,
            bhinna,
            occupied: grahas.iter().map(|giver| giver.occupied_years).sum(),
            lagna,
            called_for: self.called_for(),
            bhinna_solar: bhinna * year / SOLAR_YEAR,
            samudaya,
            samudaya_product: product,
            samudaya_solar: samudaya * NAKSHATRA_YEAR / SOLAR_YEAR,
            rules,
        }
    }
}

impl Evaluator<'_> {
    /// v. 49's two clauses: the Moon joined outside the kendras, or the
    /// tenth held by a benefic and a malefic. "Another planet" is any of
    /// the nine, as in Balabhadra's reductions.
    fn called_for(&self) -> CalledFor {
        let chart = self.chart();
        let lagna = chart.placement(Body::Lagna).sign;
        let sign_of = |graha: Graha| chart.placement(Body::Graha(graha)).sign;
        let moon = sign_of(Graha::Moon);
        let kendra = matches!(House::between(lagna, moon).get(), 1 | 4 | 7 | 10);
        let joined = NINE
            .iter()
            .any(|other| *other != Graha::Moon && sign_of(*other) == moon);
        let in_tenth: Vec<Graha> = NINE
            .iter()
            .copied()
            .filter(|graha| House::between(lagna, sign_of(*graha)).get() == 10)
            .collect();
        let has = |class: Class| in_tenth.iter().any(|graha| self.of_class(*graha, class));
        CalledFor {
            moon_joined_outside_a_kendra: !kendra && joined,
            tenth_benefic_and_malefic: has(Class::Benefic) && has(Class::Malefic),
        }
    }

    /// What of a graha's years Balabhadra's reductions leave: one less the
    /// greatest share that applies (the note to v. 46). Its bhava is whole
    /// signs from the lagna, and "another planet" any of the nine.
    fn balabhadra_factor(&self, graha: Graha) -> f64 {
        let chart = self.chart();
        let at = chart.placement(Body::Graha(graha));
        let lagna = chart.placement(Body::Lagna);
        let sign_of = |other: Graha| chart.placement(Body::Graha(other)).sign;
        let joined = NINE
            .iter()
            .any(|other| *other != graha && sign_of(*other) == at.sign);
        let debilitated = graha
            .attributes()
            .debilitation
            .is_some_and(|point| point.sign == at.sign);
        let enemy = in_enemy_sign(Enmity::Natural, graha, at.sign, |other| {
            Some(sign_of(other))
        });
        let visible = House::between(lagna.sign, at.sign).get() >= 7;
        let eclipsed = matches!(graha, Graha::Sun | Graha::Moon)
            && [Graha::Rahu, Graha::Ketu]
                .iter()
                .any(|node| sign_of(*node) == at.sign);
        let greatest = [
            (joined || debilitated || at.combust, 0.5),
            (enemy || visible || eclipsed, 1.0 / 3.0),
        ]
        .iter()
        .filter(|(applies, _)| *applies)
        .map(|(_, share)| *share)
        .fold(0.0, f64::max);
        1.0 - greatest
    }
}

/// v. 46's factor for a graha at `longitude`: retrograde Mars doubled, an
/// eclipsed graha halved, and otherwise half at debilitation and twice at
/// exaltation, in proportion between (C313).
fn factor(graha: Graha, longitude: f64, retrograde: bool, combust: bool) -> f64 {
    if graha == Graha::Mars && retrograde {
        return 2.0;
    }
    if combust {
        return 0.5;
    }
    // `by_exaltation` of 1 runs from ½ to 1 over the same arc.
    by_exaltation(graha, longitude, 1.0).map_or(1.0, |share| 0.5 + 3.0 * (share - 0.5))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_divisors_cast_out_their_cycles_and_parijata_measures_virgo_as_5() {
        assert!((pinda_years(252, Divisor::Thirty) - 8.4).abs() < 1e-12);
        // 390 / 30 = 13 years, less a twelve.
        assert!((pinda_years(390, Divisor::Thirty) - 1.0).abs() < 1e-12);
        // 252 × 7 / 27 = 65⅓, less two twenty-sevens.
        assert!(
            (pinda_years(252, Divisor::SevenOverTwentySeven) - (196.0 / 3.0 - 54.0)).abs() < 1e-9
        );
        assert_eq!(SignMeasures::Parijata.table()[5], 5);
        assert_eq!(SignMeasures::Bphs.table()[5], 6);
    }

    #[test]
    fn v46_doubles_at_exaltation_halves_at_debilitation_and_runs_evenly_between() {
        // The Sun exalted at Aries 10°, debilitated at Libra 10°, and a
        // quarter of the way round from either at 1¼ in between.
        let close = |a: f64, b: f64| (a - b).abs() < 1e-12;
        assert!(close(factor(Graha::Sun, 10.0, false, false), 2.0));
        assert!(close(factor(Graha::Sun, 190.0, false, false), 0.5));
        assert!(close(factor(Graha::Sun, 100.0, false, false), 1.25));
        assert!(close(factor(Graha::Mars, 118.0, true, false), 2.0));
        assert!(close(factor(Graha::Venus, 357.0, false, true), 0.5));
    }
}
