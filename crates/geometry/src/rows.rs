//! The layouts that ship, each a cited row (`03-design/chart-geometry.md` §2).
//!
//! | row | fixed | starts | runs | cited |
//! |---|---|---|---|---|
//! | [`north_indian`] | houses | house 1, the top diamond | anticlockwise | Wikipedia, *Kundali (astrology)*, `Kundali_02a.png` |
//! | [`south_indian`] | signs | Pisces top-left | clockwise | the same article, `Kundali_01a.png` |
//! | [`east_indian`] | signs | Aries top-centre | anticlockwise | the same article, `Kundali_03a.png` |
//! | [`nepali_lotus`] | houses | as North Indian | anticlockwise | printed janma patrikas |
//! | [`sudarshan_chakra`] | three rings of houses | house 1 starting at twelve o'clock | clockwise | the tutorials' ring order; the application's start and direction (crux C47) |
//! | [`western_wheel`] | the cusps | the ascendant at nine o'clock | anticlockwise | tutorials on reading a chart wheel, which agree |
//! | [`nepali_circle`] | houses | the cell above the hub | anticlockwise | printed janma patrikas |
//! | [`north_indian_petals`] | houses | as North Indian | anticlockwise | a consumer's kundali view |
//!
//! Every coordinate of the three square figures is a half, a third or a
//! quarter, and every one of the petalled North Indian a dyadic fraction,
//! reached by IEEE's basic operations, which every platform does
//! identically; and placement only chooses cells, so nothing here can differ
//! between architectures (§7).

use teistro_core::catalogue::Rashi;

use crate::clock;
use crate::layout::{Cell, Direction, Grid, Holds, Hub, Layout, Radial, Reference, Ring, Shape};
use crate::path::{Path, Point, Segment};
use crate::place::{CENTRE, circle};

/// Every layout the SDK ships, in catalogue order; a test holds the two to
/// the same list both ways.
///
/// ```
/// use teistro_core::catalogue::ChartLayout;
/// use teistro_geometry::rows;
///
/// let keys: Vec<String> = rows::shipped().into_iter().map(|layout| layout.key).collect();
/// let catalogue: Vec<&str> = ChartLayout::ALL.iter().map(|member| member.key()).collect();
/// assert_eq!(keys, catalogue);
/// ```
#[must_use]
pub fn shipped() -> Vec<Layout> {
    vec![
        north_indian(),
        south_indian(),
        east_indian(),
        nepali_lotus(),
        sudarshan_chakra(),
        western_wheel(),
        nepali_circle(),
        north_indian_petals(),
    ]
}

const WIKIPEDIA: &str = "Wikipedia, \"Kundali (astrology)\", figures Kundali_01a.png, \
                         Kundali_02a.png and Kundali_03a.png";

fn p(x: f64, y: f64) -> Point {
    Point::new(x, y)
}

fn square(x0: f64, y0: f64, x1: f64, y1: f64) -> Path {
    Path::polygon(&[p(x0, y0), p(x1, y0), p(x1, y1), p(x0, y1)])
}

fn border() -> Path {
    square(0.0, 0.0, 1.0, 1.0)
}

/// A sign by its place in the zodiac, Aries 0.
fn sign(index: u16) -> Rashi {
    Rashi::from_id(index % 12).unwrap_or(Rashi::Aries)
}

/// The North Indian houses' anchors: where the number and the bodies of
/// house `n` go, from the baseline application's chart (in units of 1/400).
const HOUSE_ANCHORS: [((f64, f64), (f64, f64)); 12] = [
    ((200.0, 55.0), (200.0, 110.0)),
    ((135.0, 35.0), (100.0, 40.0)),
    ((35.0, 135.0), (40.0, 100.0)),
    ((55.0, 200.0), (110.0, 200.0)),
    ((35.0, 265.0), (40.0, 300.0)),
    ((135.0, 365.0), (100.0, 360.0)),
    ((200.0, 345.0), (200.0, 290.0)),
    ((265.0, 365.0), (300.0, 360.0)),
    ((365.0, 265.0), (360.0, 300.0)),
    ((345.0, 200.0), (290.0, 200.0)),
    ((365.0, 135.0), (360.0, 100.0)),
    ((265.0, 35.0), (300.0, 40.0)),
];

