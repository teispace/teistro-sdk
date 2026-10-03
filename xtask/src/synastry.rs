//! Synastry, measured (`western-synastry.md`, C241): every pair of the
//! corpus's recorded births read against each other, in the tropical
//! zodiac and in each chart's own, and where the two readings part.
//!
//! The births are founded under the conformance profile, whose zodiac is
//! sidereal, with the outer planets, so each chart's longitudes carry its
//! own instant's ayanamsha and the two readings can differ.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::Path;

use teistro::{
    Context, Document, NatalPoint, SynastryRequest, SynastryRow, SynastryZodiac, WesternAspect,
};

use crate::births::{CHARTS, births, conformance};
use crate::generated::{Output, check, write};
use crate::measure::{Claim, arcsec, count, fill, table};

const PAGE: &str = "docs/03-design/synastry-measured.md";

/// A contact, by its two points and its aspect.
type Contact = (NatalPoint, NatalPoint, WesternAspect);

/// One pair of births read both ways.
struct Pair {
    /// Years between the two births.
    gap_years: f64,
    /// The second chart's ayanamsha less the first's, arcseconds.
    drift_arcsec: f64,
    tropical: Vec<SynastryRow>,
    charts: Vec<SynastryRow>,
}

impl Pair {
    fn contacts(rows: &[SynastryRow]) -> BTreeSet<Contact> {
        rows.iter()
            .map(|row| (row.first, row.second, row.aspect))
            .collect()
    }

    /// The contacts one reading finds and the other does not.
    fn parted(&self) -> usize {
        let (tropical, charts) = (Pair::contacts(&self.tropical), Pair::contacts(&self.charts));
        tropical.symmetric_difference(&charts).count()
    }

    /// The contacts either reading finds.
    fn either(&self) -> usize {
        Pair::contacts(&self.tropical)
            .union(&Pair::contacts(&self.charts))
            .count()
    }
}

/// Every pair of births, each read once, the earlier-named first.
struct Measured {
    births: usize,
    pairs: Vec<Pair>,
}

fn measure(root: &Path, sdk: &Context) -> Result<Measured, String> {
    let births = births(root, sdk)?;
    if births.len() < 2 {
        return Err(format!("{CHARTS} records fewer than two charts"));
    }
    let charts: Vec<(String, Document)> = births
        .iter()
        .map(|birth| Ok((birth.name.clone(), birth.with_outer_planets(sdk)?)))
        .collect::<Result<_, String>>()?;
    let read = |a: &Document, b: &Document, zodiac: SynastryZodiac| {
        sdk.chart()
            .synastry(a, b, &SynastryRequest::default().with_zodiac(zodiac))
    };
    let mut pairs = Vec::new();
    for (at, (first_name, first)) in charts.iter().enumerate() {
        for (second_name, second) in charts.iter().skip(at + 1) {
            let named = |why: teistro::Error| format!("{first_name} and {second_name}: {why}");
            let (a, b) = (&first.foundation, &second.foundation);
            pairs.push(Pair {
                gap_years: (b.instant.get() - a.instant.get()).abs() / 365.25,
                drift_arcsec: (b.zodiac.offset_deg - a.zodiac.offset_deg) * 3600.0,
                tropical: read(first, second, SynastryZodiac::Tropical).map_err(named)?,
                charts: read(first, second, SynastryZodiac::Charts).map_err(named)?,
            });
        }
    }
    Ok(Measured {
        births: births.len(),
        pairs,
    })
}

