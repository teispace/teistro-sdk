//! What the Ashtakavarga says of a chart: each graha's bindus where it
//! stands, and each sign's sarvashtakavarga
//! (`03-design/interpret-composers.md` §4).
//!
//! The sixteenth composer, and the row the section table called a
//! **reading** decision — the only one of the four so called that stayed
//! one when its sources were read. An Ashtakavarga is eighty-four
//! numbers and a dozen more, and the question was never whether they
//! cross: they have crossed as `Document.ashtakavarga` since the module
//! landed. It is *which of them a sentence is worth*.
//!
//! **Two readings, and no third.** A text quotes two things of an
//! Ashtakavarga: how many bindus a graha has **in the sign it stands
//! in**, and what a sign holds in the **sarvashtakavarga**. The first is
//! why the composer takes two inputs — the reading is indexed by sign
//! and knows nothing of where the grahas are, so the chart has to stand
//! beside it. Nineteen items: seven and twelve, with the sarva said once
//! rather than repeated inside each graha's sentence.
//!
//! **No band, and no verdict.** This is `bhava_bala`'s rule and not
//! `strength`'s: a `GrahaShadbala` carries `required_rupas`, so the
//! composer can say a sufficiency; an `AshtakavargaReading` carries no
//! threshold at all. Bindus run 0 to 8 and a sign's sarva is part of a
//! fixed 337 — both are scales a reader already has — so inventing
//! *weak*, *middling* and *strong* would be making up a rule rather than
//! saying one, which is exactly what held `shadbala-strength`'s four
//! bands out of the state corpus.
//!
//! **What is left**, and it is deliberate. The `trikona` and `reduced`
//! sums and the three pindas are all **ruleset-dependent** — the
//! measured page falsifies the text's reading of each of them against
//! the corpus on every chart — so a sentence saying one would have to
//! name the ruleset it was reduced under to mean anything, which is a
//! different sentence and a decision of its own. And the sarva is said
//! by **sign**, which is what the reading is indexed by; saying it by
//! bhava would fold in the house system, and which house a sign is
//! belongs to `houses`.

use teistro_core::catalogue::Rashi;
use teistro_intl::messages::sdk::reason;
use teistro_rules::{Body, RuleChart};
use teistro_strength::ashtakavarga::AshtakavargaReading;

use crate::Plan;

/// Each graha's bindus where it stands, then each sign's sarva.
///
/// The grahas in the reading's own order, which is the catalogue's from
/// the Sun to Saturn, and the signs in the zodiac's. A graha the chart
/// does not place is skipped rather than guessed at: the reading and the
/// chart come from one founding, so it cannot happen, and a composer
/// that would have to invent a sign to say a number says nothing
/// instead.
#[must_use]
pub fn ashtakavarga(reading: &AshtakavargaReading, chart: &RuleChart) -> Plan {
    let mut plan = Plan::default();
    for graha in &reading.grahas {
        let Some(at) = chart.placements.get(Body::Graha(graha.graha).index()) else {
            continue;
        };
        let Some(bindus) = graha.bindus.get(at.sign as usize % Rashi::ALL.len()) else {
            continue;
        };
        plan.say(&reason::Ashtakavarga {
            graha: graha.graha,
            bindus: i64::from(*bindus),
            rashi: at.sign,
        });
    }
    for (rashi, sarva) in Rashi::ALL.into_iter().zip(reading.sarva) {
        plan.say(&reason::Sarvashtakavarga {
            rashi,
            bindus: i64::from(sarva),
        });
    }
    plan
}
