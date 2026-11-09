//! What one sign's rashifal says in words: its transits, Saturn's standing
//! from it and each event of the period (`03-design/rashifal.md`, "Said in
//! words").
//!
//! It reads a [`RashiReading`] and nothing else, so like
//! [`gochar`](crate::gochar) it judges nothing: the transits are said by
//! the gochar composer itself, Saturn's phase and spell are the reading's,
//! and an event's house is good or not as v. 2 makes it. It needs no
//! corpus, since the rule and its words are the SDK's.
//!
//! **One plan a sign.** A period is read for the twelve signs, and a reader
//! reads one of them; the façade composes the twelve, Aries to Pisces.
//!
//! **What is left unsaid**, deliberately. An event's instant is a UTC
//! moment and the plan has no clock to date it by, so the date is the
//! reading's (`events[i].event.hit.instant`), and the event items are said
//! in the events' own order so the n-th of them is the n-th event. The
//! direction of an ingress is left to the reading too: the nodes always
//! move backwards, and "Rahu enters Aquarius retrograde" tells a reader
//! nothing. A hit that is neither an ingress nor a station is not an event
//! of a period, so it is not said.

use teistro_gochar::hits::{HitEvent, Motion};
use teistro_gochar::sade_sati::Phase;
use teistro_intl::messages::sdk::rashifal as said;
use teistro_rashifal::{EventFrom, RashiReading, SaturnStanding};

use crate::Plan;

/// What `reading` says, in the order a reader takes it in.
///
/// The transits first, as [`gochar`](crate::gochar) says them; then
/// Saturn's standing from the sign, its Sade Sati phase or smaller spell
/// when it is in one; then each ingress and station of the period, the
/// house it falls in from the sign and whether v. 2 makes that house good.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_gochar::Transit;
/// use teistro_rashifal::{RashifalRules, rashifal};
///
/// // Saturn in Pisces, the nodes opposite, the rest in Aries.
/// let mut transits = [Transit::new(Rashi::Aries, 5.0); 9];
/// transits[Graha::Saturn as usize] = Transit::new(Rashi::Pisces, 12.0);
/// transits[Graha::Rahu as usize] = Transit::new(Rashi::Virgo, 3.0);
/// transits[Graha::Ketu as usize] = Transit::new(Rashi::Pisces, 3.0);
/// let read = rashifal(&transits, &[], &RashifalRules::default());
///
/// let plan = teistro_interpret::rashifal(&read[0]);
/// assert_eq!(plan.items[0].key, "sdk.reason.gochar.from");
/// // Pisces is the 12th from Aries: Sade Sati rising.
/// let saturn = plan.iter().find(|item| item.key == "sdk.rashifal.saturn").unwrap();
/// assert_eq!(saturn.params["phase"], teistro_intl::Value::Str("RISING".into()));
/// ```
#[must_use]
pub fn rashifal(reading: &RashiReading) -> Plan {
    let mut plan = crate::gochar(&reading.gochar);
    say_saturn(&mut plan, reading.saturn);
    for event in &reading.events {
        say_event(&mut plan, event);
    }
    plan
}

/// Saturn's house from the sign, with its phase or its spell.
fn say_saturn(plan: &mut Plan, saturn: SaturnStanding) {
    plan.say(&said::Saturn {
        house: i64::from(saturn.house),
        phase: String::from(phase_key(saturn.sade_sati)),
        spell: String::from(yes_or_no(saturn.spell)),
    });
}

/// One event: an ingress into its sign, or a station in it.
fn say_event(plan: &mut Plan, from: &EventFrom) {
    let hit = from.event.hit;
    let house = i64::from(from.house);
    let good = String::from(yes_or_no(from.good_house));
    match hit.event {
        HitEvent::SignIngress { .. } => plan.say(&said::Ingress {
            graha: hit.graha,
            rashi: from.event.sign,
            house,
            good,
        }),
        HitEvent::Station { turns } => plan.say(&said::Station {
            graha: hit.graha,
            turns: String::from(motion_key(turns)),
            rashi: from.event.sign,
            house,
            good,
        }),
        _ => {}
    }
}

/// The selector `sdk.rashifal.saturn` reads a phase by: the phase's own
/// serde spelling, or `NONE` outside Sade Sati.
const fn phase_key(phase: Option<Phase>) -> &'static str {
    match phase {
        Some(Phase::Rising) => "RISING",
        Some(Phase::Peak) => "PEAK",
        Some(Phase::Setting) => "SETTING",
        None => "NONE",
    }
}

/// The selector `sdk.rashifal.station` reads a station by: the motion the
/// graha turns to, in its serde spelling.
const fn motion_key(motion: Motion) -> &'static str {
    match motion {
        Motion::Direct => "DIRECT",
        Motion::Retrograde => "RETROGRADE",
    }
}

