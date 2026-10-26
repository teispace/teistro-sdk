//! The rules the SDK ships as data (`03-design/rules-engine.md`).
//!
//! [`arishtas`] holds the evils at birth and their antidotes: BPHS chs. 9
//! and 10, Brihat Jataka ch. 6, Saravali chs. 10 to 12, Jataka Parijata
//! ch. 4 and Phaladeepika ch. 13.
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

use crate::language::Condition;
use crate::rule::Rule;

/// The doshas the recording engine computes in code, as rules.
const COMPUTED_DOSHAS: &str = include_str!(concat!(env!("OUT_DIR"), "/rules/computed-doshas.json"));
/// The yogas it computes in code: the Neecha Bhanga family.
const COMPUTED_YOGAS: &str = include_str!(concat!(env!("OUT_DIR"), "/rules/computed-yogas.json"));
/// The gandantas, read from BPHS ch. 92 rather than from any engine.
const GANDANTA: &str = include_str!(concat!(env!("OUT_DIR"), "/rules/classical-gandanta.json"));
/// The other inauspicious births of BPHS ch. 85, with what chs. 86 to 94
/// say of each and their remedies.
const JANMA: &str = include_str!(concat!(env!("OUT_DIR"), "/rules/classical-bphs-janma.json"));
/// The evils at birth of BPHS ch. 9 and the antidotes of ch. 10.
const ARISHTA: &str = include_str!(concat!(env!("OUT_DIR"), "/rules/classical-arishta.json"));
/// Varahamihira's Balarishta, Brihat Jataka ch. 6.
const BALARISHTA: &str = include_str!(concat!(env!("OUT_DIR"), "/rules/classical-balarishta.json"));
/// Kalyana Varma's evils at birth, Saravali ch. 10, each with its life span.
const SARAVALI: &str = include_str!(concat!(env!("OUT_DIR"), "/rules/classical-saravali.json"));
/// Jataka Parijata ch. 4's evils at birth, their years and their antidotes.
const PARIJATA: &str = include_str!(concat!(env!("OUT_DIR"), "/rules/classical-parijata.json"));
/// Phaladeepika ch. 13's evils at birth and their antidotes.
const PHALADEEPIKA: &str = include_str!(concat!(
    env!("OUT_DIR"),
    "/rules/classical-phaladeepika-arishta.json"
));
/// The rest of Phaladeepika ch. 13 v. 8: the padas of the Moon that
/// harm a parent, the native or the line.
const PHALADEEPIKA_JANMA: &str = include_str!(concat!(
    env!("OUT_DIR"),
    "/rules/classical-phaladeepika-janma.json"
));
/// The Nabhasa yogas of BPHS ch. 35.
const NABHASA: &str = include_str!(concat!(env!("OUT_DIR"), "/rules/classical-nabhasa.json"));
/// The lunar yogas of BPHS ch. 37 and the solar yogas of ch. 38.
const LUNAR_SOLAR: &str = include_str!(concat!(
    env!("OUT_DIR"),
    "/rules/classical-lunar-solar.json"
));
/// The Pancha Mahapurusha yogas and the named yogas of BPHS ch. 36.
const PARASHARA: &str = include_str!(concat!(env!("OUT_DIR"), "/rules/classical-parashara.json"));
/// The arudha gains of BPHS ch. 29 and the Jaimini associations of ch. 39.
const JAIMINI: &str = include_str!(concat!(env!("OUT_DIR"), "/rules/classical-jaimini.json"));
/// The yogas of BPHS ch. 36 that ask a graha to be strong.
const STRENGTH: &str = include_str!(concat!(env!("OUT_DIR"), "/rules/classical-strength.json"));
/// The yogas leading to asceticism of BPHS ch. 79.
const PRAVRAJYA: &str = include_str!(concat!(env!("OUT_DIR"), "/rules/classical-pravrajya.json"));
/// Saravali ch. 34's readings of a named set of grahas in a named house.
const BHAVA: &str = include_str!(concat!(env!("OUT_DIR"), "/rules/classical-bhava.json"));
/// BPHS ch. 44's fate of the corpse, read from the twenty-second decanate.
const CORPSE: &str = include_str!(concat!(env!("OUT_DIR"), "/rules/classical-corpse.json"));
/// BPHS ch. 41's combinations for wealth.
const DHANA: &str = include_str!(concat!(env!("OUT_DIR"), "/rules/classical-dhana.json"));
/// BPHS ch. 39's raja yogas and ch. 40's yogas for royal association.
const RAJA: &str = include_str!(concat!(env!("OUT_DIR"), "/rules/classical-raja.json"));
/// BPHS ch. 44's manner and place of death and the worlds before and after.
const MARANA: &str = include_str!(concat!(env!("OUT_DIR"), "/rules/classical-marana.json"));
/// Phaladeepika ch. 14's diseases, death and the births before and after.
const PHALADEEPIKA_MARANA: &str = include_str!(concat!(
    env!("OUT_DIR"),
    "/rules/classical-phaladeepika-marana.json"
));
/// BPHS ch. 83's curses in a former birth that deny a son, and its two
/// figures for no son at all.
const SHAPA: &str = include_str!(concat!(env!("OUT_DIR"), "/rules/classical-bphs-shapa.json"));
/// BPHS ch. 43's combinations for the class of life.
const AYUR: &str = include_str!(concat!(env!("OUT_DIR"), "/rules/classical-ayur.json"));
/// Jataka Parijata ch. 4's middle and long lives.
const PARIJATA_AYUR: &str = include_str!(concat!(
    env!("OUT_DIR"),
    "/rules/classical-parijata-ayur.json"
));
/// BPHS ch. 42's combinations for penury.
const PENURY: &str = include_str!(concat!(env!("OUT_DIR"), "/rules/classical-penury.json"));
/// The table the dwigraha generator expands: Brihat Jataka ch. 14's
/// twenty-one pairs and Phaladeepika ch. 18's Moon in each sign, aspected.
const DWIGRAHA: &str = include_str!(concat!(env!("OUT_DIR"), "/rules/classical-readings.json"));

