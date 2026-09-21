//! Which chara karaka each graha holds, under both schemes
//! (`03-design/interpret-composers.md` §4).
//!
//! It reads the same `RuleChart` [`placements`](crate::placements) reads, so
//! it costs a request no section of its own, and it says the two of a
//! placement's nine facts that name a graha's Jaimini significator.
//!
//! **Both schemes, because they disagree.** Over the corpus's 93 charts the
//! seven-karaka scheme and the eight-karaka one give a graha the same karaka
//! 326 times and a different one 325 — and the eight reaches one graha a
//! chart the seven does not rank at all. A composer emitting one of them
//! would be choosing for the consumer, silently, in half of all cases, so it
//! emits both and the **key** says which: a consumer filtering by key gets
//! one scheme whole rather than reading a slot to find out which it has.
//!
//! **Which scheme a chart carries is not this composer's choice either.**
//! `rule_chart` computes the karakas as BPHS ch. 32 orders them
//! (`EightKarakas::Parashara`); the recording engine puts the Pitrikaraka
//! last, and `RuleChart::with_chara_karakas` switches. This says whatever
//! the chart holds, which is why the measured page composes the corpus's
//! **recorded** karakas the way `strength` composes its recorded rupas.
//!
//! It is a composer of its own rather than lines inside
//! [`conditions`](crate::conditions) for the reason `positions` is one:
//! a chara karaka is Jaimini's reading of a placement rather than a
//! Parashari condition of it, and apart, the tradition is a knob.

use teistro_core::catalogue::Graha;
use teistro_intl::messages::sdk::karaka;
use teistro_rules::{Body, RuleChart};

use crate::Plan;

/// Which chara karaka each of the nine grahas holds, in both schemes.
///
/// The order is the catalogue's — the Sun to Ketu — and a graha's two
/// karakas are said together, the seven's first. A graha that holds none
/// under a scheme is left out of that scheme rather than said as nothing:
/// Ketu holds none under either, and Rahu none under the seven.
///
/// ```
/// # use teistro_core::catalogue::{CharaKaraka, Dignity, Rashi};
/// # use teistro_rules::{House, Placement, RuleChart};
/// let placement = Placement {
///     longitude: 15.0,
///     sign: Rashi::Aries,
///     house: House::try_new(1)?,
///     dignity: Dignity::Neutral,
///     retrograde: false,
///     combust: false,
///     karaka7: Some(CharaKaraka::Atmakaraka),
///     karaka8: None,
///     navamsha: Rashi::Aries,
/// };
/// let chart = RuleChart { placements: [placement; 10], panchanga: None, strengths: None };
///
/// let plan = teistro_interpret::karakas(&chart);
/// assert_eq!(plan.len(), 9, "one scheme holds, the other says nothing");
/// assert_eq!(plan.items[0].key, "sdk.karaka.ofSeven");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[must_use]
pub fn karakas(chart: &RuleChart) -> Plan {
    let mut plan = Plan::default();
    for graha in Graha::ALL.into_iter().take(9) {
        let Some(at) = chart.placements.get(Body::Graha(graha).index()) else {
            continue;
        };
        if let Some(held) = at.karaka7 {
            plan.say(&karaka::OfSeven {
                graha,
                karaka: held,
            });
        }
        if let Some(held) = at.karaka8 {
            plan.say(&karaka::OfEight {
                graha,
                karaka: held,
            });
        }
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

    use teistro_core::catalogue::{CharaKaraka, Dignity, Rashi};
    use teistro_rules::{EightKarakas, House, Placement};

    use super::*;
    use crate::{Item, KEYS};

    /// Ten bodies a degree apart, with no karaka of their own, so a test
    /// that wants one says so and a test that wants the kernel's ranking
    /// asks the kernel for it.
    fn bare() -> RuleChart {
        let at = |index: usize| Placement {
            longitude: f64::from(u8::try_from(index).unwrap()),
            sign: Rashi::Aries,
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

    #[test]
    fn a_graha_holding_none_is_left_out() {
        assert_eq!(karakas(&bare()).len(), 0);
    }

    #[test]
    fn the_seven_are_said_before_the_eight() {
        let mut chart = bare();
        let sun = &mut chart.placements[Body::Graha(Graha::Sun).index()];
        sun.karaka7 = Some(CharaKaraka::Atmakaraka);
        sun.karaka8 = Some(CharaKaraka::Amatyakaraka);
        let plan = karakas(&chart);
        assert_eq!(plan.len(), 2);
        assert_eq!(
            plan.items[0],
            Item::of(&karaka::OfSeven {
                graha: Graha::Sun,
                karaka: CharaKaraka::Atmakaraka,
            })
        );
        assert_eq!(
            plan.items[1],
            Item::of(&karaka::OfEight {
                graha: Graha::Sun,
                karaka: CharaKaraka::Amatyakaraka,
            })
        );
    }

    /// The two schemes disagree on real charts, which is why both are said.
    /// The kernel ranks them; this holds that the composer repeats whatever
    /// ranking the chart carries rather than deriving one of its own.
    #[test]
    fn it_says_the_ranking_the_chart_carries() {
        let chart = bare().with_chara_karakas(EightKarakas::Parashara);
        let plan = karakas(&chart);
        for item in &plan {
            assert!(item.params.contains_key("graha"), "{item:?}");
            assert!(item.params.contains_key("karaka"), "{item:?}");
        }
        let seven = plan
            .items
            .iter()
            .filter(|item| item.key == "sdk.karaka.ofSeven")
            .count();
        let eight = plan
            .items
            .iter()
            .filter(|item| item.key == "sdk.karaka.ofEight")
            .count();
        assert_eq!(seven, 7, "the seven from the Sun to Saturn");
        assert_eq!(eight, 8, "and Rahu besides");
    }

    /// The lagna stands in the chart and is a point, not a graha.
    #[test]
    fn the_lagna_is_left_out_as_it_is_everywhere_else() {
        let chart = bare().with_chara_karakas(EightKarakas::Parashara);
        let written = serde_json::to_string(&karakas(&chart)).unwrap();
        assert!(!written.contains("LAGNA"), "{written}");
    }

    #[test]
    fn it_emits_only_listed_keys() {
        let chart = bare().with_chara_karakas(EightKarakas::Parashara);
        for key in karakas(&chart).keys() {
            assert!(KEYS.contains(&key), "`{key}` is not in KEYS");
        }
    }

    #[test]
    fn a_plan_is_the_same_bytes_every_time() {
        let chart = bare().with_chara_karakas(EightKarakas::Parashara);
        let once = serde_json::to_string(&karakas(&chart)).unwrap();
        let twice = serde_json::to_string(&karakas(&chart)).unwrap();
        assert_eq!(once, twice);
        let read: Plan = serde_json::from_str(&once).unwrap();
        assert_eq!(read, karakas(&chart));
    }
}
