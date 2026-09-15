//! The falsification pass over the Ashtakavarga (Phase 5).
//!
//! The corpus's `baseline/ashtakavarga` records the recording engine's
//! Ashtakavarga for every fixture with a recorded chart, computed from the
//! lagna's sign and the seven grahas' signs it repeats: each graha's bindus,
//! their sum, the sum after the trine reduction, and under each of the
//! engine's three Ekadhipatya methods the fully reduced sum and the pindas.
//!
//! Here, unlike the dashas, a rank-1 text was read: BPHS chs. 66 to 69 in
//! English translation. It agrees with the engine on the bindu tables and
//! disagrees on three things the engine does after them, so every rule is
//! measured beside the text's:
//!
//! - **where the reductions are made** — the engine reduces the sum of the
//!   seven, the text each graha's own Ashtakavarga (chs. 67 and 69, which
//!   reduce and form pindas in the Ashtakavarga of each of the grahas);
//! - **Ekadhipatya when one sign is occupied** — the engine zeroes the empty
//!   sign; the text keeps the difference when the occupied sign has the
//!   smaller number (ch. 68);
//! - **the pindas** — the engine takes the rashi pinda of the reduced sum and
//!   the graha pinda of a graha's raw bindus times its own measure; the text
//!   takes both of each graha's reduced Ashtakavarga, the graha pinda from
//!   the grahas standing in each sign (ch. 69), and gives Virgo a measure of
//!   6 where the engine has 8.
//!
//! `cargo xtask ashtakavarga` writes the page; `check-ashtakavarga`
//! regenerates it in memory and fails on any difference.

use std::fmt::Write as _;
use std::path::Path;

use serde_json::Value;

use crate::generated::{Output, check, write};
use crate::measure::{Claim, count, fill, table};

const PAGE: &str = "docs/03-design/ashtakavarga-measured.md";
const ROOT: &str = "fixtures/baseline/ashtakavarga";
const GRAHAS: [&str; 7] = [
    "SUN", "MOON", "MARS", "MERCURY", "JUPITER", "VENUS", "SATURN",
];

/// Where each graha gains a bindu, counted from each contributor: the seven
/// grahas in order, then the lagna (BPHS ch. 66, the places not marked with
/// a dot).
const PLACES: [[&[usize]; 8]; 7] = [
    [
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[3, 6, 10, 11],
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[3, 5, 6, 9, 10, 11, 12],
        &[5, 6, 9, 11],
        &[6, 7, 12],
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[3, 4, 6, 10, 11, 12],
    ],
    [
        &[3, 6, 7, 8, 10, 11],
        &[1, 3, 6, 7, 10, 11],
        &[2, 3, 5, 6, 9, 10, 11],
        &[1, 3, 4, 5, 7, 8, 10, 11],
        &[1, 4, 7, 8, 10, 11, 12],
        &[3, 4, 5, 7, 9, 10, 11],
        &[3, 5, 6, 11],
        &[3, 6, 10, 11],
    ],
    [
        &[3, 5, 6, 10, 11],
        &[3, 6, 11],
        &[1, 2, 4, 7, 8, 10, 11],
        &[3, 5, 6, 11],
        &[6, 10, 11, 12],
        &[6, 8, 11, 12],
        &[1, 4, 7, 8, 9, 10, 11],
        &[1, 3, 6, 10, 11],
    ],
    [
        &[5, 6, 9, 11, 12],
        &[2, 4, 6, 8, 10, 11],
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[1, 3, 5, 6, 9, 10, 11, 12],
        &[6, 8, 11, 12],
        &[1, 2, 3, 4, 5, 8, 9, 11],
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[1, 2, 4, 6, 8, 10, 11],
    ],
    [
        &[1, 2, 3, 4, 7, 8, 9, 10, 11],
        &[2, 5, 7, 9, 11],
        &[1, 2, 4, 7, 8, 10, 11],
        &[1, 2, 4, 5, 6, 9, 10, 11],
        &[1, 2, 3, 4, 7, 8, 10, 11],
        &[2, 5, 6, 9, 10, 11],
        &[3, 5, 6, 12],
        &[1, 2, 4, 5, 6, 7, 9, 10, 11],
    ],
    [
        &[8, 11, 12],
        &[1, 2, 3, 4, 5, 8, 9, 11, 12],
        &[3, 5, 6, 9, 11, 12],
        &[3, 5, 6, 9, 11],
        &[5, 8, 9, 10, 11],
        &[1, 2, 3, 4, 5, 8, 9, 10, 11],
        &[3, 4, 5, 8, 9, 10, 11],
        &[1, 2, 3, 4, 5, 8, 9, 11],
    ],
    [
        &[1, 2, 4, 7, 8, 10, 11],
        &[3, 6, 11],
        &[3, 5, 6, 10, 11, 12],
        &[6, 8, 9, 10, 11, 12],
        &[5, 6, 11, 12],
        &[6, 11, 12],
        &[3, 5, 6, 11],
        &[1, 3, 4, 6, 10, 11],
    ],
];

