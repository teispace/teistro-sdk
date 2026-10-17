//! The special lagnas: four the clock drives, two the Moon does, and
//! the two Yogi points.
//!
//! The Moon's two are the Sree lagna, the lagna advanced by the Moon's
//! nakshatra fraction, and the **Indu lagna** (इन्दुलग्न), a sign counted
//! from the Moon's by the kalas (कला) of the two ninth lords — Uttara
//! Kalamrita, Kanda I section 4 v. 27. The verse names a sign; the
//! longitude it is given here is the Moon's own degree in that sign
//! (crux C406).
//!
//! The Vighati lagna does not ship: no text read defines it (crux C407).
//!
//! The bhava, hora, ghati and pranapada lagnas are **one rule at four
//! speeds**: start at the Sun *at birth* and advance 15°, 30°, 75° or 60°
//! for each hour since sunrise — one sign in five ghatis, in two and a half,
//! in one (BPHS ch. 4 vv. 2 to 8), and two signs an hour. The pranapada adds one thing more — nothing
//! if the Sun stands in a movable sign, 240° in a fixed one, 120° in a
//! dual one.
//!
//! Two conventions there are the corpus's rather than a text's, and both
//! are the kind a reader gets wrong. The start is the Sun **at birth**,
//! where most statements of the rule say sunrise, and using sunrise is
//! out by half a degree. And the hora lagna's rate is one sign an hour —
//! two and a half ghatis — while the ghati lagna's is one sign a ghati
//! (`03-design/points-measured.md` §5).
//!
//! The three are exact on 46 of the corpus's 71 fixtures and out on the
//! rest in proportion to their own rates, which is what shows that the
//! rules are right and the engine's clock is what differs: all three
//! imply the same elapsed time as each other, at most 1.633 minutes from
//! the ishtakaal recorded beside them. The SDK uses the ishtakaal it
//! computes, and the difference is registry entry 25.
//!
//! ```
//! use teistro_points::lagna::{bhava, ghati, hora, sree};
//!
//! // An hour after sunrise the hora lagna has moved one sign from the
//! // Sun and the ghati lagna two and a half.
//! assert!((hora(0.0, 1.0).expect("finite").longitude_deg - 30.0).abs() < 1e-9);
//! assert!((ghati(0.0, 1.0).expect("finite").longitude_deg - 75.0).abs() < 1e-9);
//! // The bhava lagna moves a sign in five ghatis, two hours.
//! assert!((bhava(0.0, 2.0).expect("finite").longitude_deg - 30.0).abs() < 1e-9);
//! // The Sree lagna is the lagna advanced by the Moon's nakshatra
//! // fraction, of a whole circle.
//! assert!((sree(0.0, 360.0 / 54.0).expect("finite").longitude_deg - 180.0).abs() < 1e-9);
//! ```

use teistro_core::catalogue::{Graha, Nakshatra, Point, Rashi};
use teistro_core::error::{Error, Status};

use crate::derived::Derived;

/// The degrees the bhava lagna advances in an hour after sunrise: one sign
/// in five ghatis, which is two hours (BPHS ch. 4 vv. 2 to 3).
pub const BHAVA_RATE_DEG_PER_HOUR: f64 = 15.0;

/// The degrees the hora lagna advances in an hour after sunrise: one
/// sign an hour, which is one sign in two and a half ghatis.
pub const HORA_RATE_DEG_PER_HOUR: f64 = 30.0;

/// The degrees the ghati lagna advances in an hour: one sign a ghati.
pub const GHATI_RATE_DEG_PER_HOUR: f64 = 75.0;

/// The degrees the pranapada lagna advances in an hour.
pub const PRANAPADA_RATE_DEG_PER_HOUR: f64 = 60.0;

/// What the pranapada adds beyond its rate, by the modality of the sign
/// the Sun stands in: nothing for a movable sign, two thirds of the
/// circle for a fixed one, a third for a dual one.
pub const PRANAPADA_SHIFT_DEG: [f64; 3] = [0.0, 240.0, 120.0];

/// How far the Yogi point stands from the two luminaries together:
/// seven nakshatras.
pub const YOGI_FROM_LUMINARIES_DEG: f64 = 93.0 + 20.0 / 60.0;

/// How far the Avayogi stands from the Yogi: fourteen nakshatras.
pub const AVAYOGI_FROM_YOGI_DEG: f64 = 186.0 + 40.0 / 60.0;

