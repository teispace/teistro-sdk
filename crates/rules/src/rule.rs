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

use std::collections::{BTreeMap, BTreeSet};

use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Varga};

use crate::language::{Body, Condition, House, Source};
use crate::timing::Timing;

/// Where a rule applies.
///
/// ```
/// # use teistro_rules::Rule;
/// #
/// # let rule = |body: &str| {
/// #     serde_json::from_str::<Rule>(&format!(r#"{{"key": "EXAMPLE", "category": "example", "source": {{"text": "an example"}}, {body}}}"#))
/// # };
///
/// use teistro_rules::Scope;
///
/// // The engine's `applicableTo`; a rule that says nothing is natal.
/// let milan = rule(r#""applicableTo": "milan", "conditions": [{"type": "birth-by-day"}]"#)?;
/// assert_eq!(milan.scope, Scope::Milan);
/// assert_eq!(rule(r#""conditions": [{"type": "birth-by-day"}]"#)?.scope, Scope::Natal);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
///
/// ```
/// # use teistro_rules::Rule;
/// #
/// # let rule = |body: &str| {
/// #     serde_json::from_str::<Rule>(&format!(r#"{{"key": "EXAMPLE", "category": "example", "source": {{"text": "an example"}}, {body}}}"#))
/// # };
///
/// use teistro_rules::Body;
///
/// let manglik = rule(r#""referenceConditions": [
///     {"reference": "lagna", "label": "from the lagna", "conditions": [{"type": "planet-in-house", "planet": "MARS", "houses": [1, 4, 7, 8, 12]}]},
///     {"reference": "moon", "label": "from the Moon", "weight": 2, "conditions": [{"type": "planet-in-house-from", "planet": "MARS", "reference": "MOON", "houses": [1, 4, 7, 8, 12]}]}
/// ]"#)?;
/// assert_eq!(manglik.groups[0].reference, Body::Lagna);
/// assert_eq!((manglik.groups[0].weight, manglik.groups[1].weight), (1, 2));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "RuleGroup"))]
#[serde(deny_unknown_fields)]
pub struct Group {
    /// The point its houses are counted from, which a house-weighted severity
    /// reads: a body's key, or the engine's `lagna`, `moon` or `venus`.
    #[serde(with = "reference_point")]
    #[cfg_attr(feature = "schema", schemars(with = "String"))]
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
///
/// ```
/// use teistro_rules::{Cancellation, Condition};
///
/// let burnt: Condition = serde_json::from_str(r#"{"type": "planet-combust", "planet": "MARS"}"#)?;
/// let cancellation = Cancellation { label: Some(String::from("Mars burnt")), ..Cancellation::from(burnt) };
/// // The label is written beside the condition's own parts.
/// let written = serde_json::to_value(&cancellation)?;
/// assert_eq!((written["type"].as_str(), written["label"].as_str()), (Some("planet-combust"), Some("Mars burnt")));
/// # Ok::<(), serde_json::Error>(())
/// ```
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

/// A condition as the rule language writes it, with an optional `label`
/// laid beside its parts; the condition's own reader checks the rest.
#[cfg(feature = "schema")]
impl schemars::JsonSchema for Cancellation {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("Cancellation")
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "object",
            "properties": { "label": { "type": "string" } },
            "description": "A condition, as a rule's `when` is written, and its `label`.",
        })
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
///
/// ```
/// use teistro_rules::Severity;
///
/// let fixed: Severity = serde_json::from_str(r#"{"type": "fixed", "value": 60}"#)?;
/// assert_eq!(fixed, Severity::Fixed { value: 60 });
/// // A house-weighted severity reads Mars unless it names another.
/// let weighted: Severity = serde_json::from_str(r#"{"type": "house-weighted", "weights": {"7": 100}, "default": 50}"#)?;
/// assert!(matches!(weighted, Severity::HouseWeighted { default: 50, .. }));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
        #[cfg_attr(feature = "schema", schemars(with = "BTreeMap<String, u16>"))]
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

/// What a rule says happens when it holds, beyond being present, in the ways
/// its verse says it. The texts count a life in two ways — a span (Phaladeepika
/// ch. 13 v. 6's bands, and Saravali ch. 10's verse-by-verse spans) and a class
/// of life (BPHS ch. 43 vv. 52 to 54: short, medium, long and the rest) — and
/// everywhere else they say what follows in words: a dwigraha verse gives a
/// trade and a temper, not a number. A consumer therefore reads a span or a
/// class where a text gives one and the verse's own statement where it does
/// not, and the SDK invents none of them.
///
/// ```
/// use teistro_rules::{Outcome, Unit};
///
/// let span: Outcome = serde_json::from_str(r#"{"type": "life-span", "count": 32, "unit": "years"}"#)?;
/// assert_eq!(span, Outcome::LifeSpan { count: 32.0, unit: Unit::Years });
/// assert_eq!(span.days(), Some(32.0 * 365.25));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "RuleOutcome"))]
#[serde(
    tag = "type",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Outcome {
    /// The life span the rule gives, in the unit its verse uses.
    LifeSpan {
        /// How many.
        count: f64,
        /// Of what.
        unit: Unit,
    },
    /// The class of life the verse gives, which carries its own span.
    LifeClass {
        /// Which class.
        class: LifeClass,
    },
    /// What the verse says follows, in words: the SDK's own short statement of
    /// the reading, not the translator's prose.
    Effect {
        /// The statement.
        text: String,
    },
}

/// The unit a span is counted in, as the verse counts it.
///
/// ```
/// use teistro_rules::Unit;
///
/// assert_eq!(serde_json::to_string(&Unit::Months)?, r#""months""#);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum Unit {
    /// Days.
    Days,
    /// Months.
    Months,
    /// Years.
    Years,
}

