//! The rules the SDK ships as data (`03-design/rules-engine.md`).
//!
//! [`arishtas`] holds BPHS ch. 9's evils at birth and ch. 10's antidotes.
//! [`gandantas`] holds the three gandantas of BPHS ch. 92 and its Abhukta
//! Moola, the SDK's first rules read from a text rather than mirrored from an
//! implementation. [`computed_yogas`] holds the eight yogas it computes in code, the Neecha
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
/// The gandantas, read from BPHS ch. 92 rather than from any engine.
const GANDANTA: &str = include_str!("../rules/classical-gandanta.json");
/// The evils at birth of BPHS ch. 9 and the antidotes of ch. 10.
const ARISHTA: &str = include_str!("../rules/classical-arishta.json");
/// Varahamihira's Balarishta, Brihat Jataka ch. 6.
const BALARISHTA: &str = include_str!("../rules/classical-balarishta.json");
/// Kalyana Varma's evils at birth, Saravali ch. 10, each with its life span.
const SARAVALI: &str = include_str!("../rules/classical-saravali.json");
/// The Nabhasa yogas of BPHS ch. 35.
const NABHASA: &str = include_str!("../rules/classical-nabhasa.json");
/// The lunar yogas of BPHS ch. 37 and the solar yogas of ch. 38.
const LUNAR_SOLAR: &str = include_str!("../rules/classical-lunar-solar.json");
/// The Pancha Mahapurusha yogas and the named yogas of BPHS ch. 36.
const PARASHARA: &str = include_str!("../rules/classical-parashara.json");
/// The arudha gains of BPHS ch. 29 and the Jaimini associations of ch. 39.
const JAIMINI: &str = include_str!("../rules/classical-jaimini.json");
/// The yogas of BPHS ch. 36 that ask a graha to be strong.
const STRENGTH: &str = include_str!("../rules/classical-strength.json");
/// The yogas leading to asceticism of BPHS ch. 79.
const PRAVRAJYA: &str = include_str!("../rules/classical-pravrajya.json");
/// Saravali ch. 34's readings of a named set of grahas in a named house.
const BHAVA: &str = include_str!("../rules/classical-bhava.json");
/// BPHS ch. 44's fate of the corpse, read from the twenty-second decanate.
const CORPSE: &str = include_str!("../rules/classical-corpse.json");
/// BPHS ch. 41's combinations for wealth.
const DHANA: &str = include_str!("../rules/classical-dhana.json");
/// BPHS ch. 42's combinations for penury.
const PENURY: &str = include_str!("../rules/classical-penury.json");
/// The table the dwigraha generator expands: Brihat Jataka ch. 14's
/// twenty-one pairs and Phaladeepika ch. 18's Moon in each sign, aspected.
const DWIGRAHA: &str = include_str!("../rules/classical-readings.json");

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

static READINGS: LazyLock<Vec<Rule>> = LazyLock::new(|| {
    #[allow(
        clippy::expect_used,
        reason = "embedded data, read by a test on every build"
    )]
    let table: crate::dwigraha::Table =
        serde_json::from_str(DWIGRAHA).expect("the dwigraha table reads");
    table.rules()
});
static DOSHAS: LazyLock<Vec<Rule>> = LazyLock::new(|| read(COMPUTED_DOSHAS));
static YOGAS: LazyLock<Vec<Rule>> = LazyLock::new(|| read(COMPUTED_YOGAS));
static GANDANTAS: LazyLock<Vec<Rule>> = LazyLock::new(|| read(GANDANTA));
static NABHASAS: LazyLock<Vec<Rule>> = LazyLock::new(|| {
    let mut rules = read(NABHASA);
    rules.append(&mut read(LUNAR_SOLAR));
    rules.append(&mut read(PARASHARA));
    rules.append(&mut read(JAIMINI));
    rules.append(&mut read(STRENGTH));
    rules.append(&mut read(PRAVRAJYA));
    rules.append(&mut read(BHAVA));
    rules.append(&mut read(CORPSE));
    rules.append(&mut read(DHANA));
    rules.append(&mut read(PENURY));
    rules
});
static ARISHTAS: LazyLock<Vec<Rule>> = LazyLock::new(|| {
    let mut rules = read(ARISHTA);
    rules.append(&mut read(BALARISHTA));
    rules.append(&mut read(SARAVALI));
    rules
});

