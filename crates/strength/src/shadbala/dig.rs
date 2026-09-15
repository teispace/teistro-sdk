//! The Dig bala: a graha's strength by direction, a third of its arc from the
//! kendra where it is powerless.

use teistro_core::catalogue::Graha;
use teistro_core::settings::DigKendras;

use super::{ShadbalaChart, apart, seat};

/// A graha's Dig bala, 0 to 60 (v. 7).
pub(crate) fn of(graha: Graha, chart: &ShadbalaChart, rule: DigKendras) -> f64 {
    // The house where the graha's direction is full: the Sun and Mars the
    // tenth, Mercury and Jupiter the first, Saturn the seventh, the Moon and
    // Venus the fourth.
    let (house, angle) = match graha {
        Graha::Sun | Graha::Mars => (10.0, chart.midheaven),
        Graha::Mercury | Graha::Jupiter => (1.0, chart.ascendant),
        Graha::Saturn => (7.0, chart.ascendant + 180.0),
        _ => (4.0, chart.midheaven + 180.0),
    };
    let full = if rule == DigKendras::Angles {
        angle
    } else {
        chart.ascendant + (house - 1.0) * 30.0
    };
    apart(seat(&chart.grahas, graha).longitude, full + 180.0) / 3.0
}
