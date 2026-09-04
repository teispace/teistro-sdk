//! Spike 2: the domain slice both binding toolchains bind.
//!
//! One context holding settings and an ephemeris port, and one batch call
//! that asks the port for nine positions and returns a tree-shaped result:
//! the classified positions and a Vimshottari tree to a requested depth.
//! The shapes are the SDK's (ADR-0016 nanoarcsecond classification, a
//! port with only `position` required, a tree that grows as `9^depth`), the
//! numbers are not: the ayanamsha model and the test provider are simple
//! analytic stand-ins so the spike measures binding cost, never astronomy.
//!
//! Nothing here is `unsafe`; the FFI crates of options A and B sit on top.

#![forbid(unsafe_code)]

use core::fmt;

/// One circle in nanoarcseconds (ADR-0016): 360 × 3600 × 10⁹.
pub const CIRCLE_NAS: i64 = 1_296_000_000_000_000;
/// One nakshatra in nanoarcseconds, exactly `CIRCLE_NAS / 27`.
pub const NAKSHATRA_NAS: i64 = CIRCLE_NAS / 27;
/// Days in a dasha year: Julian years, as the baseline engine counts them.
pub const DAYS_PER_YEAR: f64 = 365.25;
/// The Vimshottari cycle in years.
pub const VIMSHOTTARI_TOTAL_YEARS: u32 = 120;
/// The Julian Day of J2000.0 in UT, the epoch of the analytic models here.
pub const J2000_JD: f64 = 2_451_545.0;

/// The nine bodies of the slice, in the canonical order of the fixtures.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum Body {
    /// The Sun.
    Sun = 0,
    /// The Moon.
    Moon = 1,
    /// Mars.
    Mars = 2,
    /// Mercury.
    Mercury = 3,
    /// Jupiter.
    Jupiter = 4,
    /// Venus.
    Venus = 5,
    /// Saturn.
    Saturn = 6,
    /// The ascending lunar node.
    Rahu = 7,
    /// The descending lunar node, always opposite Rahu.
    Ketu = 8,
}

impl Body {
    /// Every body, in index order.
    pub const ALL: [Body; 9] = [
        Body::Sun,
        Body::Moon,
        Body::Mars,
        Body::Mercury,
        Body::Jupiter,
        Body::Venus,
        Body::Saturn,
        Body::Rahu,
        Body::Ketu,
    ];

    /// The stable index, `0` for the Sun to `8` for Ketu.
    #[must_use]
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// The body with a given index, `None` outside `0..=8`.
    #[must_use]
    pub const fn from_index(index: u8) -> Option<Body> {
        match index {
            0 => Some(Body::Sun),
            1 => Some(Body::Moon),
            2 => Some(Body::Mars),
            3 => Some(Body::Mercury),
            4 => Some(Body::Jupiter),
            5 => Some(Body::Venus),
            6 => Some(Body::Saturn),
            7 => Some(Body::Rahu),
            8 => Some(Body::Ketu),
            _ => None,
        }
    }

    /// The upper-case key used in fixtures and bindings.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Body::Sun => "SUN",
            Body::Moon => "MOON",
            Body::Mars => "MARS",
            Body::Mercury => "MERCURY",
            Body::Jupiter => "JUPITER",
            Body::Venus => "VENUS",
            Body::Saturn => "SATURN",
            Body::Rahu => "RAHU",
            Body::Ketu => "KETU",
        }
    }
}

/// The ayanamsha catalogue of the slice: three entries, enough to make the
/// setting real for the bindings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Ayanamsha {
    /// Lahiri (Chitrapaksha), the default.
    Lahiri = 0,
    /// B. V. Raman.
    Raman = 1,
    /// K. S. Krishnamurti.
    Krishnamurti = 2,
}

impl Ayanamsha {
    /// The stable index.
    #[must_use]
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// The ayanamsha with a given index, `None` outside `0..=2`.
    #[must_use]
    pub const fn from_index(index: u8) -> Option<Ayanamsha> {
        match index {
            0 => Some(Ayanamsha::Lahiri),
            1 => Some(Ayanamsha::Raman),
            2 => Some(Ayanamsha::Krishnamurti),
            _ => None,
        }
    }