fn house_cell(house: u8, outline: Path) -> Cell {
    let ((lx, ly), (bx, by)) = HOUSE_ANCHORS
        .get(usize::from(house.saturating_sub(1)))
        .copied()
        .unwrap_or_default();
    Cell {
        outline,
        holds: Holds::House(house),
        label: p(lx / 400.0, ly / 400.0),
        bodies: p(bx / 400.0, by / 400.0),
    }
}

/// The North Indian chart: a square, its diagonals and the diamond through
/// its midpoints. The houses are fixed, house 1 the top diamond, and they run
/// anticlockwise; the signs turn with the lagna.
///
/// ```
/// use teistro_geometry::{Holds, rows};
///
/// let layout = rows::north_indian();
/// let grid = layout.shape.as_grid().unwrap();
/// // House 1 is the top diamond, centred on the square's top half.
/// assert_eq!(grid.cells[0].holds, Holds::House(1));
/// assert!(grid.cells[0].label.y < 0.5);
/// ```
#[must_use]
pub fn north_indian() -> Layout {
    let (tl, tr, br, bl) = (p(0.0, 0.0), p(1.0, 0.0), p(1.0, 1.0), p(0.0, 1.0));
    let (top, right, bottom, left) = (p(0.5, 0.0), p(1.0, 0.5), p(0.5, 1.0), p(0.0, 0.5));
    let centre = p(0.5, 0.5);
    let (a, b, c, d) = (p(0.25, 0.25), p(0.75, 0.25), p(0.75, 0.75), p(0.25, 0.75));
    let outlines = [
        vec![top, b, centre, a],
        vec![tl, top, a],
        vec![tl, a, left],
        vec![left, a, centre, d],
        vec![left, d, bl],
        vec![bl, d, bottom],
        vec![d, centre, c, bottom],
        vec![bottom, c, br],
        vec![br, c, right],
        vec![right, c, centre, b],
        vec![right, b, tr],
        vec![tr, b, top],
    ];
    Layout {
        key: String::from("NORTH_INDIAN"),
        sources: vec![String::from(WIKIPEDIA)],
        shape: Shape::Grid(Grid {
            cells: (1..=12)
                .zip(outlines)
                .map(|(house, points)| house_cell(house, Path::polygon(&points)))
                .collect(),
            frame: vec![border()],
            direction: Direction::Anticlockwise,
            hub: None,
        }),
    }
}

/// The South Indian chart: the ring of twelve cells around a 4×4 grid's
/// empty centre. The signs are fixed, Pisces top-left and Aries beside it,
/// running clockwise; the houses turn with the lagna.
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_geometry::{Holds, Point, rows};
///
/// let layout = rows::south_indian();
/// let grid = layout.shape.as_grid().unwrap();
/// // Pisces is the top-left cell, and the centre holds nothing.
/// let pisces = grid.cells.iter().find(|cell| cell.holds == Holds::Sign(Rashi::Pisces)).unwrap();
/// assert!(pisces.outline.contains(Point::new(0.1, 0.1)));
/// assert!(grid.cells.iter().all(|cell| !cell.outline.contains(Point::new(0.5, 0.5))));
/// ```
#[must_use]
pub fn south_indian() -> Layout {
    // (column, row) of Aries, Taurus, ... Pisces, clockwise from the top.
    const CELLS: [(u8, u8); 12] = [
        (1, 0),
        (2, 0),
        (3, 0),
        (3, 1),
        (3, 2),
        (3, 3),
        (2, 3),
        (1, 3),
        (0, 3),
        (0, 2),
        (0, 1),
        (0, 0),
    ];
    let cells = (0..12u16)
        .zip(CELLS)
        .map(|(index, (column, row))| {
            let (x0, y0) = (f64::from(column) * 0.25, f64::from(row) * 0.25);
            Cell {
                outline: square(x0, y0, x0 + 0.25, y0 + 0.25),
                holds: Holds::Sign(sign(index)),
                label: p(x0 + 0.05, y0 + 0.05),
                bodies: p(x0 + 0.125, y0 + 0.15),
            }
        })
        .collect();
    Layout {
        key: String::from("SOUTH_INDIAN"),
        sources: vec![String::from(WIKIPEDIA)],
        shape: Shape::Grid(Grid {
            cells,
            frame: vec![border(), square(0.25, 0.25, 0.75, 0.75)],
            direction: Direction::Clockwise,
            hub: None,
        }),
    }
}