/// Phaladeepika ch. 6's yogas: the great men, the planets about the
/// Moon and the Sun, the exchanges of lords and the yogas of each house.
const PHALADEEPIKA_YOGA: &str = include_str!(concat!(
    env!("OUT_DIR"),
    "/rules/classical-phaladeepika-yoga.json"
));

/// Phaladeepika ch. 7's Maharaja yogas, the debilitated graha lifted
/// among them.
const PHALADEEPIKA_RAJA: &str = include_str!(concat!(
    env!("OUT_DIR"),
    "/rules/classical-phaladeepika-raja.json"
));

/// Brihat Jataka ch. 11's raja yogas: the exalted grahas, the vargottama
/// lagna and Moon, and the figures of named signs.
const BJ_RAJA: &str = include_str!(concat!(env!("OUT_DIR"), "/rules/classical-bj-raja.json"));

/// Brihat Jataka ch. 23's evil yogas: want of wife and sons, the diseases,
/// the senses and bondage.
const BJ_ANISHTA: &str = include_str!(concat!(env!("OUT_DIR"), "/rules/classical-bj-anishta.json"));

/// Brihat Jataka ch. 15's pravrajya yogas: the strongest of four or more in
/// one sign and its order, the combust leader, the lord of the Moon's sign
/// and Saturn, and the ascetic born in a raja yoga.
const BJ_PRAVRAJYA: &str = include_str!(concat!(
    env!("OUT_DIR"),
    "/rules/classical-bj-pravrajya.json"
));

