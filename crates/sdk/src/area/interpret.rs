//! `sdk.interpret`: what a reading says, as a narrative plan.

use teistro_core::error::Error;
use teistro_interpret::{Plan, placements, readings, strength};
use teistro_serial::document::Document;

use crate::context::Context;
use crate::rule_request::RulesReading;
use crate::rules_bridge::RuleInputs;

/// `sdk.interpret`: a chart's answers turned into a plan of message keys
/// and slots, which [`IntlArea`](crate::IntlArea) renders in any locale.
///
/// A plan holds no words. That is the point: one plan renders in every
/// locale the engine carries, a consumer may reorder or drop items before
/// rendering, and a golden plan tests the composer rather than the
/// translator (`03-design/interpret-composers.md`).
///
/// ```no_run
/// # use teistro::{Context, Document, Plan};
/// # fn main() -> Result<(), teistro::Error> {
/// # let sdk = Context::builder().build()?;
/// # let document: Document = unimplemented!("a reading, as `chart_reading.rs` asks for one");
/// let plan: Plan = sdk.interpret().placements(&document)?;
/// for item in &plan {
///     println!("{}", sdk.intl().render(&item.key, &item.params).text);
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct InterpretArea<'a> {
    context: &'a Context,
}

impl<'a> InterpretArea<'a> {
    pub(crate) const fn of(context: &'a Context) -> InterpretArea<'a> {
        InterpretArea { context }
    }

    /// The context this area was read off.
    #[must_use]
    pub const fn context(self) -> &'a Context {
        self.context
    }

    /// Where each of the nine grahas stands and who shares a sign.
    ///
    /// # Errors
    ///
    /// A document a rule cannot read: one without its graha states, naming
    /// the section to ask for ([`RuleInputs::of`]).
    pub fn placements(self, document: &Document) -> Result<Plan, Error> {
        Ok(placements(&RuleInputs::of(document)?.chart))
    }

    /// Each graha's Shadbala in rupas, the strongest first.
    ///
    /// # Errors
    ///
    /// A document without its Shadbala, naming the section to ask for: the
    /// six strengths are computed only when a request asks, so a composer
    /// says which knob was not turned rather than answering an empty plan.
    pub fn strength(self, document: &Document) -> Result<Plan, Error> {
        document.shadbala.as_ref().map(strength).ok_or_else(|| {
            Error::invalid_arg(
                "the document carries no Shadbala, which the strength composer says; ask for it \
                 with `ChartRequest::with_shadbala`",
            )
            .with_field("shadbala")
        })
    }

    /// What each rule the chart held says: its verse's statement, who took
    /// part, whether a cancellation moved it and how grave it is.
    ///
    /// It takes a reading rather than a document, because a reading is what
    /// carries the rules' answers — `sdk.chart().readings_with_rules` gives
    /// one — and composing from the answers rather than evaluating again is
    /// what keeps this a rename of work already done.
    #[must_use]
    pub fn readings(self, reading: &RulesReading<'_>) -> Plan {
        readings(
            reading
                .present
                .iter()
                .map(|present| (present.rule, &present.result)),
        )
    }
}
