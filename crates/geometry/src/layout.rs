//! A layout as a row of data, and the checks that refuse a wrong one.
//!
//! Layouts are two kinds (`03-design/chart-geometry.md` §2):
//!
//! - a **grid** is twelve cells fixed in the row. Each holds a sign (a
//!   sign-fixed layout, South and East Indian) or a house (a house-fixed
//!   layout, North Indian and the lotus), and placement computes the other
//!   from the lagna;
//! - a **radial** layout is rings of twelve sectors, one clock hour each,
//!   each ring counting its houses from a reference of its own (the lagna,
//!   the Moon, the Sun), so its sectors are computed per chart.
//!
//! A row is refused, not believed. [`Layout::validate`] names the cell it
//! gets wrong, so a consumer's regional layout fails as loudly as a shipped
//! one would (`03-design/chart-geometry.md` §5).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Rashi;
use teistro_core::error::Error;

use crate::path::{Path, Point};

/// What a grid cell carries, which is what the layout keeps fixed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind", content = "value", rename_all = "lowercase")]
pub enum Holds {
    /// The cell is always this sign; its house moves with the lagna.
    Sign(Rashi),
    /// The cell is always this house, 1 to 12; its sign moves with the
    /// lagna.
    House(u8),
}

/// The way signs or houses run around a layout, as a reader sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    /// With the hands of a clock.
    Clockwise,
    /// Against them.
    Anticlockwise,
}

/// One region of a grid layout.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Cell {
    /// The region's outline.
    pub outline: Path,
    /// The sign or house the cell always carries.
    pub holds: Holds,
    /// Where the sign or house number is drawn.
    pub label: Point,
    /// Where the cell's bodies are stacked about.
    pub bodies: Point,
}

/// A grid layout: twelve cells, fixed in the row.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Grid {
    /// The twelve cells, in the order the row lists them.
    pub cells: Vec<Cell>,
    /// Lines that are drawn and hold nothing: the border, a divider.
    pub frame: Vec<Path>,
    /// The way the signs or houses run.
    pub direction: Direction,
}

/// What a radial ring counts its first house from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum Reference {
    /// The lagna's sign.
    Lagna,
    /// The Moon's sign.
    Moon,
    /// The Sun's sign.
    Sun,
    /// The chart's cusps: each house a sector from its cusp to the next, as
    /// wide as the house is, the lagna's cusp at the start hour.
    Cusps,
    /// The zodiac itself: twelve signs of 30° each, turned so the lagna's
    /// degree sits at the start hour.
    Zodiac,
}

/// One ring of a radial layout: an annulus about the square's centre, cut
/// into twelve sectors. A ring counting from a sign cuts them one clock hour
/// each; a ring of cusps or of the zodiac cuts them where the chart's
/// longitudes fall.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Ring {
    /// The inner radius, a fraction of the square's side; 0 makes wedges.
    pub inner: f64,
    /// The outer radius, at most a half.
    pub outer: f64,
    /// What the ring counts its first house from.
    pub counts_from: Reference,
}

/// A radial layout: rings of sectors, from the innermost outwards. The
/// Sudarshan Chakra is three rings counting from signs; the Western wheel is
/// a ring of cusps inside a ring of the zodiac.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Radial {
    /// The rings, innermost first.
    pub rings: Vec<Ring>,
    /// The clock hour house 1 starts at, 1 to 12: 12 is the top, 9 the left
    /// where a Western wheel puts its ascendant.
    pub starts_at: u8,
    /// The way the houses run from there.
    pub direction: Direction,
}

/// The shape of a layout: fixed cells, or rings computed per chart.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Shape {
    /// Twelve cells fixed in the row.
    Grid(Grid),
    /// Rings of sectors computed from the chart.
    Radial(Radial),
}

impl Shape {
    /// The grid, when the layout is one.
    #[must_use]
    pub const fn as_grid(&self) -> Option<&Grid> {
        match self {
            Shape::Grid(grid) => Some(grid),
            Shape::Radial(_) => None,
        }
    }

    /// The grid, to change, when the layout is one.
    #[must_use]
    pub const fn as_grid_mut(&mut self) -> Option<&mut Grid> {
        match self {
            Shape::Grid(grid) => Some(grid),
            Shape::Radial(_) => None,
        }
    }

