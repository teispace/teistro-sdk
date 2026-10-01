//! A clause: one named condition from a source, and when it held.
//!
//! Raman is explicit that no moment is free of every defect, and that the
//! art is an excess of good and a deficiency of evil (*Muhurtha*, ch. V).
//! So the SDK does not collapse a day into a number: it reports each
//! clause the sources name, favourable or not, with the interval it held
//! over, and leaves the judgement to a ranking the caller chooses.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{
    Choghadiya, Graha, Kaala, Karana, Masa, MuhurtaYoga, Nakshatra, Panchaka, Rashi, Tithi, Vara,
    Yoga,
};
use teistro_core::interval::Interval;

use crate::activity::Pada;
use crate::grade::Grade;
use crate::tara::TaraReading;

/// What a clause says.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "clause", rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum ClauseKind {
    /// A tithi the rules grade best or reject (panchanga shuddhi).
    Tithi {
        /// Which.
        tithi: Tithi,
        /// Best or rejected; a middling one is not reported.
        grade: Grade,
    },
    /// A nakshatra the rules grade best or reject.
    Nakshatra {
        /// Which.
        nakshatra: Nakshatra,
        /// Best or rejected.
        grade: Grade,
    },
    /// A yoga the rules grade best or reject.
    Yoga {
        /// Which.
        yoga: Yoga,
        /// Best or rejected.
        grade: Grade,
    },
    /// A karana the rules grade best or reject.
    Karana {
        /// Which.
        karana: Karana,
        /// Best or rejected.
        grade: Grade,
    },
    /// A vara the rules grade best or reject (vara dosha).
    Vara {
        /// Which.
        vara: Vara,
        /// Best or rejected.
        grade: Grade,
    },
    /// The lunar month an activity grades best or rejects (crux C161).
    Month {
        /// Which.
        masa: Masa,
        /// Best or rejected.
        grade: Grade,
    },
    /// The Sun's sign an activity keyed on the solar month grades best or
    /// rejects.
    SolarMonth {
        /// Which.
        sign: Rashi,
        /// Best or rejected.
        grade: Grade,
    },
    /// The lagna an activity grades best or rejects.
    Lagna {
        /// Which.
        sign: Rashi,
        /// Best or rejected.
        grade: Grade,
    },
    /// The Moon in a quarter of its star the activity rejects.
    Pada {
        /// Which.
        pada: Pada,
    },
    /// One of the inauspicious eighths of the daylight: Rahu kaala,
    /// Yamaghanda or Gulika.
    Kaala {
        /// Which.
        kaala: Kaala,
    },
    /// A choghadiya, for the time when the catalogue marks it auspicious
    /// and against it otherwise.
    Choghadiya {
        /// Which.
        choghadiya: Choghadiya,
    },
    /// Abhijit, the eighth muhurta of the daylight, on a day it is
    /// effective. For the time; a rite a tradition does not hold in it
    /// names it among its bars.
    Abhijit {},
    /// A special yoga of vara, tithi and nakshatra (Raman, ch. VI).
    MuhurtaYoga {
        /// Which.
        yoga: MuhurtaYoga,
    },
    /// The day's star counted from the native's.
    Tarabala {
        /// The count and its tara.
        reading: TaraReading,
    },
    /// The day's Moon sign counted from the native's.
    Chandrabala {
        /// The house, 1 to 12.
        house: u8,
        /// Whether the house gives Chandrabala under the rules' table.
        holds: bool,
    },
    /// Kartari: malefics on both sides of the lagna, in the 2nd and the
    /// 12th (Mahadosha 3).
    Kartari {
        /// The malefics in the 2nd.
        second: Vec<Graha>,
        /// The malefics in the 12th.
        twelfth: Vec<Graha>,
    },
    /// The Moon in the 6th, 8th or 12th from the lagna (Mahadosha 4).
    MoonInDusthana {
        /// Which of the three.
        house: u8,
    },
    /// The Moon in one sign with other grahas (sagraha Chandra,
    /// Mahadosha 5).
    MoonJoined {
        /// The grahas with it.
        with: Vec<Graha>,
    },
    /// Venus in the 6th from the lagna (Bhrigu shatka, Mahadosha 10).
    VenusInSixth {},
    /// Mars in the 8th from the lagna (Kujashtama, Mahadosha 11).
    MarsInEighth {},
    /// The lagna 8th from the native's birth lagna (Mahadosha 12).
    AshtamaLagna {},
    /// The lagna in a malefic's navamsa (kunavamsa, Mahadosha 14).
    Kunavamsa {
        /// The navamsa sign.
        navamsa: Rashi,
        /// Its lord.
        lord: Graha,
    },
    /// The remainder of tithi, vara, nakshatra and lagna by nine leaves a
    /// panchaka (Raman, ch. III; crux C159).
    PanchakaRemainder {
        /// Which.
        panchaka: Panchaka,
    },
    /// The part of a sign's rising the lagna tyajya rejects (Raman, ch.
    /// II; rasi visha ghatika, Mahadosha 13).
    LagnaTyajya {
        /// The sign rising.
        sign: Rashi,
    },
    /// A graha in the 7th from the lagna, which a marriage wants empty.
    SeventhOccupied {
        /// Who.
        by: Vec<Graha>,
    },
    /// A malefic in the lagna.
    MaleficInLagna {
        /// Who.
        grahas: Vec<Graha>,
    },
    /// Venus, Mercury or Jupiter in the lagna, which "will completely
    /// destroy all other adverse influences" (neutralisation 6).
    BeneficInLagna {
        /// Who.
        grahas: Vec<Graha>,
    },
    /// An exalted graha in the lagna (neutralisation 10).
    ExaltedInLagna {
        /// Who.
        grahas: Vec<Graha>,
    },
    /// The Sun or the Moon in the 11th (neutralisation 8).
    LuminaryInEleventh {
        /// Who.
        grahas: Vec<Graha>,
    },
    /// Jupiter or Venus in a kendra with the Sun, Mars and Saturn each in
    /// the 3rd, 6th or 11th (neutralisation 11). The nodes are not asked:
    /// they always stand opposite, and no two of those houses are (C167).
    KendraBenefics {
        /// The benefics in a kendra.
        grahas: Vec<Graha>,
    },
    /// Grahas standing in a house the rite wants them out of, as its
    /// rules' `unwanted` list names them: an 8th that should be
    /// unoccupied, Mars and Saturn out of the 5th. One clause a house,
    /// whichever entries of the list named it.
    UnwantedPlacement {
        /// The house, 1 to 12.
        house: u8,
        /// The grahas in it the rules name, in the catalogue's order.
        by: Vec<Graha>,
    },
}

