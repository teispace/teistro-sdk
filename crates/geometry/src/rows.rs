//! The layouts that ship, each a cited row (`03-design/chart-geometry.md` §2).
//!
//! | row | fixed | starts | runs | cited |
//! |---|---|---|---|---|
//! | [`north_indian`] | houses | house 1, the top diamond | anticlockwise | Wikipedia, *Kundali (astrology)*, `Kundali_02a.png` |
//! | [`south_indian`] | signs | Pisces top-left | clockwise | the same article, `Kundali_01a.png` |
//! | [`east_indian`] | signs | Aries top-centre | anticlockwise | the same article, `Kundali_03a.png` |
//! | [`nepali_lotus`] | houses | as North Indian | anticlockwise | the baseline application's lotus frame |
//!
//! Every coordinate of the three square figures is a half, a third or a
//! quarter, reached by IEEE's basic operations, which every platform does
//! identically; and placement only chooses cells, so nothing here can differ
//! between architectures (§7).

use teistro_core::catalogue::Rashi;

use crate::layout::{Cell, Direction, Grid, Holds, Layout};
use crate::path::{Path, Point, Segment};

/// Every layout the SDK ships, in the order a consumer is offered them.
#[must_use]
pub fn shipped() -> Vec<Layout> {
    vec![
        north_indian(),
        south_indian(),
        east_indian(),
        nepali_lotus(),
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

/// The North Indian houses' anchors, shared with the lotus, whose petals
/// sit on the same topology: where the number and the bodies of house `n`
/// go, from the baseline application's chart (in units of 1/400).
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
        grid: Grid {
            cells: (1..=12)
                .zip(outlines)
                .map(|(house, points)| house_cell(house, Path::polygon(&points)))
                .collect(),
            frame: vec![border()],
            direction: Direction::Anticlockwise,
        },
    }
}

/// The South Indian chart: the ring of twelve cells around a 4×4 grid's
/// empty centre. The signs are fixed, Pisces top-left and Aries beside it,
/// running clockwise; the houses turn with the lagna.
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
        grid: Grid {
            cells,
            frame: vec![border(), square(0.25, 0.25, 0.75, 0.75)],
            direction: Direction::Clockwise,
        },
    }
}

/// The East Indian (Bengali, Odia, Assamese) chart: a 3×3 grid with an empty
/// centre, each corner cut from its outer corner to its inner one into two
/// signs. The signs are fixed, Aries top-centre, running anticlockwise.
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
        grid: Grid {
            cells,
            frame: vec![border()],
            direction: Direction::Anticlockwise,
        },
    }
}

/// The lotus's petals as the baseline application draws them, verbatim, in
/// its 400 by 266.67 space that it stretches half again vertically, with the
/// house each outlines.
const LOTUS_PETALS: [(u8, &str); 12] = [
    (
        1,
        "M 200,0 Q 200,36.3636 241.665,38.095 Q 283.33,39.8264 283.33,76.19 L 210 123.33 L 200 123.33 L 200 133.33 L 190 133.33 L 190 123.33 L 116.67 76.19 Q 116.67,39.8264 158.335,38.095 Q 200,36.3636 200,0 z",
    ),
    (
        10,
        "M 283.33 76.19 Q 319.6936,76.19 341.665,104.76 Q 363.6364,133.33 400,133.33 Q 363.6364,133.33 341.665,162.665 Q 319.6936,192 283.33,192 L 210,143.33 L 210,133.33 L 200 133.33 L 200 123.33 L 210 123.33 z",
    ),
    (
        7,
        "M 200,133.33 L 210,133.33 L 210,143.33 L 283.33,192 Q 283.33,228.3636 241.665,229.335 Q 200,230.3064 200,266.67 Q 200,230.3064 158.335,229.335 Q 116.67,228.3636 116.67,192 L 190,143.33 L 200,143.33 z",
    ),
    (
        4,
        "M 200,133.33 L 200,143.33 L 190,143.33 L 116.67,192 Q 80.3064,192 58.335,162.665 Q 36.3636,133.33 0,133.33 Q 36.3636,133.33 58.335,104.76 Q 80.3064,76.19 116.67,76.19 L 190,123.33 L 190,133.33 z",
    ),
    (
        12,
        "M 200,0 Q 200,36.3636 241.665,38.095 Q 283.33,39.8264 283.33,76.19 L 400, 0 z",
    ),
    (
        2,
        "M 116.67,76.19 Q 116.67,39.8264 158.335,38.095 Q 200,36.3636 200,0 L 0, 0 z",
    ),
    (
        3,
        "M 0,133.33 Q 36.3636,133.33 58.335,104.76 Q 80.3064,76.19 116.67,76.19 L 0, 0 z",
    ),
    (
        5,
        "M 116.67,192 Q 80.3064,192 58.335,162.665 Q 36.3636,133.33 0,133.33 L 0, 266.67 z",
    ),
    (
        6,
        "M 200,266.67 Q 200,230.3064 158.335,229.335 Q 116.67,228.3636 116.67,192 L 0, 266.67 z",
    ),
    (
        8,
        "M 283.33,192 Q 283.33,228.3636 241.665,229.335 Q 200,230.3064 200,266.67 L 400, 266.67 z",
    ),
    (
        9,
        "M 400,133.33 Q 363.6364,133.33 341.665,162.665 Q 319.6936,192 283.33,192 L 400, 266.67 z",
    ),
    (
        11,
        "M 283.33,76.19 Q 319.6936,76.19 341.665,104.76 Q 363.6364,133.33 400,133.33 L 400, 0 z",
    ),
];

