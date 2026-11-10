//! Chart layouts as data, and a chart placed in one (ADR-0026,
//! `03-design/chart-geometry.md`).
//!
//! A developer who can compute a chart and cannot draw one has not finished
//! evaluating the SDK. This crate is the part of drawing that is chart data:
//! which region of the page belongs to which sign or house, and where its
//! label and its bodies go. The core still never draws; a renderer, the
//! SDK's own or a consumer's, reads [`Placed`].
//!
//! - [`path`]: points and outlines in the unit square, y downwards, with the
//!   curves the lotus needs;
//! - [`layout`]: a layout as a row, and the checks that refuse a wrong one
//!   by the cell it gets wrong;
//! - [`rows`]: the North, South and East Indian charts, the Nepali lotus and
//!   round chart, the Sudarshan Chakra, the Western wheel and North Indian
//!   drawn with petals, each cited;
//! - [`clock`]: the directions a radial layout and the Nepali charts need,
//!   the same on every platform;
//! - [`place`](mod@place): a chart placed in a layout, every cell carrying both its sign
//!   and its house;
//! - [`registry`]: the shipped layouts and a consumer's own, looked up by key.
//!
//! Three things the research found (§2):
//!
//! - **"Bengali" is the East Indian chart**, which every source names the
//!   Bengali, Odia or Assamese chart, so it is one row and not two.
//! - **Layouts are two kinds.** The square charts are cells fixed in the
//!   row; the Sudarshan Chakra's rings and a Western wheel's houses are
//!   computed per chart.
//! - **The Nepali charts are printed, not invented.** The lotus is twelve
//!   equal petals about the centre and the round chart a circle cut on a
//!   square grid about a hub, both house 1 at the top and anticlockwise, as
//!   three printed janma patrikas draw them.
//!
//! ```
//! use teistro_core::catalogue::{Graha, Rashi};
//! use teistro_geometry::{Body, Placements, place, rows};
//!
//! // Every shipped layout passes the checks a consumer's own would.
//! for layout in rows::shipped() {
//!     layout.validate().expect("a shipped layout is valid");
//! }
//!
//! // In the South Indian chart the signs stay put: Aries is always the
//! // second cell of the top row, and with a Leo lagna it is the ninth house.
//! let chart = Placements::new(Rashi::Leo).with(Body::in_sign(Graha::Mars.key_id(), Rashi::Aries));
//! let placed = place(&rows::south_indian(), &chart)?;
//! let aries = placed.cells.iter().find(|cell| cell.sign == Rashi::Aries).unwrap();
//! assert_eq!(aries.house, 9);
//! assert_eq!(aries.bodies, vec![Graha::Mars.key_id()]);
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod clock;
pub mod drawing;
pub mod layout;
pub mod path;
pub mod place;
pub mod registry;
pub mod rows;

pub use drawing::{Drawing, draw, draw_from};
pub use layout::{Cell, Direction, Grid, Holds, Hub, Layout, Radial, Reference, Ring, Shape};
pub use path::{Path, Point, Segment};
pub use place::{Body, Mark, Placed, PlacedCell, Placements, place, place_from};
pub use registry::Layouts;