/// Which clause, without what it found: the name a rule uses to point at
/// a kind of clause, as [`ActivityRules::bars`](crate::activity::ActivityRules)
/// does. It serialises as the clause's own `clause` tag.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum ClauseKey {
    /// [`ClauseKind::Tithi`].
    Tithi,
    /// [`ClauseKind::Nakshatra`].
    Nakshatra,
    /// [`ClauseKind::Yoga`].
    Yoga,
    /// [`ClauseKind::Karana`].
    Karana,
    /// [`ClauseKind::Vara`].
    Vara,
    /// [`ClauseKind::Month`].
    Month,
    /// [`ClauseKind::SolarMonth`].
    SolarMonth,
    /// [`ClauseKind::Lagna`].
    Lagna,
    /// [`ClauseKind::Pada`].
    Pada,
    /// [`ClauseKind::Kaala`].
    Kaala,
    /// [`ClauseKind::Choghadiya`].
    Choghadiya,
    /// [`ClauseKind::Abhijit`].
    Abhijit,
    /// [`ClauseKind::MuhurtaYoga`].
    MuhurtaYoga,
    /// [`ClauseKind::Tarabala`].
    Tarabala,
    /// [`ClauseKind::Chandrabala`].
    Chandrabala,
    /// [`ClauseKind::Kartari`].
    Kartari,
    /// [`ClauseKind::MoonInDusthana`].
    MoonInDusthana,
    /// [`ClauseKind::MoonJoined`].
    MoonJoined,
    /// [`ClauseKind::VenusInSixth`].
    VenusInSixth,
    /// [`ClauseKind::MarsInEighth`].
    MarsInEighth,
    /// [`ClauseKind::AshtamaLagna`].
    AshtamaLagna,
    /// [`ClauseKind::Kunavamsa`].
    Kunavamsa,
    /// [`ClauseKind::PanchakaRemainder`].
    PanchakaRemainder,
    /// [`ClauseKind::LagnaTyajya`].
    LagnaTyajya,
    /// [`ClauseKind::SeventhOccupied`].
    SeventhOccupied,
    /// [`ClauseKind::MaleficInLagna`].
    MaleficInLagna,
    /// [`ClauseKind::BeneficInLagna`].
    BeneficInLagna,
    /// [`ClauseKind::ExaltedInLagna`].
    ExaltedInLagna,
    /// [`ClauseKind::LuminaryInEleventh`].
    LuminaryInEleventh,
    /// [`ClauseKind::KendraBenefics`].
    KendraBenefics,
    /// [`ClauseKind::UnwantedPlacement`].
    UnwantedPlacement,
}

