//! What a festival is and how it is kept, as facts
//! (`03-design/festival-about.md`, SR-153): for each festival or group
//! described, the acts it is kept with, in a page's order, and the sources
//! they come from. The words are the locales': a festival's `about` form
//! and each act's `observance` record.

use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

/// One festival or group described: the acts it is kept with and the
/// works they come from.
///
/// ```
/// use teistro_panchanga::festival::FestivalFacts;
///
/// let tihar = FestivalFacts::of("TIHAR").ok_or("described")?;
/// assert!(tihar.observance.iter().any(|act| act == "LAMPS"));
/// assert!(!tihar.sources.is_empty());
/// # Ok::<(), &str>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FestivalFacts {
    /// The festival's rule key or its group's key, as `INDRA_JATRA` or
    /// `TIHAR`.
    pub key: String,
    /// The acts it is kept with, in the order a page lists them, each an
    /// `observance` key.
    pub observance: Vec<String>,
    /// The works its facts come from, each cited to its page, verse or
    /// section.
    pub sources: Vec<String>,
}

/// The shipped descriptions, read once.
static FACTS: LazyLock<Vec<FestivalFacts>> = LazyLock::new(|| {
    #[allow(
        clippy::expect_used,
        reason = "embedded data, read by a test on every build"
    )]
    serde_json::from_str(include_str!("../../data/festival-about.json"))
        .expect("the shipped festival descriptions read")
});

impl FestivalFacts {
    /// Every festival and group described, in the data's order.
    ///
    /// ```
    /// use teistro_panchanga::festival::FestivalFacts;
    ///
    /// assert!(FestivalFacts::all().len() >= 40);
    /// ```
    #[must_use]
    pub fn all() -> &'static [FestivalFacts] {
        &FACTS
    }

    /// The description of one festival or group, by key.
    ///
    /// ```
    /// use teistro_panchanga::festival::FestivalFacts;
    ///
    /// assert!(FestivalFacts::of("INDRA_JATRA").is_some());
    /// assert!(FestivalFacts::of("NO_SUCH_DAY").is_none());
    /// ```
    #[must_use]
    pub fn of(key: &str) -> Option<&'static FestivalFacts> {
        FACTS.iter().find(|facts| facts.key == key)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::FestivalFacts;

    /// A key as the catalogue spells a member: capitals, digits and single
    /// underscores.
    fn well_formed(key: &str) -> bool {
        key.bytes().next().is_some_and(|b| b.is_ascii_uppercase())
            && key
                .bytes()
                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
            && !key.ends_with('_')
            && !key.contains("__")
    }

    #[test]
    fn every_description_is_keyed_once_lists_its_acts_once_and_cites_a_source() {
        let mut keys = BTreeSet::new();
        for facts in FestivalFacts::all() {
            assert!(well_formed(&facts.key), "{}", facts.key);
            assert!(keys.insert(&facts.key), "{} is described twice", facts.key);
            assert!(!facts.observance.is_empty(), "{} lists no act", facts.key);
            let acts: BTreeSet<&String> = facts.observance.iter().collect();
            assert_eq!(
                acts.len(),
                facts.observance.len(),
                "{} repeats an act",
                facts.key
            );
            assert!(
                facts.observance.iter().all(|act| well_formed(act)),
                "{}",
                facts.key
            );
            assert!(
                !facts.sources.is_empty() && facts.sources.iter().all(|s| !s.trim().is_empty()),
                "{} cites no source",
                facts.key
            );
        }
    }
}
