//! Whose each of the twelve bhavas is: the lord of every house, in order
//! (`03-design/interpret-composers.md` §4).
//!
//! The second composer over a **section** — it reads `Document.houses`,
//! which `ChartRequest::with_houses` asks for — and the first to say what a
//! graha **rules** rather than where it stands. Those are different facts
//! about the same graha, and until now a plan carried only the second.
//!
//! Like [`placements`](crate::placements) and [`strength`](crate::strength)
//! it adds no message of its own: `sdk.reason.lordship` is carried by both
//! strict locales, translated by hand, and was read by nothing.
//!
//! What it cannot say is counted rather than hidden, and here the silence is
//! larger than the other composers carry. A bhava knows the sign it falls
//! in, which third of the wheel it stands in, and whether it is a trine, a
//! house of difficulty or one that grows better with time; a chart knows
//! which bodies fall in a different house under the chalit, which house
//! systems were used, and whether the division came back degenerate. **No
//! locale carries a message for any of it**, so the plan claims none of it
//! and the measured page counts what was left unsaid. Writing an English
//! sentence and machine-translating it is the stub this project refuses.

use teistro_houses::chart::Bhava;
use teistro_intl::messages::sdk::reason;

use crate::Plan;

/// The lord of each of the twelve bhavas, first house first.
///
/// The order is the houses' own — 1 to 12 — and not a ranking, so the same
/// chart always gives the same plan. The lord is the lord of the sign the
/// house's **middle** falls in, which is the bhava's own reading of itself:
/// under an unequal division a house can begin in one sign and be centred in
/// another, and this composer does not second-guess which one the tradition
/// means.
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_houses::chart::Bhava;
/// use teistro_houses::classify::{Quadrant, lord_of};
///
/// // An Aries-rising whole-sign chart: house n falls in the nth sign.
/// let bhavas: [Bhava; 12] = core::array::from_fn(|i| Bhava {
///     number: u8::try_from(i + 1).unwrap(),
///     sign: Rashi::ALL[i],
///     lord: lord_of(Rashi::ALL[i]),
///     madhya_deg: 0.0,
///     sandhi_deg: 0.0,
///     quadrant: Quadrant::Kendra,
/// });
///
/// let plan = teistro_interpret::houses(&bhavas);
/// assert_eq!(plan.len(), 12);
/// assert_eq!(plan.items[0].key, "sdk.reason.lordship");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[must_use]
pub fn houses(bhavas: &[Bhava; 12]) -> Plan {
    let mut plan = Plan::default();
    for bhava in bhavas {
        plan.say(&reason::Lordship {
            bhava: i64::from(bhava.number),
            graha: bhava.lord,
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

    use teistro_core::catalogue::{Graha, Rashi};
    use teistro_houses::classify::{Quadrant, lord_of};

    use super::*;
    use crate::{Item, KEYS};

    /// The twelve houses of an Aries-rising whole-sign chart, which is the
    /// one shape every recorded chart of the corpus takes.
    fn bhavas() -> [Bhava; 12] {
        from_signs(&core::array::from_fn(|i| Rashi::ALL[i]))
    }

    /// Twelve bhavas over the signs given, each lorded by its sign's lord.
    ///
    /// Only the fields the composer reads carry meaning; the rest are the
    /// record's own zeroes, so a field added to `Bhava` does not reach this
    /// file with a number it would have to invent.
    fn from_signs(signs: &[Rashi; 12]) -> [Bhava; 12] {
        core::array::from_fn(|i| Bhava {
            number: u8::try_from(i + 1).unwrap(),
            sign: signs[i],
            lord: lord_of(signs[i]),
            madhya_deg: 0.0,
            sandhi_deg: 0.0,
            quadrant: Quadrant::Kendra,
        })
    }

    #[test]
    fn every_house_is_said_in_its_own_order() {
        let plan = houses(&bhavas());
        assert_eq!(plan.len(), 12);
        assert_eq!(
            plan.items[0],
            Item::of(&reason::Lordship {
                bhava: 1,
                graha: lord_of(Rashi::Aries),
            })
        );
        assert_eq!(
            plan.items.last().unwrap(),
            &Item::of(&reason::Lordship {
                bhava: 12,
                graha: lord_of(Rashi::Pisces),
            })
        );
    }

    /// A graha lords two signs, so a chart says its name twice — which is
    /// the fact, not a duplicate to collapse: the second and the ninth are
    /// different houses however they are ruled.
    #[test]
    fn a_graha_that_rules_twice_is_said_twice() {
        let plan = houses(&bhavas());
        let lords: Vec<&teistro_intl::Value> = plan
            .items
            .iter()
            .filter_map(|item| item.params.get("graha"))
            .collect();
        assert_eq!(lords.len(), 12);
        let jupiter = teistro_intl::Value::catalogued(Graha::Jupiter);
        assert_eq!(
            lords.iter().filter(|lord| ***lord == jupiter).count(),
            2,
            "Jupiter rules Sagittarius and Pisces"
        );
    }

    /// The lord follows the sign, so a chart rising in another sign says
    /// another set of lords: the composer reads the bhava and holds no
    /// table of its own.
    #[test]
    fn the_lords_follow_the_signs() {
        let shifted = from_signs(&core::array::from_fn(|i| Rashi::ALL[(i + 3) % 12]));
        let plan = houses(&shifted);
        assert_eq!(
            plan.items[0],
            Item::of(&reason::Lordship {
                bhava: 1,
                graha: lord_of(Rashi::Cancer),
            })
        );
    }

    /// The bhava carries its sign, its quadrant and its cusps, and no locale
    /// carries a message for any of them, so the plan claims none.
    #[test]
    fn it_does_not_say_what_no_locale_can_say() {
        let mut marked = bhavas();
        for bhava in &mut marked {
            bhava.madhya_deg = 123.456;
            bhava.sandhi_deg = 78.9;
            bhava.quadrant = Quadrant::Panapara;
        }
        let written = serde_json::to_string(&houses(&marked)).unwrap();
        for claim in [
            "madhya", "sandhi", "quadrant", "Panapara", "123.456", "rashi",
        ] {
            assert!(!written.contains(claim), "`{claim}` is in {written}");
        }
    }

    #[test]
    fn it_emits_only_listed_keys() {
        for key in houses(&bhavas()).keys() {
            assert!(KEYS.contains(&key), "`{key}` is not in KEYS");
        }
    }

    #[test]
    fn a_plan_is_the_same_bytes_every_time() {
        let once = serde_json::to_string(&houses(&bhavas())).unwrap();
        let twice = serde_json::to_string(&houses(&bhavas())).unwrap();
        assert_eq!(once, twice);
        let read: Plan = serde_json::from_str(&once).unwrap();
        assert_eq!(read, houses(&bhavas()));
    }
}
