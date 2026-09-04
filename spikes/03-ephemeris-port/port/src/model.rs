//! The port's vocabulary: bodies, frames, requests, columnar responses,
//! capabilities and errors. Every type serialises, so the conformance
//! report and the result envelope can quote it verbatim.

use core::fmt;
use core::fmt::Write as _;
use std::io::Read;
use std::path::Path;

use serde::Serialize;
use sha2::{Digest, Sha256};

/// The bodies the port knows. Adapters map them to their own numbering;
/// a body an adapter cannot compute is reported per cell, never guessed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum Body {
    /// The Sun.
    Sun,
    /// The Moon.
    Moon,
    /// Mercury.
    Mercury,
    /// Venus.
    Venus,
    /// Mars.
    Mars,
    /// Jupiter.
    Jupiter,
    /// Saturn.
    Saturn,
    /// Uranus.
    Uranus,
    /// Neptune.
    Neptune,
    /// Pluto.
    Pluto,
    /// The mean ascending lunar node.
    MeanNode,
    /// The true (osculating) ascending lunar node.
    TrueNode,
    /// The mean lunar apogee.
    MeanApogee,
    /// The osculating lunar apogee.
    OsculatingApogee,
}

impl Body {
    /// Every body, in stable id order.
    pub const ALL: [Body; 14] = [
        Body::Sun,
        Body::Moon,
        Body::Mercury,
        Body::Venus,
        Body::Mars,
        Body::Jupiter,
        Body::Saturn,
        Body::Uranus,
        Body::Neptune,
        Body::Pluto,
        Body::MeanNode,
        Body::TrueNode,
        Body::MeanApogee,
        Body::OsculatingApogee,
    ];

    /// The stable numeric id used at the C boundary.
    #[must_use]
    pub fn id(self) -> u16 {
        Body::ALL
            .iter()
            .position(|b| *b == self)
            .and_then(|i| u16::try_from(i).ok())
            .unwrap_or(u16::MAX)
    }

    /// The body with a given id.
    #[must_use]
    pub fn from_id(id: u16) -> Option<Body> {
        Body::ALL.get(usize::from(id)).copied()
    }

    /// The upper-case key used in fixtures and bindings.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Body::Sun => "SUN",
            Body::Moon => "MOON",
            Body::Mercury => "MERCURY",
            Body::Venus => "VENUS",
            Body::Mars => "MARS",
            Body::Jupiter => "JUPITER",
            Body::Saturn => "SATURN",
            Body::Uranus => "URANUS",
            Body::Neptune => "NEPTUNE",
            Body::Pluto => "PLUTO",
            Body::MeanNode => "MEAN_NODE",
            Body::TrueNode => "TRUE_NODE",
            Body::MeanApogee => "MEAN_APOGEE",
            Body::OsculatingApogee => "OSCULATING_APOGEE",
        }
    }

    /// Whether the body has a physical distance; nodes and apogees are
    /// directions, and a provider may report their distance as zero.
    #[must_use]
    pub const fn has_distance(self) -> bool {
        !matches!(
            self,
            Body::MeanNode | Body::TrueNode | Body::MeanApogee | Body::OsculatingApogee
        )
    }
}

/// The time scale of the instants in a request.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeScale {
    /// Universal Time (UT1).
    Ut1,
    /// Terrestrial Time.
    Tt,
}

impl TimeScale {
    /// The stable id at the C boundary.
    #[must_use]
    pub const fn id(self) -> u32 {
        match self {
            TimeScale::Ut1 => 0,
            TimeScale::Tt => 1,
        }
    }

    /// The scale with a given id.
    #[must_use]
    pub const fn from_id(id: u32) -> Option<TimeScale> {
        match id {
            0 => Some(TimeScale::Ut1),
            1 => Some(TimeScale::Tt),
            _ => None,
        }
    }
}

/// An observer on the Earth, for topocentric positions.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Observer {
    /// Degrees, east positive.
    pub longitude_deg: f64,
    /// Degrees, north positive.
    pub latitude_deg: f64,
    /// Metres above sea level.
    pub altitude_m: f64,
}

