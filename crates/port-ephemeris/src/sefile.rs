//! The `.se1` ephemeris file family that both licensed engines read:
//! what a data directory holds, what it covers, and its content hashes for
//! the provenance envelope. Shared by the adapters so the two agree on the
//! same coverage and the same hashes for the same directory. This module
//! holds no engine code: file names, sizes and hashes only.
//!
//! File names encode a 600-year block by its first century: `sepl_18.se1`
//! holds the planets for 1800 to 2399, `semo_18.se1` the Moon,
//! `seas_18.se1` the asteroids, `sepl_102.se1` the years from 10200, and
//! `seplm06.se1` the block that ends at year 0, years -600 to -1 (1 BC is
//! year 0). The years are read in the calendar the engines name a block
//! by: Julian before 15 October 1582, Gregorian from then on.

use std::path::Path;

use serde::Serialize;
use teistro_core::catalogue::{Ayanamsha, AyanamshaCategory, Catalogued};

use crate::capabilities::DataHash;

/// What a data directory offers.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SeFiles {
    /// The files found, sorted by name.
    pub names: Vec<String>,
    /// Their hashes.
    pub hashes: Vec<DataHash>,
    /// Coverage in Julian days: the union of the planet blocks present,
    /// which every body needs; `None` when no planet block is present.
    pub jd_range: Option<(f64, f64)>,
    /// Whether the fixed-star catalogue (`sefstars.txt`) is present, which
    /// the star-anchored and galactic ayanamshas need.
    pub star_catalogue: bool,
}

impl SeFiles {
    /// The ayanamshas the directory supports: every catalogued one whose
    /// definition is a formula, and the star-anchored and galactic ones
    /// only with the star catalogue.
    #[must_use]
    pub fn ayanamshas(&self) -> Vec<Ayanamsha> {
        Ayanamsha::all()
            .iter()
            .copied()
            .filter(|a| self.star_catalogue || !needs_star_catalogue(*a))
            .collect()
    }
}

impl SeFiles {
    /// The files this set lacks inside the span it claims, by name: each
    /// planet block missing between the first and the last present, and
    /// the Moon's block for each planet block without one. Empty when
    /// [`SeFiles::jd_range`] is served throughout.
    ///
    /// `jd_range` is the outer edges of the planet blocks, so a set with a
    /// hole claims the hole, and an engine asked inside it answers from
    /// its analytic fallback; a planet block without its Moon does the
    /// same for the Moon. A caller who opens a set names these files in
    /// its refusal, which is all a consumer needs to fetch the rest.
    #[must_use]
    pub fn missing(&self) -> Vec<String> {
        let first_century = |stem: &str| -> Vec<i32> {
            let mut centuries: Vec<i32> = self
                .names
                .iter()
                .filter(|name| name.starts_with(stem))
                .filter_map(|name| block_years(name))
                .map(|(first, _)| first / 100)
                .collect();
            centuries.sort_unstable();
            centuries
        };
        let planets = first_century("sepl");
        let moons = first_century("semo");
        let (Some(&low), Some(&high)) = (planets.first(), planets.last()) else {
            return Vec::new();
        };
        let mut missing = Vec::new();
        for century in (low..=high).step_by(6) {
            if planets.binary_search(&century).is_err() {
                missing.push(block_name("sepl", century));
            }
            if moons.binary_search(&century).is_err() {
                missing.push(block_name("semo", century));
            }
        }
        missing
    }
}

/// The file of one block: `block_name("sepl", 18)` is `sepl_18.se1` and
/// `block_name("semo", -6)` is `semom06.se1`, the naming [`block_years`]
/// reads.
#[must_use]
pub fn block_name(stem: &str, first_century: i32) -> String {
    let sign = if first_century < 0 { 'm' } else { '_' };
    format!("{stem}{sign}{:02}.se1", first_century.unsigned_abs())
}

/// Whether an ayanamsha is defined through a star's position.
#[must_use]
pub fn needs_star_catalogue(ayanamsha: Ayanamsha) -> bool {
    matches!(
        ayanamsha.attributes().category,
        AyanamshaCategory::StarAnchored | AyanamshaCategory::Galactic
    )
}

