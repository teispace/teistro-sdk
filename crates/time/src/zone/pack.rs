//! The zone database as data: a pack of IANA's compiled `TZif`, verified
//! and read at run time, so a government that moves its clocks does not wait
//! for an SDK release (`docs/03-design/zone-packs.md`). [`EmbeddedTzdb`]
//! stays the fallback.
//!
//! [`EmbeddedTzdb`]: super::embedded::EmbeddedTzdb

use std::collections::BTreeMap;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use jiff::tz::TimeZone;
use serde::{Deserialize, Serialize};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::time::UtcOffset;
use teistro_port_timezone::{
    LocalCandidates, LocalSeconds, OffsetInfo, TimeZoneProvider, unknown_zone,
};

use super::engine::{self, ZoneClock};

/// The format a pack names, and the one this SDK reads.
pub const ZONE_PACK_FORMAT: &str = "teistro.zones/1";

/// The largest pack read; IANA's whole compiled set is about 0.4 MiB.
const LARGEST_PACK: usize = 16 * 1024 * 1024;

/// The pack as it is written.
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
struct Written {
    format: String,
    version: String,
    zones: BTreeMap<String, String>,
    #[serde(default)]
    links: BTreeMap<String, String>,
}

/// A zone of the pack: its name as the pack spells it, and its rules.
#[derive(Clone, Debug)]
struct Zone {
    name: String,
    tz: TimeZone,
}

/// A zone database read from a pack.
///
/// ```
/// use teistro_time::{EmbeddedTzdb, TimeZoneProvider, ZonePack};
///
/// // A pack of one zone and one link, here written from the embedded data.
/// let kathmandu = jiff_tzdb::get("Asia/Kathmandu").ok_or("a zone")?.1;
/// let bytes = ZonePack::write("2026f", [("Asia/Kathmandu", kathmandu), ("Asia/Katmandu", kathmandu)]);
/// let pack = ZonePack::from_bytes(&bytes)?;
/// assert_eq!(pack.version(), "2026f");
/// assert!(pack.has_zone("Asia/Katmandu") && !pack.has_zone("Asia/Tokyo"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug)]
pub struct ZonePack {
    version: String,
    /// By lower-cased name: the embedded database reads names ignoring
    /// case, and a pack answers as it does.
    zones: BTreeMap<String, Zone>,
}

/// Whether a version is IANA's: four digits and lower-case letters
/// (`tz-link.html`). Those sort in release order as plain text.
fn is_release(version: &str) -> bool {
    let (year, letters) = version.split_at(version.len().min(4));
    year.len() == 4
        && year.bytes().all(|byte| byte.is_ascii_digit())
        && !letters.is_empty()
        && letters.bytes().all(|byte| byte.is_ascii_lowercase())
}

/// Whether a name can be a zone's: a relative path of the characters IANA's
/// names use.
fn is_zone_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('/')
        && !name.ends_with('/')
        && !name.contains("//")
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'+' | b'-' | b'/'))
}

/// A refusal on one field.
fn refused(message: String, field: String) -> Error {
    Error::invalid_arg(message).with_field(field)
}

