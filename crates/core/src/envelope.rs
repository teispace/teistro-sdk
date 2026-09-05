//! The result envelope: every value the SDK returns travels with the
//! provenance that reproduces it (ADR-0020): versions, the calculation
//! version, hashes of the input and the settings, the provider and the
//! packs, what the time layer applied, the conventions the SDK had to
//! choose, and every warning. The cache key a consumer uses is
//! `(input_hash, settings_hash, calculation_version)`.

use core::fmt;

use sha2::{Digest, Sha256};

/// A semantic version.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Version {
    /// Major.
    pub major: u16,
    /// Minor.
    pub minor: u16,
    /// Patch.
    pub patch: u16,
}

impl Version {
    /// A version.
    #[must_use]
    pub const fn new(major: u16, minor: u16, patch: u16) -> Version {
        Version {
            major,
            minor,
            patch,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// A SHA-256, rendered as sixty-four hex digits.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hash([u8; 32]);

impl Hash {
    /// The hash of some bytes.
    #[must_use]
    pub fn of(bytes: &[u8]) -> Hash {
        Hash(Sha256::digest(bytes).into())
    }

    /// The bytes.
    #[must_use]
    pub const fn bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// From sixty-four hex digits.
    #[must_use]
    pub fn from_hex(hex: &str) -> Option<Hash> {
        if hex.len() != 64 {
            return None;
        }
        let mut out = [0u8; 32];
        for (i, slot) in out.iter_mut().enumerate() {
            let pair = hex.get(2 * i..2 * i + 2)?;
            *slot = u8::from_str_radix(pair, 16).ok()?;
        }
        Some(Hash(out))
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash({self})")
    }
}

impl serde::Serialize for Hash {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> serde::Deserialize<'de> for Hash {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Hash, D::Error> {
        let text = String::deserialize(deserializer)?;
        Hash::from_hex(&text).ok_or_else(|| serde::de::Error::custom("a SHA-256 is 64 hex digits"))
    }
}

/// What the provider was.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProviderStamp {
    /// The provider's name.
    pub name: String,
    /// Its version.
    pub version: String,
    /// What identifies its data.
    pub data_version: String,
    /// The hashes of the data files it read.
    pub data_hashes: Vec<(String, Hash)>,
    /// The tier, when the provider has tiers.
    pub tier: Option<String>,
    /// The frame it returned, as its key.
    pub frame: String,
    /// The flags actually used, as names.
    pub flags_used: Vec<String>,
}

/// A pack that took part.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PackStamp {
    /// The pack id.
    pub id: String,
    /// Its version.
    pub version: String,
    /// Its content hash.
    pub hash: Hash,
}

/// What the time layer applied.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TimeStamp {
    /// The Delta T model's key.
    pub delta_t_model: String,
    /// Delta T in seconds at the instant.
    pub delta_t_seconds: f64,
    /// The leap-second table version.
    pub leap_table: String,
    /// The tzdb version.
    pub tzdb_version: String,
    /// What was applied when tzdb had no rule (`LMT before 1880` and the like).
    pub time_basis_applied: Option<String>,
    /// Delta T's uncertainty in seconds for deep-time instants.
    pub uncertainty_seconds: Option<f64>,
}

/// How a calendar date was resolved (`docs/03-design/calendar-bikram-sambat.md`).
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum CalendarResolution {
    /// A mathematical definition; exact by construction.
    Defined,
    /// From the authority's published table.
    Tabular {
        /// The authority.
        authority: String,
        /// The table's edition.
        edition: String,
    },
    /// Computed outside the table's range.
    Computed {
        /// The rule or model.
        model: String,
    },
    /// Inside the range and the two disagree; the table was followed.
    Divergent {
        /// The table's day.
        tabular: u8,
        /// The computed day.
        computed: u8,
    },
}

/// A classical model answered rather than the modern one.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Deviation {
    /// The model (`SURYA_SIDDHANTA`).
    pub model: String,
    /// What it changed.
    pub detail: String,
}

/// A convention the SDK had to choose to terminate.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Convention {
    /// The knob.
    pub knob: String,
    /// The value applied.
    pub value: String,
    /// Why.
    pub reason: String,
}

/// Whether every row that took part is verified (ADR-0018).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    /// Every table row used is verified.
    #[default]
    Verified,
    /// At least one traditional row took part.
    Unverified,
}

/// A fallback that was used.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Fallback {
    /// What fell back.
    pub what: String,
    /// Why.
    pub reason: String,
}

/// A warning: a code, and a message key with slots for localisation.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Warning {
    /// The stable code (`DEPRECATED_KEY`).
    pub code: String,
    /// The message key, when there is one.
    pub key: Option<String>,
    /// The slots.
    pub slots: Vec<(String, String)>,
}

