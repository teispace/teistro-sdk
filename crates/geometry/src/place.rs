//! A chart placed in a layout: every cell with both its sign and its house,
//! and the bodies that stand in it.
//!
//! A sign-fixed cell gets its house from the lagna and a house-fixed cell its
//! sign, so a consumer never repeats the arithmetic, and a divisional chart
//! places exactly as a rashi chart does, from its own lagna
//! (`03-design/chart-geometry.md` §4).

use serde::{Deserialize, Serialize};
use teistro_chart::foundation::ChartFoundation;
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::error::Error;
use teistro_core::key::KeyId;
use teistro_vargas::chart::VargaChart;

use crate::clock;
use crate::layout::{Direction, Grid, Holds, Layout, Radial, Reference, Shape};
use crate::path::{Path, Point, Segment};

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
    /// The ring the cell belongs to, innermost 0; a grid's cells are all 0.
    pub ring: u8,
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
/// ([`Layout::validate`]).
///
/// # Errors
///
/// A radial ring that counts from the Moon or the Sun, for a chart that lists
/// no such body: counting it from the lagna instead would draw a chakra that
/// is not the one asked for.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_geometry::{Placements, place, rows};
///
/// // A Cancer lagna with the Sun in Leo, in the North Indian chart.
/// let chart = Placements {
///     lagna: Rashi::Cancer,
///     bodies: vec![(Graha::Sun.key_id(), Rashi::Leo)],
/// };
/// let placed = place(&rows::north_indian(), &chart)?;
///
/// // House 1 is the top diamond and shows the lagna's sign; the Sun is in
/// // the second house.
/// assert_eq!((placed.cells[0].house, placed.cells[0].sign), (1, Rashi::Cancer));
/// assert!(placed.cells[0].lagna);
/// assert_eq!(placed.cells[1].bodies, vec![Graha::Sun.key_id()]);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
pub fn place(layout: &Layout, chart: &Placements) -> Result<Placed, Error> {
    let (cells, frame) = match &layout.shape {
        Shape::Grid(grid) => (place_grid(grid, chart), grid.frame.clone()),
        Shape::Radial(radial) => (
            place_radial(&layout.key, radial, chart)?,
            radial_frame(radial),
        ),
    };
    Ok(Placed {
        layout: layout.key.clone(),
        cells,
        frame,
    })
}

fn place_grid(grid: &Grid, chart: &Placements) -> Vec<PlacedCell> {
    let lagna = chart.lagna.id();
    grid.cells
        .iter()
        .map(|cell| {
            let (sign, house) = match cell.holds {
                Holds::Sign(sign) => (sign, house_of(sign.id(), lagna)),
                Holds::House(house) => (sign_after(lagna, house), house),
            };
            placed_cell(
                chart,
                cell.outline.clone(),
                sign,
                house,
                0,
                cell.label,
                cell.bodies,
            )
        })
        .collect()
}

/// The square's centre, which every ring turns about.
const CENTRE: Point = Point::new(0.5, 0.5);

fn place_radial(key: &str, radial: &Radial, chart: &Placements) -> Result<Vec<PlacedCell>, Error> {
    let mut cells = Vec::with_capacity(radial.rings.len() * 12);
    for (index, ring) in (0u8..).zip(&radial.rings) {
        let from = match ring.counts_from {
            Reference::Lagna => chart.lagna,
            Reference::Moon => sign_of_body(key, chart, Graha::Moon)?,
            Reference::Sun => sign_of_body(key, chart, Graha::Sun)?,
        };
        let (width, middle) = (
            ring.outer - ring.inner,
            f64::midpoint(ring.inner, ring.outer),
        );
        for house in 1..=12u8 {
            let begins = hour_of(radial, house);
            let ends = hour_of(radial, house + 1);
            // The half hour between the two, which is where the labels go.
            let halfway = match radial.direction {
                Direction::Clockwise => begins * 2 + 1,
                Direction::Anticlockwise => (begins * 2 + 23) % 24,
            };
            cells.push(placed_cell(
                chart,
                sector(ring.inner, ring.outer, begins, ends, radial.direction),
                sign_after(from.id(), house),
                house,
                index,
                clock::at(CENTRE, middle - width * 0.18, halfway),
                clock::at(CENTRE, middle + width * 0.18, halfway),
            ));
        }
    }
    Ok(cells)
}