/// A yes-or-no selector, `yes` matched and anything else the fallback.
const fn yes_or_no(holds: bool) -> &'static str {
    if holds { "yes" } else { "no" }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        clippy::indexing_slicing,
        reason = "tests unwrap what they build and fail by panicking"
    )]

    use teistro_core::catalogue::{Graha, Nakshatra, Rashi};
    use teistro_core::quantity::JulianDay;
    use teistro_gochar::Transit;
    use teistro_gochar::hits::Hit;
    use teistro_intl::{TypedMessage, Value};
    use teistro_rashifal::{PeriodEvent, RashifalRules, rashifal as read};

    use super::*;
    use crate::{Item, KEYS};

    /// Saturn in Pisces and the nodes across Virgo and Pisces, the rest in
    /// Aries; Jupiter enters Taurus and Mercury turns retrograde in Aries.
    fn readings() -> [RashiReading; 12] {
        let mut transits = [Transit::new(Rashi::Aries, 5.0); 9];
        transits[Graha::Saturn as usize] = Transit::new(Rashi::Pisces, 12.0);
        transits[Graha::Rahu as usize] = Transit::new(Rashi::Virgo, 3.0);
        transits[Graha::Ketu as usize] = Transit::new(Rashi::Pisces, 3.0);
        let event = |graha, event, sign| PeriodEvent {
            hit: Hit {
                instant: JulianDay::literal(2_460_676.5),
                graha,
                event,
            },
            sign,
        };
        let events = [
            event(
                Graha::Jupiter,
                HitEvent::SignIngress {
                    into: Rashi::Taurus,
                    motion: Motion::Direct,
                },
                Rashi::Taurus,
            ),
            event(
                Graha::Mercury,
                HitEvent::Station {
                    turns: Motion::Retrograde,
                },
                Rashi::Aries,
            ),
            event(
                Graha::Moon,
                HitEvent::NakshatraIngress {
                    into: Nakshatra::Ashwini,
                    motion: Motion::Direct,
                },
                Rashi::Aries,
            ),
        ];
        read(&transits, &events, &RashifalRules::default())
    }

    fn keyed<'p>(plan: &'p Plan, key: &str) -> Vec<&'p Item> {
        plan.items.iter().filter(|item| item.key == key).collect()
    }

    #[test]
    fn the_transits_are_the_gochar_composers_own() {
        for reading in &readings() {
            let plan = rashifal(reading);
            let transits = crate::gochar(&reading.gochar);
            assert_eq!(plan.items[..transits.len()], transits.items[..]);
        }
    }

    #[test]
    fn saturn_is_said_once_with_its_phase_or_spell() {
        let all = readings();
        let saturn = |rashi: Rashi| {
            let plan = rashifal(&all[rashi as usize]);
            let said = keyed(&plan, said::Saturn::KEY);
            assert_eq!(said.len(), 1, "{rashi:?}");
            said[0].clone()
        };
        // Pisces is the 12th from Aries, the 1st from itself, the 2nd from
        // Aquarius, the 8th from Leo and the 4th from Sagittarius.
        for (rashi, house, phase, spell) in [
            (Rashi::Aries, 12, "RISING", "no"),
            (Rashi::Pisces, 1, "PEAK", "no"),
            (Rashi::Aquarius, 2, "SETTING", "no"),
            (Rashi::Leo, 8, "NONE", "yes"),
            (Rashi::Sagittarius, 4, "NONE", "yes"),
            (Rashi::Taurus, 11, "NONE", "no"),
        ] {
            assert_eq!(
                saturn(rashi),
                Item::of(&said::Saturn {
                    house,
                    phase: String::from(phase),
                    spell: String::from(spell),
                }),
                "{rashi:?}"
            );
        }
    }

    #[test]
    fn each_event_is_said_in_order_from_the_sign() {
        let all = readings();
        let plan = rashifal(&all[Rashi::Aries as usize]);
        let events: Vec<&Item> = plan
            .items
            .iter()
            .filter(|item| item.key == said::Ingress::KEY || item.key == said::Station::KEY)
            .collect();
        // The nakshatra ingress is not an event of a period.
        assert_eq!(events.len(), 2, "{events:?}");
        // Taurus is the 2nd from Aries, a good house for Jupiter.
        assert_eq!(
            events[0],
            &Item::of(&said::Ingress {
                graha: Graha::Jupiter,
                rashi: Rashi::Taurus,
                house: 2,
                good: String::from("yes"),
            })
        );
        // Mercury stations in the 1st, which v. 2 does not make good.
        assert_eq!(
            events[1],
            &Item::of(&said::Station {
                graha: Graha::Mercury,
                turns: String::from("RETROGRADE"),
                rashi: Rashi::Aries,
                house: 1,
                good: String::from("no"),
            })
        );
        // The events close the plan.
        assert_eq!(plan.items.last(), Some(events[1]));
        // And are counted from each sign: Taurus is the 1st from itself.
        let taurus = rashifal(&all[Rashi::Taurus as usize]);
        assert_eq!(
            keyed(&taurus, said::Ingress::KEY)[0].params.get("house"),
            Some(&Value::Int(1))
        );
    }

    #[test]
    fn it_emits_only_listed_keys_and_the_same_bytes_every_time() {
        for reading in &readings() {
            let plan = rashifal(reading);
            for key in plan.keys() {
                assert!(KEYS.contains(&key), "`{key}` is not in KEYS");
            }
            let once = serde_json::to_string(&plan).unwrap();
            assert_eq!(once, serde_json::to_string(&rashifal(reading)).unwrap());
        }
    }
}
