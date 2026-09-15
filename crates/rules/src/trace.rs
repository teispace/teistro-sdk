//! How a rule's answer was reached (`03-design/rules-engine.md`, "Rules
//! classify and grade": the trace).
//!
//! [`Evaluator::explain`](crate::Evaluator::explain) answers what
//! [`Evaluator::evaluate`](crate::Evaluator::evaluate) answers and also returns
//! an [`Explanation`]: a tree of [`Step`]s, one for each condition checked, in
//! the order checked, with whether it held, the bodies it added and each
//! reference it resolved on the way, so "the lord of the 7th from the upapada"
//! shows the upapada's sign, the 7th from it and the lord found there.
//! Conditions a combinator never reached because it had already decided are
//! not in the tree, as they were never checked.
//!
//! Both calls run one evaluator, generic over a [`Recorder`]: `evaluate`'s
//! records nothing and compiles away, so the answer without a trace allocates
//! no trace, and the answer with one cannot differ from it.
//!
//! ```
//! use teistro_core::catalogue::{Dignity, Rashi};
//! use teistro_rules::{Body, Evaluator, House, Placement, Readings, Rule, RuleChart};
//!
//! // Every body at 15° Aries in the first house.
//! let placement = Placement {
//!     longitude: 15.0,
//!     sign: Rashi::Aries,
//!     house: House::try_new(1)?,
//!     dignity: Dignity::Neutral,
//!     retrograde: false,
//!     combust: false,
//!     karaka7: None,
//!     karaka8: None,
//!     navamsha: Rashi::Aries,
//! };
//! let chart = RuleChart { placements: [placement; 10] };
//! let rule: Rule = serde_json::from_str(r#"{
//!     "key": "LORD_OF_TEN_IN_A_KENDRA",
//!     "category": "example",
//!     "source": { "text": "an example" },
//!     "conditions": [{ "type": "planet-in-kendra", "planet": { "lordOf": 10 } }]
//! }"#)?;
//! let explanation = Evaluator::new(&chart, Readings::RECORDING_ENGINE).explain(&rule);
//! assert!(explanation.result.present);
//! assert_eq!(
//!     explanation.to_string(),
//!     "LORD_OF_TEN_IN_A_KENDRA: present, SATURN in house 1\n\
//!      \u{20} holds planet-in-kendra, adding SATURN\n\
//!      \u{20}   house 10 is CAPRICORN\n\
//!      \u{20}   the lord of house 10 is SATURN\n\
//!      \u{20}   the lord of house 10 stands in ARIES, house 1\n"
//! );
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use core::fmt;

use serde::{Serialize, Serializer};
use teistro_core::catalogue::Rashi;

use crate::eval::RuleResult;
use crate::language::{Body, Condition, House, Rule};
use crate::reference::{BodyRef, SignRef};

/// A reference resolved while a condition was checked.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Resolved {
    /// A reference to a body, and the body the chart gave it, if any.
    Body {
        /// The reference.
        reference: BodyRef,
        /// The body; none when, say, no graha holds the karaka.
        body: Option<Body>,
    },
    /// A reference to a sign, and the sign and house the chart gave it, if
    /// any.
    Sign {
        /// The reference.
        reference: SignRef,
        /// The sign and its house; none when a body it depends on is none.
        place: Option<(Rashi, House)>,
    },
}

/// One condition as it was checked.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Step<'c> {
    /// The condition.
    #[serde(rename = "type", serialize_with = "kind")]
    pub condition: &'c Condition,
    /// Whether it held.
    pub held: bool,
    /// The bodies it added to the rule's participants, in order.
    pub added: Vec<Body>,
    /// The references it resolved, innermost first.
    pub resolved: Vec<Resolved>,
    /// The conditions inside it that were checked, in order.
    pub steps: Vec<Step<'c>>,
}

/// A rule's answer and how it was reached.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Explanation<'c> {
    /// The rule.
    #[serde(serialize_with = "key")]
    pub rule: &'c Rule,
    /// The answer, as [`Evaluator::evaluate`](crate::Evaluator::evaluate)
    /// gives it.
    pub result: RuleResult,
    /// The rule's conditions as they were checked, stopping at the first that
    /// did not hold.
    pub conditions: Vec<Step<'c>>,
    /// The cancellations as they were checked, when the rule was present.
    pub cancellations: Vec<Step<'c>>,
}

#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde passes a field by reference"
)]
fn kind<S: Serializer>(condition: &&Condition, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(condition.kind())
}

#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde passes a field by reference"
)]
fn key<S: Serializer>(rule: &&Rule, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&rule.key)
}

/// What the evaluator tells as it checks. Not implemented outside the crate:
/// the two recorders are the evaluator's own.
pub trait Recorder<'c>: sealed::Sealed {
    /// A condition is about to be checked.
    fn enter(&mut self);
    /// A reference resolved, built only if the recorder keeps it.
    fn resolved(&mut self, resolved: impl FnOnce() -> Resolved);
    /// The condition entered last was checked, adding the bodies `added`
    /// gives, built only if the recorder keeps them.
    fn leave(&mut self, condition: &'c Condition, held: bool, added: impl FnOnce() -> Vec<Body>);
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::NoTrace {}
    impl Sealed for super::Tracer<'_> {}
}

