//! What a loaded corpus says of Saturn's periods from the natal Moon: the
//! reading of each Sade Sati phase and each smaller spell a report holds
//! (`03-design/state-readings.md` §8).
//!
//! Like [`phala`](crate::phala) it says what a corpus **carries** and is
//! silent until a pack of state readings is loaded. The readings are keyed
//! by the house Saturn stands in from the Moon (`gochar_bhava.SATURN_IN_12`
//! for the rising phase), so a smaller spell the corpus has a reading of
//! is said the same way as a phase, and one it has none of is not said.

use teistro_core::catalogue::Graha;
use teistro_core::quantity::JulianDay;
use teistro_gochar::GocharFrom;
use teistro_gochar::sade_sati::{Report, Spell};
use teistro_intl::messages::sdk::phala;
use teistro_intl::source::gochar_bhava_key;

use crate::{Plan, Vocabulary};

/// The form a Saturn-from-the-Moon record carries its reading under.
const SADE_SATI_FORM: &str = "sadeSati";

/// What a loaded corpus says of the periods in `report`.
///
/// Each house is said **once**, in the order Saturn first reaches it in
/// the report: a century holds three Sade Satis, and the reading of the
/// rising phase is the same words each time. A spell already running when
/// the report's window opens comes first.
///
/// A report counted from the lagna says nothing. The corpus reads Saturn's
/// house from the natal Moon, which is what the seven and a half years
/// are, and a house counted from somewhere else is a different subject
/// that happens to share a number.
#[must_use]
pub fn sade_sati(report: &Report, vocabulary: &dyn Vocabulary) -> Plan {
    let mut plan = Plan::default();
    if report.reference.from != GocharFrom::Moon {
        return plan;
    }
    let mut spells: Vec<&Spell> = report
        .sade_sati
        .iter()
        .flat_map(|sade_sati| &sade_sati.phases)
        .chain(&report.spells)
        .collect();
    // Stable, so two spells both already running keep the report's order.
    spells.sort_by(|a, b| first_entry(a).total_cmp(&first_entry(b)));
    let mut said: Vec<u8> = Vec::new();
    for spell in spells {
        if said.contains(&spell.house) {
            continue;
        }
        said.push(spell.house);
        let key = gochar_bhava_key(Graha::Saturn.key(), spell.house);
        if vocabulary.has_form(&key, SADE_SATI_FORM) {
            plan.say(&phala::SadeSati {
                house: i64::from(spell.house),
                phala: key,
            });
        }
    }
    plan
}

/// When Saturn first entered the spell's house, or before anything for a
/// spell already running when the window opened.
fn first_entry(spell: &Spell) -> f64 {
    spell.begins().map_or(f64::NEG_INFINITY, JulianDay::get)
}

#[cfg(test)]
mod tests {
    use teistro_core::catalogue::Rashi;
    use teistro_gochar::Reference;
    use teistro_gochar::sade_sati::{Phase, Reckoning, SadeSati, Visit};
    use teistro_intl::Value;

    use super::*;
    use crate::NoReadings;