/// Where a position is seen from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Center {
    /// The centre of the Earth.
    Geocentric,
    /// An observer on the Earth; the request carries the observer.
    Topocentric,
    /// The centre of the Sun.
    Heliocentric,
    /// The solar-system barycentre.
    Barycentric,
}

/// The equinox and equator the coordinates refer to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Equinox {
    /// The equinox of date.
    OfDate,
    /// The J2000.0 equinox.
    J2000,
}

/// The coordinate system of a position.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Coordinates {
    /// Ecliptic longitude and latitude.
    Ecliptic,
    /// Right ascension and declination.
    Equatorial,
}

/// An ayanamsha of the shared catalogue numbering (Lahiri is 1, Raman 3,
/// Krishnamurti 5, True Chitra 27), as Teimeris and Swiss both count.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct AyanamshaId(pub u8);

impl AyanamshaId {
    /// Lahiri (Chitrapaksha).
    pub const LAHIRI: AyanamshaId = AyanamshaId(1);
    /// B. V. Raman.
    pub const RAMAN: AyanamshaId = AyanamshaId(3);
    /// K. S. Krishnamurti.
    pub const KRISHNAMURTI: AyanamshaId = AyanamshaId(5);
    /// True Chitra, Spica at 180°.
    pub const TRUE_CHITRA: AyanamshaId = AyanamshaId(27);
}

/// Which zodiac longitudes are measured in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Zodiac {
    /// From the equinox.
    Tropical,
    /// From the equinox less an ayanamsha.
    Sidereal(AyanamshaId),
}

/// Which corrections a position includes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "four independent switches of a frame"
)]
pub struct Corrections {
    /// Light-time correction applied.
    pub light_time: bool,
    /// Annual aberration applied.
    pub aberration: bool,
    /// Gravitational deflection applied.
    pub deflection: bool,
    /// Nutation applied (true equinox rather than mean).
    pub nutation: bool,
}

impl Corrections {
    /// Every correction: an apparent position.
    pub const APPARENT: Corrections = Corrections {
        light_time: true,
        aberration: true,
        deflection: true,
        nutation: true,
    };
    /// No correction: a geometric position.
    pub const GEOMETRIC: Corrections = Corrections {
        light_time: false,
        aberration: false,
        deflection: false,
        nutation: false,
    };
}

/// A position frame: the answer to "what do these numbers mean".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Frame {
    /// Where the position is seen from.
    pub center: Center,
    /// Which equinox.
    pub equinox: Equinox,
    /// Which coordinates.
    pub coordinates: Coordinates,
    /// Which zodiac.
    pub zodiac: Zodiac,
    /// Which corrections.
    pub corrections: Corrections,
}

impl Frame {
    /// The SDK's canonical frame: apparent geocentric ecliptic of date,
    /// tropical.
    pub const CANONICAL: Frame = Frame {
        center: Center::Geocentric,
        equinox: Equinox::OfDate,
        coordinates: Coordinates::Ecliptic,
        zodiac: Zodiac::Tropical,
        corrections: Corrections::APPARENT,
    };

    /// The same frame with other coordinates.
    #[must_use]
    pub const fn with_coordinates(self, coordinates: Coordinates) -> Frame {
        Frame {
            coordinates,
            ..self
        }
    }

    /// The same frame in another zodiac.
    #[must_use]
    pub const fn with_zodiac(self, zodiac: Zodiac) -> Frame {
        Frame { zodiac, ..self }
    }

