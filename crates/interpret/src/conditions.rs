//! What a graha **is** where it stands, as against where it stands
//! (`03-design/interpret-composers.md` §4).
//!
//! It reads the same `RuleChart` [`placements`](crate::placements) and
//! [`positions`](crate::positions) read, so it costs a request no section of
//! its own, and it says the four of a placement's nine facts that neither of
//! them says: the dignity, the navamsha sign, whether that sign makes the
//! graha vargottama, whether it is retrograde and whether the Sun burns it.
//!
//! **The second composer to spend translation debt, and much the cheaper.**
//! `sdk.condition` is five messages written in English and Nepali, but two
//! of them say a value the entity namespace already names in all five
//! locales — a dignity and a rashi — so the terms that had to be written are
//! the three conditions that have no value to name: वक्री, अस्तंगत and
//! वर्गोत्तम, the tradition's own, flagged for the native review the
//! roadmap already requires, exactly as `sdk.aspect` was.
//!
//! **A dignity crosses as an entity and not as a string**, which is why the
//! message has no `.match` in it. [`Dignity`](teistro_core::catalogue::Dignity)
//! is `#[non_exhaustive]` and its members are appended, so eleven arms and a
//! catch-all would read a twelfth as the eleventh; an entity slot renders
//! each member's own word, in a locale that carries `sdk.entity` and no
//! message at all.
//!
//! **A dignity is said of every graha, `NEUTRAL` included**, where
//! [`aspects`](crate::aspects) skips `Strength::None`. The two look alike
//! and are not: no aspect is an absence, and the catch-all arm would have
//! said "fully" of it, while *sama* is a dignity the texts name. The three
//! conditions that **are** absences — not retrograde, not burnt, not
//! vargottama — are said only where they hold.

use teistro_core::catalogue::{Graha, State};
use teistro_intl::messages::sdk::condition;
use teistro_intl::messages::sdk::phala as phala_messages;
use teistro_rules::{Body, RuleChart};

use crate::{Plan, Vocabulary};

/// The form a corpus carries a condition's own reading under, the same on
/// the dignity kind and on the state kind.
const PHALA_FORM: &str = "phala";

/// What each of the nine grahas is where it stands.
///
/// The order is the catalogue's — the Sun to Ketu — and a graha's own
/// conditions are said together, so a consumer grouping a report by graha
/// reads them in one run and the same chart always gives the same plan. A
/// body the chart does not place is left out rather than said at a default,
/// as `placements` leaves it out.
///
/// ```
/// # use teistro_core::catalogue::{Dignity, Rashi};
/// # use teistro_rules::{House, Placement, RuleChart};
/// let placement = Placement {
///     longitude: 15.0,
///     sign: Rashi::Aries,
///     house: House::try_new(1)?,
///     dignity: Dignity::Exalted,
///     retrograde: true,
///     combust: false,
///     karaka7: None,
///     karaka8: None,
///     navamsha: Rashi::Aries,
/// };
/// let chart = RuleChart { placements: [placement; 10], panchanga: None, strengths: None };
///
/// let plan = teistro_interpret::conditions(&chart, &teistro_interpret::NoReadings);
/// // Each graha: its dignity, its navamsha, vargottama because the two
/// // signs agree, and retrograde. Nothing is burnt.
/// assert_eq!(plan.len(), 9 * 4);
/// assert_eq!(plan.items[0].key, "sdk.condition.dignity");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[must_use]
pub fn conditions(chart: &RuleChart, vocabulary: &dyn Vocabulary) -> Plan {
    let mut plan = Plan::default();
    for graha in Graha::ALL.into_iter().take(9) {
        let Some(at) = chart.placements.get(Body::Graha(graha).index()) else {
            continue;
        };
        plan.say(&condition::Dignity {
            graha,
            dignity: at.dignity,
        });
        if vocabulary.has_form(at.dignity.full_key(), PHALA_FORM) {
            plan.say(&phala_messages::Dignity {
                dignity: at.dignity,
            });
        }
        plan.say(&condition::Navamsha {
            graha,
            rashi: at.navamsha,
        });
        // Vargottama is the navamsha sign read as the named condition it is,
        // the way `aspects` says a `mutual` beside the two casts it is made
        // of: the fact is already said, and the name is what the texts read.
        if at.navamsha == at.sign {
            plan.say(&condition::Vargottama { graha });
            say_state(&mut plan, State::Vargottama, vocabulary);
        }
        if at.retrograde {
            plan.say(&condition::Retrograde { graha });
            say_state(&mut plan, State::Retrograde, vocabulary);
        }
        if at.combust {
            plan.say(&condition::Combust { graha });
            say_state(&mut plan, State::Combust, vocabulary);
        }
    }
    plan
}

