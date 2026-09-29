//! The Ashtakavarga: each graha's bindus by sign, their reductions and
//! pindas (`03-design/ashtakavarga-measured.md`).
//!
//! The bindu tables are BPHS ch. 66's, which the conformance corpus's
//! recording engine shares, and every chart holds the classical 337. What is
//! done after them is read two ways, and each is a setting:
//!
//! - [`Shodhana::EachGraha`], BPHS chs. 67 to 69: the trine and Ekadhipatya
//!   reductions in each graha's own Ashtakavarga, and each graha's pindas
//!   from its own reduced bindus — the rashi pinda by the signs' measures,
//!   the graha pinda by the measures of the grahas standing in each sign;
//! - [`Shodhana::Sarva`], the recording engine's: the reductions on the sum
//!   of the seven, a rashi pinda of the reduced sum by sign, and a graha
//!   pinda of all of a graha's raw bindus times its own measure.
//!
//! The Ekadhipatya rule is a setting too ([`Ekadhipatya`], crux C60). The
//! occupants that decide it, and that a graha pinda counts, are the seven
//! grahas.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::settings::{Ekadhipatya, MoonBinduFromJupiter, Settings, Shodhana};

/// The signs.
const SIGNS: usize = 12;
/// The grahas an Ashtakavarga is kept for, and that contribute, Sun to Saturn.
pub const GRAHAS: [Graha; 7] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];

/// Places 1 to 12 as a bit set, bit `n - 1` for the `n`th.
const fn set(mut places: &[u8]) -> u16 {
    let mut bits = 0;
    while let [place, rest @ ..] = places {
        bits |= 1 << (*place - 1);
        places = rest;
    }
    bits
}

/// The Moon's row and Jupiter's column of [`PLACES`]: the one cell the
/// tables part on (crux C144).
const MOON_FROM_JUPITER: (usize, usize) = (1, 4);

/// Where Jupiter gives the Moon a bindu under each reading.
const fn moon_from_jupiter(reading: MoonBinduFromJupiter) -> u16 {
    match reading {
        MoonBinduFromJupiter::Second => set(&[1, 2, 4, 7, 8, 10, 11]),
        // The default, BPHS's; the knob is closed, and a later reading is
        // its own arm.
        _ => set(&[1, 4, 7, 8, 10, 11, 12]),
    }
}