/// A class of life, BPHS ch. 43 vv. 52 to 54, from the shortest up.
///
/// ```
/// use teistro_rules::LifeClass;
///
/// assert_eq!(LifeClass::Medium.years(), Some(64));
/// // Jupiter raises a class and Saturn lowers one (vv. 49 to 50).
/// assert_eq!(LifeClass::Medium.raised(), LifeClass::Long);
/// assert_eq!(LifeClass::Balarishta.lowered(), LifeClass::Balarishta);
/// assert_eq!(LifeClass::Unlimited.years(), None);
/// # Ok::<(), ()>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum LifeClass {
    /// Death in infancy: eight years.
    Balarishta,
    /// An evil from a yoga: twenty years.
    Yogarishta,
    /// Short life, alpayu: thirty-two years.
    Short,
    /// Medium life, madhyayu: sixty-four years.
    Medium,
    /// Long life, purnayu: a hundred and twenty years.
    Long,
    /// A super-natural life: a thousand years.
    Divine,
    /// Illimitable longevity, amitayu, which the verse gives no number.
    Unlimited,
}

impl LifeClass {
    /// Its key, as a rule writes it and as a message selects on it.
    ///
    /// One list, not two: a test holds it against what serde writes, so a
    /// class renamed in one place is refused rather than half-renamed.
    ///
    /// ```
    /// use teistro_rules::LifeClass;
    ///
    /// assert_eq!(LifeClass::Medium.key(), "medium");
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            LifeClass::Balarishta => "balarishta",
            LifeClass::Yogarishta => "yogarishta",
            LifeClass::Short => "short",
            LifeClass::Medium => "medium",
            LifeClass::Long => "long",
            LifeClass::Divine => "divine",
            LifeClass::Unlimited => "unlimited",
        }
    }

    /// Every class, from the shortest up.
    ///
    /// ```
    /// use teistro_rules::LifeClass;
    ///
    /// // From the shortest up, which is also their order.
    /// assert!(LifeClass::ALL.windows(2).all(|pair| pair[0] < pair[1]));
    /// assert_eq!(LifeClass::ALL[0], LifeClass::Balarishta);
    /// ```
    pub const ALL: [LifeClass; 7] = [
        LifeClass::Balarishta,
        LifeClass::Yogarishta,
        LifeClass::Short,
        LifeClass::Medium,
        LifeClass::Long,
        LifeClass::Divine,
        LifeClass::Unlimited,
    ];

    /// The span the class is given, in years; none for the illimitable.
    ///
    /// ```
    /// use teistro_rules::LifeClass;
    ///
    /// assert_eq!(LifeClass::Long.years(), Some(120));
    /// assert_eq!(LifeClass::Short.years(), Some(32));
    /// ```
    #[must_use]
    pub const fn years(self) -> Option<u16> {
        match self {
            LifeClass::Balarishta => Some(8),
            LifeClass::Yogarishta => Some(20),
            LifeClass::Short => Some(32),
            LifeClass::Medium => Some(64),
            LifeClass::Long => Some(120),
            LifeClass::Divine => Some(1000),
            LifeClass::Unlimited => None,
        }
    }

    /// The class one step longer, as Jupiter raises it (BPHS ch. 43 vv. 48 to
    /// 50); the longest stays itself.
    ///
    /// ```
    /// use teistro_rules::LifeClass;
    ///
    /// assert_eq!(LifeClass::Short.raised(), LifeClass::Medium);
    /// assert_eq!(LifeClass::Unlimited.raised(), LifeClass::Unlimited);
    /// ```
    #[must_use]
    pub fn raised(self) -> LifeClass {
        self.step(1)
    }

    /// The class one step shorter, as Saturn lowers it (vv. 47, 49 to 50); the
    /// shortest stays itself.
    ///
    /// ```
    /// use teistro_rules::LifeClass;
    ///
    /// assert_eq!(LifeClass::Long.lowered(), LifeClass::Medium);
    /// ```
    #[must_use]
    pub fn lowered(self) -> LifeClass {
        self.step(-1)
    }

    fn step(self, by: isize) -> LifeClass {
        let at = LifeClass::ALL
            .iter()
            .position(|class| *class == self)
            .unwrap_or(0);
        at.checked_add_signed(by)
            .and_then(|next| LifeClass::ALL.get(next))
            .copied()
            .unwrap_or(self)
    }
}

impl Outcome {
    /// The span in days when the outcome is a span, a month being thirty and a
    /// year 365.25; none when the verse states its effect in words or a class.
    ///
    /// ```
    /// use teistro_rules::{Outcome, Unit};
    ///
    /// // A month is thirty days.
    /// assert_eq!(Outcome::LifeSpan { count: 6.0, unit: Unit::Months }.days(), Some(180.0));
    /// assert_eq!(Outcome::Effect { text: String::from("a warrior") }.days(), None);
    /// ```
    #[must_use]
    pub fn days(&self) -> Option<f64> {
        match self {
            Outcome::LifeSpan { count, unit } => Some(
                count
                    * match unit {
                        Unit::Days => 1.0,
                        Unit::Months => 30.0,
                        Unit::Years => 365.25,
                    },
            ),
            Outcome::Effect { .. } | Outcome::LifeClass { .. } => None,
        }
    }

    /// What the verse says follows, when it says it in words.
    ///
    /// ```
    /// use teistro_rules::Outcome;
    ///
    /// let effect = Outcome::Effect { text: String::from("a warrior") };
    /// assert_eq!(effect.text(), Some("a warrior"));
    /// ```
    #[must_use]
    pub fn text(&self) -> Option<&str> {
        match self {
            Outcome::Effect { text } => Some(text),
            Outcome::LifeSpan { .. } | Outcome::LifeClass { .. } => None,
        }
    }

