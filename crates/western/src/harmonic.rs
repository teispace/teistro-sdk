//! A chart's harmonics: every point's longitude multiplied by a whole
//! number, so that every pair standing that fraction of the circle apart
//! meets (John Addey, *Harmonics in Astrology*, 1976;
//! `03-design/western-harmonics.md`, C252 to C254).

use serde::{Deserialize, Serialize};
use teistro_core::angle::difference_deg;
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;
use teistro_core::house::{House, house_of};

use crate::aspects::{PlanetAt, refuse_unreadable};

/// The record's name where a binding sends it, which a refusal is named
/// under.
const ROOT: &str = "harmonic";

/// The highest harmonic a request may ask: the circle in whole degrees,
/// whose aspect is 1° (decision 1).
pub const MAX_HARMONIC: u16 = 360;

/// The orb of a meeting in a harmonic chart by default, degrees: the
/// smaller of the two full-circle orbs Addey offers (p. 130, C252).
pub const ADDEY_HARMONIC_ORB_DEG: f64 = 12.0;

/// The widest orb a caller may ask, degrees: a sign of the harmonic
/// chart.
pub const MAX_HARMONIC_ORB_DEG: f64 = 30.0;

/// What a chart's harmonic is asked: which, and the orb of a meeting.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HarmonicRequest {
    /// Which harmonic, 1 to [`MAX_HARMONIC`]: the number every longitude
    /// is multiplied by.
    pub number: u16,
    /// The orb of a meeting in the harmonic chart, degrees:
    /// [`ADDEY_HARMONIC_ORB_DEG`] unless a caller says otherwise.
    #[serde(default = "addey_orb")]
    pub orb_deg: f64,
}

const fn addey_orb() -> f64 {
    ADDEY_HARMONIC_ORB_DEG
}

impl HarmonicRequest {
    /// The `number`-th harmonic under Addey's orb.
    #[must_use]
    pub const fn of(number: u16) -> HarmonicRequest {
        HarmonicRequest {
            number,
            orb_deg: ADDEY_HARMONIC_ORB_DEG,
        }
    }

    /// Reads a meeting to this orb, degrees.
    #[must_use]
    pub const fn with_orb_deg(mut self, orb_deg: f64) -> HarmonicRequest {
        self.orb_deg = orb_deg;
        self
    }

    /// The request a binding sends, as JSON: `{"number": 9}`, with
    /// `orbDeg` beside it to widen or narrow the meetings.
    ///
    /// ```
    /// use teistro_western::HarmonicRequest;
    ///
    /// let ninth = HarmonicRequest::from_json(r#"{"number": 9}"#)?;
    /// assert_eq!(ninth, HarmonicRequest::of(9));
    /// let none = HarmonicRequest::from_json("{}").unwrap_err();
    /// assert_eq!(none.field(), Some("harmonic"));
    /// let zero = HarmonicRequest::from_json(r#"{"number": 0}"#).unwrap_err();
    /// assert_eq!(zero.field(), Some("harmonic.number"));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, and whatever [`HarmonicRequest::check`] refuses, each named
    /// under `harmonic`.
    pub fn from_json(text: &str) -> Result<HarmonicRequest, Error> {
        let asked: HarmonicRequest = teistro_core::strict::read(text, ROOT)?;
        asked.check().map_err(|why| why.under(ROOT))?;
        Ok(asked)
    }

    /// Refuses a harmonic outside 1 to [`MAX_HARMONIC`] and an orb that is
    /// not a positive number of degrees up to [`MAX_HARMONIC_ORB_DEG`].
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming `number` or `orbDeg`.
    pub fn check(&self) -> Result<(), Error> {
        if !(1..=MAX_HARMONIC).contains(&self.number) {
            return Err(Error::invalid_arg(format!(
                "a harmonic is a whole number from 1 to {MAX_HARMONIC}, not {}",
                self.number
            ))
            .with_field("number")
            .with_hint("the 1st is the chart itself; the 9th is the navamsa"));
        }
        if self.orb_deg.is_finite() && self.orb_deg > 0.0 && self.orb_deg <= MAX_HARMONIC_ORB_DEG {
            return Ok(());
        }
        Err(Error::invalid_arg(format!(
            "a harmonic meeting's orb is more than 0° and at most {MAX_HARMONIC_ORB_DEG}°, not {}",
            self.orb_deg
        ))
        .with_field("orbDeg")
        .with_hint(format!(
            "Addey allows {ADDEY_HARMONIC_ORB_DEG}° to 15° in every harmonic (p. 130)"
        )))
    }
}