    /// The value at J2000.0 in degrees. Rounded published values, used
    /// only to give the three settings distinct numbers.
    #[must_use]
    pub const fn value_at_j2000_deg(self) -> f64 {
        match self {
            Ayanamsha::Lahiri => 23.853_1,
            Ayanamsha::Raman => 22.404_1,
            Ayanamsha::Krishnamurti => 23.756_3,
        }
    }
}

/// Which lunar node the provider is asked for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum NodeKind {
    /// The mean node.
    Mean = 0,
    /// The true (osculating) node.
    True = 1,
}

impl NodeKind {
    /// The stable index.
    #[must_use]
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// The node kind with a given index, `None` outside `0..=1`.
    #[must_use]
    pub const fn from_index(index: u8) -> Option<NodeKind> {
        match index {
            0 => Some(NodeKind::Mean),
            1 => Some(NodeKind::True),
            _ => None,
        }
    }
}

/// The settings of a context: the one settings struct of the spike.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Settings {
    /// The ayanamsha that turns the provider's tropical longitudes sidereal.
    pub ayanamsha: Ayanamsha,
    /// Which lunar node to ask the provider for.
    pub node: NodeKind,
    /// How many dasha levels to build, `1..=5`; the tree has `9^depth`
    /// leaves.
    pub dasha_depth: u8,
}

impl Settings {
    /// The deepest tree a context builds: 66 429 nodes.
    pub const MAX_DASHA_DEPTH: u8 = 5;

    /// Lahiri, the mean node, three levels.
    pub const DEFAULT: Settings = Settings {
        ayanamsha: Ayanamsha::Lahiri,
        node: NodeKind::Mean,
        dasha_depth: 3,
    };

    /// Checks the settings are coherent.
    ///
    /// # Errors
    ///
    /// [`Error::DepthOutOfRange`] when `dasha_depth` is `0` or above
    /// [`Settings::MAX_DASHA_DEPTH`].
    pub const fn validate(self) -> Result<Settings, Error> {
        if self.dasha_depth == 0 || self.dasha_depth > Settings::MAX_DASHA_DEPTH {
            return Err(Error::DepthOutOfRange {
                depth: self.dasha_depth,
            });
        }
        Ok(self)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings::DEFAULT
    }
}

/// A tropical position as a provider returns it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    /// Tropical ecliptic longitude in degrees, any real number; the slice
    /// normalises it.
    pub longitude_deg: f64,
    /// Ecliptic latitude in degrees.
    pub latitude_deg: f64,
    /// Longitude speed in degrees per day; negative means retrograde.
    pub speed_deg_per_day: f64,
}

/// Everything that can go wrong in the slice, each with a stable code so
/// every binding reports the same number.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Error {
    /// `dasha_depth` is not in `1..=5`.
    DepthOutOfRange {
        /// The depth that was asked for.
        depth: u8,
    },
    /// The Julian Day is NaN or infinite.
    JulianDayNotFinite,
    /// The provider returned an error code for a body.
    Provider {
        /// The body being asked for.
        body: Body,
        /// The provider's own code, passed through unchanged.
        code: i32,
    },
    /// The provider returned a NaN or infinite longitude.
    PositionNotFinite {
        /// The body whose position was rejected.
        body: Body,
    },
}

impl Error {
    /// The stable numeric code of the variant.
    #[must_use]
    pub const fn code(self) -> i32 {
        match self {
            Error::DepthOutOfRange { .. } => 1,
            Error::JulianDayNotFinite => 2,
            Error::Provider { .. } => 3,
            Error::PositionNotFinite { .. } => 4,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::DepthOutOfRange { depth } => {
                write!(f, "dasha depth {depth} is outside 1..=5")
            }
            Error::JulianDayNotFinite => f.write_str("the Julian Day is not finite"),
            Error::Provider { body, code } => {
                write!(f, "the provider failed for {} with code {code}", body.key())
            }
            Error::PositionNotFinite { body } => {
                write!(
                    f,
                    "the provider returned a non-finite longitude for {}",
                    body.key()
                )
            }
        }
    }
}

impl std::error::Error for Error {}

/// The ephemeris port: the one callback the host must implement.
///
/// A provider returns tropical positions; `Err(code)` carries the
/// provider's own error code through to [`Error::Provider`].
pub trait EphemerisPort {
    /// The position of `body` at `jd_ut`, a Julian Day in UT.
    ///
    /// # Errors
    ///
    /// The provider's own code when it cannot compute the position.
    fn position(&self, jd_ut: f64, body: Body) -> Result<Position, i32>;
}

