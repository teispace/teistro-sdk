//! What the chart weighs: each graha's Shadbala, strongest first
//! (`03-design/interpret-composers.md` §4).
//!
//! The third kind of composer, and the first over a **section** rather than
//! over the chart's placements or a rule's answer: it reads
//! `Document.shadbala`, which a request asks for by name, and needs no rules
//! beside it.
//!
//! Like [`placements`](crate::placements) it adds no message of its own —
//! `sdk.reason.strength.score` is carried by both strict locales, translated
//! by hand — so a plan of strengths renders in English and in Nepali the day
//! it is written.
//!
//! What it cannot say is counted rather than hidden. The Shadbala says of
//! each graha whether it reaches the rupas its text requires, and no locale
//! carries a message for *that*: a graha's `required_rupas` and its `strong`
//! cross in the document and are absent from the plan until a message is
//! written and translated. Saying "strong" in a locale that has no word for
//! it here would be the machine-translated stub the project refuses.
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
        assert_eq!(plan.len(), 7);
        assert_eq!(
            plan.items[0],
            Item::of(&reason::strength::Score {
                graha: Graha::Saturn,
                score: 7.0,
            })
        );
        assert_eq!(
            plan.items.last().unwrap(),
            &Item::of(&reason::strength::Score {
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
        assert_eq!(said[6], "graha.SATURN");
    }

    /// Whether a graha reaches its required rupas has no message in any
    /// locale, so the plan does not claim it.
    #[test]
    fn it_does_not_say_what_no_locale_can_say() {
        let mut read = reading();
        for graha in &mut read.grahas {
            graha.strong = true;
            graha.required_rupas = 5.0;
        }
        let written = serde_json::to_string(&strength(&read)).unwrap();
        // The document carries both; the plan claims neither.
        assert!(!written.contains("strong"), "{written}");
        assert!(!written.contains("required"), "{written}");
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
