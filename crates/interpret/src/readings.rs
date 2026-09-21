//! What the rules answered: for each rule a chart held, what its verse says
//! (`03-design/interpret-composers.md` §4).
//!
//! Unlike the placements, this composer has messages of its own,
//! `sdk.reading`, because nothing in the packs said any of this. They are
//! deliberately **mechanical** — a span, a class of life, who took part,
//! whether it still stands and how grave it is — because those are the parts
//! a locale can say for itself. The verse's own statement is not translated:
//! it crosses as a slot, in the words the rule carries and cites, until a
//! locale holds a reading of that rule written by someone who reads the text.
//!
//! Every item names its rule in a `rule` slot the base messages do not
//! print, so a consumer can group a plan by rule, and a locale that wants
//! the key in its prose has it.

use teistro_core::catalogue::Point;
use teistro_intl::Value;
use teistro_intl::messages::sdk::reading;
use teistro_rules::{Body, NetStatus, Outcome, Rule, RuleResult, Unit};

use crate::{Plan, Vocabulary};

/// The unit a span is counted in, as its message selects on it.
const fn unit(unit: Unit) -> &'static str {
    match unit {
        Unit::Days => "days",
        Unit::Months => "months",
        Unit::Years => "years",
    }
}

/// The longest span a message will print: BPHS's illimitable life is a
/// thousand years and Saravali counts in days, so a million of anything is
/// past every text and past every rounding question.
const LONGEST: f64 = 1_000_000.0;

/// A span's count as its message takes it. The spans the texts give are
/// whole numbers of their unit — a test over every shipped rule holds it,
/// because a message pluralises a count and cannot pluralise half a year —
/// and anything outside the range a text could mean is clamped rather than
/// wrapped.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "clamped to 0..=1e6 first, so the cast is exact"
)]
fn whole(count: f64) -> i64 {
    count.round().clamp(0.0, LONGEST) as i64
}

/// A body as a slot: a graha by its catalogue key, the lagna as the point it
/// is.
fn body(body: Body) -> Value {
    match body {
        Body::Graha(graha) => Value::catalogued(graha),
        Body::Lagna => Value::catalogued(Point::Lagna),
    }
}

