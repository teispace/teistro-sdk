//! Declinations and parallels, measured (`western-declinations.md`,
//! C243): every recorded birth's declinations and Leo's parallels among
//! its planets, how many are contrary, how many coincide with an aspect,
//! and which planets pass the obliquity.
//!
//! The births are founded under the conformance profile with the outer
//! planets. A declination does not depend on the zodiac, so the profile's
//! sidereal frame changes none; its topocentric frame is what the chart
//! placed, and so what is read.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::Path;

use teistro::catalogue::Graha;
use teistro::{AspectRequest, Context, Declinations, ParallelRequest, ParallelRow};

use crate::births::{CHARTS, births, conformance};
use crate::generated::{Output, check, write};
use crate::measure::{Claim, arcsec, count, fill, table, verdict_of};

const PAGE: &str = "docs/03-design/declinations-measured.md";

/// The Sun's mean horizontal parallax, arcseconds: the most a topocentric
/// frame can lift it off the ecliptic.
const SUN_PARALLAX_ARCSEC: f64 = 8.794;

/// One birth read.
struct Read {
    declinations: Declinations,
    parallels: Vec<ParallelRow>,
    /// The pairs that hold an aspect under Leo's orbs, the earlier first.
    aspected: BTreeSet<(Graha, Graha)>,
}

fn measure(root: &Path, sdk: &Context) -> Result<Vec<Read>, String> {
    let births = births(root, sdk)?;
    if births.is_empty() {
        return Err(format!("{CHARTS} records no chart"));
    }
    births
        .iter()
        .map(|birth| {
            let named = |why: teistro::Error| format!("{}: {why}", birth.name);
            let chart = birth.with_outer_planets(sdk)?;
            let area = sdk.chart();
            Ok(Read {
                declinations: area.declinations(&chart).map_err(named)?,
                parallels: area
                    .parallels(&chart, &ParallelRequest::default())
                    .map_err(named)?,
                aspected: area
                    .western_aspects(&chart, &AspectRequest::default())
                    .map_err(named)?
                    .iter()
                    .map(|row| (row.first, row.second))
                    .collect(),
            })
        })
        .collect()
}

/// The parallels every birth holds, each with whether its pair also holds
/// an aspect.
fn parallels(read: &[Read]) -> impl Iterator<Item = (&ParallelRow, bool)> {
    read.iter().flat_map(|one| {
        one.parallels
            .iter()
            .map(|row| (row, one.aspected.contains(&(row.first, row.second))))
    })
}

fn claims(read: &[Read]) -> Vec<Claim> {
    let rows = parallels(read).count();
    let outside = parallels(read)
        .filter(|(row, _)| row.apart_deg > row.orb_deg)
        .count();
    let angles = read.len() * 2;
    let past = read
        .iter()
        .flat_map(|one| {
            let d = &one.declinations;
            [d.lagna_deg, d.midheaven_deg].map(|deg| deg.abs() > d.obliquity_deg)
        })
        .filter(|&past| past)
        .count();
    let unordered = read
        .iter()
        .filter(|one| {
            !one.parallels
                .windows(2)
                .all(|two| matches!(two, [a, b] if a.apart_deg <= b.apart_deg))
        })
        .count();
    let sun_past = read
        .iter()
        .filter_map(|one| {
            let d = &one.declinations;
            d.graha(Graha::Sun)
                .map(|deg| (deg.abs() - d.obliquity_deg) * 3600.0)
        })
        .fold(0.0, f64::max);
    vec![
        Claim::counted("every parallel stands inside Leo's 1°", outside, rows),
        Claim::stated(
            "the Sun passes the obliquity by no more than its parallax, 8.8″",
            verdict_of(sun_past <= SUN_PARALLAX_ARCSEC),
            format!("furthest {}″", arcsec(sun_past)),
        ),
        Claim::counted(
            "an angle's declination never passes the obliquity, since it has no latitude",
            past,
            angles,
        ),
        Claim::counted(
            "each chart's parallels come closest first",
            unordered,
            read.len(),
        ),
    ]
}