    /// The frame packed for the C boundary: bits 0 and 1 the centre, bit 2
    /// the equinox, bit 3 the coordinates, bits 4 to 7 the corrections,
    /// bit 8 sidereal, bits 16 to 23 the ayanamsha id.
    #[must_use]
    pub fn to_bits(self) -> u32 {
        let center = match self.center {
            Center::Geocentric => 0,
            Center::Topocentric => 1,
            Center::Heliocentric => 2,
            Center::Barycentric => 3,
        };
        let mut bits = center;
        if self.equinox == Equinox::J2000 {
            bits |= 1 << 2;
        }
        if self.coordinates == Coordinates::Equatorial {
            bits |= 1 << 3;
        }
        let c = self.corrections;
        bits |= u32::from(c.light_time) << 4;
        bits |= u32::from(c.aberration) << 5;
        bits |= u32::from(c.deflection) << 6;
        bits |= u32::from(c.nutation) << 7;
        if let Zodiac::Sidereal(AyanamshaId(id)) = self.zodiac {
            bits |= 1 << 8;
            bits |= u32::from(id) << 16;
        }
        bits
    }

    /// The frame from its packed form.
    #[must_use]
    pub fn from_bits(bits: u32) -> Frame {
        let center = match bits & 0b11 {
            1 => Center::Topocentric,
            2 => Center::Heliocentric,
            3 => Center::Barycentric,
            _ => Center::Geocentric,
        };
        let flag = |bit: u32| bits & (1 << bit) != 0;
        let zodiac = if flag(8) {
            Zodiac::Sidereal(AyanamshaId(u8::try_from((bits >> 16) & 0xff).unwrap_or(0)))
        } else {
            Zodiac::Tropical
        };
        Frame {
            center,
            equinox: if flag(2) {
                Equinox::J2000
            } else {
                Equinox::OfDate
            },
            coordinates: if flag(3) {
                Coordinates::Equatorial
            } else {
                Coordinates::Ecliptic
            },
            zodiac,
            corrections: Corrections {
                light_time: flag(4),
                aberration: flag(5),
                deflection: flag(6),
                nutation: flag(7),
            },
        }
    }
}

/// The one required operation's input: a grid of instants and bodies in a
/// requested frame.
#[derive(Clone, Copy, Debug)]
pub struct PositionRequest<'a> {
    /// The instants.
    pub jds: &'a [f64],
    /// Their time scale.
    pub scale: TimeScale,
    /// The bodies.
    pub bodies: &'a [Body],
    /// The frame the caller wants.
    pub frame: Frame,
    /// The observer, required when the frame is topocentric.
    pub observer: Option<Observer>,
    /// Whether speeds are wanted.
    pub speeds: bool,
}

impl PositionRequest<'_> {
    /// The number of cells: instants times bodies.
    #[must_use]
    pub fn cell_count(&self) -> usize {
        self.jds.len().saturating_mul(self.bodies.len())
    }
}

/// The status of one cell of a response; a failing cell never aborts the
/// batch.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
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
    /// The provider's own code.
    Provider(i32),
}

impl CellStatus {
    /// The stable code at the C boundary: `0` ok, negatives as
    /// [`ProviderError::code`], positives the provider's own.
    #[must_use]
    pub const fn code(self) -> i32 {
        match self {
            CellStatus::Ok => 0,
            CellStatus::NotComputed => -6,
            CellStatus::UnsupportedBody => -1,
            CellStatus::OutOfRange => -2,
            CellStatus::DataMissing => -3,
            CellStatus::Provider(code) => code,
        }
    }

    /// The status from its code.
    #[must_use]
    pub const fn from_code(code: i32) -> CellStatus {
        match code {
            0 => CellStatus::Ok,
            -6 => CellStatus::NotComputed,
            -1 => CellStatus::UnsupportedBody,
            -2 => CellStatus::OutOfRange,
            -3 => CellStatus::DataMissing,
            other => CellStatus::Provider(other),
        }
    }
}

/// Which ephemeris produced a cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
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

/// A built-in tier, stamped when the provider has tiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    /// About an arcminute.
    Compact,
    /// A few arcseconds.
    Standard,
    /// The theories' accuracy.
    Full,
    /// A DE refit.
    Reference,
}