impl ClauseKey {
    /// Every key, in declaration order, so a caller can say which kinds a
    /// search reached and which it did not. A key added is appended here
    /// too: the test below holds each entry to its declared position.
    pub const ALL: [ClauseKey; 31] = [
        ClauseKey::Tithi,
        ClauseKey::Nakshatra,
        ClauseKey::Yoga,
        ClauseKey::Karana,
        ClauseKey::Vara,
        ClauseKey::Month,
        ClauseKey::SolarMonth,
        ClauseKey::Lagna,
        ClauseKey::Pada,
        ClauseKey::Kaala,
        ClauseKey::Choghadiya,
        ClauseKey::Abhijit,
        ClauseKey::MuhurtaYoga,
        ClauseKey::Tarabala,
        ClauseKey::Chandrabala,
        ClauseKey::Kartari,
        ClauseKey::MoonInDusthana,
        ClauseKey::MoonJoined,
        ClauseKey::VenusInSixth,
        ClauseKey::MarsInEighth,
        ClauseKey::AshtamaLagna,
        ClauseKey::Kunavamsa,
        ClauseKey::PanchakaRemainder,
        ClauseKey::LagnaTyajya,
        ClauseKey::SeventhOccupied,
        ClauseKey::MaleficInLagna,
        ClauseKey::BeneficInLagna,
        ClauseKey::ExaltedInLagna,
        ClauseKey::LuminaryInEleventh,
        ClauseKey::KendraBenefics,
        ClauseKey::UnwantedPlacement,
    ];
}

impl ClauseKind {
    /// Which clause this is.
    #[must_use]
    pub const fn key(&self) -> ClauseKey {
        match self {
            ClauseKind::Tithi { .. } => ClauseKey::Tithi,
            ClauseKind::Nakshatra { .. } => ClauseKey::Nakshatra,
            ClauseKind::Yoga { .. } => ClauseKey::Yoga,
            ClauseKind::Karana { .. } => ClauseKey::Karana,
            ClauseKind::Vara { .. } => ClauseKey::Vara,
            ClauseKind::Month { .. } => ClauseKey::Month,
            ClauseKind::SolarMonth { .. } => ClauseKey::SolarMonth,
            ClauseKind::Lagna { .. } => ClauseKey::Lagna,
            ClauseKind::Pada { .. } => ClauseKey::Pada,
            ClauseKind::Kaala { .. } => ClauseKey::Kaala,
            ClauseKind::Choghadiya { .. } => ClauseKey::Choghadiya,
            ClauseKind::Abhijit { .. } => ClauseKey::Abhijit,
            ClauseKind::MuhurtaYoga { .. } => ClauseKey::MuhurtaYoga,
            ClauseKind::Tarabala { .. } => ClauseKey::Tarabala,
            ClauseKind::Chandrabala { .. } => ClauseKey::Chandrabala,
            ClauseKind::Kartari { .. } => ClauseKey::Kartari,
            ClauseKind::MoonInDusthana { .. } => ClauseKey::MoonInDusthana,
            ClauseKind::MoonJoined { .. } => ClauseKey::MoonJoined,
            ClauseKind::VenusInSixth { .. } => ClauseKey::VenusInSixth,
            ClauseKind::MarsInEighth { .. } => ClauseKey::MarsInEighth,
            ClauseKind::AshtamaLagna { .. } => ClauseKey::AshtamaLagna,
            ClauseKind::Kunavamsa { .. } => ClauseKey::Kunavamsa,
            ClauseKind::PanchakaRemainder { .. } => ClauseKey::PanchakaRemainder,
            ClauseKind::LagnaTyajya { .. } => ClauseKey::LagnaTyajya,
            ClauseKind::SeventhOccupied { .. } => ClauseKey::SeventhOccupied,
            ClauseKind::MaleficInLagna { .. } => ClauseKey::MaleficInLagna,
            ClauseKind::BeneficInLagna { .. } => ClauseKey::BeneficInLagna,
            ClauseKind::ExaltedInLagna { .. } => ClauseKey::ExaltedInLagna,
            ClauseKind::LuminaryInEleventh { .. } => ClauseKey::LuminaryInEleventh,
            ClauseKind::KendraBenefics { .. } => ClauseKey::KendraBenefics,
            ClauseKind::UnwantedPlacement { .. } => ClauseKey::UnwantedPlacement,
        }
    }

