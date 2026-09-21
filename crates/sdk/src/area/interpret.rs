//! `sdk.interpret`: what a reading says, as a narrative plan.

use teistro_core::error::Error;
use teistro_interpret::{Plan, placements};
use teistro_serial::document::Document;

use crate::context::Context;
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
}
