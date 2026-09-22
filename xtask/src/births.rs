//! The conformance corpus's recorded births, **founded by the SDK**.
//!
//! Most passes read a recorded chart's JSON directly, because what they
//! measure is the recording. The passes over the annual chart measure
//! what the SDK itself does *from* a birth, so they found each one first
//! and share the founding here rather than each keeping a copy of it —
//! the annual chart's pass was the first, and the Muntha's the second.

use std::path::Path;

use teistro::catalogue::Graha;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{ChartRequest, Context, Document, UtcOffset};

use crate::rules_corpus::read_json;

/// The recorded charts, one JSON file each.
pub(crate) const CHARTS: &str = "fixtures/baseline/charts";

/// One recorded birth, founded.
pub(crate) struct Birth {
    pub(crate) name: String,
    pub(crate) document: Document,
    /// The zone the birth was recorded in, which the return is cast in too.
    pub(crate) offset: UtcOffset,
    /// The Sun's sidereal longitude at birth, which every return returns to.
    pub(crate) natal_sun_deg: f64,
    /// The lagna the **recording engine** gives, sidereal degrees: not the
    /// SDK's, so a pass can hold the SDK's founding against it.
    pub(crate) recorded_lagna_deg: f64,
    /// The sign that lagna is in, zero-based from Aries, as recorded.
    pub(crate) recorded_lagna_sign: u16,
}

impl Birth {
    /// The birth's own instant.
    pub(crate) fn at(&self) -> f64 {
        self.document.foundation.instant.get()
    }

    /// Where it was born, for founding a return there.
    pub(crate) fn request(&self) -> ChartRequest {
        ChartRequest::at(self.document.foundation.place, self.offset)
    }
}

/// Every recorded birth, founded by the SDK under the conformance profile.
pub(crate) fn births(root: &Path, sdk: &Context) -> Result<Vec<Birth>, String> {
    let directory = root.join(CHARTS);
    let entries = std::fs::read_dir(&directory).map_err(|why| format!("{CHARTS}: {why}"))?;
    let mut paths: Vec<_> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();
    let mut out = Vec::new();
    for path in paths {
        let file = read_json(&path)?;
        let name = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();
        let at = file["input"]["place"].clone();
        let (Some(latitude), Some(longitude)) = (at["latitude"].as_f64(), at["longitude"].as_f64())
        else {
            return Err(format!("{name}: a place without a latitude or a longitude"));
        };
        let altitude = at["altitude_m"].as_f64().unwrap_or(0.0);
        let Some(jd) = file["input"]["resolved"]["jd_ut"].as_f64() else {
            return Err(format!("{name}: no resolved instant"));
        };
        let offset_minutes = file["input"]["resolved"]["tz_offset_min"]
            .as_i64()
            .unwrap_or(0);
        let place = Place::new(
            Latitude::try_new(latitude).map_err(|why| format!("{name}: {why}"))?,
            Longitude::try_new(longitude).map_err(|why| format!("{name}: {why}"))?,
            Altitude::try_new(altitude).map_err(|why| format!("{name}: {why}"))?,
        );
        let offset = UtcOffset::try_from_seconds(
            i32::try_from(offset_minutes * 60).map_err(|why| format!("{name}: {why}"))?,
        )
        .map_err(|why| format!("{name}: {why}"))?;
        let request = ChartRequest::at(place, offset);
        let document = sdk
            .chart()
            .reading(JulianDay::<Utc>::literal(jd), &request)
            .map_err(|why| format!("{name}: founding it: {why}"))?
            .value;
        let sun = document
            .foundation
            .graha(Graha::Sun)
            .ok_or_else(|| format!("{name}: a founded chart places the Sun"))?;
        let natal_sun_deg = sun.longitude_deg;
        let recorded = &file["foundation"]["lagna"];
        let (Some(recorded_lagna_deg), Some(recorded_lagna_sign)) = (
            recorded["sidereal_longitude_deg"].as_f64(),
            recorded["sign_index"]
                .as_u64()
                .and_then(|sign| u16::try_from(sign).ok()),
        ) else {
            return Err(format!("{name}: no recorded lagna"));
        };
        out.push(Birth {
            name,
            document,
            offset,
            natal_sun_deg,
            recorded_lagna_deg,
            recorded_lagna_sign,
        });
    }
    Ok(out)
}