/// The hour a house's sector starts on, in hours clockwise from twelve.
fn hour_of(radial: &Radial, house: u8) -> u32 {
    let start = u32::from(radial.starts_at % 12);
    let steps = u32::from(house - 1) % 12;
    match radial.direction {
        Direction::Clockwise => (start + steps) % 12,
        Direction::Anticlockwise => (start + 12 - steps) % 12,
    }
}

/// One sector: out along the outer arc, in along a spoke, back along the inner
/// arc; a wedge to the centre when the ring has no hole.
fn sector(inner: f64, outer: f64, begins: u32, ends: u32, direction: Direction) -> Path {
    let clockwise = direction == Direction::Clockwise;
    let (begin, end) = (begins * 2, ends * 2);
    let mut segments = vec![Segment::Arc {
        centre: CENTRE,
        clockwise,
        to: clock::at(CENTRE, outer, end),
    }];
    if inner > 0.0 {
        segments.push(Segment::Line {
            to: clock::at(CENTRE, inner, end),
        });
        segments.push(Segment::Arc {
            centre: CENTRE,
            clockwise: !clockwise,
            to: clock::at(CENTRE, inner, begin),
        });
    } else {
        segments.push(Segment::Line { to: CENTRE });
    }
    Path {
        start: clock::at(CENTRE, outer, begin),
        segments,
    }
}

/// A radial layout draws its rings' circles, each once where two rings
/// share one, since a stroke drawn twice darkens; its spokes are the
/// sectors' own edges.
fn radial_frame(radial: &Radial) -> Vec<Path> {
    let mut radii: Vec<f64> = Vec::with_capacity(radial.rings.len() * 2);
    for radius in radial
        .rings
        .iter()
        .flat_map(|ring| [ring.inner, ring.outer])
    {
        if radius > 0.0 && radii.last() != Some(&radius) {
            radii.push(radius);
        }
    }
    radii.into_iter().map(circle).collect()
}

fn circle(radius: f64) -> Path {
    let top = clock::at(CENTRE, radius, 0);
    Path {
        start: top,
        segments: vec![
            Segment::Arc {
                centre: CENTRE,
                clockwise: true,
                to: clock::at(CENTRE, radius, 12),
            },
            Segment::Arc {
                centre: CENTRE,
                clockwise: true,
                to: top,
            },
        ],
    }
}

fn sign_of_body(key: &str, chart: &Placements, graha: Graha) -> Result<Rashi, Error> {
    let wanted = graha.key_id();
    chart
        .bodies
        .iter()
        .find(|(body, _)| *body == wanted)
        .map(|(_, sign)| *sign)
        .ok_or_else(|| {
            Error::invalid_arg(format!(
                "the {key} layout has a ring that counts from the {}, and the chart lists no {}",
                graha.key(),
                graha.full_key()
            ))
            .with_field("bodies")
        })
}

fn placed_cell(
    chart: &Placements,
    outline: Path,
    sign: Rashi,
    house: u8,
    ring: u8,
    label: Point,
    anchor: Point,
) -> PlacedCell {
    PlacedCell {
        outline,
        sign,
        house,
        lagna: sign == chart.lagna,
        ring,
        label,
        anchor,
        bodies: chart
            .bodies
            .iter()
            .filter(|(_, at)| *at == sign)
            .map(|(key, _)| *key)
            .collect(),
    }
}

/// The house a sign is, counted inclusively from the lagna's sign.
fn house_of(sign: u16, lagna: u16) -> u8 {
    u8::try_from((sign + 12 - lagna) % 12 + 1).unwrap_or(1)
}
