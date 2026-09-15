//! A drawing: one chart, the founded one or one of its divisional charts,
//! placed in one layout. It is what a chart document carries
//! (`03-design/chart-geometry.md` §4).
//!
//! A request names the pairs it wants, D1 in North Indian and D9 in North
//! Indian, say. Placing every layout for every divisional chart would
//! multiply what nobody asked for, and some pairs cannot be drawn at all.

use serde::{Deserialize, Serialize};
use teistro_chart::foundation::ChartFoundation;
use teistro_core::catalogue::Varga;
use teistro_core::error::Error;
use teistro_vargas::chart::{Axis, chart as varga_chart};

use crate::layout::{Layout, Reference, Shape};
use crate::place::{Placed, Placements, place};

/// One chart placed in one layout.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Drawing {
    /// Which chart is drawn: `D1` for the founded chart, or a divisional one.
    pub varga: Varga,
    /// The chart, placed.
    pub placed: Placed,
}

impl Layout {
    /// Whether the layout draws by longitude, as a wheel does, and so
    /// needs a chart with degrees and cusps: a founded chart, never a
    /// divisional one.
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
/// # Errors
///
/// A divisional chart in a layout that draws by longitude, which a
/// divisional chart has no degrees for: refused by naming both, rather
/// than by the missing degree it would otherwise fail on. And whatever the
/// divisional chart or the placement refuses.
pub fn draw(layout: &Layout, foundation: &ChartFoundation, varga: Varga) -> Result<Drawing, Error> {
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
        placed: place(layout, &chart)?,
    })
}
