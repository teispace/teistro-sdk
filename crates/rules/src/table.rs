//! Tables a rule looks things up in, each with its citation
//! (`03-design/rules-engine.md`, "Table lookups over cited tables").
//!
//! Some rules are a table rather than a pattern: the degree of each sign that
//! is a graha's Mrityu Bhaga, the signs a birth tithi burns. A table is data
//! with its own [`Source`], so a reading with a different table is a different
//! table and not a different rule, and a rule names its table by key.
//!
//! The kinds are closed and typed: [`Table::DegreesBySign`] gives a body a
//! degree of each sign, and [`Table::SignsByTithi`] gives each tithi of the
//! fortnight a set of signs. A predicate asks for the kind it reads, and
//! [`Tables::check`] refuses a rule that names a table the set lacks or a table
//! of the other kind, before anything is evaluated.
//!
//! [`Tables::classical`] holds the tables the SDK ships, read from the texts:
//!
//! | key | source |
//! |---|---|
//! | `MRITYU_BHAGA` | Jataka Parijata ch. 1 v. 57 and its translator's statement of the other grahas' and the lagna's |
//! | `MRITYU_BHAGA_MOON_BRIHAT_PRAJAPATYA` | the Moon's row in Brihat Prajapatya, as that translator quotes it |
//! | `PUSHKARA_BHAGA_MOON` | Jataka Parijata ch. 1 v. 58 |
//! | `DAGDHA_RASHI` | Muhurta Chintamani, not read at rank 1; the engine's table and two secondary sources agree |
//!
//! ```
//! use teistro_core::catalogue::{Graha, Rashi};
//! use teistro_rules::{Body, SignDegree, Table, Tables};
//!
//! let tables = Tables::classical();
//! let Some(Table::DegreesBySign { rows, .. }) = tables.get("MRITYU_BHAGA") else {
//!     panic!("shipped")
//! };
//! // The Moon's Mrityu Bhaga in Mesha is her eighth degree (ch. 1 v. 57).
//! assert_eq!(rows[&Body::Graha(Graha::Moon)][Rashi::Aries as usize], SignDegree::try_new(8)?);
//! # Ok::<(), String>(())
//! ```

use std::collections::BTreeMap;
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Rashi, Tithi};

use crate::language::{Body, Condition, Rule, Source};

/// A degree of a sign, 1 to 30: the first degree runs from 0° to 1°.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct SignDegree(u8);

impl SignDegree {
    /// A degree.
    ///
    /// # Errors
    ///
    /// A number outside 1 to 30, named.
    pub fn try_new(degree: u8) -> Result<SignDegree, String> {
        if (1..=30).contains(&degree) {
            Ok(SignDegree(degree))
        } else {
            Err(format!("degree {degree} of a sign is not 1 to 30"))
        }
    }

    /// Its number, 1 to 30.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for SignDegree {
    type Error = String;

    fn try_from(degree: u8) -> Result<SignDegree, String> {
        SignDegree::try_new(degree)
    }
}

impl From<SignDegree> for u8 {
    fn from(degree: SignDegree) -> u8 {
        degree.0
    }
}

/// A table's key: capital letters, digits and underscores, a letter first.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct TableKey(String);

impl TableKey {
    /// A key.
    ///
    /// # Errors
    ///
    /// A key outside the grammar, named.
    pub fn try_new(key: impl Into<String>) -> Result<TableKey, String> {
        let key = key.into();
        let mut chars = key.chars();
        let well_formed = chars.next().is_some_and(|c| c.is_ascii_uppercase())
            && chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
        if well_formed {
            Ok(TableKey(key))
        } else {
            Err(format!(
                "`{key}` is not a table key: capital letters, digits and underscores, a letter first"
            ))
        }
    }

    /// The key.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for TableKey {
    type Error = String;

    fn try_from(key: String) -> Result<TableKey, String> {
        TableKey::try_new(key)
    }
}

impl From<TableKey> for String {
    fn from(key: TableKey) -> String {
        key.0
    }
}

