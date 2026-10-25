//! What a graha's placement says of its dasha: the benefic and malefic
//! points of its dignity in the seven vargas (BPHS ch. 28 vv. 7 to 10), when
//! in the dasha its effects come, and whether its placement makes the dasha
//! favourable (ch. 47 vv. 3 to 6).
//!
//! - **The points.** 60, 45, 30, 22, 15, 8, 4, 2 and 0 are the Subhankas of
//!   exaltation, moolatrikona, the own sign, a great friend's, a friend's, a
//!   neutral's, an enemy's and a great enemy's sign and debilitation; the
//!   Asubhanka is 60 less, and both are halved in the other vargas. The first
//!   five places are auspicious, the sixth neutral and the last three
//!   inauspicious (v. 10).
//! - **The timing.** A graha in its sign's first decanate gives its effects at
//!   the dasha's commencement, in the second in its middle and in the third at
//!   its end, the other way about when retrograde, and always so for Rahu and
//!   Ketu (vv. 3 and 4).
//! - **The placement.** In the lagna, exaltation, the own sign or a Shant sign
//!   the dasha is favourable; in the sixth, eighth or twelfth, debilitation or
//!   an inimical sign unfavourable (vv. 5 and 6). A placement can be both,
//!   and both flags then stand.
//!
//! The verses' "at the commencement of the Dasha" is read of the natal
//! placement, and which friendly signs are Shant is the `dasha.shanta_sign`
//! knob (crux C79).

use serde::{Deserialize, Serialize};
use teistro_core::angle::Nas;
use teistro_core::catalogue::{Dignity, Graha, Nature, Rashi};
use teistro_core::settings::ShantaSign;
use teistro_state::dignity::{is_shadow, varga_dignity};

use crate::bhava_bala::NINE;

/// Where in a dasha a graha's effects are felt (BPHS ch. 47 vv. 3 and 4).
///
/// ```
/// use teistro_strength::DashaPhase;
///
/// assert_eq!(DashaPhase::ALL[0], DashaPhase::Commencement);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DashaPhase {
    /// At its commencement.
    Commencement,
    /// In its middle.
    Middle,
    /// At its end.
    End,
}

impl DashaPhase {
    /// The phase's key, as a message selects on it.
    ///
    /// Spelled in the house style — lower case, the words the verses use —
    /// because a phase is not a catalogue member and a message matching on
    /// it needs a string to match, exactly as `LifeClass` does.
    ///
    /// ```
    /// use teistro_strength::DashaPhase;
    ///
    /// assert!(DashaPhase::ALL.iter().all(|phase| !phase.key().is_empty()));
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            DashaPhase::Commencement => "commencement",
            DashaPhase::Middle => "middle",
            DashaPhase::End => "end",
        }
    }

    /// Every phase, in the dasha's order.
    pub const ALL: [DashaPhase; 3] = [
        DashaPhase::Commencement,
        DashaPhase::Middle,
        DashaPhase::End,
    ];
}

/// One graha as the dasha phala reads it.
///
/// ```
/// use teistro_core::catalogue::{Dignity, Graha, Rashi};
/// use teistro_core::settings::ShantaSign;
/// use teistro_strength::{DashaPhalaChart, DashaPhalaGraha, DashaPhalaReading};
///
/// // Every graha exalted, at 5° of its sign, in the lagna, in all seven vargas.
/// let exaltation = |graha: Graha| graha.attributes().exaltation.map_or(Rashi::Aries, |at| at.sign);
/// let signs = [Graha::Sun, Graha::Moon, Graha::Mars, Graha::Mercury, Graha::Jupiter, Graha::Venus, Graha::Saturn, Graha::Rahu, Graha::Ketu].map(exaltation);
/// let chart = DashaPhalaChart {
///     grahas: signs.map(|sign| DashaPhalaGraha {
///         longitude: f64::from(sign as u8) * 30.0 + 5.0,
///         house: 1,
///         dignity: Dignity::Exalted,
///         retrograde: false,
///     }),
///     vargas: [signs; 7],
/// };
/// let reading = DashaPhalaReading::of(&chart, ShantaSign::Friendly);
/// assert_eq!(chart.grahas[0].longitude, 5.0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DashaPhalaGraha {
    /// Its sidereal longitude, degrees.
    pub longitude: f64,
    /// The bhava it occupies, 1 to 12.
    pub house: u8,
    /// Its dignity in the rasi chart, which reads its degrees.
    pub dignity: Dignity,
    /// Whether it is retrograde.
    pub retrograde: bool,
}

