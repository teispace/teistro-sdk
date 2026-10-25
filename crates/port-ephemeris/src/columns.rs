//! The columnar response: one vector per quantity over a grid of instants
//! and bodies, instants outermost, with a status and a source per cell
//! (`docs/03-design/ephemeris-port-and-adapters.md`, §3).

use serde::Serialize;
use teistro_core::error::Status;
use teistro_core::settings::Tier;

use crate::frame::Frame;

/// The status of one cell of a response; a failing cell never aborts the
/// batch.
///
/// ```
/// use teistro_port_ephemeris::CellStatus;
///
/// assert_eq!(serde_json::to_string(&CellStatus::OutOfRange).unwrap(), r#"{"kind":"OUT_OF_RANGE"}"#);
/// assert_eq!(serde_json::to_string(&CellStatus::Provider { code: 7 }).unwrap(), r#"{"kind":"PROVIDER","code":7}"#);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CellStatus {
    /// Computed.
    Ok,
    /// Not touched by the provider.
    NotComputed,
    /// The provider cannot compute this body.
    UnsupportedBody,
    /// The instant is outside the provider's coverage.
    OutOfRange,
    /// A data file the instant needs is missing.
    DataMissing,
    /// The provider's own code, outside the reserved range.
    Provider {
        /// The code.
        code: i32,
    },
}

impl CellStatus {
    /// The key it is serialised as: `OK`, `OUT_OF_RANGE`, `PROVIDER` for
    /// a provider's own code.
    ///
    /// ```
    /// use teistro_port_ephemeris::CellStatus;
    ///
    /// assert_eq!(CellStatus::UnsupportedBody.key(), "UNSUPPORTED_BODY");
    /// assert_eq!(CellStatus::Provider { code: 7 }.key(), "PROVIDER");
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            CellStatus::Ok => "OK",
            CellStatus::NotComputed => "NOT_COMPUTED",
            CellStatus::UnsupportedBody => "UNSUPPORTED_BODY",
            CellStatus::OutOfRange => "OUT_OF_RANGE",
            CellStatus::DataMissing => "DATA_MISSING",
            CellStatus::Provider { .. } => "PROVIDER",
        }
    }

    /// The stable code at the C boundary: `0` ok, the reserved negatives
    /// as [`crate::ProviderError::code`], anything else the provider's own.
    ///
    /// ```
    /// use teistro_port_ephemeris::{CellStatus, ProviderError};
    ///
    /// // The reserved codes are the request-level errors' codes.
    /// assert_eq!(CellStatus::OutOfRange.code(), ProviderError::OutOfRange { jd: 0.0 }.code());
    /// assert_eq!(CellStatus::Provider { code: 7 }.code(), 7);
    /// ```
    #[must_use]
    pub const fn code(self) -> i32 {
        match self {
            CellStatus::Ok => 0,
            CellStatus::NotComputed => -6,
            CellStatus::UnsupportedBody => -1,
            CellStatus::OutOfRange => -2,
            CellStatus::DataMissing => -3,
            CellStatus::Provider { code } => code,
        }
    }

    /// The SDK status a refused cell maps to, as
    /// [`crate::ProviderError::status`] maps a refused request: an instant
    /// outside the coverage is `OUT_OF_RANGE` whether one cell or the whole
    /// request was refused for it, and never a failure of the provider.
    ///
    /// ```
    /// use teistro_core::error::Status;
    /// use teistro_port_ephemeris::CellStatus;
    ///
    /// assert_eq!(CellStatus::OutOfRange.status(), Status::OutOfRange);
    /// assert_eq!(CellStatus::DataMissing.status(), Status::Provider);
    /// ```
    #[must_use]
    pub const fn status(self) -> Status {
        match self {
            CellStatus::Ok => Status::Ok,
            CellStatus::UnsupportedBody => Status::Unsupported,
            CellStatus::OutOfRange => Status::OutOfRange,
            CellStatus::NotComputed | CellStatus::DataMissing | CellStatus::Provider { .. } => {
                Status::Provider
            }
        }
    }

    /// The status from its code.
    ///
    /// ```
    /// use teistro_port_ephemeris::CellStatus;
    ///
    /// for status in [CellStatus::Ok, CellStatus::NotComputed, CellStatus::DataMissing, CellStatus::Provider { code: 42 }] {
    ///     assert_eq!(CellStatus::from_code(status.code()), status);
    /// }
    /// ```
    #[must_use]
    pub const fn from_code(code: i32) -> CellStatus {
        match code {
            0 => CellStatus::Ok,
            -6 => CellStatus::NotComputed,
            -1 => CellStatus::UnsupportedBody,
            -2 => CellStatus::OutOfRange,
            -3 => CellStatus::DataMissing,
            other => CellStatus::Provider { code: other },
        }
    }
}

