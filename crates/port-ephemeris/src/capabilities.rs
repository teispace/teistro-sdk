//! What a provider declares about itself: identity with content hashes,
//! coverage, bodies, the native frame, the overrides it offers as a bit
//! set, and the obliquity record its override answers with.

use core::fmt;
use core::fmt::Write as _;
use std::io::Read;
use std::path::Path;

use serde::Serialize;
use sha2::{Digest, Sha256};
use teistro_core::catalogue::Ayanamsha;
use teistro_core::envelope::{Hash, ProviderStamp};
use teistro_core::settings::Tier;

use crate::body::Body;
use crate::frame::Frame;

/// The native operations a provider may implement instead of the SDK,
/// as a bit set.
///
/// ```
/// use teistro_port_ephemeris::Overrides;
///
/// let set = Overrides::OBLIQUITY.with(Overrides::RISE_SET);
/// assert!(set.contains(Overrides::OBLIQUITY));
/// assert!(!set.contains(Overrides::HOUSES));
/// assert_eq!(set.names(), vec!["obliquity", "rise_set"]);
/// ```
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
    /// A station is a crossing of the speed, answered under `CROSSINGS`; a
    /// provider whose station search is separate declares this too.
    pub const STATIONS: Overrides = Overrides(1 << 8);
    /// Eclipses.
    pub const ECLIPSES: Overrides = Overrides(1 << 9);
    /// Fixed stars.
    pub const STARS: Overrides = Overrides(1 << 10);
    /// UT1 less UTC (DUT1), from the provider's own IERS bulletins.
    pub const DUT1: Overrides = Overrides(1 << 11);

    const NAMES: [(Overrides, &'static str); 12] = [
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
        (Overrides::DUT1, "dut1"),
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

    /// The names of the set overrides, in bit order.
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

    /// The hash as the envelope carries it, when the hex is well formed.
    #[must_use]
    pub fn envelope_hash(&self) -> Option<Hash> {
        Hash::from_hex(&self.sha256)
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

impl Identity {
    /// The stamp the result envelope carries: this identity, the frame
    /// returned and the switches used.
    #[must_use]
    pub fn stamp(&self, frame: Frame, flags_used: Vec<String>) -> ProviderStamp {
        ProviderStamp {
            name: self.name.clone(),
            version: self.version.clone(),
            data_version: self.data_version.clone(),
            data_hashes: self
                .data_hashes
                .iter()
                .filter_map(|h| h.envelope_hash().map(|hash| (h.file.clone(), hash)))
                .collect(),
            tier: self.tier.map(|t| t.key().to_string()),
            frame: frame.key(),
            flags_used,
        }
    }
}

impl fmt::Display for Identity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} ({})", self.name, self.version, self.data_version)?;
        if let Some(tier) = self.tier {
            write!(f, " tier {}", tier.key())?;
        }
        Ok(())
    }
}

/// What a cell's distance is measured in.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DistanceUnit {
    /// Astronomical units: an ephemeris.
    #[default]
    AstronomicalUnits = 0,
    /// The body's mean distance, so 1 is the mean: a classical model,
    /// whose hypotenuse is on the radius.
    MeanDistances = 1,
}

impl DistanceUnit {
    /// The stable id at the C boundary.
    #[must_use]
    pub const fn id(self) -> u8 {
        self as u8
    }

    /// The unit with an id.
    #[must_use]
    pub const fn from_id(id: u8) -> Option<DistanceUnit> {
        match id {
            0 => Some(DistanceUnit::AstronomicalUnits),
            1 => Some(DistanceUnit::MeanDistances),
            _ => None,
        }
    }

    /// The key stamped in provenance.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            DistanceUnit::AstronomicalUnits => "AU",
            DistanceUnit::MeanDistances => "MEAN_DISTANCES",
        }
    }
}

/// How a provider's speeds are defined.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpeedModel {
    /// The rate of the position: a central difference over a short step
    /// agrees with it, which the kit checks.
    #[default]
    Derivative = 0,
    /// A text's rule for the daily motion, which its tradition uses as
    /// the speed and which need not be the derivative of its places.
    Rule = 1,
}

impl SpeedModel {
    /// The stable id at the C boundary.
    #[must_use]
    pub const fn id(self) -> u8 {
        self as u8
    }

    /// The model with an id.
    #[must_use]
    pub const fn from_id(id: u8) -> Option<SpeedModel> {
        match id {
            0 => Some(SpeedModel::Derivative),
            1 => Some(SpeedModel::Rule),
            _ => None,
        }
    }
}

/// Which astronomy a provider computes.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Astronomy {
    /// The sky as observed: an ephemeris, whose overrides the kit holds
    /// to the SDK's IAU routines.
    #[default]
    Modern = 0,
    /// A classical text's model, whose obliquity, precession, daily
    /// motions and sunrise are the text's own definitions; the kit
    /// measures their distance from modern astronomy and publishes it
    /// rather than gating it.
    Classical = 1,
}

impl Astronomy {
    /// The stable id at the C boundary.
    #[must_use]
    pub const fn id(self) -> u8 {
        self as u8
    }

    /// The astronomy with an id.
    #[must_use]
    pub const fn from_id(id: u8) -> Option<Astronomy> {
        match id {
            0 => Some(Astronomy::Modern),
            1 => Some(Astronomy::Classical),
            _ => None,
        }
    }
}

