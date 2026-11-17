//! The packs' zone rules as the SDK's time-zone port (`TimeZoneProvider`):
//! what [`crate::TeigeodataGeo::time_zones`] hands a context, so a place
//! resolved through teigeodata is reckoned by the tzdb release its own
//! packs carry.

use std::collections::HashMap;
use std::sync::Arc;

use teigeodata_engine::{Civil, Engine, Local};
use teistro_core::error::{Error, Status};
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::time::UtcOffset;
use teistro_port_timezone::{
    LocalCandidates, LocalSeconds, OffsetInfo, TimeZoneProvider, unknown_zone,
};

/// Seconds in a day, for a Julian day's distance from the Unix epoch.
const SECONDS_PER_DAY: f64 = 86_400.0;

/// The most names an unknown zone's refusal suggests.
const SUGGESTIONS: usize = 3;

/// The zone rules of a teigeodata engine's packs.
///
/// Each zone is answered by the first pack, by teigeodata's precedence,
/// whose rules hold it; the version is the tzdb release that pack's
/// manifest states. Names are matched as tzdb spells them and, failing
/// that, without regard to case, as the SDK's own zone packs match them.
/// The packs hold `zone.tab`'s zones, not tzdb's links (`UTC`,
/// `US/Eastern`) nor its `Etc` zones: those are unknown here, save the
/// nautical zones `Etc/GMT-12` to `Etc/GMT+12`, which a resolution at sea
/// answers (SR-144) and which these rules answer themselves, each one fixed
/// offset with tzdb's abbreviation (`+06` for `Etc/GMT-6`, `GMT`).
#[derive(Debug)]
pub struct TeigeodataZones {
    engine: Arc<Engine>,
    version: String,
    /// Every zone the rules hold, in byte order.
    names: Vec<String>,
    /// Each name by its lower case, to the name as the rules spell it.
    folded: HashMap<String, usize>,
}

impl TeigeodataZones {
    /// The zone rules of `engine`, or `None` where no pack it holds has
    /// any (`tz_version` names no release, or the rules list no zone).
    ///
    /// # Errors
    ///
    /// `PROVIDER` on `zones` when the packs' rules cannot be listed, such
    /// as rules a pack carries but whose component was not installed.
    pub fn new(engine: Arc<Engine>) -> Result<Option<TeigeodataZones>, Error> {
        let Some(version) = engine.tz_version().map(str::to_owned) else {
            return Ok(None);
        };
        let names: Vec<String> = engine
            .tz_zones()
            .map_err(|why| crate::failed(&why, "zones"))?
            .into_iter()
            .map(str::to_owned)
            .collect();
        if names.is_empty() {
            return Ok(None);
        }
        let mut names = names;
        for hours in -12..=12 {
            let name = nautical_name(hours);
            if !names.contains(&name) {
                names.push(name);
            }
        }
        names.sort();
        let folded = names
            .iter()
            .enumerate()
            .map(|(at, name)| (name.to_ascii_lowercase(), at))
            .collect();
        Ok(Some(TeigeodataZones {
            engine,
            version,
            names,
            folded,
        }))
    }

    /// `zone` as the rules spell it, or the refusal naming the nearest.
    /// A nautical zone is spelt as tzdb spells it.
    fn spelled(&self, zone: &str) -> Result<&str, Error> {
        self.folded
            .get(&zone.to_ascii_lowercase())
            .and_then(|at| self.names.get(*at))
            .map(String::as_str)
            .ok_or_else(|| unknown_zone(zone, &self.suggest(zone)))
    }

    /// The names whose last part (`Kathmandu` of `Asia/Kathmandu`) begins
    /// as the unknown name's last part does, ignoring case, at most
    /// [`SUGGESTIONS`] of them.
    fn suggest(&self, zone: &str) -> Vec<String> {
        let last = |name: &str| name.rsplit('/').next().unwrap_or(name).to_ascii_lowercase();
        let wanted = last(zone);
        let stem = wanted.get(..wanted.len().min(4)).unwrap_or(&wanted);
        if stem.is_empty() {
            return Vec::new();
        }
        self.names
            .iter()
            .filter(|name| last(name).starts_with(stem))
            .take(SUGGESTIONS)
            .cloned()
            .collect()
    }

    /// A nautical zone's fixed offset and abbreviation, where the packs'
    /// rules do not hold the zone themselves.
    fn fixed(&self, zone: &str) -> Option<(i32, String)> {
        let held = self.engine.tz_rules_start(zone).ok().flatten().is_some();
        if held { None } else { nautical(zone) }
    }

    /// The first instant `zone`'s rules cover, in Unix seconds.
    fn rules_start(&self, zone: &str) -> Result<i64, Error> {
        self.engine
            .tz_rules_start(zone)
            .map_err(|why| crate::failed(&why, "zone"))?
            .ok_or_else(|| unknown_zone(zone, &[]))
    }
}

/// The whole Unix second holding `instant`, refused out of the rules'
/// range (years −9,999 to 9,999) by the port's own status. The range is
/// the rules' own: `Civil::from_seconds` refuses what they do not cover.
fn unix_seconds(instant: JulianDay<Utc>) -> Result<i64, Error> {
    let seconds = ((instant.get() - JulianDay::<Utc>::UNIX_EPOCH.get()) * SECONDS_PER_DAY).floor();
    #[allow(
        clippy::cast_possible_truncation,
        reason = "a floored second count; a far one saturates and the range check below refuses it"
    )]
    let whole = seconds as i64;
    match Civil::from_seconds(whole) {
        Ok(_) if seconds.is_finite() => Ok(whole),
        _ => Err(out_of_range(&instant.to_string())),
    }
}