/// The East Indian (Bengali, Odia, Assamese) chart: a 3×3 grid with an empty
/// centre, each corner cut from its outer corner to its inner one into two
/// signs. The signs are fixed, Aries top-centre, running anticlockwise.
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_geometry::{Holds, Point, rows};
///
/// let layout = rows::east_indian();
/// let grid = layout.shape.as_grid().unwrap();
/// // Aries is the top-centre cell.
/// let aries = grid.cells.iter().find(|cell| cell.holds == Holds::Sign(Rashi::Aries)).unwrap();
/// assert!(aries.outline.contains(Point::new(0.5, 0.15)));
/// ```
#[must_use]
pub fn east_indian() -> Layout {
    let (t, u) = (1.0 / 3.0, 2.0 / 3.0);
    // Each sign's outline, and the point its label leans towards: an
    // edge cell's outer edge, a corner triangle's right-angle vertex.
    let shapes: [(Vec<Point>, Point); 12] = [
        (vec![p(t, 0.0), p(u, 0.0), p(u, t), p(t, t)], p(0.5, 0.0)),
        (vec![p(0.0, 0.0), p(t, 0.0), p(t, t)], p(t, 0.0)),
        (vec![p(0.0, 0.0), p(t, t), p(0.0, t)], p(0.0, t)),
        (vec![p(0.0, t), p(t, t), p(t, u), p(0.0, u)], p(0.0, 0.5)),
        (vec![p(0.0, u), p(t, u), p(0.0, 1.0)], p(0.0, u)),
        (vec![p(t, u), p(t, 1.0), p(0.0, 1.0)], p(t, 1.0)),
        (vec![p(t, u), p(u, u), p(u, 1.0), p(t, 1.0)], p(0.5, 1.0)),
        (vec![p(u, u), p(1.0, 1.0), p(u, 1.0)], p(u, 1.0)),
        (vec![p(u, u), p(1.0, u), p(1.0, 1.0)], p(1.0, u)),
        (vec![p(u, t), p(1.0, t), p(1.0, u), p(u, u)], p(1.0, 0.5)),
        (vec![p(u, t), p(1.0, 0.0), p(1.0, t)], p(1.0, t)),
        (vec![p(u, 0.0), p(1.0, 0.0), p(u, t)], p(u, 0.0)),
    ];
    let cells = (0..12u16)
        .zip(shapes)
        .map(|(index, (points, towards))| {
            let outline = Path::polygon(&points);
            let centre = outline.centroid();
            Cell {
                label: p(
                    centre.x + (towards.x - centre.x) * 0.5,
                    centre.y + (towards.y - centre.y) * 0.5,
                ),
                bodies: centre,
                outline,
                holds: Holds::Sign(sign(index)),
            }
        })
        .collect();
    Layout {
        key: String::from("EAST_INDIAN"),
        sources: vec![String::from(WIKIPEDIA)],
        shape: Shape::Grid(Grid {
            cells,
            frame: vec![border()],
            direction: Direction::Anticlockwise,
            hub: None,
        }),
    }
}