    /// The class of life, when the verse gives one.
    ///
    /// ```
    /// use teistro_rules::{LifeClass, Outcome};
    ///
    /// let class: Outcome = serde_json::from_str(r#"{"type": "life-class", "class": "long"}"#)?;
    /// assert_eq!(class.class(), Some(LifeClass::Long));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    #[must_use]
    pub const fn class(&self) -> Option<LifeClass> {
        match self {
            Outcome::LifeClass { class } => Some(*class),
            Outcome::LifeSpan { .. } | Outcome::Effect { .. } => None,
        }
    }
}

/// The first span among outcomes, in days.
fn life_span(outcomes: &[Outcome]) -> Option<f64> {
    outcomes.iter().find_map(Outcome::days)
}

/// The first statement in words among outcomes.
fn effect(outcomes: &[Outcome]) -> Option<&str> {
    outcomes.iter().find_map(Outcome::text)
}

/// Whether a present rule stands, after its cancellations.
///
/// ```
/// # use teistro_core::catalogue::{Dignity, Rashi};
/// # use teistro_rules::{Body, House, Placement, RuleChart};
/// #
/// # let mut chart = RuleChart {
/// #     placements: [Placement {
/// #         longitude: 15.0,
/// #         sign: Rashi::Aries,
/// #         house: House::try_new(1).unwrap(),
/// #         dignity: Dignity::Neutral,
/// #         retrograde: false,
/// #         combust: false,
/// #         karaka7: None,
/// #         karaka8: None,
/// #         navamsha: Rashi::Aries,
/// #     }; 10],
/// #     panchanga: None,
/// #     strengths: None,
/// # };
/// # let place = |chart: &mut RuleChart, body: Body, sign: Rashi| {
/// #     let at = &mut chart.placements[body.index()];
/// #     at.sign = sign;
/// #     at.longitude = f64::from(sign as u8) * 30.0 + 15.0;
/// #     at.house = House::between(Rashi::Aries, sign);
/// # };
///
/// use teistro_core::catalogue::Graha;
/// use teistro_rules::{Evaluator, NetStatus, Readings, Rule};
///
/// // Saturn in the first, Mars combust: one of two cancellations holds.
/// chart.placements[Body::Graha(Graha::Mars).index()].combust = true;
/// let rule: Rule = serde_json::from_str(r#"{
///     "key": "EXAMPLE", "category": "example", "source": {"text": "an example"},
///     "conditions": [{"type": "planet-in-house", "planet": "SATURN", "houses": [1]}],
///     "cancellations": [{"type": "planet-combust", "planet": "MARS"}, {"type": "planet-retrograde", "planet": "MARS"}],
///     "fullCancellationThreshold": 2
/// }"#)?;
/// let result = Evaluator::new(&chart, Readings::RECORDING_ENGINE).evaluate(&rule);
/// assert_eq!(result.cancellations, [0]);
/// assert_eq!(result.status, Some(NetStatus::PartiallyCancelled));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum NetStatus {
    /// No cancellation held.
    Active,
    /// Some held, fewer than the threshold.
    PartiallyCancelled,
    /// At least the threshold held.
    FullyCancelled,
}

impl NetStatus {
    /// Its key, as a result writes it and as a message selects on it.
    ///
    /// One list, not two: a test holds it against what serde writes.
    ///
    /// ```
    /// use teistro_rules::NetStatus;
    ///
    /// assert_eq!(NetStatus::FullyCancelled.key(), "fully-cancelled");
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            NetStatus::Active => "active",
            NetStatus::PartiallyCancelled => "partially-cancelled",
            NetStatus::FullyCancelled => "fully-cancelled",
        }
    }
}

/// A rule.
///
/// ```
/// # use teistro_core::catalogue::{Dignity, Rashi};
/// # use teistro_rules::{Body, House, Placement, RuleChart};
/// #
/// # let mut chart = RuleChart {
/// #     placements: [Placement {
/// #         longitude: 15.0,
/// #         sign: Rashi::Aries,
/// #         house: House::try_new(1).unwrap(),
/// #         dignity: Dignity::Neutral,
/// #         retrograde: false,
/// #         combust: false,
/// #         karaka7: None,
/// #         karaka8: None,
/// #         navamsha: Rashi::Aries,
/// #     }; 10],
/// #     panchanga: None,
/// #     strengths: None,
/// # };
/// # let place = |chart: &mut RuleChart, body: Body, sign: Rashi| {
/// #     let at = &mut chart.placements[body.index()];
/// #     at.sign = sign;
/// #     at.longitude = f64::from(sign as u8) * 30.0 + 15.0;
/// #     at.house = House::between(Rashi::Aries, sign);
/// # };
///
/// use teistro_core::catalogue::Graha;
/// use teistro_rules::{Evaluator, Readings, Rule};
///
/// let rule: Rule = serde_json::from_str(r#"{
///     "key": "GURU_IN_KENDRA", "category": "example", "source": {"text": "an example"},
///     "conditions": [{"type": "planet-in-kendra", "planet": "JUPITER"}]
/// }"#)?;
/// let evaluator = Evaluator::new(&chart, Readings::RECORDING_ENGINE);
/// assert!(evaluator.evaluate(&rule).present);
/// // Jupiter carried to Taurus, the second house: not a kendra.
/// place(&mut chart, Body::Graha(Graha::Jupiter), Rashi::Taurus);
/// assert!(!Evaluator::new(&chart, Readings::RECORDING_ENGINE).evaluate(&rule).present);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "RuleDefinition"))]
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
    /// What it says happens when it holds, in as many ways as its verse says
    /// it: a Pancha Mahapurusha verse both describes the native and counts his
    /// years, and a rule carries both.
    pub outcomes: Vec<Outcome>,
    /// The name of the code its author computes it with instead, when the
    /// language cannot say it.
    pub computed: Option<String>,
    /// In whose periods of a dasha it gives what it says.
    pub timing: Timing,
}