fn claims(measured: &Measured) -> Vec<Claim> {
    let rows = || {
        measured
            .pairs
            .iter()
            .flat_map(|pair| pair.tropical.iter().chain(&pair.charts))
    };
    let outside = rows()
        .filter(|row| row.from_exact_deg > row.orb_deg)
        .count();
    let tables = || {
        measured
            .pairs
            .iter()
            .flat_map(|pair| [&pair.tropical, &pair.charts])
    };
    let (doubled, read) = tables().fold((0, 0), |(doubled, read), rows| {
        let points: BTreeSet<(NatalPoint, NatalPoint)> =
            rows.iter().map(|row| (row.first, row.second)).collect();
        (doubled + rows.len() - points.len(), read + rows.len())
    });
    let parted: usize = measured.pairs.iter().map(Pair::parted).sum();
    let either: usize = measured.pairs.iter().map(Pair::either).sum();
    let drift = measured
        .pairs
        .iter()
        .map(|pair| pair.drift_arcsec.abs())
        .fold(0.0, f64::max);
    let shares: Vec<f64> = BANDS
        .iter()
        .map(|&(from, to, _)| parted_share(&band(measured, (from, to))))
        .collect();
    let shrinking = shares
        .windows(2)
        .filter(|two| matches!(two, [earlier, later] if later <= earlier))
        .count();
    vec![
        Claim::counted(
            "every row stands inside the orb its model allowed",
            outside,
            rows().count(),
        ),
        Claim::counted(
            "Leo's orbs never put one pair of points at two aspects",
            doubled,
            read,
        ),
        Claim::counted(
            "the tropical zodiac and each chart's own find the same contacts",
            parted,
            either,
        )
        .with_note(format!(
            "the two charts' ayanamshas stand at most {}″ apart",
            arcsec(drift)
        )),
        Claim::counted(
            "the share of contacts the two readings part on grows with each band of years between the births",
            shrinking,
            shares.len() - 1,
        ),
    ]
}

/// The bands of years between the births the page reads by.
const BANDS: [(f64, f64, &str); 5] = [
    (0.0, 10.0, "under 10"),
    (10.0, 25.0, "10 to 25"),
    (25.0, 50.0, "25 to 50"),
    (50.0, 100.0, "50 to 100"),
    (100.0, f64::INFINITY, "100 or more"),
];

/// The pairs whose births stand this many years apart.
fn band(measured: &Measured, (from, to): (f64, f64)) -> Vec<&Pair> {
    measured
        .pairs
        .iter()
        .filter(|pair| (from..to).contains(&pair.gap_years))
        .collect()
}

/// The share of a band's contacts that one reading finds and the other
/// does not; zero for a band with none.
fn parted_share(pairs: &[&Pair]) -> f64 {
    let either: usize = pairs.iter().map(|pair| pair.either()).sum();
    let parted: usize = pairs.iter().map(|pair| pair.parted()).sum();
    if either == 0 {
        0.0
    } else {
        f64::from(u32::try_from(parted).unwrap_or(u32::MAX))
            / f64::from(u32::try_from(either).unwrap_or(u32::MAX))
    }
}

/// The two readings by the years between the births.
fn by_gap(measured: &Measured) -> String {
    let mut out = String::from(
        "| years apart | pairs | widest drift, ″ | contacts either finds | found by one only | share |\n\
         |---|---|---|---|---|---|\n",
    );
    for (from, to, label) in BANDS {
        let band = band(measured, (from, to));
        let drift = band
            .iter()
            .map(|pair| pair.drift_arcsec.abs())
            .fold(0.0, f64::max);
        let _ = writeln!(
            out,
            "| {label} | {} | {} | {} | {} | {:.1}% |",
            count(band.len()),
            if band.is_empty() {
                String::from("—")
            } else {
                arcsec(drift)
            },
            count(band.iter().map(|pair| pair.either()).sum()),
            count(band.iter().map(|pair| pair.parted()).sum()),
            100.0 * parted_share(&band),
        );
    }
    out
}

fn page(root: &Path) -> Result<String, String> {
    let sdk = conformance()?;
    let measured = measure(root, &sdk)?;
    let mut out = String::from(
        "# Synastry, measured\n\n\
         Status: `generated` by `cargo xtask synastry` from the corpus's \
         recorded births, 2026-10-03. Do not edit: `check-synastry` \
         regenerates this page and fails on any difference.\n\n",
    );
    let _ = write!(
        out,
        "Each of the {} pairs of the corpus's {} births is read against \
         each other twice (`western-synastry.md`, C241): in the tropical \
         zodiac, the default, and in each chart's own. The births are \
         founded under `conformance-baseline`, whose zodiac is sidereal, \
         with the outer planets; every point of one is read against every \
         point of the other, the lagna included, under Leo's nine aspects \
         and his orbs.\n\n",
        count(measured.pairs.len()),
        count(measured.births),
    );
    out.push_str(&table(&claims(&measured)));
    out.push_str(
        "\n## By the years between the births\n\n\
         Read in each chart's own zodiac, every separation across the two \
         moves by the difference of their ayanamshas, the precession \
         between the births. A contact parts the two readings only when \
         that shift carries it across its orb's edge.\n\n",
    );
    out.push_str(&by_gap(&measured));
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
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask synastry") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}
