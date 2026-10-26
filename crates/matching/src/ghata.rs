//! The ghata chakra: the Moon sign, tithi, weekday and nakshatra that
//! strike a native, read from the native's Moon sign (C758).
//!
//! Muhurta Chintamani gives four rows in the *Yatra-prakarana* (XI,
//! vv. 27–30, the 1954 print with the *Piyushadhara*, pp. 347–348):
//!
//! - **v. 27, the ghata chandra**, as a count from the native's sign,
//!   1, 5, 9, 2, 6, 10, 3, 7, 4, 8, 11, 12 for Aries to Pisces;
//! - **v. 28, the tithi**, as one of the five classes, which the verse
//!   cites from the *Bhupalavallabha*;
//! - **v. 29, the weekday**;
//! - **v. 30, the nakshatra**.
//!
//! The commentary keeps the four for journeys, royal service, disputes
//! and war, and not for marriage or the other auspicious rites ("in every
//! auspicious work the ghata Moon is not considered"). It also calls the
//! weekday and nakshatra rows baseless, current only in the south.
//! Panchangas printed in Nepal and India copy all four rows the same way.
//! Rows later compilations add (the lagna, the female Moon, the month,
//! yoga, karana and prahara) are not built here. Their sources disagree
//! cell by cell, and the crux records where.
//!
//! ```
//! use teistro_core::catalogue::{Nakshatra, Rashi, TithiClass, Vara};
//! use teistro_matching::Ghata;
//!
//! // A native with the Moon in Taurus: the Moon in Virgo strikes (the
//! // 5th), and so do the Purna tithis, Saturday and Hasta.
//! let ghata = Ghata::of(Rashi::Taurus);
//! assert_eq!(ghata.chandra, Rashi::Virgo);
//! assert_eq!(ghata.tithi, TithiClass::Purna);
//! assert_eq!(ghata.vara, Vara::Shanivara);
//! assert_eq!(ghata.nakshatra, Nakshatra::Hasta);
//! ```

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Nakshatra, Rashi, TithiClass, Vara};

/// The four things that strike a native of one Moon sign (Muhurta
/// Chintamani XI vv. 27–30; C758).
///
/// ```
/// use teistro_core::catalogue::{Rashi, Vara};
/// use teistro_matching::Ghata;
///
/// // An Aries native is struck by the Moon in Aries itself, the 1st.
/// assert_eq!(Ghata::of(Rashi::Aries).chandra, Rashi::Aries);
/// assert_eq!(Ghata::of(Rashi::Aries).vara, Vara::Ravivara);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Ghata {
    /// The sign the Moon strikes from (v. 27), counted from the native's
    /// sign as the verse counts it and given here as the sign itself.
    pub chandra: Rashi,
    /// The class of tithi that strikes (v. 28).
    pub tithi: TithiClass,
    /// The weekday that strikes (v. 29).
    pub vara: Vara,
    /// The nakshatra that strikes (v. 30).
    pub nakshatra: Nakshatra,
}

/// One sign's row: the house the Moon strikes from (v. 27,
/// *bhū-pañca-aṅka-dvi-aṅga-dig-vahni-sapta-veda-aṣṭa-īśa-arkāḥ*), the
/// tithi class (v. 28), the weekday (v. 29) and the nakshatra (v. 30).
///
/// v. 30's reading *dhātry-agni-sārpam* would give Aquarius Krittika for
/// Ardra; the 1954 text and every panchanga read print Ardra.
const fn row(sign: Rashi) -> (u8, TithiClass, Vara, Nakshatra) {
    use Nakshatra as N;
    use TithiClass::{Bhadra, Jaya, Nanda, Purna, Rikta};
    use Vara as V;
    match sign {
        Rashi::Aries => (1, Nanda, V::Ravivara, N::Magha),
        Rashi::Taurus => (5, Purna, V::Shanivara, N::Hasta),
        Rashi::Gemini => (9, Bhadra, V::Somavara, N::Swati),
        Rashi::Cancer => (2, Bhadra, V::Budhavara, N::Anuradha),
        Rashi::Leo => (6, Jaya, V::Shanivara, N::Mula),
        Rashi::Virgo => (10, Purna, V::Shanivara, N::Shravana),
        Rashi::Libra => (3, Rikta, V::Guruvara, N::Shatabhisha),
        Rashi::Scorpio => (7, Nanda, V::Shukravara, N::Revati),
        Rashi::Sagittarius => (4, Jaya, V::Shukravara, N::Bharani),
        Rashi::Capricorn => (8, Rikta, V::Mangalavara, N::Rohini),
        Rashi::Aquarius => (11, Jaya, V::Guruvara, N::Ardra),
        // Pisces, the last of the twelve; the catalogue's enum is open.
        _ => (12, Purna, V::Shukravara, N::Ashlesha),
    }
}

impl Ghata {
    /// The ghata of a native whose Moon is in `moon_sign`.
    ///
    /// ```
    /// use teistro_core::catalogue::{Nakshatra, Rashi, TithiClass};
    /// use teistro_matching::Ghata;
    ///
    /// // Pisces counts to its own 12th, Aquarius.
    /// let pisces = Ghata::of(Rashi::Pisces);
    /// assert_eq!(pisces.chandra, Rashi::Aquarius);
    /// assert_eq!((pisces.tithi, pisces.nakshatra), (TithiClass::Purna, Nakshatra::Ashlesha));
    /// ```
    #[must_use]
    pub fn of(moon_sign: Rashi) -> Ghata {
        let (house, tithi, vara, nakshatra) = row(moon_sign);
        Ghata {
            chandra: Rashi::from_id((moon_sign.id() + u16::from(house) - 1) % 12)
                .unwrap_or(moon_sign),
            tithi,
            vara,
            nakshatra,
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{Ghata, row};
    use teistro_core::catalogue::{Rashi, TithiClass};

    /// The commentary turns the verse's counts into signs in a verse of
    /// its own, *meṣa-kanyā-ghaṭa-hari-nakra-yugma-dhanur-vṛṣāḥ /
    /// mīna-siṃha-dhanuḥ-kumbhāḥ*; the counts must give the same twelve.
    #[test]
    fn the_counts_give_the_commentarys_signs() {
        use Rashi::*;
        let signs = [
            Aries,
            Virgo,
            Aquarius,
            Leo,
            Capricorn,
            Gemini,
            Sagittarius,
            Taurus,
            Pisces,
            Leo,
            Sagittarius,
            Aquarius,
        ];
        for (native, sign) in Rashi::ALL.into_iter().zip(signs) {
            assert_eq!(Ghata::of(native).chandra, sign, "{native:?}");
        }
        // The twelve counts are the twelve houses, each once.
        let mut counts = Rashi::ALL.map(|sign| row(sign).0);
        counts.sort_unstable();
        assert_eq!(counts, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
    }

    #[test]
    fn every_class_strikes_some_sign_and_jaya_strikes_three() {
        let classes: Vec<TithiClass> = Rashi::ALL.map(|sign| Ghata::of(sign).tithi).to_vec();
        let jaya = classes.iter().filter(|c| **c == TithiClass::Jaya).count();
        let purna = classes.iter().filter(|c| **c == TithiClass::Purna).count();
        assert_eq!((jaya, purna), (3, 3));
        for class in [TithiClass::Nanda, TithiClass::Bhadra, TithiClass::Rikta] {
            assert_eq!(
                classes.iter().filter(|c| **c == class).count(),
                2,
                "{class:?}"
            );
        }
    }
}