/// The kalas (कला, "rays") of the seven grahas the Indu lagna sums, in
/// weekday order from the Sun: 30, 16, 6, 8, 10, 12 and 1 (Uttara
/// Kalamrita, Kanda I section 4 v. 27, which gives them as numerals in
/// the verse itself). Indexed by the graha's catalogue id, which is the
/// weekday order for these seven.
pub const INDU_KALAS: [u8; 7] = [30, 16, 6, 8, 10, 12, 1];

/// How far round the Indu lagna's count reaches: the kalas are reduced
/// by twelves, and a sum that twelve divides counts the twelfth sign
/// (crux C406).
pub const INDU_SIGNS: u8 = 12;

/// The longest elapsed time since sunrise a chart can have, hours.
///
/// A day of the tradition runs sunrise to sunrise, a day and a night. A
/// polar day has no sunrise, and the `NEAREST_EVENT` policy counts from the
/// nearest one there was: a birth at noon under Tromsø's midnight sun on
/// 21 June 1988 is 102 hours from it, and a hora lagna 102 hours on is not a
/// time the verses count. So the bound refuses it, as it refuses a caller
/// passing days rather than hours.
pub const LONGEST_HOURS: f64 = 36.0;

/// The bhava lagna: the Sun advanced one sign for each five ghatis since
/// sunrise (BPHS ch. 4 vv. 2 to 3). The verse starts from the Sun at
/// sunrise; this starts from the Sun at birth, as its three siblings do,
/// which is the recording engine's convention for them (the module's notes).
///
/// # Errors
///
/// As [`driven`].
pub fn bhava(sun_deg: f64, hours_after_sunrise: f64) -> Result<Derived, Error> {
    driven(
        Point::BhavaLagna,
        sun_deg,
        hours_after_sunrise,
        BHAVA_RATE_DEG_PER_HOUR,
        0.0,
    )
}

/// The hora lagna: the Sun advanced one sign for each hour since
/// sunrise.
///
/// # Errors
///
/// As [`driven`].
pub fn hora(sun_deg: f64, hours_after_sunrise: f64) -> Result<Derived, Error> {
    driven(
        Point::HoraLagna,
        sun_deg,
        hours_after_sunrise,
        HORA_RATE_DEG_PER_HOUR,
        0.0,
    )
}

/// The ghati lagna: the Sun advanced one sign for each ghati, which is
/// two and a half signs an hour.
///
/// # Errors
///
/// As [`driven`].
pub fn ghati(sun_deg: f64, hours_after_sunrise: f64) -> Result<Derived, Error> {
    driven(
        Point::GhatiLagna,
        sun_deg,
        hours_after_sunrise,
        GHATI_RATE_DEG_PER_HOUR,
        0.0,
    )
}

/// The pranapada lagna: two signs an hour, and a shift by the modality
/// of the sign the Sun stands in.
///
/// # Errors
///
/// As [`driven`].
pub fn pranapada(sun_deg: f64, hours_after_sunrise: f64) -> Result<Derived, Error> {
    let shift = pranapada_shift(sun_deg)?;
    driven(
        Point::PranapadaLagna,
        sun_deg,
        hours_after_sunrise,
        PRANAPADA_RATE_DEG_PER_HOUR,
        shift,
    )
}

/// What the pranapada adds for a Sun in a given sign.
///
/// # Errors
///
/// `INVALID_ARG` for a longitude that is not a finite number.
pub fn pranapada_shift(sun_deg: f64) -> Result<f64, Error> {
    let sign = Derived::at(Point::PranapadaLagna, sun_deg)?.sign;
    let modality = usize::from(sign as u16) % PRANAPADA_SHIFT_DEG.len();
    Ok(PRANAPADA_SHIFT_DEG
        .get(modality)
        .copied()
        .unwrap_or_default())
}

/// One of the three lagnas the clock drives.
///
/// # Errors
///
/// `INVALID_ARG` for a longitude or an elapsed time that is not a finite
/// number, and `OUT_OF_RANGE` for a time outside 0 to
/// [`LONGEST_HOURS`] — a negative one is a birth before the sunrise it
/// is measured from, which is a caller's mistake and not a chart.
pub fn driven(
    point: Point,
    sun_deg: f64,
    hours_after_sunrise: f64,
    rate_deg_per_hour: f64,
    shift_deg: f64,
) -> Result<Derived, Error> {
    if !hours_after_sunrise.is_finite() || !(0.0..=LONGEST_HOURS).contains(&hours_after_sunrise) {
        return Err(Error::new(
            Status::OutOfRange,
            format!(
                "{} needs an elapsed time of 0 to {LONGEST_HOURS} hours since sunrise, \
                 not {hours_after_sunrise}",
                point.key()
            ),
        )
        .with_field("points.hours_after_sunrise"));
    }
    Derived::at(
        point,
        sun_deg + rate_deg_per_hour * hours_after_sunrise + shift_deg,
    )
}