/// The source's values that are thirds written to two places, and the thirds
/// they are. Without them the lotus's bottom edge sits at 1.0000125 and
/// its centre is not the centre.
const LOTUS_THIRDS: [(f64, f64); 6] = [
    (116.67, 350.0 / 3.0),
    (123.33, 370.0 / 3.0),
    (133.33, 400.0 / 3.0),
    (143.33, 430.0 / 3.0),
    (266.67, 800.0 / 3.0),
    (283.33, 850.0 / 3.0),
];

/// The Nepali lotus (Ashtadala Padma): the North Indian houses drawn as
/// petals, their edges quadratic curves.
#[must_use]
pub fn nepali_lotus() -> Layout {
    let cells = LOTUS_PETALS
        .iter()
        .map(|&(house, d)| house_cell(house, lotus_path(d)))
        .collect();
    Layout {
        key: String::from("NEPALI_LOTUS"),
        sources: vec![String::from(
            "the baseline application's kundali lotus frame, as drawn to its readers",
        )],
        grid: Grid {
            cells,
            frame: vec![border()],
            direction: Direction::Anticlockwise,
        },
    }
}

/// One of the lotus's path strings in the unit square. The source uses
/// only absolute `M`, `L` and `Q` commands and `z`, so that is all this
/// reads; a command it does not know would be a changed source and fails
/// the layout's validation rather than passing silently.
fn lotus_path(d: &str) -> Path {
    let thirds = |value: f64| {
        LOTUS_THIRDS
            .iter()
            .find(|(written, _)| (value - written).abs() < 1e-9)
            .map_or(value, |(_, exact)| *exact)
    };
    let point = |x: f64, y: f64| p(thirds(x) / 400.0, thirds(y) * 1.5 / 400.0);
    let mut numbers = d
        .split(|c: char| c.is_whitespace() || c == ',' || c.is_ascii_alphabetic())
        .filter(|token| !token.is_empty())
        .map(|token| token.parse::<f64>().unwrap_or(f64::NAN));
    let mut take = || numbers.next().unwrap_or(f64::NAN);
    let mut path = Path {
        start: p(f64::NAN, f64::NAN),
        segments: Vec::new(),
    };
    for command in d.chars().filter(char::is_ascii_alphabetic) {
        match command {
            'M' => {
                let (x, y) = (take(), take());
                path.start = point(x, y);
            }
            'L' => {
                let (x, y) = (take(), take());
                path.segments.push(Segment::Line { to: point(x, y) });
            }
            'Q' => {
                let (cx, cy, x, y) = (take(), take(), take(), take());
                path.segments.push(Segment::Quad {
                    control: point(cx, cy),
                    to: point(x, y),
                });
            }
            _ => {}
        }
    }
    path
}
