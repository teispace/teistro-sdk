//! The shipped layouts against their reference figures, and the checks that
//! refuse a wrong layout, each made to fire (`03-design/chart-geometry.md`
//! §5 and §6).
//!
//! Structure is not correctness: a layout can pass every structural check
//! with two signs swapped. So each square layout is also held to the figure
//! it cites, and each Nepali chart to the printed janma patrika it is drawn
//! from. Each number the figure prints is transcribed once, as the point it
//! is printed at and the sign it names, and must land in the cell that
//! shows that sign.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index their own fixtures"
)]

use teistro_core::catalogue::{Graha, Rashi};
use teistro_geometry::layout::{Cell, Direction, Grid, Holds, Hub, Reference};
use teistro_geometry::path::{Point, Segment};
use teistro_geometry::{Body, Layout, Placements, place, rows};

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
    fn points(&self) -> Vec<(f64, f64, u16, Point)> {
        self.numbers
            .iter()
            .map(|&(x, y, number)| {
                let point = Point::new((x - self.left) / self.side, (y - self.top) / self.side);
                (x, y, number, point)
            })
            .collect()
    }
}

/// The radius both Nepali rows draw their circle at, which a photograph's
/// circle is scaled to.
const NEPALI_RADIUS: f64 = 15.0 / 32.0;

/// A printed round chart, photographed: a page photographed at an angle is
/// foreshortened, so its circle is measured across and down separately, from
/// the chart's own centre (where the petals meet, or the hub's centre), in
/// the photograph's pixels. Each number is where the patrika prints it and
/// the sign it names. Only the chart's geometry and its numbers are
/// transcribed; the patrikas are private and are not in the repository.
struct Photo {
    centre: (f64, f64),
    radius: (f64, f64),
    lagna: Rashi,
    numbers: &'static [(f64, f64, u16)],
}

impl Photo {
    fn points(&self) -> Vec<(f64, f64, u16, Point)> {
        let ((cx, cy), (rx, ry)) = (self.centre, self.radius);
        self.numbers
            .iter()
            .map(|&(x, y, number)| {
                let point = Point::new(
                    0.5 + (x - cx) / rx * NEPALI_RADIUS,
                    0.5 + (y - cy) / ry * NEPALI_RADIUS,
                );
                (x, y, number, point)
            })
            .collect()
    }
}

/// The first scan's जन्मकुण्डली, a lotus of twelve petals, read from page 3
/// at 300 dpi (a 600 by 560 crop): drawn for a lagna in sign 1, so each
/// number is both the sign and the house, 1 at the top and rising
/// anticlockwise.
const LOTUS_PRINT: Photo = Photo {
    centre: (277.0, 285.0),
    radius: (259.5, 243.0),
    lagna: Rashi::Aries,
    numbers: &[
        (269.0, 151.0, 1),
        (197.0, 171.0, 2),
        (155.0, 224.0, 3),
        (139.0, 286.0, 4),
        (163.0, 350.0, 5),
        (208.0, 392.0, 6),
        (277.0, 392.0, 7),
        (350.0, 390.0, 8),
        (386.0, 334.0, 9),
        (413.0, 274.0, 10),
        (396.0, 214.0, 11),
        (343.0, 168.0, 12),
    ],
};

/// The third scan's ल chart, the first of its four round charts, read from
/// page 3 at 300 dpi (a 480 by 540 crop), from the hub's centre: the same
/// lagna in sign 1. Each number sits beside the hub, in the cell the grid's
/// lines and diagonals cut.
const ROUND_PRINT: Photo = Photo {
    centre: (229.0, 261.0),
    radius: (205.5, 246.5),
    lagna: Rashi::Aries,
    numbers: &[
        (220.0, 187.0, 1),
        (165.0, 172.0, 2),
        (122.0, 192.0, 3),
        (150.0, 272.0, 4),
        (135.0, 330.0, 5),
        (160.0, 370.0, 6),
        (227.0, 345.0, 7),
        (295.0, 372.0, 8),
        (320.0, 335.0, 9),
        (305.0, 272.0, 10),
        (325.0, 190.0, 11),
        (290.0, 145.0, 12),
    ],
};

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

/// A consumer's kundali view of North Indian drawn with petals (a 550 px
/// square at (12, 22) of a 571 by 592 screenshot), drawn for a lagna in
/// sign 2: house 1 the top kendra shows 2, and the signs rise
/// anticlockwise. Only the geometry and the numbers are transcribed.
const PETALS_VIEW: Figure = Figure {
    left: 12.0,
    top: 22.0,
    side: 550.0,
    lagna: Rashi::Taurus,
    numbers: &[
        (287.0, 95.0, 2),
        (197.0, 68.0, 3),
        (58.0, 206.0, 4),
        (87.0, 296.0, 5),
        (58.0, 384.0, 6),
        (197.0, 523.0, 7),
        (287.0, 495.0, 8),
        (376.0, 523.0, 9),
        (513.0, 384.0, 10),
        (487.0, 296.0, 11),
        (513.0, 206.0, 12),
        (376.0, 68.0, 1),
    ],
};