/// Everything that reproduces a result.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Provenance {
    /// The SDK's version.
    pub sdk_version: Version,
    /// The versions of the modules that took part.
    pub module_versions: Vec<(String, Version)>,
    /// The calculation version (ADR-0020).
    pub calculation_version: u32,
    /// The catalogue version.
    pub catalogue_version: u32,
    /// The profile id.
    pub profile: String,
    /// The hash of the resolved settings.
    pub settings_hash: Hash,
    /// The hash of the input.
    pub input_hash: Hash,
    /// The provider.
    pub provider: ProviderStamp,
    /// The packs.
    pub packs: Vec<PackStamp>,
    /// What the time layer applied.
    pub time: TimeStamp,
    /// The calendar resolution, when a calendar took part.
    pub calendar: Option<CalendarResolution>,
    /// A classical model's deviation, when one answered.
    pub deviation: Option<Deviation>,
    /// The conventions applied.
    pub applied_conventions: Vec<Convention>,
    /// The confidence.
    pub confidence: Confidence,
    /// The fallbacks used.
    pub fallbacks_used: Vec<Fallback>,
    /// The warnings.
    pub warnings: Vec<Warning>,
    /// The hash of the canonical serialisation of the value.
    pub content_hash: Hash,
}

impl Provenance {
    /// A provenance with the identifying fields set and the rest empty.
    #[must_use]
    pub fn new(
        sdk_version: Version,
        calculation_version: u32,
        catalogue_version: u32,
        profile: impl Into<String>,
        settings_hash: Hash,
        input_hash: Hash,
    ) -> Provenance {
        Provenance {
            sdk_version,
            module_versions: Vec::new(),
            calculation_version,
            catalogue_version,
            profile: profile.into(),
            settings_hash,
            input_hash,
            provider: ProviderStamp::default(),
            packs: Vec::new(),
            time: TimeStamp::default(),
            calendar: None,
            deviation: None,
            applied_conventions: Vec::new(),
            confidence: Confidence::Verified,
            fallbacks_used: Vec::new(),
            warnings: Vec::new(),
            content_hash: Hash::of(&[]),
        }
    }

    /// The cache key every binding documents.
    #[must_use]
    pub fn cache_key(&self) -> (Hash, Hash, u32) {
        (
            self.input_hash,
            self.settings_hash,
            self.calculation_version,
        )
    }

    /// Records a convention.
    pub fn convention(&mut self, knob: &str, value: &str, reason: &str) {
        self.applied_conventions.push(Convention {
            knob: knob.to_string(),
            value: value.to_string(),
            reason: reason.to_string(),
        });
    }

    /// Records a warning.
    pub fn warn(&mut self, code: &str, key: Option<&str>, slots: Vec<(String, String)>) {
        self.warnings.push(Warning {
            code: code.to_string(),
            key: key.map(str::to_string),
            slots,
        });
    }
}

/// A value with its provenance.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Envelope<T> {
    /// The value.
    pub value: T,
    /// How it was produced.
    pub provenance: Provenance,
}

impl<T> Envelope<T> {
    /// Wraps a value.
    #[must_use]
    pub const fn new(value: T, provenance: Provenance) -> Envelope<T> {
        Envelope { value, provenance }
    }

    /// Maps the value, keeping the provenance.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Envelope<U> {
        Envelope {
            value: f(self.value),
            provenance: self.provenance,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;

    #[test]
    fn hashes_versions_and_the_envelope_round_trip() {
        let hash = Hash::of(b"teistro");
        assert_eq!(hash.to_string().len(), 64);
        assert_eq!(Hash::from_hex(&hash.to_string()), Some(hash));
        assert_eq!(Hash::from_hex("xyz"), None);
        assert_eq!(Version::new(1, 2, 3).to_string(), "1.2.3");
        let mut provenance =
            Provenance::new(Version::new(0, 1, 0), 1, 1, "nepali-default", hash, hash);
        provenance.convention(
            "polar_policy",
            "FALLBACK_WHOLE_SIGN",
            "Placidus undefined at 70N",
        );
        provenance.warn(
            "DEPRECATED_KEY",
            Some("sdk.warning.deprecatedKey"),
            vec![("key".into(), "VISHKAMBA".into())],
        );
        provenance.calendar = Some(CalendarResolution::Divergent {
            tabular: 30,
            computed: 31,
        });
        let envelope = Envelope::new(42u32, provenance.clone());
        let json = serde_json::to_string(&envelope).unwrap_or_default();
        let back: Envelope<u32> = serde_json::from_str(&json).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(back, envelope);
        assert_eq!(back.provenance.cache_key(), (hash, hash, 1));
        assert_eq!(envelope.map(|v| v * 2).value, 84);
        assert!(json.contains("\"kind\":\"divergent\""));
    }
}
