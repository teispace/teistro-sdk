//! The `.se1` ephemeris file family that both licensed engines read:
//! what a data directory holds, what it covers, and its content hashes for
//! the provenance envelope. Shared by the adapters so the two agree on the
//! same coverage and the same hashes for the same directory.
//!
//! File names encode a 600-year block: `sepl_18.se1` holds the planets
//! for 1800 to 2399, `semo_18.se1` the Moon, `seas_18.se1` the asteroids;
//! `seplm06.se1` is the block before year 0.

use std::path::Path;

use serde::Serialize;

use crate::model::{AyanamshaId, DataHash};

/// What a data directory offers.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SeFiles {
    /// The files found, sorted by name.
    pub names: Vec<String>,
    /// Their hashes.
    pub hashes: Vec<DataHash>,
    /// Coverage in Julian Days: the union of the planet blocks present,
    /// which every body needs; `None` when no planet block is present.
    pub jd_range: Option<(f64, f64)>,
    /// Whether the fixed-star catalogue (`sefstars.txt`) is present, which
    /// the star-anchored ayanamshas need.
    pub star_catalogue: bool,
}

impl SeFiles {
    /// The ayanamshas the directory supports: the formula-defined ones
    /// always, True Chitra only with the star catalogue.
    #[must_use]
    pub fn ayanamshas(&self) -> Vec<AyanamshaId> {
        let mut ids = vec![
            AyanamshaId::LAHIRI,
            AyanamshaId::RAMAN,
            AyanamshaId::KRISHNAMURTI,
        ];
        if self.star_catalogue {
            ids.push(AyanamshaId::TRUE_CHITRA);
        }
        ids
    }
}

/// The years a block file covers, from its name: `(first year, last year)`.
#[must_use]
pub fn block_years(name: &str) -> Option<(i32, i32)> {
    let stem = name.strip_suffix(".se1")?;
    let (prefix, digits) = stem.split_at(stem.len().checked_sub(2)?);
    let block: i32 = digits.parse().ok()?;
    let negative = prefix.ends_with('m');
    let kind = prefix.trim_end_matches('m').trim_end_matches('_');
    if !matches!(kind, "sepl" | "semo" | "seas") {
        return None;
    }
    let start = if negative {
        -(block * 100) - 600
    } else {
        block * 100
    };
    Some((start, start + 599))
}

/// The Julian Day at 0h UT of a proleptic Gregorian date (Meeus, chapter 7).
#[must_use]
pub fn julian_day(year: i32, month: u32, day: u32) -> f64 {
    let (y, m) = if month <= 2 {
        (year - 1, month + 12)
    } else {
        (year, month)
    };
    let a = y.div_euclid(100);
    let b = 2 - a + a.div_euclid(4);
    let yd = f64::from(y);
    let md = f64::from(m);
    (365.25 * (yd + 4716.0)).floor()
        + (30.6001 * (md + 1.0)).floor()
        + f64::from(day)
        + f64::from(b)
        - 1524.5
}

/// Scans a directory for the family and hashes what it finds.
///
/// # Errors
///
/// When the directory cannot be read or a file cannot be hashed.
pub fn scan(dir: &Path) -> std::io::Result<SeFiles> {
    let mut names: Vec<String> = std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| block_years(name).is_some())
        .collect();
    names.sort();
    let hashes = names
        .iter()
        .map(|name| DataHash::of_file(&dir.join(name)))
        .collect::<std::io::Result<Vec<DataHash>>>()?;
    let mut range: Option<(f64, f64)> = None;
    for name in names.iter().filter(|n| n.starts_with("sepl")) {
        if let Some((first, last)) = block_years(name) {
            let lo = julian_day(first, 1, 1);
            let hi = julian_day(last, 12, 31);
            range = Some(range.map_or((lo, hi), |(a, b)| (a.min(lo), b.max(hi))));
        }
    }
    Ok(SeFiles {
        names,
        hashes,
        jd_range: range,
        star_catalogue: dir.join("sefstars.txt").is_file(),
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, reason = "a test fails by panicking")]

    use super::*;

    #[test]
    fn block_names_decode_to_years() {
        assert_eq!(block_years("sepl_18.se1"), Some((1800, 2399)));
        assert_eq!(block_years("semo_24.se1"), Some((2400, 2999)));
        assert_eq!(block_years("seas_06.se1"), Some((600, 1199)));
        assert_eq!(block_years("seplm06.se1"), Some((-1200, -601)));
        assert_eq!(block_years("sefstars.txt"), None);
    }

    #[test]
    fn julian_day_matches_the_reference_dates() {
        assert!((julian_day(2000, 1, 1) - 2_451_544.5).abs() < 1e-9);
        assert!((julian_day(1800, 1, 1) - 2_378_496.5).abs() < 1e-9);
        assert!((julian_day(2399, 12, 31) - 2_597_640.5).abs() < 1e-9);
    }
}
