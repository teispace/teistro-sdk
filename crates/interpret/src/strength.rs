//! What the chart weighs: each graha's Shadbala, strongest first
//! (`03-design/interpret-composers.md` §4).
//!
//! The third kind of composer, and the first over a **section** rather than
//! over the chart's placements or a rule's answer: it reads
//! `Document.shadbala`, which a request asks for by name, and needs no rules
//! beside it.
//!
//! `sdk.reason.strength.score` was carried by both strict locales before
//! this composer was written; `sdk.reason.strength.meets` is its own, two
//! sentences a locale.
//!
//! It says **two** things of each graha, because they are two facts: what it
//! weighs (`score`) and whether that is enough (`meets`). The Shadbala
//! carries both — `rupas` beside `required_rupas` and `strong` — and for
//! four composers the second crossed in the document and was absent from
//! the plan, counted on the measured page at 341 of 497 grahas reaching
//! their requirement.
//!
//! **It says the requirement and not a verdict.** The message names the
//! rupas the text asks for and whether the graha reaches them, which is
//! what the Shadbala computes; it does not say "strong", which is a word
//! the tradition spends carefully and a machine translation of it would be
//! the stub the project refuses. The two items sit together, score then
//! sufficiency, so a consumer filtering to `score` still reads the ranking
//! in the items' order.
//!
//! `sdk.reason.strength.rank` is **not** emitted, and not because it is
//! missing: it renders an ordinal alone — `1st`, `१लो` — which is a
//! fragment a consumer formats with, not a sentence a plan says. The rank is
//! carried by the order of the items instead, which is what an ordered plan
//! is for.

use teistro_intl::messages::sdk::reason;
use teistro_strength::shadbala::ShadbalaReading;

use crate::Plan;

/// Each graha's Shadbala in rupas, the strongest first.
///
/// Ties keep the reading's own order, which is the catalogue's — the Sun to
/// Saturn — so the same chart always gives the same plan.
#[must_use]
pub fn strength(shadbala: &ShadbalaReading) -> Plan {
    let mut ranked: Vec<&teistro_strength::shadbala::GrahaShadbala> =
        shadbala.grahas.iter().collect();
    // `sort_by` is stable, so grahas that weigh the same keep the reading's
    // order and the plan stays the same bytes for the same chart.
    ranked.sort_by(|a, b| b.rupas.total_cmp(&a.rupas));
    let mut plan = Plan::default();
    for graha in ranked {
        plan.say(&reason::strength::Score {
            graha: graha.graha,
            score: graha.rupas,
        });
        plan.say(&reason::strength::Meets {
            graha: graha.graha,
            required: graha.required_rupas,
            reaches: String::from(if graha.strong { REACHES } else { SHORT }),
        });
    }
    plan
}

