//! Declinations and parallels (`03-design/western-declinations.md`).
//!
//! Alan Leo adds to the aspects "the Parallel of Declination, when the two
//! bodies are the same distance, whether north or south, from the equator"
//! (*How to Judge a Nativity*, p. 42), and allows it 1° (p. 47). A pair on
//! opposite sides of the equator is a parallel for him (C243); each row
//! says so, for a reader who splits the two.
//!
//! ```
//! use teistro_core::catalogue::Graha;
//! use teistro_western::{Declined, ParallelRequest, parallels};
//!
//! let bodies = [
//!     Declined::new(Graha::Moon, -2.67),
//!     Declined::new(Graha::Mercury, 14.17),
//!     Declined::new(Graha::Venus, 13.28),
//! ];
//! let found = parallels(&bodies, &ParallelRequest::default())?;
//! assert_eq!(found.len(), 1);
//! assert_eq!((found[0].first, found[0].second), (Graha::Mercury, Graha::Venus));
//! assert!(!found[0].contrary);
//! assert!((found[0].apart_deg - 0.89).abs() < 1e-9);
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;

use crate::aspects::refuse_repeats;

/// The record's name where a binding sends it, which a refusal is named
/// under.
const ROOT: &str = "parallels";

/// Leo's orb for the parallel (p. 47).
pub const LEO_PARALLEL_ORB_DEG: f64 = 1.0;

/// The widest orb a request may ask: beyond it, every pair of planets near
/// the ecliptic would stand in parallel with every other.
pub const MAX_PARALLEL_ORB_DEG: f64 = 10.0;

/// A body as the parallels read it: its distance from the equator.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Declined {
    /// Which.
    pub graha: Graha,
    /// Its declination, degrees, north positive.
    pub declination_deg: f64,
}

impl Declined {
    /// A body at a declination.
    #[must_use]
    pub const fn new(graha: Graha, declination_deg: f64) -> Declined {
        Declined {
            graha,
            declination_deg,
        }
    }
}

/// What the parallels are asked: how close two distances from the equator
/// must stand.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct ParallelRequest {
    /// The orb, degrees: Leo's 1° unless a caller says otherwise.
    pub orb_deg: f64,
}

impl Default for ParallelRequest {
    fn default() -> ParallelRequest {
        ParallelRequest {
            orb_deg: LEO_PARALLEL_ORB_DEG,
        }
    }
}

impl ParallelRequest {
    /// Holds a parallel to this orb, degrees.
    #[must_use]
    pub const fn with_orb_deg(mut self, orb_deg: f64) -> ParallelRequest {
        self.orb_deg = orb_deg;
        self
    }

    /// The request a binding sends, as JSON: `{"orbDeg": 1}`, or `{}` for
    /// Leo's.
    ///
    /// ```
    /// use teistro_western::ParallelRequest;
    ///
    /// assert_eq!(ParallelRequest::from_json("{}")?.orb_deg, 1.0);
    /// let wide = ParallelRequest::from_json(r#"{"orbDeg": 30}"#).unwrap_err();
    /// assert_eq!(wide.field(), Some("parallels.orbDeg"));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, and whatever [`ParallelRequest::check`] refuses, each named
    /// under `parallels`.
    pub fn from_json(text: &str) -> Result<ParallelRequest, Error> {
        let asked: ParallelRequest = teistro_core::strict::read(text, ROOT)?;
        asked.check().map_err(|why| why.under(ROOT))?;
        Ok(asked)
    }

    /// Refuses an orb that is not a positive number of degrees up to
    /// [`MAX_PARALLEL_ORB_DEG`].
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming `orbDeg`.
    pub fn check(&self) -> Result<(), Error> {
        if self.orb_deg.is_finite() && self.orb_deg > 0.0 && self.orb_deg <= MAX_PARALLEL_ORB_DEG {
            return Ok(());
        }
        Err(Error::invalid_arg(format!(
            "a parallel's orb is more than 0° and at most {MAX_PARALLEL_ORB_DEG}°, not {}",
            self.orb_deg
        ))
        .with_field("orbDeg")
        .with_hint(format!("Leo allows {LEO_PARALLEL_ORB_DEG}° (p. 47)")))
    }
}

/// One pair of bodies the same distance from the equator, within the orb.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParallelRow {
    /// The first body, in the order the bodies were given.
    pub first: Graha,
    /// The second.
    pub second: Graha,
    /// Whether the two stand on opposite sides of the equator: the
    /// contra-parallel, which Leo counts a parallel (C243).
    pub contrary: bool,
    /// How far apart their distances from the equator are, degrees.
    pub apart_deg: f64,
    /// The orb the request allowed, degrees.
    pub orb_deg: f64,
}