/// Saravali ch. 35's raja yogas, vv. 7 to 185: the figures of named signs,
/// the Moon aspected and in her navamshas, the Yavanas' houses, the
/// yogas of the royal-born and the closing verses' nakshatras.
const SARAVALI_RAJA: &str = include_str!(concat!(
    env!("OUT_DIR"),
    "/rules/classical-saravali-raja.json"
));
/// Saravali chs. 13 and 14: Sunapha, Anapha and Durudhara around the Moon and
/// veshi, voshi and ubhayachari around the Sun, in general and graha by graha.
const SARAVALI_LUMINARIES: &str = include_str!(concat!(
    env!("OUT_DIR"),
    "/rules/classical-saravali-luminaries.json"
));
/// Jataka Parijata ch. 7 vv. 128 to 163: the named yogas, Parvata to
/// Hariharabrahma, four of them as BPHS ch. 36 writes them.
const PARIJATA_NAMED: &str = include_str!(concat!(
    env!("OUT_DIR"),
    "/rules/classical-parijata-named.json"
));
/// Jataka Parijata ch. 7 vv. 1 to 127: the raja yogas, the Moon's and the
/// Sun's yogas with their poverty and its cancellations, and the named yogas
/// Bhaskara to Budha.
const PARIJATA_RAJA: &str = include_str!(concat!(
    env!("OUT_DIR"),
    "/rules/classical-parijata-raja.json"
));
/// Brihat Jataka chs. 12 and 13: the thirty-two Nabhasa yogas, and the Moon's
/// yogas, Sunapha to Kemadruma, with her place from the Sun and the benefics
/// in the upachayas.
const BJ_NABHASA_CHANDRA: &str = include_str!(concat!(
    env!("OUT_DIR"),
    "/rules/classical-bj-nabhasa-chandra.json"
));
/// Jataka Parijata ch. 5A and ch. 6: the manner of death and the worlds before
/// and after, and the marring of a horoscope, its poverty, servitude and
/// diseases, some as BPHS ch. 44 and Brihat Jataka ch. 23 write them.
const PARIJATA_BHANGA: &str = include_str!(concat!(
    env!("OUT_DIR"),
    "/rules/classical-parijata-bhanga.json"
));
/// Phaladeepika chs. 10, 12 and 27: the seventh house and the wife, the
/// fifth and children, and the ascetics, one yoga of them as Brihat Jataka
/// ch. 23 writes it.
const PHALADEEPIKA_MORE: &str = include_str!(concat!(
    env!("OUT_DIR"),
    "/rules/classical-phaladeepika-more.json"
));
/// Brihat Jataka chs. 5, 7, 22, 24 and 25: the circumstances of the birth, the
/// life beyond reckoning, the happy births, a woman's horoscope, and the manner
/// of death and the worlds before and after, ch. 25 as Jataka Parijata ch. 5A
/// restates it.
const BJ_MORE: &str = include_str!(concat!(env!("OUT_DIR"), "/rules/classical-bj-more.json"));
/// Jataka Parijata ch. 3, ch. 8 and ch. 9: manifold births, the sex of the
/// child, twins, the cord and the parentage; the counts of exalted, own,
/// friendly and hostile grahas; the ganda nakshatras and the evil limbs of the
/// day, some as BPHS chs. 85 to 87 write them.
const PARIJATA_YOGA: &str = include_str!(concat!(
    env!("OUT_DIR"),
    "/rules/classical-parijata-yoga.json"
));
/// Uttara Kalamrita, Kanda I section 4 and Kanda II vv. 79 to 105: the lords
/// of the ninth and tenth and their kin, the houses destroyed and promoted,
/// the Indu lagna's wealth, the atmakaraka's companions, and the lagnas'
/// pairs, one yoga each as Phaladeepika ch. 7 and Jataka Parijata ch. 7 write
/// them.
const UTTARA_KALAMRITA: &str = include_str!(concat!(
    env!("OUT_DIR"),
    "/rules/classical-uttara-kalamrita.json"
));
/// BPHS chs. 12 to 23, the effects of the twelve houses: each verse that
/// states a figure, from the body and wealth to the father, work, gains and
/// losses, some as Brihat Jataka ch. 5, Jataka Parijata and Phaladeepika ch. 6
/// write them.
const BPHS_MORE: &str = include_str!(concat!(env!("OUT_DIR"), "/rules/classical-bphs-more.json"));
/// Saravali chs. 20, 38, 39, 44 and 46: the ascetics of four or more grahas in
/// one sign, the raja yogas undone, the long and unlimited lives, the counts of
/// exalted, own, friendly and hostile grahas, and the manner of death, some as
/// Brihat Jataka chs. 15 and 25, Phaladeepika ch. 27 and Jataka Parijata ch. 8
/// write them.
const SARAVALI_MORE: &str = include_str!(concat!(
    env!("OUT_DIR"),
    "/rules/classical-saravali-more.json"
));
/// Saravali ch. 45, a woman's chart, most as Brihat Jataka ch. 24 writes it,
/// and ch. 46 vv. 22 to 45, the 22nd drekkana's causes of death.
const SARAVALI_LATE: &str = include_str!(concat!(
    env!("OUT_DIR"),
    "/rules/classical-saravali-late.json"
));
/// Phaladeepika ch. 16, the effects of the twelve houses: each verse that
/// states a figure, from the lagna lord's place to the doers of the tenth.
const PHALADEEPIKA_LATE: &str = include_str!(concat!(
    env!("OUT_DIR"),
    "/rules/classical-phaladeepika-late.json"
));
/// Jataka Parijata ch. 3 vv. 61 to 79: the place of birth, the lamp, the
/// lying-in chamber and the marks of the body, most as Brihat Jataka ch. 5
/// writes them.
const PARIJATA_LATE: &str = include_str!(concat!(
    env!("OUT_DIR"),
    "/rules/classical-parijata-late.json"
));
/// BPHS chs. 24, 30, 33 and 80: the lords of the houses in the twelve
/// houses, the upapada, the Karakamsha and the woman's chart, some as Brihat
/// Jataka ch. 24, Phaladeepika ch. 10 and BPHS ch. 44 write them.
const BPHS_LATE: &str = include_str!(concat!(env!("OUT_DIR"), "/rules/classical-bphs-late.json"));
/// Brihat Jataka ch. 21, the grahas in the vargas: the counts of grahas in
/// their own, friends' and enemies' signs, the exalted graha with a friend,
/// the horas, the Moon's drekkana, the rising navamsha and the trimshamshas,
/// the friendly counts as Jataka Parijata ch. 8 writes them.
const BJ_VARGAS: &str = include_str!(concat!(env!("OUT_DIR"), "/rules/classical-bj-vargas.json"));

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
    rules.append(&mut read(RAJA));
    rules.append(&mut read(AYUR));
    rules.append(&mut read(PARIJATA_AYUR));
    rules.append(&mut read(MARANA));
    rules.append(&mut read(PHALADEEPIKA_MARANA));
    rules.append(&mut read(SHAPA));
    rules.append(&mut read(JANMA));
    rules.append(&mut read(PHALADEEPIKA_JANMA));
    rules.append(&mut read(PHALADEEPIKA_YOGA));
    rules.append(&mut read(PHALADEEPIKA_RAJA));
    rules.append(&mut read(BJ_RAJA));
    rules.append(&mut read(BJ_ANISHTA));
    rules.append(&mut read(BJ_PRAVRAJYA));
    rules.append(&mut read(SARAVALI_RAJA));
    rules.append(&mut read(SARAVALI_LUMINARIES));
    rules.append(&mut read(PARIJATA_NAMED));
    rules.append(&mut read(PARIJATA_RAJA));
    rules.append(&mut read(BJ_NABHASA_CHANDRA));
    rules.append(&mut read(PARIJATA_BHANGA));
    rules.append(&mut read(PHALADEEPIKA_MORE));
    rules.append(&mut read(BJ_MORE));
    rules.append(&mut read(PARIJATA_YOGA));
    rules.append(&mut read(UTTARA_KALAMRITA));
    rules.append(&mut read(BPHS_MORE));
    rules.append(&mut read(SARAVALI_MORE));
    rules.append(&mut read(SARAVALI_LATE));
    rules.append(&mut read(PHALADEEPIKA_LATE));
    rules.append(&mut read(PARIJATA_LATE));
    rules.append(&mut read(BPHS_LATE));
    rules.append(&mut read(BJ_VARGAS));
    rules
});
static ARISHTAS: LazyLock<Vec<Rule>> = LazyLock::new(|| {
    let mut rules = read(ARISHTA);
    rules.append(&mut read(BALARISHTA));
    rules.append(&mut read(SARAVALI));
    rules.append(&mut read(PARIJATA));
    rules.append(&mut read(PHALADEEPIKA));
    // The birth "in a gandanta" of BPHS ch. 9 v. 13 and Phaladeepika
    // ch. 13 v. 9 is ch. 92's, named by key.
    link(&mut rules, &GANDANTAS);
    rules
});

