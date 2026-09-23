//! What a consumer asks a chart reading to answer by rule
//! (`03-design/rules-at-the-boundary.md`).
//!
//! A [`RuleRequest`] names the kernel's shipped sets and a consumer's own
//! rules, the readings to evaluate them under, and whether to add the house
//! and longevity readings. It is the record the C boundary's `rules_json`
//! carries, so Rust and every binding read one type. [`RuleRequest::rule_set`]
//! validates it into a [`RuleSet`], which a reading evaluates.

use serde::{Deserialize, Serialize};
use teistro_core::error::Error;
use teistro_rules::longevity::{Ayurdaya, Marakas, ThreePairs};
use teistro_rules::{HouseReading, Readings, Rule, RuleResult, check_references, shipped};

/// A set of rules the kernel ships.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ShippedRules {
    /// The recording engine's doshas the kernel computes.
    Doshas,
    /// The recording engine's yogas the kernel computes.
    Yogas,
    /// The gandantas of BPHS ch. 92.
    Gandantas,
    /// The evils at birth and their cancellations.
    Arishtas,
    /// The generated readings of a graha, a pair and a rising part.
    Readings,
    /// The yogas, doshas and classes of life written from the texts.
    Nabhasas,
}

impl ShippedRules {
    /// Every shipped set.
    pub const ALL: [ShippedRules; 6] = [
        ShippedRules::Doshas,
        ShippedRules::Yogas,
        ShippedRules::Gandantas,
        ShippedRules::Arishtas,
        ShippedRules::Readings,
        ShippedRules::Nabhasas,
    ];

    /// Its rules.
    #[must_use]
    pub fn rules(self) -> &'static [Rule] {
        match self {
            ShippedRules::Doshas => shipped::computed_doshas(),
            ShippedRules::Yogas => shipped::computed_yogas(),
            ShippedRules::Gandantas => shipped::gandantas(),
            ShippedRules::Arishtas => shipped::arishtas(),
            ShippedRules::Readings => shipped::readings(),
            ShippedRules::Nabhasas => shipped::nabhasas(),
        }
    }
}

/// The two named readings a request evaluates rules under.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuleReadings {
    /// The texts' readings wherever a text settles one.
    #[default]
    Texts,
    /// The recording engine's.
    RecordingEngine,
}

impl RuleReadings {
    /// The kernel's readings.
    #[must_use]
    pub const fn readings(self) -> Readings {
        match self {
            RuleReadings::Texts => Readings::TEXTS,
            RuleReadings::RecordingEngine => Readings::RECORDING_ENGINE,
        }
    }
}

/// What a chart reading is asked to answer by rule.
///
/// ```
/// use teistro::{RuleRequest, ShippedRules};
///
/// let request = RuleRequest::from_json(r#"{"shipped": ["nabhasas"], "houses": true}"#)?;
/// assert_eq!(request, RuleRequest::shipped([ShippedRules::Nabhasas]).with_houses());
/// let set = request.rule_set()?;
/// assert_eq!(set.rules(), ShippedRules::Nabhasas.rules());
/// assert!(set.houses() && !set.longevity());
///
/// let wrong = RuleRequest::from_json(r#"{"rules": [{"key": "X", "category": "raja"}]}"#).unwrap_err();
/// assert_eq!(wrong.field(), Some("rules[0]"));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct RuleRequest {
    /// The shipped sets to evaluate.
    pub shipped: Vec<ShippedRules>,
    /// A consumer's own rules, which may name shipped rules by key.
    pub rules: Vec<Rule>,
    /// The readings to evaluate under.
    pub readings: RuleReadings,
    /// Whether to add the twelve house readings.
    pub houses: bool,
    /// Whether to add the three pairs, the three spans and the marakas.
    pub longevity: bool,
}

impl RuleRequest {
    /// A request for shipped sets.
    #[must_use]
    pub fn shipped(sets: impl IntoIterator<Item = ShippedRules>) -> RuleRequest {
        RuleRequest {
            shipped: sets.into_iter().collect(),
            ..RuleRequest::default()
        }
    }

    /// The same request with a consumer's own rules added.
    #[must_use]
    pub fn with_rules(mut self, rules: impl IntoIterator<Item = Rule>) -> RuleRequest {
        self.rules.extend(rules);
        self
    }

    /// The same request under other readings.
    #[must_use]
    pub const fn with_readings(mut self, readings: RuleReadings) -> RuleRequest {
        self.readings = readings;
        self
    }

    /// The same request with the house readings.
    #[must_use]
    pub const fn with_houses(mut self) -> RuleRequest {
        self.houses = true;
        self
    }

