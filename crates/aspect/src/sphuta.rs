//! The sphuta drishti: a graha's aspect on a point, measured by degree.
//!
//! Sripati's construction, as B.V. Raman gives it (*Graha and Bhava Balas*,
//! Arts. 110–115) and says Parashara gives too. The aspect angle is the
//! aspected longitude less the aspecting one, and the drishti in virupas
//! rises from nothing at 30° to 15 at 60° and 45 at 90°, falls to 30 at 120°
//! and nothing at 150°, jumps to rise to the full 60 at 180°, and falls
//! evenly to nothing at 300°. Mars, Jupiter and Saturn add 15, 30 and 45
//! over the two 30° spans of their special aspects.
//!
//! At the middle of every house it is exactly the whole-sign table's value
//! ([`crate::drishti::Strength::virupas`]), the special aspects' full 60
//! included; between the middles it measures the degrees the whole-sign table
//! rounds away (crux C45).
//!
//! ```
//! use teistro_aspect::sphuta::drishti;
//! use teistro_core::catalogue::Graha;
//!
//! // Every graha looks fully at the point opposite it.
//! assert_eq!(drishti(Graha::Sun, 10.0, 190.0), 60.0);
//! // Nothing within 30° ahead of it or 60° behind it.
//! assert_eq!(drishti(Graha::Sun, 10.0, 25.0), 0.0);
//! // Jupiter's fifth: the general 30° less the angle past 120°, and its 30.
//! assert_eq!(drishti(Graha::Jupiter, 0.0, 130.0), 20.0 + 30.0);
//! ```

use teistro_core::catalogue::Graha;

/// The special aspects: each graha's two spans of aspect angle, degrees, and
/// what each adds, virupas (Raman, Art. 115).
/// A special aspect: the graha, its two spans of aspect angle, and what it
/// adds.
type Visesha = (Graha, [(f64, f64); 2], f64);

const VISESHA: [Visesha; 3] = [
    (Graha::Mars, [(90.0, 120.0), (210.0, 240.0)], 15.0),
    (Graha::Jupiter, [(120.0, 150.0), (240.0, 270.0)], 30.0),
    (Graha::Saturn, [(60.0, 90.0), (270.0, 300.0)], 45.0),
];

/// The general drishti at an aspect angle, degrees, in virupas.
#[must_use]
pub fn general(angle: f64) -> f64 {
    let k = angle.rem_euclid(360.0);
    if k < 30.0 {
        0.0
    } else if k < 60.0 {
        (k - 30.0) / 2.0
    } else if k < 90.0 {
        k - 60.0 + 15.0
    } else if k < 120.0 {
        (120.0 - k) / 2.0 + 30.0
    } else if k < 150.0 {
        150.0 - k
    } else if k < 180.0 {
        (k - 150.0) * 2.0
    } else if k < 300.0 {
        (300.0 - k) / 2.0
    } else {
        0.0
    }
}

/// What a graha's special aspect adds at an aspect angle, in virupas; nothing
/// for a graha without one.
#[must_use]
pub fn visesha(graha: Graha, angle: f64) -> f64 {
    let k = angle.rem_euclid(360.0);
    VISESHA
        .iter()
        .find(|(who, _, _)| *who == graha)
        .filter(|(_, spans, _)| spans.iter().any(|(from, to)| (*from..*to).contains(&k)))
        .map_or(0.0, |(_, _, adds)| *adds)
}

/// A graha's drishti, in virupas, from its longitude on another longitude:
/// the general value and its special aspect's.
#[must_use]
pub fn drishti(graha: Graha, aspecting: f64, aspected: f64) -> f64 {
    let angle = aspected - aspecting;
    general(angle) + visesha(graha, angle)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp, reason = "the construction's corners are exact")]

    use super::*;

    /// Degrees and minutes.
    fn dm(degrees: f64, minutes: f64) -> f64 {
        degrees + minutes / 60.0
    }

    #[test]
    fn the_general_drishti_is_continuous_but_for_its_one_jump() {
        for (angle, value) in [
            (30.0, 0.0),
            (60.0, 15.0),
            (90.0, 45.0),
            (120.0, 30.0),
            (150.0, 0.0),
            (180.0, 60.0),
            (240.0, 30.0),
            (300.0, 0.0),
            (330.0, 0.0),
        ] {
            assert_eq!(general(angle), value, "{angle}");
        }
        // Approaching 150° from either side: nothing, then rising again.
        assert!(general(149.999) < 0.01 && general(150.001) < 0.01);
        assert_eq!(general(-180.0), 60.0);
    }

    #[test]
    fn the_special_aspects_add_over_their_own_spans_alone() {
        assert_eq!(visesha(Graha::Mars, 100.0), 15.0);
        assert_eq!(visesha(Graha::Mars, 225.0), 15.0);
        assert_eq!(visesha(Graha::Jupiter, 250.0), 30.0);
        assert_eq!(visesha(Graha::Saturn, 75.0), 45.0);
        assert_eq!(visesha(Graha::Saturn, 100.0), 0.0);
        assert_eq!(visesha(Graha::Venus, 100.0), 0.0);
        assert_eq!(visesha(Graha::Rahu, 100.0), 0.0);
    }

    /// Every graha, every house: at the middle of the house the sphuta
    /// drishti is the whole-sign table's value.
    #[test]
    fn at_each_house_s_middle_it_is_the_whole_sign_table() {
        use crate::drishti::quarters;
        for graha in [
            Graha::Sun,
            Graha::Moon,
            Graha::Mars,
            Graha::Mercury,
            Graha::Jupiter,
            Graha::Venus,
            Graha::Saturn,
        ] {
            for house in 1..=12_u8 {
                let middle = f64::from(house - 1) * 30.0;
                let table = f64::from(quarters(graha, house).virupas());
                assert_eq!(
                    drishti(graha, 15.0, 15.0 + middle),
                    table,
                    "{graha:?} {house}"
                );
            }
        }
    }

    /// Raman's Standard Horoscope, Example 54: each drishti his tables give,
    /// within the rounding of his degrees-and-minutes arithmetic.
    #[test]
    fn raman_s_worked_drishtis_reproduce() {
        let (sun, moon, mars) = (dm(180.0, 54.0), dm(311.0, 17.0), dm(229.0, 31.0));
        let (jupiter, venus, saturn) = (dm(84.0, 1.0), dm(171.0, 10.0), dm(124.0, 23.0));
        for (who, from, on, raman) in [
            (Graha::Jupiter, jupiter, sun, 41.50),
            (Graha::Moon, moon, sun, 35.20),
            (Graha::Saturn, saturn, sun, 13.25),
            (Graha::Jupiter, jupiter, mars, 30.00 + 4.50),
            (Graha::Moon, moon, mars, 10.88),
            (Graha::Venus, venus, mars, 14.17),
            (Graha::Mars, mars, saturn, 22.56),
            (Graha::Mars, mars, jupiter, 15.00 + 42.75),
            (Graha::Venus, venus, sun, 0.0),
        ] {
            let ours = drishti(who, from, on);
            assert!(
                (ours - raman).abs() < 0.1,
                "{who:?} on {on}: {ours} against {raman}"
            );
        }
    }
}