/// The Sree lagna: the lagna advanced by the fraction of its nakshatra
/// the Moon has crossed — of a whole **circle**, not of a sign.
///
/// The three other readings a reader might try are wrong by tens of
/// degrees over the corpus, so this is settled rather than chosen
/// (`03-design/points-measured.md` §4).
///
/// # Errors
///
/// `INVALID_ARG` for a longitude that is not a finite number.
pub fn sree(lagna_deg: f64, moon_deg: f64) -> Result<Derived, Error> {
    let fraction = nakshatra_fraction(moon_deg)?;
    Derived::at(Point::SreeLagna, lagna_deg + fraction * 360.0)
}

/// How far into its nakshatra a longitude stands, as a fraction of one.
///
/// # Errors
///
/// `INVALID_ARG` for a longitude that is not a finite number.
pub fn nakshatra_fraction(longitude_deg: f64) -> Result<f64, Error> {
    let width = 360.0 / f64::from(u16::try_from(Nakshatra::ALL.len()).unwrap_or(27));
    let inside = Derived::at(Point::SreeLagna, longitude_deg)?.longitude_deg % width;
    Ok(inside / width)
}

/// The Yogi point: the two luminaries together, and seven nakshatras on.
///
/// # Errors
///
/// `INVALID_ARG` for a longitude that is not a finite number.
pub fn yogi(sun_deg: f64, moon_deg: f64) -> Result<Derived, Error> {
    Derived::at(Point::Yogi, sun_deg + moon_deg + YOGI_FROM_LUMINARIES_DEG)
}

/// The Avayogi: fourteen nakshatras past the Yogi.
///
/// # Errors
///
/// `INVALID_ARG` for a longitude that is not a finite number.
pub fn avayogi(sun_deg: f64, moon_deg: f64) -> Result<Derived, Error> {
    let yogi = yogi(sun_deg, moon_deg)?;
    Derived::at(Point::Avayogi, yogi.longitude_deg + AVAYOGI_FROM_YOGI_DEG)
}

/// A graha's kala for the Indu lagna, or `None` for one of the nodes or
/// the outer planets, which lord no sign the count reads.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_points::lagna::indu_kala;
///
/// assert_eq!(indu_kala(Graha::Sun), Some(30));
/// assert_eq!(indu_kala(Graha::Saturn), Some(1));
/// assert_eq!(indu_kala(Graha::Rahu), None);
/// ```
#[must_use]
pub fn indu_kala(graha: Graha) -> Option<u8> {
    INDU_KALAS.get(usize::from(graha.id())).copied()
}

/// The kala of the ninth sign's lord from a sign, counting the sign
/// itself as the first. A sign's lord is always one of the seven, each
/// with a kala, so the count needs no refusal.
fn ninth_kala(from: Rashi) -> u8 {
    let mut signs = Rashi::ALL;
    signs.rotate_left(usize::from(from.id()));
    let [_, _, _, _, _, _, _, _, ninth, ..] = signs;
    let kala = indu_kala(ninth.attributes().lord);
    debug_assert!(kala.is_some(), "{ninth:?}'s lord has a kala");
    kala.unwrap_or_default()
}

/// Which sign from the Moon's the Indu lagna stands in, counted from one
/// with the Moon's own sign the first: the kalas of the ninth lords from
/// the lagna and from the Moon, added and reduced by twelves, a sum that
/// twelve divides counting twelve.
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_points::lagna::indu_count;
///
/// // A Libra lagna and a Capricorn Moon both have Mercury, eight kalas,
/// // as the ninth lord: sixteen, less twelve, is the fourth sign.
/// assert_eq!(indu_count(Rashi::Libra, Rashi::Capricorn), 4);
/// // A Leo lagna and a Pisces Moon both have Mars, six: twelve, which
/// // twelve divides, counts the twelfth.
/// assert_eq!(indu_count(Rashi::Leo, Rashi::Pisces), 12);
/// ```
#[must_use]
pub fn indu_count(lagna: Rashi, moon: Rashi) -> u8 {
    match (ninth_kala(lagna) + ninth_kala(moon)) % INDU_SIGNS {
        0 => INDU_SIGNS,
        rest => rest,
    }
}

