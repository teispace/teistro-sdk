//! The engine's data, block by block: which files serve which years, how
//! big each one is, and the digest each must have.
//!
//! The engine carries 1800 to 2399 in its own binary (`TM_EMBED_TIER`
//! `MODERN`, the build every package ships), and that is what a provider
//! reads when nothing else is named. Any other span is a **range** an app
//! adds: a 600-year block, served by two files (`sepl_*` the planets,
//! `semo_*` the Moon) and a third for the asteroids (`seas_*`), which
//! only a chart that names Chiron, Ceres, Pallas, Juno or Vesta reads. An
//! app downloads the block's files from wherever it keeps them and hands
//! them to [`crate::Data`] by path or as bytes; each is checked here
//! first, so a truncated or altered download is refused by name instead
//! of being read.
//!
//! The manifest is `data/ranges.json`, written by
//! `teistro-ephemeris-teimeris-ranges` from the engine's whole published
//! set (13200 BC to AD 17191) and compiled into the adapter, so a check
//! reads no file. Its sizes are published in
//! `docs/03-design/teimeris-ranges-measured.md`.

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use teistro_port_ephemeris::{DataHash, ProviderError, sefile};

/// The manifest's schema, which a reader refuses when it differs.
pub const SCHEMA: &str = "teistro.teimeris-ranges/1";

/// The manifest as `data/ranges.json` holds it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// [`SCHEMA`].
    pub schema: String,
    /// Every block file, ordered by the block's first year and then by
    /// name.
    pub files: Vec<Published>,
}

/// One published file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Published {
    /// Its name, which is what the engine resolves: `sepl_12.se1`.
    pub file: String,
    /// Its size in bytes.
    pub bytes: u64,
    /// Its SHA-256, lower-case hex.
    pub sha256: String,
    /// The first instant its own header records, a Julian day.
    pub jd_start: f64,
    /// The last.
    pub jd_end: f64,
}

impl Manifest {
    /// The manifest of a set of hashed files, each with the span its
    /// header records: the block files among them, in the manifest's
    /// order.
    #[must_use]
    pub fn of(schema: &str, files: impl IntoIterator<Item = (DataHash, (f64, f64))>) -> Manifest {
        let mut files: Vec<Published> = files
            .into_iter()
            .filter(|(hash, _)| sefile::block_years(&hash.file).is_some())
            .map(|(hash, (jd_start, jd_end))| Published {
                file: hash.file,
                bytes: hash.bytes,
                sha256: hash.sha256,
                jd_start,
                jd_end,
            })
            .collect();
        files.sort_by_cached_key(|published| {
            let first = sefile::block_years(&published.file).map_or(i32::MAX, |(first, _)| first);
            (first, published.file.clone())
        });
        Manifest {
            schema: schema.to_owned(),
            files,
        }
    }
}

/// The manifest compiled into the adapter.
///
/// # Panics
///
/// Never on a built adapter: `data/ranges.json` is read by this crate's
/// own tests, which fail on a manifest that does not parse.
#[must_use]
#[allow(
    clippy::expect_used,
    reason = "the manifest is compiled in and held to parse by the tests"
)]
pub fn manifest() -> &'static Manifest {
    static MANIFEST: OnceLock<Manifest> = OnceLock::new();
    MANIFEST.get_or_init(|| {
        serde_json::from_str(include_str!("../data/ranges.json"))
            .expect("data/ranges.json is the manifest the tests read")
    })
}

/// The published file of this name.
#[must_use]
pub fn published(file: &str) -> Option<&'static Published> {
    manifest()
        .files
        .iter()
        .find(|published| published.file == file)
}

/// Checks a file against the manifest: the published file of its name,
/// of the same size, with the same digest.
///
/// # Errors
///
/// [`ProviderError::Invalid`] naming the file, what it is, and what the
/// published file is: a name the engine does not publish, or a size or
/// digest that differ, which is a download cut short or a file altered.
pub fn verify(hash: &DataHash) -> Result<(), ProviderError> {
    let Some(published) = published(&hash.file) else {
        return Err(ProviderError::invalid(format!(
            "`{}` is not a data file the engine publishes; its blocks are named \
             `sepl_18.se1`, `semo_18.se1`, `seas_18.se1` and so on \
             (docs/03-design/teimeris-ranges-measured.md lists them)",
            hash.file
        )));
    };
    if published.bytes == hash.bytes && published.sha256 == hash.sha256 {
        return Ok(());
    }
    Err(ProviderError::invalid(format!(
        "`{}` is {} bytes with SHA-256 {}, and the published file is {} bytes with SHA-256 {}: \
         download it again (a short file is a download cut off)",
        hash.file, hash.bytes, hash.sha256, published.bytes, published.sha256
    )))
}