/// What a provider declares about itself.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Capabilities {
    /// Who.
    pub identity: Identity,
    /// Coverage in UT1 Julian days, inclusive.
    pub jd_range: (f64, f64),
    /// The bodies it computes.
    pub bodies: Vec<Body>,
    /// The frame `positions` returns.
    pub native_frame: Frame,
    /// Which astronomy it computes.
    pub astronomy: Astronomy,
    /// Whether it returns speeds.
    pub speeds: bool,
    /// How its speeds are defined.
    pub speed_model: SpeedModel,
    /// What its distances are measured in.
    pub distance_unit: DistanceUnit,
    /// The native overrides it offers.
    pub overrides: Overrides,
    /// The ayanamshas its override knows.
    pub ayanamshas: Vec<Ayanamsha>,
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

    /// Whether an override is declared.
    #[must_use]
    pub const fn has(&self, overrides: Overrides) -> bool {
        self.overrides.contains(overrides)
    }

    /// Whether the ayanamsha override knows an ayanamsha.
    #[must_use]
    pub fn has_ayanamsha(&self, ayanamsha: Ayanamsha) -> bool {
        self.has(Overrides::AYANAMSHA) && self.ayanamshas.contains(&ayanamsha)
    }

    /// One line for a report: bodies, coverage and overrides.
    #[must_use]
    pub fn describe(&self) -> String {
        let ayanamshas: Vec<&str> = self.ayanamshas.iter().map(|a| a.key()).collect();
        format!(
            "{}: {} bodies, JD {:.1} to {:.1}, native frame {}, distances in {}, overrides [{}], ayanamshas [{}]{}{}{}",
            self.identity,
            self.bodies.len(),
            self.jd_range.0,
            self.jd_range.1,
            self.native_frame,
            self.distance_unit.key(),
            self.overrides,
            ayanamshas.join(", "),
            if self.astronomy == Astronomy::Classical {
                ", classical"
            } else {
                ""
            },
            if self.speed_model == SpeedModel::Rule {
                ", speeds by rule"
            } else {
                ""
            },
            if self.deterministic {
                ""
            } else {
                ", non-deterministic"
            }
        )
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

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    #[test]
    fn overrides_name_their_bits() {
        let set = Overrides::OBLIQUITY.with(Overrides::AYANAMSHA);
        assert!(set.contains(Overrides::OBLIQUITY));
        assert!(!set.contains(Overrides::HOUSES));
        assert_eq!(set.names(), vec!["obliquity", "ayanamsha"]);
        assert_eq!(Overrides::from_bits(set.bits()), set);
        assert_eq!(set.to_string(), "obliquity, ayanamsha");
        assert_eq!(
            serde_json::to_string(&set).unwrap(),
            "[\"obliquity\",\"ayanamsha\"]"
        );
    }

    #[test]
    fn a_file_hashes_and_stamps() {
        let dir = std::env::temp_dir().join("teistro-port-ephemeris-hash");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("sample.txt");
        std::fs::write(&path, b"abc").unwrap();
        let hash = DataHash::of_file(&path).unwrap();
        assert_eq!(hash.file, "sample.txt");
        assert_eq!(hash.bytes, 3);
        assert_eq!(
            hash.sha256,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert!(hash.envelope_hash().is_some());
        let identity = Identity {
            name: String::from("test"),
            version: String::from("1"),
            data_version: String::from("d"),
            tier: Some(Tier::Standard),
            data_hashes: vec![hash],
        };
        let stamp = identity.stamp(Frame::CANONICAL, vec![String::from("speed")]);
        assert_eq!(stamp.data_hashes.len(), 1);
        assert_eq!(stamp.tier.as_deref(), Some("STANDARD"));
        assert_eq!(stamp.frame, Frame::CANONICAL.key());
        assert_eq!(identity.to_string(), "test 1 (d) tier STANDARD");
    }

    #[test]
    fn capabilities_answer_coverage_and_bodies() {
        let capabilities = Capabilities {
            identity: Identity {
                name: String::from("t"),
                version: String::from("1"),
                data_version: String::new(),
                tier: None,
                data_hashes: Vec::new(),
            },
            jd_range: (0.0, 10.0),
            bodies: vec![Body::Sun],
            native_frame: Frame::CANONICAL,
            astronomy: Astronomy::Modern,
            speeds: true,
            speed_model: SpeedModel::Derivative,
            distance_unit: DistanceUnit::AstronomicalUnits,
            overrides: Overrides::AYANAMSHA,
            ayanamshas: vec![Ayanamsha::Lahiri],
            deterministic: false,
        };
        assert!(capabilities.covers(5.0) && !capabilities.covers(11.0));
        assert!(capabilities.has_body(Body::Sun) && !capabilities.has_body(Body::Moon));
        assert!(capabilities.has_ayanamsha(Ayanamsha::Lahiri));
        assert!(!capabilities.has_ayanamsha(Ayanamsha::Raman));
        assert!(capabilities.describe().ends_with("non-deterministic"));
        assert!(capabilities.describe().contains("distances in AU"));
        assert_eq!(
            DistanceUnit::from_id(DistanceUnit::MeanDistances.id()),
            Some(DistanceUnit::MeanDistances)
        );
        assert_eq!(
            SpeedModel::from_id(SpeedModel::Rule.id()),
            Some(SpeedModel::Rule)
        );
        assert_eq!(DistanceUnit::from_id(9), None);
        assert_eq!(SpeedModel::from_id(9), None);
        assert_eq!(
            Astronomy::from_id(Astronomy::Classical.id()),
            Some(Astronomy::Classical)
        );
        assert_eq!(Astronomy::from_id(9), None);
    }
}
