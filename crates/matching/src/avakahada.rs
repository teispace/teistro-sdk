//! The avakahada: the janma-patrika's birth summary read off the Moon
//! (`03-design/matching.md`, C301).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Gana, Graha, Nadi, Nakshatra, Rashi, Varna, Yoni};
use teistro_core::error::Error;

use crate::{BirthSyllable, Native, birth_syllable, sign_varna};

/// What a janma-patrika prints of the Moon: its star and pada, the
/// syllable the child is named by, and the readings a koota takes of one
/// native (C301).
///
/// A gathering, not doctrine of its own: each reading is the one the Ashta
/// Koota reads of the same Moon, so it carries no rules. Vashya, paya,
/// disha and tatwa are not here (C300, C301).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Avakahada {
    /// The Moon's nakshatra.
    pub nakshatra: Nakshatra,
    /// Its pada, 1 to 4.
    pub pada: u8,
    /// The Moon's sign.
    pub rashi: Rashi,
    /// The nakshatra's lord, the Vimshottari dasha's.
    pub nakshatra_lord: Graha,
    /// The sign's lord, the one Graha Maitri reads.
    pub rashi_lord: Graha,
    /// The sign's varna, as Varna koota reads it (VI.22).
    pub varna: Varna,
    /// The nakshatra's yoni.
    pub yoni: Yoni,
    /// The nakshatra's gana.
    pub gana: Gana,
    /// The nakshatra's nadi.
    pub nadi: Nadi,
    /// The syllable the child is named by, with its varga (C297 to C299).
    pub syllable: BirthSyllable,
}

/// The avakahada of a Moon (C301).
///
/// ```
/// use teistro_core::catalogue::{Gana, Nakshatra};
/// use teistro_matching::{Native, avakahada};
///
/// // A Moon at 0°: Ashvini's first pada, named by चू.
/// let read = avakahada(Native::of_moon(0.0)?)?;
/// assert_eq!((read.nakshatra, read.pada, read.gana), (Nakshatra::Ashwini, 1, Gana::Deva));
/// assert_eq!(read.syllable.devanagari, "चू");
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// A native whose `pada` is outside 1 to 4, named `pada`.
pub fn avakahada(moon: Native) -> Result<Avakahada, Error> {
    let star = moon.nakshatra.attributes();
    Ok(Avakahada {
        nakshatra: moon.nakshatra,
        pada: moon.pada,
        rashi: moon.rashi,
        nakshatra_lord: star.vimshottari_lord,
        rashi_lord: moon.rashi.attributes().lord,
        varna: sign_varna(moon.rashi),
        yoni: star.yoni,
        gana: star.gana,
        nadi: star.nadi,
        syllable: birth_syllable(moon.nakshatra, moon.pada)?,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_core::catalogue::Nakshatra;

    use crate::{KootaReading, KootaRules, Native, ashta_koota, avakahada};

    #[test]
    fn every_pada_reads_as_the_ashta_koota_reads_its_moon() {
        // C301: a gathering, so each reading is the koota's own of the same
        // native, matched with itself.
        for nakshatra in Nakshatra::ALL {
            for pada in 1..=4 {
                let moon = Native::of_pada(nakshatra, pada).unwrap();
                let read = avakahada(moon).unwrap();
                let koota = ashta_koota(moon, moon, KootaRules::default());
                for row in &koota.kootas {
                    match row.reading {
                        KootaReading::Varna { bride, .. } => assert_eq!(bride, read.varna),
                        KootaReading::Yoni { bride, .. } => assert_eq!(bride, read.yoni),
                        KootaReading::GrahaMaitri { bride, .. } => {
                            assert_eq!(bride, read.rashi_lord);
                        }
                        KootaReading::Gana { bride, .. } => assert_eq!(bride, read.gana),
                        KootaReading::Nadi { bride, .. } => assert_eq!(bride, read.nadi),
                        _ => {}
                    }
                }
                assert_eq!(
                    (read.nakshatra, read.pada, read.rashi),
                    (nakshatra, pada, moon.rashi)
                );
            }
        }
    }
}