/// The Sudarshan Chakra: the chart read from three places at once, as three
/// rings of twelve houses. The lagna's ring is innermost, then the Moon's,
/// then the Sun's, which is the order every tutorial gives. The application
/// the rings' proportions come from draws them the other way round, and its
/// start (house 1 beginning at twelve o'clock) and direction (clockwise)
/// have no second source, so they are this row's declared properties rather
/// than the chakra's (crux C47).
///
/// ```
/// use teistro_geometry::{Direction, rows};
///
/// let layout = rows::sudarshan_chakra();
/// let radial = layout.shape.as_radial().unwrap();
/// assert_eq!((radial.starts_at, radial.direction), (12, Direction::Clockwise));
/// ```
#[must_use]
pub fn sudarshan_chakra() -> Layout {
    // The application's radii, 50, 110, 165 and 220 in a 480 square.
    let radius = |r: f64| r / 480.0;
    let ring = |inner: f64, outer: f64, counts_from: Reference| Ring {
        inner: radius(inner),
        outer: radius(outer),
        counts_from,
    };
    Layout {
        key: String::from("SUDARSHAN_CHAKRA"),
        sources: vec![
            String::from("tutorials on the Sudarshana chakra, for the order of its rings"),
            String::from(
                "the baseline application's Sudarshan chakra view, for its proportions, start and direction",
            ),
        ],
        shape: Shape::Radial(Radial {
            rings: vec![
                ring(50.0, 110.0, Reference::Lagna),
                ring(110.0, 165.0, Reference::Moon),
                ring(165.0, 220.0, Reference::Sun),
            ],
            starts_at: 12,
            direction: Direction::Clockwise,
        }),
    }
}

/// The Western chart wheel: the houses as wide as their cusps make them,
/// inside a ring of the zodiac, the ascendant at nine o'clock and the houses
/// and signs running anticlockwise from it, which is the convention every
/// source agrees on. The bodies are drawn at their own longitudes
/// ([`crate::place::Mark`]), and the hub is left for the aspects a renderer
/// draws across it.
///
/// ```
/// use teistro_geometry::{Direction, Reference, rows};
///
/// let layout = rows::western_wheel();
/// let radial = layout.shape.as_radial().unwrap();
/// assert_eq!((radial.starts_at, radial.direction), (9, Direction::Anticlockwise));
/// let rings: Vec<Reference> = radial.rings.iter().map(|ring| ring.counts_from).collect();
/// assert_eq!(rings, [Reference::Cusps, Reference::Zodiac]);
/// ```
#[must_use]
pub fn western_wheel() -> Layout {
    Layout {
        key: String::from("WESTERN_WHEEL"),
        sources: vec![String::from(
            "tutorials on reading an astrological chart wheel: the first house's cusp at nine \
             o'clock, the houses numbered anticlockwise",
        )],
        shape: Shape::Radial(Radial {
            rings: vec![
                Ring {
                    inner: 0.18,
                    outer: 0.39,
                    counts_from: Reference::Cusps,
                },
                Ring {
                    inner: 0.39,
                    outer: 0.48,
                    counts_from: Reference::Zodiac,
                },
            ],
            starts_at: 9,
            direction: Direction::Anticlockwise,
        }),
    }
}

/// What the two Nepali charts are drawn from.
const PATRIKAS: &str = "printed Nepali janma patrikas, three scans of two printers' forms read \
                        2026-11-10: the lotus (जन्मकुण्डली, चलितकुण्डली, राशिकुण्डली) on two, \
                        the round charts (ल, रा, न, भा) on the third";

/// The radius of the circle both Nepali charts are drawn in: the print sets
/// it in a decorated square, and 15/32 leaves a thirty-second of margin.
const NEPALI_RADIUS: f64 = 15.0 / 32.0;