/// Which ephemeris produced a cell.
///
/// ```
/// use teistro_port_ephemeris::{Body, EphemerisKind, EphemerisProvider, Frame, PositionRequest, TestProvider, TimeScale};
///
/// let request = PositionRequest::new(&[2_451_545.0], TimeScale::Ut1, &[Body::Sun], Frame::CANONICAL);
/// let sun = TestProvider::new().positions(&request)?.at(0, 0).unwrap();
/// assert_eq!(sun.source.kind, EphemerisKind::Test);
/// # Ok::<(), teistro_port_ephemeris::ProviderError>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EphemerisKind {
    /// Compressed ephemeris files.
    Files,
    /// A JPL binary ephemeris.
    Jpl,
    /// An analytic model.
    Analytic,
    /// A test table, no astronomy.
    Test,
    /// The provider did not say.
    Unknown,
}

impl EphemerisKind {
    /// The key stamped in provenance.
    ///
    /// ```
    /// use teistro_port_ephemeris::EphemerisKind;
    ///
    /// assert_eq!(EphemerisKind::Jpl.key(), "JPL");
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            EphemerisKind::Files => "FILES",
            EphemerisKind::Jpl => "JPL",
            EphemerisKind::Analytic => "ANALYTIC",
            EphemerisKind::Test => "TEST",
            EphemerisKind::Unknown => "UNKNOWN",
        }
    }
}

/// Where a cell came from.
///
/// ```
/// use teistro_core::settings::Tier;
/// use teistro_port_ephemeris::{EphemerisKind, Source};
///
/// let source = Source { kind: EphemerisKind::Files, tier: Some(Tier::Standard) };
/// assert_eq!(Source::from_bits(source.to_bits()), source);
/// assert_eq!(Source::UNKNOWN.kind, EphemerisKind::Unknown);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Source {
    /// The ephemeris kind.
    pub kind: EphemerisKind,
    /// The tier, when the provider has tiers.
    pub tier: Option<Tier>,
}

impl Source {
    /// A source the provider did not describe.
    pub const UNKNOWN: Source = Source {
        kind: EphemerisKind::Unknown,
        tier: None,
    };

    /// Packed for the C boundary: the kind in the low byte, the tier plus
    /// one in the next.
    ///
    /// ```
    /// use teistro_core::settings::Tier;
    /// use teistro_port_ephemeris::{EphemerisKind, Source};
    ///
    /// // The kind in the low byte, the tier plus one in the next.
    /// assert_eq!(Source { kind: EphemerisKind::Files, tier: Some(Tier::Standard) }.to_bits(), 0x0201);
    /// assert_eq!(Source { kind: EphemerisKind::Test, tier: None }.to_bits(), 4);
    /// ```
    #[must_use]
    pub const fn to_bits(self) -> u32 {
        let kind = match self.kind {
            EphemerisKind::Files => 1,
            EphemerisKind::Jpl => 2,
            EphemerisKind::Analytic => 3,
            EphemerisKind::Test => 4,
            EphemerisKind::Unknown => 0,
        };
        kind | (tier_bits(self.tier) << 8)
    }

