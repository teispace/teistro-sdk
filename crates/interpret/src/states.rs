//! The half of a graha's state no composer said: how it stands to its
//! dispositor, and the four avasthas
//! (`03-design/interpret-composers.md` §4).
//!
//! The twelfth composer, and the one the section table asked for. `STATE`
//! read as answered because [`conditions`](crate::conditions) says every
//! fact a `Placement` carries — nine of them, all said — while the
//! section's own `GrahaState` carries a dozen, and the other half reached
//! no reader at all.
//!
//! **Almost every word of it was already bought.** `Relationship` and the
//! four avastha kinds are catalogue members that all five locales name, so
//! the frame is the whole cost: six messages, not one new term, and
//! nothing for the native review to look at that it has not already seen.
//! That is the opposite of `DASHA_PHALA`, whose `nature` no locale names,
//! and it is why this composer follows it rather than leading.
//!
//! **What it does not say, it does not say by name.** The Sayanadi and the
//! Cheshta avasthas are catalogued and **unnamed** — the vetted tables
//! stop at the four — so a message for them would print nothing a locale
//! carries; `war` and `boundaries` are records whose own shape has not
//! been decided. All four are listed on the measured page's section table
//! rather than left to be noticed.
//!
//! **The undecided lajjitadi are said, and that is the point of them.**
//! `Lajjitadi` names three lists — holding, ruled out, and the ones
//! nothing in the chart decides — because the tradition's necessary
//! condition holds and what narrows it further is not on the page. A plan
//! that printed only what held would turn "we cannot tell" into "no",
//! which is the silent default this project refuses.

use teistro_intl::Value;
use teistro_intl::messages::sdk::condition;
use teistro_intl::messages::sdk::phala as phala_messages;
use teistro_state::chart::GrahaState;

use crate::{Plan, Vocabulary};

/// The form a corpus carries an avastha's own reading under, the same on
/// each of the four kinds.
const PHALA_FORM: &str = "phala";

/// What each graha **is**, where `conditions` says what its placement is.
///
/// One graha's items are in this order: how it stands to its dispositor,
/// the fifth of its sign it occupies, whether it is awake, its brightness
/// where the chart decides one, the lajjitadi that hold and the ones
/// nothing decides — then what a loaded corpus says of each of those four
/// avasthas.
///
/// The order is the section's, which is the catalogue's, so the same chart
/// always gives the same plan. A state whose dispositor the catalogue does
/// not give says nothing of its friendships rather than naming a default.
#[must_use]
pub fn states(states: &[GrahaState], vocabulary: &dyn Vocabulary) -> Plan {
    let mut plan = Plan::default();
    for state in states {
        if let Some(dispositor) = state.friendship.dispositor {
            plan.say(&condition::Friendship {
                graha: state.graha,
                dispositor,
                compound: state.friendship.compound,
                natural: state.friendship.natural,
                temporary: state.friendship.temporary,
            });
        }
        plan.say(&condition::Age {
            graha: state.graha,
            age: state.age,
        });
        plan.say(&condition::Wakefulness {
            graha: state.graha,
            wakefulness: state.wakefulness,
        });
        if let Some(deeptadi) = state.deeptadi {
            plan.say(&condition::Brightness {
                graha: state.graha,
                deeptadi,
            });
        }
        say_lajjitadi(&mut plan, state);
        say_what_a_corpus_carries(&mut plan, state, vocabulary);
    }
    plan
}

/// The lajjitadi that hold, and the ones the chart does not decide.
///
/// Two items and not one, because they are two claims: a state that holds
/// and a state nothing settles are different answers, and a plan that ran
/// them together would read the second as the first.
fn say_lajjitadi(plan: &mut Plan, state: &GrahaState) {
    let listed = |states: &[teistro_core::catalogue::AvasthaLajjitadi]| -> Vec<Value> {
        states.iter().map(|it| Value::catalogued(*it)).collect()
    };
    let holding = listed(&state.lajjitadi.holding);
    if !holding.is_empty() {
        plan.say(&condition::Lajjitadi {
            graha: state.graha,
            states: holding,
        });
    }
    let undecided = listed(&state.lajjitadi.undecided);
    if !undecided.is_empty() {
        plan.say(&condition::Undecided {
            graha: state.graha,
            count: i64::try_from(undecided.len()).unwrap_or(i64::MAX),
            states: undecided,
        });
    }
}