/// The direction house `house`'s cell faces, in half hours clockwise from
/// twelve: house 1 at the top and the rest anticlockwise, a clock hour each,
/// as both Nepali charts run.
fn facing(house: u8) -> u32 {
    (24 - 2 * (u32::from(house).saturating_sub(1) % 12)) % 24
}

/// How far out along the line it shares with its neighbour a petal stays
/// with it, and where that line pulls the petal's curve to the tip.
const PETAL_SHARED: f64 = 7.0 / 32.0;
const PETAL_PULL: f64 = 11.0 / 32.0;

/// Where a petal's number and bodies go along its axis: the number towards
/// the tip, the bodies where the petal is widest.
const PETAL_LABEL: f64 = 3.0 / 8.0;
const PETAL_BODIES: f64 = 1.0 / 4.0;

/// The Nepali lotus as printed janma patrikas draw it: twelve equal petals
/// radiating from the centre inside a circle, house 1's petal at twelve
/// o'clock and the houses running anticlockwise, a clock hour apart, so the
/// petals are North Indian's houses in North Indian's order.
///
/// Each petal shares with its neighbour the line half an hour off its axis
/// for its inner half, as the print's petals touch, then curves to a tip on
/// the circle; the background shows between the tips. Every point is a
/// clock direction times a radius in 32nds, so the row is the same on every
/// platform (`03-design/chart-geometry.md` §7).
///
/// ```
/// use teistro_geometry::{Holds, Point, Segment, rows};
///
/// let lotus = rows::nepali_lotus();
/// let grid = lotus.shape.as_grid().unwrap();
/// // House 1's petal points to twelve o'clock, house 4's to nine.
/// let petal = |house: u8| grid.cells.iter().find(|cell| cell.holds == Holds::House(house)).unwrap();
/// assert!(petal(1).outline.contains(Point::new(0.5, 0.2)));
/// assert!(petal(4).outline.contains(Point::new(0.2, 0.5)));
/// // Twelve equal petals: each encloses the same area.
/// let first = petal(1).outline.area();
/// assert!(grid.cells.iter().all(|cell| (cell.outline.area() - first).abs() < 1e-12));
/// // Curved to their tips.
/// assert!(petal(7).outline.segments.iter().any(|step| matches!(step, Segment::Quad { .. })));
/// ```
#[must_use]
pub fn nepali_lotus() -> Layout {
    let cells = (1..=12u8)
        .map(|house| {
            let axis = facing(house);
            let (right, left) = ((axis + 1) % 24, (axis + 23) % 24);
            let outline = Path {
                start: CENTRE,
                segments: vec![
                    Segment::Line {
                        to: clock::at(CENTRE, PETAL_SHARED, right),
                    },
                    Segment::Quad {
                        control: clock::at(CENTRE, PETAL_PULL, right),
                        to: clock::at(CENTRE, NEPALI_RADIUS, axis),
                    },
                    Segment::Quad {
                        control: clock::at(CENTRE, PETAL_PULL, left),
                        to: clock::at(CENTRE, PETAL_SHARED, left),
                    },
                ],
            };
            Cell {
                outline,
                holds: Holds::House(house),
                label: clock::at(CENTRE, PETAL_LABEL, axis),
                bodies: clock::at(CENTRE, PETAL_BODIES, axis),
            }
        })
        .collect();
    Layout {
        key: String::from("NEPALI_LOTUS"),
        sources: vec![String::from(PATRIKAS)],
        shape: Shape::Grid(Grid {
            cells,
            frame: vec![border(), circle(NEPALI_RADIUS)],
            direction: Direction::Anticlockwise,
            hub: None,
        }),
    }
}

/// Half the side of the round chart's hub square: a fifth of the circle's
/// radius, as the print measures (0.205 and 0.209 of it, down and across).
const HUB_HALF: f64 = NEPALI_RADIUS / 5.0;