    /// Unpacked from the C boundary.
    ///
    /// ```
    /// use teistro_port_ephemeris::{EphemerisKind, Source};
    ///
    /// // An unknown kind byte reads as unknown rather than failing.
    /// assert_eq!(Source::from_bits(0xff).kind, EphemerisKind::Unknown);
    /// ```
    #[must_use]
    pub const fn from_bits(bits: u32) -> Source {
        let kind = match bits & 0xff {
            1 => EphemerisKind::Files,
            2 => EphemerisKind::Jpl,
            3 => EphemerisKind::Analytic,
            4 => EphemerisKind::Test,
            _ => EphemerisKind::Unknown,
        };
        Source {
            kind,
            tier: tier_from_bits((bits >> 8) & 0xff),
        }
    }
}

/// A tier as the byte the C boundary carries: zero for none.
///
/// ```
/// use teistro_core::settings::Tier;
/// use teistro_port_ephemeris::columns::tier_bits;
///
/// assert_eq!(tier_bits(None), 0);
/// assert_eq!(tier_bits(Some(Tier::Full)), 3);
/// ```
#[must_use]
pub const fn tier_bits(tier: Option<Tier>) -> u32 {
    match tier {
        Some(Tier::Compact) => 1,
        Some(Tier::Standard) => 2,
        Some(Tier::Full) => 3,
        Some(Tier::Reference) => 4,
        // None, and a tier core adds before this crate learns it, which
        // has no byte yet.
        _ => 0,
    }
}

/// The tier a byte of the C boundary names.
///
/// ```
/// use teistro_core::settings::Tier;
/// use teistro_port_ephemeris::columns::{tier_bits, tier_from_bits};
///
/// assert_eq!(tier_from_bits(tier_bits(Some(Tier::Reference))), Some(Tier::Reference));
/// assert_eq!(tier_from_bits(9), None);
/// ```
#[must_use]
pub const fn tier_from_bits(bits: u32) -> Option<Tier> {
    match bits {
        1 => Some(Tier::Compact),
        2 => Some(Tier::Standard),
        3 => Some(Tier::Full),
        4 => Some(Tier::Reference),
        _ => None,
    }
}

/// One cell of a response.
///
/// ```
/// use teistro_port_ephemeris::{Body, EphemerisProvider, Frame, PositionRequest, TestProvider, TimeScale};
///
/// let request = PositionRequest::new(&[2_451_545.0], TimeScale::Ut1, &[Body::Sun], Frame::CANONICAL);
/// let sun = TestProvider::new().positions(&request)?.at(0, 0).unwrap();
/// // A degree a day, more or less.
/// assert!(sun.is_ok() && (0.95..1.05).contains(&sun.lon_speed));
/// assert!((0.0..360.0).contains(&sun.lon));
/// # Ok::<(), teistro_port_ephemeris::ProviderError>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Cell {
    /// Longitude or right ascension, degrees.
    pub lon: f64,
    /// Latitude or declination, degrees.
    pub lat: f64,
    /// Distance, astronomical units.
    pub dist: f64,
    /// Longitude speed, degrees per day.
    pub lon_speed: f64,
    /// Latitude speed, degrees per day.
    pub lat_speed: f64,
    /// Distance speed, astronomical units per day.
    pub dist_speed: f64,
    /// The cell's status.
    pub status: CellStatus,
    /// Where it came from.
    pub source: Source,
}

impl Cell {
    /// An untouched cell.
    pub const EMPTY: Cell = Cell {
        lon: 0.0,
        lat: 0.0,
        dist: 0.0,
        lon_speed: 0.0,
        lat_speed: 0.0,
        dist_speed: 0.0,
        status: CellStatus::NotComputed,
        source: Source::UNKNOWN,
    };

