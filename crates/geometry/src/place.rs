//! A chart placed in a layout: every cell with both its sign and its house,
//! the bodies that stand in it, and, for a layout drawn by longitude, where
//! each body stands on the circle.
//!
//! A sign-fixed cell gets its house from the lagna and a house-fixed cell its
//! sign, so a consumer never repeats the arithmetic, and a divisional chart
//! places exactly as a rashi chart does, from its own lagna
//! (`03-design/chart-geometry.md` §4).

use serde::{Deserialize, Serialize};
use teistro_chart::bhava::Bhavas;
use teistro_chart::foundation::ChartFoundation;
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::error::Error;
use teistro_core::key::KeyId;
use teistro_core::math;
use teistro_vargas::chart::VargaChart;

use crate::clock;
use crate::layout::{Direction, Grid, Holds, Layout, Radial, Reference, Ring, Shape};
use crate::path::{Path, Point, Segment};

/// One body as placement reads it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Body {
    /// The body's catalogue key (`graha.SUN`, `point.GULIKA`).
    pub key: KeyId,
    /// The sign it stands in.
    pub sign: Rashi,
    /// Its longitude in the chart's zodiac, degrees, when it has one: a
    /// divisional chart's bodies have a sign and no degree.
    pub longitude_deg: Option<f64>,
}

impl Body {
    /// A body known by its sign alone.
    #[must_use]
    pub const fn in_sign(key: KeyId, sign: Rashi) -> Body {
        Body {
            key,
            sign,
            longitude_deg: None,
        }
    }

    /// A body at a longitude, which also gives its sign.
    #[must_use]
    pub fn at(key: KeyId, longitude_deg: f64) -> Body {
        Body {
            key,
            sign: sign_of_longitude(longitude_deg),
            longitude_deg: Some(longitude_deg),
        }
    }
}

/// What placement reads of a chart.
#[derive(Clone, Debug, PartialEq)]
pub struct Placements {
    /// The sign the lagna is in.
    pub lagna: Rashi,
    /// The lagna's longitude, degrees, when the chart has one.
    pub lagna_deg: Option<f64>,
    /// The chart's bhavas, when it has them: where each house begins, which
    /// a wheel's house sectors run between, and the rule that says which
    /// house a longitude is in, so the wheel and the chart cannot disagree.
    pub houses: Option<Bhavas>,
    /// The bodies, in the chart's order.
    pub bodies: Vec<Body>,
}

impl Placements {
    /// A chart known by its lagna's sign, with no bodies yet.
    #[must_use]
    pub const fn new(lagna: Rashi) -> Placements {
        Placements {
            lagna,
            lagna_deg: None,
            houses: None,
            bodies: Vec::new(),
        }
    }

    /// The same chart with a body added.
    #[must_use]
    pub fn with(mut self, body: Body) -> Placements {
        self.bodies.push(body);
        self
    }

    /// A founded chart: its lagna and grahas at their longitudes, and the
    /// cusps of the bhavas it places its grahas in.
    #[must_use]
    pub fn of_chart(chart: &ChartFoundation) -> Placements {
        Placements {
            lagna: sign_of(chart.lagna_sign_index()),
            lagna_deg: Some(chart.lagna_deg),
            houses: Some(chart.houses),
            bodies: chart
                .grahas
                .iter()
                .map(|body| Body::at(body.graha.key_id(), body.longitude_deg))
                .collect(),
        }
    }

    /// A divisional chart's own lagna and grahas, by sign.
    #[must_use]
    pub fn of_varga(chart: &VargaChart) -> Placements {
        Placements {
            bodies: chart
                .grahas
                .iter()
                .map(|placed| Body::in_sign(placed.graha.key_id(), placed.at.sign))
                .collect(),
            ..Placements::new(chart.lagna.sign)
        }
    }
}

/// One cell of a placed chart.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PlacedCell {
    /// The cell's outline.
    pub outline: Path,
    /// The sign the cell shows; for a house between cusps, the sign its cusp
    /// is in.
    pub sign: Rashi,
    /// The house the cell shows, 1 to 12.
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

/// A body drawn where it stands on a wheel: at its own longitude, not stacked
/// in a cell.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Mark {
    /// The body.
    pub body: KeyId,
    /// The ring it is drawn in.
    pub ring: u8,
    /// Where it is drawn.
    pub at: Point,
    /// The longitude that put it there, degrees.
    pub longitude_deg: f64,
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
    /// The bodies at their longitudes, for a layout with a ring drawn by
    /// longitude; empty otherwise.
    pub marks: Vec<Mark>,
}