/// A set's coverage narrowed to what its outermost files record: the
/// earliest and latest published blocks hold less than their names' 600
/// years (the first begins in 12999 BC, not 13200 BC), and a set holding
/// either claims only what is there. A file that is not the published one
/// keeps the span its name gives, since nothing is known of its header.
#[must_use]
pub fn recorded(files: &sefile::SeFiles, (low, high): (f64, f64)) -> (f64, f64) {
    let centuries: Vec<i32> = files
        .names
        .iter()
        .filter(|name| name.starts_with(PLANETS))
        .filter_map(|name| sefile::block_years(name))
        .map(|(first, _)| first / 100)
        .collect();
    let (Some(&first), Some(&last)) = (centuries.iter().min(), centuries.iter().max()) else {
        return (low, high);
    };
    // The published copy of one of a block's two needed files, when the
    // set holds exactly that file.
    let held = |century: i32| {
        [PLANETS, MOON].into_iter().filter_map(move |stem| {
            let name = sefile::block_name(stem, century);
            let hash = files.hashes.iter().find(|hash| hash.file == name)?;
            published(&name).filter(|p| p.sha256 == hash.sha256 && p.bytes == hash.bytes)
        })
    };
    let low = held(first).fold(low, |low, p| low.max(p.jd_start));
    let high = held(last).fold(high, |high, p| high.min(p.jd_end));
    (low, high)
}

/// One 600-year block of the engine's data, named by its first year.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Block {
    /// The first century: 18 for 1800 to 2399, -6 for -600 to -1.
    century: i32,
}

/// The planets' stem, which every body needs.
const PLANETS: &str = "sepl";
/// The Moon's stem.
const MOON: &str = "semo";
/// The asteroids' stem, read only for Chiron, Ceres, Pallas, Juno and
/// Vesta.
const ASTEROIDS: &str = "seas";

impl Block {
    /// The block the engine builds in: 1800 to 2399.
    pub const BUILT_IN: Block = Block { century: 18 };

    /// Every block the engine publishes, earliest first: each with its
    /// planet and Moon files in the manifest.
    #[must_use]
    pub fn all() -> Vec<Block> {
        let mut blocks: Vec<Block> = manifest()
            .files
            .iter()
            .filter(|published| published.file.starts_with(PLANETS))
            .filter_map(|published| sefile::block_years(&published.file))
            .map(|(first, _)| Block {
                century: first / 100,
            })
            .filter(|block| published(&block.file(MOON)).is_some())
            .collect();
        blocks.sort_unstable();
        blocks.dedup();
        blocks
    }

    /// The block that serves this year (astronomical: 1 BC is year 0), when
    /// the engine publishes one.
    #[must_use]
    pub fn of_year(year: i32) -> Option<Block> {
        let century = year.div_euclid(600) * 6;
        Block::all()
            .into_iter()
            .find(|block| block.century == century)
    }

    /// The block that serves this instant (a Julian day, UT), when the
    /// engine publishes one: the block whose first day is the last one at
    /// or before it, each first day read in the calendar the engine names
    /// blocks by.
    #[must_use]
    pub fn of_jd(jd: f64) -> Option<Block> {
        let blocks = Block::all();
        let at = blocks.partition_point(|block| block.first_jd() <= jd);
        let block = *blocks.get(at.checked_sub(1)?)?;
        (jd < block.end_jd()).then_some(block)
    }

    /// The blocks that serve every instant from `from` to `to` (Julian
    /// days, UT), earliest first; `None` when part of the span is outside
    /// what the engine publishes.
    #[must_use]
    pub fn spanning(from: f64, to: f64) -> Option<Vec<Block>> {
        let (low, high) = if from <= to { (from, to) } else { (to, from) };
        let first = Block::of_jd(low)?;
        let last = Block::of_jd(high)?;
        Some(
            Block::all()
                .into_iter()
                .filter(|block| (first..=last).contains(block))
                .collect(),
        )
    }

    /// The first and last years it serves.
    #[must_use]
    pub fn years(self) -> (i32, i32) {
        let first = self.century * 100;
        (first, first + 599)
    }