    /// The same request with the longevity readings.
    #[must_use]
    pub const fn with_longevity(mut self) -> RuleRequest {
        self.longevity = true;
        self
    }

    /// A request read from JSON. A value that does not read is refused by
    /// where it stands — `rules[3].when`, `houses` — with what was wrong.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for JSON that is not a request, naming the field.
    pub fn from_json(text: &str) -> Result<RuleRequest, Error> {
        let mut value: serde_json::Value = serde_json::from_str(text)
            .map_err(|err| Error::invalid_arg(format!("the rule request is not JSON: {err}")))?;
        // Each rule is read on its own, so a refusal names which one.
        let rules = match value
            .as_object_mut()
            .and_then(|fields| fields.remove("rules"))
        {
            None => Vec::new(),
            Some(serde_json::Value::Array(rules)) => rules
                .into_iter()
                .enumerate()
                .map(|(at, rule)| {
                    teistro_core::strict::deserialize::<Rule>(&rule, &format!("rules[{at}]"))
                })
                .collect::<Result<Vec<_>, _>>()?,
            Some(_) => {
                return Err(
                    Error::invalid_arg("`rules` is not an array of rules").with_field("rules")
                );
            }
        };
        // Named from the request's own root, which its caller prefixes.
        let request: RuleRequest =
            teistro_core::strict::deserialize(&value, "").map_err(|err| {
                err.with_hint(
                    "an object of `shipped`, `rules`, `readings`, `houses` and `longevity`",
                )
            })?;
        Ok(request.with_rules(rules))
    }

    /// The request validated into a set: the shipped sets and the consumer's
    /// rules together, each key once, every key a rule names in the set and
    /// no rule reaching itself.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for a key given twice or a reference the set cannot
    /// resolve, naming it.
    pub fn rule_set(&self) -> Result<RuleSet, Error> {
        let mut rules: Vec<Rule> = self
            .shipped
            .iter()
            .flat_map(|set| set.rules().iter().cloned())
            .collect();
        rules.extend(self.rules.iter().cloned());
        let mut keys = std::collections::BTreeSet::new();
        for rule in &rules {
            if !keys.insert(rule.key.as_str()) {
                return Err(Error::invalid_arg(format!(
                    "the rule `{}` is given twice, once in a shipped set or twice among the rules",
                    rule.key
                ))
                .with_field("rules"));
            }
        }
        check_references(&rules)
            .map_err(|reason| Error::invalid_arg(reason).with_field("rules"))?;
        Ok(RuleSet {
            rules,
            readings: self.readings,
            houses: self.houses,
            longevity: self.longevity,
        })
    }
}

/// A validated set of rules with what else a reading should answer.
#[derive(Clone, Debug, PartialEq)]
pub struct RuleSet {
    rules: Vec<Rule>,
    readings: RuleReadings,
    houses: bool,
    longevity: bool,
}

impl RuleSet {
    /// Its rules, the shipped sets' first in the order asked, then the
    /// consumer's.
    #[must_use]
    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    /// The readings it is evaluated under.
    #[must_use]
    pub const fn readings(&self) -> RuleReadings {
        self.readings
    }

    /// Whether the house readings are asked for.
    #[must_use]
    pub const fn houses(&self) -> bool {
        self.houses
    }

    /// Whether the longevity readings are asked for.
    #[must_use]
    pub const fn longevity(&self) -> bool {
        self.longevity
    }
}

/// A rule that held on a chart, and what it answered.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Present<'r> {
    /// Which rule.
    #[serde(serialize_with = "teistro_rules::key_of")]
    pub rule: &'r Rule,
    /// What it answered.
    pub result: RuleResult,
}

/// The longevity readings of a chart (BPHS chs. 43 and 44).
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Longevity {
    /// The three pairs, when the chart carried its hora lagna.
    pub three_pairs: Option<ThreePairs>,
    /// The three spans.
    pub ayurdaya: Ayurdaya,
    /// The marakas.
    pub marakas: Marakas,
}

/// What a chart answers by rule.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RulesReading<'r> {
    /// Every rule of the set that held, in the set's order.
    pub present: Vec<Present<'r>>,
    /// The twelve house readings, when asked for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub houses: Option<Vec<HouseReading<'r>>>,
    /// The longevity readings, when asked for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longevity: Option<Longevity>,
    /// The inputs a rule named that this chart could not have — `points` for
    /// a birth with no sunrise — every rule reading one having answered false.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unreadable: Vec<&'static str>,
}