/// What a loaded corpus says of this graha's four avasthas.
///
/// The subject of each is the avastha itself and not the graha, which is
/// how the corpus keys them: `avastha_baladi.BALA` carries the reading of
/// *infancy*, and it is this graha's because this graha is in it.
fn say_what_a_corpus_carries(plan: &mut Plan, state: &GrahaState, vocabulary: &dyn Vocabulary) {
    if vocabulary.has_form(state.age.full_key(), PHALA_FORM) {
        plan.say(&phala_messages::AvasthaBaladi { avastha: state.age });
    }
    if vocabulary.has_form(state.wakefulness.full_key(), PHALA_FORM) {
        plan.say(&phala_messages::AvasthaJagradadi {
            avastha: state.wakefulness,
        });
    }
    if let Some(deeptadi) = state.deeptadi
        && vocabulary.has_form(deeptadi.full_key(), PHALA_FORM)
    {
        plan.say(&phala_messages::AvasthaDeeptadi { avastha: deeptadi });
    }
    for holding in &state.lajjitadi.holding {
        if vocabulary.has_form(holding.full_key(), PHALA_FORM) {
            plan.say(&phala_messages::AvasthaLajjitadi { avastha: *holding });
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use teistro_core::angle::Nas;
    use teistro_core::boundary::Boundaries;
    use teistro_core::catalogue::{
        AvasthaBaladi, AvasthaDeeptadi, AvasthaJagradadi, AvasthaLajjitadi, Dignity, Graha, Rashi,
        Relationship,
    };
    use teistro_core::quantity::Degrees;
    use teistro_state::avastha::Lajjitadi;
    use teistro_state::burn::Combustion;
    use teistro_state::chart::{GrahaState, Motion};
    use teistro_state::dignity::Friendship;

    use super::states;
    use crate::{KEYS, NoReadings, Vocabulary};

    /// A vocabulary carrying exactly the keys and forms it was given.
    struct Carries(&'static [(&'static str, &'static str)]);

    impl Vocabulary for Carries {
        fn has_form(&self, key: &str, form: &str) -> bool {
            self.0.iter().any(|(k, f)| *k == key && *f == form)
        }
    }

    /// One graha's state, with every field the composer reads settled and
    /// the rest at what a founded chart would give. Written out rather
    /// than defaulted, because a default would decide the very fields
    /// this composer is about.
    fn state(graha: Graha) -> GrahaState {
        GrahaState {
            graha,
            sign: Rashi::Aries,
            house: 1,
            dignity: Dignity::Neutral,
            friendship: Friendship {
                natural: Relationship::Friend,
                temporary: Relationship::Enemy,
                compound: Relationship::Neutral,
                dispositor: Some(Graha::Mars),
            },
            combustion: Combustion::clear(Some(60.0)),
            motion: Motion {
                retrograde: false,
                speed_deg_per_day: 1.0,
            },
            age: AvasthaBaladi::Kumara,
            wakefulness: AvasthaJagradadi::Jagrat,
            deeptadi: Some(AvasthaDeeptadi::Deepta),
            lajjitadi: Lajjitadi {
                holding: vec![AvasthaLajjitadi::Garvita],
                ruled_out: Vec::new(),
                undecided: vec![AvasthaLajjitadi::Lajjita],
            },
            war: None,
            sayanadi: None,
            boundaries: Boundaries::of(Nas::from_degrees(Degrees::try_new(15.0).unwrap())),
        }
    }

    /// Every item a graha contributes without a corpus, in the order the
    /// composer states.
    #[test]
    fn a_state_says_its_friendships_and_its_avasthas() {
        let plan = states(&[state(Graha::Sun)], &NoReadings);
        let said: Vec<&str> = plan.items.iter().map(|item| item.key.as_str()).collect();
        assert_eq!(
            said,
            [
                "sdk.condition.friendship",
                "sdk.condition.age",
                "sdk.condition.wakefulness",
                "sdk.condition.brightness",
                "sdk.condition.lajjitadi",
                "sdk.condition.undecided",
            ]
        );
        for key in plan.keys() {
            assert!(KEYS.contains(&key), "`{key}` is not in KEYS");
        }
    }

    /// What the chart does not decide is said as undecided and never as a
    /// state that holds, because a plan that printed only what held would
    /// turn "we cannot tell" into "no".
    #[test]
    fn the_undecided_are_their_own_item() {
        let plan = states(&[state(Graha::Sun)], &NoReadings);
        let at = |key: &str| {
            plan.items
                .iter()
                .find(|item| item.key == key)
                .unwrap_or_else(|| panic!("`{key}` is not in {plan:?}"))
        };
        let holding = at("sdk.condition.lajjitadi");
        let undecided = at("sdk.condition.undecided");
        assert_ne!(holding.params.get("states"), undecided.params.get("states"));
    }

    /// A graha whose brightness the chart does not decide says nothing of
    /// it, and one with neither list says neither item.
    #[test]
    fn nothing_undecided_is_said_at_a_default() {
        let mut quiet = state(Graha::Sun);
        quiet.deeptadi = None;
        quiet.lajjitadi.holding.clear();
        quiet.lajjitadi.undecided.clear();
        let plan = states(&[quiet], &NoReadings);
        let said: Vec<&str> = plan.items.iter().map(|item| item.key.as_str()).collect();
        assert_eq!(
            said,
            [
                "sdk.condition.friendship",
                "sdk.condition.age",
                "sdk.condition.wakefulness",
            ]
        );
    }

    /// A body the catalogue gives no sign has no dispositor, so its
    /// friendships are left out rather than said at a default.
    #[test]
    fn a_state_with_no_dispositor_says_nothing_of_its_friendships() {
        let mut orphan = state(Graha::Sun);
        orphan.friendship.dispositor = None;
        let plan = states(&[orphan], &NoReadings);
        assert!(
            plan.items
                .iter()
                .all(|item| item.key != "sdk.condition.friendship"),
            "{plan:?}"
        );
    }

    /// The corpus's readings are of the avastha and not of the graha, so
    /// they are said for the members this graha is in and no others.
    #[test]
    fn a_corpus_reading_is_of_the_avastha_this_graha_is_in() {
        let vocabulary = Carries(&[
            ("avastha_baladi.KUMARA", "phala"),
            ("avastha_jagradadi.JAGRAT", "phala"),
            ("avastha_deeptadi.DEEPTA", "phala"),
            ("avastha_lajjitadi.GARVITA", "phala"),
            // The one the chart does not decide: it must not be said.
            ("avastha_lajjitadi.LAJJITA", "phala"),
            // And another member of a kind this graha is not in.
            ("avastha_baladi.BALA", "phala"),
        ]);
        let plan = states(&[state(Graha::Sun)], &vocabulary);
        let said: Vec<&str> = plan
            .items
            .iter()
            .map(|item| item.key.as_str())
            .filter(|key| key.starts_with("sdk.phala."))
            .collect();
        assert_eq!(
            said,
            [
                "sdk.phala.avasthaBaladi",
                "sdk.phala.avasthaJagradadi",
                "sdk.phala.avasthaDeeptadi",
                "sdk.phala.avasthaLajjitadi",
            ],
            "one a kind, for the member this graha is in"
        );
        assert!(
            states(&[state(Graha::Sun)], &NoReadings)
                .items
                .iter()
                .all(|item| !item.key.starts_with("sdk.phala."))
        );
    }

    /// The order is the section's, so two grahas' items do not interleave.
    #[test]
    fn a_grahas_items_are_said_together() {
        let plan = states(&[state(Graha::Sun), state(Graha::Moon)], &NoReadings);
        let grahas: Vec<&teistro_intl::Value> = plan
            .items
            .iter()
            .filter_map(|item| item.params.get("graha"))
            .collect();
        let half = grahas.len() / 2;
        assert!(grahas[..half].iter().all(|it| *it == grahas[0]));
        assert!(grahas[half..].iter().all(|it| *it == grahas[half]));
        assert_ne!(grahas[0], grahas[half]);
    }
}
