//! The span of life the dashas give, Nakshatra dasayus (*Jataka Parijata*
//! ch. 5 v. 27, C309).
//!
//! "The Ayurdaya whose initial portion consists of the years due to the
//! unexpired ghatikas of the yogatara at a birth, whereof the lord is one of
//! the nine planets from the Sun onwards — this Ayurdaya is called Dasayus
//! or more commonly Nakshatradasayus" (pp. 256 to 257). The years are ch. 18
//! v. 3's, counted from Krittika in nines: the Sun 6, the Moon 10, Mars 7,
//! Rahu 18, Jupiter 16, Saturn 19, Mercury 17, Ketu 7 and Venus 20, which
//! are Vimshottari's. The span is the first lord's balance and the eight
//! dashas after it: the whole cycle of 120 years less what of the first had
//! run before birth.
//!
//! The balance is the SDK's own, measured as the chart's settings measure
//! it; "the unexpired ghatikas" read as time is its temporal method.

use serde::Serialize;
use teistro_core::catalogue::Graha;
use teistro_dasha::{VIMSHOTTARI_LORDS, VIMSHOTTARI_YEARS};

/// The span the dashas give from birth.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Dasayus {
    /// The lord of the Moon's nakshatra, whose dasha runs at birth.
    pub first_lord: Graha,
    /// The fraction of its dasha still to run at birth, 0 to 1.
    pub remaining: f64,
    /// Those years: the balance at birth.
    pub balance: f64,
    /// The balance and the eight dashas after it, in years of 360 days.
    pub years: f64,
}

/// Dasayus from the first lord and the fraction of its dasha still to run;
/// none for a graha that lords no Vimshottari dasha.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_rules::longevity::dasayus;
///
/// // Born with a quarter of Venus's twenty years to run: five, and the
/// // hundred the eight after her give.
/// let span = dasayus(Graha::Venus, 0.25).unwrap();
/// assert!((span.balance - 5.0).abs() < 1e-12 && (span.years - 105.0).abs() < 1e-12);
/// ```
#[must_use]
pub fn dasayus(first_lord: Graha, remaining: f64) -> Option<Dasayus> {
    let lord = VIMSHOTTARI_LORDS
        .iter()
        .find(|lord| lord.graha == first_lord)?;
    let remaining = remaining.clamp(0.0, 1.0);
    let full = f64::from(lord.years);
    let balance = full * remaining;
    Some(Dasayus {
        first_lord,
        remaining,
        balance,
        years: f64::from(VIMSHOTTARI_YEARS) - full + balance,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_span_runs_from_a_whole_cycle_to_a_cycle_less_its_first_dasha() {
        // ch. 18 v. 3's years, which are Vimshottari's.
        let years = [
            (Graha::Sun, 6),
            (Graha::Moon, 10),
            (Graha::Mars, 7),
            (Graha::Rahu, 18),
            (Graha::Jupiter, 16),
            (Graha::Saturn, 19),
            (Graha::Mercury, 17),
            (Graha::Ketu, 7),
            (Graha::Venus, 20),
        ];
        for (graha, full) in years {
            let span = |remaining| dasayus(graha, remaining).map(|span| span.years);
            assert!(
                span(1.0).is_some_and(|years| (years - 120.0).abs() < 1e-12),
                "{graha:?}"
            );
            assert!(
                span(0.0).is_some_and(|years| (years - (120.0 - f64::from(full))).abs() < 1e-12),
                "{graha:?}"
            );
        }
    }
}
