//! Where each graha stands **to the degree**, which is what `placements`
//! rounds away (`03-design/interpret-composers.md` §4).
//!
//! It reads the same `RuleChart` [`placements`](crate::placements) reads,
//! so it costs a request no section of its own, and like every composer
//! before it it adds no message: `sdk.reason.grahaAt` was carried by both
//! strict locales, translated by hand and tested, and read by nothing.
//!
//! **Why it is not a line inside `placements`.** `grahaInRashi` says the
//! sign and `grahaAt` says the sign *and* the degree, so one subsumes the
//! other and a composer emitting both would repeat itself once a graha.
//! Apart, the precision is a knob: a narrative report asks for `placements`,
//! a position table asks for this, and a consumer asking for both can see
//! that it is paying for the sign twice.
//!
//! It does **not** emit `sdk.reason.exactLongitude`, which renders a
//! longitude alone — `222°34′35″`. That is a fragment a consumer formats
//! with rather than a sentence a plan says, which is the line
//! [`strength`](crate::strength) drew at `strength.rank`. The rule has held
//! twice now: a message in the pack is a plan item when it says something
//! on its own.
//!
//! One caveat belongs to the locale engine and not to this composer.
//! `grahaAt` rounds to arc-minutes to display, so a body at 29.9999° Aries
//! renders as `0°00′ Taurus` — `crates/intl` tests that case on purpose. A
//! plan carrying both composers therefore says "Sun in Aries" and "Sun at
//! 0°00′ Taurus" of one body. Neither is wrong: one reads the longitude and
//! the other reads it rounded, and rounding the sign to match would make the
//! plan disagree with the chart.

use teistro_core::catalogue::Graha;
use teistro_intl::messages::sdk::reason;
use teistro_rules::{Body, RuleChart};

use crate::Plan;

/// Where each of the nine grahas stands, to the degree.
///
/// The order is the catalogue's — the Sun to Ketu — so the same chart
/// always gives the same plan. A body the chart does not place is left out
/// rather than said at zero, as `placements` leaves it out.
///
/// ```
/// # use teistro_core::catalogue::{Dignity, Rashi};
/// # use teistro_rules::{House, Placement, RuleChart};
/// # let placement = Placement {
/// #     longitude: 222.5763,
/// #     sign: Rashi::Scorpio,
/// #     house: House::try_new(1)?,
/// #     dignity: Dignity::Neutral,
/// #     retrograde: false,
/// #     combust: false,
/// #     karaka7: None,
/// #     karaka8: None,
/// #     navamsha: Rashi::Scorpio,
/// # };
/// # let chart = RuleChart { placements: [placement; 10], panchanga: None, strengths: None };
/// let plan = teistro_interpret::positions(&chart);
/// assert_eq!(plan.len(), 9, "the nine grahas, the lagna is a point");
/// assert_eq!(plan.items[0].key, "sdk.reason.grahaAt");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[must_use]
pub fn positions(chart: &RuleChart) -> Plan {
    let mut plan = Plan::default();
    for graha in Graha::ALL.into_iter().take(9) {
        let Some(at) = chart.placements.get(Body::Graha(graha).index()) else {
            continue;
        };
        plan.say(&reason::GrahaAt {
            graha,
            longitude: at.longitude,
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

    use teistro_core::catalogue::{Dignity, Rashi};
    use teistro_intl::Value;
    use teistro_rules::{House, Placement};

    use super::*;
    use crate::{Item, KEYS, placements};

    /// Ten bodies, each 33° further round the zodiac than the last, so a
    /// composer that read the wrong one is visible in the answer.
    fn chart() -> RuleChart {
        let at = |index: usize| Placement {
            longitude: degrees(index),
            sign: Rashi::ALL[index % 12],
            house: House::try_new(1).unwrap(),
            dignity: Dignity::Neutral,
            retrograde: false,
            combust: false,
            karaka7: None,
            karaka8: None,
            navamsha: Rashi::Aries,
        };
        RuleChart {
            placements: core::array::from_fn(at),
            panchanga: None,
            strengths: None,
        }
    }

    /// Where the nth body stands, as a number no cast rounds.
    fn degrees(index: usize) -> f64 {
        f64::from(u8::try_from(index).unwrap()) * 33.0
    }

    #[test]
    fn every_graha_is_said_where_it_stands() {
        let plan = positions(&chart());
        assert_eq!(plan.len(), 9, "the nine grahas, the lagna is a point");
        assert_eq!(
            plan.items[0],
            Item::of(&reason::GrahaAt {
                graha: Graha::Sun,
                longitude: degrees(0),
            })
        );
        assert_eq!(
            plan.items[8],
            Item::of(&reason::GrahaAt {
                graha: Graha::Ketu,
                longitude: degrees(8),
            })
        );
    }

    /// The lagna stands in the chart and is a point, not a graha, so this
    /// composer leaves it out exactly as `placements` does.
    #[test]
    fn the_lagna_is_left_out_as_it_is_everywhere_else() {
        let written = serde_json::to_string(&positions(&chart())).unwrap();
        assert!(!written.contains("LAGNA"), "{written}");
    }

    /// The longitude crosses as a number and not as a rendered angle: the
    /// words are the locale's, and the rounding is the renderer's.
    #[test]
    fn a_longitude_crosses_as_a_number() {
        let plan = positions(&chart());
        assert_eq!(
            plan.items[1].params.get("longitude"),
            Some(&Value::Num(degrees(1)))
        );
        let written = serde_json::to_string(&plan).unwrap();
        for rendered in ["°", "′", "″"] {
            assert!(!written.contains(rendered), "{written}");
        }
    }

    /// It says the sign as `placements` does, and more: a plan asking for
    /// both carries the sign twice, which is the consumer's choice to make
    /// and the reason the two are separate composers.
    #[test]
    fn it_says_what_placements_says_and_the_degree_besides() {
        let chart = chart();
        let both = positions(&chart).len() + placements(&chart).len();
        assert_eq!(positions(&chart).len(), 9);
        assert!(both > placements(&chart).len(), "a knob, not a replacement");
    }

    #[test]
    fn it_emits_only_listed_keys() {
        for key in positions(&chart()).keys() {
            assert!(KEYS.contains(&key), "`{key}` is not in KEYS");
        }
    }

    #[test]
    fn a_plan_is_the_same_bytes_every_time() {
        let once = serde_json::to_string(&positions(&chart())).unwrap();
        let twice = serde_json::to_string(&positions(&chart())).unwrap();
        assert_eq!(once, twice);
        let read: Plan = serde_json::from_str(&once).unwrap();
        assert_eq!(read, positions(&chart()));
    }
}
