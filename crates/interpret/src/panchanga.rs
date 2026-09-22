//! The almanac of the day the chart belongs to: its five limbs, the Moon's
//! pada, and whether the birth fell by day
//! (`03-design/interpret-composers.md` §4).
//!
//! The thirteenth composer, and the answer to Q39. `PANCHANGA` was the
//! only section a composer read and said **nothing of its own**: `phala`
//! renders a loaded pack's reading of the tithi, vara, nakshatra and yoga
//! and is silent without one, so a consumer with no pack got no item from
//! this section at all — and the **karana** was said by nothing anywhere.
//! A library whose subject is the almanac should be able to say the
//! almanac.
//!
//! **Every slot is a catalogue member every locale already names**, so the
//! frame was the whole cost: seven messages, not one new term, and nothing
//! for the native review it has not already seen.
//!
//! **The tithi says its paksha**, because the two are one fact and the
//! entity names split them: `tithi.SHUKLA_PRATIPADA` is named *Pratipada*
//! and the paksha is a kind of its own, so a message printing the tithi
//! alone would lose the half that says which fortnight.
//!
//! **Where the scope was drawn, and why there rather than further.** The
//! same record carries the ghatikas each limb had used and had left, the
//! sankranti window and the eclipse, and none of the three is said. A span
//! is a pair of ghatikas a consumer formats with, which is the line
//! `sdk.reason.exactLongitude` is already on; a sankranti and an eclipse
//! are **conditions of the day** rather than limbs of it, and belong with
//! whatever says the rest of those. The measured page's section table
//! carries that as what is left, so the line is measured rather than
//! remembered.

use teistro_intl::messages::sdk::reason::panchanga as messages;
use teistro_rules::RuleChart;

use crate::Plan;

/// The key `byDay` selects on where the chart says the birth was by day.
const BY_DAY: &str = "yes";

/// The key it selects on where the chart says it was by night.
///
/// Its own arm rather than the catch-all, so that a chart which says
/// **nothing** cannot be read as saying night: the item is not emitted at
/// all in that case, and the two arms both name what they mean.
const BY_NIGHT: &str = "no";

/// The almanac of the chart's day, in the panchanga's own order.
///
/// Tithi, vara, nakshatra, the Moon's pada in it, yoga, karana — the five
/// limbs as every almanac prints them, with the pada beside the nakshatra
/// it divides — and then whether the birth fell by day, which is said only
/// where the chart says it.
///
/// A chart with no panchanga says nothing, rather than an empty almanac:
/// the section is one a request asks for by name.
#[must_use]
pub fn panchanga(chart: &RuleChart) -> Plan {
    let mut plan = Plan::default();
    let Some(day) = chart.panchanga.as_ref() else {
        return plan;
    };
    plan.say(&messages::Tithi {
        tithi: day.tithi,
        paksha: day.tithi.attributes().paksha,
    });
    plan.say(&messages::Vara { vara: day.vara });
    plan.say(&messages::Nakshatra {
        nakshatra: day.nakshatra,
    });
    plan.say(&messages::Pada {
        nakshatra: day.nakshatra,
        pada: i64::from(day.pada.get()),
    });
    plan.say(&messages::Yoga { yoga: day.yoga });
    plan.say(&messages::Karana { karana: day.karana });
    // Only where the chart says: `by_day` is an option because a chart
    // whose maker did not record the sunrise cannot answer, and a default
    // would invent an answer to a question nobody asked.
    if let Some(by_day) = day.by_day {
        plan.say(&messages::ByDay {
            by_day: String::from(if by_day { BY_DAY } else { BY_NIGHT }),
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
        reason = "tests fail by panicking"
    )]

    use teistro_core::catalogue::{Karana, Nakshatra, Paksha, Tithi, Vara, Yoga};
    use teistro_rules::{Pada, Panchanga, Spans};

    use super::panchanga;
    use teistro_intl::Value;

    use crate::KEYS;

    fn day() -> Panchanga {
        Panchanga {
            tithi: Tithi::KrishnaPanchami,
            vara: Vara::Ravivara,
            nakshatra: Nakshatra::Bharani,
            pada: Pada::try_new(3).unwrap(),
            yoga: Yoga::Vishkambha,
            karana: Karana::Bava,
            spans: Spans::default(),
            by_day: None,
            on_sankranti: false,
            eclipse: None,
        }
    }

    fn chart(panchanga: Option<Panchanga>) -> teistro_rules::RuleChart {
        let mut chart = crate::placements::tests::chart();
        chart.panchanga = panchanga;
        chart
    }

    /// The five limbs and the pada, in the almanac's own order.
    #[test]
    fn the_limbs_are_said_in_the_almanacs_order() {
        let plan = panchanga(&chart(Some(day())));
        let said: Vec<&str> = plan.items.iter().map(|item| item.key.as_str()).collect();
        assert_eq!(
            said,
            [
                "sdk.reason.panchanga.tithi",
                "sdk.reason.panchanga.vara",
                "sdk.reason.panchanga.nakshatra",
                "sdk.reason.panchanga.pada",
                "sdk.reason.panchanga.yoga",
                "sdk.reason.panchanga.karana",
            ],
            "and no byDay, which this chart does not say"
        );
        for key in plan.keys() {
            assert!(KEYS.contains(&key), "`{key}` is not in KEYS");
        }
    }

    /// The tithi carries its paksha, because the entity names split what
    /// the key joins: `KRISHNA_PANCHAMI` is named *Panchami* alone.
    #[test]
    fn the_tithi_says_which_fortnight_it_is_in() {
        let plan = panchanga(&chart(Some(day())));
        let tithi = &plan.items[0];
        assert_eq!(
            tithi.params.get("paksha"),
            Some(&Value::catalogued(Paksha::Krishna))
        );
        assert_eq!(
            tithi.params.get("tithi"),
            Some(&Value::catalogued(Tithi::KrishnaPanchami))
        );
    }

    /// Whether the birth fell by day is said only where the chart says it,
    /// and each answer names itself rather than riding the catch-all.
    #[test]
    fn the_day_and_the_night_are_both_named() {
        let mut by_day = day();
        by_day.by_day = Some(true);
        let plan = panchanga(&chart(Some(by_day)));
        assert_eq!(
            plan.items[plan.len() - 1].params.get("byDay"),
            Some(&Value::Str(String::from("yes")))
        );

        let mut by_night = day();
        by_night.by_day = Some(false);
        let plan = panchanga(&chart(Some(by_night)));
        assert_eq!(
            plan.items[plan.len() - 1].params.get("byDay"),
            Some(&Value::Str(String::from("no")))
        );
    }

    /// A chart with no panchanga says nothing, rather than an almanac of
    /// defaults: the section is one a request asks for by name.
    #[test]
    fn a_chart_with_no_almanac_says_nothing() {
        assert!(panchanga(&chart(None)).is_empty());
    }
}