/// A point of a harmonic chart, in the order a tie is broken: the grahas
/// in the catalogue's order, then the ascendant, then the midheaven.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "point", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HarmonicPoint {
    /// A planet.
    Graha {
        /// Which.
        graha: Graha,
    },
    /// The ascendant.
    Ascendant,
    /// The midheaven.
    Midheaven,
}

/// A point's place in the harmonic chart.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HarmonicPlaced {
    /// Which point.
    pub point: HarmonicPoint,
    /// Its longitude multiplied by the harmonic, degrees in `[0, 360)`.
    pub longitude_deg: f64,
    /// Its equal house from the harmonic ascendant (C254).
    pub house: House,
}

/// Two points meeting in a harmonic chart: within the orb of each other
/// there, and so within the orb divided by the harmonic of an aspect of
/// it in the chart itself.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HarmonicRow {
    /// The earlier point, in [`HarmonicPoint`]'s order.
    pub first: HarmonicPoint,
    /// The later.
    pub second: HarmonicPoint,
    /// How far apart they stand in the harmonic chart, degrees 0 to the
    /// orb.
    pub apart_deg: f64,
    /// Which multiple of the harmonic's aspect they stand at in the chart
    /// itself, 0 to half the harmonic: k for the aspect of k × 360° / n,
    /// so 1 of the 4th is the square and 2 of the 5th the biquintile.
    pub multiple: u16,
    /// The orb the request allowed, degrees.
    pub orb_deg: f64,
}

/// A chart's harmonic chart.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HarmonicChart {
    /// Which harmonic.
    pub harmonic: u16,
    /// Every point, the planets in the order given, then the ascendant and
    /// the midheaven.
    pub points: Vec<HarmonicPlaced>,
    /// The points meeting within the orb, closest first.
    pub rows: Vec<HarmonicRow>,
}

/// A longitude in the `harmonic`-th harmonic, degrees in `[0, 360)`:
/// multiplied, less the multiples of the circle (Addey, p. 100).
///
/// ```
/// use teistro_western::harmonic_deg;
///
/// // Addey's worked Moon, 23°33′ Scorpio in the 5th: 27°45′ Gemini.
/// let moon = harmonic_deg(233.0 + 33.0 / 60.0, 5);
/// assert!((moon - (87.0 + 45.0 / 60.0)).abs() < 1e-9);
/// ```
#[must_use]
pub fn harmonic_deg(longitude_deg: f64, harmonic: u16) -> f64 {
    (longitude_deg * f64::from(harmonic)).rem_euclid(360.0)
}

