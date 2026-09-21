//! What a consumer asks a chart reading to say in words
//! (`03-design/plans-at-the-boundary.md`).
//!
//! A [`PlanRequest`] names the composers to run over every chart. It is the
//! record the C boundary's `interpret_json` carries, so Rust and every
//! binding read one type, and it is deliberately a set of named members
//! rather than a bit set: a composer will want options of its own, and a bit
//! set has nowhere to put them.

use serde::{Deserialize, Serialize};
use teistro_core::error::Error;

/// Which narrative plans a chart reading is asked for.
///
/// ```
/// use teistro::PlanRequest;
///
/// let request = PlanRequest::from_json(r#"{"placements": true}"#)?;
/// assert_eq!(request, PlanRequest::default().with_placements());
/// assert!(request.asks_for_something());
///
/// // A composer that does not exist is refused rather than ignored.
/// let wrong = PlanRequest::from_json(r#"{"plcaements": true}"#).unwrap_err();
/// assert!(wrong.to_string().contains("placements"));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
// **Why a bool a composer rather than a bit set.** `struct_excessive_bools` asks
// whether a bit set was meant; this design answered that before the fourth
// composer arrived, and the answer has not changed since. A composer will want options of its own — which rules to
// read, which house — and a bit set has nowhere to put them, while `sections`
// is a bit set because its members never will
// (`03-design/plans-at-the-boundary.md` §3). The members are also the JSON
// the boundary reads, one name a composer, so they are named in two senses.
#[expect(
    clippy::struct_excessive_bools,
    reason = "a named member per composer, decided in plans-at-the-boundary.md §3"
)]
pub struct PlanRequest {
    /// Where each of the nine grahas stands and who shares a sign.
    pub placements: bool,
    /// What each rule the chart held says — which needs rules to have been
    /// asked for, since a reading composes what rules answered.
    pub readings: bool,
    /// Each graha's Shadbala in rupas, the strongest first. It reads the
    /// Shadbala section, which the request computes for it.
    pub strength: bool,
    /// The lord of each of the twelve bhavas. It reads the houses section,
    /// which the request computes for it.
    pub houses: bool,
    /// Where each graha stands to the degree, which `placements` rounds
    /// away. It reads what `placements` reads, so it costs no section.
    pub positions: bool,
}

impl PlanRequest {
    /// A request for the placements.
    #[must_use]
    pub const fn with_placements(mut self) -> PlanRequest {
        self.placements = true;
        self
    }

    /// A request for the readings. It needs a rule request beside it.
    #[must_use]
    pub const fn with_readings(mut self) -> PlanRequest {
        self.readings = true;
        self
    }

    /// A request for the strengths.
    #[must_use]
    pub const fn with_strength(mut self) -> PlanRequest {
        self.strength = true;
        self
    }

    /// A request for the houses' lords.
    #[must_use]
    pub const fn with_houses(mut self) -> PlanRequest {
        self.houses = true;
        self
    }

    /// A request for the positions, to the degree.
    #[must_use]
    pub const fn with_positions(mut self) -> PlanRequest {
        self.positions = true;
        self
    }

    /// Whether any composer was asked for, so a caller can skip the work
    /// rather than compose an empty answer.
    #[must_use]
    pub const fn asks_for_something(self) -> bool {
        self.placements || self.readings || self.strength || self.houses || self.positions
    }

    /// A request read from JSON.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for JSON that is not a request, and for a composer that
    /// does not exist — which serde names beside the ones that do, because a
    /// member silently composing nothing is the dead end a typo deserves to
    /// be caught by.
    pub fn from_json(text: &str) -> Result<PlanRequest, Error> {
        serde_json::from_str(text).map_err(|err| {
            Error::invalid_arg(format!("the plan request does not read: {err}")).with_hint(
                "an object of `placements`, `readings`, `strength`, `houses` and `positions`",
            )
        })
    }

    /// The request checked against what else was asked for.
    ///
    /// `readings` composes what rules answered, so without a rule request
    /// there is nothing for it to say — and an empty plan would tell the
    /// consumer nothing about why. A set of rules none of which hold is not
    /// this case: that composes to a plan with no items, which is an answer.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on `readings` where it is asked for without rules.
    pub fn check(self, has_rules: bool) -> Result<(), Error> {
        if self.readings && !has_rules {
            return Err(Error::invalid_arg(
                "`readings` says what the rules a chart held answer, so it needs rules to \
                 answer",
            )
            .with_field("readings")
            .with_hint("name the rules in the rule request beside this one"));
        }
        Ok(())
    }
}