    /// The first instant it serves, a Julian day: its first year's first
    /// day, or the later first instant its two files record, which only
    /// the earliest block has.
    #[must_use]
    pub fn first_jd(self) -> f64 {
        let [planets, moon] = self.required();
        sefile::civil_julian_day(self.years().0, 1, 1)
            .max(planets.jd_start)
            .max(moon.jd_start)
    }

    /// The instant it stops serving: the first day of the block after it,
    /// or the earlier last instant its two files record, which only the
    /// latest block has.
    #[must_use]
    pub fn end_jd(self) -> f64 {
        let [planets, moon] = self.required();
        sefile::civil_julian_day(self.years().1 + 1, 1, 1)
            .min(planets.jd_end)
            .min(moon.jd_end)
    }

    /// The files it needs for every body but the five asteroids: the
    /// planets and the Moon.
    #[must_use]
    pub fn required(self) -> [&'static Published; 2] {
        self.required_or_none()
            .unwrap_or_else(|| unreachable!("a block is built from the manifest's pairs"))
    }

    /// Its asteroid file, when the engine publishes one for it.
    #[must_use]
    pub fn asteroids(self) -> Option<&'static Published> {
        published(&self.file(ASTEROIDS))
    }

    /// Every file of it: the two it needs, then the asteroids'.
    #[must_use]
    pub fn files(self) -> Vec<&'static Published> {
        let mut files = self.required().to_vec();
        files.extend(self.asteroids());
        files
    }

    /// What its two needed files weigh together, in bytes.
    #[must_use]
    pub fn required_bytes(self) -> u64 {
        self.required()
            .iter()
            .map(|published| published.bytes)
            .sum()
    }

    /// The file of this block under a stem.
    fn file(self, stem: &str) -> String {
        sefile::block_name(stem, self.century)
    }

    /// The two needed files, when both are published.
    fn required_or_none(self) -> Option<[&'static Published; 2]> {
        Some([
            published(&self.file(PLANETS))?,
            published(&self.file(MOON))?,
        ])
    }
}