/// The one context type: settings plus a provider, no global state.
pub struct Context {
    settings: Settings,
    provider: Box<dyn EphemerisPort>,
}

impl fmt::Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Context")
            .field("settings", &self.settings)
            .finish_non_exhaustive()
    }
}

/// A classified sidereal position.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BodyPosition {
    /// The body.
    pub body: Body,
    /// Sidereal longitude in degrees, `[0, 360)`.
    pub longitude_deg: f64,
    /// The same longitude in nanoarcseconds, the value the classification
    /// is made from.
    pub longitude_nas: i64,
    /// Ecliptic latitude in degrees, as the provider gave it.
    pub latitude_deg: f64,
    /// Longitude speed in degrees per day, as the provider gave it.
    pub speed_deg_per_day: f64,
    /// Sign index, `0` is Aries.
    pub sign: u8,
    /// Nakshatra index, `0` is Ashwini.
    pub nakshatra: u8,
    /// Pada, `1..=4`.
    pub pada: u8,
    /// Whether the speed is negative.
    pub retrograde: bool,
}

/// One period of the tree, with its sub-periods.
#[derive(Clone, Debug, PartialEq)]
pub struct DashaNode {
    /// The lord of the period.
    pub lord: Body,
    /// `1` for a mahadasha, up to `5`.
    pub level: u8,
    /// Start, Julian Day in UT.
    pub start_jd: f64,
    /// End, Julian Day in UT; the start of the next period.
    pub end_jd: f64,
    /// The sub-periods, empty at the requested depth.
    pub children: Vec<DashaNode>,
}

/// One row of the tree in pre-order: the flat shape the bindings marshal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DashaRow {
    /// `1` for a mahadasha, up to `5`.
    pub level: u8,
    /// The lord of the period.
    pub lord: Body,
    /// The row index of the parent, `-1` for a mahadasha.
    pub parent: i32,
    /// Start, Julian Day in UT.
    pub start_jd: f64,
    /// End, Julian Day in UT.
    pub end_jd: f64,
}

/// The tree-shaped result of the one batch call.
#[derive(Clone, Debug, PartialEq)]
pub struct Chart {
    /// The instant the chart was computed for.
    pub jd_ut: f64,
    /// The ayanamsha value applied, in degrees.
    pub ayanamsha_deg: f64,
    /// The nine positions in [`Body::ALL`] order.
    pub positions: Vec<BodyPosition>,
    /// The mahadashas, each with its sub-periods to the requested depth.
    pub dasha: Vec<DashaNode>,
}

impl Chart {
    /// The tree flattened in pre-order with parent links.
    #[must_use]
    pub fn dasha_rows(&self) -> Vec<DashaRow> {
        let mut rows = Vec::with_capacity(Chart::node_count_for_depth(self.depth()));
        for node in &self.dasha {
            push_rows(node, -1, &mut rows);
        }
        rows
    }

    /// The number of nodes in the tree.
    #[must_use]
    pub fn dasha_node_count(&self) -> usize {
        self.dasha.iter().map(count_nodes).sum()
    }

    /// The depth the tree was built to, `0` for an empty tree.
    #[must_use]
    pub fn depth(&self) -> u8 {
        self.dasha.first().map_or(0, depth_of)
    }

    /// The node count of a full tree of a given depth: `9 + 81 + … + 9^depth`.
    #[must_use]
    pub const fn node_count_for_depth(depth: u8) -> usize {
        let mut total = 0usize;
        let mut layer = 1usize;
        let mut level = 0u8;
        while level < depth {
            layer *= 9;
            total += layer;
            level += 1;
        }
        total
    }
}

fn push_rows(node: &DashaNode, parent: i32, rows: &mut Vec<DashaRow>) {
    let index = i32::try_from(rows.len()).unwrap_or(i32::MAX);
    rows.push(DashaRow {
        level: node.level,
        lord: node.lord,
        parent,
        start_jd: node.start_jd,
        end_jd: node.end_jd,
    });
    for child in &node.children {
        push_rows(child, index, rows);
    }
}

fn count_nodes(node: &DashaNode) -> usize {
    1 + node.children.iter().map(count_nodes).sum::<usize>()
}

