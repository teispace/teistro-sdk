//! The Yuddha bala: what two grahas at war win and lose.

use teistro_core::catalogue::Graha;

use super::{GrahaShadbala, ShadbalaChart, apart, seat};

/// The diameters of the planets' discs, arcseconds (B.V. Raman, Art. 77).
const DISCS: [(Graha, f64); 5] = [
    (Graha::Mars, 9.4),
    (Graha::Mercury, 6.6),
    (Graha::Jupiter, 190.4),
    (Graha::Venus, 16.6),
    (Graha::Saturn, 158.0),
];

/// Applies the Yuddha bala of every war in a chart to its grahas' Kaala.
///
/// Two of Mars to Saturn are at war within a degree; the one of lesser
/// longitude wins; the difference of their Sthana, Dig and Kaala up to the
/// Hora bala, over the difference of their discs, is the victor's gain and the
/// vanquished's loss (Arts. 76–77). Every war is weighed on the strengths
/// before any Yuddha bala is applied.
pub(crate) fn apply(chart: &ShadbalaChart, grahas: &mut [GrahaShadbala]) {
    let weight = |g: &GrahaShadbala| g.sthana.total() + g.dig + g.kaala.to_hora();
    let mut gains: Vec<(Graha, f64)> = Vec::new();
    for (i, (first, first_disc)) in DISCS.iter().enumerate() {
        for (second, second_disc) in DISCS.iter().skip(i + 1) {
            let (a, b) = (
                seat(&chart.grahas, *first).longitude,
                seat(&chart.grahas, *second).longitude,
            );
            let discs = (first_disc - second_disc).abs();
            if apart(a, b) >= 1.0 || discs == 0.0 {
                continue;
            }
            let strength = |graha: Graha| grahas.iter().find(|g| g.graha == graha).map(weight);
            let (Some(sa), Some(sb)) = (strength(*first), strength(*second)) else {
                continue;
            };
            // The lesser longitude, across the zodiac's start: the one the
            // other is just ahead of.
            let first_wins = (b - a).rem_euclid(360.0) < 180.0;
            let (victor, vanquished) = if first_wins {
                (*first, *second)
            } else {
                (*second, *first)
            };
            let yuddha = (sa - sb).abs() / discs;
            gains.push((victor, yuddha));
            gains.push((vanquished, -yuddha));
        }
    }
    for (graha, gain) in gains {
        if let Some(entry) = grahas.iter_mut().find(|g| g.graha == graha) {
            entry.kaala.yuddha += gain;
        }
    }
}
