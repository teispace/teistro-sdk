//! A clause: one named condition from a source, and when it held.
//!
//! Raman is explicit that no moment is free of every defect, and that the
//! art is an excess of good and a deficiency of evil (*Muhurtha*, ch. V).
//! So the SDK does not collapse a day into a number: it reports each
//! clause the sources name, favourable or not, with the interval it held
//! over, and leaves the judgement to a ranking the caller chooses.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Karana, MuhurtaYoga, Nakshatra, Tithi, Vara, Yoga};
use teistro_core::interval::Interval;

use crate::tara::TaraReading;

/// What a clause says.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
            _ => false,
        }
    }
}

/// A clause and the interval it held over.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
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