fn depth_of(node: &DashaNode) -> u8 {
    1 + node.children.first().map_or(0, depth_of)
}

impl Context {
    /// Builds a context after validating the settings.
    ///
    /// # Errors
    ///
    /// The settings' own validation error.
    pub fn new(settings: Settings, provider: Box<dyn EphemerisPort>) -> Result<Context, Error> {
        let settings = settings.validate()?;
        Ok(Context { settings, provider })
    }

    /// The settings the context was built with.
    #[must_use]
    pub const fn settings(&self) -> Settings {
        self.settings
    }

    /// The one batch call: nine port calls, classification, and the tree.
    ///
    /// # Errors
    ///
    /// [`Error::JulianDayNotFinite`], or the provider's failure for the
    /// first body it cannot compute, or a non-finite longitude from it.
    pub fn compute_chart(&self, jd_ut: f64) -> Result<Chart, Error> {
        if !jd_ut.is_finite() {
            return Err(Error::JulianDayNotFinite);
        }
        let ayanamsha_deg = ayanamsha_deg(self.settings.ayanamsha, jd_ut);
        let mut positions = Vec::with_capacity(Body::ALL.len());
        let mut moon_nas = 0i64;
        for body in Body::ALL {
            let raw = self
                .provider
                .position(jd_ut, body)
                .map_err(|code| Error::Provider { body, code })?;
            if !raw.longitude_deg.is_finite() {
                return Err(Error::PositionNotFinite { body });
            }
            let sidereal = normalise_deg(raw.longitude_deg - ayanamsha_deg);
            let nas = nas_from_deg(sidereal);
            if body == Body::Moon {
                moon_nas = nas;
            }
            positions.push(BodyPosition {
                body,
                longitude_deg: sidereal,
                longitude_nas: nas,
                latitude_deg: raw.latitude_deg,
                speed_deg_per_day: raw.speed_deg_per_day,
                sign: division_index(nas, 12),
                nakshatra: division_index(nas, 27),
                pada: division_index(nas, 108) % 4 + 1,
                retrograde: raw.speed_deg_per_day < 0.0,
            });
        }
        let dasha = build_vimshottari(moon_nas, jd_ut, self.settings.dasha_depth);
        Ok(Chart {
            jd_ut,
            ayanamsha_deg,
            positions,
            dasha,
        })
    }
}

/// Normalises degrees into `[0, 360)`.
#[must_use]
pub fn normalise_deg(deg: f64) -> f64 {
    let wrapped = deg.rem_euclid(360.0);
    if wrapped >= 360.0 { 0.0 } else { wrapped }
}

/// Degrees to nanoarcseconds: normalised, then rounded to the nearest
/// nanoarcsecond, the one place the slice rounds (ADR-0016).
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    reason = "the value is inside [0, CIRCLE_NAS) after normalisation, which fits i64"
)]
pub fn nas_from_deg(deg: f64) -> i64 {
    let scaled = (normalise_deg(deg) * 3_600_000_000_000.0).round() as i64;
    scaled.rem_euclid(CIRCLE_NAS)
}

/// The zero-based division index of a longitude: `floor(nas × divisions /
/// CIRCLE_NAS)` in 128-bit integer arithmetic, so a boundary is exact and
/// lower-inclusive.
#[must_use]
pub fn division_index(nas: i64, divisions: i64) -> u8 {
    let nas = nas.rem_euclid(CIRCLE_NAS);
    let index = i128::from(nas) * i128::from(divisions) / i128::from(CIRCLE_NAS);
    u8::try_from(index).unwrap_or(u8::MAX)
}

/// The ayanamsha value at an instant: the J2000 value plus a linear rate of
/// 50.29 arcseconds per Julian year. A stand-in, not a model of precession.
#[must_use]
pub fn ayanamsha_deg(kind: Ayanamsha, jd_ut: f64) -> f64 {
    let years = (jd_ut - J2000_JD) / DAYS_PER_YEAR;
    kind.value_at_j2000_deg() + years * (50.29 / 3600.0)
}

/// The Vimshottari sequence as `(lord, years)`, from Ketu.
pub const VIMSHOTTARI: [(Body, u32); 9] = [
    (Body::Ketu, 7),
    (Body::Venus, 20),
    (Body::Sun, 6),
    (Body::Moon, 10),
    (Body::Mars, 7),
    (Body::Rahu, 18),
    (Body::Jupiter, 16),
    (Body::Saturn, 19),
    (Body::Mercury, 17),
];