/// Writes each rule of `other` that a condition of `rules` names in place
/// of the name, as the conditions it holds by: the one, or their `and`.
///
/// A shipped set is asked on its own, and a set naming a rule it lacks is
/// refused (`check_references`); so where a verse means another set's rule,
/// as BPHS ch. 9 v. 13 means ch. 92's gandantas, the set carries that
/// rule's conditions rather than a second copy of them in its data.
fn link(rules: &mut [Rule], other: &[Rule]) {
    fn walk(condition: &mut Condition, other: &[Rule]) {
        if let Condition::RuleHolds { key } = condition
            && let Some(named) = other.iter().find(|rule| rule.key == *key)
        {
            // A rule holding by one condition is that condition, and an
            // `and` of one would read differently while saying the same.
            *condition = match named.conditions.as_slice() {
                [only] => only.clone(),
                all => Condition::And {
                    conditions: all.to_vec(),
                },
            };
            return;
        }
        for child in condition.children_mut() {
            walk(child, other);
        }
    }
    for rule in rules {
        for condition in &mut rule.conditions {
            walk(condition, other);
        }
    }
}

/// The seventeen doshas the recording engine computes in code, as rules.
///
/// ```
/// use teistro_rules::shipped;
///
/// let doshas = shipped::computed_doshas();
/// assert_eq!(doshas.len(), 17);
/// assert!(doshas.iter().any(|rule| rule.key == "KALSARPA"));
/// ```
#[must_use]
pub fn computed_doshas() -> &'static [Rule] {
    &DOSHAS
}