/// Where a cell came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Source {
    /// The ephemeris kind.
    pub kind: EphemerisKind,
    /// The tier, when the provider has tiers.
    pub tier: Option<Tier>,
}

impl Source {
    /// Packed for the C boundary: the kind in the low byte, the tier plus
    /// one in the next.
    #[must_use]
    pub const fn to_bits(self) -> u32 {
        let kind = match self.kind {
            EphemerisKind::Files => 1,
            EphemerisKind::Jpl => 2,
            EphemerisKind::Analytic => 3,
            EphemerisKind::Test => 4,
            EphemerisKind::Unknown => 0,
        };
        let tier = match self.tier {
            None => 0,
            Some(Tier::Compact) => 1,
            Some(Tier::Standard) => 2,
            Some(Tier::Full) => 3,
            Some(Tier::Reference) => 4,
        };
        kind | (tier << 8)
    }

    /// Unpacked from the C boundary.
    #[must_use]
    pub const fn from_bits(bits: u32) -> Source {
        let kind = match bits & 0xff {
            1 => EphemerisKind::Files,
            2 => EphemerisKind::Jpl,
            3 => EphemerisKind::Analytic,
            4 => EphemerisKind::Test,
            _ => EphemerisKind::Unknown,
        };
        let tier = match (bits >> 8) & 0xff {
            1 => Some(Tier::Compact),
            2 => Some(Tier::Standard),
            3 => Some(Tier::Full),
            4 => Some(Tier::Reference),
            _ => None,
        };
        Source { kind, tier }
    }
}

/// One cell of a response.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Cell {
    /// Longitude or right ascension, degrees.
    pub lon: f64,
    /// Latitude or declination, degrees.
    pub lat: f64,
    /// Distance, AU.
    pub dist: f64,
    /// Longitude speed, degrees per day.
    pub lon_speed: f64,
    /// Latitude speed, degrees per day.
    pub lat_speed: f64,
    /// Distance speed, AU per day.
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
        source: Source {
            kind: EphemerisKind::Unknown,
            tier: None,
        },
    };
}

/// The one required operation's output: columns over a grid, instants
/// outermost (`index = jd_index × body_count + body_index`).
#[derive(Clone, Debug, PartialEq, Serialize)]
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
    /// Distances, AU.
    pub dist: Vec<f64>,
    /// Longitude speeds, degrees per day.
    pub lon_speed: Vec<f64>,
    /// Latitude speeds, degrees per day.
    pub lat_speed: Vec<f64>,
    /// Distance speeds, AU per day.
    pub dist_speed: Vec<f64>,
    /// Per-cell status.
    pub status: Vec<CellStatus>,
    /// Per-cell source.
    pub source: Vec<Source>,
}

impl PositionColumns {
    /// Empty columns of a grid, every cell [`Cell::EMPTY`].
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
            source: vec![Cell::EMPTY.source; n],
        }
    }

    /// The number of cells.
    #[must_use]
    pub fn len(&self) -> usize {
        self.lon.len()
    }

    /// Whether the grid is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.lon.is_empty()
    }

    /// The flat index of a cell.
    #[must_use]
    pub fn index(&self, jd_index: usize, body_index: usize) -> Option<usize> {
        (jd_index < self.jd_count && body_index < self.body_count)
            .then(|| jd_index * self.body_count + body_index)
    }

    /// One cell by flat index.
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
    #[must_use]
    pub fn at(&self, jd_index: usize, body_index: usize) -> Option<Cell> {
        self.cell(self.index(jd_index, body_index)?)
    }

    /// Writes one cell by flat index; out of range is ignored.
    pub fn set(&mut self, index: usize, cell: Cell) {
        let Some(slot) = self.lon.get_mut(index) else {
            return;
        };
        *slot = cell.lon;
        if let Some(v) = self.lat.get_mut(index) {
            *v = cell.lat;
        }
        if let Some(v) = self.dist.get_mut(index) {
            *v = cell.dist;
        }
        if let Some(v) = self.lon_speed.get_mut(index) {
            *v = cell.lon_speed;
        }
        if let Some(v) = self.lat_speed.get_mut(index) {
            *v = cell.lat_speed;
        }
        if let Some(v) = self.dist_speed.get_mut(index) {
            *v = cell.dist_speed;
        }
        if let Some(v) = self.status.get_mut(index) {
            *v = cell.status;
        }
        if let Some(v) = self.source.get_mut(index) {
            *v = cell.source;
        }
    }

    /// Every cell in flat order.
    pub fn cells(&self) -> impl Iterator<Item = Cell> + '_ {
        (0..self.len()).filter_map(|i| self.cell(i))
    }

    /// Whether every cell is [`CellStatus::Ok`].
    #[must_use]
    pub fn all_ok(&self) -> bool {
        self.status.iter().all(|s| *s == CellStatus::Ok)
    }

    /// Whether two responses are bit-identical in every numeric column.
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

