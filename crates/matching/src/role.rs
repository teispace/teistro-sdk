//! The two sides of a match.

use serde::{Deserialize, Serialize};

/// Which side of a match a birth stands on. Varna and Gana read
/// differently when the two swap, and the Kuja dosha is read on each, so a
/// match names them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MatchRole {
    /// The bride's birth.
    Bride,
    /// The groom's birth.
    Groom,
}

impl MatchRole {
    /// The other side.
    #[must_use]
    pub const fn other(self) -> MatchRole {
        match self {
            MatchRole::Bride => MatchRole::Groom,
            MatchRole::Groom => MatchRole::Bride,
        }
    }

    /// The side's name in lower case, which a refusal is named by.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            MatchRole::Bride => "bride",
            MatchRole::Groom => "groom",
        }
    }
}
