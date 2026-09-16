//! The rules the SDK ships as data (`03-design/rules-engine.md`).
//!
//! [`computed_yogas`] holds the eight yogas it computes in code, the Neecha
//! Bhanga family, each written over any debilitated graha with a `for-any`.
//! [`computed_doshas`] holds the seventeen doshas the recording engine
//! computes in code rather than from its own condition language, written here
//! in the language instead: Kalsarpa and its twelve named forms, Kala Amrita,
//! Mrityu Bhaga over the `MRITYU_BHAGA` table, Dagdha Rashi over
//! `DAGDHA_RASHI`, and Badhaka over a badhaka reference. Each carries the
//! citation and the severity the engine's own rule declares, and each is
//! measured against what the engine recorded
//! (`03-design/doshas-measured.md`).
//!
//! ```
//! use teistro_rules::{shipped, Tables};
//!
//! let rules = shipped::computed_doshas();
//! assert_eq!(rules.len(), 17);
//! assert!(rules.iter().all(|rule| rule.is_evaluable()));
//! // Two of them read the shipped tables, and the set has both.
//! for rule in rules {
//!     Tables::classical().check(rule)?;
//! }
//! # Ok::<(), String>(())
//! ```

use std::sync::LazyLock;

use serde::Deserialize;

use crate::rule::Rule;

/// The doshas the recording engine computes in code, as rules.
const COMPUTED_DOSHAS: &str = include_str!("../rules/computed-doshas.json");
/// The yogas it computes in code: the Neecha Bhanga family.
const COMPUTED_YOGAS: &str = include_str!("../rules/computed-yogas.json");

/// The rules of one shipped file.
fn read(json: &str) -> Vec<Rule> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct File {
        rules: Vec<Rule>,
    }
    #[allow(
        clippy::expect_used,
        reason = "embedded data, read by a test on every build"
    )]
    let file: File = serde_json::from_str(json).expect("the shipped rules read");
    file.rules
}

static DOSHAS: LazyLock<Vec<Rule>> = LazyLock::new(|| read(COMPUTED_DOSHAS));
static YOGAS: LazyLock<Vec<Rule>> = LazyLock::new(|| read(COMPUTED_YOGAS));

/// The seventeen doshas the recording engine computes in code, as rules.
#[must_use]
pub fn computed_doshas() -> &'static [Rule] {
    &DOSHAS
}

/// The eight yogas it computes in code, the Neecha Bhanga family, as rules:
/// the aggregate and its seven cancellations, each over any debilitated graha.
#[must_use]
pub fn computed_yogas() -> &'static [Rule] {
    &YOGAS
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        reason = "tests unwrap what they read and fail by panicking"
    )]

    use super::*;
    use crate::language::EvidenceRank;
    use crate::table::Tables;

    #[test]
    fn every_shipped_rule_and_table_says_how_good_its_evidence_is() {
        let rules: Vec<&Rule> = computed_doshas().iter().chain(computed_yogas()).collect();
        assert_eq!(rules.len(), 25);
        for rule in &rules {
            let rank = rule
                .source
                .rank
                .unwrap_or_else(|| panic!("{}: a citation says its rank", rule.key));
            assert!((1..=3).contains(&rank.get()), "{}: {rank:?}", rule.key);
            assert!(
                rule.source.note.is_some(),
                "{}: a citation says what it rests on",
                rule.key
            );
        }
        for table in Tables::classical().iter() {
            assert!(
                table.source().rank.is_some(),
                "{}: a table says its rank",
                table.key()
            );
        }
        // The rules resting on no verse found are the ones the texts do not
        // carry: Kalsarpa's family, Kala Amrita and the Dagdha table's rule.
        let secondary: Vec<&str> = rules
            .iter()
            .filter(|rule| rule.source.rank == EvidenceRank::try_new(3).ok())
            .map(|rule| rule.key.as_str())
            .collect();
        assert_eq!(secondary.len(), 15, "{secondary:?}");
        assert!(secondary.contains(&"KALSARPA") && secondary.contains(&"DAGDHA_RASHI_DOSHA"));
        assert!(EvidenceRank::try_new(0).is_err() && EvidenceRank::try_new(5).is_err());
    }
}