impl core::fmt::Display for TableKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A table, of one of the kinds a predicate reads.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Table {
    /// For each body it lists, a degree of each sign, Mesha first.
    DegreesBySign {
        /// Its key.
        key: TableKey,
        /// Where it comes from.
        source: Source,
        /// Each body's twelve degrees.
        rows: BTreeMap<Body, [SignDegree; 12]>,
    },
    /// For each tithi of the fortnight, 1 to 15 in either paksha, a set of
    /// signs.
    SignsByTithi {
        /// Its key.
        key: TableKey,
        /// Where it comes from.
        source: Source,
        /// The fifteen tithis' signs, Pratipada first.
        rows: Box<[Vec<Rashi>; 15]>,
    },
}

impl Table {
    /// Its key.
    #[must_use]
    pub const fn key(&self) -> &TableKey {
        match self {
            Table::DegreesBySign { key, .. } | Table::SignsByTithi { key, .. } => key,
        }
    }

    /// Where it comes from.
    #[must_use]
    pub const fn source(&self) -> &Source {
        match self {
            Table::DegreesBySign { source, .. } | Table::SignsByTithi { source, .. } => source,
        }
    }

    /// Its kind, as a table writes it.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Table::DegreesBySign { .. } => DEGREES_BY_SIGN,
            Table::SignsByTithi { .. } => SIGNS_BY_TITHI,
        }
    }

    /// The degree a body's row gives a sign, if the table has the row.
    #[must_use]
    pub fn degree(&self, body: Body, sign: Rashi) -> Option<SignDegree> {
        match self {
            Table::DegreesBySign { rows, .. } => rows
                .get(&body)
                .and_then(|row| row.get(sign as usize))
                .copied(),
            Table::SignsByTithi { .. } => None,
        }
    }

    /// The signs a tithi's row gives, by its number in its paksha.
    #[must_use]
    pub fn signs(&self, tithi: Tithi) -> &[Rashi] {
        match self {
            Table::SignsByTithi { rows, .. } => rows
                .get(usize::from(tithi.attributes().number).saturating_sub(1))
                .map_or(&[], Vec::as_slice),
            Table::DegreesBySign { .. } => &[],
        }
    }
}

const DEGREES_BY_SIGN: &str = "degrees-by-sign";
const SIGNS_BY_TITHI: &str = "signs-by-tithi";

/// A set of tables, each key once.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tables {
    /// Sorted by key.
    tables: Vec<Table>,
}

/// The tables the SDK ships, as data.
const CLASSICAL: &str = include_str!("../tables/classical.json");

static CLASSICAL_TABLES: LazyLock<Tables> = LazyLock::new(|| {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct File {
        tables: Vec<Table>,
    }
    #[allow(
        clippy::expect_used,
        reason = "embedded data, read by a test on every build"
    )]
    let file: File = serde_json::from_str(CLASSICAL).expect("the shipped tables read");
    #[allow(
        clippy::expect_used,
        reason = "embedded data, read by a test on every build"
    )]
    Tables::new(file.tables).expect("the shipped tables' keys are distinct")
});

impl Tables {
    /// No tables.
    pub const EMPTY: Tables = Tables { tables: Vec::new() };

    /// A set of tables.
    ///
    /// # Errors
    ///
    /// Two tables with one key, naming it.
    pub fn new(mut tables: Vec<Table>) -> Result<Tables, String> {
        tables.sort_by(|a, b| a.key().cmp(b.key()));
        if let Some(pair) = tables
            .windows(2)
            .find(|pair| matches!(pair, [a, b] if a.key() == b.key()))
        {
            let key = pair.first().map(Table::key).map_or("", TableKey::as_str);
            return Err(format!("table `{key}` is given twice"));
        }
        Ok(Tables { tables })
    }

