//! A clause: one named condition from a source, and when it held.
//!
//! Raman is explicit that no moment is free of every defect, and that the
//! art is an excess of good and a deficiency of evil (*Muhurtha*, ch. V).
//! So the SDK does not collapse a day into a number: it reports each
//! clause the sources name, favourable or not, with the interval it held
//! over, and leaves the judgement to a ranking the caller chooses.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{
    Graha, Karana, MuhurtaYoga, Nakshatra, Panchaka, Rashi, Tithi, Vara, Yoga,
};
use teistro_core::interval::Interval;

use crate::tara::TaraReading;

/// What a clause says.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "clause", rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum ClauseKind {
    /// A tithi the rules reject (panchanga shuddhi).
    Tithi {
        /// Which.
        tithi: Tithi,
    },
    /// A nakshatra the rules reject.
    Nakshatra {
        /// Which.
        nakshatra: Nakshatra,
    },
    /// A yoga the rules reject.
    Yoga {
        /// Which.
        yoga: Yoga,
    },
    /// A karana the rules reject.
    Karana {
        /// Which.
        karana: Karana,
    },
    /// A vara the rules reject (vara dosha).
    Vara {
        /// Which.
        vara: Vara,
    },
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
    /// Jupiter or Venus in a kendra with every malefic in the 3rd, 6th
    /// or 11th (neutralisation 11).
    KendraBenefics {
        /// The benefics in a kendra.
        grahas: Vec<Graha>,
    },
}

impl ClauseKind {
    /// Whether the clause counts for the time rather than against it.
    ///
    /// A rejected limb is against it; a special yoga is for it when the
    /// catalogue marks the yoga auspicious (the two pushkara yogas
    /// multiply whatever the day brings and are not); a tara and a
    /// Chandrabala say which way they went.
    #[must_use]
    pub fn favourable(&self) -> bool {
        match self {
            ClauseKind::MuhurtaYoga { yoga } => yoga.attributes().auspicious,
            ClauseKind::Tarabala { reading } => reading.tara.favourable(),
            ClauseKind::Chandrabala { holds, .. } => *holds,
            ClauseKind::BeneficInLagna { .. }
            | ClauseKind::ExaltedInLagna { .. }
            | ClauseKind::LuminaryInEleventh { .. }
            | ClauseKind::KendraBenefics { .. } => true,
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