/// The Indu lagna: the sign [`indu_count`] reaches from the Moon's
/// (Uttara Kalamrita, Kanda I section 4 v. 27), at the Moon's own degree
/// in it.
///
/// The verse names a sign and no degree. Carrying the Moon's degree
/// keeps the point's sign edge honest: with the lagna's sign held, the
/// Indu lagna changes sign exactly when the Moon does, so the Moon's
/// distance to its edge is the point's (crux C406).
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_points::lagna::indu;
///
/// // The lagna at 5° Libra, the Moon at 5° Capricorn: four signs on
/// // from the Moon, counting its own, is Aries, at the Moon's 5°.
/// let point = indu(185.0, 275.0).expect("finite");
/// assert_eq!(point.sign, Rashi::Aries);
/// assert!((point.longitude_deg - 5.0).abs() < 1e-9);
/// ```
///
/// # Errors
///
/// `INVALID_ARG` for a longitude that is not a finite number.
pub fn indu(lagna_deg: f64, moon_deg: f64) -> Result<Derived, Error> {
    let lagna = Derived::at(Point::InduLagna, lagna_deg)?.sign;
    let moon = Derived::at(Point::InduLagna, moon_deg)?;
    let count = indu_count(lagna, moon.sign);
    Derived::at(
        Point::InduLagna,
        moon.longitude_deg + f64::from(count.saturating_sub(1)) * 30.0,
    )
}

/// The nakshatra a point falls in, which for the Yogi is the "yogi
/// nakshatra" an almanac prints beside it.
#[must_use]
pub fn nakshatra_of(point: &Derived) -> Option<Nakshatra> {
    let width = 360.0 / f64::from(u16::try_from(Nakshatra::ALL.len()).unwrap_or(27));
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a normalised longitude over a nakshatra's width is 0 to 26"
    )]
    let index = (point.longitude_deg / width) as u16;
    Nakshatra::from_id(index)
}