/// Every pair of bodies in parallel, closest first.
///
/// A pair is read once, in the order the bodies are given. Two distances
/// from the equator within the orb are a parallel, on either side of it.
///
/// # Errors
///
/// What [`ParallelRequest::check`] refuses; a body given twice; a
/// declination that is not a finite number of degrees between the poles.
pub fn parallels(
    bodies: &[Declined],
    request: &ParallelRequest,
) -> Result<Vec<ParallelRow>, Error> {
    request.check()?;
    refuse_repeats(bodies.iter().map(|one| one.graha.key()), "a body")
        .map_err(|why| why.with_field("bodies"))?;
    if let Some((at, one)) = bodies
        .iter()
        .enumerate()
        .find(|(_, one)| !(one.declination_deg.is_finite() && one.declination_deg.abs() <= 90.0))
    {
        return Err(Error::invalid_arg(format!(
            "a declination is between -90° and 90°, not {}",
            one.declination_deg
        ))
        .with_field(format!("bodies[{at}].declinationDeg")));
    }
    let mut rows: Vec<ParallelRow> = bodies
        .iter()
        .enumerate()
        .flat_map(|(at, first)| {
            bodies
                .iter()
                .skip(at + 1)
                .map(move |second| (first, second))
        })
        .filter_map(|(first, second)| {
            let apart_deg = (first.declination_deg.abs() - second.declination_deg.abs()).abs();
            (apart_deg <= request.orb_deg).then_some(ParallelRow {
                first: first.graha,
                second: second.graha,
                contrary: first.declination_deg * second.declination_deg < 0.0,
                apart_deg,
                orb_deg: request.orb_deg,
            })
        })
        .collect();
    // Stable, so pairs equally close keep the order the bodies were given.
    rows.sort_by(|a, b| a.apart_deg.total_cmp(&b.apart_deg));
    Ok(rows)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;

    #[test]
    fn either_side_of_the_equator_is_a_parallel_and_says_which() {
        let bodies = [
            Declined::new(Graha::Moon, -2.6727),
            Declined::new(Graha::Sun, 22.2997),
            Declined::new(Graha::Jupiter, -22.9401),
            Declined::new(Graha::Neptune, 2.6495),
        ];
        let found = parallels(&bodies, &ParallelRequest::default()).unwrap();
        let pairs: Vec<(Graha, Graha, bool)> = found
            .iter()
            .map(|row| (row.first, row.second, row.contrary))
            .collect();
        assert_eq!(
            pairs,
            [
                (Graha::Moon, Graha::Neptune, true),
                (Graha::Sun, Graha::Jupiter, true)
            ]
        );
        assert!(found.iter().all(|row| row.apart_deg <= row.orb_deg));
    }

    #[test]
    fn the_orb_is_the_callers_and_held_to_its_bounds() {
        let bodies = [
            Declined::new(Graha::Sun, 10.0),
            Declined::new(Graha::Mars, 11.5),
        ];
        let none = parallels(&bodies, &ParallelRequest::default()).unwrap();
        assert!(none.is_empty(), "{none:?}");
        let wider = ParallelRequest::default().with_orb_deg(2.0);
        assert_eq!(parallels(&bodies, &wider).unwrap().len(), 1);
        for orb in [0.0, -1.0, 10.5, f64::NAN] {
            let refused =
                parallels(&bodies, &ParallelRequest::default().with_orb_deg(orb)).unwrap_err();
            assert_eq!(refused.field(), Some("orbDeg"), "{orb}");
        }
    }

    #[test]
    fn a_body_twice_or_past_a_pole_is_refused_by_field() {
        let twice = [
            Declined::new(Graha::Sun, 1.0),
            Declined::new(Graha::Sun, 2.0),
        ];
        let refused = parallels(&twice, &ParallelRequest::default()).unwrap_err();
        assert_eq!(refused.field(), Some("bodies"));
        let past = [
            Declined::new(Graha::Sun, 1.0),
            Declined::new(Graha::Moon, 91.0),
        ];
        let refused = parallels(&past, &ParallelRequest::default()).unwrap_err();
        assert_eq!(refused.field(), Some("bodies[1].declinationDeg"));
    }

    #[test]
    fn the_record_reads_back_what_it_writes() {
        let asked = ParallelRequest::default().with_orb_deg(1.5);
        let text = serde_json::to_string(&asked).unwrap();
        assert_eq!(text, r#"{"orbDeg":1.5}"#);
        assert_eq!(ParallelRequest::from_json(&text).unwrap(), asked);
        let typo = ParallelRequest::from_json(r#"{"orb": 1}"#).unwrap_err();
        assert_eq!(typo.field(), Some("parallels.orb"));
    }
}
