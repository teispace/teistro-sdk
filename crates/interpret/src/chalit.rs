//! Where the two house readings disagree
//! (`03-design/interpret-composers.md` §4).
//!
//! A chart places every graha **twice**: under the placement system, which
//! is what most of the tradition means by "in the seventh", and under the
//! chalit, which reads the cusps. Both are kept and neither is recomputed
//! (`03-design/chart-bhava-chalit.md`), and over the recorded corpus they
//! differ for 135 of 675 placings — a fact a reader wants and no composer
//! said.
//!
//! **It says only the grahas that differ.** Agreement is the ordinary case
//! and an item a graha would bury the disagreement in eight repetitions of
//! it. A chart whose readings agree throughout composes to nothing here,
//! which is the answer and not a failure.
//!
//! **Why it is not a line inside `placements`.** That composer reads a
//! `RuleChart`, which carries one house a graha, and the façade's
//! `placements(document)` is held by a test to be exactly
//! `interpret::placements(&RuleInputs::of(document)?.chart)` — it adapts
//! and does not re-derive. Both readings live on the chart foundation, so
//! this composer reads that, and takes the narrowest thing that carries
//! them as every composer here does.

use teistro_chart::foundation::GrahaPosition;
use teistro_intl::messages::sdk::reason;

use crate::Plan;

/// Every graha the two house readings put in different bhavas, in the
/// chart's own order.
///
/// ```
/// use teistro_interpret::chalit;
///
/// // A chart whose readings agree says nothing: the disagreement is the
/// // fact, and there is none.
/// assert!(chalit(&[]).is_empty());
/// ```
#[must_use]
pub fn chalit(grahas: &[GrahaPosition]) -> Plan {
    let mut plan = Plan::default();
    for position in grahas {
        if position.house.bhava == position.placement.bhava {
            continue;
        }
        plan.say(&reason::ChalitShift {
            graha: position.graha,
            bhava: i64::from(position.house.bhava),
            chalit: i64::from(position.placement.bhava),
        });
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

    use teistro_chart::bhava::Placement;
    use teistro_core::catalogue::{Graha, HouseSystem};

    use super::*;
    use crate::{Item, KEYS};

    fn placed(bhava: u8) -> Placement {
        Placement {
            bhava,
            method: HouseSystem::WholeSign,
            through: 0.0,
            from_madhya_deg: 0.0,
        }
    }

    /// Each graha in the house the first number names by sign and the
    /// second by chalit.
    fn chart(readings: &[(Graha, u8, u8)]) -> Vec<GrahaPosition> {
        readings
            .iter()
            .map(|(graha, sign, chalit)| GrahaPosition {
                graha: *graha,
                longitude_deg: 0.0,
                tropical_deg: 0.0,
                latitude_deg: 0.0,
                distance_au: 0.0,
                speed_deg_per_day: 0.0,
                placement: placed(*chalit),
                house: placed(*sign),
            })
            .collect()
    }

    #[test]
    fn only_the_grahas_that_differ_are_said() {
        let plan = chalit(&chart(&[
            (Graha::Sun, 1, 1),
            (Graha::Moon, 6, 5),
            (Graha::Mars, 9, 9),
            (Graha::Mercury, 12, 1),
        ]));
        assert_eq!(plan.len(), 2, "the two that disagree");
        assert_eq!(
            plan.items[0],
            Item::of(&reason::ChalitShift {
                graha: Graha::Moon,
                bhava: 6,
                chalit: 5,
            })
        );
        assert_eq!(
            plan.items[1],
            Item::of(&reason::ChalitShift {
                graha: Graha::Mercury,
                bhava: 12,
                chalit: 1,
            }),
            "a shift across the first house is a shift like any other"
        );
    }

    /// A chart whose readings agree everywhere says nothing, which is the
    /// answer: there is no disagreement to report.
    #[test]
    fn a_chart_that_agrees_says_nothing() {
        let plan = chalit(&chart(&[(Graha::Sun, 1, 1), (Graha::Moon, 7, 7)]));
        assert!(plan.is_empty());
    }

    #[test]
    fn it_emits_only_listed_keys() {
        let plan = chalit(&chart(&[(Graha::Sun, 1, 2)]));
        for key in plan.keys() {
            assert!(KEYS.contains(&key), "`{key}` is not in KEYS");
        }
    }
}