/// The recorder that keeps nothing.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoTrace;

impl Recorder<'_> for NoTrace {
    #[inline(always)]
    fn enter(&mut self) {}

    #[inline(always)]
    fn resolved(&mut self, _: impl FnOnce() -> Resolved) {}

    #[inline(always)]
    fn leave(&mut self, _: &Condition, _: bool, _: impl FnOnce() -> Vec<Body>) {}
}

/// The recorder that builds the tree.
#[derive(Clone, Debug, Default)]
pub struct Tracer<'c> {
    /// The steps finished at each open depth, the outermost first, with the
    /// references resolved there so far.
    open: Vec<(Vec<Step<'c>>, Vec<Resolved>)>,
    finished: Vec<Step<'c>>,
}

impl<'c> Tracer<'c> {
    /// The steps recorded at the outermost depth.
    pub(crate) fn finish(self) -> Vec<Step<'c>> {
        self.finished
    }
}

impl<'c> Recorder<'c> for Tracer<'c> {
    fn enter(&mut self) {
        self.open.push((Vec::new(), Vec::new()));
    }

    fn resolved(&mut self, resolved: impl FnOnce() -> Resolved) {
        // A named body resolves to itself: nothing to explain.
        let resolved = resolved();
        if matches!(
            resolved,
            Resolved::Body {
                reference: BodyRef::Body(_),
                ..
            }
        ) {
            return;
        }
        if let Some((_, open)) = self.open.last_mut() {
            open.push(resolved);
        }
    }

    fn leave(&mut self, condition: &'c Condition, held: bool, added: impl FnOnce() -> Vec<Body>) {
        let (steps, resolved) = self.open.pop().unwrap_or_default();
        let step = Step {
            condition,
            held,
            added: added(),
            resolved,
            steps,
        };
        match self.open.last_mut() {
            Some((steps, _)) => steps.push(step),
            None => self.finished.push(step),
        }
    }
}

impl fmt::Display for Resolved {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Resolved::Body {
                reference,
                body: Some(body),
            } => {
                write!(f, "{reference} is {}", body.key())
            }
            Resolved::Body {
                reference,
                body: None,
            } => write!(f, "{reference} is no body"),
            Resolved::Sign {
                reference: reference @ SignRef::House(_),
                place: Some((sign, _)),
            } => write!(f, "{reference} is {}", sign.key()),
            Resolved::Sign {
                reference: SignRef::Of(body),
                place: Some((sign, house)),
            } => write!(f, "{body} stands in {}, house {}", sign.key(), house.get()),
            Resolved::Sign {
                reference,
                place: Some((sign, house)),
            } => write!(f, "{reference} is {}, house {}", sign.key(), house.get()),
            Resolved::Sign {
                reference,
                place: None,
            } => write!(f, "{reference} is no sign"),
        }
    }
}

impl Step<'_> {
    fn write(&self, f: &mut fmt::Formatter<'_>, depth: usize) -> fmt::Result {
        let indent = "  ".repeat(depth + 1);
        write!(
            f,
            "{indent}{} {}",
            if self.held { "holds" } else { "fails" },
            self.condition.kind()
        )?;
        if !self.added.is_empty() {
            write!(f, ", adding {}", keys(&self.added))?;
        }
        writeln!(f)?;
        for resolved in &self.resolved {
            writeln!(f, "{indent}  {resolved}")?;
        }
        self.steps
            .iter()
            .try_for_each(|step| step.write(f, depth + 1))
    }
}

fn keys(bodies: &[Body]) -> String {
    bodies
        .iter()
        .map(|b| b.key())
        .collect::<Vec<_>>()
        .join(", ")
}

impl fmt::Display for Explanation<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let result = &self.result;
        write!(f, "{}: ", self.rule.key)?;
        if result.present {
            let bodies: Vec<Body> = result.participants.iter().collect();
            write!(f, "present, {}", keys(&bodies))?;
            if !result.houses.is_empty() {
                let houses: Vec<String> =
                    result.houses.iter().map(|h| h.get().to_string()).collect();
                write!(
                    f,
                    " in house{} {}",
                    if houses.len() == 1 { "" } else { "s" },
                    houses.join(", ")
                )?;
            }
            if result.is_cancelled() {
                write!(f, ", cancelled")?;
            }
            writeln!(f)?;
        } else {
            writeln!(f, "not present")?;
        }
        self.conditions
            .iter()
            .try_for_each(|step| step.write(f, 0))?;
        if !self.cancellations.is_empty() {
            writeln!(f, "  cancellations:")?;
            self.cancellations
                .iter()
                .try_for_each(|step| step.write(f, 1))?;
        }
        Ok(())
    }
}
