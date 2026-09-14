//! The shipped layouts against their reference figures, and the checks that
//! refuse a wrong layout, each made to fire (`03-design/chart-geometry.md`
//! §5 and §6).
//!
//! Structure is not correctness: a layout can pass every structural check
//! with two signs swapped. So each square layout is also held to the figure
//! it cites. Each number the figure prints is transcribed once, as the
//! point it is printed at and the sign it names, and must land in the
//! cell that shows that sign.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index their own fixtures"
)]

use teistro_core::catalogue::Rashi;
use teistro_geometry::layout::{Cell, Direction, Grid, Holds, Reference};
use teistro_geometry::path::Point;
use teistro_geometry::{Layout, Placements, place, rows};

/// A figure's printed numbers: where each is printed, in the figure's
/// pixels, and the sign number (1 for Aries) it names.
struct Figure {
    /// The chart's square in the image: its left and top edge, and its side.
    left: f64,
    top: f64,
    side: f64,
    /// The lagna the figure is drawn for.
    lagna: Rashi,
    numbers: &'static [(f64, f64, u16)],
}

impl Figure {
    fn point(&self, x: f64, y: f64) -> Point {
        Point::new((x - self.left) / self.side, (y - self.top) / self.side)
    }
}

/// `Kundali_03a.png`: a sign-fixed chart, so its numbers are signs.
const EAST: Figure = Figure {
    left: 35.0,
    top: 28.0,
    side: 945.0,
    lagna: Rashi::Aries,
    numbers: &[
        (505.0, 145.0, 1),
        (265.0, 145.0, 2),
        (152.0, 262.0, 3),
        (148.0, 515.0, 4),
        (155.0, 740.0, 5),
        (268.0, 862.0, 6),
        (512.0, 860.0, 7),
        (742.0, 855.0, 8),
        (858.0, 735.0, 9),
        (862.0, 497.0, 10),
        (862.0, 265.0, 11),
        (748.0, 150.0, 12),
    ],
};

/// `Kundali_01a.png`: a sign-fixed chart.
const SOUTH: Figure = Figure {
    left: 42.0,
    top: 42.0,
    side: 946.0,
    lagna: Rashi::Cancer,
    numbers: &[
        (395.0, 158.0, 1),
        (632.0, 160.0, 2),
        (870.0, 165.0, 3),
        (870.0, 395.0, 4),
        (868.0, 632.0, 5),
        (868.0, 868.0, 6),
        (628.0, 862.0, 7),
        (398.0, 865.0, 8),
        (167.0, 870.0, 9),
        (162.0, 635.0, 10),
        (162.0, 395.0, 11),
        (165.0, 160.0, 12),
    ],
};

/// `Kundali_02a.png`: a house-fixed chart drawn for a Cancer lagna, so its
/// numbers are the signs its houses show, 4 in the top diamond.
const NORTH: Figure = Figure {
    left: 45.0,
    top: 35.0,
    side: 945.0,
    lagna: Rashi::Cancer,
    numbers: &[
        (515.0, 95.0, 4),
        (160.0, 88.0, 5),
        (110.0, 162.0, 6),
        (105.0, 505.0, 7),
        (100.0, 855.0, 8),
        (160.0, 920.0, 9),
        (512.0, 912.0, 10),
        (862.0, 930.0, 11),
        (930.0, 845.0, 12),
        (928.0, 508.0, 1),
        (928.0, 160.0, 2),
        (868.0, 85.0, 3),
    ],
};

/// A grid layout's grid, for a test to break.
fn grid(layout: &mut Layout) -> &mut Grid {
    layout.shape.as_grid_mut().expect("a grid layout")
}

/// Every printed number lands in the cell showing that sign.
fn holds_to(layout: &Layout, figure: &Figure) {
    let placed = place(
        layout,
        &Placements {
            lagna: figure.lagna,
            bodies: Vec::new(),
        },
    )
    .unwrap();
    for &(x, y, number) in figure.numbers {
        let point = figure.point(x, y);
        let holder = placed
            .cells
            .iter()
            .find(|cell| cell.outline.contains(point))
            .unwrap_or_else(|| panic!("{}: the {number} at ({x}, {y}) is in no cell", layout.key));
        assert_eq!(
            holder.sign,
            Rashi::from_id(number - 1).unwrap(),
            "{}: the figure prints {number} at ({x}, {y})",
            layout.key
        );
    }
}

#[test]
fn every_shipped_layout_is_valid_and_its_key_is_unique() {
    let shipped = rows::shipped();
    let mut keys: Vec<&str> = shipped.iter().map(|layout| layout.key.as_str()).collect();
    for layout in &shipped {
        layout
            .validate()
            .unwrap_or_else(|error| panic!("{}: {error}", layout.key));
    }
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(keys.len(), shipped.len(), "two layouts share a key");
}

#[test]
fn the_square_charts_are_the_figures_they_cite() {
    holds_to(&rows::east_indian(), &EAST);
    holds_to(&rows::south_indian(), &SOUTH);
    holds_to(&rows::north_indian(), &NORTH);
}

#[test]
fn two_adjacent_signs_swapped_are_refused_by_the_direction_they_break() {
    // Taurus and Gemini exchanged in their corner: each sign is still held
    // once and every anchor is still inside its cell, but taken in sign
    // order the cells now step backwards once, which is what refuses it.
    let mut swapped = rows::east_indian();
    grid(&mut swapped).cells[1].holds = Holds::Sign(Rashi::Gemini);
    grid(&mut swapped).cells[2].holds = Holds::Sign(Rashi::Taurus);
    refuses(&swapped, "shape.direction");
}

