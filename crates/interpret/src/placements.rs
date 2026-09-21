//! What the chart is: where each graha stands, and which of them share a
//! sign (`03-design/interpret-composers.md` §4).
//!
//! This composer adds no message of its own. Every key it emits is one the
//! locale packs already carry in both strict locales, translated by hand, so
//! a plan of placements renders in English and in Nepali the day it is
//! written — which is why it is the first composer rather than the one the
//! roadmap names first.
//!
//! What it cannot say is counted rather than hidden: the **lagna** is placed
//! in the chart like a graha but is `point.LAGNA` in the catalogue, and the
//! messages here read a `graha`, so the composer leaves it out until a
//! message of its own is written and translated.

use teistro_core::catalogue::{Graha, Rashi};
use teistro_intl::{Value, params};
use teistro_rules::{Body, RuleChart};

use crate::{Plan, entity};

/// Every message key this module can emit.
///
/// `check-interpret` holds it against the packs and against what the
/// composers emit over the corpus, both ways: a key no locale carries
/// renders as a visible fallback, and a key nothing emits is a message
/// nobody reads.
pub const KEYS: [&str; 3] = [
    "sdk.reason.grahaInRashi",
    "sdk.reason.grahaInBhava",
    "sdk.reason.occupants",
];

/// Where each of the nine grahas stands, and who shares a sign.
///
/// The order is the catalogue's — the Sun to Ketu, each in its sign and then
/// in its house — and then the shared signs in zodiac order, so the same
/// chart always gives the same plan.
#[must_use]
pub fn placements(chart: &RuleChart) -> Plan {
    let mut plan = Plan::default();
    for graha in Graha::ALL.into_iter().take(9) {
        let Some(at) = chart.placements.get(Body::Graha(graha).index()) else {
            continue;
        };
        let who = entity(graha.full_key());
        plan.push(
            "sdk.reason.grahaInRashi",
            params([
                ("graha", who.clone()),
                ("rashi", entity(at.sign.full_key())),
            ]),
        );
        plan.push(
            "sdk.reason.grahaInBhava",
            params([
                ("graha", who),
                ("bhava", Value::Int(i64::from(at.house.get()))),
            ]),
        );
    }
    for sign in Rashi::ALL {
        let together: Vec<Value> = Graha::ALL
            .into_iter()
            .take(9)
            .filter(|graha| {
                chart
                    .placements
                    .get(Body::Graha(*graha).index())
                    .is_some_and(|at| at.sign == sign)
            })
            .map(|graha| entity(graha.full_key()))
            .collect();
        if together.len() > 1 {
            plan.push(
                "sdk.reason.occupants",
                params([
                    ("grahas", Value::List(together)),
                    ("rashi", entity(sign.full_key())),
                ]),
            );
        }
    }
    plan
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        reason = "tests unwrap what they build and fail by panicking"
    )]

    use teistro_core::catalogue::Dignity;
    use teistro_rules::{House, Placement};

    use super::*;

    /// A chart with each graha one sign further round, the lagna in Aries.
    fn chart() -> RuleChart {
        let mut placements = [Placement {
            longitude: 15.0,
            sign: Rashi::Aries,
            house: House::try_new(1).unwrap(),
            dignity: Dignity::Neutral,
            retrograde: false,
            combust: false,
            karaka7: None,
            karaka8: None,
            navamsha: Rashi::Aries,
        }; 10];
        for (at, placement) in placements.iter_mut().enumerate() {
            let sign = Rashi::ALL[at % 12];
            placement.sign = sign;
            placement.house = House::try_new(u8::try_from(at % 12).unwrap() + 1).unwrap();
        }
        // The Moon joins the Sun in Aries, so one sign is shared.
        placements[1].sign = Rashi::Aries;
        placements[1].house = House::try_new(1).unwrap();
        RuleChart {
            placements,
            panchanga: None,
            strengths: None,
        }
    }

    #[test]
    fn every_graha_is_placed_in_its_sign_and_its_house() {
        let plan = placements(&chart());
        // Two items a graha, then the one shared sign.
        assert_eq!(plan.len(), 9 * 2 + 1);
        assert_eq!(
            plan.items[0],
            crate::Item::new(
                "sdk.reason.grahaInRashi",
                params([
                    ("graha", entity("graha.SUN")),
                    ("rashi", entity("rashi.ARIES")),
                ])
            )
        );
        assert_eq!(
            plan.items[1],
            crate::Item::new(
                "sdk.reason.grahaInBhava",
                params([("graha", entity("graha.SUN")), ("bhava", Value::Int(1))])
            )
        );
        assert_eq!(
            plan.items.last().unwrap(),
            &crate::Item::new(
                "sdk.reason.occupants",
                params([
                    (
                        "grahas",
                        Value::List(vec![entity("graha.SUN"), entity("graha.MOON")])
                    ),
                    ("rashi", entity("rashi.ARIES")),
                ])
            )
        );
    }

    /// The lagna stands in the chart and is not in the plan: the messages
    /// read a graha, and `point.LAGNA` is not one.
    #[test]
    fn the_lagna_is_left_out_until_it_has_a_message() {
        let plan = placements(&chart());
        let written = serde_json::to_string(&plan).unwrap();
        assert!(!written.contains("LAGNA"), "{written}");
    }

    /// Every key the composer emits is listed, so the gate over the packs
    /// sees all of them.
    #[test]
    fn it_emits_only_listed_keys() {
        let plan = placements(&chart());
        for key in plan.keys() {
            assert!(KEYS.contains(&key), "`{key}` is not in KEYS");
        }
        // And every listed key is emitted by a chart that exercises them all.
        for key in KEYS {
            assert!(plan.keys().contains(&key), "`{key}` is emitted by nothing");
        }
    }

    /// The same chart gives the same plan, byte for byte.
    #[test]
    fn a_plan_is_the_same_bytes_every_time() {
        let once = serde_json::to_string(&placements(&chart())).unwrap();
        let twice = serde_json::to_string(&placements(&chart())).unwrap();
        assert_eq!(once, twice);
        let read: Plan = serde_json::from_str(&once).unwrap();
        assert_eq!(read, placements(&chart()));
    }
}
