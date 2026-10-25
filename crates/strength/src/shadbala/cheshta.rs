//! The Cheshta bala: a graha's motional strength, from its seeghra kendra for
//! Mars to Saturn and by convention for the Sun and the Moon.

use teistro_core::catalogue::Graha;
use teistro_core::settings::{Cheshta, LuminaryCheshta};

use super::kaala::{ayana, paksha};
use super::{ShadbalaChart, ShadbalaRules, apart, elongation, seat};

/// The Julian day of J2000, the recording engine's elements' epoch.
const J2000: f64 = 2_451_545.0;

/// The Julian day (UT) of Kedarnath Dutt's epoch: 0h local mean time on
/// 1 January 1900 at 76° E, Ujjain.
const DUTT_EPOCH: f64 = 2_415_020.5 - 76.0 / 360.0;

/// Kedarnath Dutt's mean elements as B.V. Raman gives them (Tables IV–IX):
/// the mean Sun, Mars, Jupiter and Saturn's mean longitudes, and Mercury's
/// and Venus's seeghrochchas, each an epoch longitude and a daily motion,
/// degrees, with the correction in degrees for `t` years after 1900.
fn dutt(graha: Graha, days: f64) -> f64 {
    let t = days / 365.25;
    let (epoch, motion, correction) = match graha {
        Graha::Mars => (270.22, 0.524_019, 0.0),
        Graha::Jupiter => (220.04, 0.083_096_7, -(3.33 + 0.0067 * t)),
        Graha::Saturn => (236.74, 0.033_439, 5.0 + 0.001 * t),
        Graha::Mercury => (164.0, 4.092_318, 6.67 - 0.001_33 * t),
        Graha::Venus => (328.51, 1.602_147, -(5.0 + 0.001 * t)),
        // The mean Sun, which is the mean longitude of Mercury and Venus and
        // the seeghrochcha of Mars, Jupiter and Saturn.
        _ => (257.4568, 0.985_602_65, 0.0),
    };
    (epoch + motion * days + correction).rem_euclid(360.0)
}

/// The recording engine's J2000 elements: sidereal longitude and daily motion,
/// degrees, which it takes for Mercury and Venus as both mean and seeghrochcha.
const fn engine_elements(graha: Graha) -> Option<(f64, f64)> {
    match graha {
        Graha::Mars => Some((311.293, 0.524_039)),
        Graha::Mercury => Some((226.704, 4.092_339)),
        Graha::Jupiter => Some((10.7163, 0.083_091)),
        Graha::Venus => Some((157.247, 1.602_136)),
        Graha::Saturn => Some((25.1077, 0.033_461)),
        _ => None,
    }
}

/// The seeghra kendra's strength: a third of the arc from the seeghrochcha to
/// the mean of the mean and true longitudes, folded past 180° (vv. 24 and 25).
fn from_kendra(seeghrochcha: f64, mean: f64, true_longitude: f64) -> f64 {
    let mut arc = (true_longitude - mean).rem_euclid(360.0);
    if arc > 180.0 {
        arc -= 360.0;
    }
    let madhyama = (mean + arc / 2.0).rem_euclid(360.0);
    (apart(seeghrochcha, madhyama) / 3.0).clamp(0.0, 60.0)
}

/// A graha's Cheshta bala.
pub(crate) fn of(graha: Graha, chart: &ShadbalaChart, rules: &ShadbalaRules) -> f64 {
    let true_longitude = seat(&chart.grahas, graha).longitude;
    match (graha, rules.luminary_cheshta) {
        (Graha::Sun | Graha::Moon, LuminaryCheshta::None) => return 0.0,
        (Graha::Sun, _) => return ayana(graha, chart, rules.kranti),
        (Graha::Moon, LuminaryCheshta::AyanaAndPaksha) => return paksha(graha, chart, rules),
        (Graha::Moon, _) => return elongation(chart) / 3.0,
        _ => {}
    }
    if rules.cheshta == Cheshta::Sripati {
        let days = chart.instant - DUTT_EPOCH;
        let mean_sun = dutt(Graha::Sun, days);
        let (seeghrochcha, mean) = if matches!(graha, Graha::Mercury | Graha::Venus) {
            (dutt(graha, days), mean_sun)
        } else {
            (mean_sun, dutt(graha, days))
        };
        return from_kendra(seeghrochcha, mean, true_longitude);
    }
    engine_elements(graha).map_or(0.0, |(epoch, motion)| {
        let days = chart.instant - J2000;
        let mean = (epoch + motion * days).rem_euclid(360.0);
        let seeghrochcha = if matches!(graha, Graha::Mercury | Graha::Venus) {
            mean
        } else {
            // The mean Sun, tropical less the ayanamsha.
            (280.46646 + 0.985_647_4 * days - chart.ayanamsha).rem_euclid(360.0)
        };
        from_kendra(seeghrochcha, mean, true_longitude)
    })
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn dutt_s_elements_move_and_correct_by_the_century() {
        // A Julian century after the epoch, t = 100 years: each epoch, plus
        // the daily motion times 36,525 days, plus the century's correction,
        // worked by hand and reduced past 360°.
        for (graha, expected) in [
            (Graha::Mars, 330.013_975),
            (Graha::Jupiter, 11.146_967_5),
            (Graha::Saturn, 23.199_475),
            (Graha::Mercury, 242.451_95),
            (Graha::Venus, 161.829_175),
            (Graha::Sun, 256.593_591_25),
        ] {
            let found = dutt(graha, 36_525.0);
            assert!((found - expected).abs() < 1e-6, "{graha:?}: {found}");
        }
    }

    #[test]
    fn the_kendra_folds_an_arc_past_half_the_circle() {
        // The true longitude 20° behind the mean: the madhyama is 10° behind,
        // not 170° ahead.
        assert_eq!(from_kendra(0.0, 10.0, 350.0), 0.0);
        assert_eq!(from_kendra(180.0, 10.0, 350.0), 60.0);
        // Exactly half the circle is not folded: the madhyama is 90° ahead.
        assert_eq!(from_kendra(100.0, 10.0, 190.0), 0.0);
        assert_eq!(from_kendra(280.0, 10.0, 190.0), 60.0);
    }
}