/// The three gandantas of BPHS ch. 92 and the Abhukta Moola of its fifth
/// verse, each measured in ghatikas as the verses measure them, so a chart
/// must carry the ghatikas of the limb a rule reads
/// ([`Panchanga::spans`](crate::Panchanga)). These are the SDK's own rules,
/// read from the text rather than from any implementation. Ch. 85's other
/// inauspicious births, which say what follows in words, are in
/// [`nabhasas`].
///
/// ```
/// use teistro_rules::shipped;
///
/// // Three gandantas and the Abhukta Moola, each measured in ghatikas.
/// let gandantas = shipped::gandantas();
/// assert_eq!(gandantas.len(), 4);
/// assert!(gandantas.iter().all(|rule| rule.is_evaluable()));
/// ```
#[must_use]
pub fn gandantas() -> &'static [Rule] {
    &GANDANTAS
}

/// The evils at birth of BPHS ch. 9 that the language can say, Brihat
/// Jataka ch. 6's Balarishta and Saravali ch. 10's spans beside them, and the
/// antidotes of BPHS ch. 10 and Saravali chs. 11 and 12, which stand in their
/// own chapters and so ship as rules of their own as well as being named by
/// the evils they cancel. A verse that turns on a graha being strong or
/// weak reads the chart's strength measure, so on a chart that carries none
/// it never holds, and neither does a cancellation that asks for strength.
///
/// ```
/// use teistro_rules::shipped;
///
/// let arishtas = shipped::arishtas();
/// // Every evil and antidote cites its text.
/// assert!(!arishtas.is_empty());
/// assert!(arishtas.iter().all(|rule| !rule.source.text.is_empty()));
/// ```
#[must_use]
pub fn arishtas() -> &'static [Rule] {
    &ARISHTAS
}

