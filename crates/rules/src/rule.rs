//! A rule: what must hold, from which references, what cancels it and how
//! grave it is (`03-design/rules-engine.md`, "Cancellation is first-class, for
//! yogas and doshas").
//!
//! A yoga is its conditions and cancellations. A dosha adds the three things a
//! yoga rule never needed, and the kernel models them once for both:
//!
//! - **reference groups**, each a set of conditions under a label and a
//!   reference point, at least one of which must hold ("Mars in 1/2/4/7/8/12
//!   from the lagna, the Moon or Venus"); every group that holds is a place the
//!   rule was found from;
//! - **severity**, a [`Severity`] rule over where it was found from;
//! - **a cancellation threshold**: how many cancellations cancel it fully,
//!   fewer cancelling it partly ([`NetStatus`]).
//!
//! Rules are written in the recording engine's shape, so its yoga and dosha
//! rules read unchanged: `referenceConditions`, `severityRule`,
//! `fullCancellationThreshold`, `remedyKeys`, a cancellation's `label`, and a
//! `customResultKey` naming a rule the engine computes in code, which the
//! kernel reads and reports as not evaluable. The engine's two
//! `customSeverityKey` formulas are read as the severity rules they are.
//!
//! ```
//! use teistro_rules::{Rule, Severity};
//!
//! let rule: Rule = serde_json::from_str(r#"{
//!     "key": "SHANI_EXAMPLE",
//!     "category": "planetary",
//!     "source": { "text": "an example" },
//!     "referenceConditions": [{
//!         "reference": "lagna",
//!         "label": "Saturn in an afflicted house",
//!         "conditions": [{ "type": "planet-in-house", "planet": "SATURN", "houses": [1, 4, 7, 8, 10] }]
//!     }],
//!     "cancellations": [{ "type": "planet-aspects-planet", "from": "JUPITER", "target": "SATURN", "label": "Jupiter aspects Saturn" }],
//!     "severityRule": { "type": "count-based", "perOccurrence": 25, "cap": 100 },
//!     "fullCancellationThreshold": 2,
//!     "remedyKeys": ["SHANI_REMEDY"]
//! }"#)?;
//! assert!(rule.is_evaluable());
//! assert_eq!(rule.groups[0].label, "Saturn in an afflicted house");
//! assert_eq!(rule.cancellations[0].label.as_deref(), Some("Jupiter aspects Saturn"));
//! assert_eq!(rule.severity, Some(Severity::CountBased { per_occurrence: 25, cap: 100 }));
//! # Ok::<(), serde_json::Error>(())
//! ```

use std::collections::BTreeMap;

use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Graha;

use crate::language::{Body, Condition, House, Source};

/// Where a rule applies.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Scope {
    /// A birth chart: the default.
    #[default]
    Natal,
    /// Two charts matched for marriage.
    Milan,
    /// A moment chosen for an undertaking.
    Muhurta,
    /// Every one of those.
    All,
}

/// A group of conditions found from one reference point.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    /// The point its houses are counted from, which a house-weighted severity
    /// reads: a body's key, or the engine's `lagna`, `moon` or `venus`.
    #[serde(with = "reference_point")]
    pub reference: Body,
    /// What a result says it was found from.
    pub label: String,
    /// How much this place counts for a count-based severity; one unless the
    /// rule weighs it more.
    #[serde(default = "one", skip_serializing_if = "is_one")]
    pub weight: u16,
    /// What must all hold.
    pub conditions: Vec<Condition>,
}

/// A condition that cancels a present rule, with the label a result names it
/// by when it has one.
#[derive(Clone, Debug, PartialEq)]
pub struct Cancellation {
    /// Its label.
    pub label: Option<String>,
    /// The condition.
    pub condition: Condition,
}

impl From<Condition> for Cancellation {
    fn from(condition: Condition) -> Cancellation {
        Cancellation {
            label: None,
            condition,
        }
    }
}