/// The parallels by side of the equator, and how many share their pair
/// with an aspect.
fn by_kind(read: &[Read]) -> String {
    let mut out = String::from(
        "| kind | parallels | the pair also holds an aspect | share |\n|---|---|---|---|\n",
    );
    for (label, contrary) in [("same side", false), ("contrary", true)] {
        let rows: Vec<bool> = parallels(read)
            .filter(|(row, _)| row.contrary == contrary)
            .map(|(_, aspected)| aspected)
            .collect();
        let both = rows.iter().filter(|&&aspected| aspected).count();
        let _ = writeln!(
            out,
            "| {label} | {} | {} | {} |",
            count(rows.len()),
            count(both),
            share(both, rows.len()),
        );
    }
    out
}

/// Each planet past the obliquity: in how many births, and how far at
/// most.
fn out_of_bounds(read: &[Read]) -> String {
    let mut out = String::from(
        "| body | births it is placed in | past the obliquity | furthest past |\n|---|---|---|---|\n",
    );
    let order: Vec<Graha> = read
        .first()
        .map(|one| one.declinations.grahas.iter().map(|at| at.graha).collect())
        .unwrap_or_default();
    for graha in order {
        let excess: Vec<f64> = read
            .iter()
            .filter_map(|one| {
                let d = &one.declinations;
                d.graha(graha).map(|deg| deg.abs() - d.obliquity_deg)
            })
            .collect();
        let past: Vec<f64> = excess.iter().copied().filter(|&e| e > 0.0).collect();
        let furthest = past.iter().copied().fold(None, |most: Option<f64>, e| {
            Some(most.map_or(e, |m| m.max(e)))
        });
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} |",
            graha.key().to_lowercase(),
            count(excess.len()),
            count(past.len()),
            furthest.map_or_else(|| String::from("—"), past_by),
        );
    }
    out
}

/// How far past the obliquity: degrees, or arcseconds when it is less
/// than a hundredth of one.
fn past_by(excess_deg: f64) -> String {
    if excess_deg < 0.01 {
        format!("{}″", arcsec(excess_deg * 3600.0))
    } else {
        format!("{excess_deg:.2}°")
    }
}

/// A share as a percentage to one place; a dash for none.
fn share(part: usize, whole: usize) -> String {
    if whole == 0 {
        return String::from("—");
    }
    let ratio = f64::from(u32::try_from(part).unwrap_or(u32::MAX))
        / f64::from(u32::try_from(whole).unwrap_or(u32::MAX));
    format!("{:.1}%", 100.0 * ratio)
}

fn page(root: &Path) -> Result<String, String> {
    let sdk = conformance()?;
    let read = measure(root, &sdk)?;
    let mut out = String::from(
        "# Declinations and parallels, measured\n\n\
         Status: `generated` by `cargo xtask declinations` from the \
         corpus's recorded births, 2026-10-03. Do not edit: \
         `check-declinations` regenerates this page and fails on any \
         difference.\n\n",
    );
    let _ = write!(
        out,
        "Each of the corpus's {} births is founded under \
         `conformance-baseline` with the outer planets, and read for its \
         declinations and for Leo's parallels among its planets: two \
         distances from the equator within 1°, on either side of it \
         (`western-declinations.md`, C243). A declination does not depend \
         on the zodiac; the profile's topocentric frame is what the chart \
         placed, and so what is read.\n\n",
        count(read.len()),
    );
    out.push_str(&table(&claims(&read)));
    out.push_str(
        "\n## Same side and contrary\n\n\
         Leo counts a pair on opposite sides of the equator a parallel \
         (C243), and reads a pair that also holds an aspect by the aspect \
         (p. 43). The aspect is any of his nine under his orbs.\n\n",
    );
    out.push_str(&by_kind(&read));
    out.push_str(
        "\n## Past the obliquity\n\n\
         A planet off the ecliptic can stand further from the equator than \
         the Sun ever does. Leo gives this no reading, and the page \
         decides none; it counts how often each body does. The Sun's \
         own latitude is the topocentric frame's parallax, a few \
         arcseconds, so a birth at a solstice can carry it past by \
         that much.\n\n",
    );
    out.push_str(&out_of_bounds(&read));
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
            i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask declinations") != 0)
        }
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}
