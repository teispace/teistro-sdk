//! The teva: the nine planets by Lal Kitab house (1952 pp. ~6–7).
//!
//! Whatever sign rises, the lagna's box is house 1 and the boxes run on in
//! order; the planets stay where they are and the signs are dropped, so a
//! planet's house is its whole-sign house from the lagna (crux LK-C1).
//! House `n` is then read as rashi `n` for every table that names a sign.

use serde::Serialize;
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::error::Error;

use crate::tables::{PLANETS, house_of_sign, reads};

/// The nine planets, each in one of the twelve houses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Teva {
    /// Each planet's house, in [`PLANETS`] order.
    houses: [u8; 9],
}

impl Teva {
    /// The teva of a chart: each planet's whole-sign house from the
    /// lagna's sign.
    ///
    /// ```
    /// use teistro_core::catalogue::{Graha, Rashi};
    /// use teistro_lalkitab::Teva;
    ///
    /// // A Libra lagna: the planet in Libra is in house 1, Aries's in 7.
    /// let mut signs = vec![
    ///     (Graha::Sun, Rashi::Libra),
    ///     (Graha::Moon, Rashi::Aries),
    /// ];
    /// signs.extend(
    ///     [Graha::Mars, Graha::Mercury, Graha::Jupiter, Graha::Venus, Graha::Saturn, Graha::Rahu, Graha::Ketu]
    ///         .map(|graha| (graha, Rashi::Cancer)),
    /// );
    /// let teva = Teva::from_signs(Rashi::Libra, signs)?;
    /// assert_eq!(teva.house_of(Graha::Sun), Some(1));
    /// assert_eq!(teva.house_of(Graha::Moon), Some(7));
    /// assert_eq!(teva.house_of(Graha::Mars), Some(10));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`Teva::from_houses`] refuses the placements.
    pub fn from_signs(
        lagna: Rashi,
        signs: impl IntoIterator<Item = (Graha, Rashi)>,
    ) -> Result<Teva, Error> {
        let rising = house_of_sign(lagna);
        Teva::from_houses(signs.into_iter().map(|(graha, sign)| {
            let house = (house_of_sign(sign) + 12 - rising) % 12 + 1;
            (graha, house)
        }))
    }

    /// A teva from each planet's house, as a reader who already has one
    /// writes it.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming its field: a graha Lal Kitab does not read, a
    /// planet given twice, or a house outside 1 to 12 (`placements[i]`); a
    /// planet missing (`placements`, naming it).
    pub fn from_houses(placements: impl IntoIterator<Item = (Graha, u8)>) -> Result<Teva, Error> {
        let mut houses = [0_u8; 9];
        for (index, (graha, house)) in placements.into_iter().enumerate() {
            let field = || format!("placements[{index}]");
            let slot = PLANETS
                .iter()
                .position(|planet| *planet == graha)
                .and_then(|at| houses.get_mut(at))
                .filter(|_| reads(graha))
                .ok_or_else(|| {
                    Error::invalid_arg(format!(
                        "Lal Kitab reads the nine grahas, and {} is not one",
                        graha.key()
                    ))
                    .with_field(field())
                })?;
            if *slot != 0 {
                return Err(Error::invalid_arg(format!(
                    "{} is placed twice; a teva places each planet once",
                    graha.key()
                ))
                .with_field(field()));
            }
            if !(1..=12).contains(&house) {
                return Err(
                    Error::invalid_arg(format!("a house is 1 to 12, not {house}"))
                        .with_field(field()),
                );
            }
            *slot = house;
        }
        if let Some((graha, _)) = PLANETS.iter().zip(&houses).find(|(_, house)| **house == 0) {
            return Err(Error::invalid_arg(format!(
                "{} is not placed; a teva places all nine planets",
                graha.key()
            ))
            .with_field("placements"));
        }
        Ok(Teva { houses })
    }

    /// The house a planet is in; `None` for a graha Lal Kitab does not read.
    #[must_use]
    pub fn house_of(&self, graha: Graha) -> Option<u8> {
        PLANETS
            .iter()
            .zip(&self.houses)
            .find(|(planet, _)| **planet == graha)
            .map(|(_, house)| *house)
    }

    /// The planets in a house, in [`PLANETS`] order.
    #[must_use]
    pub fn occupants(&self, house: u8) -> Vec<Graha> {
        self.placements()
            .filter(|(_, at)| *at == house)
            .map(|(graha, _)| graha)
            .collect()
    }

    /// Whether a house holds any planet.
    #[must_use]
    pub fn occupied(&self, house: u8) -> bool {
        self.houses.contains(&house)
    }

    /// Every planet with its house, in [`PLANETS`] order.
    pub fn placements(&self) -> impl Iterator<Item = (Graha, u8)> + '_ {
        PLANETS.iter().copied().zip(self.houses.iter().copied())
    }

    /// The same teva with every planet moved as `moved` says of its house:
    /// what an annual chart does, the occupants of a house moving together.
    pub(crate) fn moved(&self, moved: impl Fn(u8) -> u8) -> Teva {
        Teva {
            houses: self.houses.map(&moved),
        }
    }
}
