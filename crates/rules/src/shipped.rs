//! The rules the SDK ships as data (`03-design/rules-engine.md`).
//!
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

static COMPUTED: LazyLock<Vec<Rule>> = LazyLock::new(|| {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct File {
        rules: Vec<Rule>,
    }
    #[allow(
        clippy::expect_used,
        reason = "embedded data, read by a test on every build"
    )]
    let file: File = serde_json::from_str(COMPUTED_DOSHAS).expect("the shipped rules read");
    file.rules
});

/// The seventeen doshas the recording engine computes in code, as rules.
#[must_use]
pub fn computed_doshas() -> &'static [Rule] {
    &COMPUTED
}