    /// The rings, when the layout is radial.
    #[must_use]
    pub const fn as_radial(&self) -> Option<&Radial> {
        match self {
            Shape::Radial(radial) => Some(radial),
            Shape::Grid(_) => None,
        }
    }
}

/// A chart layout: a key, what cites it, and its shape.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Layout {
    /// The key (`NORTH_INDIAN`), in the key grammar.
    pub key: String,
    /// The sources the row comes from.
    pub sources: Vec<String>,
    /// Its cells or its rings.
    pub shape: Shape,
}

/// How many regions a grid layout has.
pub const CELLS: usize = 12;

/// The sampling grid the overlap check uses: prime, and offset so no
/// sample falls on the halves, thirds and quarters the shipped layouts'
/// edges lie on.
const SAMPLES: u32 = 97;
const SAMPLE_OFFSET: f64 = 0.37;

/// The largest area two cells may share before they overlap, as a
/// fraction of the square: what flattening a curve can leave between two
/// petals that meet along it.
const SHARED_AREA: f64 = 1e-6;

impl Layout {
    /// Every check a row passes before it is accepted, shipped or registered.
    ///
    /// # Errors
    ///
    /// The first check the row fails, with the field that fails it
    /// (`shape.cells[3].label`).
    pub fn validate(&self) -> Result<(), Error> {
        if !teistro_core::key::is_key_name(&self.key) {
            return Err(refused("key", format!("`{}` is not a key name", self.key)));
        }
        if self.sources.is_empty() {
            return Err(refused(
                "sources",
                "a layout names what its cells come from, as every catalogue row does",
            ));
        }
        match &self.shape {
            Shape::Grid(grid) => validate_grid(grid),
            Shape::Radial(radial) => validate_radial(radial),
        }
    }
}

fn validate_grid(grid: &Grid) -> Result<(), Error> {
    if grid.cells.len() != CELLS {
        return Err(refused(
            "shape.cells",
            format!("a grid has {CELLS} cells, not {}", grid.cells.len()),
        ));
    }
    holds_each_once(&grid.cells)?;
    for (index, cell) in grid.cells.iter().enumerate() {
        let at = |field: &str| format!("shape.cells[{index}].{field}");
        if cell.outline.segments.len() < 2 {
            return Err(refused(
                &at("outline"),
                "an outline needs at least three points",
            ));
        }
        if let Some(outside) = cell
            .outline
            .flatten()
            .into_iter()
            .find(|p| !p.in_unit_square())
        {
            return Err(refused(
                &at("outline"),
                format!("({}, {}) is outside the unit square", outside.x, outside.y),
            ));
        }
        if cell.outline.area() <= 0.0 {
            return Err(refused(&at("outline"), "the outline encloses nothing"));
        }
        for (field, anchor) in [("label", cell.label), ("bodies", cell.bodies)] {
            if !cell.outline.contains(anchor) {
                return Err(refused(
                    &at(field),
                    format!(
                        "({}, {}) is not inside the cell's outline",
                        anchor.x, anchor.y
                    ),
                ));
            }
        }
    }
    cells_do_not_overlap(&grid.cells)?;
    runs_the_declared_way(grid)
}

/// A radial layout's rings nest without overlapping inside the square, its
/// start is a clock hour, and no two rings count from the same place.
fn validate_radial(radial: &Radial) -> Result<(), Error> {
    if radial.rings.is_empty() {
        return Err(refused("shape.rings", "a radial layout needs a ring"));
    }
    if !(1..=12).contains(&radial.starts_at) {
        return Err(refused(
            "shape.starts_at",
            format!("{} is not a clock hour, 1 to 12", radial.starts_at),
        ));
    }
    let mut outside = 0.0;
    for (index, ring) in radial.rings.iter().enumerate() {
        let field = format!("shape.rings[{index}]");
        if !(ring.inner >= outside && ring.inner < ring.outer && ring.outer <= 0.5) {
            return Err(refused(
                &field,
                format!(
                    "a ring runs from {} to {}; rings run innermost first, each from where \
                     the last ended or beyond, to at most a half",
                    ring.inner, ring.outer
                ),
            ));
        }
        if radial
            .rings
            .iter()
            .take(index)
            .any(|earlier| earlier.counts_from == ring.counts_from)
        {
            return Err(refused(
                &format!("{field}.counts_from"),
                "an earlier ring already counts from there",
            ));
        }
        outside = ring.outer;
    }
    Ok(())
}

