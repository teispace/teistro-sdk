//! When a rule gives what it says: in whose periods of a dasha.
//!
//! The texts time a combination by the periods of what formed it. BPHS ch. 31
//! says an intervention's effects "will be derived in the Dasha periods of the
//! Rashi or planet concerned", ch. 33 vv. 94 to 99 say the same of Kemadruma,
//! and ch. 41 v. 16 gives wealth "during their Dasha periods" to the lords of
//! the ninth and fifth and the grahas joining them. Ch. 35 v. 50 is the other
//! kind: the Nabhasa yogas are "felt throughout, in all the Dasha periods".
//!
//! So timing is a property of a rule, [`Timing`], and delivery is a relation
//! between a present result and the periods running at an instant, which
//! [`Evaluator::delivery`](crate::Evaluator::delivery) answers. The kernel does
//! not compute a dasha: it takes the running periods as [`Running`], each a
//! lord and, in a sign-based dasha, a sign, which any dasha a caller holds can
//! give it.
//!
//! ```
//! use teistro_core::catalogue::{Graha, Rashi};
//! use teistro_rules::{Levels, Running, Timing};
//!
//! let chain = [
//!     Running::graha(Graha::Jupiter),
//!     Running::sign(Rashi::Leo, Graha::Sun),
//! ];
//! assert_eq!(Timing::default(), Timing::Concerned);
//! assert_eq!(Levels::all(chain.len()).iter().collect::<Vec<_>>(), [0, 1]);
//! ```

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi};

use crate::eval::{Evaluator, RuleResult};
use crate::language::Body;
use crate::rule::Rule;

/// In whose periods a rule gives what it says.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Timing {
    /// In the periods of the grahas that formed it, and in a sign-based
    /// dasha of the signs they stand in: "the Rashi or planet concerned"
    /// (BPHS ch. 31, ch. 33 vv. 94 to 99). What every rule does unless its
    /// verse says otherwise.
    #[default]
    Concerned,
    /// In every period: the Nabhasa yogas' effects "will be felt throughout,
    /// in all the Dasha periods" (BPHS ch. 35 v. 50).
    Throughout,
}

impl Timing {
    /// Whether it is the default, which a written rule leaves unsaid.
    #[allow(
        clippy::trivially_copy_pass_by_ref,
        reason = "serde passes a field by reference"
    )]
    pub(crate) fn is_concerned(&self) -> bool {
        *self == Timing::Concerned
    }
}

/// One period running at an instant, as the rules read it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Running {
    /// Its lord.
    pub lord: Graha,
    /// The sign it is the period of, in a sign-based dasha; nothing in a
    /// nakshatra-seeded one, whose periods are their lords'.
    pub sign: Option<Rashi>,
}

impl Running {
    /// A graha's period.
    #[must_use]
    pub const fn graha(lord: Graha) -> Running {
        Running { lord, sign: None }
    }

    /// A sign's period, with the lord its dasha gives it.
    #[must_use]
    pub const fn sign(sign: Rashi, lord: Graha) -> Running {
        Running {
            lord,
            sign: Some(sign),
        }
    }
}

/// The levels of a running chain, 0 the mahadasha, whose periods deliver a
/// result. A chain is at most eight levels deep.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Levels(u8);

impl Levels {
    /// The deepest chain it can describe.
    pub const MAX: usize = 8;

    /// No level.
    pub const NONE: Levels = Levels(0);

    /// Every level of a chain `len` deep.
    #[must_use]
    pub fn all(len: usize) -> Levels {
        (0..len.min(Levels::MAX)).fold(Levels::NONE, Levels::with)
    }

    /// The same levels and `level`, when it is under [`Levels::MAX`].
    #[must_use]
    pub fn with(self, level: usize) -> Levels {
        match u32::try_from(level)
            .ok()
            .and_then(|at| 1_u8.checked_shl(at))
        {
            Some(bit) => Levels(self.0 | bit),
            None => self,
        }
    }

    /// Whether `level` delivers.
    #[must_use]
    pub fn contains(self, level: usize) -> bool {
        self.with(level) == self && level < Levels::MAX
    }

    /// Whether none does.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The levels that deliver, from the mahadasha down.
    pub fn iter(self) -> impl Iterator<Item = usize> {
        (0..Levels::MAX).filter(move |level| self.contains(*level))
    }

    /// The highest level that delivers, the mahadasha being the highest.
    #[must_use]
    pub fn highest(self) -> Option<usize> {
        self.iter().next()
    }
}

impl Evaluator<'_> {
    /// Which of the running periods deliver a result of `rule` on this chart,
    /// by level from the mahadasha down: none when the result is not present;
    /// every level when the rule gives its effects throughout; otherwise each
    /// level whose graha is one of the result's participants, or, in a
    /// sign-based dasha, whose sign one of them stands in. A sign's period is
    /// matched by its sign and not by the lord its dasha gives it, the verse
    /// naming "the Rashi or planet concerned".
    ///
    /// The periods are read from the mahadasha down, as far as
    /// [`Levels::MAX`]; a caller passes whatever dasha it holds.
    #[must_use]
    pub fn delivery(
        &self,
        rule: &Rule,
        result: &RuleResult,
        running: impl IntoIterator<Item = Running>,
    ) -> Levels {
        if !result.present {
            return Levels::NONE;
        }
        let chart = self.chart();
        running
            .into_iter()
            .take(Levels::MAX)
            .enumerate()
            .filter(|(_, period)| match (rule.timing, period.sign) {
                (Timing::Throughout, _) => true,
                (Timing::Concerned, Some(sign)) => result
                    .participants
                    .iter()
                    .any(|body| chart.placement(body).sign == sign),
                (Timing::Concerned, None) => result
                    .participants
                    .iter()
                    .any(|body| body == Body::Graha(period.lord)),
            })
            .fold(Levels::NONE, |levels, (level, _)| levels.with(level))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_hold_what_was_added_and_nothing_past_the_deepest() {
        let levels = Levels::NONE.with(0).with(3).with(Levels::MAX);
        assert_eq!(levels.iter().collect::<Vec<_>>(), [0, 3]);
        assert_eq!(levels.highest(), Some(0));
        assert!(!levels.contains(Levels::MAX));
        assert!(Levels::NONE.is_empty() && Levels::NONE.highest().is_none());
        assert_eq!(Levels::all(20), Levels::all(Levels::MAX));
    }

    #[test]
    fn a_timing_is_written_as_the_verse_names_it() {
        assert_eq!(
            serde_json::to_string(&Timing::Throughout).ok().as_deref(),
            Some(r#""throughout""#)
        );
    }
}