/// The years a block file covers, from its name: `(first year, last year)`,
/// astronomical years (1 BC is year 0).
///
/// The digits after the stem are the block's first century, two or three
/// of them (`sepl_18`, `sepl_102`); an `m` in place of the underscore
/// counts back from year 0, so `seplm06` begins at year -600. That is the
/// engines' naming rule read backwards: an instant in year `y` is served
/// by the block whose first century is `y / 100` rounded down to a
/// multiple of six.
#[must_use]
pub fn block_years(name: &str) -> Option<(i32, i32)> {
    let stem = name.strip_suffix(".se1")?;
    let (kind, rest) = stem.split_at_checked(4)?;
    if !matches!(kind, "sepl" | "semo" | "seas") {
        return None;
    }
    let (negative, digits) = match rest.split_at_checked(1)? {
        ("_", digits) => (false, digits),
        ("m", digits) => (true, digits),
        _ => return None,
    };
    if !(2..=3).contains(&digits.len()) || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let century: i32 = digits.parse().ok()?;
    if century % 6 != 0 || (negative && century == 0) {
        return None;
    }
    let start = if negative {
        -century * 100
    } else {
        century * 100
    };
    Some((start, start + 599))
}

/// The Julian day the engines' calendar gives a date: the Julian calendar
/// before 15 October 1582 (JD 2299160.5) and the Gregorian from then on,
/// the rule by which an instant is given its block. A block's first and
/// last days are read in it, so the coverage claimed for a block before
/// 1582 is the coverage its file records.
#[must_use]
pub fn civil_julian_day(year: i32, month: u32, day: u32) -> f64 {
    let gregorian = julian_day(year, month, day);
    if gregorian >= GREGORIAN_REFORM {
        gregorian
    } else {
        julian_calendar_day(year, month, day)
    }
}

/// 15 October 1582, the first day of the Gregorian calendar.
const GREGORIAN_REFORM: f64 = 2_299_160.5;

/// The Julian day at 0h UT of a date in the Julian calendar (Meeus,
/// chapter 7, with no century correction).
fn julian_calendar_day(year: i32, month: u32, day: u32) -> f64 {
    let (y, m) = if month <= 2 {
        (year - 1, month + 12)
    } else {
        (year, month)
    };
    (365.25 * (f64::from(y) + 4716.0)).floor()
        + (30.6001 * (f64::from(m) + 1.0)).floor()
        + f64::from(day)
        - 1524.5
}

/// The Julian day at 0h UT of a proleptic Gregorian date (Meeus,
/// *Astronomical Algorithms*, chapter 7), for the block boundaries.
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
    Ok(SeFiles::of(hashes, dir.join("sefstars.txt").is_file()))
}