/// How a reference to a point is keyed in a condition's compact JSON.
const POINT_KEY: &[u8] = b"\"point\":";

impl Rule {
    /// A rule with nothing in it yet, natal, to be filled in:
    /// `Rule { conditions, ..Rule::new(key, category, source) }`.
    ///
    /// ```
    /// use teistro_rules::{Condition, Rule, Scope, Source};
    ///
    /// let by_day: Condition = serde_json::from_str(r#"{"type": "birth-by-day"}"#)?;
    /// let rule = Rule { conditions: vec![by_day], ..Rule::new("BY_DAY", "example", Source::text("an example")) };
    /// assert_eq!(rule.scope, Scope::Natal);
    /// assert!(rule.is_evaluable());
    /// # Ok::<(), serde_json::Error>(())
    /// ```
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
            outcomes: Vec::new(),
            computed: None,
            timing: Timing::Concerned,
        }
    }

    /// Whether the language can evaluate it: it has conditions or groups, and
    /// its author does not compute it in code.
    ///
    /// ```
    /// # use teistro_rules::Rule;
    /// #
    /// # let rule = |body: &str| {
    /// #     serde_json::from_str::<Rule>(&format!(r#"{{"key": "EXAMPLE", "category": "example", "source": {{"text": "an example"}}, {body}}}"#))
    /// # };
    ///
    /// // Nothing to evaluate, or computed in its author's code.
    /// assert!(!teistro_rules::Rule::new("EMPTY", "example", teistro_rules::Source::text("an example")).is_evaluable());
    /// assert!(!rule(r#""customResultKey": "KALSARPA", "conditions": [{"type": "birth-by-day"}]"#)?.is_evaluable());
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    #[must_use]
    pub fn is_evaluable(&self) -> bool {
        self.computed.is_none() && !(self.conditions.is_empty() && self.groups.is_empty())
    }

    /// The conditions it holds outright: its own, its groups' and its
    /// cancellations', each the root of its tree.
    ///
    /// ```
    /// # use teistro_rules::Rule;
    /// #
    /// # let rule = |body: &str| {
    /// #     serde_json::from_str::<Rule>(&format!(r#"{{"key": "EXAMPLE", "category": "example", "source": {{"text": "an example"}}, {body}}}"#))
    /// # };
    ///
    /// let rule = rule(r#""conditions": [{"type": "and", "conditions": [{"type": "birth-by-day"}, {"type": "planet-in-kendra", "planet": "MARS"}]}],
    ///     "cancellations": [{"type": "planet-combust", "planet": "MARS"}]"#)?;
    /// // The `and` and the cancellation; not what is inside the `and`.
    /// assert_eq!(rule.top_conditions().count(), 2);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn top_conditions(&self) -> impl Iterator<Item = &Condition> {
        self.conditions
            .iter()
            .chain(self.groups.iter().flat_map(|g| &g.conditions))
            .chain(self.cancellations.iter().map(|c| &c.condition))
    }

    /// Every condition it holds, and every one inside them.
    ///
    /// ```
    /// # use teistro_rules::Rule;
    /// #
    /// # let rule = |body: &str| {
    /// #     serde_json::from_str::<Rule>(&format!(r#"{{"key": "EXAMPLE", "category": "example", "source": {{"text": "an example"}}, {body}}}"#))
    /// # };
    ///
    /// let rule = rule(r#""conditions": [{"type": "and", "conditions": [{"type": "birth-by-day"}, {"type": "planet-in-kendra", "planet": "MARS"}]}]"#)?;
    /// assert_eq!(rule.every_condition().count(), 3);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn every_condition(&self) -> impl Iterator<Item = &Condition> {
        self.top_conditions().flat_map(Condition::walk)
    }

    /// Every rule it names by key, in the order it names them.
    ///
    /// ```
    /// # use teistro_rules::Rule;
    /// #
    /// # let rule = |body: &str| {
    /// #     serde_json::from_str::<Rule>(&format!(r#"{{"key": "EXAMPLE", "category": "example", "source": {{"text": "an example"}}, {body}}}"#))
    /// # };
    ///
    /// let rule = rule(r#""conditions": [{"type": "rule", "key": "GAJA_KESARI"}, {"type": "not", "condition": {"type": "rule", "key": "KEMADRUMA"}}]"#)?;
    /// assert_eq!(rule.references().collect::<Vec<_>>(), ["GAJA_KESARI", "KEMADRUMA"]);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn references(&self) -> impl Iterator<Item = &str> {
        self.every_condition()
            .filter_map(|condition| match condition {
                Condition::RuleHolds { key } => Some(key.as_str()),
                _ => None,
            })
    }

    /// The span of life its verse gives, in days, when one of its outcomes is
    /// a span.
    ///
    /// ```
    /// # use teistro_rules::Rule;
    /// #
    /// # let rule = |body: &str| {
    /// #     serde_json::from_str::<Rule>(&format!(r#"{{"key": "EXAMPLE", "category": "example", "source": {{"text": "an example"}}, {body}}}"#))
    /// # };
    ///
    /// let rule = rule(r#""conditions": [{"type": "birth-by-day"}], "outcomes": [{"type": "life-span", "count": 36, "unit": "years"}]"#)?;
    /// assert_eq!(rule.life_span(), Some(36.0 * 365.25));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    #[must_use]
    pub fn life_span(&self) -> Option<f64> {
        life_span(&self.outcomes)
    }

    /// What its verse says follows, in words, when one of its outcomes says it
    /// in words.
    ///
    /// ```
    /// # use teistro_rules::Rule;
    /// #
    /// # let rule = |body: &str| {
    /// #     serde_json::from_str::<Rule>(&format!(r#"{{"key": "EXAMPLE", "category": "example", "source": {{"text": "an example"}}, {body}}}"#))
    /// # };
    ///
    /// let rule = rule(r#""conditions": [{"type": "birth-by-day"}], "outcomes": [{"type": "effect", "text": "a warrior"}]"#)?;
    /// assert_eq!(rule.effect(), Some("a warrior"));
    /// assert_eq!(rule.life_span(), None);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    #[must_use]
    pub fn effect(&self) -> Option<&str> {
        effect(&self.outcomes)
    }

    /// The class of life its verse gives, when one of its outcomes is one.
    ///
    /// ```
    /// # use teistro_rules::Rule;
    /// #
    /// # let rule = |body: &str| {
    /// #     serde_json::from_str::<Rule>(&format!(r#"{{"key": "EXAMPLE", "category": "example", "source": {{"text": "an example"}}, {body}}}"#))
    /// # };
    ///
    /// use teistro_rules::LifeClass;
    ///
    /// let rule = rule(r#""conditions": [{"type": "birth-by-day"}], "outcomes": [{"type": "life-class", "class": "short"}]"#)?;
    /// assert_eq!(rule.life_class(), Some(LifeClass::Short));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    #[must_use]
    pub fn life_class(&self) -> Option<LifeClass> {
        self.outcomes.iter().find_map(Outcome::class)
    }

    /// Whether any of its conditions asks a question of strength, so a caller
    /// knows to give the chart [`Strengths`](crate::Strengths). An
    /// intervention counts: it reads a strength when the numbers of grahas do
    /// not settle it (BPHS ch. 31 v. 4).
    ///
    /// ```
    /// # use teistro_rules::Rule;
    /// #
    /// # let rule = |body: &str| {
    /// #     serde_json::from_str::<Rule>(&format!(r#"{{"key": "EXAMPLE", "category": "example", "source": {{"text": "an example"}}, {body}}}"#))
    /// # };
    ///
    /// assert!(rule(r#""conditions": [{"type": "planet-strong", "planet": "JUPITER"}]"#)?.reads_strength());
    /// assert!(!rule(r#""conditions": [{"type": "planet-in-kendra", "planet": "JUPITER"}]"#)?.reads_strength());
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    #[must_use]
    pub fn reads_strength(&self) -> bool {
        self.every_condition().any(Condition::reads_strength)
    }

    /// Whether any of its conditions names a point — an upagraha, a special
    /// lagna, a sphuta — so a caller knows to give the evaluator the chart's
    /// points.
    ///
    /// ```
    /// # use teistro_rules::Rule;
    /// #
    /// # let rule = |body: &str| {
    /// #     serde_json::from_str::<Rule>(&format!(r#"{{"key": "EXAMPLE", "category": "example", "source": {{"text": "an example"}}, {body}}}"#))
    /// # };
    ///
    /// assert!(rule(r#""conditions": [{"type": "same-sign", "of": {"point": "GULIKA"}, "as": 1}]"#)?.reads_points());
    /// assert!(!rule(r#""conditions": [{"type": "birth-by-day"}]"#)?.reads_points());
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    #[must_use]
    pub fn reads_points(&self) -> bool {
        // A reference is what its serialisation holds, and a point is written
        // `{"point": …}` wherever a sign can stand. In compact JSON an
        // unescaped `"point":` is only ever a key (a string holding the
        // quotes writes them escaped), so the text is searched, not built
        // into a tree: a rule set's worth of trees cost a request 9 ms.
        let mut text = Vec::new();
        self.top_conditions().any(|condition| {
            text.clear();
            serde_json::to_writer(&mut text, condition).is_ok()
                && text.windows(POINT_KEY.len()).any(|at| at == POINT_KEY)
        })
    }

    /// The divisional charts its conditions read, each once, in the order they
    /// are first named, so a caller knows which to compute.
    ///
    /// ```
    /// # use teistro_rules::Rule;
    /// #
    /// # let rule = |body: &str| {
    /// #     serde_json::from_str::<Rule>(&format!(r#"{{"key": "EXAMPLE", "category": "example", "source": {{"text": "an example"}}, {body}}}"#))
    /// # };
    ///
    /// use teistro_core::catalogue::Varga;
    ///
    /// // The navamsha named twice is asked for once.
    /// let rule = rule(r#""conditions": [
    ///     {"type": "in-varga", "varga": "D9", "condition": {"type": "planet-in-kendra", "planet": "MARS"}},
    ///     {"type": "in-varga", "varga": "D10", "condition": {"type": "planet-in-kendra", "planet": "SUN"}},
    ///     {"type": "in-varga", "varga": "D9", "condition": {"type": "planet-in-kendra", "planet": "VENUS"}}
    /// ]"#)?;
    /// assert_eq!(rule.vargas().collect::<Vec<_>>(), [Varga::D9, Varga::D10]);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn vargas(&self) -> impl Iterator<Item = Varga> + '_ {
        // A catalogue id is a varga's bit; the catalogue holds 21.
        let mut seen = 0_u64;
        self.every_condition()
            .filter_map(move |condition| match condition {
                Condition::InVarga { varga, .. } => {
                    let bit = 1_u64.checked_shl(u32::from(*varga as u16)).unwrap_or(0);
                    let first = seen & bit == 0 || bit == 0;
                    seen |= bit;
                    first.then_some(*varga)
                }
                _ => None,
            })
    }

    /// Whether any of its conditions reads the chart's panchanga, so a caller
    /// knows to give the chart one.
    ///
    /// ```
    /// # use teistro_rules::Rule;
    /// #
    /// # let rule = |body: &str| {
    /// #     serde_json::from_str::<Rule>(&format!(r#"{{"key": "EXAMPLE", "category": "example", "source": {{"text": "an example"}}, {body}}}"#))
    /// # };
    ///
    /// assert!(rule(r#""conditions": [{"type": "panchanga-vara", "varas": ["SHANIVARA"]}]"#)?.reads_panchanga());
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    #[must_use]
    pub fn reads_panchanga(&self) -> bool {
        self.every_condition().any(Condition::reads_panchanga)
    }

    /// Whether any of its conditions reads the sky in transit, so a caller
    /// knows to evaluate it at an instant with that instant's sky
    /// ([`Evaluator::with_transits`](crate::Evaluator::with_transits)).
    ///
    /// ```
    /// # use teistro_rules::Rule;
    /// #
    /// # let rule = |body: &str| {
    /// #     serde_json::from_str::<Rule>(&format!(r#"{{"key": "EXAMPLE", "category": "example", "source": {{"text": "an example"}}, {body}}}"#))
    /// # };
    ///
    /// let sade_sati = rule(r#""conditions": [{"type": "transit-in-house", "planet": "SATURN", "houses": [12, 1, 2], "from": "MOON"}]"#)?;
    /// assert!(sade_sati.reads_transits());
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    #[must_use]
    pub fn reads_transits(&self) -> bool {
        self.every_condition().any(Condition::reads_transits)
    }

    /// Whether any of its conditions reads what the instant falls in, so a
    /// caller knows to give the chart its moment.
    ///
    /// ```
    /// # use teistro_rules::Rule;
    /// #
    /// # let rule = |body: &str| {
    /// #     serde_json::from_str::<Rule>(&format!(r#"{{"key": "EXAMPLE", "category": "example", "source": {{"text": "an example"}}, {body}}}"#))
    /// # };
    ///
    /// assert!(rule(r#""conditions": [{"type": "birth-during-eclipse"}]"#)?.reads_moment());
    /// // The tithi's part is the panchanga's; the lagna's would be the moment's.
    /// assert!(!rule(r#""conditions": [{"type": "limb-part", "limb": "tithi", "of": 6, "parts": [2]}]"#)?.reads_moment());
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    #[must_use]
    pub fn reads_moment(&self) -> bool {
        self.every_condition().any(Condition::reads_moment)
    }

    /// The status a present rule stands at when `cancelled` of its
    /// cancellations held and it was found from `found` places.
    ///
    /// ```
    /// # use teistro_rules::Rule;
    /// #
    /// # let rule = |body: &str| {
    /// #     serde_json::from_str::<Rule>(&format!(r#"{{"key": "EXAMPLE", "category": "example", "source": {{"text": "an example"}}, {body}}}"#))
    /// # };
    ///
    /// use teistro_rules::NetStatus;
    ///
    /// // Unset, the threshold is as many places as it was found from.
    /// let rule = rule(r#""conditions": [{"type": "birth-by-day"}]"#)?;
    /// assert_eq!(rule.net_status(0, 3), NetStatus::Active);
    /// assert_eq!(rule.net_status(2, 3), NetStatus::PartiallyCancelled);
    /// assert_eq!(rule.net_status(3, 3), NetStatus::FullyCancelled);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    outcomes: Vec<Outcome>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    custom_result_key: Option<String>,
    #[serde(default, skip_serializing_if = "Timing::is_concerned")]
    timing: Timing,
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
                weights: BTreeMap::from([(House::try_new(8).map_err(|why| why.to_string())?, 80)]),
                default: 50,
            }),
            Some(other) => {
                return Err(format!(
                    "rule `{}`: `{other}` is not a severity formula the kernel knows: COUNT_TIMES_25_CAP_100 or SHANI_HOUSE_8_ESCALATION",
                    written.key
                ));
            }
        };
        let rule = Rule {
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
            outcomes: written.outcomes,
            computed: written.custom_result_key,
            timing: written.timing,
        };
        // `SELF` is the body a `for-any` binds, and means nothing outside one.
        if let Some(kind) = rule.top_conditions().find_map(Condition::unbound_self) {
            return Err(format!(
                "rule `{}`: `{kind}` names SELF with no `for-any` above it to bind one",
                rule.key
            ));
        }
        if let Some(wrong) = rule.top_conditions().find_map(Condition::malformed_part) {
            return Err(format!("rule `{}`: {wrong}", rule.key));
        }
        // A divisional chart moves signs, not degrees.
        if let Some(kind) = rule
            .top_conditions()
            .find_map(Condition::longitude_in_varga)
        {
            return Err(format!(
                "rule `{}`: `{kind}` reads a longitude, which `in-varga` does not move",
                rule.key
            ));
        }
        Ok(rule)
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
            outcomes: rule.outcomes,
            custom_result_key: rule.computed,
            timing: rule.timing,
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
                    .and_then(|house| House::try_new(house).map_err(|why| why.to_string()))
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

