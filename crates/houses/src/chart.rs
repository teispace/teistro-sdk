//! The service: the twelve bhavas of a founded chart, both readings of
//! where each body stands, and whether the chart can be trusted.

use serde::{Deserialize, Serialize};
use teistro_astro::houses::Outcome;
use teistro_chart::bhava::Bhavas;
use teistro_chart::foundation::ChartFoundation;
use teistro_core::angle::Nas;
use teistro_core::catalogue::{Graha, HouseSystem, Rashi};
use teistro_core::error::Error;
use teistro_core::quantity::Degrees;

use crate::classify::{self, HOUSES, Quadrant};

/// One bhava of a chart: where it is, whose it is, and what kind it is.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bhava {
    /// The bhava, 1 to 12.
    pub number: u8,
    /// The sign its **middle** falls in, which is the sign the tradition
    /// means by "the house's sign": under an unequal division a house
    /// can begin in one sign and be centred in another.
    pub sign: Rashi,
    /// The lord of that sign.
    pub lord: Graha,
    /// Its middle, degrees.
    pub madhya_deg: f64,
    /// Where it begins, degrees.
    pub sandhi_deg: f64,
    /// Which third of the wheel it stands in.
    pub quadrant: Quadrant,
}

impl Bhava {
    /// Whether it is a trine.
    #[must_use]
    pub fn is_trikona(&self) -> bool {
        classify::is_trikona(self.number)
    }

    /// Whether it is a house of difficulty.
    #[must_use]
    pub fn is_dusthana(&self) -> bool {
        classify::is_dusthana(self.number)
    }

    /// Whether it grows better with time.
    #[must_use]
    pub fn is_upachaya(&self) -> bool {
        classify::is_upachaya(self.number)
    }
}

/// Where one body stands under both readings of a chart.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Placed {
    /// Which body.
    pub graha: Graha,
    /// Its house under the chart's placement system.
    pub placement: u8,
    /// Its house under the chart's chalit.
    pub chalit: u8,
}

impl Placed {
    /// Whether the two readings disagree, which is what the recording
    /// engine records as a "shifted" body.
    #[must_use]
    pub const fn shifted(&self) -> bool {
        self.placement != self.chalit
    }
}

/// The houses of one founded chart, under both readings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Houses {
    bhavas: [Bhava; 12],
    placed: Vec<Placed>,
    placement_system: HouseSystem,
    chalit_system: HouseSystem,
    outcome: Outcome,
}

impl Houses {
    /// The houses of a founded chart.
    ///
    /// Both readings are already on the foundation — this does not
    /// recompute cusps that `chart` has placed. The outcome is the
    /// caller's to supply, because whether a system was computed as
    /// asked is a fact about the computation that made the foundation
    /// and not about the foundation itself; [`Houses::of`] takes
    /// [`Outcome::Defined`] for a chart that came back without trouble.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for a foundation carrying a longitude that is not
    /// a finite number.
    pub fn of(foundation: &ChartFoundation) -> Result<Houses, Error> {
        Houses::with_outcome(foundation, Outcome::Defined)
    }

    /// The same, with the outcome the house computation reported.
    ///
    /// A degenerate chart is **not** an error: the outcome is reported
    /// and the caller decides whether to trust it, which is what makes
    /// the polar policy mean anything
    /// (`03-design/houses-service.md` §5).
    ///
    /// # Errors
    ///
    /// As [`Houses::of`].
    pub fn with_outcome(foundation: &ChartFoundation, outcome: Outcome) -> Result<Houses, Error> {
        let bhavas = bhavas_of(&foundation.houses)?;
        let mut placed = Vec::with_capacity(foundation.grahas.len());
        for position in &foundation.grahas {
            placed.push(Placed {
                graha: position.graha,
                placement: position.house.bhava,
                chalit: position.placement.bhava,
            });
        }
        Ok(Houses {
            bhavas,
            placed,
            placement_system: foundation.houses.chalit.method,
            chalit_system: foundation.chalit.chalit.method,
            outcome,
        })
    }

    /// The twelve bhavas, the first one first.
    #[must_use]
    pub const fn all(&self) -> &[Bhava; 12] {
        &self.bhavas
    }

    /// One bhava, or `None` outside the twelve.
    #[must_use]
    pub fn bhava(&self, number: u8) -> Option<&Bhava> {
        self.bhavas.get(usize::from(number.checked_sub(1)?))
    }

    /// Where a body stands under both readings.
    #[must_use]
    pub fn placed(&self, graha: Graha) -> Option<&Placed> {
        self.placed.iter().find(|placed| placed.graha == graha)
    }

    /// Every body, under both readings.
    #[must_use]
    pub fn bodies(&self) -> &[Placed] {
        &self.placed
    }

    /// The bodies the chalit moves out of their placement house, which
    /// is what a chart display draws differently and what the recording
    /// engine records as its `shifted` list.
    pub fn moved(&self) -> impl Iterator<Item = &Placed> {
        self.placed.iter().filter(|placed| placed.shifted())
    }