/// The signs a graha rules two of, as pairs (Mars, Venus, Mercury, Jupiter,
/// Saturn).
const PAIRS: [(usize, usize); 5] = [(0, 7), (1, 6), (2, 5), (8, 11), (9, 10)];

/// Each sign's measure: the text's (BPHS ch. 69) and the engine's, which
/// differ at Virgo.
const TEXT_MEASURES: [u32; 12] = [7, 10, 8, 4, 10, 6, 7, 8, 9, 5, 11, 12];
const ENGINE_MEASURES: [u32; 12] = [7, 10, 8, 4, 10, 8, 7, 8, 9, 5, 11, 12];

/// Each graha's measure (BPHS ch. 69), which the engine shares.
const GRAHA_MEASURES: [u32; 7] = [5, 5, 8, 5, 10, 7, 5];

/// One recorded Ashtakavarga.
struct Record {
    lagna: usize,
    signs: [usize; 7],
    bav: [[u32; 12]; 7],
    sav: [u32; 12],
    trikona: [u32; 12],
    methods: Vec<Method>,
}

/// One Ekadhipatya method's recorded answer: its name, the reduced sum, the
/// rashi pinda by sign, and the graha and yoga pindas by graha.
type Method = (String, [u32; 12], [u32; 12], [u32; 7], [u32; 7]);

fn twelve(value: &Value) -> Result<[u32; 12], String> {
    let mut out = [0; 12];
    let items = value
        .as_array()
        .filter(|a| a.len() == 12)
        .ok_or("not twelve")?;
    for (slot, item) in out.iter_mut().zip(items) {
        *slot = item
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or("not a count")?;
    }
    Ok(out)
}

fn seven(value: &Value) -> Result<[u32; 7], String> {
    let mut out = [0; 7];
    for (slot, graha) in out.iter_mut().zip(GRAHAS) {
        *slot = value[graha]
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or("not a count")?;
    }
    Ok(out)
}