/// The seventeen doshas the recording engine computes in code, as rules.
#[must_use]
pub fn computed_doshas() -> &'static [Rule] {
    &DOSHAS
}

/// The three gandantas of BPHS ch. 92 and the Abhukta Moola of its fifth
/// verse, each measured in ghatikas as the verses measure them, so a chart
/// must carry the ghatikas of the limb a rule reads
/// ([`Panchanga::spans`](crate::Panchanga)). These are the SDK's own rules,
/// read from the text rather than from any implementation.
#[must_use]
pub fn gandantas() -> &'static [Rule] {
    &GANDANTAS
}

/// The evils at birth of BPHS ch. 9 that the language can say, Brihat
/// Jataka ch. 6's Balarishta and Saravali ch. 10's spans beside them, and the
/// antidotes of BPHS ch. 10 and Saravali chs. 11 and 12, which stand in their
/// own chapters and so ship as rules of their own as well as being named by
/// the evils they cancel. The verses that turn on a graha being "strong" are
/// not here: the kernel has no strength measure, and a cancellation that fires
/// too often is worse than one that is missing.
#[must_use]
pub fn arishtas() -> &'static [Rule] {
    &ARISHTAS
}

/// The readings the texts give in one shape, built from a table rather than
/// written out: Brihat Jataka ch. 14's twenty-one pairs of grahas sharing a
/// sign, which Phaladeepika ch. 18 repeats; Phaladeepika's own Moon in each of
/// the twelve signs under each of six aspects; and Jataka Parijata's lists of
/// two to six of the seven sharing one sign, every combination of each size.
/// None of them grades anything, so each carries what its reading says in
/// words and nothing else.
#[must_use]
pub fn readings() -> &'static [Rule] {
    &READINGS
}