/// What the dasha phala reads of a chart.
///
/// ```
/// use teistro_core::catalogue::{Dignity, Graha, Rashi};
/// use teistro_core::settings::ShantaSign;
/// use teistro_strength::{DashaPhalaChart, DashaPhalaGraha, DashaPhalaReading};
///
/// // Every graha exalted, at 5° of its sign, in the lagna, in all seven vargas.
/// let exaltation = |graha: Graha| graha.attributes().exaltation.map_or(Rashi::Aries, |at| at.sign);
/// let signs = [Graha::Sun, Graha::Moon, Graha::Mars, Graha::Mercury, Graha::Jupiter, Graha::Venus, Graha::Saturn, Graha::Rahu, Graha::Ketu].map(exaltation);
/// let chart = DashaPhalaChart {
///     grahas: signs.map(|sign| DashaPhalaGraha {
///         longitude: f64::from(sign as u8) * 30.0 + 5.0,
///         house: 1,
///         dignity: Dignity::Exalted,
///         retrograde: false,
///     }),
///     vargas: [signs; 7],
/// };
/// let reading = DashaPhalaReading::of(&chart, ShantaSign::Friendly);
/// assert_eq!(reading.grahas.len(), 9);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DashaPhalaChart {
    /// The nine grahas, Sun to Ketu.
    pub grahas: [DashaPhalaGraha; 9],
    /// Their signs in the seven vargas of
    /// [`SAPTAVARGAJA_VARGAS`](crate::shadbala::SAPTAVARGAJA_VARGAS), the
    /// rasi chart first; each row the nine, Sun to Ketu.
    pub vargas: [[Rashi; 9]; 7],
}

/// One graha's dasha phala.
///
/// ```
/// use teistro_core::catalogue::{Dignity, Graha, Rashi};
/// use teistro_core::settings::ShantaSign;
/// use teistro_strength::{DashaPhalaChart, DashaPhalaGraha, DashaPhalaReading};
///
/// // Every graha exalted, at 5° of its sign, in the lagna, in all seven vargas.
/// let exaltation = |graha: Graha| graha.attributes().exaltation.map_or(Rashi::Aries, |at| at.sign);
/// let signs = [Graha::Sun, Graha::Moon, Graha::Mars, Graha::Mercury, Graha::Jupiter, Graha::Venus, Graha::Saturn, Graha::Rahu, Graha::Ketu].map(exaltation);
/// let chart = DashaPhalaChart {
///     grahas: signs.map(|sign| DashaPhalaGraha {
///         longitude: f64::from(sign as u8) * 30.0 + 5.0,
///         house: 1,
///         dignity: Dignity::Exalted,
///         retrograde: false,
///     }),
///     vargas: [signs; 7],
/// };
/// let reading = DashaPhalaReading::of(&chart, ShantaSign::Friendly);
/// use teistro_core::catalogue::Nature;
/// use teistro_strength::DashaPhase;
///
/// let sun = &reading.grahas[0];
/// // Exalted in all seven: 60 in the rasi and 30 in each of the six others.
/// assert_eq!((sun.subhanka, sun.asubhanka), (240.0, 0.0));
/// assert_eq!(sun.nature, Nature::Benefic);
/// // The first decanate gives its effects at the dasha's commencement.
/// assert_eq!(sun.phase, DashaPhase::Commencement);
/// assert!(sun.favourable && !sun.unfavourable);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GrahaDashaPhala {
    /// Which graha.
    pub graha: Graha,
    /// Its Subhanka in each of the seven vargas, the rasi chart first: out of
    /// 60 there and 30 in the others.
    pub subhankas: [f64; 7],
    /// The seven together, out of 240.
    pub subhanka: f64,
    /// Their complements together, out of 240.
    pub asubhanka: f64,
    /// Whether its rasi placement is auspicious, neutral or inauspicious
    /// (v. 10).
    pub nature: Nature,
    /// Where in its dasha its effects are felt.
    pub phase: DashaPhase,
    /// Whether its placement makes its dasha favourable.
    pub favourable: bool,
    /// Whether its placement makes its dasha unfavourable.
    pub unfavourable: bool,
}