/// Where each graha gains a bindu, counted from each contributor — the seven
/// grahas in order, then the lagna — as bit sets of the places 1 to 12
/// (bit `n - 1`), from BPHS ch. 66; the Moon's from Jupiter is
/// [`moon_from_jupiter`]'s.
const PLACES: [[u16; 8]; 7] = {
    [
        [
            set(&[1, 2, 4, 7, 8, 9, 10, 11]),
            set(&[3, 6, 10, 11]),
            set(&[1, 2, 4, 7, 8, 9, 10, 11]),
            set(&[3, 5, 6, 9, 10, 11, 12]),
            set(&[5, 6, 9, 11]),
            set(&[6, 7, 12]),
            set(&[1, 2, 4, 7, 8, 9, 10, 11]),
            set(&[3, 4, 6, 10, 11, 12]),
        ],
        [
            set(&[3, 6, 7, 8, 10, 11]),
            set(&[1, 3, 6, 7, 10, 11]),
            set(&[2, 3, 5, 6, 9, 10, 11]),
            set(&[1, 3, 4, 5, 7, 8, 10, 11]),
            moon_from_jupiter(MoonBinduFromJupiter::Twelfth),
            set(&[3, 4, 5, 7, 9, 10, 11]),
            set(&[3, 5, 6, 11]),
            set(&[3, 6, 10, 11]),
        ],
        [
            set(&[3, 5, 6, 10, 11]),
            set(&[3, 6, 11]),
            set(&[1, 2, 4, 7, 8, 10, 11]),
            set(&[3, 5, 6, 11]),
            set(&[6, 10, 11, 12]),
            set(&[6, 8, 11, 12]),
            set(&[1, 4, 7, 8, 9, 10, 11]),
            set(&[1, 3, 6, 10, 11]),
        ],
        [
            set(&[5, 6, 9, 11, 12]),
            set(&[2, 4, 6, 8, 10, 11]),
            set(&[1, 2, 4, 7, 8, 9, 10, 11]),
            set(&[1, 3, 5, 6, 9, 10, 11, 12]),
            set(&[6, 8, 11, 12]),
            set(&[1, 2, 3, 4, 5, 8, 9, 11]),
            set(&[1, 2, 4, 7, 8, 9, 10, 11]),
            set(&[1, 2, 4, 6, 8, 10, 11]),
        ],
        [
            set(&[1, 2, 3, 4, 7, 8, 9, 10, 11]),
            set(&[2, 5, 7, 9, 11]),
            set(&[1, 2, 4, 7, 8, 10, 11]),
            set(&[1, 2, 4, 5, 6, 9, 10, 11]),
            set(&[1, 2, 3, 4, 7, 8, 10, 11]),
            set(&[2, 5, 6, 9, 10, 11]),
            set(&[3, 5, 6, 12]),
            set(&[1, 2, 4, 5, 6, 7, 9, 10, 11]),
        ],
        [
            set(&[8, 11, 12]),
            set(&[1, 2, 3, 4, 5, 8, 9, 11, 12]),
            set(&[3, 5, 6, 9, 11, 12]),
            set(&[3, 5, 6, 9, 11]),
            set(&[5, 8, 9, 10, 11]),
            set(&[1, 2, 3, 4, 5, 8, 9, 10, 11]),
            set(&[3, 4, 5, 8, 9, 10, 11]),
            set(&[1, 2, 3, 4, 5, 8, 9, 11]),
        ],
        [
            set(&[1, 2, 4, 7, 8, 10, 11]),
            set(&[3, 6, 11]),
            set(&[3, 5, 6, 10, 11, 12]),
            set(&[6, 8, 9, 10, 11, 12]),
            set(&[5, 6, 11, 12]),
            set(&[6, 11, 12]),
            set(&[3, 5, 6, 11]),
            set(&[1, 3, 4, 6, 10, 11]),
        ],
    ]
};

/// The sign pairs one graha rules (Mars, Venus, Mercury, Jupiter, Saturn).
const PAIRS: [(usize, usize); 5] = [(0, 7), (1, 6), (2, 5), (8, 11), (9, 10)];

/// Each sign's measure under BPHS ch. 69, Aries to Pisces.
pub const RASHI_MEASURES: [u16; SIGNS] = [7, 10, 8, 4, 10, 6, 7, 8, 9, 5, 11, 12];
/// Each sign's measure as the recording engine takes it, Virgo's 8.
pub const RECORDED_RASHI_MEASURES: [u16; SIGNS] = [7, 10, 8, 4, 10, 8, 7, 8, 9, 5, 11, 12];
/// Each graha's measure, Sun to Saturn (BPHS ch. 69).
pub const GRAHA_MEASURES: [u16; 7] = [5, 5, 8, 5, 10, 7, 5];

/// What an Ashtakavarga reads of a chart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AshtakavargaChart {
    /// The lagna's sign.
    pub lagna: Rashi,
    /// The seven grahas' signs, Sun to Saturn.
    pub signs: [Rashi; 7],
}

/// The choices an Ashtakavarga is reduced under.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AshtakavargaRules {
    /// Where the reductions and pindas are made.
    pub shodhana: Shodhana,
    /// How a co-ruled sign beside an occupied one is reduced.
    pub ekadhipatya: Ekadhipatya,
    /// Where Jupiter gives the Moon a bindu, which the bindus themselves
    /// are counted under. A document written before this was a setting
    /// has none, and was counted under the 12th.
    #[serde(default = "counted_before_the_knob")]
    pub moon_bindu_from_jupiter: MoonBinduFromJupiter,
}

/// The one table there was before the Moon's bindu from Jupiter was a
/// setting, BPHS's, for reading a document written then.
const fn counted_before_the_knob() -> MoonBinduFromJupiter {
    MoonBinduFromJupiter::Twelfth
}