/// The nautical zone `hours` ahead of UTC, as tzdb names it: its sign
/// inverted, `Etc/GMT` at zero.
fn nautical_name(hours: i32) -> String {
    match hours {
        0 => String::from("Etc/GMT"),
        east if east > 0 => format!("Etc/GMT-{east}"),
        west => format!("Etc/GMT+{}", -west),
    }
}

/// The offset of a nautical zone the packs do not hold, in seconds, and
/// its abbreviation; `None` for any other name.
fn nautical(zone: &str) -> Option<(i32, String)> {
    let hours = match zone.strip_prefix("Etc/GMT")? {
        "" => 0,
        west if west.starts_with('+') => west.get(1..)?.parse::<i32>().ok()?,
        east if east.starts_with('-') => -east.get(1..)?.parse::<i32>().ok()?,
        _ => return None,
    };
    // Spelt exactly, as `nautical_name` writes it (no `+06`, no `-0`).
    if !(-12..=12).contains(&hours) || nautical_name(-hours) != zone {
        return None;
    }
    let ahead = -hours;
    let abbreviation = match ahead {
        0 => String::from("GMT"),
        east if east > 0 => format!("+{east:02}"),
        west => format!("-{:02}", -west),
    };
    Some((ahead * 3_600, abbreviation))
}

fn out_of_range(what: &str) -> Error {
    Error::new(
        Status::OutOfRange,
        format!("{what} is outside the zone rules' range (years -9999 to 9999)"),
    )
}

/// A pack's offset in seconds as the port's offset; a pack offset no
/// clock uses (beyond ±25:59:59) is the provider's failure.
fn offset(seconds: i32) -> Result<UtcOffset, Error> {
    UtcOffset::try_from_seconds(seconds).map_err(|why| {
        Error::new(
            Status::Provider,
            format!("teigeodata's rules state an offset no clock keeps: {why}"),
        )
        .with_field("zone")
    })
}

impl TimeZoneProvider for TeigeodataZones {
    fn version(&self) -> &str {
        &self.version
    }

    fn has_zone(&self, zone: &str) -> bool {
        self.folded.contains_key(&zone.to_ascii_lowercase())
    }

    fn zones(&self) -> Vec<String> {
        self.names.clone()
    }

    fn offset_at(&self, zone: &str, instant: JulianDay<Utc>) -> Result<OffsetInfo, Error> {
        let zone = self.spelled(zone)?;
        let seconds = unix_seconds(instant)?;
        if let Some((fixed, abbreviation)) = self.fixed(zone) {
            return Ok(OffsetInfo {
                offset: offset(fixed)?,
                abbreviation,
                is_dst: false,
                before_rules: false,
            });
        }
        let found = self
            .engine
            .tz_offset_at(zone, seconds)
            .map_err(|why| crate::failed(&why, "zone"))?
            .ok_or_else(|| unknown_zone(zone, &[]))?;
        Ok(OffsetInfo {
            offset: offset(found.seconds)?,
            abbreviation: found.abbr().to_owned(),
            is_dst: found.is_dst,
            before_rules: seconds < self.rules_start(zone)?,
        })
    }

    fn candidates(&self, zone: &str, local: LocalSeconds) -> Result<LocalCandidates, Error> {
        let zone = self.spelled(zone)?;
        let civil =
            Civil::from_seconds(local.seconds).map_err(|_| out_of_range(&local.to_string()))?;
        if let Some((fixed, _)) = self.fixed(zone) {
            return Ok(LocalCandidates::Unambiguous {
                offset: offset(fixed)?,
            });
        }
        let found = self
            .engine
            .tz_local_candidates(zone, civil)
            .map_err(|why| crate::failed(&why, "zone"))?
            .ok_or_else(|| unknown_zone(zone, &[]))?;
        Ok(match found {
            Local::Unique(seconds) => LocalCandidates::Unambiguous {
                offset: offset(seconds)?,
            },
            Local::Gap { before, after } => LocalCandidates::Gap {
                before: offset(before)?,
                after: offset(after)?,
            },
            Local::Overlap { earlier, later } => LocalCandidates::Overlap {
                earlier: offset(earlier)?,
                later: offset(later)?,
            },
        })
    }

    /// The offsets in force at noon UTC on the first of January, April,
    /// July and October of the year the tzdb release names (`2026f` is
    /// 2026), sorted, each once: the sampling the SDK's own zone
    /// databases use, so the two agree on what "today's rules" are.
    fn current_offsets(&self, zone: &str) -> Result<Vec<UtcOffset>, Error> {
        let zone = self.spelled(zone)?;
        if let Some((fixed, _)) = self.fixed(zone) {
            return Ok(vec![offset(fixed)?]);
        }
        let year: i32 = self
            .version
            .get(..4)
            .and_then(|digits| digits.parse().ok())
            .unwrap_or(FALLBACK_YEAR);
        let mut offsets = Vec::with_capacity(2);
        for month in [1_u8, 4, 7, 10] {
            let noon = Civil {
                year,
                month,
                day: 1,
                hour: 12,
                minute: 0,
                second: 0,
            };
            let at = noon
                .seconds()
                .map_err(|_| out_of_range(&format!("1 {month} {year}")))?;
            let found = self
                .engine
                .tz_offset_at(zone, at)
                .map_err(|why| crate::failed(&why, "zone"))?
                .ok_or_else(|| unknown_zone(zone, &[]))?;
            let found = offset(found.seconds)?;
            if !offsets.contains(&found) {
                offsets.push(found);
            }
        }
        offsets.sort();
        Ok(offsets)
    }
}

/// The year `current_offsets` reads when the release's name begins with
/// no year: the SDK's own zone databases fall back to the same year
/// (`teistro-time`'s `FALLBACK_VERSION_YEAR`).
const FALLBACK_YEAR: i32 = 2026;
