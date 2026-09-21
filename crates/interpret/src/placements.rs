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
//! messages here read a graha, so the composer leaves it out until a message
//! of its own is written and translated.

use teistro_core::catalogue::{Graha, Rashi};
use teistro_intl::Value;
use teistro_intl::messages::sdk::reason;
use teistro_rules::{Body, RuleChart};

use crate::Plan;

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
        plan.say(&reason::GrahaInRashi {
            graha,
            rashi: at.sign,
        });
        plan.say(&reason::GrahaInBhava {
            graha,
            bhava: i64::from(at.house.get()),
        });
    }
    for rashi in Rashi::ALL {
        let together: Vec<Value> = Graha::ALL
            .into_iter()
            .take(9)
            .filter(|graha| {
                chart
                    .placements
                    .get(Body::Graha(*graha).index())
                    .is_some_and(|at| at.sign == rashi)
            })
            .map(Value::catalogued)
            .collect();
        if together.len() > 1 {
            plan.say(&reason::Occupants {
                grahas: together,
                rashi,
            });
        }
    }
    plan
}

#[cfg(test)]
pub(crate) mod tests {
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
    use crate::{Item, KEYS};

    /// A chart with each graha one sign further round, the Moon joining the
    /// Sun so that one sign is shared.
    pub(crate) fn chart() -> RuleChart {
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
            placement.sign = Rashi::ALL[at % 12];
            placement.house = House::try_new(u8::try_from(at % 12).unwrap() + 1).unwrap();
        }
        placements[1].sign = Rashi::Aries;
        placements[1].house = House::try_new(1).unwrap();
        // The lagna rises in Aries, so a whole-sign house is the sign's own
        // number and a rule reading houses reads what these placements say.
        placements[9].sign = Rashi::Aries;
        placements[9].house = House::try_new(1).unwrap();
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
            Item::of(&reason::GrahaInRashi {
                graha: Graha::Sun,
                rashi: Rashi::Aries,
            })
        );
        assert_eq!(
            plan.items[1],
            Item::of(&reason::GrahaInBhava {
                graha: Graha::Sun,
                bhava: 1,
            })
        );
        assert_eq!(
            plan.items.last().unwrap(),
            &Item::of(&reason::Occupants {
                grahas: vec![
                    Value::catalogued(Graha::Sun),
                    Value::catalogued(Graha::Moon)
                ],
                rashi: Rashi::Aries,
            })
        );
    }

    /// The lagna stands in the chart and is not in the plan: the messages
    /// read a graha, and `point.LAGNA` is not one.
    #[test]
    fn the_lagna_is_left_out_until_it_has_a_message() {
        let written = serde_json::to_string(&placements(&chart())).unwrap();
        assert!(!written.contains("LAGNA"), "{written}");
    }

    /// Every key it emits is listed, so the gate over the packs sees all of
    /// them.
    #[test]
    fn it_emits_only_listed_keys() {
        for key in placements(&chart()).keys() {
            assert!(KEYS.contains(&key), "`{key}` is not in KEYS");
        }
    }

    /// The same chart gives the same plan, byte for byte, and reads back.
    #[test]
    fn a_plan_is_the_same_bytes_every_time() {
        let once = serde_json::to_string(&placements(&chart())).unwrap();
        let twice = serde_json::to_string(&placements(&chart())).unwrap();
        assert_eq!(once, twice);
        let read: Plan = serde_json::from_str(&once).unwrap();
        assert_eq!(read, placements(&chart()));
    }
}