/// A chart's dasha phala.
///
/// ```
/// use teistro_core::catalogue::{Dignity, Graha, Rashi};
/// use teistro_core::settings::ShantaSign;
/// use teistro_strength::{DashaPhalaChart, DashaPhalaGraha, DashaPhalaReading};
///
/// // Every graha exalted, at 5° of its sign, in the lagna, in all seven vargas.
/// let exaltation = |graha: Graha| graha.attributes().exaltation.map_or(Rashi::Aries, |at| at.sign);
/// let signs = [Graha::Sun, Graha::Moon, Graha::Mars, Graha::Mercury, Graha::Jupiter, Graha::Venus, Graha::Saturn, Graha::Rahu, Graha::Ketu].map(exaltation);
/// let chart = DashaPhalaChart {
///     grahas: signs.map(|sign| DashaPhalaGraha {
///         longitude: f64::from(sign as u8) * 30.0 + 5.0,
///         house: 1,
///         dignity: Dignity::Exalted,
///         retrograde: false,
///     }),
///     vargas: [signs; 7],
/// };
/// let reading = DashaPhalaReading::of(&chart, ShantaSign::Friendly);
/// use teistro_strength::DashaPhase;
///
/// // Rahu and Ketu give theirs the other way about: at the end.
/// assert_eq!(reading.grahas[Graha::Rahu as usize].phase, DashaPhase::End);
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DashaPhalaReading {
    /// Each graha's, Sun to Ketu.
    pub grahas: Vec<GrahaDashaPhala>,
}

/// A dignity's Subhanka in the rasi chart (vv. 7 to 9).
const fn subhanka(dignity: Dignity) -> f64 {
    match dignity {
        Dignity::DeepExalted | Dignity::Exalted => 60.0,
        Dignity::Mooltrikona => 45.0,
        Dignity::OwnSign => 30.0,
        Dignity::GreatFriend => 22.0,
        Dignity::Friend => 15.0,
        Dignity::Enemy => 4.0,
        Dignity::GreatEnemy => 2.0,
        Dignity::Debilitated | Dignity::DeepDebilitated => 0.0,
        _ => 8.0,
    }
}

/// Whether a dignity's place is auspicious, neutral or inauspicious (v. 10).
const fn nature(dignity: Dignity) -> Nature {
    match dignity {
        Dignity::DeepExalted
        | Dignity::Exalted
        | Dignity::Mooltrikona
        | Dignity::OwnSign
        | Dignity::GreatFriend
        | Dignity::Friend => Nature::Benefic,
        Dignity::Enemy | Dignity::GreatEnemy | Dignity::Debilitated | Dignity::DeepDebilitated => {
            Nature::Malefic
        }
        _ => Nature::Neutral,
    }
}

/// Where in the dasha the effects of a graha at `longitude` come.
fn phase(graha: Graha, longitude: f64, retrograde: bool) -> DashaPhase {
    // The decanate: every sign holds three of the circle's thirty-six.
    let decanate = Nas::try_from_degrees(longitude.rem_euclid(360.0)).map_or(0, |at| {
        usize::try_from(at.division_index(36) % 3).unwrap_or(0)
    });
    let index = if retrograde || is_shadow(graha) {
        2 - decanate.min(2)
    } else {
        decanate.min(2)
    };
    DashaPhase::ALL
        .get(index)
        .copied()
        .unwrap_or(DashaPhase::Middle)
}