    /// A cell that failed with a status, its values zero.
    ///
    /// ```
    /// use teistro_port_ephemeris::{Cell, CellStatus};
    ///
    /// let missing = Cell::failed(CellStatus::DataMissing);
    /// assert!(!missing.is_ok());
    /// assert_eq!((missing.lon, missing.status), (0.0, CellStatus::DataMissing));
    /// ```
    #[must_use]
    pub const fn failed(status: CellStatus) -> Cell {
        Cell {
            status,
            ..Cell::EMPTY
        }
    }

    /// Whether the cell was computed.
    ///
    /// ```
    /// use teistro_port_ephemeris::{Cell, CellStatus};
    ///
    /// assert!(Cell { status: CellStatus::Ok, ..Cell::EMPTY }.is_ok());
    /// assert!(!Cell::EMPTY.is_ok());
    /// ```
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.status == CellStatus::Ok
    }
}

/// The one required operation's output: columns over a grid, instants
/// outermost (`index = jd_index × body_count + body_index`).
///
/// ```
/// use teistro_port_ephemeris::{Cell, CellStatus, Frame, PositionColumns};
///
/// let mut columns = PositionColumns::new(2, 3, Frame::CANONICAL);
/// assert_eq!(columns.index(1, 2), Some(5));
/// columns.set(5, Cell { lon: 1.5, status: CellStatus::Ok, ..Cell::EMPTY });
/// assert_eq!(columns.at(1, 2).map(|c| c.lon), Some(1.5));
/// assert!(!columns.all_ok());
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PositionColumns {
    /// The number of instants.
    pub jd_count: usize,
    /// The number of bodies.
    pub body_count: usize,
    /// The frame the values are in.
    pub frame: Frame,
    /// Longitudes or right ascensions, degrees.
    pub lon: Vec<f64>,
    /// Latitudes or declinations, degrees.
    pub lat: Vec<f64>,
    /// Distances, astronomical units.
    pub dist: Vec<f64>,
    /// Longitude speeds, degrees per day.
    pub lon_speed: Vec<f64>,
    /// Latitude speeds, degrees per day.
    pub lat_speed: Vec<f64>,
    /// Distance speeds, astronomical units per day.
    pub dist_speed: Vec<f64>,
    /// Per-cell status.
    pub status: Vec<CellStatus>,
    /// Per-cell source.
    pub source: Vec<Source>,
}

impl PositionColumns {
    /// Empty columns of a grid, every cell [`Cell::EMPTY`].
    ///
    /// ```
    /// use teistro_port_ephemeris::{CellStatus, Frame, PositionColumns};
    ///
    /// let columns = PositionColumns::new(2, 3, Frame::CANONICAL);
    /// assert_eq!(columns.len(), 6);
    /// assert!(columns.status.iter().all(|status| *status == CellStatus::NotComputed));
    /// ```
    #[must_use]
    pub fn new(jd_count: usize, body_count: usize, frame: Frame) -> PositionColumns {
        let n = jd_count.saturating_mul(body_count);
        PositionColumns {
            jd_count,
            body_count,
            frame,
            lon: vec![0.0; n],
            lat: vec![0.0; n],
            dist: vec![0.0; n],
            lon_speed: vec![0.0; n],
            lat_speed: vec![0.0; n],
            dist_speed: vec![0.0; n],
            status: vec![CellStatus::NotComputed; n],
            source: vec![Source::UNKNOWN; n],
        }
    }

