//! A layout as a row of data, and the checks that refuse a wrong one.
//!
//! A **grid** layout is twelve cells fixed in the row. Each cell holds a
//! sign (a sign-fixed layout, South and East Indian) or a house (a
//! house-fixed layout, North Indian and the lotus). Placement computes the
//! other from the lagna ([`crate::place`]).
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

/// A chart layout: a key, what cites it, and its shape.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Layout {
    /// The key (`NORTH_INDIAN`), in the key grammar.
    pub key: String,
    /// The sources the row's cells come from.
    pub sources: Vec<String>,
    /// The cells.
    pub grid: Grid,
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
    /// (`grid.cells[3].label`).
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
        let grid = &self.grid;
        if grid.cells.len() != CELLS {
            return Err(refused(
                "grid.cells",
                format!("a grid has {CELLS} cells, not {}", grid.cells.len()),
            ));
        }
        holds_each_once(&grid.cells)?;
        for (index, cell) in grid.cells.iter().enumerate() {
            let at = |field: &str| format!("grid.cells[{index}].{field}");
            if cell.outline.segments.len() < 2 {
                return Err(refused(
                    &at("outline"),
                    "an outline needs at least three points",
                ));
            }
            if let Some(outside) = cell.outline.points().find(|p| !p.in_unit_square()) {
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
}

fn refused(field: &str, message: impl Into<String>) -> Error {
    Error::invalid_arg(message.into()).with_field(field.to_owned())
}

/// Each sign, or each house, exactly once, and never a mix of the two.
fn holds_each_once(cells: &[Cell]) -> Result<(), Error> {
    let mut seen = [false; CELLS];
    let first = cells.first().map(|cell| cell.holds);
    for (index, cell) in cells.iter().enumerate() {
        let field = format!("grid.cells[{index}].holds");
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
            "grid.cells",
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
                    &format!("grid.cells[{b}].outline"),
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
                "grid.direction",
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
            "grid.direction",
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
