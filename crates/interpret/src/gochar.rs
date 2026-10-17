//! What a gochar reading says in words: the sign the transits are counted
//! from, each graha's house and verdict, who obstructs it, where in its
//! sign it bears fruit, and its bindus where the reading carries the
//! Ashtakavarga (`03-design/gochar.md` §7).
//!
//! It reads a [`GocharReading`] and nothing else, so it says what the
//! kernel judged and judges nothing itself: the verdict is Phaladeepika
//! ch. 26's as the settings' `gochar` group read it, and a sentence never
//! softens or sharpens it. Unlike [`sade_sati`](crate::sade_sati) it needs
//! no corpus — the verses' rule is the SDK's, so its words are too — and
//! like [`ashtakavarga`](crate::ashtakavarga) it invents no band: the
//! bindus are said with the reading's own threshold, never a scale of its
//! own.
//!
//! **One reading, one instant.** A batch of instants is a batch of
//! readings, and the plan of each says one sky; the façade composes one
//! plan an instant rather than one plan that would have to say which
//! instant every sentence is of.
//!
//! **What is left unsaid**, deliberately. The kakshya and the sign's
//! sarvashtakavarga are the Ashtakavarga's finer readings, which v. 16
//! reports as what "others say"; the vedha house when nothing stands in
//! it, which is the verdict `GOOD` already; and a graha whose transit
//! bears fruit throughout the sign (Mercury and the nodes, crux C138),
//! for which "where it bears fruit" says nothing a reader can use.

use teistro_core::catalogue::Graha;
use teistro_gochar::{Fruition, GocharFrom, GocharReading, GrahaGochar, Verdict};
use teistro_intl::Value;
use teistro_intl::messages::sdk::reason::gochar as said;

use crate::Plan;

/// What `reading` says, in the order a reader takes it in.
///
/// The reference first, then each graha the Sun to Ketu: its house and
/// verdict, who obstructs it when something does, and where in its sign
/// it bears fruit. The seven's bindus close the plan when the reading
/// carries them, the Sun to Saturn.
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_gochar::{GocharRules, Reference, Transit, gochar};
///
/// // Every graha at 5° Aries, counted from a Moon in Aries.
/// let transits = [Transit::new(Rashi::Aries, 5.0); 9];
/// let reading = gochar(Reference::moon(Rashi::Aries), &transits, GocharRules::TEXT);
/// let plan = teistro_interpret::gochar(&reading);
/// assert_eq!(plan.items[0].key, "sdk.reason.gochar.from");
/// assert_eq!(plan.items[1].key, "sdk.reason.gochar.transit");
/// ```
#[must_use]
pub fn gochar(reading: &GocharReading) -> Plan {
    let mut plan = Plan::default();
    if let Some(from) = from_key(reading.reference.from) {
        plan.say(&said::From {
            from: String::from(from),
            rashi: reading.reference.sign,
        });
    }
    for graha in &reading.grahas {
        say_graha(&mut plan, graha);
    }
    for transit in reading.ashtakavarga.iter().flatten() {
        plan.say(&said::Ashtakavarga {
            graha: transit.graha,
            bindus: i64::from(transit.bindus),
            good: String::from(yes_or_no(transit.good)),
        });
    }
    plan
}

/// One graha's sentences: the transit, its obstruction, its fruition.
fn say_graha(plan: &mut Plan, graha: &GrahaGochar) {
    plan.say(&said::Transit {
        graha: graha.graha,
        house: i64::from(graha.house),
        verdict: String::from(verdict_key(graha.verdict)),
    });
    if let (Verdict::Obstructed, Some(house)) = (graha.verdict, graha.vedha_house) {
        plan.say(&said::Vedha {
            grahas: graha
                .obstructed_by
                .iter()
                .map(|by: &Graha| Value::catalogued(*by))
                .collect(),
            house: i64::from(house),
        });
    }
    if let Some(part) = part_key(graha.fruition) {
        plan.say(&said::Fruition {
            graha: graha.graha,
            part: String::from(part),
            now: String::from(yes_or_no(graha.fruitful_now)),
        });
    }
}

/// The selector `sdk.reason.gochar.from` reads a reference by, or none
/// for a reference the message has no words for, which leaves the plan
/// without its first line rather than calling it the Moon.
const fn from_key(from: GocharFrom) -> Option<&'static str> {
    match from {
        GocharFrom::Moon => Some("MOON"),
        GocharFrom::Lagna => Some("LAGNA"),
        _ => None,
    }
}

/// The selector `sdk.reason.gochar.transit` reads a verdict by: the
/// verdict's own serde spelling.
const fn verdict_key(verdict: Verdict) -> &'static str {
    match verdict {
        Verdict::Good => "GOOD",
        Verdict::Obstructed => "OBSTRUCTED",
        Verdict::NotGood => "NOT_GOOD",
    }
}

