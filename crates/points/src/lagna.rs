//! The special lagnas: three the clock drives, one the Moon does, and
//! the two Yogi points.
//!
//! The hora, ghati and pranapada lagnas are **one rule at three
//! speeds**: start at the Sun *at birth* and advance 30°, 75° or 60° for
//! each hour since sunrise. The pranapada adds one thing more — nothing
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
//! use teistro_points::lagna::{ghati, hora, sree};
//!
//! // An hour after sunrise the hora lagna has moved one sign from the
//! // Sun and the ghati lagna two and a half.
//! assert!((hora(0.0, 1.0).expect("finite").longitude_deg - 30.0).abs() < 1e-9);
//! assert!((ghati(0.0, 1.0).expect("finite").longitude_deg - 75.0).abs() < 1e-9);
//! // The Sree lagna is the lagna advanced by the Moon's nakshatra
//! // fraction, of a whole circle.
//! assert!((sree(0.0, 360.0 / 54.0).expect("finite").longitude_deg - 180.0).abs() < 1e-9);
//! ```

use teistro_core::catalogue::{Nakshatra, Point, Rashi};
use teistro_core::error::{Error, Status};

use crate::derived::Derived;

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

/// The longest elapsed time since sunrise a chart can have, hours.
///
/// A day of the tradition runs sunrise to sunrise, and the longest one
/// the SDK's own polar policies produce is well inside this; a caller
/// passing days rather than hours is what the bound catches.
pub const LONGEST_HOURS: f64 = 36.0;

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
        AVAYOGI_FROM_YOGI_DEG, LONGEST_HOURS, PRANAPADA_SHIFT_DEG, YOGI_FROM_LUMINARIES_DEG,
        avayogi, ghati, hora, nakshatra_fraction, nakshatra_of, pranapada, pranapada_shift,
        sign_of, sree, yogi,
    };
    use teistro_core::catalogue::{Nakshatra, Point, Rashi};

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