    /// Whether the system asked for was computed, substituted or
    /// clamped.
    #[must_use]
    pub const fn outcome(&self) -> Outcome {
        self.outcome
    }

    /// Whether the chart's houses are the ones asked for.
    #[must_use]
    pub fn is_defined(&self) -> bool {
        self.outcome == Outcome::Defined
    }

    /// The system the placements were made under.
    #[must_use]
    pub const fn placement_system(&self) -> HouseSystem {
        self.placement_system
    }

    /// The chalit the second reading was made under.
    #[must_use]
    pub const fn chalit_system(&self) -> HouseSystem {
        self.chalit_system
    }

    /// The bhavas of a kind, in order.
    pub fn quadrant(&self, quadrant: Quadrant) -> impl Iterator<Item = &Bhava> {
        self.bhavas
            .iter()
            .filter(move |bhava| bhava.quadrant == quadrant)
    }
}

/// The twelve bhavas of a chart's own division.
fn bhavas_of(bhavas: &Bhavas) -> Result<[Bhava; 12], Error> {
    let mut found = Vec::with_capacity(usize::from(HOUSES));
    for (index, (madhya, sandhi)) in bhavas.madhya.iter().zip(&bhavas.sandhi).enumerate() {
        let number = u8::try_from(index + 1).unwrap_or(1);
        let madhya = at(*madhya)?;
        let sign = madhya.sign();
        found.push(Bhava {
            number,
            sign,
            lord: classify::lord_of(sign),
            madhya_deg: madhya.to_degrees(),
            sandhi_deg: at(*sandhi)?.to_degrees(),
            quadrant: classify::quadrant(number).unwrap_or(Quadrant::Kendra),
        });
    }
    found.try_into().map_err(|_| {
        Error::invalid_arg(String::from("a chart's division must have twelve bhavas"))
            .with_field("houses.bhavas")
    })
}

/// A longitude the foundation carries, as the canonical angle.
fn at(degrees: f64) -> Result<Nas, Error> {
    Ok(Nas::from_degrees(Degrees::try_new(
        degrees.rem_euclid(360.0),
    )?))
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index a fixed-length division"
    )]

    use super::{Placed, at, bhavas_of};
    use crate::classify::Quadrant;
    use teistro_chart::bhava::{Bhavas, Chalit};
    use teistro_core::catalogue::{Graha, HouseSystem, Rashi};

    fn equal_from(ascendant: f64) -> Bhavas {
        let cusps: [f64; 12] = core::array::from_fn(|index| {
            (ascendant + f64::from(u8::try_from(index).unwrap_or(0)) * 30.0).rem_euclid(360.0)
        });
        Bhavas::of(Chalit::of(HouseSystem::Equal), &cusps)
    }

    #[test]
    fn a_bhava_takes_the_sign_its_middle_falls_in() {
        // An equal division from 25° Aries: the first house runs 25° to
        // 55°, so it begins in Aries and is centred in Taurus.
        let found = bhavas_of(&equal_from(25.0)).unwrap();
        assert_eq!(found[0].number, 1);
        assert!((found[0].sandhi_deg - 25.0).abs() < 1e-9);
        assert!((found[0].madhya_deg - 40.0).abs() < 1e-9);
        assert_eq!(found[0].sign, Rashi::Taurus, "the sign of the middle");
        assert_eq!(found[0].lord, Graha::Venus);
    }

    #[test]
    fn the_twelve_come_back_numbered_and_classified() {
        let found = bhavas_of(&equal_from(0.0)).unwrap();
        for (index, bhava) in found.iter().enumerate() {
            assert_eq!(bhava.number, u8::try_from(index + 1).unwrap());
            assert_eq!(
                bhava.quadrant,
                crate::classify::quadrant(bhava.number).unwrap()
            );
            assert!((0.0..360.0).contains(&bhava.madhya_deg));
            assert!((0.0..360.0).contains(&bhava.sandhi_deg));
        }
        assert_eq!(found[0].quadrant, Quadrant::Kendra);
        assert!(found[0].is_trikona() && !found[0].is_dusthana());
        assert!(found[5].is_dusthana() && found[5].is_upachaya());
    }

    #[test]
    fn a_body_shifts_when_the_two_readings_disagree() {
        let same = Placed {
            graha: Graha::Sun,
            placement: 4,
            chalit: 4,
        };
        let moved = Placed {
            graha: Graha::Mars,
            placement: 12,
            chalit: 11,
        };
        assert!(!same.shifted());
        assert!(moved.shifted());
    }

    #[test]
    fn a_longitude_is_wrapped_and_a_nan_is_refused() {
        assert!((at(370.0).unwrap().to_degrees() - 10.0).abs() < 1e-9);
        assert!(at(f64::NAN).is_err());
    }
}