/// The hub's circle, inside its square.
const HUB_RADIUS: f64 = HUB_HALF * 0.75;

/// Where the numbers and the bodies go: in an edge cell along its axis, the
/// number beside the hub; in a corner cell along the clock hour it faces.
const EDGE_LABEL: f64 = 1.0 / 8.0;
const EDGE_BODIES: f64 = 9.0 / 32.0;
const CORNER_LABEL: f64 = 1.0 / 4.0;
const CORNER_BODIES: f64 = 3.0 / 8.0;

/// The Nepali round chart as a printed janma patrika draws it: a circle cut
/// on a square grid. The lines of a central square's four sides run out to
/// the circle, and the square's diagonals run from its corners to the
/// circle, so the topology is the East Indian chart's, clipped to a circle.
/// The square is the hub, where the print names the chart (ल, रा, न, भा).
/// The houses are fixed, house 1 the cell above the hub, running
/// anticlockwise; an edge cell spans about 23° of the circle and a corner
/// cell about 34°, so the cells are not equal wedges.
///
/// ```
/// use teistro_geometry::{Holds, Point, rows};
///
/// let round = rows::nepali_circle();
/// let grid = round.shape.as_grid().unwrap();
/// let cell = |house: u8| grid.cells.iter().find(|cell| cell.holds == Holds::House(house)).unwrap();
/// // House 1 above the hub, house 2 the upper half of the top-left corner,
/// // house 3 its lower half, house 4 left of the hub.
/// assert!(cell(1).outline.contains(Point::new(0.5, 0.2)));
/// assert!(cell(2).outline.contains(Point::new(0.3, 0.2)));
/// assert!(cell(3).outline.contains(Point::new(0.2, 0.3)));
/// assert!(cell(4).outline.contains(Point::new(0.2, 0.5)));
/// // The hub holds no house.
/// assert!(grid.cells.iter().all(|cell| !cell.outline.contains(Point::new(0.5, 0.5))));
/// ```
#[must_use]
pub fn nepali_circle() -> Layout {
    let (low, high) = (0.5 - HUB_HALF, 0.5 + HUB_HALF);
    // Where the lines of the hub's sides meet the circle.
    let reach = (NEPALI_RADIUS * NEPALI_RADIUS - HUB_HALF * HUB_HALF).sqrt();
    let (near, far) = (0.5 - reach, 0.5 + reach);
    let corner = |half_hours: u32| clock::at(CENTRE, NEPALI_RADIUS, half_hours);
    let (top_right, bottom_right, bottom_left, top_left) =
        (corner(3), corner(9), corner(15), corner(21));
    // Each house: the hub's corners it touches, clockwise about the centre,
    // and the arc of the circle it runs anticlockwise along.
    let houses: [(Vec<Point>, Point, Point); 12] = [
        (vec![p(low, low), p(high, low)], p(high, near), p(low, near)),
        (vec![p(low, low)], p(low, near), top_left),
        (vec![p(low, low)], top_left, p(near, low)),
        (vec![p(low, high), p(low, low)], p(near, low), p(near, high)),
        (vec![p(low, high)], p(near, high), bottom_left),
        (vec![p(low, high)], bottom_left, p(low, far)),
        (vec![p(high, high), p(low, high)], p(low, far), p(high, far)),
        (vec![p(high, high)], p(high, far), bottom_right),
        (vec![p(high, high)], bottom_right, p(far, high)),
        (vec![p(high, low), p(high, high)], p(far, high), p(far, low)),
        (vec![p(high, low)], p(far, low), top_right),
        (vec![p(high, low)], top_right, p(high, near)),
    ];
    let cells = (1..=12u8)
        .zip(houses)
        .map(|(house, (hub, from, to))| {
            let mut segments: Vec<Segment> =
                hub.iter().skip(1).map(|&to| Segment::Line { to }).collect();
            segments.push(Segment::Line { to: from });
            segments.push(Segment::Arc {
                centre: CENTRE,
                clockwise: false,
                to,
            });
            let start = hub.first().copied().unwrap_or(CENTRE);
            let (label, bodies) = if house % 3 == 1 {
                (EDGE_LABEL, EDGE_BODIES)
            } else {
                (CORNER_LABEL, CORNER_BODIES)
            };
            Cell {
                outline: Path { start, segments },
                holds: Holds::House(house),
                label: clock::at(CENTRE, label, facing(house)),
                bodies: clock::at(CENTRE, bodies, facing(house)),
            }
        })
        .collect();
    Layout {
        key: String::from("NEPALI_CIRCLE"),
        sources: vec![String::from(PATRIKAS)],
        shape: Shape::Grid(Grid {
            cells,
            frame: vec![border(), circle(NEPALI_RADIUS), circle(HUB_RADIUS)],
            direction: Direction::Anticlockwise,
            hub: Some(Hub {
                centre: CENTRE,
                radius: HUB_RADIUS,
            }),
        }),
    }
}