/// The yogas the SDK reads from BPHS: ch. 35's thirty-two Nabhasa yogas —
/// three ashraya, two dala, twenty akriti and seven sankhya, the sankhya seven
/// naming the other twenty-five as cancellations, as v. 17 requires — and
/// ch. 37's lunar and ch. 38's solar yogas beside them, the five Pancha
/// Mahapurusha yogas as Saravali ch. 37 gives them, and the named yogas of
/// ch. 36 the language can say. Each carries what its verse says and nothing
/// the SDK invented: a description, a span of life, or both.
#[must_use]
pub fn nabhasas() -> &'static [Rule] {
    &NABHASAS
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
        clippy::expect_used,
        clippy::panic,
        reason = "tests unwrap what they read and fail by panicking"
    )]

    use super::*;
    use crate::language::EvidenceRank;
    use crate::table::Tables;

    #[test]
    fn every_shipped_rule_and_table_says_how_good_its_evidence_is() {
        let rules: Vec<&Rule> = computed_doshas()
            .iter()
            .chain(computed_yogas())
            .chain(gandantas())
            .chain(arishtas())
            .chain(readings())
            .chain(nabhasas())
            .collect();
        assert_eq!(rules.len(), 878);
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
        // Every rule a shipped rule names is shipped beside it, and none
        // reaches itself.
        let owned: Vec<Rule> = rules.iter().map(|rule| (*rule).clone()).collect();
        assert_eq!(
            crate::rule::check_references(&owned),
            Ok(()),
            "the shipped rules name each other soundly"
        );
        assert!(secondary.contains(&"KALSARPA") && secondary.contains(&"DAGDHA_RASHI_DOSHA"));
        // A verse may say more than one thing, and a rule carries each: the
        // Pancha Mahapurusha verses both describe the native and count his
        // years, and they are the only rules that say two things.
        let two: Vec<&str> = rules
            .iter()
            .filter(|rule| rule.outcomes.len() > 1)
            .map(|rule| rule.key.as_str())
            .collect();
        assert_eq!(
            two,
            [
                "MAHAPURUSHA_RUCHAKA",
                "MAHAPURUSHA_BHADRA",
                "MAHAPURUSHA_HAMSA",
                "MAHAPURUSHA_MALAVYA",
                "MAHAPURUSHA_SASA"
            ]
        );
        for rule in rules.iter().filter(|rule| rule.outcomes.len() > 1) {
            assert_eq!(rule.outcomes.len(), 2, "{}", rule.key);
            assert!(rule.effect().is_some(), "{}", rule.key);
            let span = rule.life_span().unwrap_or_else(|| panic!("{}", rule.key));
            assert!((70.0 * 365.25..=100.0 * 365.25).contains(&span), "{}", rule.key);
        }
        assert!(EvidenceRank::try_new(0).is_err() && EvidenceRank::try_new(5).is_err());
    }

    /// The invariants the whole shipped set must hold, which no pack can check
    /// for itself: that a key names one rule, that every rule reads back as
    /// itself, and that the categories are a closed vocabulary.
    ///
    /// Key uniqueness is the load-bearing one. A rule names another by key for
    /// its cancellations, and an evaluator resolves that name from the set it
    /// was given, so two rules sharing a key would let a cancellation resolve
    /// to whichever the set happened to hold first.
    #[test]
    fn a_key_names_one_rule_and_every_rule_reads_back_as_itself() {
        let rules: Vec<&Rule> = computed_doshas()
            .iter()
            .chain(computed_yogas())
            .chain(gandantas())
            .chain(arishtas())
            .chain(readings())
            .chain(nabhasas())
            .collect();

        let mut seen: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
        for rule in &rules {
            *seen.entry(rule.key.as_str()).or_default() += 1;
        }
        let twice: Vec<&str> = seen
            .iter()
            .filter(|(_, count)| **count > 1)
            .map(|(key, _)| *key)
            .collect();
        assert!(twice.is_empty(), "keys naming more than one rule: {twice:?}");
        assert_eq!(seen.len(), rules.len());

        // Every rule writes out in the language and reads back the same, so a
        // pack the SDK ships is a pack a consumer can round-trip.
        let owned: Vec<Rule> = rules.iter().map(|rule| (*rule).clone()).collect();
        let written = serde_json::to_value(&owned).expect("the shipped rules write");
        let back: Vec<Rule> = serde_json::from_value(written).expect("and read back");
        assert_eq!(back, owned);

        // Every one of them is evaluable: the SDK ships no rule it cannot
        // answer, the engine's computed eight having been written out in the
        // language rather than carried as code.
        for rule in &rules {
            assert!(rule.is_evaluable(), "{} is evaluable", rule.key);
        }

        // The categories are a closed vocabulary, so a pack cannot quietly
        // invent one that a consumer grouping by category would miss.
        let categories: std::collections::BTreeSet<&str> =
            rules.iter().map(|rule| rule.category.as_str()).collect();
        assert_eq!(categories.into_iter().collect::<Vec<_>>(), CATEGORIES);
    }

    /// Every category the shipped rules use.
    const CATEGORIES: [&str; 28] = [
        "arishta",
        "arishta-bhanga",
        "arishta-father",
        "arishta-mother",
        "ayur",
        "chandra",
        "chandra-drishti",
        "daridra",
        "dhana",
        "dwigraha",
        "gandanta",
        "graha-in-bhava",
        "graha-in-rasi",
        "grahas-together",
        "house-based",
        "jaimini",
        "kalatra",
        "mahapurusha",
        "miscellaneous",
        "nabhasa",
        "neecha-bhanga",
        "nodal",
        "pair-in-angle",
        "panchanga",
        "positional",
        "pravrajya",
        "rising-part",
        "surya",
    ];
}
