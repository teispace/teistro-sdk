//! Antiscia, measured (`western-antiscia.md`, C244): every recorded
//! birth's pairs in antiscion and contrantiscion under Lilly's moieties
//! and under Leo's orbs, and how many share their pair with a parallel of
//! declination, the same equality read off the equator.
//!
//! The births are founded under the conformance profile with the outer
//! planets. An antiscion is a reflection about the tropical solstices, so
//! the profile's sidereal frame changes none.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::Path;

use teistro::catalogue::Graha;
use teistro::{
    Antiscia, AntisciaRequest, AspectOrb, Context, OrbModel, ParallelRequest, WesternAspect,
};

use crate::births::{CHARTS, births, conformance};
use crate::generated::{Output, check, write};
use crate::measure::{Claim, count, fill, share, table};

const PAGE: &str = "docs/03-design/antiscia-measured.md";

/// The outer three, which Lilly's moieties give no orb.
const OUTER: [Graha; 3] = [Graha::Uranus, Graha::Neptune, Graha::Pluto];

/// How one birth's antiscia were read, by which orbs.
type Model = fn(&Read) -> &Antiscia;

/// One birth read.
struct Read {
    /// Under Lilly's moieties, the default (C244).
    moieties: Antiscia,
    /// Under Leo's orbs, which give every planet one.
    leo: Antiscia,
    /// Within the parallel's own 1°, so the two equalities are compared
    /// at one width.
    narrow: Antiscia,
    /// The pairs in parallel of declination within Leo's 1°, with
    /// whether contrary, the earlier first.
    parallels: BTreeSet<(Graha, Graha, bool)>,
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
            let narrow = AntisciaRequest::default().with_orbs(OrbModel::ByAspect {
                orbs: vec![AspectOrb {
                    aspect: WesternAspect::Conjunction,
                    orb_deg: ParallelRequest::default().orb_deg,
                }],
            });
            Ok(Read {
                moieties: area
                    .antiscia(&chart, &AntisciaRequest::default())
                    .map_err(named)?,
                leo: area
                    .antiscia(&chart, &AntisciaRequest::default().with_orbs(OrbModel::Leo))
                    .map_err(named)?,
                narrow: area.antiscia(&chart, &narrow).map_err(named)?,
                parallels: area
                    .parallels(&chart, &ParallelRequest::default())
                    .map_err(named)?
                    .iter()
                    .map(|row| (row.first, row.second, row.contrary))
                    .collect(),
            })
        })
        .collect()
}

fn claims(read: &[Read]) -> Vec<Claim> {
    let tables = || read.iter().flat_map(|one| [&one.moieties, &one.leo]);
    let rows = tables().map(|one| one.pairs.len()).sum();
    let outside = tables()
        .flat_map(|one| &one.pairs)
        .filter(|row| row.apart_deg > row.orb_deg)
        .count();
    let unordered = tables()
        .filter(|one| {
            !one.pairs
                .windows(2)
                .all(|two| matches!(two, [a, b] if a.apart_deg <= b.apart_deg))
        })
        .count();
    let misplaced = read
        .iter()
        .filter(|one| one.moieties.unpaired != OUTER || !one.leo.unpaired.is_empty())
        .count();
    vec![
        Claim::counted("every pair stands inside its orb", outside, rows),
        Claim::counted(
            "each table's pairs come closest first",
            unordered,
            read.len() * 2,
        ),
        Claim::counted(
            "the moieties leave exactly the outer three unpaired, and Leo's orbs none",
            misplaced,
            read.len(),
        ),
    ]
}

/// The pairs by relation under each model, and how many share their pair
/// and their side with a parallel of declination.
fn by_kind(read: &[Read]) -> String {
    let mut out = String::from(
        "| orbs | relation | pairs | also a parallel of declination | share |\n|---|---|---|---|---|\n",
    );
    let models: [(&str, Model); 3] = [
        ("Lilly's moieties", |one| &one.moieties),
        ("Leo's", |one| &one.leo),
        ("1°, the parallel's", |one| &one.narrow),
    ];
    for (label, of) in models {
        for (relation, contrary) in [("antiscion", false), ("contrantiscion", true)] {
            let rows: Vec<bool> = read
                .iter()
                .flat_map(|one| {
                    of(one)
                        .pairs
                        .iter()
                        .filter(move |row| row.contrary == contrary)
                        .map(move |row| one.parallels.contains(&(row.first, row.second, contrary)))
                })
                .collect();
            let both = rows.iter().filter(|&&held| held).count();
            let _ = writeln!(
                out,
                "| {label} | {relation} | {} | {} | {} |",
                count(rows.len()),
                count(both),
                share(both, rows.len()),
            );
        }
    }
    out
}

fn page(root: &Path) -> Result<String, String> {
    let sdk = conformance()?;
    let read = measure(root, &sdk)?;
    let mut out = String::from(
        "# Antiscia, measured\n\n\
         Status: `generated` by `cargo xtask antiscia` from the corpus's \
         recorded births, 2026-10-03. Do not edit: `check-antiscia` \
         regenerates this page and fails on any difference.\n\n",
    );
    let _ = write!(
        out,
        "Each of the corpus's {} births is founded under \
         `conformance-baseline` with the outer planets, and read for its \
         antiscia twice: under Lilly's moieties, the default (C244), and \
         under Leo's orbs, both at the conjunction \
         (`western-antiscia.md`). An antiscion is a reflection about the \
         tropical solstices, so the profile's sidereal frame changes \
         none.\n\n",
        count(read.len()),
    );
    out.push_str(&table(&claims(&read)));
    out.push_str(
        "\n## Antiscia and parallels\n\n\
         Two degrees in antiscion have the same declination, so two \
         planets in antiscion would stand in parallel if neither had a \
         latitude. The table counts how often a pair holds both, on the \
         same side for the antiscion and on opposite sides for the \
         contrantiscion, the parallel within Leo's 1° (p. 47). The \
         moieties and Leo's orbs are many degrees wide and hold pairs no \
         1° parallel could, so the last rows read the antiscia at the \
         parallel's own 1°. Without latitude, a 1° antiscion is \
         always a parallel too, since a degree of longitude moves a \
         declination by at most 0.4°; so the pairs that fall short do \
         so by their planets' latitudes, which an antiscion does not \
         read.\n\n",
    );
    out.push_str(&by_kind(&read));
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
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask antiscia") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}