/// What the petalled North Indian chart is drawn from.
const PETALS_VIEW: &str = "a Nepali app's kundali view, read 2026-11-10";

/// Where a kendra's curved edges meet the corner diagonals: 37/128 of the
/// side along each, where plain North Indian meets them at a quarter (the
/// view measures 0.291).
const PETAL_VERTEX: f64 = 37.0 / 128.0;

/// How far in from the side a kendra's curve flattens: the view measures
/// 0.142 of the side.
const PETAL_FLAT: f64 = 9.0 / 64.0;

/// Half the side of the hub where the diagonals stop: the view's hub is
/// about 0.055 of the side across.
const PETAL_HUB: f64 = 7.0 / 256.0;

/// A point turned a quarter anticlockwise about the square's centre, as a
/// reader sees it: the top midpoint goes to the left one. Each coordinate
/// is a dyadic fraction or one minus one, so the turn is exact.
fn quarter(point: Point) -> Point {
    p(point.y, 1.0 - point.x)
}

/// An outline turned `turns` quarters anticlockwise.
fn turned(outline: &Path, turns: u8) -> Path {
    let turn = |point: Point| (0..turns).fold(point, |point, _| quarter(point));
    Path {
        start: turn(outline.start),
        segments: outline
            .segments
            .iter()
            .map(|&segment| match segment {
                Segment::Line { to } => Segment::Line { to: turn(to) },
                Segment::Quad { control, to } => Segment::Quad {
                    control: turn(control),
                    to: turn(to),
                },
                Segment::Arc {
                    centre,
                    clockwise,
                    to,
                } => Segment::Arc {
                    centre: turn(centre),
                    clockwise,
                    to: turn(to),
                },
            })
            .collect(),
    }
}

