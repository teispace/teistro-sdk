//! The two sides of a match.

use serde::{Deserialize, Serialize};

/// Which side of a match a birth stands on. Varna and Gana read
/// differently when the two swap, and the Kuja dosha is read on each, so a
/// match names them.
///
/// ```
/// use teistro_matching::MatchRole;
///
/// assert_eq!(MatchRole::Bride.other(), MatchRole::Groom);
/// ```
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
    ///
    /// ```
    /// use teistro_matching::MatchRole;
    ///
    /// assert_eq!(MatchRole::Groom.other(), MatchRole::Bride);
    /// assert_eq!(MatchRole::Bride.other().other(), MatchRole::Bride);
    /// ```
    #[must_use]
    pub const fn other(self) -> MatchRole {
        match self {
            MatchRole::Bride => MatchRole::Groom,
            MatchRole::Groom => MatchRole::Bride,
        }
    }

    /// The side's name in lower case, which a refusal is named by.
    ///
    /// ```
    /// use teistro_matching::{MatchRole, NaamRules, naam_milan};
    ///
    /// assert_eq!(MatchRole::Groom.name(), "groom");
    /// // A refusal of the groom's name is named under it.
    /// let refused = naam_milan("राम", "", NaamRules::default()).unwrap_err();
    /// assert_eq!(refused.field(), Some("groom.name"));
    /// ```
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            MatchRole::Bride => "bride",
            MatchRole::Groom => "groom",
        }
    }
}