impl AshtakavargaRules {
    /// The rules the settings choose, from their `strength` group.
    #[must_use]
    pub const fn of(settings: &Settings) -> AshtakavargaRules {
        AshtakavargaRules {
            shodhana: settings.strength.shodhana,
            ekadhipatya: settings.strength.ekadhipatya,
            moon_bindu_from_jupiter: settings.strength.moon_bindu_from_jupiter,
        }
    }
}

/// One graha's Ashtakavarga.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GrahaAshtakavarga {
    /// Which graha.
    pub graha: Graha,
    /// Its bindus by sign, Aries to Pisces, 0 to 8.
    pub bindus: [u8; SIGNS],
    /// The same after both reductions, when they are made in each graha's own
    /// Ashtakavarga.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub reduced: Option<[u8; SIGNS]>,
    /// Its rashi pinda.
    pub rashi_pinda: u32,
    /// Its graha pinda.
    pub graha_pinda: u32,
    /// Its yoga pinda, the two together.
    pub yoga_pinda: u32,
}

/// A chart's Ashtakavarga.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AshtakavargaReading {
    /// The choices it was reduced under.
    pub rules: AshtakavargaRules,
    /// Each graha's, Sun to Saturn.
    pub grahas: Vec<GrahaAshtakavarga>,
    /// The seven grahas' bindus by sign, the sarvashtakavarga, 337 in all.
    pub sarva: [u16; SIGNS],
    /// The sum after the trine reduction.
    pub trikona: [u16; SIGNS],
    /// The sum after both reductions.
    pub reduced: [u16; SIGNS],
}

fn sign_index(sign: Rashi) -> usize {
    sign as usize % SIGNS
}

/// The contributors' bit for the lagna in a [`prastara`] cell: the seven
/// grahas take bits 0 to 6 by their ids.
pub const LAGNA_BIT: u8 = 1 << 7;

/// The prastara: for each graha, Sun to Saturn, and each sign, Aries to
/// Pisces, **which** contributors gave it a bindu — bit `n` the graha with
/// id `n`, and [`LAGNA_BIT`] the lagna (Phaladeepika ch. 23 v. 17's table of
/// 96 squares a graha). What times a bindu's fruit in transit reads, since
/// each part of a sign belongs to one contributor (vv. 16 to 19).
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_core::settings::MoonBinduFromJupiter;
/// use teistro_strength::ashtakavarga::{AshtakavargaChart, bindus, prastara};
///
/// let chart = AshtakavargaChart { lagna: Rashi::Aries, signs: [Rashi::Leo; 7] };
/// let reading = MoonBinduFromJupiter::Twelfth;
/// // Each cell's bindus are its contributors, counted.
/// let counted = prastara(&chart, reading).map(|row| row.map(|cell| cell.count_ones() as u8));
/// assert_eq!(counted, bindus(&chart, reading));
/// ```
#[must_use]
pub fn prastara(chart: &AshtakavargaChart, reading: MoonBinduFromJupiter) -> [[u8; SIGNS]; 7] {
    let mut table = PLACES;
    if let Some(cell) = table
        .get_mut(MOON_FROM_JUPITER.0)
        .and_then(|row| row.get_mut(MOON_FROM_JUPITER.1))
    {
        *cell = moon_from_jupiter(reading);
    }
    let contributors: [Rashi; 8] = [
        chart.signs[0],
        chart.signs[1],
        chart.signs[2],
        chart.signs[3],
        chart.signs[4],
        chart.signs[5],
        chart.signs[6],
        chart.lagna,
    ];
    let mut out = [[0; SIGNS]; 7];
    for (row, places) in out.iter_mut().zip(table) {
        for (contributor, (from, bits)) in contributors.iter().zip(places).enumerate() {
            for (offset, slot) in (0..SIGNS).map(|k| (k, (sign_index(*from) + k) % SIGNS)) {
                if bits & (1 << offset) != 0 {
                    if let Some(cell) = row.get_mut(slot) {
                        *cell |= 1 << contributor;
                    }
                }
            }
        }
    }
    out
}