fn records(root: &Path) -> Result<Vec<Record>, String> {
    let mut out = Vec::new();
    for dir in ["charts", "variants"] {
        let mut paths: Vec<_> = std::fs::read_dir(root.join(ROOT).join(dir))
            .map_err(|err| format!("{dir}: {err}"))?
            .flatten()
            .map(|entry| entry.path())
            .collect();
        paths.sort();
        for path in paths {
            let at = |err: String| format!("{}: {err}", path.display());
            let file: Value = serde_json::from_str(
                &std::fs::read_to_string(&path).map_err(|e| at(e.to_string()))?,
            )
            .map_err(|e| at(e.to_string()))?;
            let mut bav = [[0; 12]; 7];
            let mut signs = [0; 7];
            for (i, graha) in GRAHAS.iter().enumerate() {
                bav[i] = twelve(&file["bav"][graha]).map_err(&at)?;
                signs[i] = file["inputs"]["graha_sign_index"][graha]
                    .as_u64()
                    .and_then(|n| usize::try_from(n).ok())
                    .ok_or_else(|| at(String::from("sign")))?;
            }
            let methods = file["ekadhipatya"]
                .as_object()
                .ok_or_else(|| at(String::from("ekadhipatya")))?
                .iter()
                .map(|(name, m)| {
                    Ok((
                        name.clone(),
                        twelve(&m["sav_reduced"])?,
                        twelve(&m["rashi_pinda"])?,
                        seven(&m["graha_pinda"])?,
                        seven(&m["yoga_pinda"])?,
                    ))
                })
                .collect::<Result<Vec<_>, String>>()
                .map_err(&at)?;
            out.push(Record {
                lagna: file["inputs"]["lagna_sign_index"]
                    .as_u64()
                    .and_then(|n| usize::try_from(n).ok())
                    .ok_or_else(|| at(String::from("lagna")))?,
                signs,
                bav,
                sav: twelve(&file["sav"]).map_err(&at)?,
                trikona: twelve(&file["sav_trikona"]).map_err(&at)?,
                methods,
            });
        }
    }
    Ok(out)
}

// ── The rules proposed ─────────────────────────────────────────────────────

/// Each graha's bindus by the table.
fn bav(record: &Record) -> [[u32; 12]; 7] {
    let mut out = [[0; 12]; 7];
    let contributors: Vec<usize> = record
        .signs
        .iter()
        .copied()
        .chain(std::iter::once(record.lagna))
        .collect();
    for (graha, places) in PLACES.iter().enumerate() {
        for (from, offsets) in contributors.iter().zip(places) {
            for offset in *offsets {
                out[graha][(from + offset - 1) % 12] += 1;
            }
        }
    }
    out
}

fn sum(rows: &[[u32; 12]; 7]) -> [u32; 12] {
    let mut out = [0; 12];
    for row in rows {
        for (slot, value) in out.iter_mut().zip(row) {
            *slot += value;
        }
    }
    out
}

/// The trine reduction: the least of each trine taken from all three.
fn trikona(values: [u32; 12]) -> [u32; 12] {
    let mut out = values;
    for start in 0..4 {
        let least = (0..3).map(|k| values[start + 4 * k]).min().unwrap_or(0);
        for k in 0..3 {
            out[start + 4 * k] -= least;
        }
    }
    out
}

#[derive(Clone, Copy)]
enum Ekadhipatya {
    /// The engine's: an empty sign beside an occupied one goes to zero.
    EmptyToZero,
    /// The text's: an empty sign beside an occupied one with the smaller
    /// number keeps the difference.
    Text,
}

fn ekadhipatya(values: [u32; 12], occupied: &[bool; 12], rule: Ekadhipatya) -> [u32; 12] {
    let mut out = values;
    for (a, b) in PAIRS {
        let (va, vb) = (values[a], values[b]);
        if va == 0 || vb == 0 {
            continue;
        }
        match (occupied[a], occupied[b]) {
            (true, true) => {}
            (false, false) => {
                let least = va.min(vb);
                let (na, nb) = if va == vb { (0, 0) } else { (least, least) };
                out[a] = na;
                out[b] = nb;
            }
            (occupied_a, _) => {
                let (full, empty) = if occupied_a { (a, b) } else { (b, a) };
                out[empty] = match rule {
                    Ekadhipatya::Text if values[full] < values[empty] => {
                        values[empty] - values[full]
                    }
                    _ => 0,
                };
            }
        }
    }
    out
}

fn occupied(record: &Record) -> [bool; 12] {
    let mut out = [false; 12];
    for sign in record.signs {
        out[sign] = true;
    }
    out
}

fn dot(values: &[u32; 12], measures: &[u32; 12]) -> u32 {
    values.iter().zip(measures).map(|(v, m)| v * m).sum()
}