/// A native operation a provider may implement instead of the SDK.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Overrides(u32);

impl Overrides {
    /// Nothing native.
    pub const NONE: Overrides = Overrides(0);
    /// The obliquity and nutation.
    pub const OBLIQUITY: Overrides = Overrides(1);
    /// Delta T.
    pub const DELTA_T: Overrides = Overrides(1 << 1);
    /// Sidereal time.
    pub const SIDEREAL_TIME: Overrides = Overrides(1 << 2);
    /// The ayanamsha value.
    pub const AYANAMSHA: Overrides = Overrides(1 << 3);
    /// Topocentric positions.
    pub const TOPOCENTRIC: Overrides = Overrides(1 << 4);
    /// House cusps.
    pub const HOUSES: Overrides = Overrides(1 << 5);
    /// Rise, set and transit.
    pub const RISE_SET: Overrides = Overrides(1 << 6);
    /// Crossings.
    pub const CROSSINGS: Overrides = Overrides(1 << 7);
    /// Stations.
    pub const STATIONS: Overrides = Overrides(1 << 8);
    /// Eclipses.
    pub const ECLIPSES: Overrides = Overrides(1 << 9);
    /// Fixed stars.
    pub const STARS: Overrides = Overrides(1 << 10);

    const NAMES: [(Overrides, &'static str); 11] = [
        (Overrides::OBLIQUITY, "obliquity"),
        (Overrides::DELTA_T, "delta_t"),
        (Overrides::SIDEREAL_TIME, "sidereal_time"),
        (Overrides::AYANAMSHA, "ayanamsha"),
        (Overrides::TOPOCENTRIC, "topocentric"),
        (Overrides::HOUSES, "houses"),
        (Overrides::RISE_SET, "rise_set"),
        (Overrides::CROSSINGS, "crossings"),
        (Overrides::STATIONS, "stations"),
        (Overrides::ECLIPSES, "eclipses"),
        (Overrides::STARS, "stars"),
    ];

    /// The union.
    #[must_use]
    pub const fn with(self, other: Overrides) -> Overrides {
        Overrides(self.0 | other.0)
    }

    /// Whether every bit of `other` is set.
    #[must_use]
    pub const fn contains(self, other: Overrides) -> bool {
        self.0 & other.0 == other.0
    }

    /// The raw bits for the C boundary.
    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// From raw bits.
    #[must_use]
    pub const fn from_bits(bits: u32) -> Overrides {
        Overrides(bits)
    }

    /// The names of the set overrides.
    #[must_use]
    pub fn names(self) -> Vec<&'static str> {
        Overrides::NAMES
            .iter()
            .filter(|(flag, _)| self.contains(*flag))
            .map(|(_, name)| *name)
            .collect()
    }
}

impl Serialize for Overrides {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.names().serialize(serializer)
    }
}

impl fmt::Display for Overrides {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.names().join(", "))
    }
}

/// The content hash of a data file, part of the provenance envelope.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DataHash {
    /// The file name.
    pub file: String,
    /// SHA-256 as lower-case hex.
    pub sha256: String,
    /// The size in bytes.
    pub bytes: u64,
}