impl Serialize for Cancellation {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut value = serde_json::to_value(&self.condition).map_err(serde::ser::Error::custom)?;
        if let (Some(label), Some(object)) = (&self.label, value.as_object_mut()) {
            object.insert(
                String::from("label"),
                serde_json::Value::from(label.as_str()),
            );
        }
        value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Cancellation {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Cancellation, D::Error> {
        let mut value = serde_json::Value::deserialize(deserializer)?;
        let label = match value.as_object_mut().and_then(|o| o.remove("label")) {
            None => None,
            Some(serde_json::Value::String(label)) => Some(label),
            Some(_) => return Err(de::Error::custom("a cancellation's `label` is text")),
        };
        let condition = Condition::deserialize(value).map_err(de::Error::custom)?;
        Ok(Cancellation { label, condition })
    }
}

/// How grave a present rule is, out of the scale its values use (the engine's
/// is 100).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Severity {
    /// Always this.
    Fixed {
        /// The severity.
        value: u16,
    },
    /// By the house a body stands in counted from each reference the rule was
    /// found from, the gravest of them; the default where no weight is given.
    HouseWeighted {
        /// The body, Mars unless the rule names another (the engine's
        /// evaluator reads Mars).
        #[serde(default = "mars", skip_serializing_if = "is_mars")]
        planet: Body,
        /// Each weighted house's severity.
        #[serde(with = "house_keys")]
        weights: BTreeMap<House, u16>,
        /// Every other house's.
        default: u16,
    },
    /// From `min` when the body is exalted or in its own sign to `max` when
    /// debilitated, halfway otherwise.
    PlanetStrengthInverse {
        /// The body.
        base_planet: Body,
        /// When debilitated.
        max: u16,
        /// When exalted or in its own sign.
        min: u16,
    },
    /// So much for each place it was found from, by its weight, up to a cap.
    CountBased {
        /// For each.
        per_occurrence: u16,
        /// At most.
        cap: u16,
    },
    /// By how far a matched pair's koota score falls short: without the pair
    /// only `full` is known, and that is the severity.
    KootShortfall {
        /// With no score at all.
        full: u16,
        /// Less for each point the score regains.
        per_point_missing: u16,
    },
}

const fn one() -> u16 {
    1
}

#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde passes a field by reference"
)]
fn is_one(weight: &u16) -> bool {
    *weight == 1
}

const fn mars() -> Body {
    Body::Graha(Graha::Mars)
}

#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde passes a field by reference"
)]
fn is_mars(body: &Body) -> bool {
    *body == mars()
}

/// Whether a present rule stands, after its cancellations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NetStatus {
    /// No cancellation held.
    Active,
    /// Some held, fewer than the threshold.
    PartiallyCancelled,
    /// At least the threshold held.
    FullyCancelled,
}

/// A rule.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "Written", into = "Written")]
pub struct Rule {
    /// Its key.
    pub key: String,
    /// Its category.
    pub category: String,
    /// Where it applies.
    pub scope: Scope,
    /// Its citation.
    pub source: Source,
    /// What must all hold.
    pub conditions: Vec<Condition>,
    /// Groups of which at least one must hold, each a place it is found from.
    pub groups: Vec<Group>,
    /// What cancels it when it is present.
    pub cancellations: Vec<Cancellation>,
    /// How grave it is, when it says.
    pub severity: Option<Severity>,
    /// How many cancellations cancel it fully; unset, as many as the places it
    /// was found from, and at least one.
    pub full_cancellation_threshold: Option<u8>,
    /// The remedies it names, by key.
    pub remedies: Vec<String>,
    /// The name of the code its author computes it with instead, when the
    /// language cannot say it.
    pub computed: Option<String>,
}

impl Rule {
    /// A rule with nothing in it yet, natal, to be filled in:
    /// `Rule { conditions, ..Rule::new(key, category, source) }`.
    #[must_use]
    pub fn new(key: impl Into<String>, category: impl Into<String>, source: Source) -> Rule {
        Rule {
            key: key.into(),
            category: category.into(),
            scope: Scope::Natal,
            source,
            conditions: Vec::new(),
            groups: Vec::new(),
            cancellations: Vec::new(),
            severity: None,
            full_cancellation_threshold: None,
            remedies: Vec::new(),
            computed: None,
        }
    }

    /// Whether the language can evaluate it: it has conditions or groups, and
    /// its author does not compute it in code.
    #[must_use]
    pub fn is_evaluable(&self) -> bool {
        self.computed.is_none() && !(self.conditions.is_empty() && self.groups.is_empty())
    }

    /// Every condition it holds: its conditions, its groups' and its
    /// cancellations', and every one inside them.
    pub fn every_condition(&self) -> impl Iterator<Item = &Condition> {
        self.conditions
            .iter()
            .chain(self.groups.iter().flat_map(|g| &g.conditions))
            .chain(self.cancellations.iter().map(|c| &c.condition))
            .flat_map(Condition::walk)
    }

    /// Whether any of its conditions reads the chart's panchanga, so a caller
    /// knows to give the chart one.
    #[must_use]
    pub fn reads_panchanga(&self) -> bool {
        self.every_condition().any(Condition::reads_panchanga)
    }

    /// The status a present rule stands at when `cancelled` of its
    /// cancellations held and it was found from `found` places.
    #[must_use]
    pub fn net_status(&self, cancelled: usize, found: usize) -> NetStatus {
        let threshold = self
            .full_cancellation_threshold
            .map_or_else(|| found.max(1), usize::from);
        if cancelled >= threshold {
            NetStatus::FullyCancelled
        } else if cancelled > 0 {
            NetStatus::PartiallyCancelled
        } else {
            NetStatus::Active
        }
    }
}