impl core::fmt::Display for Block {
    /// `1800–2399`, the years as a reader writes them, with no year 0:
    /// the block of astronomical years -600 to -1 is `601 BC–2 BC`.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let year = |year: i32| {
            if year <= 0 {
                format!("{} BC", 1 - year)
            } else {
                year.to_string()
            }
        };
        let (first, last) = self.years();
        write!(f, "{}–{}", year(first), year(last))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    #[test]
    fn the_manifest_parses_and_covers_the_published_span() {
        let manifest = manifest();
        assert_eq!(manifest.schema, SCHEMA);
        let blocks = Block::all();
        assert_eq!(blocks.first().map(|b| b.years().0), Some(-13_200));
        assert_eq!(blocks.last().map(|b| b.years().0), Some(16_800));
        // Contiguous, 600 years apart.
        for pair in blocks.windows(2) {
            if let [a, b] = pair {
                assert_eq!(b.years().0 - a.years().0, 600, "{a} then {b}");
            }
        }
        assert!(blocks.contains(&Block::BUILT_IN));
        // In the manifest's own order, so a rewrite of it diffs cleanly.
        let reordered = Manifest::of(
            SCHEMA,
            manifest.files.iter().rev().map(|p| {
                let hash = DataHash {
                    file: p.file.clone(),
                    sha256: p.sha256.clone(),
                    bytes: p.bytes,
                };
                (hash, (p.jd_start, p.jd_end))
            }),
        );
        assert_eq!(&reordered, manifest);
    }

    /// The premise the coverage rests on: every block but the two at the
    /// ends holds its whole nominal span in both its files, so the span of
    /// a run of blocks is its ends', and only the ends are clamped.
    #[test]
    fn every_inner_block_holds_its_nominal_years() {
        let blocks = Block::all();
        for (index, block) in blocks.iter().enumerate() {
            let (first, last) = block.years();
            let nominal = (
                sefile::civil_julian_day(first, 1, 1),
                sefile::civil_julian_day(last + 1, 1, 1),
            );
            let clamped = (block.first_jd(), block.end_jd());
            if index == 0 {
                assert!(clamped.0 > nominal.0, "{block} starts where its name does");
                continue;
            }
            if index + 1 == blocks.len() {
                assert!(clamped.1 < nominal.1, "{block} ends where its name does");
                continue;
            }
            assert_eq!(clamped, nominal, "{block}");
        }
    }

    #[test]
    fn the_engine_s_own_files_are_the_published_ones() {
        for carried in teimeris::Context::embedded_files() {
            if sefile::block_years(&carried.name).is_none() {
                continue;
            }
            let hash = DataHash {
                file: carried.name.clone(),
                sha256: carried.sha256.to_owned(),
                bytes: carried.size as u64,
            };
            verify(&hash).unwrap_or_else(|e| panic!("{e}"));
        }
    }

    #[test]
    fn a_file_that_is_not_the_published_one_is_refused_by_name() {
        let good = published("sepl_12.se1").unwrap();
        let cut = DataHash {
            file: good.file.clone(),
            sha256: good.sha256.clone(),
            bytes: good.bytes - 1,
        };
        let refusal = verify(&cut).unwrap_err().to_string();
        assert!(
            refusal.contains("sepl_12.se1") && refusal.contains("again"),
            "{refusal}"
        );
        let unknown = DataHash {
            file: String::from("sepl_19.se1"),
            sha256: String::new(),
            bytes: 0,
        };
        assert!(
            verify(&unknown)
                .unwrap_err()
                .to_string()
                .contains("not a data file")
        );
    }

    #[test]
    fn a_block_is_found_by_its_year_and_its_instant() {
        let block = |year| Block::of_year(year).unwrap();
        assert_eq!(block(2026), Block::BUILT_IN);
        assert_eq!(block(1800), Block::BUILT_IN);
        assert_eq!(block(1799).years(), (1200, 1799));
        assert_eq!(block(-1).years(), (-600, -1));
        assert_eq!(block(0).years(), (0, 599));
        assert_eq!(Block::of_year(17_400), None);
        assert_eq!(Block::of_jd(2_461_000.5), Some(Block::BUILT_IN));
        // The block's first day is its own, and the day before is the
        // block before's, on each side of the calendar reform.
        for b in [block(1800), block(1200), block(-600)] {
            assert_eq!(Block::of_jd(b.first_jd()), Some(b));
            assert_ne!(Block::of_jd(b.first_jd() - 0.5), Some(b));
        }
        let span = Block::spanning(block(1500).first_jd() + 10.0, 2_461_000.5).unwrap();
        assert_eq!(span, [block(1200), Block::BUILT_IN]);
        assert_eq!(Block::spanning(2_461_000.5, 9.0e9), None);
    }

    #[test]
    fn a_set_claims_what_its_outermost_files_record() {
        let set = |blocks: &[Block]| {
            sefile::SeFiles::of(
                blocks
                    .iter()
                    .flat_map(|block| block.required())
                    .map(|p| DataHash {
                        file: p.file.clone(),
                        sha256: p.sha256.clone(),
                        bytes: p.bytes,
                    }),
                false,
            )
        };
        let claim = |blocks: &[Block]| {
            let files = set(blocks);
            files.jd_range.map(|range| recorded(&files, range))
        };
        let all = Block::all();
        let (Some(&earliest), Some(&latest)) = (all.first(), all.last()) else {
            panic!("no blocks");
        };
        let middle = Block::of_year(1500).unwrap();
        // Inner blocks claim their names' years; the outermost, their files'.
        assert_eq!(
            claim(&[middle, Block::BUILT_IN]),
            set(&[middle, Block::BUILT_IN]).jd_range
        );
        let whole = claim(&all).unwrap();
        assert_eq!(whole, (earliest.first_jd(), latest.end_jd().min(whole.1)));
        assert!(whole.0 > set(&all).jd_range.unwrap().0);
        assert!(whole.1 < set(&all).jd_range.unwrap().1);
        // A copy that is not the published file keeps its name's span.
        let mut altered = set(&[earliest]);
        let nominal = altered.jd_range.unwrap();
        for hash in &mut altered.hashes {
            hash.sha256 = String::from("0");
        }
        assert_eq!(recorded(&altered, nominal), nominal);
    }

    #[test]
    fn a_block_names_its_files_and_their_weight() {
        let block = Block::of_year(1500).unwrap();
        let names: Vec<&str> = block.files().iter().map(|p| p.file.as_str()).collect();
        assert_eq!(names, ["sepl_12.se1", "semo_12.se1", "seas_12.se1"]);
        assert_eq!(
            block.required_bytes(),
            block.required().iter().map(|p| p.bytes).sum::<u64>()
        );
        assert_eq!(block.to_string(), "1200–1799");
        assert_eq!(Block::of_year(-1).unwrap().to_string(), "601 BC–2 BC");
        assert_eq!(Block::of_year(0).unwrap().to_string(), "1 BC–599");
    }
}