/// Builds the mahadashas from the Moon's longitude with the spatial balance:
/// the first period is shortened by the elapsed fraction of the nakshatra,
/// computed from exact nanoarcsecond integers.
#[must_use]
#[allow(
    clippy::cast_precision_loss,
    reason = "the two integers are below 2^46, exactly representable in f64"
)]
pub fn build_vimshottari(moon_nas: i64, birth_jd: f64, depth: u8) -> Vec<DashaNode> {
    if depth == 0 {
        return Vec::new();
    }
    let nakshatra = i64::from(division_index(moon_nas, 27));
    let within = moon_nas.rem_euclid(CIRCLE_NAS) - nakshatra * NAKSHATRA_NAS;
    let remaining = NAKSHATRA_NAS - within;
    let fraction = remaining as f64 / NAKSHATRA_NAS as f64;
    let start_index = usize::try_from(nakshatra % 9).unwrap_or(0);
    let mut nodes = Vec::with_capacity(9);
    let mut start = birth_jd;
    for offset in 0..9 {
        let index = (start_index + offset) % 9;
        let (lord, years) = VIMSHOTTARI.get(index).copied().unwrap_or((Body::Ketu, 7));
        let full = f64::from(years) * DAYS_PER_YEAR;
        let duration = if offset == 0 { full * fraction } else { full };
        let end = start + duration;
        let children = if depth > 1 {
            build_sub_periods(index, start, duration, 2, depth)
        } else {
            Vec::new()
        };
        nodes.push(DashaNode {
            lord,
            level: 1,
            start_jd: start,
            end_jd: end,
            children,
        });
        start = end;
    }
    nodes
}

fn build_sub_periods(
    parent_index: usize,
    parent_start: f64,
    parent_duration: f64,
    level: u8,
    depth: u8,
) -> Vec<DashaNode> {
    let mut nodes = Vec::with_capacity(9);
    let mut start = parent_start;
    for offset in 0..9 {
        let index = (parent_index + offset) % 9;
        let (lord, years) = VIMSHOTTARI.get(index).copied().unwrap_or((Body::Ketu, 7));
        let duration = parent_duration * f64::from(years) / f64::from(VIMSHOTTARI_TOTAL_YEARS);
        let end = start + duration;
        let children = if level < depth {
            build_sub_periods(index, start, duration, level + 1, depth)
        } else {
            Vec::new()
        };
        nodes.push(DashaNode {
            lord,
            level,
            start_jd: start,
            end_jd: end,
            children,
        });
        start = end;
    }
    nodes
}

/// A deterministic analytic provider: mean motions from J2000 with one
/// periodic term, so a chart costs microseconds and the host provider's
/// callback cost stands out. Not an ephemeris.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TestProvider;

impl TestProvider {
    /// `(longitude at J2000, mean motion in degrees per day, amplitude of
    /// the periodic term)` per body.
    const ELEMENTS: [(f64, f64, f64); 9] = [
        (280.46, 0.985_647_4, 1.915),
        (218.32, 13.176_396, 6.289),
        (355.45, 0.524_033, 10.69),
        (252.25, 4.092_339, 23.44),
        (34.35, 0.083_129, 5.55),
        (181.98, 1.602_130, 0.77),
        (50.08, 0.033_444, 6.41),
        (125.04, -0.052_954, 0.0),
        (305.04, -0.052_954, 0.0),
    ];
}