    /// Whether the clause counts for the time rather than against it.
    ///
    /// A graded limb is for it when best and against it when rejected; a special yoga is for it when the
    /// catalogue marks the yoga auspicious (the two pushkara yogas
    /// multiply whatever the day brings and are not); a tara and a
    /// Chandrabala say which way they went.
    #[must_use]
    pub fn favourable(&self) -> bool {
        match self {
            ClauseKind::Tithi { grade, .. }
            | ClauseKind::Nakshatra { grade, .. }
            | ClauseKind::Yoga { grade, .. }
            | ClauseKind::Karana { grade, .. }
            | ClauseKind::Vara { grade, .. }
            | ClauseKind::Month { grade, .. }
            | ClauseKind::SolarMonth { grade, .. }
            | ClauseKind::Lagna { grade, .. } => *grade == Grade::Best,
            ClauseKind::MuhurtaYoga { yoga } => yoga.attributes().auspicious,
            ClauseKind::Choghadiya { choghadiya } => choghadiya.attributes().auspicious,
            ClauseKind::Tarabala { reading } => reading.tara.favourable(),
            ClauseKind::Chandrabala { holds, .. } => *holds,
            ClauseKind::BeneficInLagna { .. }
            | ClauseKind::ExaltedInLagna { .. }
            | ClauseKind::LuminaryInEleventh { .. }
            | ClauseKind::KendraBenefics { .. }
            | ClauseKind::Abhijit { .. } => true,
            _ => false,
        }
    }
}

/// A clause and the interval it held over.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Clause {
    /// What it says.
    #[serde(flatten)]
    pub kind: ClauseKind,
    /// When it held, clipped to the day.
    pub at: Interval,
}

impl Clause {
    /// Whether the clause counts for the time.
    #[must_use]
    pub fn favourable(&self) -> bool {
        self.kind.favourable()
    }
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]
mod tests {
    use super::{ClauseKey, ClauseKind};
    use crate::activity::Pada;
    use crate::grade::Grade;
    use teistro_core::catalogue::{Graha, Nakshatra, Tithi};

    /// A key names a clause the way the clause's own tag does, so a rule
    /// written in JSON can point at a clause by the name it reads on one.
    /// The two share a variant list (`key` matches every one) and a serde
    /// case, and this holds a sample of shapes — fielded, empty, nested —
    /// to it.
    #[test]
    fn every_key_is_listed_once_in_its_declared_place() {
        for (place, key) in ClauseKey::ALL.iter().enumerate() {
            assert_eq!(*key as usize, place, "{key:?}");
        }
    }

    #[test]
    fn a_key_serialises_as_its_clauses_tag() {
        for kind in [
            ClauseKind::Tithi {
                tithi: Tithi::ShuklaPratipada,
                grade: Grade::Rejected,
            },
            ClauseKind::VenusInSixth {},
            ClauseKind::Abhijit {},
            ClauseKind::Pada {
                pada: Pada {
                    nakshatra: Nakshatra::Mula,
                    pada: 1,
                },
            },
            ClauseKind::MoonJoined {
                with: vec![Graha::Mars],
            },
            ClauseKind::LuminaryInEleventh {
                grahas: vec![Graha::Sun],
            },
        ] {
            let tag = serde_json::to_value(&kind).expect("a clause serialises")["clause"].clone();
            let key = serde_json::to_value(kind.key()).expect("a key serialises");
            assert_eq!(tag, key, "{kind:?}");
            let back: ClauseKey = serde_json::from_value(tag).expect("a tag reads as a key");
            assert_eq!(back, kind.key());
        }
    }
}