/// Places a chart in a layout.
///
/// The layout is assumed valid, as every shipped and registered one is
/// ([`Layout::validate`]).
///
/// # Errors
///
/// A ring the chart cannot fill, refused by what it lacks rather than drawn
/// from something else: a ring counting from the Moon or the Sun for a chart
/// that lists neither, a ring of cusps for a chart with no cusps, a ring by
/// longitude for a chart whose lagna or bodies have no degree.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_geometry::{Body, Placements, place, rows};
///
/// // A Cancer lagna with the Sun in Leo, in the North Indian chart.
/// let chart = Placements::new(Rashi::Cancer).with(Body::in_sign(Graha::Sun.key_id(), Rashi::Leo));
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
    match &layout.shape {
        Shape::Grid(grid) => Ok(Placed {
            layout: layout.key.clone(),
            cells: place_grid(grid, chart),
            frame: grid.frame.clone(),
            marks: Vec::new(),
        }),
        Shape::Radial(radial) => place_radial(&layout.key, radial, chart),
    }
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
            PlacedCell {
                outline: cell.outline.clone(),
                sign,
                house,
                lagna: sign == chart.lagna,
                ring: 0,
                label: cell.label,
                anchor: cell.bodies,
                bodies: in_sign(chart, sign),
            }
        })
        .collect()
}

/// The square's centre, which every ring turns about.
const CENTRE: Point = Point::new(0.5, 0.5);

/// A direction from the centre: a whole number of half hours on the clock,
/// exact on every platform, or a bearing in degrees from a longitude, which
/// takes a platform's `sin` and is rounded to [`GRAIN`].
#[derive(Clone, Copy, Debug)]
enum Bearing {
    HalfHours(u32),
    Degrees(f64),
}

/// What a coordinate reached through `sin` and `cos` is rounded to: far finer
/// than any drawing shows, and coarse enough that the last unit a platform's
/// trigonometry may differ in never reaches the output (§7).
pub const GRAIN: f64 = 1e-9;

fn point(radius: f64, bearing: Bearing) -> Point {
    match bearing {
        Bearing::HalfHours(half_hours) => clock::at(CENTRE, radius, half_hours),
        Bearing::Degrees(degrees) => {
            let angle = degrees.to_radians();
            let grain = |v: f64| (v / GRAIN).round() * GRAIN;
            let (sin, cos) = math::sin_cos(angle);
            Point::new(
                grain(CENTRE.x + radius * sin),
                grain(CENTRE.y - radius * cos),
            )
        }
    }
}

/// One sector's bounds, the way it runs, and where its words go.
struct Span {
    begins: Bearing,
    ends: Bearing,
    middle: Bearing,
}

fn place_radial(key: &str, radial: &Radial, chart: &Placements) -> Result<Placed, Error> {
    let clockwise = radial.direction == Direction::Clockwise;
    let mut cells = Vec::with_capacity(radial.rings.len() * 12);
    let mut marks = Vec::new();
    for (index, ring) in (0u8..).zip(&radial.rings) {
        let spans = spans(key, radial, ring, chart)?;
        for (span, (sign, house, bodies)) in spans {
            let (width, middle) = (
                ring.outer - ring.inner,
                f64::midpoint(ring.inner, ring.outer),
            );
            cells.push(PlacedCell {
                outline: sector(ring.inner, ring.outer, span.begins, span.ends, clockwise),
                sign,
                house,
                lagna: sign == chart.lagna,
                ring: index,
                label: point(middle - width * 0.18, span.middle),
                anchor: point(middle + width * 0.18, span.middle),
                bodies,
            });
        }
        if matches!(ring.counts_from, Reference::Cusps | Reference::Zodiac) && marks.is_empty() {
            let ascendant = lagna_deg(key, chart)?;
            for (position, body) in chart.bodies.iter().enumerate() {
                let longitude = degree_of(key, position, body)?;
                marks.push(Mark {
                    body: body.key,
                    ring: index,
                    at: point(
                        f64::midpoint(ring.inner, ring.outer),
                        Bearing::Degrees(bearing_of(radial, ascendant, longitude)),
                    ),
                    longitude_deg: longitude,
                });
            }
        }
    }
    Ok(Placed {
        layout: key.to_owned(),
        cells,
        frame: radial_frame(radial),
        marks,
    })
}

type Filled = (Span, (Rashi, u8, Vec<KeyId>));