impl EphemerisPort for TestProvider {
    fn position(&self, jd_ut: f64, body: Body) -> Result<Position, i32> {
        let (l0, rate, amplitude) = TestProvider::ELEMENTS
            .get(usize::from(body.index()))
            .copied()
            .ok_or(-1)?;
        let t = jd_ut - J2000_JD;
        let mean = l0 + rate * t;
        let anomaly = (mean * 0.017_453_292_519_943_295).sin();
        Ok(Position {
            longitude_deg: normalise_deg(mean + amplitude * anomaly),
            latitude_deg: 0.05 * amplitude * (mean * 0.008_726_646_259_971_648).cos(),
            speed_deg_per_day: rate
                + amplitude
                    * rate
                    * 0.017_453_292_519_943_295
                    * (mean * 0.017_453_292_519_943_295).cos(),
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::items_after_statements,
        reason = "a test fails by panicking and keeps its helpers beside the assertions"
    )]

    use super::*;

    fn context(depth: u8) -> Context {
        Context::new(
            Settings {
                dasha_depth: depth,
                ..Settings::DEFAULT
            },
            Box::new(TestProvider),
        )
        .unwrap_or_else(|err| panic!("valid settings: {err}"))
    }

    #[test]
    fn node_count_follows_the_geometric_series() {
        assert_eq!(Chart::node_count_for_depth(0), 0);
        assert_eq!(Chart::node_count_for_depth(1), 9);
        assert_eq!(Chart::node_count_for_depth(3), 819);
        assert_eq!(Chart::node_count_for_depth(5), 66_429);
        let chart = context(3)
            .compute_chart(2_460_000.5)
            .unwrap_or_else(|err| panic!("chart: {err}"));
        assert_eq!(chart.dasha_node_count(), 819);
        assert_eq!(chart.dasha_rows().len(), 819);
        assert_eq!(chart.depth(), 3);
    }

    #[test]
    fn classification_is_lower_inclusive_and_exact() {
        assert_eq!(division_index(0, 12), 0);
        assert_eq!(division_index(CIRCLE_NAS / 12, 12), 1);
        assert_eq!(division_index(CIRCLE_NAS / 12 - 1, 12), 0);
        assert_eq!(division_index(NAKSHATRA_NAS, 27), 1);
        assert_eq!(division_index(CIRCLE_NAS - 1, 27), 26);
        assert_eq!(nas_from_deg(360.0), 0);
        assert_eq!(nas_from_deg(-0.000_000_000_1), CIRCLE_NAS - 360);
        assert_eq!(nas_from_deg(-0.000_000_000_000_1), 0);
        assert_eq!(nas_from_deg(30.0), CIRCLE_NAS / 12);
    }

    #[test]
    fn children_tile_their_parent_exactly_enough() {
        let chart = context(2)
            .compute_chart(2_451_545.0)
            .unwrap_or_else(|err| panic!("chart: {err}"));
        for node in &chart.dasha {
            let first = node.children.first().map_or(0.0, |c| c.start_jd);
            let last = node.children.last().map_or(0.0, |c| c.end_jd);
            assert!((first - node.start_jd).abs() < 1e-9);
            assert!((last - node.end_jd).abs() < 1e-6);
            assert_eq!(node.children.len(), 9);
            assert_eq!(node.children.first().map(|c| c.lord), Some(node.lord));
        }
        let total: f64 = chart.dasha.iter().map(|n| n.end_jd - n.start_jd).sum();
        assert!(total <= 120.0 * DAYS_PER_YEAR + 1e-6);
    }

    #[test]
    fn errors_have_stable_codes_and_reach_the_caller() {
        let bad = Settings {
            dasha_depth: 6,
            ..Settings::DEFAULT
        };
        assert_eq!(bad.validate(), Err(Error::DepthOutOfRange { depth: 6 }));
        assert_eq!(Error::DepthOutOfRange { depth: 6 }.code(), 1);
        struct Failing;
        impl EphemerisPort for Failing {
            fn position(&self, _: f64, body: Body) -> Result<Position, i32> {
                Err(i32::from(body.index()) + 100)
            }
        }
        let ctx = Context::new(Settings::DEFAULT, Box::new(Failing))
            .unwrap_or_else(|err| panic!("valid settings: {err}"));
        assert_eq!(
            ctx.compute_chart(2_460_000.5),
            Err(Error::Provider {
                body: Body::Sun,
                code: 100
            })
        );
        assert_eq!(ctx.compute_chart(f64::NAN), Err(Error::JulianDayNotFinite));
    }

    #[test]
    fn the_test_provider_is_deterministic_and_finite() {
        let a = TestProvider
            .position(2_460_000.5, Body::Moon)
            .unwrap_or_else(|code| panic!("code {code}"));
        let b = TestProvider
            .position(2_460_000.5, Body::Moon)
            .unwrap_or_else(|code| panic!("code {code}"));
        assert_eq!(a, b);
        assert!(a.longitude_deg.is_finite() && (0.0..360.0).contains(&a.longitude_deg));
    }
}