/// The sign a longitude falls in, which the modality shift reads.
#[must_use]
pub fn sign_of(longitude_deg: f64) -> Option<Rashi> {
    Derived::at(Point::SreeLagna, longitude_deg)
        .ok()
        .map(|derived| derived.sign)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::float_cmp,
        clippy::indexing_slicing,
        reason = "tests fail by panicking, index their own tables, and compare longitudes computed the same way"
    )]

    use super::{
        AVAYOGI_FROM_YOGI_DEG, INDU_KALAS, LONGEST_HOURS, PRANAPADA_SHIFT_DEG,
        YOGI_FROM_LUMINARIES_DEG, avayogi, ghati, hora, indu, indu_count, indu_kala,
        nakshatra_fraction, nakshatra_of, pranapada, pranapada_shift, sign_of, sree, yogi,
    };
    use teistro_core::catalogue::{Graha, Nakshatra, Point, Rashi};

    #[test]
    fn the_indu_lagna_takes_the_printed_answer_of_the_worked_chart() {
        // P. S. Sastri's notes to Uttara Kalamrita ch. 4 v. 27 (Kanda I
        // section 4 in Sareen's print) work his chart 81, dated 1905.2.4
        // and drawn in the South Indian square on p. 98: the lagna at
        // Makara 25° and the Moon at Makara 17.4 (read as 17°04′), so the
        // ninth from both is Kanya and its lord Budha, eight kalas each,
        // sixteen in all; twelve expunged leaves four, and the fourth from
        // the Moon is Mesha, holding Guru. The printed answer is the sign;
        // the degree is the SDK's convention (crux C406), the Moon's.
        let lagna = 270.0 + 25.0;
        let moon = 270.0 + 17.0 + 4.0 / 60.0;
        let point = indu(lagna, moon).unwrap();
        assert_eq!(point.point, Point::InduLagna);
        assert_eq!(point.sign, Rashi::Aries, "Mesha");
        assert!((point.longitude_deg - (17.0 + 4.0 / 60.0)).abs() < 1e-9);
        assert_eq!(indu_count(Rashi::Capricorn, Rashi::Capricorn), 4);
        // A Tula lagna also has Budha for its ninth lord, so it would give
        // the same answer had the chart been drawn with one.
        assert_eq!(indu_count(Rashi::Libra, Rashi::Capricorn), 4);
    }

    #[test]
    fn the_kalas_are_the_verse_s_and_only_the_seven_have_them() {
        // Uttara Kalamrita Kanda I section 4 v. 27: 30, 16, 6, 8, 10, 12
        // and 1, the Sun first, in weekday order.
        let seven = [
            Graha::Sun,
            Graha::Moon,
            Graha::Mars,
            Graha::Mercury,
            Graha::Jupiter,
            Graha::Venus,
            Graha::Saturn,
        ];
        let kalas: Vec<u8> = seven
            .into_iter()
            .map(|graha| indu_kala(graha).unwrap())
            .collect();
        assert_eq!(kalas, INDU_KALAS.to_vec());
        assert_eq!(kalas, vec![30, 16, 6, 8, 10, 12, 1]);
        for graha in [Graha::Rahu, Graha::Ketu] {
            assert_eq!(indu_kala(graha), None, "{graha:?}");
        }
    }

    #[test]
    fn the_indu_lagna_counts_from_the_moon_by_hand() {
        // An Aries lagna's ninth is Sagittarius, Jupiter's: 10. A Taurus
        // Moon's ninth is Capricorn, Saturn's: 1. Eleven, under twelve,
        // is the eleventh sign from Taurus counting Taurus: Pisces. The
        // Moon at 15° Taurus (45°) puts it at 15° Pisces, 45 + 10 × 30 =
        // 345°.
        assert_eq!(indu_count(Rashi::Aries, Rashi::Taurus), 11);
        let point = indu(0.0, 45.0).unwrap();
        assert_eq!(point.sign, Rashi::Pisces);
        assert!((point.longitude_deg - 345.0).abs() < 1e-9);
        // A Leo lagna's ninth is Aries and a Pisces Moon's is Scorpio,
        // both Mars's: 6 + 6 = 12, which twelve divides. That counts the
        // twelfth sign from Pisces, Aquarius (crux C406): the Moon at 10°
        // Pisces (340°) puts it at 340 + 11 × 30 = 670, which is 310°.
        assert_eq!(indu_count(Rashi::Leo, Rashi::Pisces), 12);
        let twelfth = indu(125.0, 340.0).unwrap();
        assert_eq!(twelfth.sign, Rashi::Aquarius);
        assert!((twelfth.longitude_deg - 310.0).abs() < 1e-9);
        // The Sun as both ninth lords is the largest sum, 60, and twelve
        // divides it too: a Sagittarius lagna (ninth Leo) and a
        // Sagittarius Moon count the twelfth, Scorpio.
        assert_eq!(indu_count(Rashi::Sagittarius, Rashi::Sagittarius), 12);
    }

    #[test]
    fn the_indu_lagna_is_the_moons_degree_in_a_counted_sign_everywhere() {
        for lagna in Rashi::ALL {
            for moon in Rashi::ALL {
                let count = indu_count(lagna, moon);
                assert!((1..=12).contains(&count), "{lagna:?} {moon:?}");
                let moon_deg = f64::from(moon as u16) * 30.0 + 7.25;
                let point = indu(f64::from(lagna as u16) * 30.0 + 1.0, moon_deg).unwrap();
                assert_eq!(
                    point.sign as u16,
                    (moon as u16 + u16::from(count) - 1) % 12,
                    "{lagna:?} {moon:?}"
                );
                assert!(
                    (point.in_sign_deg() - 7.25).abs() < 1e-9,
                    "the Moon's degree"
                );
            }
        }
    }

    #[test]
    fn a_longitude_that_is_not_a_number_has_no_indu_lagna() {
        for bad in [f64::NAN, f64::INFINITY] {
            let error = indu(bad, 10.0).expect_err("not a lagna");
            assert_eq!(error.field(), Some("points.longitude_deg"));
            assert!(error.message.contains("INDU_LAGNA"), "{error}");
            assert!(indu(10.0, bad).is_err(), "nor a Moon");
        }
    }

    #[test]
    fn the_three_rates_are_one_sign_an_hour_a_ghati_and_a_half_hour() {
        assert!((hora(0.0, 1.0).unwrap().longitude_deg - 30.0).abs() < 1e-9);
        assert!((ghati(0.0, 1.0).unwrap().longitude_deg - 75.0).abs() < 1e-9);
        // The pranapada's rate is sixty; Aries is movable, so no shift.
        assert!((pranapada(0.0, 1.0).unwrap().longitude_deg - 60.0).abs() < 1e-9);
        // Twelve hours of the hora lagna is the whole circle.
        assert!(hora(0.0, 12.0).unwrap().longitude_deg.abs() < 1e-9);
        assert_eq!(hora(10.0, 0.0).unwrap().longitude_deg, 10.0, "at sunrise");
    }

    #[test]
    fn the_pranapada_shifts_by_the_suns_modality() {
        // Aries movable, Taurus fixed, Gemini dual.
        assert!(pranapada_shift(0.0).unwrap().abs() < 1e-12);
        assert!((pranapada_shift(35.0).unwrap() - 240.0).abs() < 1e-12);
        assert!((pranapada_shift(65.0).unwrap() - 120.0).abs() < 1e-12);
        // And it repeats every three signs, all the way round.
        for sign in 0..12_u16 {
            let sun = f64::from(sign) * 30.0 + 5.0;
            assert!(
                (pranapada_shift(sun).unwrap() - PRANAPADA_SHIFT_DEG[usize::from(sign) % 3]).abs()
                    < 1e-12,
                "sign {sign}"
            );
        }
        assert!((pranapada(35.0, 0.0).unwrap().longitude_deg - 275.0).abs() < 1e-9);
    }

    #[test]
    fn the_sree_lagna_takes_the_fraction_of_a_circle() {
        // The Moon halfway through a nakshatra puts it half a circle on.
        let half = sree(0.0, 360.0 / 54.0).unwrap();
        assert!((half.longitude_deg - 180.0).abs() < 1e-9);
        // At a nakshatra's start it is the lagna itself.
        assert!((sree(45.0, 0.0).unwrap().longitude_deg - 45.0).abs() < 1e-9);
        assert!((sree(45.0, 360.0 / 27.0).unwrap().longitude_deg - 45.0).abs() < 1e-9);
        assert_eq!(sree(0.0, 0.0).unwrap().point, Point::SreeLagna);
        // And the fraction itself runs from nothing to one.
        assert!(nakshatra_fraction(0.0).unwrap().abs() < 1e-12);
        assert!((0.0..1.0).contains(&nakshatra_fraction(123.456).unwrap()));
    }

    #[test]
    fn the_yogi_is_the_luminaries_and_the_avayogi_is_past_it() {
        let point = yogi(10.0, 20.0).unwrap();
        assert!((point.longitude_deg - (30.0 + YOGI_FROM_LUMINARIES_DEG)).abs() < 1e-9);
        let other = avayogi(10.0, 20.0).unwrap();
        assert!(
            (other.longitude_deg
                - (30.0 + YOGI_FROM_LUMINARIES_DEG + AVAYOGI_FROM_YOGI_DEG).rem_euclid(360.0))
            .abs()
                < 1e-9
        );
        assert_eq!(point.point, Point::Yogi);
        assert_eq!(other.point, Point::Avayogi);
        // The two stand fourteen nakshatras apart, always.
        for tenth in 0..720 {
            let moon = f64::from(tenth) / 2.0;
            let apart = (avayogi(0.0, moon).unwrap().longitude_deg
                - yogi(0.0, moon).unwrap().longitude_deg)
                .rem_euclid(360.0);
            assert!((apart - AVAYOGI_FROM_YOGI_DEG).abs() < 1e-9, "moon {moon}");
        }
    }

    #[test]
    fn a_point_falls_in_the_nakshatra_its_longitude_gives() {
        assert_eq!(
            nakshatra_of(&sree(0.0, 0.0).unwrap()),
            Nakshatra::from_id(0)
        );
        let last = sree(359.9, 0.0).unwrap();
        assert_eq!(nakshatra_of(&last), Nakshatra::from_id(26));
        assert_eq!(sign_of(45.0), Some(Rashi::Taurus));
        assert_eq!(sign_of(f64::NAN), None);
    }

    #[test]
    fn an_elapsed_time_outside_a_day_and_a_half_is_refused_by_field() {
        for bad in [-0.1, LONGEST_HOURS + 0.1, f64::NAN, f64::INFINITY] {
            let error = hora(0.0, bad).expect_err("not an elapsed time");
            assert_eq!(error.field(), Some("points.hours_after_sunrise"), "{bad}");
            assert!(error.message.contains("HORA_LAGNA"), "{error}");
        }
        assert!(hora(0.0, 0.0).is_ok() && hora(0.0, LONGEST_HOURS).is_ok());
        assert!(ghati(f64::NAN, 1.0).is_err(), "and so is the Sun");
    }
}
