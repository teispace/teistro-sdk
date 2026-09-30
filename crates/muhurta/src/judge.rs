//! A window judged: every clause that held over it, what bars it, and an
//! order among windows that the caller chooses.
//!
//! The judgement is the clause list and nothing collapsed from it. What
//! the SDK adds is the two things a list alone does not say: which clauses
//! **bar** the rite under its rules (the marriage chapter's six
//! considerations), and an order ([`Ranking`]) — which is named on the
//! answer, because an order is a choice and not a fact (crux C162).

use core::cmp::Ordering;

use serde::{Deserialize, Serialize};
use teistro_core::interval::Interval;

use crate::activity::ActivityRules;
use crate::clause::{Clause, ClauseKey};

/// Every clause that held over one window.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Judgement {
    /// The window.
    pub at: Interval,
    /// Every clause that held over it, each with its own interval: a day's
    /// clause keeps the span it held over, so a reader sees where it began.
    pub clauses: Vec<Clause>,
    /// The clauses among them that bar the rite, by key, in the order the
    /// rules list them.
    pub barred_by: Vec<ClauseKey>,
}

impl Judgement {
    /// Judges a window: the clauses of `held` that overlap it, and the
    /// bars the rules find among them.
    ///
    /// A clause that overlaps a window at all holds over the whole of it,
    /// because the window was cut wherever any clause changes
    /// (`window.rs`).
    #[must_use]
    pub fn of<'c>(
        at: Interval,
        held: impl IntoIterator<Item = &'c Clause>,
        rules: &ActivityRules,
    ) -> Judgement {
        let clauses: Vec<Clause> = held
            .into_iter()
            .filter(|c| c.at.overlaps(at))
            .cloned()
            .collect();
        let barred_by = rules
            .bars
            .iter()
            .copied()
            .filter(|key| clauses.iter().any(|c| c.kind.key() == *key))
            .collect();
        Judgement {
            at,
            clauses,
            barred_by,
        }
    }

    /// Whether the rules let the rite be held in it.
    #[must_use]
    pub fn open(&self) -> bool {
        self.barred_by.is_empty()
    }

    /// How many clauses count for the time.
    #[must_use]
    pub fn for_count(&self) -> usize {
        self.clauses.iter().filter(|c| c.favourable()).count()
    }

    /// How many count against it.
    #[must_use]
    pub fn against_count(&self) -> usize {
        self.clauses.len() - self.for_count()
    }
}

/// How windows are ordered (crux C162).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Ranking {
    /// Raman's rule, an excess of good and a deficiency of evil: open
    /// windows first, then the fewest clauses against, then the most for,
    /// then the earliest. No clause cancels another here — the
    /// neutralisations are reported as clauses for the time and weigh as
    /// that — because which neutralisation lifts which dosha the texts
    /// give case by case, and a table of it is not yet sourced.
    Texts,
}

impl Ranking {
    /// The order of two judgements, the better first.
    #[must_use]
    pub fn compare(self, a: &Judgement, b: &Judgement) -> Ordering {
        match self {
            Ranking::Texts => b
                .open()
                .cmp(&a.open())
                .then(a.against_count().cmp(&b.against_count()))
                .then(b.for_count().cmp(&a.for_count()))
                .then(a.at.from.get().total_cmp(&b.at.from.get())),
        }
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, reason = "tests index fixed lists")]
mod tests {
    use super::{Judgement, Ranking};
    use crate::activity::ActivityRules;
    use crate::clause::{Clause, ClauseKey, ClauseKind};
    use teistro_core::catalogue::Kaala;
    use teistro_core::interval::Interval;

    const T: f64 = 2_460_000.5;

    fn clause(kind: ClauseKind, from: f64, to: f64) -> Clause {
        Clause {
            kind,
            at: Interval::literal(T + from, T + to),
        }
    }

    #[test]
    fn a_window_holds_the_clauses_that_overlap_it_and_the_bars_among_them() {
        let rules = ActivityRules::raman_marriage();
        let held = [
            clause(ClauseKind::MarsInEighth {}, 0.0, 0.2),
            clause(ClauseKind::Abhijit {}, 0.1, 0.3),
            clause(
                ClauseKind::Kaala {
                    kaala: Kaala::RahuKaala,
                },
                0.3,
                0.4,
            ),
        ];
        let early = Judgement::of(Interval::literal(T + 0.1, T + 0.2), &held, &rules);
        assert_eq!(early.clauses.len(), 2);
        assert_eq!(early.barred_by, [ClauseKey::MarsInEighth]);
        assert!(!early.open());
        assert_eq!((early.for_count(), early.against_count()), (1, 1));
        // Touching at an end is not overlapping.
        let late = Judgement::of(Interval::literal(T + 0.3, T + 0.4), &held, &rules);
        assert_eq!(late.clauses.len(), 1);
        assert!(late.open());
    }

    #[test]
    fn the_texts_order_open_first_then_fewer_against_then_more_for() {
        let rules = ActivityRules::raman_marriage();
        let at = |from: f64| Interval::literal(T + from, T + from + 0.01);
        let rahu = ClauseKind::Kaala {
            kaala: Kaala::RahuKaala,
        };
        let barred = Judgement::of(
            at(0.0),
            &[clause(ClauseKind::MarsInEighth {}, 0.0, 1.0)],
            &rules,
        );
        let one_against = Judgement::of(at(0.1), &[clause(rahu.clone(), 0.0, 1.0)], &rules);
        let clean = Judgement::of(at(0.2), &[], &rules);
        let clean_and_good =
            Judgement::of(at(0.3), &[clause(ClauseKind::Abhijit {}, 0.0, 1.0)], &rules);
        let mut all = [barred, one_against, clean, clean_and_good];
        all.sort_by(|a, b| Ranking::Texts.compare(a, b));
        let order: Vec<f64> = all
            .iter()
            .map(|j| ((j.at.from.get() - T) * 10.0).round())
            .collect();
        assert_eq!(order, [3.0, 2.0, 1.0, 0.0]);
    }
}