/// What `sdk.reason.strength.meets` selects on when a graha reaches the
/// rupas its text requires, and when it does not.
///
/// A word and not a boolean, because the message is a `.match` like every
/// other selector in the packs, and a locale reads the arm it needs. The
/// two are named here so the composer and the message cannot drift: a
/// third state would be a third arm and a third constant.
const REACHES: &str = "yes";
/// The other arm, which the message reaches through its catch-all.
const SHORT: &str = "no";

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        clippy::indexing_slicing,
        reason = "tests unwrap what they build and fail by panicking"
    )]

    use teistro_core::catalogue::Graha;

    use super::*;
    use crate::{Item, KEYS};

    /// A reading whose rupas rise with the catalogue's order, so the plan
    /// must come back reversed.
    fn reading() -> ShadbalaReading {
        rupas(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0])
    }

    /// A reading of the seven grahas, Sun to Saturn, weighing what is given.
    ///
    /// Only the two fields the composer reads are set; the rest are the
    /// records' own zeroes, so a component added to the Shadbala does not
    /// reach this file.
    fn rupas(each: &[f64; 7]) -> ShadbalaReading {
        ShadbalaReading {
            rules: teistro_strength::shadbala::ShadbalaRules::BPHS,
            grahas: Graha::ALL
                .into_iter()
                .take(7)
                .zip(each)
                .map(|(graha, rupas)| teistro_strength::shadbala::GrahaShadbala {
                    graha,
                    sthana: teistro_strength::shadbala::SthanaBala::default(),
                    dig: 0.0,
                    kaala: teistro_strength::shadbala::KaalaBala::default(),
                    cheshta: 0.0,
                    naisargika: 0.0,
                    drik: 0.0,
                    virupas: rupas * 60.0,
                    rupas: *rupas,
                    required_rupas: 0.0,
                    strong: false,
                    ishta: 0.0,
                    kashta: 0.0,
                    subha_rashmi: 0.0,
                    ashubha_rashmi: 0.0,
                })
                .collect(),
        }
    }

    #[test]
    fn the_strongest_graha_is_said_first() {
        let plan = strength(&reading());
        assert_eq!(plan.len(), 7 * 2, "a score and a sufficiency each");
        assert_eq!(
            plan.items[0],
            Item::of(&reason::strength::Score {
                graha: Graha::Saturn,
                score: 7.0,
            })
        );
        assert_eq!(
            plan.items[plan.len() - 2],
            Item::of(&reason::strength::Score {
                graha: Graha::Sun,
                score: 1.0,
            })
        );
    }

    /// Two grahas of equal weight keep the reading's order, so a plan does
    /// not depend on the sort's accidents.
    #[test]
    fn a_tie_keeps_the_catalogue_s_order() {
        let plan = strength(&rupas(&[5.0; 7]));
        let said: Vec<&str> = plan
            .items
            .iter()
            .filter_map(|item| match item.params.get("graha") {
                Some(teistro_intl::Value::Entity(key)) => Some(key.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(said[0], "graha.SUN");
        assert_eq!(said[13], "graha.SATURN");
    }

    /// Whether a graha reaches its required rupas is the second fact the
    /// Shadbala carries, and both arms of the message are exercised: the
    /// catch-all is the one a plan reaches when a graha falls short, and a
    /// test over a reading where every graha is strong would never take it.
    #[test]
    fn it_says_the_requirement_and_whether_each_graha_reaches_it() {
        let mut read = reading();
        for (at, graha) in read.grahas.iter_mut().enumerate() {
            graha.required_rupas = 5.0;
            graha.strong = graha.rupas >= graha.required_rupas;
            assert_eq!(graha.strong, at >= 4, "the four weakest fall short");
        }
        let plan = strength(&read);
        assert_eq!(
            plan.items[1],
            Item::of(&reason::strength::Meets {
                graha: Graha::Saturn,
                required: 5.0,
                reaches: String::from(REACHES),
            }),
            "the strongest reaches it"
        );
        assert_eq!(
            plan.items.last().unwrap(),
            &Item::of(&reason::strength::Meets {
                graha: Graha::Sun,
                required: 5.0,
                reaches: String::from(SHORT),
            }),
            "the weakest does not"
        );
        // It says the requirement, never a verdict: no locale is asked for
        // a word like "strong", which is the stub the project refuses.
        let written = serde_json::to_string(&plan).unwrap();
        assert!(!written.contains("strong"), "{written}");
    }

    #[test]
    fn it_emits_only_listed_keys() {
        for key in strength(&reading()).keys() {
            assert!(KEYS.contains(&key), "`{key}` is not in KEYS");
        }
    }

    #[test]
    fn a_plan_is_the_same_bytes_every_time() {
        let once = serde_json::to_string(&strength(&reading())).unwrap();
        let twice = serde_json::to_string(&strength(&reading())).unwrap();
        assert_eq!(once, twice);
        let read: Plan = serde_json::from_str(&once).unwrap();
        assert_eq!(read, strength(&reading()));
    }
}