/// The readings the texts give in one shape, built from a table rather than
/// written out: Brihat Jataka ch. 14's twenty-one pairs of grahas sharing a
/// sign, which Phaladeepika ch. 18 repeats; Phaladeepika's own Moon in each of
/// the twelve signs under each of six aspects; and Jataka Parijata's lists of
/// two to six of the seven sharing one sign, every combination of each size;
/// and Saravali's graha in each sign, alone and under each aspect, in each
/// bhava, in pairs in the angles, and the part of a sign that rises. None of them grades anything, so each carries what its reading says in
/// words and nothing else.
///
/// ```
/// use teistro_rules::shipped;
///
/// // None grades anything: each says what its reading says, in words.
/// assert!(shipped::readings().iter().all(|rule| rule.effect().is_some() && rule.severity.is_none()));
/// ```
#[must_use]
pub fn readings() -> &'static [Rule] {
    &READINGS
}

/// The yogas the SDK reads from BPHS: ch. 35's thirty-two Nabhasa yogas —
/// three ashraya, two dala, twenty akriti and seven sankhya, the sankhya seven
/// naming the other twenty-five as cancellations, as v. 17 requires — and
/// ch. 37's lunar and ch. 38's solar yogas beside them, the five Pancha
/// Mahapurusha yogas as Saravali ch. 37 gives them, and the named yogas of
/// ch. 36 the language can say, and beside BPHS ch. 44's manner of death,
/// Phaladeepika ch. 14's diseases, death and the births before and after,
/// BPHS ch. 83's curses in a former birth that deny a son, and ch. 85's
/// inauspicious births with what chs. 86 to 94 say of each.
/// Each carries what its verse says and nothing the SDK invented: a
/// description, a span of life, or both.
///
/// ```
/// use teistro_rules::shipped;
///
/// // Each carries what its verse says: a description, a span of life, or both.
/// let nabhasas = shipped::nabhasas();
/// assert!(nabhasas.iter().all(|rule| rule.effect().is_some() || rule.life_span().is_some() || rule.life_class().is_some()));
/// ```
#[must_use]
pub fn nabhasas() -> &'static [Rule] {
    &NABHASAS
}

/// The eight yogas it computes in code, the Neecha Bhanga family, as rules:
/// the aggregate and its seven cancellations, each over any debilitated graha.
///
/// ```
/// use teistro_rules::shipped;
///
/// // The Neecha Bhanga family: the aggregate and its seven cancellations.
/// assert_eq!(shipped::computed_yogas().len(), 8);
/// ```
#[must_use]
pub fn computed_yogas() -> &'static [Rule] {
    &YOGAS
}