/// A rule as the recording engine writes one.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Written {
    key: String,
    category: String,
    #[serde(default, skip_serializing_if = "is_natal")]
    applicable_to: Scope,
    source: Source,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    conditions: Vec<Condition>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    reference_conditions: Vec<Group>,
    #[serde(default)]
    cancellations: Vec<Cancellation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    severity_rule: Option<Severity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    custom_severity_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    full_cancellation_threshold: Option<u8>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    remedy_keys: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    custom_result_key: Option<String>,
}

#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde passes a field by reference"
)]
fn is_natal(scope: &Scope) -> bool {
    *scope == Scope::Natal
}

impl TryFrom<Written> for Rule {
    type Error = String;

    fn try_from(written: Written) -> Result<Rule, String> {
        let severity = match written.custom_severity_key.as_deref() {
            None => written.severity_rule,
            // Both of the engine's formulas are severity rules the language has:
            // 25 for each place found from, to 100; and Saturn in the eighth 80,
            // anywhere else 50.
            Some("COUNT_TIMES_25_CAP_100") => Some(Severity::CountBased {
                per_occurrence: 25,
                cap: 100,
            }),
            Some("SHANI_HOUSE_8_ESCALATION") => Some(Severity::HouseWeighted {
                planet: Body::Graha(Graha::Saturn),
                weights: BTreeMap::from([(House::try_new(8)?, 80)]),
                default: 50,
            }),
            Some(other) => {
                return Err(format!(
                    "rule `{}`: `{other}` is not a severity formula the kernel knows: COUNT_TIMES_25_CAP_100 or SHANI_HOUSE_8_ESCALATION",
                    written.key
                ));
            }
        };
        Ok(Rule {
            key: written.key,
            category: written.category,
            scope: written.applicable_to,
            source: written.source,
            conditions: written.conditions,
            groups: written.reference_conditions,
            cancellations: written.cancellations,
            severity,
            full_cancellation_threshold: written.full_cancellation_threshold,
            remedies: written.remedy_keys,
            computed: written.custom_result_key,
        })
    }
}

impl From<Rule> for Written {
    fn from(rule: Rule) -> Written {
        Written {
            key: rule.key,
            category: rule.category,
            applicable_to: rule.scope,
            source: rule.source,
            conditions: rule.conditions,
            reference_conditions: rule.groups,
            cancellations: rule.cancellations,
            severity_rule: rule.severity,
            custom_severity_key: None,
            full_cancellation_threshold: rule.full_cancellation_threshold,
            remedy_keys: rule.remedies,
            custom_result_key: rule.computed,
        }
    }
}

/// A map keyed by house, whose keys JSON writes as strings: read as a number
/// or the string of one.
mod house_keys {
    use std::collections::BTreeMap;

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use crate::language::House;

    pub(super) fn serialize<S: Serializer>(
        weights: &BTreeMap<House, u16>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        weights.serialize(serializer)
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<BTreeMap<House, u16>, D::Error> {
        #[derive(Deserialize, PartialEq, Eq, PartialOrd, Ord)]
        #[serde(untagged)]
        enum Key {
            Number(u8),
            Text(String),
        }
        BTreeMap::<Key, u16>::deserialize(deserializer)?
            .into_iter()
            .map(|(key, weight)| {
                let number = match key {
                    Key::Number(n) => Ok(n),
                    Key::Text(text) => text
                        .parse::<u8>()
                        .map_err(|_| format!("`{text}` is not a house number")),
                };
                number
                    .and_then(House::try_new)
                    .map(|house| (house, weight))
                    .map_err(serde::de::Error::custom)
            })
            .collect()
    }
}

/// A group's reference point: a body's key, or the engine's lowercase `lagna`,
/// `moon` and `venus`.
mod reference_point {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use crate::language::Body;