    /// The tables the SDK ships, each with its citation.
    #[must_use]
    pub fn classical() -> &'static Tables {
        &CLASSICAL_TABLES
    }

    /// These tables and `other`'s, a key in both refused.
    ///
    /// # Errors
    ///
    /// A key both sets hold, naming it.
    pub fn with(&self, other: &Tables) -> Result<Tables, String> {
        Tables::new(self.tables.iter().chain(&other.tables).cloned().collect())
    }

    /// The table under a key.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Table> {
        self.tables
            .binary_search_by(|table| table.key().as_str().cmp(key))
            .ok()
            .and_then(|at| self.tables.get(at))
    }

    /// Every table, by key.
    pub fn iter(&self) -> impl Iterator<Item = &Table> {
        self.tables.iter()
    }

    /// Whether every table a rule names is here and of the kind its predicate
    /// reads.
    ///
    /// # Errors
    ///
    /// The first table missing or of the other kind, naming the rule, the
    /// table, and what the set holds.
    pub fn check(&self, rule: &Rule) -> Result<(), String> {
        for condition in rule
            .conditions
            .iter()
            .chain(&rule.cancellations)
            .flat_map(Condition::walk)
        {
            let (table, wanted) = match condition {
                Condition::PlanetAtTableDegree { table, .. } => (table, DEGREES_BY_SIGN),
                Condition::PlanetInTableSign { table, .. } => (table, SIGNS_BY_TITHI),
                _ => continue,
            };
            match self.get(table.as_str()) {
                Some(found) if found.kind() == wanted => {}
                Some(found) => {
                    return Err(format!(
                        "rule `{}`: `{}` reads a {wanted} table, and `{table}` is {}",
                        rule.key,
                        condition.kind(),
                        found.kind()
                    ));
                }
                None => {
                    let known: Vec<&str> = self.iter().map(|t| t.key().as_str()).collect();
                    return Err(format!(
                        "rule `{}`: no table `{table}`; the tables are {}",
                        rule.key,
                        if known.is_empty() {
                            String::from("none")
                        } else {
                            known.join(", ")
                        }
                    ));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        clippy::panic,
        reason = "tests unwrap and index what they read, and fail by panicking"
    )]

    use teistro_core::catalogue::Graha;

    use super::*;

    #[test]
    fn the_shipped_tables_read_and_hold_the_texts_values() {
        let tables = Tables::classical();
        let keys: Vec<&str> = tables.iter().map(|t| t.key().as_str()).collect();
        assert_eq!(
            keys,
            [
                "DAGDHA_RASHI",
                "MRITYU_BHAGA",
                "MRITYU_BHAGA_MOON_BRIHAT_PRAJAPATYA",
                "PUSHKARA_BHAGA_MOON"
            ]
        );
        let mrityu = tables.get("MRITYU_BHAGA").unwrap();
        assert_eq!(mrityu.source().chapter.as_deref(), Some("1"));
        // Every body a rule names has a row, and the corners are the page's.
        let Table::DegreesBySign { rows, .. } = mrityu else {
            panic!("degrees")
        };
        assert_eq!(rows.len(), 10);
        let at = |body, sign| mrityu.degree(body, sign).map(SignDegree::get);
        assert_eq!(at(Body::Graha(Graha::Sun), Rashi::Aries), Some(20));
        assert_eq!(at(Body::Graha(Graha::Ketu), Rashi::Pisces), Some(14));
        assert_eq!(at(Body::Lagna, Rashi::Aquarius), Some(24));
        let moon = |key| {
            tables
                .get(key)
                .unwrap()
                .degree(Body::Graha(Graha::Moon), Rashi::Aquarius)
        };
        assert_eq!(moon("MRITYU_BHAGA").map(SignDegree::get), Some(20));
        assert_eq!(
            moon("MRITYU_BHAGA_MOON_BRIHAT_PRAJAPATYA").map(SignDegree::get),
            Some(5)
        );
        assert_eq!(moon("PUSHKARA_BHAGA_MOON").map(SignDegree::get), Some(19));
        assert_eq!(
            tables
                .get("MRITYU_BHAGA_MOON_BRIHAT_PRAJAPATYA")
                .unwrap()
                .degree(Body::Graha(Graha::Sun), Rashi::Aries),
            None
        );

        // A tithi reads by its number in either paksha.
        let dagdha = tables.get("DAGDHA_RASHI").unwrap();
        assert_eq!(
            dagdha.signs(Tithi::ShuklaPratipada),
            [Rashi::Libra, Rashi::Capricorn]
        );
        assert_eq!(
            dagdha.signs(Tithi::ShuklaPratipada),
            dagdha.signs(Tithi::KrishnaPratipada)
        );
        assert!(
            dagdha.signs(Tithi::Purnima).is_empty() && dagdha.signs(Tithi::Amavasya).is_empty()
        );
        assert_eq!(dagdha.signs(Tithi::KrishnaChaturdashi).len(), 4);
        assert_eq!(dagdha.degree(Body::Lagna, Rashi::Aries), None);
    }

    #[test]
    fn a_table_or_a_set_that_is_malformed_is_refused_with_its_reason() {
        let read = |json: &str| serde_json::from_str::<Table>(json).unwrap_err().to_string();
        let source = r#""source": {"text": "a text"}"#;
        for (json, reason) in [
            (
                format!(
                    r#"{{"kind": "degrees-by-sign", "key": "T", {source}, "rows": {{"SUN": [31,1,1,1,1,1,1,1,1,1,1,1]}}}}"#
                ),
                "degree 31",
            ),
            (
                format!(
                    r#"{{"kind": "degrees-by-sign", "key": "T", {source}, "rows": {{"SUN": [1,1]}}}}"#
                ),
                "an array of length 12",
            ),
            (
                format!(
                    r#"{{"kind": "degrees-by-sign", "key": "T", {source}, "rows": {{"MANDI": [1,1,1,1,1,1,1,1,1,1,1,1]}}}}"#
                ),
                "`MANDI` is not a body",
            ),
            (
                format!(r#"{{"kind": "signs-by-tithi", "key": "lower", {source}, "rows": []}}"#),
                "`lower` is not a table key",
            ),
            (
                format!(r#"{{"kind": "signs-by-weekday", "key": "T", {source}, "rows": []}}"#),
                "signs-by-weekday",
            ),
            (
                format!(
                    r#"{{"kind": "signs-by-tithi", "key": "T", "extra": 1, {source}, "rows": []}}"#
                ),
                "extra",
            ),
        ] {
            let error = read(&json);
            assert!(error.contains(reason), "{json}: {error}");
        }
        let twice = Tables::classical().with(Tables::classical()).unwrap_err();
        assert!(twice.contains("given twice"), "{twice}");
    }

    #[test]
    fn a_rule_naming_a_missing_table_or_the_wrong_kind_is_refused_before_evaluating() {
        let rule = |condition: &str| -> Rule {
            serde_json::from_str(&format!(
                r#"{{"key": "R", "category": "example", "source": {{"text": "a text"}}, "conditions": [{condition}]}}"#
            ))
            .unwrap()
        };
        let tables = Tables::classical();
        let degree = rule(
            r#"{"type": "planet-at-table-degree", "planet": "MOON", "table": "MRITYU_BHAGA"}"#,
        );
        assert_eq!(tables.check(&degree), Ok(()));
        let wrong =
            rule(r#"{"type": "planet-in-table-sign", "planet": "MOON", "table": "MRITYU_BHAGA"}"#);
        assert_eq!(
            tables.check(&wrong),
            Err(String::from(
                "rule `R`: `planet-in-table-sign` reads a signs-by-tithi table, and `MRITYU_BHAGA` is degrees-by-sign"
            ))
        );
        let missing = rule(
            r#"{"type": "not", "condition": {"type": "planet-at-table-degree", "planet": "SUN", "table": "NONE"}}"#,
        );
        assert!(
            tables
                .check(&missing)
                .unwrap_err()
                .contains("no table `NONE`; the tables are DAGDHA_RASHI")
        );
        assert!(
            Tables::EMPTY
                .check(&missing)
                .unwrap_err()
                .ends_with("the tables are none")
        );
    }
}