/// A chart's **harmonic chart** (Addey): each planet, the ascendant and the
/// midheaven at its longitude multiplied, in its equal house from the
/// harmonic ascendant (C254), and every pair within the orb of each other,
/// closest first (C252). Rows equally close keep the points' order.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_western::{HarmonicPoint, HarmonicRequest, PlanetAt, harmonic_chart};
///
/// // A square 1° past exact meets in the 4th harmonic 4° apart, as the
/// // first multiple of its 90°.
/// let chart = harmonic_chart(
///     &[PlanetAt::new(Graha::Sun, 10.0), PlanetAt::new(Graha::Mars, 101.0)],
///     250.0,
///     175.0,
///     &HarmonicRequest::of(4),
/// )?;
/// let row = chart.rows[0];
/// assert_eq!(row.first, HarmonicPoint::Graha { graha: Graha::Sun });
/// assert_eq!(row.second, HarmonicPoint::Graha { graha: Graha::Mars });
/// assert_eq!(row.multiple, 1);
/// assert!((row.apart_deg - 4.0).abs() < 1e-9);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// What [`HarmonicRequest::check`] refuses; a planet given twice; a
/// longitude, ascendant or midheaven that is not a finite number, named by
/// its field.
pub fn harmonic_chart(
    bodies: &[PlanetAt],
    ascendant_deg: f64,
    midheaven_deg: f64,
    request: &HarmonicRequest,
) -> Result<HarmonicChart, Error> {
    request.check()?;
    refuse_unreadable(bodies, "bodies")?;
    for (value, field) in [
        (ascendant_deg, "ascendantDeg"),
        (midheaven_deg, "midheavenDeg"),
    ] {
        if !value.is_finite() {
            return Err(Error::invalid_arg(format!("{value} is not finite")).with_field(field));
        }
    }
    let n = request.number;
    let radix: Vec<(HarmonicPoint, f64)> = bodies
        .iter()
        .map(|body| {
            (
                HarmonicPoint::Graha { graha: body.graha },
                body.longitude_deg,
            )
        })
        .chain([
            (HarmonicPoint::Ascendant, ascendant_deg),
            (HarmonicPoint::Midheaven, midheaven_deg),
        ])
        .collect();
    let rising = harmonic_deg(ascendant_deg, n);
    let mut cusp = rising - 30.0;
    let cusps = [(); 12].map(|()| {
        cusp = (cusp + 30.0).rem_euclid(360.0);
        cusp
    });
    let points = radix
        .iter()
        .map(|&(point, longitude)| {
            let longitude_deg = harmonic_deg(longitude, n);
            HarmonicPlaced {
                point,
                longitude_deg,
                house: house_of(longitude_deg, &cusps, 0.0),
            }
        })
        .collect();
    let mut rows: Vec<HarmonicRow> = radix
        .iter()
        .enumerate()
        .flat_map(|(at, first)| {
            radix
                .iter()
                .skip(at + 1)
                .map(move |second| (*first, *second))
        })
        .filter_map(|(a, b)| meeting(a, b, request))
        .collect();
    rows.sort_by(|a, b| a.apart_deg.total_cmp(&b.apart_deg));
    Ok(HarmonicChart {
        harmonic: n,
        points,
        rows,
    })
}

/// Two points' row when they meet within the orb, the earlier first.
fn meeting(
    (a, at_a): (HarmonicPoint, f64),
    (b, at_b): (HarmonicPoint, f64),
    request: &HarmonicRequest,
) -> Option<HarmonicRow> {
    let n = request.number;
    let apart_deg = difference_deg(harmonic_deg(at_a, n), harmonic_deg(at_b, n)).abs();
    if apart_deg > request.orb_deg {
        return None;
    }
    let radical = difference_deg(at_a, at_b).abs();
    let (first, second) = if a <= b { (a, b) } else { (b, a) };
    Some(HarmonicRow {
        first,
        second,
        apart_deg,
        multiple: multiple_of(radical, n),
        orb_deg: request.orb_deg,
    })
}