impl DashaPhalaReading {
    /// A chart's dasha phala.
    ///
    /// ```
    /// use teistro_core::catalogue::{Dignity, Graha, Rashi};
    /// use teistro_core::settings::ShantaSign;
    /// use teistro_strength::{DashaPhalaChart, DashaPhalaGraha, DashaPhalaReading};
    ///
    /// // Every graha exalted, at 5° of its sign, in the lagna, in all seven vargas.
    /// let exaltation = |graha: Graha| graha.attributes().exaltation.map_or(Rashi::Aries, |at| at.sign);
    /// let signs = [Graha::Sun, Graha::Moon, Graha::Mars, Graha::Mercury, Graha::Jupiter, Graha::Venus, Graha::Saturn, Graha::Rahu, Graha::Ketu].map(exaltation);
    /// let chart = DashaPhalaChart {
    ///     grahas: signs.map(|sign| DashaPhalaGraha {
    ///         longitude: f64::from(sign as u8) * 30.0 + 5.0,
    ///         house: 1,
    ///         dignity: Dignity::Exalted,
    ///         retrograde: false,
    ///     }),
    ///     vargas: [signs; 7],
    /// };
    /// let reading = DashaPhalaReading::of(&chart, ShantaSign::Friendly);
    /// assert!(reading.grahas.iter().take(7).all(|graha| graha.favourable));
    /// ```
    #[must_use]
    pub fn of(chart: &DashaPhalaChart, shanta: ShantaSign) -> DashaPhalaReading {
        let [rasi, ..] = chart.vargas;
        let rasi_sign_of = |graha: Graha| {
            NINE.iter()
                .position(|g| *g == graha)
                .and_then(|index| rasi.get(index).copied())
        };
        let grahas = NINE
            .iter()
            .zip(chart.grahas)
            .enumerate()
            .map(|(index, (graha, at))| {
                let mut subhankas = [0.0; 7];
                for (k, (slot, row)) in subhankas.iter_mut().zip(chart.vargas).enumerate() {
                    *slot = if k == 0 {
                        subhanka(at.dignity)
                    } else {
                        let sign = row.get(index).copied().unwrap_or(Rashi::Aries);
                        subhanka(varga_dignity(*graha, sign, rasi_sign_of)) / 2.0
                    };
                }
                let total: f64 = subhankas.iter().sum();
                let friendly = match at.dignity {
                    Dignity::Friend => true,
                    Dignity::GreatFriend => shanta != ShantaSign::Friend,
                    _ => false,
                };
                GrahaDashaPhala {
                    graha: *graha,
                    subhankas,
                    subhanka: total,
                    asubhanka: 240.0 - total,
                    nature: nature(at.dignity),
                    phase: phase(*graha, at.longitude, at.retrograde),
                    favourable: at.house == 1
                        || friendly
                        || matches!(
                            at.dignity,
                            Dignity::DeepExalted
                                | Dignity::Exalted
                                | Dignity::Mooltrikona
                                | Dignity::OwnSign
                        ),
                    unfavourable: matches!(at.house, 6 | 8 | 12)
                        || matches!(
                            at.dignity,
                            Dignity::Enemy
                                | Dignity::GreatEnemy
                                | Dignity::Debilitated
                                | Dignity::DeepDebilitated
                        ),
                }
            })
            .collect();
        DashaPhalaReading { grahas }
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::indexing_slicing,
        clippy::float_cmp,
        reason = "tests index what they built, and the points are exact halves"
    )]

    use super::*;

    const AT: DashaPhalaGraha = DashaPhalaGraha {
        longitude: 0.0,
        house: 2,
        dignity: Dignity::Neutral,
        retrograde: false,
    };

    fn chart(vargas: [[Rashi; 9]; 7]) -> DashaPhalaChart {
        DashaPhalaChart {
            grahas: [AT; 9],
            vargas,
        }
    }

    #[test]
    fn the_points_halve_outside_the_rasi_chart() {
        // The Sun exalted in the rasi chart and in Aries in every varga: 60,
        // then 30 six times, 240 in all and nothing inauspicious.
        let mut chart = chart([[Rashi::Aries; 9]; 7]);
        chart.grahas[0].dignity = Dignity::Exalted;
        let sun = DashaPhalaReading::of(&chart, ShantaSign::Friendly).grahas[0];
        assert_eq!(sun.subhankas, [60.0, 30.0, 30.0, 30.0, 30.0, 30.0, 30.0]);
        assert!((sun.subhanka - 240.0).abs() < 1e-12);
        assert!(sun.asubhanka.abs() < 1e-12);
        assert_eq!(sun.nature, Nature::Benefic);
        assert!(sun.favourable && !sun.unfavourable);
    }

    #[test]
    fn a_varga_sign_takes_the_compound_friendship_from_the_rasi_seats() {
        // Every graha in Aries in the rasi chart; the Moon in Leo elsewhere.
        // The Sun is her natural friend, and seated with her in the rasi
        // chart he is a temporary enemy: neutral, 8 halved to 4.
        let mut vargas = [[Rashi::Aries; 9]; 7];
        for row in vargas.iter_mut().skip(1) {
            row[1] = Rashi::Leo;
        }
        let moon = DashaPhalaReading::of(&chart(vargas), ShantaSign::Friendly).grahas[1];
        assert_eq!(moon.subhankas[1..], [4.0; 6]);
        // Each graha's own rasi seat: Mars in Taurus has the Sun in Aries in
        // his twelfth, a temporary friend, and his natural friend too, so in
        // the Sun's Leo he is a great friend's guest, 22 halved to 11.
        let mut vargas = [[Rashi::Aries; 9]; 7];
        vargas[0][1] = Rashi::Libra;
        vargas[0][2] = Rashi::Taurus;
        for row in vargas.iter_mut().skip(1) {
            row[2] = Rashi::Leo;
        }
        let mars = DashaPhalaReading::of(&chart(vargas), ShantaSign::Friendly).grahas[2];
        assert_eq!(mars.subhankas[1..], [11.0; 6]);
    }

    #[test]
    fn a_phase_s_key_is_its_word() {
        let keys = DashaPhase::ALL.map(DashaPhase::key);
        assert_eq!(keys, ["commencement", "middle", "end"]);
    }

    #[test]
    fn the_decanate_times_the_effects_and_retrogression_reverses_them() {
        let mut chart = chart([[Rashi::Aries; 9]; 7]);
        chart.grahas[2].longitude = 45.0; // Mars at 15° Taurus: the middle.
        chart.grahas[3].longitude = 25.0; // Mercury at 25°: the end,
        chart.grahas[4].longitude = 25.0; // Jupiter too, but retrograde.
        chart.grahas[4].retrograde = true;
        chart.grahas[7].longitude = 5.0; // Rahu at 5°, reversed: the end.
        let reading = DashaPhalaReading::of(&chart, ShantaSign::Friendly);
        assert_eq!(reading.grahas[2].phase, DashaPhase::Middle);
        assert_eq!(reading.grahas[3].phase, DashaPhase::End);
        assert_eq!(reading.grahas[4].phase, DashaPhase::Commencement);
        assert_eq!(reading.grahas[7].phase, DashaPhase::End);
        assert_eq!(reading.grahas[0].phase, DashaPhase::Commencement);
    }

    #[test]
    fn a_placement_can_be_favourable_unfavourable_both_or_neither() {
        let mut chart = chart([[Rashi::Aries; 9]; 7]);
        chart.grahas[0].house = 1; // In the lagna.
        chart.grahas[1].house = 8; // In the eighth.
        chart.grahas[2].house = 8; // Exalted in the eighth.
        chart.grahas[2].dignity = Dignity::Exalted;
        chart.grahas[3].dignity = Dignity::GreatFriend;
        chart.grahas[5].dignity = Dignity::Friend;
        let reading = DashaPhalaReading::of(&chart, ShantaSign::Friendly);
        let flags = |i: usize| (reading.grahas[i].favourable, reading.grahas[i].unfavourable);
        assert_eq!(flags(0), (true, false));
        assert_eq!(flags(1), (false, true));
        assert_eq!(flags(2), (true, true));
        assert_eq!(flags(3), (true, false));
        assert_eq!(flags(4), (false, false));
        assert_eq!(flags(5), (true, false));
        // A friend's sign alone is Shant under the literal reading.
        let literal = DashaPhalaReading::of(&chart, ShantaSign::Friend);
        assert!(!literal.grahas[3].favourable);
        assert!(literal.grahas[5].favourable);
    }

    #[test]
    fn each_place_s_points_and_nature_follow_the_ladder() {
        let ladder = [
            (Dignity::Exalted, 60.0, Nature::Benefic),
            (Dignity::Mooltrikona, 45.0, Nature::Benefic),
            (Dignity::OwnSign, 30.0, Nature::Benefic),
            (Dignity::GreatFriend, 22.0, Nature::Benefic),
            (Dignity::Friend, 15.0, Nature::Benefic),
            (Dignity::Neutral, 8.0, Nature::Neutral),
            (Dignity::Enemy, 4.0, Nature::Malefic),
            (Dignity::GreatEnemy, 2.0, Nature::Malefic),
            (Dignity::Debilitated, 0.0, Nature::Malefic),
        ];
        for (dignity, points, place) in ladder {
            assert!((subhanka(dignity) - points).abs() < 1e-12, "{dignity:?}");
            assert_eq!(nature(dignity), place, "{dignity:?}");
        }
    }
}