/// A ring's twelve sectors, each with the sign, house and bodies it shows.
fn spans(
    key: &str,
    radial: &Radial,
    ring: &Ring,
    chart: &Placements,
) -> Result<Vec<Filled>, Error> {
    let by_hours = |from: Rashi| -> Vec<Filled> {
        (1..=12u8)
            .map(|house| {
                let begins = hour_of(radial, house);
                let ends = hour_of(radial, house + 1);
                let middle = match radial.direction {
                    Direction::Clockwise => begins * 2 + 1,
                    Direction::Anticlockwise => (begins * 2 + 23) % 24,
                };
                let sign = sign_after(from.id(), house);
                (
                    Span {
                        begins: Bearing::HalfHours(begins * 2),
                        ends: Bearing::HalfHours(ends * 2),
                        middle: Bearing::HalfHours(middle),
                    },
                    (sign, house, in_sign(chart, sign)),
                )
            })
            .collect()
    };
    Ok(match ring.counts_from {
        Reference::Lagna => by_hours(chart.lagna),
        Reference::Moon => by_hours(sign_of_body(key, chart, Graha::Moon)?),
        Reference::Sun => by_hours(sign_of_body(key, chart, Graha::Sun)?),
        Reference::Zodiac => {
            let ascendant = lagna_deg(key, chart)?;
            (0..12u16)
                .map(|sign_index| {
                    let start = f64::from(sign_index) * 30.0;
                    let sign = Rashi::from_id(sign_index).unwrap_or(Rashi::Aries);
                    (
                        by_longitude(radial, ascendant, start, start + 30.0),
                        (
                            sign,
                            house_of(sign_index, chart.lagna.id()),
                            in_sign(chart, sign),
                        ),
                    )
                })
                .collect()
        }
        Reference::Cusps => {
            let ascendant = lagna_deg(key, chart)?;
            let houses = chart.houses.as_ref().ok_or_else(|| {
                Error::invalid_arg(format!(
                    "the {key} layout draws its houses between cusps, and the chart has none"
                ))
                .with_field("houses")
            })?;
            let mut filled = Vec::with_capacity(12);
            let starts = houses
                .sandhi
                .iter()
                .zip(houses.sandhi.iter().cycle().skip(1));
            for (house, (start, next)) in (1..=12u8).zip(starts) {
                let mut bodies = Vec::new();
                for (position, body) in chart.bodies.iter().enumerate() {
                    if houses.place(degree_of(key, position, body)?).bhava == house {
                        bodies.push(body.key);
                    }
                }
                filled.push((
                    by_longitude(radial, ascendant, *start, *next),
                    (sign_of_longitude(*start), house, bodies),
                ));
            }
            filled
        }
    })
}

/// The sector from one longitude forward to the next, drawn with the lagna
/// at the start hour.
fn by_longitude(radial: &Radial, ascendant: f64, start: f64, next: f64) -> Span {
    let width = (next - start).rem_euclid(360.0);
    Span {
        begins: Bearing::Degrees(bearing_of(radial, ascendant, start)),
        ends: Bearing::Degrees(bearing_of(radial, ascendant, next)),
        middle: Bearing::Degrees(bearing_of(radial, ascendant, start + width / 2.0)),
    }
}

/// Where a longitude is drawn, in degrees clockwise from twelve o'clock: the
/// lagna at the start hour, longitudes increasing the way the houses run.
fn bearing_of(radial: &Radial, ascendant: f64, longitude: f64) -> f64 {
    let start = f64::from(radial.starts_at % 12) * 30.0;
    let past = (longitude - ascendant).rem_euclid(360.0);
    match radial.direction {
        Direction::Clockwise => (start + past).rem_euclid(360.0),
        Direction::Anticlockwise => (start - past).rem_euclid(360.0),
    }
}

fn lagna_deg(key: &str, chart: &Placements) -> Result<f64, Error> {
    chart.lagna_deg.ok_or_else(|| {
        Error::invalid_arg(format!(
            "the {key} layout draws by longitude, and the chart's lagna has no degree"
        ))
        .with_field("lagna_deg")
    })
}

fn degree_of(key: &str, position: usize, body: &Body) -> Result<f64, Error> {
    body.longitude_deg.ok_or_else(|| {
        Error::invalid_arg(format!(
            "the {key} layout draws by longitude, and this body has no degree"
        ))
        .with_field(format!("bodies[{position}].longitude_deg"))
    })
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
fn sector(inner: f64, outer: f64, begins: Bearing, ends: Bearing, clockwise: bool) -> Path {
    let mut segments = vec![Segment::Arc {
        centre: CENTRE,
        clockwise,
        to: point(outer, ends),
    }];
    if inner > 0.0 {
        segments.push(Segment::Line {
            to: point(inner, ends),
        });
        segments.push(Segment::Arc {
            centre: CENTRE,
            clockwise: !clockwise,
            to: point(inner, begins),
        });
    } else {
        segments.push(Segment::Line { to: CENTRE });
    }
    Path {
        start: point(outer, begins),
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
        .find(|body| body.key == wanted)
        .map(|body| body.sign)
        .ok_or_else(|| {
            Error::invalid_arg(format!(
                "the {key} layout has a ring that counts from the {}, and the chart lists no {}",
                graha.key(),
                graha.full_key()
            ))
            .with_field("bodies")
        })
}

fn in_sign(chart: &Placements, sign: Rashi) -> Vec<KeyId> {
    chart
        .bodies
        .iter()
        .filter(|body| body.sign == sign)
        .map(|body| body.key)
        .collect()
}

fn sign_of(index: u8) -> Rashi {
    Rashi::from_id(u16::from(index) % 12).unwrap_or(Rashi::Aries)
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a normalised longitude over thirty is 0 to 11"
)]
fn sign_of_longitude(longitude_deg: f64) -> Rashi {
    Rashi::from_id((longitude_deg.rem_euclid(360.0) / 30.0) as u16 % 12).unwrap_or(Rashi::Aries)
}

/// The sign a house is, counting inclusively from the lagna's sign.
fn sign_after(lagna: u16, house: u8) -> Rashi {
    Rashi::from_id((lagna + u16::from(house) + 11) % 12).unwrap_or(Rashi::Aries)
}

/// The house a sign is, counted inclusively from the lagna's sign.
fn house_of(sign: u16, lagna: u16) -> u8 {
    u8::try_from((sign + 12 - lagna) % 12 + 1).unwrap_or(1)
}