/// A grid layout's grid, for a test to break.
fn grid(layout: &mut Layout) -> &mut Grid {
    layout.shape.as_grid_mut().expect("a grid layout")
}

/// Every printed number lands in the cell showing that sign.
fn holds_to(layout: &Layout, figure: &Figure) {
    lands(layout, figure.lagna, &figure.points());
}

/// Every number a photographed patrika prints lands in the cell showing
/// that sign.
fn holds_to_print(layout: &Layout, photo: &Photo) {
    lands(layout, photo.lagna, &photo.points());
}

fn lands(layout: &Layout, lagna: Rashi, points: &[(f64, f64, u16, Point)]) {
    let placed = place(layout, &Placements::new(lagna)).unwrap();
    for &(x, y, number, point) in points {
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
fn the_shipped_rows_and_the_catalogue_name_the_same_layouts() {
    // Both ways: every catalogue member has a row, and every row's key is a
    // catalogue member, in the same order.
    let rows: Vec<String> = rows::shipped()
        .into_iter()
        .map(|layout| layout.key)
        .collect();
    let catalogue: Vec<&str> = teistro_core::catalogue::ChartLayout::ALL
        .iter()
        .map(|member| member.key())
        .collect();
    assert_eq!(rows, catalogue);
}

#[test]
fn a_registered_layout_is_checked_found_and_cannot_replace_a_shipped_one() {
    let mut layouts = teistro_geometry::Layouts::new();
    assert_eq!(layouts.iter().count(), rows::shipped().len());

    // A broken row is refused by the same checks, naming the field.
    let mut broken = rows::south_indian();
    broken.key = String::from("ACME_BROKEN");
    grid(&mut broken).direction = Direction::Anticlockwise;
    assert_eq!(
        layouts.register(broken).unwrap_err().field(),
        Some("shape.direction")
    );

    let mut own = rows::east_indian();
    own.key = String::from("ACME_ODIA");
    let id = layouts.register(own.clone()).unwrap();
    assert!(id.is_registered());
    assert_eq!(layouts.get("ACME_ODIA"), Some(&own));
    assert_eq!(layouts.iter().count(), rows::shipped().len() + 1);

    // Nor twice, nor a shipped key, nor after sealing.
    assert!(layouts.register(own.clone()).is_err());
    let mut takeover = own.clone();
    takeover.key = String::from("WESTERN_WHEEL");
    assert_eq!(layouts.register(takeover).unwrap_err().field(), Some("key"));
    layouts.seal();
    own.key = String::from("ACME_LATER");
    assert!(layouts.register(own).is_err());
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
    holds_to(&rows::north_indian_petals(), &PETALS_VIEW);
}

/// A curve as its three points' bits: start, control and end.
type Curve = [(u64, u64); 3];

fn bits(point: Point) -> (u64, u64) {
    (point.x.to_bits(), point.y.to_bits())
}

#[test]
fn the_petals_right_half_is_its_left_half_mirrored_to_the_bit() {
    let layout = rows::north_indian_petals();
    let mut curves: Vec<Curve> = Vec::new();
    for cell in &layout.shape.as_grid().unwrap().cells {
        let mut from = cell.outline.start;
        for &step in &cell.outline.segments {
            if let Segment::Quad { control, to } = step {
                curves.push([bits(from), bits(control), bits(to)]);
            }
            from = step.end();
        }
    }
    // Sixteen curves, each drawn by the kendra and the corner cell either
    // side of it.
    assert_eq!(curves.len(), 32);
    let mirror = |(x, y): (u64, u64)| ((1.0 - f64::from_bits(x)).to_bits(), y);
    let drawn = |curve: Curve| {
        let back = [curve[2], curve[1], curve[0]];
        curves.contains(&curve) || curves.contains(&back)
    };
    let left: Vec<&Curve> = curves
        .iter()
        .filter(|curve| curve.iter().all(|&(x, _)| f64::from_bits(x) <= 0.5))
        .collect();
    assert_eq!(left.len(), 16, "half the curves are left of the middle");
    for curve in left {
        let mirrored = [mirror(curve[0]), mirror(curve[1]), mirror(curve[2])];
        assert!(drawn(mirrored), "{curve:?} has no mirror");
    }
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
fn the_nepali_charts_are_the_patrikas_they_are_drawn_from() {
    holds_to_print(&rows::nepali_lotus(), &LOTUS_PRINT);
    holds_to_print(&rows::nepali_circle(), &ROUND_PRINT);
    // Both are drawn in a circle of the radius the photographs are scaled
    // to: the frame's second path starts at its top.
    for layout in [rows::nepali_lotus(), rows::nepali_circle()] {
        let circle = &layout.shape.as_grid().unwrap().frame[1];
        assert_eq!(
            circle.start,
            Point::new(0.5, 0.5 - NEPALI_RADIUS),
            "{}",
            layout.key
        );
    }
}

#[test]
fn the_patrikas_grahas_land_where_they_are_written() {
    // Both scans' chart, anonymised to its pattern: the lagna in sign 1,
    // the Moon, Jupiter and Saturn in 2, Rahu in 3, Mars in 4, the Sun,
    // Mercury and Venus in 5 and Ketu in 9, as the printed sphuta table
    // gives them and both patrikas write them.
    let written: [(Graha, u16); 9] = [
        (Graha::Moon, 2),
        (Graha::Jupiter, 2),
        (Graha::Saturn, 2),
        (Graha::Rahu, 3),
        (Graha::Mars, 4),
        (Graha::Sun, 5),
        (Graha::Mercury, 5),
        (Graha::Venus, 5),
        (Graha::Ketu, 9),
    ];
    let chart = written
        .iter()
        .fold(Placements::new(Rashi::Aries), |chart, &(graha, sign)| {
            chart.with(Body::in_sign(
                graha.key_id(),
                Rashi::from_id(sign - 1).unwrap(),
            ))
        });
    for (layout, photo) in [
        (rows::nepali_lotus(), &LOTUS_PRINT),
        (rows::nepali_circle(), &ROUND_PRINT),
    ] {
        let placed = place(&layout, &chart).unwrap();
        let printed = photo.points();
        for &(graha, sign) in &written {
            let holder = placed
                .cells
                .iter()
                .find(|cell| cell.bodies.contains(&graha.key_id()))
                .unwrap();
            // The lagna is sign 1, so the sign is the house, and the cell is
            // the one the patrika prints that number in.
            assert_eq!(
                (holder.sign.id() + 1, u16::from(holder.house)),
                (sign, sign),
                "{}: {}",
                layout.key,
                graha.key()
            );
            let &(.., at) = printed
                .iter()
                .find(|(.., number, _)| *number == sign)
                .unwrap();
            assert!(
                holder.outline.contains(at),
                "{}: {} is not in the cell the patrika writes {sign} in",
                layout.key,
                graha.key()
            );
        }
        // Lagna's own cell is house 1, at the top, and is marked.
        let lagna: Vec<_> = placed.cells.iter().filter(|cell| cell.lagna).collect();
        assert_eq!(lagna.len(), 1);
        assert_eq!((lagna[0].house, lagna[0].sign), (1, Rashi::Aries));
    }
}

#[test]
#[should_panic(expected = "the figure prints")]
fn a_lotus_turned_by_one_petal_fails_its_patrika() {
    // Every house moved one petal along still passes every structural check.
    let mut turned = rows::nepali_lotus();
    for cell in &mut grid(&mut turned).cells {
        if let Holds::House(house) = cell.holds {
            cell.holds = Holds::House(house % 12 + 1);
        }
    }
    turned.validate().expect("structurally, nothing is wrong");
    holds_to_print(&turned, &LOTUS_PRINT);
}

#[test]
fn the_lotus_is_twelve_equal_petals_and_the_round_chart_is_not_twelve_wedges() {
    // The lotus: every petal the same area, each a clock hour apart. The
    // background shows between the tips, so the petals cover less than the
    // circle.
    let lotus = rows::nepali_lotus();
    let petals: Vec<f64> = lotus
        .shape
        .as_grid()
        .unwrap()
        .cells
        .iter()
        .map(|cell| cell.outline.area())
        .collect();
    assert!(
        petals.iter().all(|area| (area - petals[0]).abs() < 1e-12),
        "{petals:?}"
    );
    let disc = std::f64::consts::PI * NEPALI_RADIUS * NEPALI_RADIUS;
    let covered: f64 = petals.iter().sum();
    assert!(
        covered < disc,
        "the petals cover {covered} of a disc of {disc}"
    );

    // The round chart: a square grid in the circle, so an edge cell (house
    // 1) and a corner cell (house 2) differ, and the hub holds none.
    let round = rows::nepali_circle();
    let grid = round.shape.as_grid().unwrap();
    let (edge, corner) = (grid.cells[0].outline.area(), grid.cells[1].outline.area());
    assert!((edge - corner).abs() > 1e-3, "{edge} and {corner}");
    let hub = grid.hub.expect("the round chart names its chart in a hub");
    let cells: f64 = grid.cells.iter().map(|cell| cell.outline.area()).sum();
    let square = (2.0 * NEPALI_RADIUS / 5.0).powi(2);
    assert!(
        (cells + square - disc).abs() < 1e-3,
        "the cells and the hub's square fill the disc"
    );
    assert!(
        grid.cells
            .iter()
            .all(|cell| !cell.outline.contains(hub.centre))
    );
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

    // A hub a cell holds, and one that leaves the square.
    let mut held = rows::nepali_circle();
    grid(&mut held).hub = Some(Hub {
        centre: Point::new(0.5, 0.2),
        radius: 0.05,
    });
    refuses(&held, "shape.hub");
    // A hub past each edge of the square, and one with no size.
    for (x, y, radius) in [
        (0.98, 0.5, 0.05),
        (0.02, 0.5, 0.05),
        (0.5, 0.02, 0.05),
        (0.5, 0.98, 0.05),
        (0.5, 0.5, 0.0),
    ] {
        let mut beyond = rows::nepali_circle();
        grid(&mut beyond).hub = Some(Hub {
            centre: Point::new(x, y),
            radius,
        });
        refuses(&beyond, "shape.hub");
    }

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