/// Each graha's bindus by sign: its [`prastara`]'s contributors, counted.
#[must_use]
pub fn bindus(chart: &AshtakavargaChart, reading: MoonBinduFromJupiter) -> [[u8; SIGNS]; 7] {
    #[allow(
        clippy::cast_possible_truncation,
        reason = "eight contributors at most, so a count of set bits fits a byte"
    )]
    prastara(chart, reading).map(|row| row.map(|cell| cell.count_ones() as u8))
}

/// The trine reduction: the least of each trine taken from all three.
fn trikona<T: Copy + Ord + core::ops::Sub<Output = T>>(values: [T; SIGNS]) -> [T; SIGNS] {
    let mut out = values;
    for start in 0..4 {
        let trine = [start, start + 4, start + 8];
        let Some(least) = trine.iter().filter_map(|i| values.get(*i)).min().copied() else {
            continue;
        };
        for i in trine {
            if let Some(cell) = out.get_mut(i) {
                *cell = *cell - least;
            }
        }
    }
    out
}

/// The Ekadhipatya reduction of a trine-reduced row.
fn ekadhipatya<T: Copy + Ord + Default + core::ops::Sub<Output = T>>(
    values: [T; SIGNS],
    occupied: &[bool; SIGNS],
    rule: Ekadhipatya,
) -> [T; SIGNS] {
    let zero = T::default();
    let mut out = values;
    for (a, b) in PAIRS {
        let (Some(&va), Some(&vb)) = (values.get(a), values.get(b)) else {
            continue;
        };
        if va == zero || vb == zero {
            continue;
        }
        let (occupied_a, occupied_b) = (
            occupied.get(a) == Some(&true),
            occupied.get(b) == Some(&true),
        );
        let (na, nb) = match (occupied_a, occupied_b) {
            (true, true) => (va, vb),
            (false, false) if va == vb => (zero, zero),
            (false, false) => (va.min(vb), va.min(vb)),
            _ => {
                let (full, empty) = if occupied_a { (va, vb) } else { (vb, va) };
                let rest = if rule == Ekadhipatya::Bphs && full < empty {
                    empty - full
                } else {
                    zero
                };
                if occupied_a { (va, rest) } else { (rest, vb) }
            }
        };
        if let Some(cell) = out.get_mut(a) {
            *cell = na;
        }
        if let Some(cell) = out.get_mut(b) {
            *cell = nb;
        }
    }
    out
}

fn dot<T: Copy + Into<u32>>(values: &[T; SIGNS], measures: &[u16; SIGNS]) -> u32 {
    values
        .iter()
        .zip(measures)
        .map(|(value, measure)| (*value).into() * u32::from(*measure))
        .sum()
}

