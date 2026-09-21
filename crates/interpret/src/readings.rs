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

use crate::Plan;

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
#[must_use]
pub fn readings<'r, I>(present: I) -> Plan
where
    I: IntoIterator<Item = (&'r Rule, &'r RuleResult)>,
{
    let mut plan = Plan::default();
    for (rule, result) in present {
        let named = || rule.key.clone();
        for outcome in &result.outcomes {
            match outcome {
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
    use crate::{Item, KEYS};

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
        let plan = readings([(&rule, &result)]);
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
        assert!(readings([(&quiet, &result)]).is_empty());
    }

    /// The status and the severity are said when the result carries them.
    #[test]
    fn a_cancelled_rule_says_so() {
        let (rule, mut result) = present();
        result.status = Some(NetStatus::PartiallyCancelled);
        result.severity = Some(60);
        let plan = readings([(&rule, &result)]);
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
}