/// The whole k nearest `radical_deg` / (360° / n): the multiple of the
/// harmonic's aspect a radical arc of 0° to 180° stands at, 0 to n / 2.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "an arc of 0° to 180° in steps of 360° / n rounds to 0 to n / 2 ≤ 180"
)]
fn multiple_of(radical_deg: f64, harmonic: u16) -> u16 {
    (radical_deg * f64::from(harmonic) / 360.0).round() as u16
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;

    /// Churchill at Blenheim, 30 November 1874, "about 2 minutes before"
    /// 1.30 a.m. local (Addey, p. 98, note 5), recast by pyswisseph
    /// (Moshier), tropical: the ten planets, the ascendant and midheaven.
    const CHURCHILL: [(Graha, f64); 10] = [
        (Graha::Sun, 247.7216),
        (Graha::Moon, 149.6458),
        (Graha::Mercury, 227.5939),
        (Graha::Venus, 262.0273),
        (Graha::Mars, 196.5509),
        (Graha::Jupiter, 203.5679),
        (Graha::Saturn, 309.5938),
        (Graha::Uranus, 135.2292),
        (Graha::Neptune, 28.4335),
        (Graha::Pluto, 51.4243),
    ];
    const ASCENDANT: f64 = 180.5332;
    const MIDHEAVEN: f64 = 90.6966;

    fn churchill(request: &HarmonicRequest) -> HarmonicChart {
        let bodies: Vec<PlanetAt> = CHURCHILL
            .iter()
            .map(|&(graha, at)| PlanetAt::new(graha, at))
            .collect();
        harmonic_chart(&bodies, ASCENDANT, MIDHEAVEN, request).unwrap()
    }

    fn placed(chart: &HarmonicChart, point: HarmonicPoint) -> HarmonicPlaced {
        *chart.points.iter().find(|one| one.point == point).unwrap()
    }

    const fn graha(graha: Graha) -> HarmonicPoint {
        HarmonicPoint::Graha { graha }
    }

    #[test]
    fn churchills_ninth_harmonic_reads_as_addey_read_it() {
        let ninth = churchill(&HarmonicRequest::of(9));
        let sign = |point| (placed(&ninth, point).longitude_deg / 30.0).floor();
        let house = |point| placed(&ninth, point).house.get();
        // "The 9th harmonic conjunction of Moon and Saturn in the 3rd", in
        // Sagittarius, "opposite this conjunction at 26° Gemini".
        let row = ninth
            .rows
            .iter()
            .find(|row| (row.first, row.second) == (graha(Graha::Moon), graha(Graha::Saturn)))
            .unwrap();
        assert!(row.apart_deg < 0.6, "{}", row.apart_deg);
        assert_eq!(
            (sign(graha(Graha::Moon)), house(graha(Graha::Moon))),
            (8.0, 3)
        );
        assert_eq!(house(graha(Graha::Saturn)), 3);
        // About 160° apart in the radix: the fourth multiple of 40°.
        assert_eq!(row.multiple, 4);
        // "The rising Venus in Libra" and "the tenth house Pluto".
        assert_eq!(
            (sign(graha(Graha::Venus)), house(graha(Graha::Venus))),
            (6.0, 1)
        );
        assert_eq!(house(graha(Graha::Pluto)), 10);
        // The harmonic ascendant is in its own first house.
        assert_eq!(house(HarmonicPoint::Ascendant), 1);
        // "Navamsa Mars in Aquarius opposite radical Moon", and the
        // harmonic Mercury on the radical Sun.
        let mars = placed(&ninth, graha(Graha::Mars)).longitude_deg;
        assert_eq!((mars / 30.0).floor(), 10.0);
        assert!(difference_deg(mars, CHURCHILL[1].1 + 180.0).abs() < 1.0);
        let mercury = placed(&ninth, graha(Graha::Mercury)).longitude_deg;
        assert!(difference_deg(mercury, CHURCHILL[0].1).abs() < 1.0);
    }

    #[test]
    fn the_ninth_harmonic_is_the_parashari_navamsa() {
        // The navamsa counted as Parashara counts it: the sign's own
        // ninths, from Aries for a movable sign, Capricorn for a fixed and
        // Libra for a dual, which is (sign × 9 + part) mod 12.
        let mut longitude: f64 = 0.0;
        while longitude < 360.0 {
            let sign = (longitude / 30.0).floor();
            let part = ((longitude - sign * 30.0) / (30.0 / 9.0)).floor();
            let navamsa = (sign * 9.0 + part).rem_euclid(12.0);
            let ninth = (harmonic_deg(longitude, 9) / 30.0).floor();
            assert_eq!(ninth, navamsa, "{longitude}");
            longitude += 0.37;
        }
    }

    #[test]
    fn the_first_harmonic_is_the_chart_and_its_rows_the_conjunctions() {
        let first = churchill(&HarmonicRequest::of(1));
        for (point, (_, at)) in first.points.iter().zip(CHURCHILL) {
            assert!((point.longitude_deg - at).abs() < 1e-9);
        }
        assert!(first.rows.iter().all(|row| row.multiple == 0));
        // Mars and Jupiter, 7.0° apart in Libra, are the closest pair.
        assert_eq!(
            (first.rows[0].first, first.rows[0].second),
            (graha(Graha::Mars), graha(Graha::Jupiter))
        );
    }

    #[test]
    fn a_meeting_is_the_orb_in_the_harmonic_and_the_orb_divided_in_the_radix() {
        // A square 3° wide meets in the 4th within 12°, and not within 10°.
        let bodies = [
            PlanetAt::new(Graha::Sun, 0.0),
            PlanetAt::new(Graha::Moon, 93.0),
        ];
        let meets = |orb: f64| {
            harmonic_chart(
                &bodies,
                200.0,
                300.0,
                &HarmonicRequest::of(4).with_orb_deg(orb),
            )
            .unwrap()
            .rows
            .iter()
            .any(|row| row.first == graha(Graha::Sun) && row.second == graha(Graha::Moon))
        };
        assert!(meets(12.0) && !meets(11.9));
        // The angles are points like the planets: 72° apart, they meet in
        // the 5th (Addey, p. 102).
        let fifth = harmonic_chart(&[], 10.0, 82.0, &HarmonicRequest::of(5)).unwrap();
        assert_eq!(
            (
                fifth.rows[0].first,
                fifth.rows[0].second,
                fifth.rows[0].multiple
            ),
            (HarmonicPoint::Ascendant, HarmonicPoint::Midheaven, 1)
        );
    }

    #[test]
    fn rows_run_closest_first_across_zero_aries() {
        // 359.9° and 0.1° are 0.2° apart in every harmonic's radix, and
        // 1.8° in the 9th.
        let chart = harmonic_chart(
            &[
                PlanetAt::new(Graha::Venus, 359.9),
                PlanetAt::new(Graha::Mars, 0.1),
            ],
            100.0,
            10.0,
            &HarmonicRequest::of(9),
        )
        .unwrap();
        assert!((chart.rows[0].apart_deg - 1.8).abs() < 1e-9);
        assert_eq!(chart.rows[0].multiple, 0);
        assert!(
            chart
                .rows
                .windows(2)
                .all(|two| two[0].apart_deg <= two[1].apart_deg)
        );
    }

    #[test]
    fn a_request_is_refused_by_field() {
        let field = |text: &str| {
            HarmonicRequest::from_json(text)
                .unwrap_err()
                .field()
                .map(str::to_owned)
        };
        assert_eq!(
            field(r#"{"number": 0}"#).as_deref(),
            Some("harmonic.number")
        );
        assert_eq!(
            field(r#"{"number": 361}"#).as_deref(),
            Some("harmonic.number")
        );
        assert_eq!(
            field(r#"{"number": 2.5}"#).as_deref(),
            Some("harmonic.number")
        );
        assert_eq!(
            field(r#"{"number": 9, "orbDeg": 31}"#).as_deref(),
            Some("harmonic.orbDeg")
        );
        assert_eq!(
            field(r#"{"number": 9, "orb": 3}"#).as_deref(),
            Some("harmonic.orb")
        );
        assert_eq!(field("{}").as_deref(), Some("harmonic"));
        let wide = HarmonicRequest::from_json(r#"{"number": 360, "orbDeg": 30}"#).unwrap();
        assert_eq!(wide, HarmonicRequest::of(360).with_orb_deg(30.0));
        let twice = [
            PlanetAt::new(Graha::Sun, 1.0),
            PlanetAt::new(Graha::Sun, 2.0),
        ];
        assert_eq!(
            harmonic_chart(&twice, 0.0, 0.0, &HarmonicRequest::of(2))
                .unwrap_err()
                .field(),
            Some("bodies")
        );
        assert_eq!(
            harmonic_chart(&[], f64::NAN, 0.0, &HarmonicRequest::of(2))
                .unwrap_err()
                .field(),
            Some("ascendantDeg")
        );
    }
}