/// North Indian drawn with petals, as a consumer's kundali view draws it:
/// the same houses, house 1 the top kendra and running anticlockwise, with
/// North Indian's anchors, but each kendra's four straight edges are ogee
/// curves, and the corner diagonals stop at a small hub drawn as a
/// swastika.
///
/// Each curve leaves a side's midpoint at right angles to the side,
/// flattens 9/64 of the side in, and meets the diagonal at 37/128 of it.
/// It is two quadratic steps through the point halfway along the flat, so
/// the curve has no corner. The hub's arms are the kendras' inner edges:
/// each diagonal ends at an arm's tip, and each kendra takes the quarter of
/// the hub its arm opens towards, so the cells still fill the square.
///
/// The figure is drawn once, the top kendra and the two cells left of it,
/// and turned a quarter at a time; the right half of every curve is the
/// left half's mirror to the bit (`03-design/chart-geometry.md` §7h).
///
/// ```
/// use teistro_geometry::{Holds, Point, Segment, rows};
///
/// let petals = rows::north_indian_petals();
/// let plain = rows::north_indian();
/// let (grid, square) = (petals.shape.as_grid().unwrap(), plain.shape.as_grid().unwrap());
/// // North Indian's houses at North Indian's anchors.
/// for (cell, same) in grid.cells.iter().zip(&square.cells) {
///     assert_eq!((cell.holds, cell.label, cell.bodies), (same.holds, same.label, same.bodies));
/// }
/// // A kendra is bounded by curves; a corner cell by one.
/// let quads = |house: usize| {
///     grid.cells[house - 1].outline.segments.iter().filter(|step| matches!(step, Segment::Quad { .. })).count()
/// };
/// assert_eq!((quads(1), quads(2)), (4, 2));
/// // The hub's quarters are the kendras', each the one its arm opens to.
/// let holder = |x: f64, y: f64| {
///     grid.cells.iter().find(|cell| cell.outline.contains(Point::new(x, y))).map(|cell| cell.holds)
/// };
/// assert_eq!(holder(0.49, 0.49), Some(Holds::House(1)));
/// assert_eq!(holder(0.49, 0.51), Some(Holds::House(4)));
/// assert_eq!(holder(0.51, 0.51), Some(Holds::House(7)));
/// assert_eq!(holder(0.51, 0.49), Some(Holds::House(10)));
/// ```
#[must_use]
pub fn north_indian_petals() -> Layout {
    let (v, f, h) = (PETAL_VERTEX, PETAL_FLAT, PETAL_HUB);
    // Halfway along the flat, where a curve's two steps meet.
    let q = f64::midpoint(0.5, v);
    let (low, high) = (0.5 - h, 0.5 + h);
    let top = p(0.5, 0.0);
    // The top kendra: out along the right curve, down its diagonal to the
    // arm's tip, round the hub's arms, up the left diagonal and back along
    // the left curve.
    let kendra = Path {
        start: top,
        segments: vec![
            Segment::Quad {
                control: p(0.5, f),
                to: p(1.0 - q, f),
            },
            Segment::Quad {
                control: p(1.0 - v, f),
                to: p(1.0 - v, v),
            },
            Segment::Line { to: p(high, low) },
            Segment::Line { to: p(0.5, low) },
            Segment::Line { to: CENTRE },
            Segment::Line { to: p(low, 0.5) },
            Segment::Line { to: p(low, low) },
            Segment::Line { to: p(v, v) },
            Segment::Quad {
                control: p(v, f),
                to: p(q, f),
            },
            Segment::Quad {
                control: p(0.5, f),
                to: top,
            },
        ],
    };
    // House 2: the corner, the top midpoint, the curve down to the
    // diagonal, and back along it.
    let upper = Path {
        start: p(0.0, 0.0),
        segments: vec![
            Segment::Line { to: top },
            Segment::Quad {
                control: p(0.5, f),
                to: p(q, f),
            },
            Segment::Quad {
                control: p(v, f),
                to: p(v, v),
            },
        ],
    };
    // House 3: the corner, along the diagonal, and the left kendra's curve
    // out to the left midpoint, which is the top one's turned a quarter.
    let lower = Path {
        start: p(0.0, 0.0),
        segments: vec![
            Segment::Line { to: p(v, v) },
            Segment::Quad {
                control: p(f, v),
                to: p(f, q),
            },
            Segment::Quad {
                control: p(f, 0.5),
                to: p(0.0, 0.5),
            },
        ],
    };
    let cells = (0..4u8)
        .flat_map(|turns| {
            [&kendra, &upper, &lower]
                .into_iter()
                .zip(1..)
                .map(move |(outline, place)| house_cell(3 * turns + place, turned(outline, turns)))
        })
        .collect();
    Layout {
        key: String::from("NORTH_INDIAN_PETALS"),
        sources: vec![String::from(PETALS_VIEW)],
        shape: Shape::Grid(Grid {
            cells,
            frame: vec![border()],
            direction: Direction::Anticlockwise,
            hub: None,
        }),
    }
}