impl AshtakavargaReading {
    /// A chart's Ashtakavarga, reduced under `rules`.
    #[must_use]
    pub fn of(chart: &AshtakavargaChart, rules: AshtakavargaRules) -> AshtakavargaReading {
        let bindus = bindus(chart, rules.moon_bindu_from_jupiter);
        let mut occupied = [false; SIGNS];
        for sign in chart.signs {
            if let Some(cell) = occupied.get_mut(sign_index(sign)) {
                *cell = true;
            }
        }
        let widen = |row: &[u8; SIGNS]| row.map(u16::from);
        let mut sarva = [0_u16; SIGNS];
        for row in &bindus {
            for (slot, value) in sarva.iter_mut().zip(widen(row)) {
                *slot += value;
            }
        }
        let (grahas, trikona_sum, reduced_sum) = if rules.shodhana == Shodhana::Sarva {
            let trine = trikona(sarva);
            let reduced = ekadhipatya(trine, &occupied, rules.ekadhipatya);
            let grahas = bindus
                .iter()
                .zip(GRAHAS)
                .zip(chart.signs)
                .zip(GRAHA_MEASURES)
                .map(|(((row, graha), sign), measure)| {
                    let rashi_pinda = reduced
                        .get(sign_index(sign))
                        .zip(RECORDED_RASHI_MEASURES.get(sign_index(sign)))
                        .map_or(0, |(value, m)| u32::from(*value) * u32::from(*m));
                    let raw: u32 = row.iter().map(|b| u32::from(*b)).sum();
                    let graha_pinda = raw * u32::from(measure);
                    GrahaAshtakavarga {
                        graha,
                        bindus: *row,
                        reduced: None,
                        rashi_pinda,
                        graha_pinda,
                        yoga_pinda: rashi_pinda + graha_pinda,
                    }
                })
                .collect();
            (grahas, trine, reduced)
        } else {
            let mut trine_sum = [0_u16; SIGNS];
            let mut reduced_sum = [0_u16; SIGNS];
            let grahas = bindus
                .iter()
                .zip(GRAHAS)
                .map(|(row, graha)| {
                    let trine = trikona(*row);
                    let reduced = ekadhipatya(trine, &occupied, rules.ekadhipatya);
                    for ((t, r), (tv, rv)) in trine_sum
                        .iter_mut()
                        .zip(reduced_sum.iter_mut())
                        .zip(widen(&trine).into_iter().zip(widen(&reduced)))
                    {
                        *t += tv;
                        *r += rv;
                    }
                    let rashi_pinda = dot(&reduced, &RASHI_MEASURES);
                    let graha_pinda = chart
                        .signs
                        .iter()
                        .zip(GRAHA_MEASURES)
                        .map(|(sign, measure)| {
                            reduced
                                .get(sign_index(*sign))
                                .map_or(0, |value| u32::from(*value) * u32::from(measure))
                        })
                        .sum();
                    GrahaAshtakavarga {
                        graha,
                        bindus: *row,
                        reduced: Some(reduced),
                        rashi_pinda,
                        graha_pinda,
                        yoga_pinda: rashi_pinda + graha_pinda,
                    }
                })
                .collect();
            (grahas, trine_sum, reduced_sum)
        };
        AshtakavargaReading {
            rules,
            grahas,
            sarva,
            trikona: trikona_sum,
            reduced: reduced_sum,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::indexing_slicing,
        clippy::unwrap_used,
        reason = "tests index what they built and fail by panicking"
    )]

    use super::*;

    fn chart() -> AshtakavargaChart {
        // The corpus's first chart: Pisces rising, the Sun and Mercury in
        // Aries, the Moon in Scorpio, Mars and Venus in Aquarius, Jupiter in
        // Gemini, Saturn in Capricorn.
        AshtakavargaChart {
            lagna: Rashi::Pisces,
            signs: [
                Rashi::Aries,
                Rashi::Scorpio,
                Rashi::Aquarius,
                Rashi::Aries,
                Rashi::Gemini,
                Rashi::Aquarius,
                Rashi::Capricorn,
            ],
        }
    }

    #[test]
    fn every_chart_holds_the_classical_totals_under_either_reading() {
        for reading in MoonBinduFromJupiter::ALL.iter().copied() {
            for lagna in Rashi::ALL {
                let rows = bindus(&AshtakavargaChart { lagna, ..chart() }, reading);
                let totals: Vec<u32> = rows
                    .iter()
                    .map(|r| r.iter().map(|b| u32::from(*b)).sum())
                    .collect();
                assert_eq!(totals, [48, 49, 39, 54, 56, 52, 39], "{reading:?}");
            }
        }
    }