/// What a corpus says of one condition, where it says anything.
///
/// The subject is the **condition** and not the graha, which is how the
/// corpus keys it: `state.COMBUST` carries the reading of *being burnt*,
/// and it is this graha's because this graha is burnt. So the item stands
/// beside the one that named the fact rather than replacing it — the fact
/// is the SDK's and the reading is the corpus's.
fn say_state(plan: &mut Plan, state: State, vocabulary: &dyn Vocabulary) {
    if vocabulary.has_form(state.full_key(), PHALA_FORM) {
        plan.say(&phala_messages::State { state });
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        clippy::indexing_slicing,
        reason = "tests unwrap what they build and fail by panicking"
    )]

    use teistro_core::catalogue::{Dignity, Rashi};
    use teistro_rules::{House, Placement};

    use super::*;
    use crate::NoReadings;
    use crate::{Item, KEYS};

    /// Ten bodies, each in the next sign, none of them vargottama,
    /// retrograde or burnt — so a composer that said an absence is visible
    /// in the count.
    fn quiet() -> RuleChart {
        let at = |index: usize| Placement {
            longitude: 15.0,
            sign: Rashi::ALL[index % 12],
            house: House::try_new(1).unwrap(),
            dignity: Dignity::Neutral,
            retrograde: false,
            combust: false,
            karaka7: None,
            karaka8: None,
            navamsha: Rashi::ALL[(index + 1) % 12],
        };
        RuleChart {
            placements: core::array::from_fn(at),
            panchanga: None,
            strengths: None,
        }
    }

    #[test]
    fn a_dignity_and_a_navamsha_are_said_of_every_graha() {
        let plan = conditions(&quiet(), &NoReadings);
        assert_eq!(plan.len(), 9 * 2, "nine grahas, two facts each");
        assert_eq!(
            plan.items[0],
            Item::of(&condition::Dignity {
                graha: Graha::Sun,
                dignity: Dignity::Neutral,
            })
        );
        assert_eq!(
            plan.items[1],
            Item::of(&condition::Navamsha {
                graha: Graha::Sun,
                rashi: Rashi::Taurus,
            })
        );
    }

    /// `NEUTRAL` is *sama*, a dignity the texts name, so it is said — unlike
    /// `Strength::None`, which `aspects` skips because it is an absence.
    #[test]
    fn a_neutral_dignity_is_a_dignity_and_is_said() {
        let plan = conditions(&quiet(), &NoReadings);
        let neutral = plan
            .items
            .iter()
            .filter(|item| item.key == "sdk.condition.dignity")
            .count();
        assert_eq!(neutral, 9);
    }

    /// The three conditions that are absences are said only where they hold.
    #[test]
    fn an_absence_is_not_said() {
        let written = serde_json::to_string(&conditions(&quiet(), &NoReadings)).unwrap();
        for absent in ["vargottama", "retrograde", "combust"] {
            assert!(!written.contains(absent), "{absent} in {written}");
        }
    }

    #[test]
    fn a_condition_is_said_where_it_holds() {
        let mut chart = quiet();
        let sun = &mut chart.placements[Body::Graha(Graha::Sun).index()];
        sun.navamsha = sun.sign;
        sun.retrograde = true;
        sun.combust = true;
        let plan = conditions(&chart, &NoReadings);
        assert_eq!(plan.len(), 9 * 2 + 3, "the Sun gains three");
        assert_eq!(
            plan.items[2],
            Item::of(&condition::Vargottama { graha: Graha::Sun })
        );
        assert_eq!(
            plan.items[3],
            Item::of(&condition::Retrograde { graha: Graha::Sun })
        );
        assert_eq!(
            plan.items[4],
            Item::of(&condition::Combust { graha: Graha::Sun })
        );
    }

    /// Vargottama is the navamsha sign read as a name, so it never stands
    /// without the fact it is read from.
    #[test]
    fn vargottama_never_stands_without_its_navamsha() {
        let mut chart = quiet();
        for placement in &mut chart.placements {
            placement.navamsha = placement.sign;
        }
        let plan = conditions(&chart, &NoReadings);
        let named = plan
            .items
            .iter()
            .filter(|item| item.key == "sdk.condition.vargottama")
            .count();
        let facts = plan
            .items
            .iter()
            .filter(|item| item.key == "sdk.condition.navamsha")
            .count();
        assert_eq!(named, 9);
        assert_eq!(facts, 9);
    }

    /// The lagna stands in the chart and is a point, not a graha, so this
    /// composer leaves it out exactly as `placements` does.
    #[test]
    fn the_lagna_is_left_out_as_it_is_everywhere_else() {
        let written = serde_json::to_string(&conditions(&quiet(), &NoReadings)).unwrap();
        assert!(!written.contains("LAGNA"), "{written}");
    }

    /// A vocabulary carrying exactly the keys and forms it was given.
    struct Carries(&'static [(&'static str, &'static str)]);

    impl Vocabulary for Carries {
        fn has_form(&self, key: &str, form: &str) -> bool {
            self.0.iter().any(|(k, f)| *k == key && *f == form)
        }
    }

    /// A corpus's reading of a condition stands **beside** the item that
    /// named the fact, and its subject is the condition and not the graha:
    /// `dignity.EXALTED` carries the reading of exaltation, and it is this
    /// graha's because this graha is exalted.
    #[test]
    fn a_corpus_reading_of_a_condition_stands_beside_the_fact() {
        let mut chart = quiet();
        let sun = &mut chart.placements[Body::Graha(Graha::Sun).index()];
        sun.dignity = Dignity::Exalted;
        sun.navamsha = sun.sign;
        sun.retrograde = true;
        let carried = Carries(&[
            ("dignity.EXALTED", "phala"),
            ("state.RETROGRADE", "phala"),
            ("state.VARGOTTAMA", "phala"),
        ]);
        let plan = conditions(&chart, &carried);
        let said: Vec<&str> = plan
            .items
            .iter()
            .take(6)
            .map(|item| item.key.as_str())
            .collect();
        assert_eq!(
            said,
            [
                "sdk.condition.dignity",
                "sdk.phala.dignity",
                "sdk.condition.navamsha",
                "sdk.condition.vargottama",
                "sdk.phala.state",
                "sdk.condition.retrograde",
            ],
            "the reading follows the fact it reads"
        );
        for key in plan.keys() {
            assert!(KEYS.contains(&key), "`{key}` is not in KEYS");
        }
        // Nothing is said of a condition the chart does not hold, however
        // much a pack carries: no graha here is burnt.
        let burnt = Carries(&[("state.COMBUST", "phala")]);
        assert!(
            conditions(&chart, &burnt)
                .items
                .iter()
                .all(|item| item.key != "sdk.phala.state"),
            "nothing is burnt"
        );
        // And a consumer with no pack gets the plan it had before.
        assert!(
            conditions(&chart, &NoReadings)
                .items
                .iter()
                .all(|item| !item.key.starts_with("sdk.phala.")),
        );
    }

    #[test]
    fn it_emits_only_listed_keys() {
        let mut chart = quiet();
        for placement in &mut chart.placements {
            placement.navamsha = placement.sign;
            placement.retrograde = true;
            placement.combust = true;
        }
        for key in conditions(&chart, &NoReadings).keys() {
            assert!(KEYS.contains(&key), "`{key}` is not in KEYS");
        }
    }

    #[test]
    fn a_plan_is_the_same_bytes_every_time() {
        let once = serde_json::to_string(&conditions(&quiet(), &NoReadings)).unwrap();
        let twice = serde_json::to_string(&conditions(&quiet(), &NoReadings)).unwrap();
        assert_eq!(once, twice);
        let read: Plan = serde_json::from_str(&once).unwrap();
        assert_eq!(read, conditions(&quiet(), &NoReadings));
    }
}