    /// A vocabulary carrying the `sadeSati` form on exactly these houses.
    struct Carries(&'static [u8]);

    impl Vocabulary for Carries {
        fn has_form(&self, key: &str, form: &str) -> bool {
            form == SADE_SATI_FORM
                && self
                    .0
                    .iter()
                    .any(|house| gochar_bhava_key("SATURN", *house) == key)
        }
    }

    fn spell(house: u8, from: Option<f64>, to: f64) -> Spell {
        Spell {
            house,
            visits: vec![Visit {
                from: from.map(JulianDay::literal),
                to: Some(JulianDay::literal(to)),
            }],
        }
    }

    /// Two Sade Satis a circuit apart with the 4th and 8th between, the
    /// 8th already running when the window opens.
    fn report(from: GocharFrom) -> Report {
        let circuit = 10_760.0;
        let sade_sati = |at: f64| SadeSati {
            phases: Phase::ALL
                .iter()
                .zip(0_u32..)
                .map(|(phase, k)| {
                    let begins = at + f64::from(k) * 900.0;
                    spell(phase.house(), Some(begins), begins + 900.0)
                })
                .collect(),
        };
        Report {
            reference: Reference {
                from,
                sign: Rashi::Aries,
            },
            reckoning: Reckoning::Sign,
            sade_sati: vec![sade_sati(5_000.0), sade_sati(5_000.0 + circuit)],
            spells: vec![spell(8, None, 1_000.0), spell(4, Some(10_000.0), 10_900.0)],
        }
    }

    /// The houses a plan says, each item checked to name its own house's
    /// record.
    fn houses(plan: &Plan) -> Vec<i64> {
        plan.items
            .iter()
            .map(|item| {
                assert_eq!(item.key, "sdk.phala.sadeSati");
                assert!(crate::KEYS.contains(&item.key.as_str()));
                let Some(Value::Int(house)) = item.params.get("house") else {
                    unreachable!("the frame's house slot is an integer: {item:?}");
                };
                assert_eq!(
                    item.params.get("phala"),
                    Some(&Value::Entity(format!("gochar_bhava.SATURN_IN_{house}")))
                );
                *house
            })
            .collect()
    }

    #[test]
    fn nothing_is_said_without_a_corpus() {
        assert!(sade_sati(&report(GocharFrom::Moon), &NoReadings).is_empty());
    }

    /// Each house once, in the order Saturn first reached it, and a house
    /// the corpus has no reading of is passed over.
    #[test]
    fn each_house_is_said_once_in_the_order_saturn_reached_it() {
        let every = sade_sati(&report(GocharFrom::Moon), &Carries(&[12, 1, 2, 4, 8]));
        assert_eq!(houses(&every), [8, 12, 1, 2, 4]);
        let phases = sade_sati(&report(GocharFrom::Moon), &Carries(&[12, 1, 2]));
        assert_eq!(houses(&phases), [12, 1, 2]);
    }

    /// The migration and the composer name a phase by the same key.
    ///
    /// The migration writes `rising` as `SATURN_IN_12` by a table, and the
    /// composer asks for `Phase::Rising.house()`: two spellings of one
    /// fact, held here so a phase that moved house in one would be caught.
    /// The smaller spells are the default ones, by the house they name.
    #[test]
    fn the_migration_keys_each_period_where_the_composer_asks() {
        use teistro_gochar::sade_sati::DEFAULT_SPELLS;
        use teistro_intl::migrate::STATE_KEY_ALIASES;

        let migrated: Vec<(&str, &str)> = STATE_KEY_ALIASES
            .iter()
            .filter(|(category, _, _)| *category == "sade-sati-phala")
            .map(|(_, from, to)| (*from, *to))
            .collect();
        let named = |from: &str| {
            migrated
                .iter()
                .find(|(key, _)| *key == from)
                .map(|(_, to)| format!("gochar_bhava.{to}"))
        };
        for (phase, name) in Phase::ALL.iter().zip(["rising", "peak", "setting"]) {
            assert_eq!(
                named(name),
                Some(gochar_bhava_key("SATURN", phase.house())),
                "{phase:?}"
            );
        }
        for house in DEFAULT_SPELLS {
            assert_eq!(
                named(&format!("dhaiyya_{house}th")),
                Some(gochar_bhava_key("SATURN", house))
            );
        }
        assert_eq!(migrated.len(), Phase::ALL.len() + DEFAULT_SPELLS.len());
    }

    /// A report counted from the lagna is not the corpus's subject.
    #[test]
    fn a_report_from_the_lagna_says_nothing() {
        let lagna = sade_sati(&report(GocharFrom::Lagna), &Carries(&[12, 1, 2, 4, 8]));
        assert!(lagna.is_empty());
    }
}
