//! A drawing: one chart, the founded one or one of its divisional charts,
//! placed in one layout. It is what a chart document carries
//! (`03-design/chart-geometry.md` §4).
//!
//! A request names the pairs it wants, D1 in North Indian and D9 in North
//! Indian, say. Placing every layout for every divisional chart would
//! multiply what nobody asked for, and some pairs cannot be drawn at all.

use serde::{Deserialize, Serialize};
use teistro_chart::foundation::ChartFoundation;
use teistro_core::catalogue::{ChartFrame, Varga};
use teistro_core::error::Error;
use teistro_vargas::chart::{Axis, chart as varga_chart};

use crate::layout::{Layout, Reference, Shape};
use crate::place::{Placed, Placements, place_from};

/// One chart placed in one layout.
///
/// ```
/// # let foundation = teistro_chart::sample::foundation()?;
/// use teistro_core::catalogue::Varga;
/// use teistro_geometry::{draw, rows};
///
/// let drawing = draw(&rows::north_indian(), &foundation, Varga::D9)?;
/// assert_eq!(drawing.varga, Varga::D9);
/// assert_eq!(drawing.placed.cells.len(), 12);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Drawing {
    /// Which chart is drawn: `D1` for the founded chart, or a divisional one.
    pub varga: Varga,
    /// What its houses count from: the lagna unless the request named
    /// another, and left out of the document then.
    #[serde(default = "lagna", skip_serializing_if = "is_lagna")]
    pub counts_from: ChartFrame,
    /// The chart, placed.
    pub placed: Placed,
}

const fn lagna() -> ChartFrame {
    ChartFrame::Lagna
}

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde's skip_serializing_if passes a reference"
)]
fn is_lagna(frame: &ChartFrame) -> bool {
    *frame == ChartFrame::Lagna
}

impl Layout {
    /// Whether the layout draws by longitude, as a wheel does, and so
    /// needs a chart with degrees and cusps: a founded chart, never a
    /// divisional one.
    ///
    /// ```
    /// use teistro_geometry::rows;
    ///
    /// assert!(rows::western_wheel().draws_by_longitude());
    /// // The chakra's rings count from signs, so any chart can fill them.
    /// assert!(!rows::sudarshan_chakra().draws_by_longitude());
    /// assert!(!rows::north_indian().draws_by_longitude());
    /// ```
    #[must_use]
    pub fn draws_by_longitude(&self) -> bool {
        match &self.shape {
            Shape::Grid(_) => false,
            Shape::Radial(radial) => radial
                .rings
                .iter()
                .any(|ring| matches!(ring.counts_from, Reference::Cusps | Reference::Zodiac)),
        }
    }
}

/// Draws a founded chart, or one of its divisional charts, in a layout.
///
/// ```
/// # let foundation = teistro_chart::sample::foundation()?;
/// use teistro_core::catalogue::Varga;
/// use teistro_geometry::{draw, rows};
///
/// assert!(draw(&rows::western_wheel(), &foundation, Varga::D1).is_ok());
/// // A divisional chart has no degrees for a wheel, and is refused by name.
/// let refused = draw(&rows::western_wheel(), &foundation, Varga::D9).unwrap_err();
/// assert_eq!(refused.field(), Some("varga"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// A divisional chart in a layout that draws by longitude, which a
/// divisional chart has no degrees for: refused by naming both, rather
/// than by the missing degree it would otherwise fail on. And whatever the
/// divisional chart or the placement refuses.
pub fn draw(layout: &Layout, foundation: &ChartFoundation, varga: Varga) -> Result<Drawing, Error> {
    draw_from(layout, foundation, varga, ChartFrame::Lagna)
}

/// Draws a founded chart, or one of its divisional charts, in a layout, its
/// houses counted as `counts_from` says ([`place_from`]).
///
/// ```
/// # let foundation = teistro_chart::sample::foundation()?;
/// use teistro_core::catalogue::{ChartFrame, Graha, Varga};
/// use teistro_geometry::{draw_from, rows};
///
/// let moon = draw_from(&rows::nepali_circle(), &foundation, Varga::D1, ChartFrame::Moon)?;
/// let first = moon.placed.cells.iter().find(|cell| cell.house == 1).unwrap();
/// assert!(first.bodies.contains(&Graha::Moon.key_id()));
/// // A divisional chart has no cusps, so no bhava chalit.
/// let refused = draw_from(&rows::nepali_circle(), &foundation, Varga::D9, ChartFrame::Bhava).unwrap_err();
/// assert_eq!(refused.field(), Some("from"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`draw`], and whatever [`place_from`] refuses.
pub fn draw_from(
    layout: &Layout,
    foundation: &ChartFoundation,
    varga: Varga,
    counts_from: ChartFrame,
) -> Result<Drawing, Error> {
    let chart = if varga == Varga::D1 {
        Placements::of_chart(foundation)
    } else {
        if layout.draws_by_longitude() {
            return Err(Error::invalid_arg(format!(
                "the {} layout draws by longitude, and a divisional chart such as {} has signs \
                 but no degrees; draw it from D1",
                layout.key,
                varga.key()
            ))
            .with_field("varga")
            .with_hint(format!(
                "draw {} in a grid layout or the Sudarshan Chakra",
                varga.key()
            )));
        }
        Placements::of_varga(&varga_chart(foundation, Axis::of(varga))?)
    };
    Ok(Drawing {
        varga,
        counts_from,
        placed: place_from(layout, &chart, counts_from)?,
    })
}