    /// The number of cells.
    ///
    /// ```
    /// use teistro_port_ephemeris::{Frame, PositionColumns};
    ///
    /// assert_eq!(PositionColumns::new(4, 2, Frame::CANONICAL).len(), 8);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.lon.len()
    }

    /// Whether the grid is empty.
    ///
    /// ```
    /// use teistro_port_ephemeris::{Frame, PositionColumns};
    ///
    /// assert!(PositionColumns::new(0, 2, Frame::CANONICAL).is_empty());
    /// assert!(!PositionColumns::new(1, 1, Frame::CANONICAL).is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.lon.is_empty()
    }

    /// The flat index of a cell.
    ///
    /// ```
    /// use teistro_port_ephemeris::{Frame, PositionColumns};
    ///
    /// let columns = PositionColumns::new(2, 3, Frame::CANONICAL);
    /// // Instants outermost.
    /// assert_eq!(columns.index(1, 0), Some(3));
    /// assert_eq!(columns.index(2, 0), None);
    /// assert_eq!(columns.index(0, 3), None);
    /// ```
    #[must_use]
    pub fn index(&self, jd_index: usize, body_index: usize) -> Option<usize> {
        (jd_index < self.jd_count && body_index < self.body_count)
            .then(|| jd_index * self.body_count + body_index)
    }

    /// Copies one instant's row of every column from `from`, whose bodies
    /// are this grid's bodies in the same order. A row outside either grid
    /// copies nothing.
    pub(crate) fn copy_row(&mut self, row: usize, from: &PositionColumns, from_row: usize) {
        /// One column's row, when both grids hold it.
        fn copy<T: Copy>(into: &mut [T], out_of: &[T], width: usize, row: usize, from_row: usize) {
            let target = into.chunks_exact_mut(width).nth(row);
            if let (Some(target), Some(source)) = (target, out_of.chunks_exact(width).nth(from_row))
            {
                target.copy_from_slice(source);
            }
        }
        debug_assert_eq!(self.body_count, from.body_count, "rows of different bodies");
        let width = self.body_count;
        if width == 0 || width != from.body_count {
            return;
        }
        for (into, out_of) in [
            (&mut self.lon, &from.lon),
            (&mut self.lat, &from.lat),
            (&mut self.dist, &from.dist),
            (&mut self.lon_speed, &from.lon_speed),
            (&mut self.lat_speed, &from.lat_speed),
            (&mut self.dist_speed, &from.dist_speed),
        ] {
            copy(into, out_of, width, row, from_row);
        }
        copy(&mut self.status, &from.status, width, row, from_row);
        copy(&mut self.source, &from.source, width, row, from_row);
    }

    /// One cell by flat index.
    ///
    /// ```
    /// use teistro_port_ephemeris::{Cell, Frame, PositionColumns};
    ///
    /// let mut columns = PositionColumns::new(1, 2, Frame::CANONICAL);
    /// columns.set(1, Cell { lat: -0.5, ..Cell::EMPTY });
    /// assert_eq!(columns.cell(1).map(|cell| cell.lat), Some(-0.5));
    /// assert_eq!(columns.cell(2), None);
    /// ```
    #[must_use]
    pub fn cell(&self, index: usize) -> Option<Cell> {
        Some(Cell {
            lon: *self.lon.get(index)?,
            lat: *self.lat.get(index)?,
            dist: *self.dist.get(index)?,
            lon_speed: *self.lon_speed.get(index)?,
            lat_speed: *self.lat_speed.get(index)?,
            dist_speed: *self.dist_speed.get(index)?,
            status: *self.status.get(index)?,
            source: *self.source.get(index)?,
        })
    }

    /// One cell by grid position.
    ///
    /// ```
    /// use teistro_port_ephemeris::{Cell, Frame, PositionColumns};
    ///
    /// let mut columns = PositionColumns::new(2, 2, Frame::CANONICAL);
    /// columns.set(3, Cell { dist: 2.5, ..Cell::EMPTY });
    /// assert_eq!(columns.at(1, 1).map(|cell| cell.dist), Some(2.5));
    /// ```
    #[must_use]
    pub fn at(&self, jd_index: usize, body_index: usize) -> Option<Cell> {
        self.cell(self.index(jd_index, body_index)?)
    }

    /// Writes one cell by flat index; an index out of range writes nothing.
    ///
    /// ```
    /// use teistro_port_ephemeris::{Cell, Frame, PositionColumns};
    ///
    /// let mut columns = PositionColumns::new(1, 1, Frame::CANONICAL);
    /// // Out of range writes nothing.
    /// columns.set(5, Cell { lon: 1.0, ..Cell::EMPTY });
    /// assert_eq!(columns.cell(0), Some(Cell::EMPTY));
    /// ```
    pub fn set(&mut self, index: usize, cell: Cell) {
        if index >= self.len() {
            return;
        }
        for (column, value) in [
            (&mut self.lon, cell.lon),
            (&mut self.lat, cell.lat),
            (&mut self.dist, cell.dist),
            (&mut self.lon_speed, cell.lon_speed),
            (&mut self.lat_speed, cell.lat_speed),
            (&mut self.dist_speed, cell.dist_speed),
        ] {
            if let Some(slot) = column.get_mut(index) {
                *slot = value;
            }
        }
        if let Some(slot) = self.status.get_mut(index) {
            *slot = cell.status;
        }
        if let Some(slot) = self.source.get_mut(index) {
            *slot = cell.source;
        }
    }

    /// Writes one cell by grid position; a position outside the grid
    /// writes nothing.
    ///
    /// ```
    /// use teistro_port_ephemeris::{Cell, CellStatus, Frame, PositionColumns};
    ///
    /// let mut columns = PositionColumns::new(2, 2, Frame::CANONICAL);
    /// columns.set_at(1, 0, Cell::failed(CellStatus::OutOfRange));
    /// assert_eq!(columns.status[2], CellStatus::OutOfRange);
    /// ```
    pub fn set_at(&mut self, jd_index: usize, body_index: usize, cell: Cell) {
        if let Some(index) = self.index(jd_index, body_index) {
            self.set(index, cell);
        }
    }

    /// Every cell in flat order.
    ///
    /// ```
    /// use teistro_port_ephemeris::{Body, EphemerisProvider, Frame, PositionRequest, TestProvider, TimeScale};
    ///
    /// let request = PositionRequest::new(&[2_451_545.0, 2_451_546.0], TimeScale::Ut1, &[Body::Sun, Body::Moon], Frame::CANONICAL);
    /// let columns = TestProvider::new().positions(&request)?;
    /// // The Moon outruns the Sun in every row.
    /// let speeds: Vec<f64> = columns.cells().map(|cell| cell.lon_speed).collect();
    /// assert!(speeds.chunks(2).all(|row| row[1] > row[0]));
    /// # Ok::<(), teistro_port_ephemeris::ProviderError>(())
    /// ```
    pub fn cells(&self) -> impl Iterator<Item = Cell> + '_ {
        (0..self.len()).filter_map(|i| self.cell(i))
    }

    /// The instants of `later` added after this grid's, as one answer to
    /// the instants of both requests in order: how a request split into
    /// parts is joined again. Grids of other bodies or another frame are
    /// not joined, and the grid is left as it was.
    ///
    /// ```
    /// use teistro_port_ephemeris::{Cell, Frame, PositionColumns};
    ///
    /// let mut first = PositionColumns::new(1, 2, Frame::CANONICAL);
    /// let mut later = PositionColumns::new(2, 2, Frame::CANONICAL);
    /// later.set_at(1, 1, Cell { lon: 7.0, ..Cell::EMPTY });
    /// assert!(first.append(later));
    /// assert_eq!(first.jd_count, 3);
    /// assert_eq!(first.at(2, 1).map(|cell| cell.lon), Some(7.0));
    /// ```
    pub fn append(&mut self, later: PositionColumns) -> bool {
        if later.body_count != self.body_count || later.frame != self.frame {
            return false;
        }
        self.jd_count += later.jd_count;
        self.lon.extend(later.lon);
        self.lat.extend(later.lat);
        self.dist.extend(later.dist);
        self.lon_speed.extend(later.lon_speed);
        self.lat_speed.extend(later.lat_speed);
        self.dist_speed.extend(later.dist_speed);
        self.status.extend(later.status);
        self.source.extend(later.source);
        true
    }

    /// Whether every cell is [`CellStatus::Ok`].
    ///
    /// ```
    /// use teistro_port_ephemeris::{Body, EphemerisProvider, Frame, PositionRequest, TestProvider, TimeScale};
    ///
    /// let request = PositionRequest::new(&[2_451_545.0], TimeScale::Ut1, &[Body::Sun, Body::Moon], Frame::CANONICAL);
    /// assert!(TestProvider::new().positions(&request)?.all_ok());
    /// # Ok::<(), teistro_port_ephemeris::ProviderError>(())
    /// ```
    #[must_use]
    pub fn all_ok(&self) -> bool {
        self.status.iter().all(|s| *s == CellStatus::Ok)
    }

    /// Whether two responses are bit-identical in every numeric column
    /// and equal in every status.
    ///
    /// ```
    /// use teistro_port_ephemeris::{Body, EphemerisProvider, Frame, PositionRequest, TestProvider, TimeScale};
    ///
    /// // The test provider is deterministic: the same request, the same bits.
    /// let request = PositionRequest::new(&[2_451_545.0], TimeScale::Ut1, &[Body::Moon], Frame::CANONICAL);
    /// let (first, second) = (TestProvider::new().positions(&request)?, TestProvider::new().positions(&request)?);
    /// assert!(first.bit_identical(&second));
    /// # Ok::<(), teistro_port_ephemeris::ProviderError>(())
    /// ```
    #[must_use]
    pub fn bit_identical(&self, other: &PositionColumns) -> bool {
        let same = |a: &[f64], b: &[f64]| {
            a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits())
        };
        same(&self.lon, &other.lon)
            && same(&self.lat, &other.lat)
            && same(&self.dist, &other.dist)
            && same(&self.lon_speed, &other.lon_speed)
            && same(&self.lat_speed, &other.lat_speed)
            && same(&self.dist_speed, &other.dist_speed)
            && self.status == other.status
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    #[test]
    fn statuses_and_sources_round_trip() {
        for status in [
            CellStatus::Ok,
            CellStatus::NotComputed,
            CellStatus::UnsupportedBody,
            CellStatus::OutOfRange,
            CellStatus::DataMissing,
            CellStatus::Provider { code: -107 },
        ] {
            assert_eq!(CellStatus::from_code(status.code()), status);
            // The key is the serialised spelling, so a program matching on
            // either reads the same word.
            let serialised = serde_json::to_value(status).unwrap();
            assert_eq!(
                serialised.get("kind"),
                Some(&serde_json::json!(status.key()))
            );
        }
        let source = Source {
            kind: EphemerisKind::Files,
            tier: Some(Tier::Standard),
        };
        assert_eq!(Source::from_bits(source.to_bits()), source);
        assert_eq!(Source::from_bits(0), Source::UNKNOWN);
        assert_eq!(
            tier_from_bits(tier_bits(Some(Tier::Reference))),
            Some(Tier::Reference)
        );
    }

    #[test]
    fn columns_index_instants_outermost() {
        let mut columns = PositionColumns::new(2, 3, Frame::CANONICAL);
        assert_eq!(columns.len(), 6);
        assert!(!columns.is_empty());
        assert_eq!(columns.index(1, 2), Some(5));
        assert_eq!(columns.index(2, 0), None);
        columns.set_at(
            1,
            2,
            Cell {
                lon: 1.5,
                status: CellStatus::Ok,
                ..Cell::EMPTY
            },
        );
        columns.set(99, Cell::failed(CellStatus::OutOfRange));
        assert_eq!(columns.at(1, 2).map(|c| c.lon), Some(1.5));
        assert!(columns.at(1, 2).unwrap().is_ok());
        assert!(!columns.all_ok());
        assert_eq!(columns.cells().count(), 6);
        assert!(columns.bit_identical(&columns.clone()));
        let mut other = columns.clone();
        other.set(0, Cell::failed(CellStatus::DataMissing));
        assert!(!columns.bit_identical(&other));
    }
}