/// Depth-first, keeping the path so a cycle can name itself.
fn walk_references<'r>(
    rule: &'r Rule,
    by_key: &BTreeMap<&'r str, &'r Rule>,
    path: &mut Vec<&'r str>,
    done: &mut BTreeSet<&'r str>,
) -> Result<(), String> {
    if done.contains(rule.key.as_str()) {
        return Ok(());
    }
    if path.contains(&rule.key.as_str()) {
        path.push(&rule.key);
        return Err(format!(
            "rules name each other in a circle: {}",
            path.join(" → ")
        ));
    }
    path.push(&rule.key);
    for key in rule.references() {
        let named = by_key.get(key).ok_or_else(|| {
            format!(
                "rule `{}` names `{key}`, which the set does not hold",
                rule.key
            )
        })?;
        walk_references(named, by_key, path, done)?;
    }
    path.pop();
    done.insert(&rule.key);
    Ok(())
}

/// A rule written as its key, where the rule itself is the set's and a result
/// need only name it.
///
/// ```
/// use teistro_rules::{Rule, Source, key_of};
///
/// #[derive(serde::Serialize)]
/// struct Named<'a> {
///     #[serde(serialize_with = "key_of")]
///     rule: &'a Rule,
/// }
/// let rule = Rule::new("GAJA_KESARI", "yoga", Source::text("an example"));
/// assert_eq!(serde_json::to_string(&Named { rule: &rule })?, r#"{"rule":"GAJA_KESARI"}"#);
/// # Ok::<(), serde_json::Error>(())
/// ```
///
/// # Errors
///
/// The serialiser's.
#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde passes a field by reference"
)]
pub fn key_of<S: serde::Serializer>(rule: &&Rule, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&rule.key)
}

