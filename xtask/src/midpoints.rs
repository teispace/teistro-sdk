//! Equal distances, measured (`western-midpoints.md`, C245, C246): every
//! recorded birth's planets equally distant from two others, at three
//! orbs from the axis, how many stand on the far point, and how many also
//! stand in an aspect to one of the two, which Leo reads instead.
//!
//! The births are founded under the conformance profile with the outer
//! planets. An equal distance is a fact about arcs, which the profile's
//! sidereal frame does not move.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::Path;

use teistro::catalogue::Graha;
use teistro::{AspectRequest, Context, MidpointRequest, MidpointRow};

use crate::births::{CHARTS, births, conformance};
use crate::generated::{Output, check, write};
use crate::measure::{Claim, count, fill, share, table};

const PAGE: &str = "docs/03-design/midpoints-measured.md";

/// The orbs from the axis the page reads, degrees: the default (C245),
/// the parallel's own 1°, and twice it.
const ORBS: [f64; 3] = [0.5, 1.0, 2.0];

/// One birth read.
struct Read {
    /// The equal distances at each of [`ORBS`], in its order.
    rows: Vec<Vec<MidpointRow>>,
    /// The pairs of planets in an aspect under Leo's table, the earlier
    /// first.
    aspects: BTreeSet<(Graha, Graha)>,
}

impl Read {
    /// Whether a row's planet between stands in an aspect to either of the
    /// two, which Leo reads instead of the equal distance (p. 48).
    fn aspected(&self, row: &MidpointRow) -> bool {
        [row.first, row.second]
            .into_iter()
            .any(|one| self.aspects.contains(&ordered(one, row.middle)))
    }
}

/// Whether two rows name the same three planets the same way.
fn same(a: &MidpointRow, b: &MidpointRow) -> bool {
    (a.first, a.second, a.middle) == (b.first, b.second, b.middle)
}

/// Two planets in the catalogue's order.
fn ordered(a: Graha, b: Graha) -> (Graha, Graha) {
    if a.id() <= b.id() { (a, b) } else { (b, a) }
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
                rows: ORBS
                    .iter()
                    .map(|&orb| {
                        area.midpoints(&chart, &MidpointRequest::default().with_orb_deg(orb))
                            .map_err(named)
                    })
                    .collect::<Result<_, _>>()?,
                aspects: area
                    .western_aspects(&chart, &AspectRequest::default())
                    .map_err(named)?
                    .iter()
                    .map(|row| ordered(row.first, row.second))
                    .collect(),
            })
        })
        .collect()
}

fn claims(read: &[Read]) -> Vec<Claim> {
    let tables = || read.iter().flat_map(|one| &one.rows);
    let rows = tables().map(Vec::len).sum();
    let outside = tables()
        .flatten()
        .filter(|row| row.from_axis_deg > row.orb_deg)
        .count();
    let unordered = tables()
        .filter(|rows| {
            !rows
                .windows(2)
                .all(|two| matches!(two, [a, b] if a.from_axis_deg <= b.from_axis_deg))
        })
        .count();
    let unnested = read
        .iter()
        .flat_map(|one| one.rows.windows(2))
        .filter(|two| matches!(two, [narrow, wide] if !narrow.iter().all(|row| wide.iter().any(|other| same(other, row)))))
        .count();
    vec![
        Claim::counted("every equal distance stands inside its orb", outside, rows),
        Claim::counted(
            "each table's rows come closest first",
            unordered,
            read.len() * ORBS.len(),
        ),
        Claim::counted(
            "a wider orb keeps every row a narrower one holds",
            unnested,
            read.len() * (ORBS.len() - 1),
        ),
    ]
}

/// The equal distances at each orb: how many, on the far point, and with
/// an aspect to one of the two.
fn by_orb(read: &[Read]) -> String {
    let mut out = String::from(
        "| orb from the axis | equal distances | a birth, on average | on the far point | share | the planet between also in an aspect to one of the two | share |\n|---|---|---|---|---|---|---|\n",
    );
    for (at, orb) in ORBS.iter().enumerate() {
        let rows: Vec<(&Read, &MidpointRow)> = read
            .iter()
            .flat_map(|one| {
                one.rows
                    .get(at)
                    .into_iter()
                    .flatten()
                    .map(move |row| (one, row))
            })
            .collect();
        let far = rows.iter().filter(|(_, row)| row.far).count();
        let aspected = rows.iter().filter(|(one, row)| one.aspected(row)).count();
        let _ = writeln!(
            out,
            "| {orb}° | {} | {:.1} | {} | {} | {} | {} |",
            count(rows.len()),
            average(rows.len(), read.len()),
            count(far),
            share(far, rows.len()),
            count(aspected),
            share(aspected, rows.len()),
        );
    }
    out
}

/// A mean over a count of births.
fn average(total: usize, births: usize) -> f64 {
    let (Ok(total), Ok(births)) = (u32::try_from(total), u32::try_from(births)) else {
        return f64::NAN;
    };
    f64::from(total) / f64::from(births.max(1))
}

fn page(root: &Path) -> Result<String, String> {
    let sdk = conformance()?;
    let read = measure(root, &sdk)?;
    let mut out = String::from(
        "# Equal distances, measured\n\n\
         Status: `generated` by `cargo xtask midpoints` from the corpus's \
         recorded births, 2026-10-03. Do not edit: `check-midpoints` \
         regenerates this page and fails on any difference.\n\n",
    );
    let _ = write!(
        out,
        "Each of the corpus's {} births is founded under \
         `conformance-baseline` with the outer planets, and read for the \
         planets equally distant from two others (`western-midpoints.md`) \
         at three orbs from the axis through the two's midpoint: the \
         default 0.5° (C245), the parallel's 1° and 2°. An equal distance \
         is a fact about arcs, so the profile's sidereal frame changes \
         none.\n\n",
        count(read.len()),
    );
    out.push_str(&table(&claims(&read)));
    out.push_str(
        "\n## By orb\n\n\
         Leo's only example stands on the far point of the axis (C246), \
         and he reads an aspect instead where the three stand in one \
         (p. 48). The table counts both: the rows on the far point, and \
         the rows whose planet between stands in an aspect to one of the \
         two under Leo's own table and orbs (C240).\n\n",
    );
    out.push_str(&by_orb(&read));
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
            i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask midpoints") != 0)
        }
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}