// ── The page ───────────────────────────────────────────────────────────────

/// The bindus, their sum and the trine reduction.
fn bindu_claims(records: &[Record]) -> Vec<Claim> {
    let charts = records.len();
    let mut claims = Vec::new();

    let wrong = records.iter().filter(|r| bav(r) != r.bav).count();
    claims.push(Claim::counted(
        "each graha's bindus are the table's places counted from the seven grahas and the lagna",
        wrong,
        charts,
    ));
    let totals = [48, 49, 39, 54, 56, 52, 39];
    let wrong = records
        .iter()
        .filter(|r| r.bav.iter().map(|row| row.iter().sum::<u32>()).ne(totals))
        .count();
    claims.push(Claim::counted(
        "whatever the chart, the grahas hold 48, 49, 39, 54, 56, 52 and 39 bindus, 337 in all",
        wrong,
        charts,
    ));
    let wrong = records
        .iter()
        .filter(|r| sum(&r.bav) != r.sav || trikona(r.sav) != r.trikona)
        .count();
    claims.push(Claim::counted(
        "the sum is the seven grahas' bindus by sign, and the trine reduction takes the least of each trine from the **sum**",
        wrong,
        charts,
    ));
    let wrong = records
        .iter()
        .filter(|r| {
            let reduced: [[u32; 12]; 7] = r.bav.map(trikona);
            sum(&reduced) != r.trikona
        })
        .count();
    claims.push(Claim::counted(
        "the same reduction made in each graha's Ashtakavarga and then summed, as the text makes it",
        wrong,
        charts,
    ));
    claims
}

/// The engine's reductions and pindas beside the text's.
fn reduction_claims(records: &[Record]) -> Vec<Claim> {
    let classical: Vec<(&Record, &Method)> = records
        .iter()
        .filter_map(|r| {
            r.methods
                .iter()
                .find(|m| m.0 == "classical")
                .map(|m| (r, m))
        })
        .collect();
    let mut claims = Vec::new();
    for (rule, which) in [
        (
            "the engine's classical Ekadhipatya on the reduced sum: both signs occupied, nothing; both empty, both the smaller number or both zero when equal; one occupied, the empty sign **zero**",
            Ekadhipatya::EmptyToZero,
        ),
        (
            "the text's: the same, except that an empty sign beside an occupied sign with the **smaller** number keeps the difference",
            Ekadhipatya::Text,
        ),
    ] {
        let wrong = classical
            .iter()
            .filter(|(r, m)| ekadhipatya(r.trikona, &occupied(r), which) != m.1)
            .count();
        claims.push(Claim::counted(rule, wrong, classical.len()));
    }
    for (rule, measures) in [
        (
            "the rashi pinda is the reduced sum times each sign's measure, Virgo's **8**",
            ENGINE_MEASURES,
        ),
        (
            "the same with Virgo's measure **6**, as the text gives it",
            TEXT_MEASURES,
        ),
    ] {
        let wrong = classical
            .iter()
            .filter(|(_, m)| {
                m.1.iter()
                    .zip(measures)
                    .map(|(v, x)| v * x)
                    .ne(m.2.iter().copied())
            })
            .count();
        claims.push(Claim::counted(rule, wrong, classical.len()));
    }
    let wrong = classical
        .iter()
        .filter(|(r, m)| {
            (0..7).any(|g| {
                m.3[g] != r.bav[g].iter().sum::<u32>() * GRAHA_MEASURES[g]
                    || m.4[g] != m.2[r.signs[g]] + m.3[g]
            })
        })
        .count();
    claims.push(Claim::counted(
        "a graha's graha pinda is all its raw bindus times its own measure, and its yoga pinda that plus the rashi pinda of the sign it stands in",
        wrong,
        classical.len(),
    ));
    let wrong = classical
        .iter()
        .filter(|(r, m)| {
            let occupied = occupied(r);
            (0..7).any(|g| {
                let reduced = ekadhipatya(trikona(r.bav[g]), &occupied, Ekadhipatya::Text);
                let rashi = dot(&reduced, &TEXT_MEASURES);
                let graha: u32 = (0..7)
                    .map(|other| reduced[r.signs[other]] * GRAHA_MEASURES[other])
                    .sum();
                m.4[g] != rashi + graha
            })
        })
        .count();
    claims.push(Claim::counted(
        "the text's yoga pinda: each graha's own reduced Ashtakavarga times the signs' measures, plus each sign's reduced bindus times the measure of every graha standing in it",
        wrong,
        classical.len(),
    ));
    claims
}