/// The rules of a set in key order, as their places in it, so a `{"type":
/// "rule"}` condition finds the rule it names by halving the set rather
/// than by reading it through: over the shipped sets that was 2,473
/// references, each a walk of 3,130 rules.
///
/// An index is of one slice. Given another, an evaluator still answers
/// right, since a key the index does not lead to is looked for in the
/// slice itself ([`Evaluator::with_indexed_rules`](crate::Evaluator::with_indexed_rules)).
///
/// ```
/// use teistro_rules::{Rule, RuleIndex};
///
/// let named = |key: &str| {
///     serde_json::from_str::<Rule>(&format!(r#"{{"key": "{key}", "category": "example", "source": {{"text": "an example"}}}}"#))
/// };
/// let rules = [named("B")?, named("A")?, named("C")?];
/// let index = RuleIndex::of(&rules);
/// assert_eq!(index.find(&rules, "A").map(|rule| rule.key.as_str()), Some("A"));
/// assert!(index.find(&rules, "D").is_none());
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuleIndex {
    /// Each rule's place in the slice, ordered by key and, among equal
    /// keys, by place, so the first of a key given twice is the one found.
    by_key: Vec<usize>,
}

impl RuleIndex {
    /// The index of a slice of rules.
    ///
    /// ```
    /// use teistro_rules::{RuleIndex, shipped};
    ///
    /// let index = RuleIndex::of(shipped::nabhasas());
    /// let first = &shipped::nabhasas()[0];
    /// assert_eq!(index.find(shipped::nabhasas(), &first.key), Some(first));
    /// ```
    #[must_use]
    pub fn of(rules: &[Rule]) -> RuleIndex {
        let mut by_key: Vec<usize> = (0..rules.len()).collect();
        by_key.sort_by(|a, b| {
            let key = |at: &usize| rules.get(*at).map(|rule| rule.key.as_str());
            key(a).cmp(&key(b))
        });
        RuleIndex { by_key }
    }

