//! A second birth a chart is read against: the partner a match, a
//! synastry or a Davison chart takes (`03-design/matching.md`,
//! `03-design/western-synastry.md`).

use serde::{Deserialize, Serialize};
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_core::time::UtcOffset;

/// A second birth, which every chart of a batch is read against.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Partner {
    /// The birth's instant.
    pub instant: JulianDay<Utc>,
    /// Where it happened.
    pub place: Place,
    /// Its civil clock, which a chart's day is reckoned by; UTC when left
    /// out.
    #[serde(rename = "utcOffsetSeconds", default)]
    pub utc_offset: UtcOffset,
}