impl SeFiles {
    /// What a set of files offers, from their hashes alone: the blocks
    /// among them, sorted, their coverage, and whether the star catalogue
    /// is with them.
    ///
    /// [`scan`] reads a directory into this; an engine that carries its
    /// data in its own binary, hashed when it was built, passes those
    /// hashes, so the same files give the same coverage and provenance
    /// however they are carried and nothing is hashed at run time. A file
    /// that is not a block (the star catalogue, the orbital elements) is
    /// left out of `names` and `hashes`, as `scan` leaves it out.
    #[must_use]
    pub fn of(hashes: impl IntoIterator<Item = DataHash>, star_catalogue: bool) -> SeFiles {
        let mut hashes: Vec<DataHash> = hashes
            .into_iter()
            .filter(|hash| block_years(&hash.file).is_some())
            .collect();
        hashes.sort_by(|a, b| a.file.cmp(&b.file));
        let names: Vec<String> = hashes.iter().map(|hash| hash.file.clone()).collect();
        let mut range: Option<(f64, f64)> = None;
        for name in names.iter().filter(|n| n.starts_with("sepl")) {
            if let Some((first, last)) = block_years(name) {
                let lo = civil_julian_day(first, 1, 1);
                let hi = civil_julian_day(last, 12, 31);
                range = Some(range.map_or((lo, hi), |(a, b)| (a.min(lo), b.max(hi))));
            }
        }
        SeFiles {
            names,
            hashes,
            jd_range: range,
            star_catalogue,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    #[test]
    fn block_names_decode_to_years() {
        assert_eq!(block_years("sepl_18.se1"), Some((1800, 2399)));
        assert_eq!(block_years("semo_24.se1"), Some((2400, 2999)));
        assert_eq!(block_years("seas_06.se1"), Some((600, 1199)));
        assert_eq!(block_years("seplm06.se1"), Some((-600, -1)));
        assert_eq!(block_years("semom132.se1"), Some((-13_200, -12_601)));
        assert_eq!(block_years("sepl_00.se1"), Some((0, 599)));
        assert_eq!(block_years("sepl_102.se1"), Some((10_200, 10_799)));
        assert_eq!(block_years("seas_168.se1"), Some((16_800, 17_399)));
        for refused in [
            "sefstars.txt",
            "x.se1",
            "sepl_1.se1",
            "sepl_1020.se1",
            "sepl_19.se1",
            "seplm00.se1",
            "sepl-18.se1",
            "sepl_1a.se1",
            "sepm_18.se1",
            "sepl_+6.se1",
        ] {
            assert_eq!(block_years(refused), None, "{refused}");
        }
    }

    #[test]
    fn a_hole_and_a_moonless_block_are_named() {
        let set = |names: &[&str]| {
            SeFiles::of(
                names.iter().map(|name| DataHash {
                    file: (*name).to_owned(),
                    sha256: String::new(),
                    bytes: 0,
                }),
                false,
            )
        };
        assert_eq!(
            set(&["sepl_18.se1", "semo_18.se1"]).missing(),
            Vec::<String>::new()
        );
        assert_eq!(set(&[]).missing(), Vec::<String>::new());
        assert_eq!(
            set(&["sepl_12.se1", "semo_12.se1", "sepl_24.se1", "semo_24.se1"]).missing(),
            ["sepl_18.se1", "semo_18.se1"]
        );
        assert_eq!(
            set(&["seplm06.se1", "sepl_00.se1", "semo_00.se1"]).missing(),
            ["semom06.se1"]
        );
        for (stem, century) in [("sepl", 18), ("semo", -6), ("seas", 102), ("sepl", 0)] {
            let name = block_name(stem, century);
            assert_eq!(
                block_years(&name).map(|(first, _)| first / 100),
                Some(century),
                "{name}"
            );
        }
    }

    /// The first days of blocks, as the engine's own files record them:
    /// `tfstart` of `seplm06.se1` is JD 1501907.5 and of `sepl_00.se1`
    /// 1721057.5, both 1 January in the Julian calendar, and of
    /// `sepl_18.se1` 2378496.5, 1 January 1800 in the Gregorian.
    #[test]
    fn a_block_begins_where_its_file_does() {
        assert!((civil_julian_day(-600, 1, 1) - 1_501_907.5).abs() < 1e-9);
        assert!((civil_julian_day(0, 1, 1) - 1_721_057.5).abs() < 1e-9);
        assert!((civil_julian_day(1800, 1, 1) - 2_378_496.5).abs() < 1e-9);
        // The calendar changes at the reform, and the days run on.
        assert!((civil_julian_day(1582, 10, 4) - 2_299_159.5).abs() < 1e-9);
        assert!((civil_julian_day(1582, 10, 15) - 2_299_160.5).abs() < 1e-9);
    }

    #[test]
    fn julian_day_matches_the_reference_dates() {
        assert!((julian_day(2000, 1, 1) - 2_451_544.5).abs() < 1e-9);
        assert!((julian_day(1800, 1, 1) - 2_378_496.5).abs() < 1e-9);
        assert!((julian_day(2399, 12, 31) - 2_597_640.5).abs() < 1e-9);
    }

    #[test]
    fn a_directory_without_files_covers_nothing() {
        let dir = std::env::temp_dir().join("teistro-port-ephemeris-sefile-empty");
        std::fs::create_dir_all(&dir).unwrap();
        let files = scan(&dir).unwrap();
        assert!(files.names.is_empty() && files.jd_range.is_none());
        assert!(!files.star_catalogue);
        let formula_only = files.ayanamshas();
        assert!(formula_only.contains(&Ayanamsha::Lahiri));
        assert!(!formula_only.contains(&Ayanamsha::TrueChitra));
        let with_stars = SeFiles {
            star_catalogue: true,
            ..files
        };
        assert!(with_stars.ayanamshas().contains(&Ayanamsha::TrueChitra));
        assert_eq!(with_stars.ayanamshas().len(), Ayanamsha::all().len());
    }

    /// Files carried in an engine's binary give what the same files in a
    /// directory give: the same names, hashes and coverage.
    #[test]
    fn files_described_by_their_hashes_match_the_same_files_scanned() {
        let dir = std::env::temp_dir().join("teistro-port-ephemeris-sefile-of");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for (name, bytes) in [
            ("semo_18.se1", &b"moon"[..]),
            ("sepl_18.se1", b"planets"),
            ("seorbel.txt", b"elements"),
        ] {
            std::fs::write(dir.join(name), bytes).unwrap();
        }
        let scanned = scan(&dir).unwrap();
        let described = SeFiles::of(
            ["seorbel.txt", "sepl_18.se1", "semo_18.se1"]
                .map(|name| DataHash::of_file(&dir.join(name)).unwrap()),
            false,
        );
        assert_eq!(described, scanned);
        assert_eq!(described.names, ["semo_18.se1", "sepl_18.se1"]);
        assert_eq!(
            described.jd_range,
            Some((julian_day(1800, 1, 1), julian_day(2399, 12, 31)))
        );
    }
}