impl ZonePack {
    /// A pack's bytes from a release's compiled zones, each `(name, TZif)`.
    /// Zones with the same bytes are held once: the first by name is the
    /// zone and the rest are links to it, so the file does not hold what a
    /// compiled tree's copies repeat.
    ///
    /// ```
    /// use teistro_time::ZonePack;
    ///
    /// let utc = jiff_tzdb::get("UTC").ok_or("a zone")?.1;
    /// let bytes = ZonePack::write("2026f", [("Etc/UTC", utc), ("UTC", utc)]);
    /// let text = String::from_utf8(bytes)?;
    /// assert!(text.contains(r#""links":{"UTC":"Etc/UTC"}"#));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn write<'a>(
        version: &str,
        zones: impl IntoIterator<Item = (&'a str, &'a [u8])>,
    ) -> Vec<u8> {
        let mut sorted: Vec<_> = zones.into_iter().collect();
        sorted.sort_unstable();
        let mut first: BTreeMap<&[u8], &str> = BTreeMap::new();
        let mut written = Written {
            format: ZONE_PACK_FORMAT.to_string(),
            version: version.to_string(),
            zones: BTreeMap::new(),
            links: BTreeMap::new(),
        };
        for (name, bytes) in sorted {
            if let Some(target) = first.get(bytes) {
                written
                    .links
                    .insert(name.to_string(), (*target).to_string());
            } else {
                first.insert(bytes, name);
                written
                    .zones
                    .insert(name.to_string(), STANDARD.encode(bytes));
            }
        }
        teistro_core::envelope::canonical_json(&written).into_bytes()
    }

    /// A pack from its bytes, unverified: for a caller whose own transport
    /// has checked them. Every zone is read here, so a bad row refuses the
    /// pack and never a later answer.
    ///
    /// ```
    /// use teistro_time::ZonePack;
    ///
    /// let refused = ZonePack::from_bytes(br#"{"format":"teistro.zones/9","version":"2026f","zones":{}}"#);
    /// assert_eq!(refused.unwrap_err().field(), Some("format"));
    /// ```
    ///
    /// # Errors
    ///
    /// `LIMIT` on `pack` past 16 MiB; `INVALID_ARG` on `pack` for bytes that
    /// are not UTF-8 or not the record, on `format` for a format this SDK
    /// does not read, on `version` for one that is not IANA's, on `zones`
    /// for none, on `zones.<name>` for a bad name, base64 or `TZif`, and on
    /// `links.<name>` for a link to a name `zones` lacks or a name given
    /// twice.
    pub fn from_bytes(bytes: &[u8]) -> Result<ZonePack, Error> {
        if bytes.len() > LARGEST_PACK {
            return Err(Error::limit(format!(
                "a zone pack of {} bytes is past the {LARGEST_PACK} this SDK reads",
                bytes.len()
            ))
            .with_field("pack"));
        }
        let text = core::str::from_utf8(bytes)
            .map_err(|why| refused(format!("a zone pack is UTF-8 JSON: {why}"), "pack".into()))?;
        let written: Written = teistro_core::strict::read(text, "pack")?;
        if written.format != ZONE_PACK_FORMAT {
            return Err(refused(
                format!(
                    "this SDK reads `{ZONE_PACK_FORMAT}`, not `{}`",
                    written.format
                ),
                "format".into(),
            ));
        }
        if !is_release(&written.version) {
            return Err(refused(
                format!(
                    "`{}` is not an IANA release, a year and letters (`2026f`)",
                    written.version
                ),
                "version".into(),
            ));
        }
        if written.zones.is_empty() {
            return Err(refused("a zone pack holds a zone".into(), "zones".into()));
        }
        let mut zones = BTreeMap::new();
        for (name, encoded) in &written.zones {
            let field = || format!("zones.{name}");
            if !is_zone_name(name) {
                return Err(refused(format!("`{name}` is no zone name"), field()));
            }
            let bytes = STANDARD
                .decode(encoded)
                .map_err(|why| refused(format!("`{name}` is not base64: {why}"), field()))?;
            let tz = TimeZone::tzif(name, &bytes)
                .map_err(|why| refused(format!("`{name}` is not TZif: {why}"), field()))?;
            let zone = Zone {
                name: name.clone(),
                tz,
            };
            if zones.insert(name.to_ascii_lowercase(), zone).is_some() {
                return Err(refused(format!("`{name}` is given twice"), field()));
            }
        }
        for (name, target) in &written.links {
            let field = || format!("links.{name}");
            if !is_zone_name(name) {
                return Err(refused(format!("`{name}` is no zone name"), field()));
            }
            let tz = zones
                .get(&target.to_ascii_lowercase())
                .filter(|zone: &&Zone| written.zones.contains_key(&zone.name))
                .ok_or_else(|| refused(format!("`{target}` is no zone of the pack"), field()))?
                .tz
                .clone();
            let zone = Zone {
                name: name.clone(),
                tz,
            };
            if zones.insert(name.to_ascii_lowercase(), zone).is_some() {
                return Err(refused(format!("`{name}` is given twice"), field()));
            }
        }
        Ok(ZonePack {
            version: written.version,
            zones,
        })
    }

    /// A pack from a signed file: its exact bytes, the publisher's detached
    /// Ed25519 signature over them (64 bytes) and the publisher's public key
    /// (32 bytes), which the caller fixes at its own build. A release before
    /// `at_least`, the newest the caller has applied, is refused, so a
    /// replayed pack cannot roll a correction back; `""` takes any.
    ///
    /// ```
    /// use teistro_time::ZonePack;
    ///
    /// // A key whose y-coordinate, 2, has no curve point is refused before
    /// // any signature is checked.
    /// let mut key = [0_u8; 32];
    /// key[0] = 2;
    /// let refused = ZonePack::from_signed(b"{}", &[0; 64], &key, "").unwrap_err();
    /// assert_eq!(refused.field(), Some("key"));
    /// ```
    ///
    /// # Errors
    ///
    /// As [`teistro_core::signed::verify`] and [`ZonePack::from_bytes`];
    /// `INVALID_ARG` on `at_least` for text that is not a release, and on
    /// `version` for a pack older than it.
    #[cfg(feature = "signed-packs")]
    pub fn from_signed(
        bytes: &[u8],
        signature: &[u8],
        key: &[u8],
        at_least: &str,
    ) -> Result<ZonePack, Error> {
        if !at_least.is_empty() && !is_release(at_least) {
            return Err(refused(
                format!("`{at_least}` is not an IANA release, a year and letters (`2026f`)"),
                "at_least".into(),
            ));
        }
        teistro_core::signed::verify(bytes, signature, key)?;
        let pack = ZonePack::from_bytes(bytes)?;
        if pack.version.as_str() < at_least {
            return Err(refused(
                format!(
                    "release {} is older than {at_least}, the newest applied; a replayed pack cannot roll a correction back",
                    pack.version
                ),
                "version".into(),
            ));
        }
        Ok(pack)
    }

    /// The release it holds (`2026f`).
    ///
    /// ```
    /// use teistro_time::ZonePack;
    ///
    /// let utc = jiff_tzdb::get("UTC").ok_or("a zone")?.1;
    /// let pack = ZonePack::from_bytes(&ZonePack::write("2026f", [("UTC", utc)]))?;
    /// assert_eq!(pack.version(), "2026f");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    fn zone(&self, name: &str) -> Result<&TimeZone, Error> {
        self.zones
            .get(&name.to_ascii_lowercase())
            .map(|zone| &zone.tz)
            .ok_or_else(|| unknown_zone(name, &self.suggest(name)))
    }

    /// The nearest names the pack holds to an unknown one.
    ///
    /// ```
    /// use teistro_time::ZonePack;
    ///
    /// let utc = jiff_tzdb::get("Asia/Tokyo").ok_or("a zone")?.1;
    /// let pack = ZonePack::from_bytes(&ZonePack::write("2026f", [("Asia/Tokyo", utc)]))?;
    /// assert_eq!(pack.suggest("Asia/Tokio"), ["Asia/Tokyo"]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn suggest(&self, name: &str) -> Vec<String> {
        engine::suggest(self.zones.values().map(|zone| zone.name.clone()), name)
    }

    /// A zone of the pack as a clock.
    ///
    /// ```
    /// use teistro_time::{LocalClock, ZonePack};
    ///
    /// let utc = jiff_tzdb::get("UTC").ok_or("a zone")?.1;
    /// let pack = ZonePack::from_bytes(&ZonePack::write("2026f", [("UTC", utc)]))?;
    /// assert_eq!(pack.clock("UTC")?.describe(), "UTC (tzdb 2026f)");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// An unknown zone, with the nearest names as the hint.
    pub fn clock(&self, name: &str) -> Result<ZoneClock, Error> {
        Ok(ZoneClock::new(
            self.zone(name)?.clone(),
            name,
            &self.version,
        ))
    }
}

impl TimeZoneProvider for ZonePack {
    fn version(&self) -> &str {
        &self.version
    }

    fn has_zone(&self, zone: &str) -> bool {
        self.zones.contains_key(&zone.to_ascii_lowercase())
    }

    fn zones(&self) -> Vec<String> {
        self.zones.values().map(|zone| zone.name.clone()).collect()
    }

    fn offset_at(&self, zone: &str, instant: JulianDay<Utc>) -> Result<OffsetInfo, Error> {
        engine::offset_info(self.zone(zone)?, instant)
    }

    fn candidates(&self, zone: &str, local: LocalSeconds) -> Result<LocalCandidates, Error> {
        engine::candidates(self.zone(zone)?, local)
    }

    fn current_offsets(&self, zone: &str) -> Result<Vec<UtcOffset>, Error> {
        engine::current_offsets(self.zone(zone)?, &self.version)
    }
}