/// The selector `sdk.reason.gochar.fruition` reads a decanate by, or none
/// for a graha that bears fruit throughout its sign.
const fn part_key(fruition: Fruition) -> Option<&'static str> {
    match fruition {
        Fruition::First => Some("FIRST"),
        Fruition::Middle => Some("MIDDLE"),
        Fruition::Last => Some("LAST"),
        Fruition::Throughout => None,
    }
}

/// A yes-or-no selector, `yes` matched and anything else the fallback.
const fn yes_or_no(holds: bool) -> &'static str {
    if holds { "yes" } else { "no" }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        clippy::indexing_slicing,
        reason = "tests unwrap what they build and fail by panicking"
    )]

    use teistro_core::catalogue::Rashi;
    use teistro_gochar::{GocharRules, Reference, Transit, gochar as judge};
    use teistro_intl::TypedMessage;

    use super::*;
    use crate::KEYS;

    /// The Moon in Aries; the Sun in the 11th, which is good, with Mars in
    /// its vedha house the 5th (no exemption spares the pair), Jupiter in
    /// the 2nd in its middle decanate, and the rest in the 1st, late.
    fn reading() -> GocharReading {
        let at = |house: usize, degrees: f64| Transit::new(Rashi::ALL[house - 1], degrees);
        let mut transits = [at(1, 25.0); 9];
        transits[Graha::Sun as usize] = at(11, 5.0);
        transits[Graha::Mars as usize] = at(5, 25.0);
        transits[Graha::Jupiter as usize] = at(2, 15.0);
        judge(Reference::moon(Rashi::Aries), &transits, GocharRules::TEXT)
    }

    fn keyed<'p>(plan: &'p Plan, key: &str) -> Vec<&'p crate::Item> {
        plan.items.iter().filter(|item| item.key == key).collect()
    }

    #[test]
    fn the_reference_comes_first_and_every_graha_is_said_once() {
        let plan = gochar(&reading());
        assert_eq!(
            plan.items[0],
            crate::Item::of(&said::From {
                from: String::from("MOON"),
                rashi: Rashi::Aries,
            })
        );
        let transits = keyed(&plan, said::Transit::KEY);
        assert_eq!(transits.len(), 9);
        assert!(
            plan.items
                .iter()
                .all(|item| KEYS.contains(&item.key.as_str()))
        );
    }

    #[test]
    fn an_obstruction_names_who_and_from_where() {
        let read = reading();
        let sun = &read.grahas[Graha::Sun as usize];
        assert_eq!(sun.verdict, Verdict::Obstructed, "{sun:?}");
        assert_eq!(sun.obstructed_by, [Graha::Mars], "{sun:?}");
        let plan = gochar(&read);
        let vedha = keyed(&plan, said::Vedha::KEY);
        let obstructed = read
            .grahas
            .iter()
            .filter(|graha| graha.verdict == Verdict::Obstructed)
            .count();
        assert_eq!(vedha.len(), obstructed, "one line an obstruction");
        assert!(vedha.contains(&&crate::Item::of(&said::Vedha {
            grahas: vec![Value::catalogued(Graha::Mars)],
            house: 5,
        })));
        // Every vedha line follows the transit it obstructs.
        for (at, item) in plan.items.iter().enumerate() {
            if item.key == said::Vedha::KEY {
                let before = &plan.items[at - 1];
                assert_eq!(before.key, said::Transit::KEY, "{before:?}");
                assert_eq!(
                    before.params.get("verdict"),
                    Some(&Value::Str(String::from("OBSTRUCTED")))
                );
            }
        }
    }

    #[test]
    fn fruition_is_said_for_the_six_bound_to_a_decanate() {
        let plan = gochar(&reading());
        let fruition = keyed(&plan, said::Fruition::KEY);
        // Mercury, Rahu and Ketu bear fruit throughout their signs.
        assert_eq!(fruition.len(), 6, "{fruition:?}");
        assert!(fruition.contains(&&crate::Item::of(&said::Fruition {
            graha: Graha::Jupiter,
            part: String::from("MIDDLE"),
            now: String::from("yes"),
        })));
        assert!(fruition.contains(&&crate::Item::of(&said::Fruition {
            graha: Graha::Sun,
            part: String::from("FIRST"),
            now: String::from("yes"),
        })));
        assert!(fruition.contains(&&crate::Item::of(&said::Fruition {
            graha: Graha::Mars,
            part: String::from("FIRST"),
            now: String::from("no"),
        })));
    }

    #[test]
    fn a_reading_from_the_lagna_says_so() {
        let mut read = reading();
        read.reference = Reference::lagna(Rashi::Leo);
        assert_eq!(
            gochar(&read).items[0],
            crate::Item::of(&said::From {
                from: String::from("LAGNA"),
                rashi: Rashi::Leo,
            })
        );
    }

    #[test]
    fn the_bindus_close_the_plan_only_when_the_reading_carries_them() {
        assert_eq!(
            keyed(&gochar(&reading()), said::Ashtakavarga::KEY),
            Vec::<&crate::Item>::new()
        );
    }
}