fn claims(records: &[Record]) -> Vec<Claim> {
    let mut claims = bindu_claims(records);
    claims.extend(reduction_claims(records));
    claims
}

fn page(root: &Path) -> Result<String, String> {
    let records = records(root)?;
    if records.is_empty() {
        return Err(String::from("the corpus records no Ashtakavarga"));
    }
    let mut out = String::new();
    let _ = write!(
        out,
        "# The Ashtakavarga, measured\n\n\
         Status: `generated` by `cargo xtask ashtakavarga` over the conformance corpus's \
         `baseline/ashtakavarga`, 2026-09-15. Do not edit: `check-ashtakavarga` regenerates this \
         page and fails on any difference. The design it measures is \
         [`strength-schemes.md`](strength-schemes.md).\n\n\
         The corpus records the recording engine's Ashtakavarga for {charts} charts, computed from \
         each chart's recorded lagna and seven grahas' signs. Beside it stands a rank-1 text, BPHS \
         chs. 66 to 69 in English translation, so every rule the engine takes after the bindu tables \
         is measured beside the text's.\n\n\
         ## What the corpus decides\n\n{table}\n",
        charts = count(records.len()),
        table = table(&claims(&records)),
    );
    out.push_str(
        "## What it means for the module\n\n\
         **The bindu tables are settled**: the engine's are the text's, every chart holds the \
         classical 337, and nothing about them is a choice.\n\n\
         **Everything after them is the engine's reading, and the text reads it otherwise.** The \
         engine reduces the sum where the text reduces each graha's own Ashtakavarga; zeroes an \
         empty co-ruled sign where the text keeps a difference; and builds its pindas from the \
         reduced sum and raw bindus, with Virgo's measure 8, where the text builds each graha's from \
         its own reduced Ashtakavarga and the grahas standing in each sign, with Virgo's 6. The \
         corpus confirms the engine and refuses the text on every chart where they part. A rank-1 \
         text corrects a rank-2 value, so the module's default is the text's and the engine's \
         reading is two settings, which the conformance profile takes (cruxes C59–C62).\n",
    );
    Ok(fill(&out))
}

pub(crate) fn generate(root: &Path) -> i32 {
    match page(root) {
        Ok(text) => write(root, &[Output::new(PAGE, text)]),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    match page(root) {
        Ok(text) => {
            i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask ashtakavarga") != 0)
        }
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_text_keeps_a_difference_the_engine_zeroes() {
        // Aries 5 occupied, Scorpio 7 empty: the text keeps 2, the engine 0.
        let mut values = [0; 12];
        values[0] = 5;
        values[7] = 7;
        let mut occupied = [false; 12];
        occupied[0] = true;
        assert_eq!(ekadhipatya(values, &occupied, Ekadhipatya::Text)[7], 2);
        assert_eq!(
            ekadhipatya(values, &occupied, Ekadhipatya::EmptyToZero)[7],
            0
        );
        // Both empty and unequal: both the smaller.
        let none = [false; 12];
        assert_eq!(
            ekadhipatya(values, &none, Ekadhipatya::Text)[0..8],
            [5, 0, 0, 0, 0, 0, 0, 5]
        );
    }
}
