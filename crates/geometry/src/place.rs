//! A chart placed in a layout: every cell with both its sign and its house,
//! and the bodies that stand in it.
//!
//! A sign-fixed cell gets its house from the lagna and a house-fixed cell its
//! sign, so a consumer never repeats the arithmetic, and a divisional chart
//! places exactly as a rashi chart does, from its own lagna
//! (`03-design/chart-geometry.md` §4).

use serde::{Deserialize, Serialize};
use teistro_chart::foundation::ChartFoundation;
use teistro_core::catalogue::Rashi;
use teistro_core::key::KeyId;
use teistro_vargas::chart::VargaChart;

use crate::layout::{Holds, Layout};
use crate::path::{Path, Point};

/// What placement reads of a chart: the lagna's sign, and each body's key and
/// sign in the order the chart lists them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placements {
    /// The sign the lagna is in.
    pub lagna: Rashi,
    /// Each body and the sign it stands in.
    pub bodies: Vec<(KeyId, Rashi)>,
}

impl Placements {
    /// A founded chart's lagna and grahas.
    #[must_use]
    pub fn of_chart(chart: &ChartFoundation) -> Placements {
        Placements {
            lagna: sign_of(chart.lagna_sign_index()),
            bodies: chart
                .grahas
                .iter()
                .map(|body| (body.graha.key_id(), sign_of(body.sign_index())))
                .collect(),
        }
    }

    /// A divisional chart's own lagna and grahas.
    #[must_use]
    pub fn of_varga(chart: &VargaChart) -> Placements {
        Placements {
            lagna: chart.lagna.sign,
            bodies: chart
                .grahas
                .iter()
                .map(|placed| (placed.graha.key_id(), placed.at.sign))
                .collect(),
        }
    }
}

fn sign_of(index: u8) -> Rashi {
    Rashi::from_id(u16::from(index) % 12).unwrap_or(Rashi::Aries)
}

/// The sign a house is, counting inclusively from the lagna's sign.
fn sign_after(lagna: u16, house: u8) -> Rashi {
    Rashi::from_id((lagna + u16::from(house) + 11) % 12).unwrap_or(Rashi::Aries)
}

/// One cell of a placed chart.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PlacedCell {
    /// The cell's outline.
    pub outline: Path,
    /// The sign the cell shows.
    pub sign: Rashi,
    /// The house the cell shows, 1 to 12, counted from the lagna's sign.
    pub house: u8,
    /// Whether the lagna stands in this cell.
    pub lagna: bool,
    /// Where the sign or house number is drawn.
    pub label: Point,
    /// Where the bodies are stacked about.
    pub anchor: Point,
    /// The bodies standing in the cell, in the chart's order.
    pub bodies: Vec<KeyId>,
}

/// A chart placed in a layout: what a renderer draws, in any language.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Placed {
    /// The layout's key.
    pub layout: String,
    /// The cells, in the layout's order.
    pub cells: Vec<PlacedCell>,
    /// The lines drawn that hold nothing.
    pub frame: Vec<Path>,
}

/// Places a chart in a layout.
///
/// The layout is assumed valid, as every shipped and registered one is
/// ([`Layout::validate`]), so placement cannot fail: it only chooses.
///
/// ```
/// use teistro_core::catalogue::{Catalogued, Graha, Rashi};
/// use teistro_geometry::{Placements, place, rows};
///
/// // A Cancer lagna with the Sun in Leo, in the North Indian chart.
/// let chart = Placements {
///     lagna: Rashi::Cancer,
///     bodies: vec![(Graha::Sun.key_id(), Rashi::Leo)],
/// };
/// let placed = place(&rows::north_indian(), &chart);
///
/// // House 1 is the top diamond and shows the lagna's sign; the Sun is in
/// // the second house.
/// assert_eq!((placed.cells[0].house, placed.cells[0].sign), (1, Rashi::Cancer));
/// assert!(placed.cells[0].lagna);
/// assert_eq!(placed.cells[1].bodies, vec![Graha::Sun.key_id()]);
/// ```
#[must_use]
pub fn place(layout: &Layout, chart: &Placements) -> Placed {
    let lagna = chart.lagna.id();
    let cells = layout
        .grid
        .cells
        .iter()
        .map(|cell| {
            let (sign, house) = match cell.holds {
                Holds::Sign(sign) => (sign, house_of(sign.id(), lagna)),
                Holds::House(house) => (sign_after(lagna, house), house),
            };
            PlacedCell {
                outline: cell.outline.clone(),
                sign,
                house,
                lagna: sign.id() == lagna,
                label: cell.label,
                anchor: cell.bodies,
                bodies: chart
                    .bodies
                    .iter()
                    .filter(|(_, at)| *at == sign)
                    .map(|(key, _)| *key)
                    .collect(),
            }
        })
        .collect();
    Placed {
        layout: layout.key.clone(),
        cells,
        frame: layout.grid.frame.clone(),
    }
}

/// The house a sign is, counted inclusively from the lagna's sign.
fn house_of(sign: u16, lagna: u16) -> u8 {
    u8::try_from((sign + 12 - lagna) % 12 + 1).unwrap_or(1)
}