/// What each rule a chart held says.
///
/// One rule's items are in this order: what its verse says, in as many
/// statements as it makes; who took part; whether a cancellation moved it;
/// and how grave it is. A rule whose verse states nothing and
/// whose result says nothing else contributes nothing — a composer that
/// filled the gap would be inventing.
///
/// **What its verse says has two forms, and the vocabulary chooses.** Where
/// the base locale carries a reading of the rule, the item is
/// `sdk.reading.says`, which names the reading as an entity and lets each
/// locale render its own words. Where it does not, the item is
/// `sdk.reading.effect`, which carries the verse's cited words as a slot
/// and prints them untranslated — the visible seam, kept rather than
/// papered over with a machine translation.
///
/// A rule that states its effect in several statements gets **one** reading
/// where it has one: the reading is of the rule, not of a statement, and
/// saying the same passage three times would be a defect. Its cited
/// statements are what it has instead when no reading was written.
///
/// The question goes to the **base** locale, so a plan is the same whoever
/// reads it ([`Vocabulary`]).
#[must_use]
pub fn readings<'r, I>(present: I, vocabulary: &dyn Vocabulary) -> Plan
where
    I: IntoIterator<Item = (&'r Rule, &'r RuleResult)>,
{
    let mut plan = Plan::default();
    for (rule, result) in present {
        let named = || rule.key.clone();
        // A reading is of the **rule**, not of one of its statements, so it
        // is said once for a rule that has one and it replaces whatever
        // statements that rule cites. That also means a rule stating no
        // effect at all — every computed dosha, the whole neecha-bhanga
        // family — says its reading where one was written, which is the
        // case the first draft missed: those rules were silent in words
        // and a written reading is exactly what they were missing.
        let carried = vocabulary.has_reading(&rule.key);
        if carried {
            plan.say(&reading::Says {
                rule: named(),
                reading: crate::reading_key(&rule.key),
            });
        }
        for outcome in &result.outcomes {
            match outcome {
                Outcome::Effect { .. } if carried => {}
                Outcome::Effect { text } => plan.say(&reading::Effect {
                    rule: named(),
                    text: text.clone(),
                }),
                Outcome::LifeSpan { count, unit: each } => plan.say(&reading::LifeSpan {
                    rule: named(),
                    // Whole by construction — a test over every shipped rule
                    // says so — because a message pluralises a count and
                    // cannot pluralise half a year.
                    count: whole(*count),
                    unit: String::from(unit(*each)),
                }),
                Outcome::LifeClass { class } => plan.say(&reading::LifeClass {
                    rule: named(),
                    class: String::from(class.key()),
                }),
            }
        }
        let grahas: Vec<Value> = result.participants.iter().map(body).collect();
        if !grahas.is_empty() {
            plan.say(&reading::Participants {
                rule: named(),
                // The count is the message's, for the verb it agrees with;
                // the list is what it prints.
                count: i64::try_from(grahas.len()).unwrap_or(i64::MAX),
                grahas,
            });
        }
        // Only when it is not simply in force: a rule that nothing cancelled
        // says so by saying nothing, as the prose of a rule does.
        if let Some(status) = result.status.filter(|status| *status != NetStatus::Active) {
            plan.say(&reading::Status {
                rule: named(),
                status: String::from(status.key()),
            });
        }
        if let Some(severity) = result.severity {
            plan.say(&reading::Severity {
                rule: named(),
                severity: i64::from(severity),
            });
        }
    }
    plan
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        clippy::indexing_slicing,
        reason = "tests unwrap what they build and fail by panicking"
    )]

    use teistro_core::catalogue::Graha;
    use teistro_rules::{Evaluator, Readings};

    use super::*;
    use crate::placements::tests::chart;
    use crate::{Item, KEYS, NoReadings, Plan};

    /// A vocabulary that carries a reading for the rules named, which is
    /// what a loaded readings pack gives a composer.
    struct Carried(&'static [&'static str]);

    impl Vocabulary for Carried {
        fn has_form(&self, key: &str, form: &str) -> bool {
            form == crate::NAME_FORM
                && key
                    .strip_prefix("rule.")
                    .is_some_and(|rule| self.0.contains(&rule))
        }
    }

    fn rule(json: &str) -> Rule {
        serde_json::from_str(json).unwrap_or_else(|err| panic!("{json} reads: {err}"))
    }

    /// A rule that holds on the test chart, with a verse that says three
    /// kinds of thing at once.
    fn present() -> (Rule, RuleResult) {
        let rule = rule(
            r#"{
                "key": "AN_EXAMPLE",
                "category": "example",
                "source": { "text": "an example" },
                "conditions": [{ "type": "planet-in-house", "planet": "SUN", "houses": [1] }],
                "outcomes": [
                    { "type": "effect", "text": "wealth and a kingdom" },
                    { "type": "life-span", "count": 70, "unit": "years" },
                    { "type": "life-class", "class": "long" }
                ]
            }"#,
        );
        let chart = chart();
        let result = Evaluator::new(&chart, Readings::TEXTS).evaluate(&rule);
        assert!(result.present);
        (rule, result)
    }

    #[test]
    fn a_rule_says_its_verse_and_who_made_it() {
        let (rule, result) = present();
        let plan = readings([(&rule, &result)], &NoReadings);
        assert_eq!(plan.len(), 4);
        assert_eq!(
            plan.items[0],
            Item::of(&reading::Effect {
                rule: String::from("AN_EXAMPLE"),
                text: String::from("wealth and a kingdom"),
            })
        );
        assert_eq!(
            plan.items[1],
            Item::of(&reading::LifeSpan {
                rule: String::from("AN_EXAMPLE"),
                count: 70,
                unit: String::from("years"),
            })
        );
        assert_eq!(
            plan.items[2],
            Item::of(&reading::LifeClass {
                rule: String::from("AN_EXAMPLE"),
                class: String::from("long"),
            })
        );
        assert_eq!(
            plan.items[3],
            Item::of(&reading::Participants {
                rule: String::from("AN_EXAMPLE"),
                count: 1,
                grahas: vec![Value::catalogued(Graha::Sun)],
            })
        );
        for key in plan.keys() {
            assert!(KEYS.contains(&key), "`{key}` is not in KEYS");
        }
    }

    /// A rule whose verse states nothing and whose result says nothing else
    /// contributes nothing: the composer does not invent a sentence.
    #[test]
    fn a_rule_with_nothing_to_say_says_nothing() {
        let quiet = rule(
            r#"{
                "key": "QUIET",
                "category": "example",
                "source": { "text": "an example" },
                "conditions": [{ "type": "birth-by-day" }]
            }"#,
        );
        let chart = chart();
        let result = Evaluator::new(&chart, Readings::TEXTS).evaluate(&quiet);
        assert!(!result.present, "the chart says nothing of the day");
        // Present or not, a rule with no outcome and no participants says
        // nothing at all.
        assert!(readings([(&quiet, &result)], &NoReadings).is_empty());
    }

    /// The status and the severity are said when the result carries them.
    #[test]
    fn a_cancelled_rule_says_so() {
        let (rule, mut result) = present();
        result.status = Some(NetStatus::PartiallyCancelled);
        result.severity = Some(60);
        let plan = readings([(&rule, &result)], &NoReadings);
        assert_eq!(
            plan.items[plan.len() - 2],
            Item::of(&reading::Status {
                rule: String::from("AN_EXAMPLE"),
                status: String::from("partially-cancelled"),
            })
        );
        assert_eq!(
            plan.items[plan.len() - 1],
            Item::of(&reading::Severity {
                rule: String::from("AN_EXAMPLE"),
                severity: 60,
            })
        );
    }

    /// Every life span a shipped rule gives is a whole number of its unit,
    /// which is what lets the message select a plural: a fractional one
    /// would be rounded silently, so this fails instead.
    #[test]
    fn every_shipped_span_is_whole() {
        for pack in [
            teistro_rules::shipped::nabhasas(),
            teistro_rules::shipped::arishtas(),
            teistro_rules::shipped::gandantas(),
            teistro_rules::shipped::readings(),
            teistro_rules::shipped::computed_doshas(),
            teistro_rules::shipped::computed_yogas(),
        ] {
            for rule in pack {
                for outcome in &rule.outcomes {
                    if let Outcome::LifeSpan { count, .. } = outcome {
                        assert!(
                            (count - count.round()).abs() < f64::EPSILON,
                            "{} gives {count}, which no message can pluralise",
                            rule.key
                        );
                    }
                }
            }
        }
    }

    /// Where the base locale carries a reading, it replaces the verse's
    /// cited words — once for the rule, however many statements it cites.
    #[test]
    fn a_reading_replaces_the_words_the_verse_cites() {
        let (rule, result) = present();
        let without = readings([(&rule, &result)], &NoReadings);
        let with = readings([(&rule, &result)], &Carried(&["AN_EXAMPLE"]));

        let effects = |plan: &Plan| {
            plan.items
                .iter()
                .filter(|item| item.key == "sdk.reading.effect")
                .count()
        };
        let said = |plan: &Plan| {
            plan.items
                .iter()
                .filter(|item| item.key == "sdk.reading.says")
                .count()
        };
        assert_eq!(effects(&without), 1);
        assert_eq!(said(&without), 0);
        assert_eq!(effects(&with), 0, "the reading replaced it");
        assert_eq!(said(&with), 1, "said once for the rule");
        assert_eq!(
            with.items[0],
            Item::of(&reading::Says {
                rule: String::from("AN_EXAMPLE"),
                reading: String::from("rule.AN_EXAMPLE"),
            })
        );
        assert_eq!(
            with.len(),
            without.len(),
            "one reading for one cited statement"
        );
    }

    /// A rule that states no effect at all still says its reading, which is
    /// the case that matters: every kernel rule with a reading is one of
    /// the silent ones.
    #[test]
    fn a_rule_that_says_nothing_in_words_still_says_its_reading() {
        let quiet = rule(
            r#"{
                "key": "QUIET",
                "category": "example",
                "source": { "text": "an example" },
                "conditions": [{ "type": "birth-by-day" }]
            }"#,
        );
        let chart = chart();
        let result = Evaluator::new(&chart, Readings::TEXTS).evaluate(&quiet);
        // With nothing written for it, such a rule contributes nothing.
        assert!(readings([(&quiet, &result)], &NoReadings).is_empty());
        // With a reading, it says the one thing it has to say.
        let with = readings([(&quiet, &result)], &Carried(&["QUIET"]));
        assert_eq!(with.len(), 1);
        assert_eq!(with.items[0].key, "sdk.reading.says");
    }

    /// The vocabulary is asked by the rule's own key and answers for that
    /// rule alone: nothing is matched by resemblance
    /// (`03-design/interpretation-records.md` §4).
    #[test]
    fn a_reading_is_not_shared_with_a_rule_of_a_similar_name() {
        let (rule, result) = present();
        let plan = readings([(&rule, &result)], &Carried(&["AN_EXAMPLE_OF_SOMETHING"]));
        assert!(
            plan.items.iter().all(|item| item.key != "sdk.reading.says"),
            "{plan:?}"
        );
    }

    #[test]
    fn a_reading_key_is_the_rule_under_its_open_kind() {
        assert_eq!(crate::reading_key("RUCHAKA"), "rule.RUCHAKA");
    }
}