/// Every rule the crate ships, pack by pack: the one list of them, so a
/// consumer that reads them all (a timing, an audit) cannot miss one a
/// later chapter adds.
///
/// ```
/// use teistro_rules::{Tables, check_references, shipped};
///
/// // Every rule the crate ships names only rules and tables it ships.
/// let every: Vec<_> = shipped::packs().into_iter().flatten().cloned().collect();
/// check_references(&every)?;
/// for rule in &every {
///     Tables::classical().check(rule)?;
/// }
/// # Ok::<(), String>(())
/// ```
#[must_use]
pub fn packs() -> [&'static [Rule]; 6] {
    [
        nabhasas(),
        arishtas(),
        gandantas(),
        readings(),
        computed_doshas(),
        computed_yogas(),
    ]
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
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

    /// `packs` is the one list: it holds every rule every staged pack file
    /// holds, each once, so a file a later chapter adds and leaves out of
    /// the packs fails here, as does a pack that comes back empty.
    #[test]
    fn the_packs_hold_every_staged_rule_once() {
        let dir = concat!(env!("OUT_DIR"), "/rules");
        let mut staged = std::collections::BTreeMap::new();
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            let text = std::fs::read_to_string(&path).unwrap();
            // The two-graha readings are a table the rules are built from.
            let rules = if name == "classical-readings.json" {
                serde_json::from_str::<crate::dwigraha::Table>(&text)
                    .unwrap()
                    .rules()
            } else {
                read(&text)
            };
            assert!(!rules.is_empty(), "{name} holds no rule");
            for rule in rules {
                if let Some(earlier) = staged.insert(rule.key.clone(), name.clone()) {
                    panic!("{} is in both {earlier} and {name}", rule.key);
                }
            }
        }
        let mut shipped = std::collections::BTreeSet::new();
        for rule in packs().into_iter().flatten() {
            assert!(
                shipped.insert(rule.key.clone()),
                "{} is in two packs",
                rule.key
            );
        }
        let staged: std::collections::BTreeSet<String> = staged.into_keys().collect();
        let unwired: Vec<&String> = staged.difference(&shipped).collect();
        let unstaged: Vec<&String> = shipped.difference(&staged).collect();
        assert!(unwired.is_empty(), "staged but in no pack: {unwired:?}");
        assert!(
            unstaged.is_empty(),
            "in a pack but staged by no file: {unstaged:?}"
        );
    }

    /// What a caller asks a chart reading for is derived from the rules: the
    /// divisions they step into, whether they read strength, and whether they
    /// name a point. Held over the whole shipped set so a pack that starts
    /// reading a new division or a point is noticed.
    #[test]
    fn the_shipped_rules_say_which_divisions_strengths_and_points_they_read() {
        use teistro_core::catalogue::Varga;

        let rules: Vec<&Rule> = computed_doshas()
            .iter()
            .chain(computed_yogas())
            .chain(gandantas())
            .chain(arishtas())
            .chain(readings())
            .chain(nabhasas())
            .collect();
        let mut vargas: Vec<Varga> = rules.iter().flat_map(|rule| rule.vargas()).collect();
        vargas.sort();
        vargas.dedup();
        assert_eq!(
            vargas,
            [
                Varga::D2,
                Varga::D3,
                Varga::D7,
                Varga::D9,
                Varga::D10,
                Varga::D12,
                Varga::D16,
                Varga::D30,
                Varga::D60
            ]
        );
        // One rule names a division once however many times it steps in.
        for rule in &rules {
            let named: Vec<Varga> = rule.vargas().collect();
            let mut distinct = named.clone();
            distinct.dedup();
            assert_eq!(named.len(), distinct.len(), "{}", rule.key);
        }
        let strength = rules.iter().filter(|rule| rule.reads_strength()).count();
        let points = rules.iter().filter(|rule| rule.reads_points()).count();
        assert_eq!((strength, points), (344, 27));
        // And a rule naming a point, nested where a sign stands, says so.
        let gulika: Rule = serde_json::from_str(
            r#"{"key": "GULIKA_IN_LAGNA", "category": "arishta", "source": {"text": "BPHS"},
                "conditions": [{"type": "not", "condition": {"type": "same-sign",
                    "of": {"from": {"point": "GULIKA"}, "house": 1}, "as": 1}}]}"#,
        )
        .expect("a rule naming a point");
        assert!(gulika.reads_points() && !gulika.reads_strength());
    }

    #[test]
    fn the_arishtas_carry_the_gandantas_they_name_and_stand_alone() {
        let twilight = arishtas()
            .iter()
            .find(|rule| rule.key == "ARISHTA_TWILIGHT_MOONS_HORA_AND_GANDANTA")
            .expect("shipped");
        // No condition names a rule outside the set any more: each gandanta
        // is written as its own conditions.
        let referenced: Vec<&str> = arishtas()
            .iter()
            .flat_map(|rule| rule.conditions.iter())
            .flat_map(Condition::walk)
            .filter_map(|condition| match condition {
                Condition::RuleHolds { key } => Some(key.as_str()),
                _ => None,
            })
            .collect();
        assert!(
            gandantas()
                .iter()
                .all(|rule| !referenced.contains(&rule.key.as_str()))
        );
        let Some(Condition::Or { conditions }) = twilight.conditions.last() else {
            panic!("the gandanta is an `or` of three");
        };
        // A linked rule holding by several conditions is their `and`, and one
        // holding by a single condition is that condition.
        let written: Vec<&[Condition]> = conditions
            .iter()
            .map(|condition| match condition {
                Condition::And { conditions } => conditions.as_slice(),
                one => core::slice::from_ref(one),
            })
            .collect();
        let gandanta: Vec<&[Condition]> =
            ["TITHI_GANDANTA", "NAKSHATRA_GANDANTA", "LAGNA_GANDANTA"]
                .iter()
                .map(|key| {
                    gandantas()
                        .iter()
                        .find(|rule| rule.key == *key)
                        .map(|rule| rule.conditions.as_slice())
                        .expect("ch. 92 ships it")
                })
                .collect();
        assert_eq!(written, gandanta);
        let owned: Vec<Rule> = arishtas().to_vec();
        assert_eq!(crate::rule::check_references(&owned), Ok(()));
    }

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
        assert_eq!(rules.len(), 4819);
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
        // years. And a text may give one figure in two verses or three, each
        // with its own word, which one rule carries rather than a duplicate
        // for each verse: Saravali ch. 20's sets of grahas in one sign and
        // ch. 38's Sun in the tenth degree of Libra; BPHS ch. 33's Karakamsha
        // figures its later verses give again, and the verses of chs. 24, 30
        // and 80 that read one figure twice, as the visha kanya and her
        // children. They are the only rules that say more than one thing.
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
                "MAHAPURUSHA_SASA",
                "SARAVALI_PRAVRAJYA_SUN_MOON_MARS_MERCURY",
                "SARAVALI_PRAVRAJYA_SUN_MARS_MERCURY_VENUS_SATURN",
                "SARAVALI_PRAVRAJYA_SUN_MOON_MARS_MERCURY_JUPITER",
                "SARAVALI_PRAVRAJYA_SUN_MARS_MERCURY_JUPITER",
                "SARAVALI_PRAVRAJYA_MARS_MERCURY_JUPITER_SATURN",
                "SARAVALI_PRAVRAJYA_SUN_MOON_MARS_MERCURY_VENUS_SATURN",
                "SARAVALI_PRAVRAJYA_SUN_MOON_MARS_JUPITER_VENUS_SATURN",
                "SARAVALI_BHANGA_SUN_IN_THE_TENTH_DEGREE_OF_LIBRA",
                "BPHS_BHAVESHA_EIGHTH_LORD_IN_THE_EIGHTH_WEAK",
                "BPHS_UPAPADA_SUN_JUPITER_AND_RAHU_NINTH_FROM_IT",
                "BPHS_KARAKAMSHA_MALEFIC_THIRD_FROM_IT",
                "BPHS_KARAKAMSHA_BENEFIC_THIRD_FROM_IT",
                "BPHS_KARAKAMSHA_VENUS_IN_IT_OR_THE_FIFTH",
                "BPHS_KARAKAMSHA_MERCURY_IN_IT_OR_THE_FIFTH",
                "BPHS_STRI_VISHA_KANYA_BY_DAY_TITHI_AND_NAKSHATRA",
                "BPHS_STRI_VISHA_KANYA_BY_THE_LAGNA_AND_MALEFICS_IN_ENEMY_SIGNS",
            ]
        );
        for rule in rules
            .iter()
            .filter(|rule| rule.key.starts_with("MAHAPURUSHA_"))
        {
            assert_eq!(rule.outcomes.len(), 2, "{}", rule.key);
            assert!(rule.effect().is_some(), "{}", rule.key);
            let span = rule.life_span().unwrap_or_else(|| panic!("{}", rule.key));
            assert!(
                (70.0 * 365.25..=100.0 * 365.25).contains(&span),
                "{}",
                rule.key
            );
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
        assert!(
            twice.is_empty(),
            "keys naming more than one rule: {twice:?}"
        );
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
    const CATEGORIES: [&str; 37] = [
        "arishta",
        "arishta-bhanga",
        "arishta-father",
        "arishta-mother",
        "ashubha-janma",
        "ayur",
        "chandra",
        "chandra-drishti",
        "chandra-nakshatra",
        "daridra",
        "dhana",
        "dwigraha",
        "gandanta",
        "graha-aspected-in-rasi",
        "graha-in-bhava",
        "graha-in-rasi",
        "grahas-together",
        "house-based",
        "jaimini",
        "kalatra",
        "loka",
        "mahapurusha",
        "marana",
        "miscellaneous",
        "nabhasa",
        "navamsa-drishti",
        "neecha-bhanga",
        "nodal",
        "pair-in-angle",
        "panchanga",
        "positional",
        "pravrajya",
        "raja",
        "rising-part",
        "roga",
        "shapa",
        "surya",
    ];
}