    /// The rule of `rules` keyed `key`, the first in the slice when two
    /// are, found through the index; a slice the index was not made of is
    /// read through instead, so the answer is the slice's either way.
    ///
    /// ```
    /// use teistro_rules::{RuleIndex, shipped};
    ///
    /// // An index of another set still finds a rule of this one.
    /// let index = RuleIndex::of(shipped::gandantas());
    /// let first = &shipped::nabhasas()[0];
    /// assert_eq!(index.find(shipped::nabhasas(), &first.key), Some(first));
    /// ```
    #[must_use]
    pub fn find<'r>(&self, rules: &'r [Rule], key: &str) -> Option<&'r Rule> {
        let key_at = |at: usize| rules.get(at).map(|rule| rule.key.as_str());
        if self.by_key.len() == rules.len() {
            let first = self.by_key.partition_point(|at| key_at(*at) < Some(key));
            if let Some(rule) = self.by_key.get(first).and_then(|at| rules.get(*at))
                && rule.key == key
            {
                return Some(rule);
            }
        }
        rules.iter().find(|rule| rule.key == key)
    }
}

/// Whether every rule a set names is in it and no rule reaches itself, so an
/// evaluator given the set cannot loop.
///
/// ```
/// use teistro_rules::{Rule, check_references};
///
/// let named = |key: &str, names: &str| {
///     serde_json::from_str::<Rule>(&format!(r#"{{"key": "{key}", "category": "example", "source": {{"text": "an example"}}, "conditions": [{{"type": "rule", "key": "{names}"}}]}}"#))
/// };
/// // A names B and B names A: a loop.
/// assert!(check_references(&[named("A", "B")?, named("B", "A")?]).is_err());
/// // A names a key the set lacks.
/// assert!(check_references(&[named("A", "MISSING")?]).unwrap_err().contains("MISSING"));
/// # Ok::<(), serde_json::Error>(())
/// ```
///
/// # Errors
///
/// The first rule naming a key the set lacks, or the first cycle, naming the
/// rules on it.
pub fn check_references(rules: &[Rule]) -> Result<(), String> {
    let by_key: BTreeMap<&str, &Rule> =
        rules.iter().map(|rule| (rule.key.as_str(), rule)).collect();
    let mut done = BTreeSet::new();
    for rule in rules {
        walk_references(rule, &by_key, &mut Vec::new(), &mut done)?;
    }
    Ok(())
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests unwrap and index what they read"
    )]

    use super::*;

    /// A key and what serde writes are one list, not two.
    #[test]
    fn a_key_is_what_serde_writes() {
        for class in LifeClass::ALL {
            assert_eq!(
                serde_json::to_value(class).unwrap(),
                serde_json::Value::from(class.key())
            );
        }
        for status in [
            NetStatus::Active,
            NetStatus::PartiallyCancelled,
            NetStatus::FullyCancelled,
        ] {
            assert_eq!(
                serde_json::to_value(status).unwrap(),
                serde_json::Value::from(status.key())
            );
        }
    }

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

    /// A rule writes back only what differs from a default: the natal
    /// scope, a place's weight of one and Mars as the weighed body go
    /// unwritten, and anything else is written.
    #[test]
    fn a_rule_writes_only_what_differs_from_the_defaults() {
        let written = |json: String| serde_json::to_value(rule(&json).unwrap()).unwrap();
        let group = |weight: &str| {
            format!(
                r#""referenceConditions": [{{"reference": "moon", "label": "Moon"{weight}, "conditions": [{{"type": "planet-in-kendra", "planet": "MARS"}}]}}]"#
            )
        };
        let severity = |planet: &str| {
            format!(
                r#""severityRule": {{"type": "house-weighted"{planet}, "weights": {{"7": 100}}, "default": 50}}"#
            )
        };
        let plain = written(format!("{{{HEAD}, {}, {}}}", group(""), severity("")));
        assert!(plain.get("applicableTo").is_none(), "{plain}");
        assert!(
            plain["referenceConditions"][0].get("weight").is_none(),
            "{plain}"
        );
        assert!(plain["severityRule"].get("planet").is_none(), "{plain}");
        assert!(plain.get("timing").is_none(), "{plain}");
        let set = written(format!(
            r#"{{{HEAD}, "applicableTo": "all", {}, {}}}"#,
            group(r#", "weight": 2"#),
            severity(r#", "planet": "SATURN""#)
        ));
        assert_eq!(set["applicableTo"], "all");
        assert_eq!(set["referenceConditions"][0]["weight"], 2);
        assert_eq!(set["severityRule"]["planet"], "SATURN");
    }

    /// A rule reads the transits or the moment when any condition does.
    #[test]
    fn a_rule_reads_what_any_of_its_conditions_reads() {
        let kendra = rule(&format!(
            r#"{{{HEAD}, "conditions": [{{"type": "planet-in-kendra", "planet": "SUN"}}]}}"#
        ))
        .unwrap();
        assert!(!kendra.reads_transits() && !kendra.reads_moment());
        let eclipse = rule(&format!(
            r#"{{{HEAD}, "conditions": [{{"type": "planet-in-kendra", "planet": "SUN"}}, {{"type": "birth-during-eclipse"}}]}}"#
        ))
        .unwrap();
        assert!(eclipse.reads_moment() && !eclipse.reads_transits());
        let transit = rule(&format!(
            r#"{{{HEAD}, "applicableTo": "all", "conditions": [{{"type": "transit-in-house", "planet": "SATURN", "houses": [12, 1, 2], "from": "MOON"}}]}}"#
        ))
        .unwrap();
        assert!(transit.reads_transits() && !transit.reads_moment());
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
    fn a_set_of_rules_that_names_itself_in_a_circle_or_names_nothing_is_refused() {
        let naming = |key: &str, names: &str| {
            rule(&format!(
                r#"{{"key": "{key}", "category": "c", "source": {{"text": "t"}},
                    "conditions": [{{"type": "rule", "key": "{names}"}}]}}"#
            ))
            .unwrap()
        };
        let plain = rule(&format!(
            r#"{{{HEAD}, "conditions": [{{"type": "planet-in-kendra", "planet": "SUN"}}]}}"#
        ))
        .unwrap();
        // A rule may name another that is there.
        let good = vec![naming("A", "R"), plain.clone()];
        assert_eq!(check_references(&good), Ok(()));
        assert_eq!(good[0].references().collect::<Vec<_>>(), ["R"]);

        // Naming a rule the set does not hold is refused, and says which.
        let dangling = check_references(&[naming("A", "MISSING")]).unwrap_err();
        assert!(
            dangling.contains("rule `A` names `MISSING`, which the set does not hold"),
            "{dangling}"
        );

        // So is a circle, however long, and the message walks it.
        let circle =
            check_references(&[naming("A", "B"), naming("B", "C"), naming("C", "A")]).unwrap_err();
        assert!(circle.contains("A → B → C → A"), "{circle}");
        let itself = check_references(&[naming("A", "A")]).unwrap_err();
        assert!(itself.contains("A → A"), "{itself}");
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

    #[test]
    fn a_limb_part_the_limb_does_not_have_is_refused() {
        let rule = |condition: &str| {
            serde_json::from_str::<Rule>(&format!(
                r#"{{"key": "K", "category": "panchanga", "source": {{"text": "BPHS"}},
                    "conditions": [{{"type": "not", "condition": {condition}}}]}}"#
            ))
            .map_err(|e| e.to_string())
        };
        let seventh = rule(r#"{"type": "limb-part", "limb": "tithi", "of": 6, "parts": [7]}"#);
        assert!(
            seventh
                .as_ref()
                .is_err_and(|e| e.contains("names part 7 of 6; the parts are 1 to 6")),
            "{seventh:?}"
        );
        let whole = rule(r#"{"type": "limb-part", "limb": "tithi", "of": 1, "parts": [1]}"#);
        assert!(whole.is_err_and(|e| e.contains("needs at least 2")));
        let none = rule(r#"{"type": "limb-part", "limb": "tithi", "of": 6, "parts": []}"#);
        assert!(none.is_err_and(|e| e.contains("names no part")));
        assert!(rule(r#"{"type": "limb-part", "limb": "tithi", "of": 6, "parts": [6]}"#).is_ok());
    }
}