impl DataHash {
    /// Hashes a file, streaming.
    ///
    /// # Errors
    ///
    /// When the file cannot be read.
    pub fn of_file(path: &Path) -> std::io::Result<DataHash> {
        let mut file = std::fs::File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; 1 << 16];
        let mut bytes = 0u64;
        loop {
            let read = file.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            hasher.update(buffer.get(..read).unwrap_or_default());
            bytes += read as u64;
        }
        let digest = hasher.finalize();
        let sha256 = digest.iter().fold(String::with_capacity(64), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        });
        Ok(DataHash {
            file: path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
            sha256,
            bytes,
        })
    }
}

/// Who the provider is.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Identity {
    /// The provider's name.
    pub name: String,
    /// The provider's version.
    pub version: String,
    /// The data version, or what identifies the data.
    pub data_version: String,
    /// The tier, when the provider has tiers.
    pub tier: Option<Tier>,
    /// The hashes of the data files in use.
    pub data_hashes: Vec<DataHash>,
}

/// What a provider declares about itself.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Capabilities {
    /// Who.
    pub identity: Identity,
    /// Coverage in UT1 Julian Days, inclusive.
    pub jd_range: (f64, f64),
    /// The bodies it computes.
    pub bodies: Vec<Body>,
    /// The frame `positions` returns.
    pub native_frame: Frame,
    /// Whether it returns speeds.
    pub speeds: bool,
    /// The native overrides it offers.
    pub overrides: Overrides,
    /// The ayanamshas its override knows.
    pub ayanamshas: Vec<AyanamshaId>,
    /// Whether identical requests give identical bits.
    pub deterministic: bool,
}

impl Capabilities {
    /// Whether an instant is inside the coverage.
    #[must_use]
    pub fn covers(&self, jd: f64) -> bool {
        jd >= self.jd_range.0 && jd <= self.jd_range.1
    }

    /// Whether a body is offered.
    #[must_use]
    pub fn has_body(&self, body: Body) -> bool {
        self.bodies.contains(&body)
    }
}

/// The obliquity of the ecliptic and the nutation, degrees.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Obliquity {
    /// Mean obliquity.
    pub mean_deg: f64,
    /// True obliquity, mean plus the nutation in obliquity.
    pub true_deg: f64,
    /// Nutation in longitude.
    pub nutation_lon_deg: f64,
    /// Nutation in obliquity.
    pub nutation_obl_deg: f64,
}

/// Which implementation of a computation the SDK prefers (ADR-0013).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OverridePolicy {
    /// A declared native override is used; the SDK otherwise.
    PreferNative,
    /// The SDK always, for byte-identical results across providers.
    SdkOnly,
    /// A declared native override or a refusal, never the SDK.
    NativeOnly,
}

/// Why a provider could not answer.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProviderError {
    /// The operation or option is not implemented by this provider.
    Unsupported {
        /// What was asked for.
        what: String,
    },
    /// The instant is outside coverage.
    OutOfRange {
        /// The instant.
        jd: f64,
    },
    /// A data file is missing.
    DataMissing {
        /// Which.
        detail: String,
    },
    /// The provider refused rather than answer silently with something
    /// else (a fallback model, a stale setting).
    Refused {
        /// Why.
        detail: String,
    },
    /// The request is malformed.
    Invalid {
        /// What is wrong.
        detail: String,
    },
    /// The provider's own error.
    Provider {
        /// Its code.
        code: i32,
        /// Its message.
        detail: String,
    },
}

impl ProviderError {
    /// The stable code at the C boundary.
    #[must_use]
    pub fn code(&self) -> i32 {
        match self {
            ProviderError::Unsupported { .. } => -1,
            ProviderError::OutOfRange { .. } => -2,
            ProviderError::DataMissing { .. } => -3,
            ProviderError::Refused { .. } => -4,
            ProviderError::Invalid { .. } => -5,
            ProviderError::Provider { code, .. } => *code,
        }
    }

