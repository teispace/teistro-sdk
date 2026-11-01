//! The avakahada: the janma-patrika's birth summary read off the Moon
//! (`03-design/matching.md`, C301).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Gana, Graha, Nadi, Nakshatra, Paya, Rashi, Varna, Yoni};
use teistro_core::error::Error;

use crate::{
    BirthSyllable, Ghata, Native, birth_syllable, house_from_lagna, paya_by_house,
    paya_by_nakshatra, sign_varna,
};

/// What a janma-patrika prints of the Moon: its star and pada, the
/// syllable the child is named by, and the readings a koota takes of one
/// native (C301).
///
/// A gathering, not doctrine of its own: each reading is the one the Ashta
/// Koota reads of the same Moon, so it carries no rules. The paya is
/// read by both methods a patrika prints (C1105 to C1107); vashya, disha
/// and tatwa are not here (C300, C301).
///
/// ```
/// use teistro_core::catalogue::{Graha, Nadi, Rashi};
/// use teistro_matching::{Native, avakahada};
///
/// // Rohini, in Taurus: the Moon's own star in Venus's sign.
/// let read = avakahada(Native::of_moon(45.0)?)?;
/// assert_eq!((read.rashi, read.nakshatra_lord, read.rashi_lord), (Rashi::Taurus, Graha::Moon, Graha::Venus));
/// assert_eq!(read.nadi, Nadi::Antya);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
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
    /// The ghata chakra of the Moon's sign: what strikes the native
    /// (C758).
    pub ghata: Ghata,
    /// The paya by the Moon's nakshatra (C1106).
    pub paya_by_nakshatra: Paya,
    /// The paya by the Moon's house counted from the lagna's sign
    /// (C1105, C1107); `None` until [`Avakahada::with_lagna`] reads a
    /// lagna, and always present on a chart's avakahada.
    pub paya_by_house: Option<Paya>,
}

impl Avakahada {
    /// The avakahada with the paya by house read from the lagna's sign
    /// (C1105, C1107).
    ///
    /// ```
    /// use teistro_core::catalogue::{Paya, Rashi};
    /// use teistro_matching::{Native, avakahada};
    ///
    /// // A Moon in Taurus under a Virgo lagna is in the 9th: silver.
    /// let read = avakahada(Native::of_moon(45.0)?)?.with_lagna(Rashi::Virgo);
    /// assert_eq!(read.paya_by_house, Some(Paya::Rajata));
    /// // Rohini, by its star: iron.
    /// assert_eq!(read.paya_by_nakshatra, Paya::Loha);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn with_lagna(self, lagna: Rashi) -> Self {
        Self {
            paya_by_house: paya_by_house(house_from_lagna(lagna, self.rashi)),
            ..self
        }
    }
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
        ghata: Ghata::of(moon.rashi),
        paya_by_nakshatra: paya_by_nakshatra(moon.nakshatra),
        paya_by_house: None,
    })
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_core::catalogue::Nakshatra;

    use crate::{Ghata, KootaReading, KootaRules, Native, ashta_koota, avakahada};

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
                assert_eq!(read.ghata, Ghata::of(moon.rashi));
            }
        }
    }
}