    /// A document written before the knob carries rules without it, and
    /// was counted under the 12th, so that is how it reads back.
    #[test]
    fn rules_written_before_the_knob_read_as_the_twelfth() {
        let rules: AshtakavargaRules =
            serde_json::from_str(r#"{"shodhana": "EACH_GRAHA", "ekadhipatya": "BPHS"}"#).unwrap();
        assert_eq!(rules.moon_bindu_from_jupiter, MoonBinduFromJupiter::Twelfth);
        let written = serde_json::to_value(AshtakavargaRules {
            moon_bindu_from_jupiter: MoonBinduFromJupiter::Second,
            ..rules
        })
        .unwrap();
        assert_eq!(written["moon_bindu_from_jupiter"], "SECOND");
    }

    /// The readings part on one cell and nowhere else: Jupiter's bindu in
    /// the Moon's row moves from the 12th from Jupiter to the 2nd, for
    /// Jupiter in every sign.
    #[test]
    fn the_readings_move_only_jupiters_bindu_for_the_moon() {
        let jupiter_bit = 1_u8 << 4;
        for sign in Rashi::ALL {
            let mut signs = chart().signs;
            signs[4] = sign;
            let at = AshtakavargaChart {
                lagna: Rashi::Aries,
                signs,
            };
            let twelfth = prastara(&at, MoonBinduFromJupiter::Twelfth);
            let second = prastara(&at, MoonBinduFromJupiter::Second);
            for (row, (a, b)) in twelfth.iter().zip(&second).enumerate() {
                for (slot, (x, y)) in a.iter().zip(b).enumerate() {
                    let from_jupiter = (slot + SIGNS - sign_index(sign)) % SIGNS + 1;
                    let (x, y) = (x & jupiter_bit != 0, y & jupiter_bit != 0);
                    let expected = match (row, from_jupiter) {
                        (1, 12) => (true, false),
                        (1, 2) => (false, true),
                        _ => (x, x),
                    };
                    assert_eq!(
                        (x, y),
                        expected,
                        "row {row}, the {from_jupiter}th from Jupiter"
                    );
                    assert_eq!(
                        a[slot] & !jupiter_bit,
                        b[slot] & !jupiter_bit,
                        "another contributor moved"
                    );
                }
            }
        }
    }

    #[test]
    fn ekadhipatya_keeps_a_difference_only_under_the_text() {
        let mut values = [0_u8; SIGNS];
        values[0] = 5;
        values[7] = 7;
        let mut occupied = [false; SIGNS];
        occupied[0] = true;
        assert_eq!(ekadhipatya(values, &occupied, Ekadhipatya::Bphs)[7], 2);
        assert_eq!(
            ekadhipatya(values, &occupied, Ekadhipatya::EmptyToZero)[7],
            0
        );
        // The occupied sign with the bigger number: the empty one goes to zero
        // under both.
        values[0] = 9;
        assert_eq!(ekadhipatya(values, &occupied, Ekadhipatya::Bphs)[7], 0);
        // Both empty and equal: both zero; unequal: both the smaller.
        let none = [false; SIGNS];
        values[0] = 7;
        assert_eq!(ekadhipatya(values, &none, Ekadhipatya::Bphs)[0], 0);
        values[0] = 3;
        assert_eq!(ekadhipatya(values, &none, Ekadhipatya::Bphs)[7], 3);
        // A sign with no number after the trine reduction: nothing.
        values[0] = 0;
        assert_eq!(ekadhipatya(values, &none, Ekadhipatya::Bphs)[7], 7);
    }

    #[test]
    fn each_reading_sums_its_own_reductions_and_forms_its_pindas() {
        let each = AshtakavargaReading::of(
            &chart(),
            AshtakavargaRules {
                shodhana: Shodhana::EachGraha,
                ekadhipatya: Ekadhipatya::Bphs,
                moon_bindu_from_jupiter: MoonBinduFromJupiter::Twelfth,
            },
        );
        assert_eq!(each.sarva.iter().map(|v| u32::from(*v)).sum::<u32>(), 337);
        let reduced: Vec<[u8; SIGNS]> = each.grahas.iter().map(|g| g.reduced.unwrap()).collect();
        for (i, total) in each.reduced.iter().enumerate() {
            assert_eq!(
                u32::from(*total),
                reduced.iter().map(|r| u32::from(r[i])).sum::<u32>()
            );
        }
        for graha in &each.grahas {
            assert_eq!(graha.yoga_pinda, graha.rashi_pinda + graha.graha_pinda);
        }
        let sarva = AshtakavargaReading::of(
            &chart(),
            AshtakavargaRules {
                shodhana: Shodhana::Sarva,
                ekadhipatya: Ekadhipatya::EmptyToZero,
                moon_bindu_from_jupiter: MoonBinduFromJupiter::Twelfth,
            },
        );
        assert_eq!(
            sarva.sarva, each.sarva,
            "the bindus do not depend on the reading"
        );
        assert!(sarva.grahas.iter().all(|g| g.reduced.is_none()));
        // The Sun's graha pinda is its 48 bindus times its measure, 5.
        assert_eq!(sarva.grahas[0].graha_pinda, 240);
    }
}