    /// The error a C code stands for, with a context string as detail.
    #[must_use]
    pub fn from_code(code: i32, context: &str) -> ProviderError {
        match code {
            -1 => ProviderError::Unsupported {
                what: context.to_string(),
            },
            -2 => ProviderError::OutOfRange { jd: f64::NAN },
            -3 => ProviderError::DataMissing {
                detail: context.to_string(),
            },
            -4 => ProviderError::Refused {
                detail: context.to_string(),
            },
            -5 => ProviderError::Invalid {
                detail: context.to_string(),
            },
            other => ProviderError::Provider {
                code: other,
                detail: context.to_string(),
            },
        }
    }

    /// An unsupported-operation error.
    #[must_use]
    pub fn unsupported(what: &str) -> ProviderError {
        ProviderError::Unsupported {
            what: what.to_string(),
        }
    }
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProviderError::Unsupported { what } => {
                write!(f, "the provider does not support {what}")
            }
            ProviderError::OutOfRange { jd } => {
                write!(f, "JD {jd} is outside the provider's coverage")
            }
            ProviderError::DataMissing { detail } => write!(f, "a data file is missing: {detail}"),
            ProviderError::Refused { detail } => write!(f, "the provider refused: {detail}"),
            ProviderError::Invalid { detail } => write!(f, "invalid request: {detail}"),
            ProviderError::Provider { code, detail } => {
                write!(f, "provider error {code}: {detail}")
            }
        }
    }
}

impl std::error::Error for ProviderError {}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, reason = "a test fails by panicking")]

    use super::*;

    #[test]
    fn frames_round_trip_through_bits() {
        let frames = [
            Frame::CANONICAL,
            Frame::CANONICAL.with_coordinates(Coordinates::Equatorial),
            Frame::CANONICAL.with_zodiac(Zodiac::Sidereal(AyanamshaId::TRUE_CHITRA)),
            Frame {
                center: Center::Topocentric,
                equinox: Equinox::J2000,
                coordinates: Coordinates::Equatorial,
                zodiac: Zodiac::Tropical,
                corrections: Corrections::GEOMETRIC,
            },
        ];
        for frame in frames {
            assert_eq!(Frame::from_bits(frame.to_bits()), frame);
        }
    }

    #[test]
    fn bodies_and_statuses_round_trip() {
        for body in Body::ALL {
            assert_eq!(Body::from_id(body.id()), Some(body));
        }
        for status in [
            CellStatus::Ok,
            CellStatus::NotComputed,
            CellStatus::UnsupportedBody,
            CellStatus::OutOfRange,
            CellStatus::DataMissing,
            CellStatus::Provider(7),
        ] {
            assert_eq!(CellStatus::from_code(status.code()), status);
        }
        let source = Source {
            kind: EphemerisKind::Files,
            tier: Some(Tier::Standard),
        };
        assert_eq!(Source::from_bits(source.to_bits()), source);
    }

    #[test]
    fn columns_index_instants_outermost() {
        let mut columns = PositionColumns::new(2, 3, Frame::CANONICAL);
        assert_eq!(columns.index(1, 2), Some(5));
        assert_eq!(columns.index(2, 0), None);
        columns.set(
            5,
            Cell {
                lon: 1.5,
                status: CellStatus::Ok,
                ..Cell::EMPTY
            },
        );
        assert_eq!(columns.at(1, 2).map(|c| c.lon), Some(1.5));
        assert!(!columns.all_ok());
        assert!(columns.bit_identical(&columns.clone()));
    }

    #[test]
    fn overrides_name_their_bits() {
        let set = Overrides::OBLIQUITY.with(Overrides::AYANAMSHA);
        assert!(set.contains(Overrides::OBLIQUITY));
        assert!(!set.contains(Overrides::HOUSES));
        assert_eq!(set.names(), vec!["obliquity", "ayanamsha"]);
        assert_eq!(Overrides::from_bits(set.bits()), set);
    }
}
