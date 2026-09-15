//! The Drik bala: a graha's strength from the drishtis it receives.

use teistro_aspect::drishti::quarters;
use teistro_aspect::sphuta;
use teistro_core::catalogue::{Graha, Nature};
use teistro_core::settings::Drik;

use super::{ShadbalaChart, ShadbalaRules, is_benefic, seat};
use crate::ashtakavarga::GRAHAS;

/// A graha's Drik bala, which may be negative (v. 19).
pub(crate) fn of(graha: Graha, chart: &ShadbalaChart, rules: &ShadbalaRules) -> f64 {
    let target = seat(&chart.grahas, graha);
    let others = GRAHAS
        .iter()
        .zip(chart.grahas)
        .filter(|(from, _)| **from != graha);
    if rules.drik == Drik::Full {
        // The recording engine: the whole-sign graded glance from each house,
        // Mercury a malefic, bounded at ±60.
        let sum: f64 = others
            .map(|(from, at)| {
                let count = (target.house + 12 - at.house) % 12 + 1;
                let virupas = f64::from(quarters(*from, count).virupas());
                let benefic =
                    from.attributes().descriptors.map(|d| d.nature) == Some(Nature::Benefic);
                if benefic { virupas } else { -virupas }
            })
            .sum();
        return sum.clamp(-60.0, 60.0);
    }
    others
        .map(|(from, at)| {
            let virupas = sphuta::drishti(*from, at.longitude, target.longitude);
            let quarter = if is_benefic(*from, chart, rules.benefics) {
                virupas / 4.0
            } else {
                -virupas / 4.0
            };
            let full = rules.drik == Drik::QuarterWithJupiterMercury
                && matches!(from, Graha::Jupiter | Graha::Mercury);
            quarter + if full { virupas } else { 0.0 }
        })
        .sum()
}
