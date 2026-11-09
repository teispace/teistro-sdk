//! The tithis and nakshatras a vakya almanac prints for a day, each
//! with the moment it ends, as nadis and vinadis after the day's sunrise.
//!
//! The almanac reckons the Sun and the Moon at each sunrise
//! ([`true_sun`], [`almanac_moon`]) and finds where a limb's last point
//! falls between two sunrises by proportion, the text's rule for every
//! table (I.6–8): a boundary `b` between the places `a` at one sunrise
//! and `a′` at the next ends `(b − a) / (a′ − a)` of the day after the
//! first. The day's fraction is read as the time after the place's own
//! sunrise, as the almanac's sankrantis are (`super`'s module doc).
//!
//! The Manonmani Vilasam almanac prints a second end in one day as the
//! nadis after the first, not after sunrise (crux C1292), so its "55-22"
//! after a "3-23" is [`LimbEnd::at`] 58 nadis 45 vinadis; this module
//! gives every end after sunrise.

use super::moon::{ARC_PER_CIRCLE, ARC_PER_DEGREE, Arc, almanac_moon, rounded, true_sun};
use super::{KaliMoment, PARTS_PER_DAY};

/// A tithi's arc of the elongation: 12°.
const TITHI_ARC: i64 = 12 * ARC_PER_DEGREE;

/// A nakshatra's arc: 13° 20′.
const NAKSHATRA_ARC: i64 = ARC_PER_CIRCLE / 27;

/// A limb that ends within a day: its number, the tithi from 1 (shukla
/// pratipada) to 30 (amavasya) or the nakshatra from 1 (Ashvini) to 27
/// (Revati), and when it ends.
///
/// ```
/// use teistro_siddhanta::vakya::{KaliMoment, LimbEnd};
///
/// let end = LimbEnd { number: 4, at: KaliMoment::from_parts(4 * (14 * 60 + 23)) };
/// assert_eq!(end.at.since_sunrise(), (14, 23, 0));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LimbEnd {
    /// The limb that ends, counted from 1.
    pub number: u8,
    /// When it ends.
    pub at: KaliMoment,
}

/// The ends of the arcs of `size` a place crosses in the Kali day `day`
/// (sunrise to sunrise, the epoch's being day 1), the place at each
/// sunrise given by `place` of the Kali days elapsed then.
fn ends(day: i64, size: i64, place: impl Fn(i64) -> Arc) -> Vec<LimbEnd> {
    let (first, next) = (place(day - 1), place(day));
    let travel = next.since(first).quarter_seconds();
    let start = first.quarter_seconds();
    let count = ARC_PER_CIRCLE / size;
    let mut out = Vec::new();
    if travel == 0 {
        return out;
    }
    let mut boundary = (start.div_euclid(size) + 1) * size;
    while boundary <= start + travel {
        let parts = rounded((boundary - start) * PARTS_PER_DAY, travel);
        let number = (boundary / size - 1).rem_euclid(count) + 1;
        out.push(LimbEnd {
            number: u8::try_from(number).unwrap_or(u8::MAX),
            at: KaliMoment::from_parts((day - 1) * PARTS_PER_DAY + parts),
        });
        boundary += size;
    }
    out
}

/// The Moon less the Sun, both the almanac's, after so many Kali days.
///
/// ```
/// use teistro_siddhanta::vakya;
///
/// // Example 3's day: the Moon 4s 11° 33′ with its manyadi, the Sun
/// // 5s 23° 28′, so the Moon is 318° ahead, the 27th tithi running.
/// let (signs, degrees, minutes, _) = vakya::elongation(1_844_004).rashi();
/// assert_eq!((signs, degrees, minutes), (10, 18, 4));
/// ```
#[must_use]
pub fn elongation(kali_days: i64) -> Arc {
    let sun = true_sun(KaliMoment::from_parts(kali_days * PARTS_PER_DAY));
    almanac_moon(kali_days).since(sun)
}

/// The tithis that end in a Kali day, in order, as a vakya almanac
/// prints them: the elongation of the [`almanac_moon`] from the
/// [`true_sun`] at the day's sunrise and the next, each tithi's last
/// point found between them by proportion.
///
/// ```
/// use teistro_siddhanta::vakya;
///
/// // 18 September 2023 (Kali day 1,871,741): the tritiya ends 13 nadis
/// // 42 vinadis after sunrise and the chaturthi begins, so it holds most
/// // of that day's midday, where the modern sky's begins after it.
/// let ends = vakya::tithi_ends(1_871_741);
/// let (nadis, vinadis, _) = ends[0].at.since_sunrise();
/// assert_eq!((ends.len(), ends[0].number, nadis, vinadis), (1, 3, 13, 42));
/// ```
#[must_use]
pub fn tithi_ends(day: i64) -> Vec<LimbEnd> {
    ends(day, TITHI_ARC, elongation)
}

/// The nakshatras the Moon leaves in a Kali day, in order: the
/// [`almanac_moon`] at the day's sunrise and the next, each nakshatra's
/// last point found between them by proportion.
///
/// ```
/// use teistro_siddhanta::vakya;
///
/// // 15 June 1998 (Kali day 1,862,515): Dhanishtha ends 33 nadis 23
/// // vinadis after sunrise, as the Manonmani Vilasam almanac prints it.
/// let ends = vakya::nakshatra_ends(1_862_515);
/// assert_eq!(ends.iter().map(|end| (end.number, end.at.since_sunrise().0)).collect::<Vec<_>>(), [(23, 33)]);
/// ```
#[must_use]
pub fn nakshatra_ends(day: i64) -> Vec<LimbEnd> {
    ends(day, NAKSHATRA_ARC, almanac_moon)
}
