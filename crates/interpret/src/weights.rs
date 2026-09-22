//! What the chart weighs besides the Shadbala: each bhava's strength and
//! each graha's Vimshopaka (`03-design/interpret-composers.md` §4).
//!
//! The fourteenth and fifteenth composers, and the two the section table
//! had recorded as **decisions** rather than as work. Reading the sources
//! dissolved both, which is the third time in one day that a row's stated
//! blocker did not survive its own code.
//!
//! **`BHAVA_BALA` was said to want "what a bhava must reach".** It wants
//! nothing of the kind. A `GrahaShadbala` carries `required_rupas` beside
//! `strong`, which is why [`strength`](crate::strength) says a
//! sufficiency; a `BhavaStrength` carries the four parts and their total
//! and **no requirement at all**. There is no verdict to decline and no
//! threshold to invent: the composer says what the chart weighs, in the
//! virupas the reading is in, and stops. It needed **no new vocabulary**
//! — an ordinal frame and a number, both already in use.
//!
//! **`VIMSHOPAKA` was said to want a knob**, because one composer saying
//! all four schemes "would say the same graha four times". It does say the
//! same graha four times, and so does every per-graha composer; what would
//! make that repetition is a message that did not say **which** scheme it
//! meant. The four are not a catalogue kind — they are four fields of
//! `GrahaVimshopaka` — so the scheme crosses as its key and the message
//! matches on it, the way `sdk.reading.lifeClass` does. Four words, and
//! the knob was never the blocker.

use teistro_intl::messages::sdk::reason;
use teistro_strength::bhava_bala::BhavaBalaReading;
use teistro_strength::vimshopaka::VimshopakaReading;

use crate::Plan;

/// The four schemes a Vimshopaka is scored over, as the message selects on
/// them and in the order a text gives them: the six vargas to the sixteen.
const SCHEMES: [&str; 4] = ["shadvarga", "saptavarga", "dashavarga", "shodashavarga"];

/// Each bhava's strength in virupas, the first house first.
///
/// The reading's own order, which is the wheel's, so a consumer reading a
/// plan walks the houses rather than a ranking. `strength` sorts because a
/// graha's weight is read against the others; a bhava's is read in place.
#[must_use]
pub fn bhava_bala(reading: &BhavaBalaReading) -> Plan {
    let mut plan = Plan::default();
    for bhava in &reading.bhavas {
        plan.say(&reason::BhavaBala {
            bhava: i64::from(bhava.bhava),
            virupas: bhava.virupas,
        });
    }
    plan
}

/// Each graha's Vimshopaka under all four schemes, the catalogue's order.
///
/// All four, because the chart holds all four and each item names the
/// scheme it belongs to. A plan that said one of them would be choosing
/// for the consumer which varga group the reading is worth, which is the
/// dead end the maintainer's rule forbids — and the four together are 28
/// items, fewer than the conditions.
#[must_use]
pub fn vimshopaka(reading: &VimshopakaReading) -> Plan {
    let mut plan = Plan::default();
    for graha in &reading.grahas {
        let scores = [
            graha.shadvarga,
            graha.saptavarga,
            graha.dashavarga,
            graha.shodashavarga,
        ];
        for (scheme, score) in SCHEMES.into_iter().zip(scores) {
            plan.say(&reason::Vimshopaka {
                graha: graha.graha,
                scheme: String::from(scheme),
                score,
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
        reason = "tests fail by panicking"
    )]

    use teistro_core::catalogue::Graha;
    use teistro_intl::Value;
    use teistro_strength::bhava_bala::{BhavaBalaReading, BhavaStrength};
    use teistro_strength::vimshopaka::{GrahaVimshopaka, VimshopakaReading};

    use super::{SCHEMES, bhava_bala, vimshopaka};
    use crate::KEYS;

    fn bhava(number: u8, virupas: f64) -> BhavaStrength {
        BhavaStrength {
            bhava: number,
            lord: Graha::Mars,
            adhipati: 0.0,
            dig: 0.0,
            drishti: 0.0,
            special: 0.0,
            virupas,
        }
    }

    /// The houses are said in the wheel's order and not ranked, because a
    /// bhava's weight is read in place.
    #[test]
    fn the_houses_are_said_in_the_wheels_order() {
        let reading = BhavaBalaReading {
            rules: teistro_strength::bhava_bala::BhavaBalaRules::BPHS,
            bhavas: vec![bhava(1, 100.0), bhava(2, 500.0), bhava(3, 250.0)],
        };
        let plan = bhava_bala(&reading);
        let said: Vec<Option<&Value>> = plan
            .items
            .iter()
            .map(|item| item.params.get("bhava"))
            .collect();
        assert_eq!(
            said,
            [
                Some(&Value::Int(1)),
                Some(&Value::Int(2)),
                Some(&Value::Int(3))
            ],
            "the weakest is not first, and the strongest is not either"
        );
        for key in plan.keys() {
            assert!(KEYS.contains(&key), "`{key}` is not in KEYS");
        }
    }

    /// It says the weight and never a verdict: a `BhavaStrength` carries
    /// no requirement, so there is nothing to be short of.
    #[test]
    fn a_bhava_is_weighed_and_not_judged() {
        let reading = BhavaBalaReading {
            rules: teistro_strength::bhava_bala::BhavaBalaRules::BPHS,
            bhavas: vec![bhava(1, 100.0)],
        };
        let plan = bhava_bala(&reading);
        assert_eq!(plan.len(), 1, "one item a bhava, not a score and a verdict");
        let written = serde_json::to_string(&plan).unwrap();
        for verdict in ["strong", "weak", "required", "reaches"] {
            assert!(!written.contains(verdict), "{verdict} in {written}");
        }
    }

    /// All four schemes, each naming itself, so four items about one graha
    /// are four facts rather than one repeated.
    #[test]
    fn every_scheme_names_itself() {
        let reading = VimshopakaReading {
            scoring: teistro_core::settings::Vimshopaka::Bphs,
            grahas: vec![GrahaVimshopaka {
                graha: Graha::Sun,
                shadvarga: 1.0,
                saptavarga: 2.0,
                dashavarga: 3.0,
                shodashavarga: 4.0,
            }],
        };
        let plan = vimshopaka(&reading);
        assert_eq!(plan.len(), 4);
        let schemes: Vec<Option<&Value>> = plan
            .items
            .iter()
            .map(|item| item.params.get("scheme"))
            .collect();
        let expected: Vec<Option<Value>> = SCHEMES
            .iter()
            .map(|scheme| Some(Value::Str((*scheme).to_owned())))
            .collect();
        assert_eq!(
            schemes,
            expected.iter().map(Option::as_ref).collect::<Vec<_>>()
        );
        // And the score said with each is that scheme's own.
        assert_eq!(plan.items[0].params.get("score"), Some(&Value::Num(1.0)));
        assert_eq!(plan.items[3].params.get("score"), Some(&Value::Num(4.0)));
        for key in plan.keys() {
            assert!(KEYS.contains(&key), "`{key}` is not in KEYS");
        }
    }
}