#[test]
#[should_panic(expected = "the figure prints")]
fn a_ring_turned_by_one_cell_passes_every_check_and_fails_its_figure() {
    // The one mistake structure cannot see, and the one a transcribed layout
    // makes: every sign moved one cell along. Each is held once, every
    // anchor is inside its cell and the cells run the declared way, so only
    // the figure can say Aries is not where it was drawn.
    let mut turned = rows::east_indian();
    for cell in &mut grid(&mut turned).cells {
        if let Holds::Sign(sign) = cell.holds {
            cell.holds = Holds::Sign(Rashi::from_id((sign.id() + 1) % 12).unwrap());
        }
    }
    turned.validate().expect("structurally, nothing is wrong");
    holds_to(&turned, &EAST);
}

#[test]
fn the_lotus_is_the_north_indian_chart_drawn_as_petals() {
    // The same figure's points, in the lotus: every house where North
    // Indian has it. The points sit near each cell's inner corner, where a
    // petal's curve is farthest from a straight edge, so this is not
    // satisfied by accident.
    holds_to(&rows::nepali_lotus(), &NORTH);
    // And the petals fill the square as the diamonds do.
    let lotus = rows::nepali_lotus();
    let area: f64 = lotus
        .shape
        .as_grid()
        .unwrap()
        .cells
        .iter()
        .map(|cell| cell.outline.area())
        .sum();
    assert!((area - 1.0).abs() < 1e-3, "the petals cover {area}");
}

/// A shipped layout, broken one way, and the field the refusal must name.
fn refuses(broken: &Layout, field: &str) {
    let error = broken.validate().expect_err("a broken layout is refused");
    assert_eq!(error.field(), Some(field), "{error}");
}

#[test]
fn each_check_refuses_the_row_it_should() {
    // Two signs swapped, so one is held twice.
    let mut twice = rows::south_indian();
    grid(&mut twice).cells[1].holds = Holds::Sign(Rashi::Aries);
    refuses(&twice, "shape.cells[1].holds");

    // A mix of signs and houses.
    let mut mixed = rows::south_indian();
    grid(&mut mixed).cells[3].holds = Holds::House(4);
    refuses(&mixed, "shape.cells[3].holds");

    // A house that is not a house.
    let mut thirteenth = rows::north_indian();
    grid(&mut thirteenth).cells[0].holds = Holds::House(13);
    refuses(&thirteenth, "shape.cells[0].holds");

    // An anchor outside its cell.
    let mut astray = rows::east_indian();
    grid(&mut astray).cells[0].label = Point::new(0.5, 0.9);
    refuses(&astray, "shape.cells[0].label");

    // An outline leaving the square.
    let mut outside = rows::north_indian();
    grid(&mut outside).cells[2].outline.start = Point::new(-0.1, 0.0);
    refuses(&outside, "shape.cells[2].outline");

    // Two cells on top of each other.
    let mut overlapping = rows::south_indian();
    let cells = &mut grid(&mut overlapping).cells;
    let copied = cells[4].clone();
    cells[5] = Cell {
        holds: cells[5].holds,
        ..copied
    };
    refuses(&overlapping, "shape.cells[5].outline");

    // The direction declared the wrong way round.
    let mut backwards = rows::east_indian();
    grid(&mut backwards).direction = Direction::Clockwise;
    refuses(&backwards, "shape.direction");

    // Eleven cells.
    let mut short = rows::north_indian();
    grid(&mut short).cells.pop();
    refuses(&short, "shape.cells");

    // No source, and a key that is not one.
    let mut uncited = rows::north_indian();
    uncited.sources.clear();
    refuses(&uncited, "sources");
    let mut unkeyed = rows::north_indian();
    unkeyed.key = String::from("north indian");
    refuses(&unkeyed, "key");
}

#[test]
fn a_radial_layout_is_refused_by_the_ring_it_gets_wrong() {
    let radial = |layout: &mut Layout| {
        match &mut layout.shape {
            teistro_geometry::Shape::Radial(radial) => radial,
            teistro_geometry::Shape::Grid(_) => panic!("a radial layout"),
        }
        .clone()
    };
    let with = |edit: &dyn Fn(&mut teistro_geometry::Radial)| {
        let mut layout = rows::sudarshan_chakra();
        let mut rings = radial(&mut layout);
        edit(&mut rings);
        layout.shape = teistro_geometry::Shape::Radial(rings);
        layout
    };

    refuses(&with(&|r| r.starts_at = 0), "shape.starts_at");
    refuses(&with(&|r| r.rings.clear()), "shape.rings");
    // Rings out of order: the Sun's inside the lagna's.
    refuses(&with(&|r| r.rings.swap(0, 2)), "shape.rings[1]");
    // A ring past the square's edge.
    refuses(&with(&|r| r.rings[2].outer = 0.6), "shape.rings[2]");
    // Two rings counting from the Moon.
    refuses(
        &with(&|r| r.rings[2].counts_from = Reference::Moon),
        "shape.rings[2].counts_from",
    );
}

#[test]
fn a_layout_reads_back_from_its_own_json() {
    for layout in rows::shipped() {
        let json = serde_json::to_string(&layout).unwrap();
        let read: Layout = serde_json::from_str(&json).unwrap();
        assert_eq!(read, layout);
        read.validate().unwrap();
    }
}