    #[allow(
        clippy::trivially_copy_pass_by_ref,
        reason = "serde passes a field by reference"
    )]
    pub(super) fn serialize<S: Serializer>(body: &Body, serializer: S) -> Result<S::Ok, S::Error> {
        body.serialize(serializer)
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Body, D::Error> {
        let key = String::deserialize(deserializer)?;
        Body::from_key(&key.to_ascii_uppercase()).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests unwrap and index what they read"
    )]

    use super::*;

    fn rule(json: &str) -> Result<Rule, serde_json::Error> {
        serde_json::from_str(json)
    }

    const HEAD: &str = r#""key": "R", "category": "c", "source": {"text": "t"}"#;

    #[test]
    fn a_rule_reads_the_engine_s_shape_and_writes_it_back() {
        let written = format!(
            r#"{{{HEAD}, "applicableTo": "all",
                "referenceConditions": [{{"reference": "moon", "label": "Moon", "conditions": [{{"type": "planet-in-kendra", "planet": "MARS"}}]}}],
                "cancellations": [{{"type": "planet-combust", "planet": "MARS", "label": "Mars burnt"}}, {{"type": "planet-retrograde", "planet": "MARS"}}],
                "severityRule": {{"type": "house-weighted", "weights": {{"7": 100, "8": 90}}, "default": 50}},
                "fullCancellationThreshold": 2, "remedyKeys": ["A"]}}"#
        );
        let parsed = rule(&written).unwrap();
        assert_eq!(parsed.scope, Scope::All);
        assert_eq!(parsed.groups[0].reference, Body::Graha(Graha::Moon));
        assert_eq!(parsed.cancellations[0].label.as_deref(), Some("Mars burnt"));
        assert_eq!(parsed.cancellations[1].label, None);
        let Some(Severity::HouseWeighted {
            planet,
            weights,
            default,
        }) = &parsed.severity
        else {
            unreachable!("house-weighted")
        };
        assert_eq!((*planet, weights.len(), *default), (mars(), 2, 50));
        assert!(parsed.is_evaluable());
        let back: Rule = serde_json::from_value(serde_json::to_value(&parsed).unwrap()).unwrap();
        assert_eq!(back, parsed);
    }

    #[test]
    fn the_engine_s_severity_formulas_read_as_severity_rules_and_others_are_refused() {
        let custom = |key: &str| {
            rule(&format!(
                r#"{{{HEAD}, "conditions": [{{"type": "planet-in-kendra", "planet": "SUN"}}], "severityRule": {{"type": "fixed", "value": 1}}, "customSeverityKey": "{key}"}}"#
            ))
        };
        assert_eq!(
            custom("COUNT_TIMES_25_CAP_100").unwrap().severity,
            Some(Severity::CountBased {
                per_occurrence: 25,
                cap: 100
            })
        );
        let shani = custom("SHANI_HOUSE_8_ESCALATION").unwrap().severity;
        assert!(matches!(
            shani,
            Some(Severity::HouseWeighted {
                planet: Body::Graha(Graha::Saturn),
                default: 50,
                ..
            })
        ));
        let refused = custom("SOMETHING_ELSE").unwrap_err().to_string();
        assert!(
            refused.contains("`SOMETHING_ELSE` is not a severity formula"),
            "{refused}"
        );
    }

    #[test]
    fn a_computed_or_empty_rule_is_not_evaluable_and_malformed_parts_are_refused() {
        let computed = rule(&format!(r#"{{{HEAD}, "customResultKey": "KALSARPA"}}"#)).unwrap();
        assert!(!computed.is_evaluable());
        assert!(
            !rule(&format!(r#"{{{HEAD}, "conditions": []}}"#))
                .unwrap()
                .is_evaluable()
        );
        for (json, reason) in [
            (format!(r#"{{{HEAD}, "extra": 1}}"#), "extra"),
            (
                format!(r#"{{{HEAD}, "applicableTo": "transit"}}"#),
                "transit",
            ),
            (
                format!(
                    r#"{{{HEAD}, "cancellations": [{{"type": "planet-combust", "planet": "SUN", "label": 3}}]}}"#
                ),
                "label",
            ),
            (
                format!(
                    r#"{{{HEAD}, "referenceConditions": [{{"reference": "mercury-ish", "label": "x", "conditions": []}}]}}"#
                ),
                "mercury-ish",
            ),
            (
                format!(r#"{{{HEAD}, "severityRule": {{"type": "fixed"}}}}"#),
                "value",
            ),
        ] {
            let error = rule(&json).unwrap_err().to_string();
            assert!(error.to_lowercase().contains(reason), "{json}: {error}");
        }
    }

    #[test]
    fn the_net_status_counts_cancellations_against_the_threshold() {
        let mut r = rule(&format!(
            r#"{{{HEAD}, "conditions": [{{"type": "planet-in-kendra", "planet": "SUN"}}]}}"#
        ))
        .unwrap();
        assert_eq!(r.net_status(0, 1), NetStatus::Active);
        assert_eq!(r.net_status(1, 1), NetStatus::FullyCancelled);
        // Unset, the threshold is the number of places found from.
        assert_eq!(r.net_status(1, 3), NetStatus::PartiallyCancelled);
        assert_eq!(r.net_status(3, 3), NetStatus::FullyCancelled);
        r.full_cancellation_threshold = Some(3);
        assert_eq!(r.net_status(2, 1), NetStatus::PartiallyCancelled);
    }
}