fn refused(field: &str, message: impl Into<String>) -> Error {
    Error::invalid_arg(message.into()).with_field(field.to_owned())
}

/// Each sign, or each house, exactly once, and never a mix of the two.
fn holds_each_once(cells: &[Cell]) -> Result<(), Error> {
    let mut seen = [false; CELLS];
    let first = cells.first().map(|cell| cell.holds);
    for (index, cell) in cells.iter().enumerate() {
        let field = format!("shape.cells[{index}].holds");
        let slot = match (first, cell.holds) {
            (Some(Holds::Sign(_)), Holds::Sign(sign)) => usize::from(sign.id()),
            (Some(Holds::House(_)), Holds::House(house)) if (1..=12).contains(&house) => {
                usize::from(house - 1)
            }
            (_, Holds::House(house)) if !(1..=12).contains(&house) => {
                return Err(refused(&field, format!("house {house} is not 1 to 12")));
            }
            _ => {
                return Err(refused(
                    &field,
                    "a grid holds signs in every cell or houses in every cell, not both",
                ));
            }
        };
        match seen.get_mut(slot) {
            Some(held) if !*held => *held = true,
            _ => return Err(refused(&field, "held by an earlier cell as well")),
        }
    }
    Ok(())
}

/// No point of the square lies inside two cells, sampled on an offset grid,
/// and the cells together cover no more than the square.
fn cells_do_not_overlap(cells: &[Cell]) -> Result<(), Error> {
    let total: f64 = cells.iter().map(|cell| cell.outline.area()).sum();
    if total > 1.0 + SHARED_AREA * 12.0 {
        return Err(refused(
            "shape.cells",
            format!("the cells cover {total} of a square of area 1, so some overlap"),
        ));
    }
    for i in 0..SAMPLES {
        for j in 0..SAMPLES {
            let point = Point::new(
                (f64::from(i) + SAMPLE_OFFSET) / f64::from(SAMPLES),
                (f64::from(j) + SAMPLE_OFFSET) / f64::from(SAMPLES),
            );
            let mut holders = cells
                .iter()
                .enumerate()
                .filter(|(_, cell)| cell.outline.contains(point))
                .map(|(index, _)| index);
            if let (Some(a), Some(b)) = (holders.next(), holders.next()) {
                return Err(refused(
                    &format!("shape.cells[{b}].outline"),
                    format!("overlaps cell {a} at ({}, {})", point.x, point.y),
                ));
            }
        }
    }
    Ok(())
}

/// Taken in sign or house order, the cells' centroids turn once around the
/// square's centre, every step the way the row says.
fn runs_the_declared_way(grid: &Grid) -> Result<(), Error> {
    let mut ordered: Vec<(usize, Point)> = grid
        .cells
        .iter()
        .map(|cell| {
            let order = match cell.holds {
                Holds::Sign(sign) => usize::from(sign.id()),
                Holds::House(house) => usize::from(house) - 1,
            };
            (order, cell.outline.centroid())
        })
        .collect();
    ordered.sort_by_key(|(order, _)| *order);
    // With y downwards, a clockwise turn increases the angle `atan2`
    // measures, so an anticlockwise row's steps are all negative.
    let sign = match grid.direction {
        Direction::Clockwise => 1.0,
        Direction::Anticlockwise => -1.0,
    };
    let angle = |p: Point| (p.y - 0.5).atan2(p.x - 0.5);
    let mut turned = 0.0;
    let next = ordered.iter().cycle().skip(1);
    for (step, ((_, a), (_, b))) in ordered.iter().zip(next).enumerate() {
        let mut delta = angle(*b) - angle(*a);
        if delta > std::f64::consts::PI {
            delta -= std::f64::consts::TAU;
        } else if delta < -std::f64::consts::PI {
            delta += std::f64::consts::TAU;
        }
        if delta * sign <= 0.0 {
            return Err(refused(
                "shape.direction",
                format!(
                    "the step from the {} to the {} cell turns the other way",
                    ordinal(step + 1),
                    ordinal((step + 1) % CELLS + 1)
                ),
            ));
        }
        turned += delta;
    }
    if (turned.abs() - std::f64::consts::TAU).abs() > 1e-9 {
        return Err(refused(
            "shape.direction",
            "the cells do not go once around the centre",
        ));
    }
    Ok(())
}

fn ordinal(n: usize) -> String {
    let suffix = match (n % 10, n % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}
