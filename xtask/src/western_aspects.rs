//! The Western aspects, measured (`western-aspects.md`, C240): Leo's nine
//! under his orbs and Lilly's five under his moieties, read over every
//! recorded birth, and where the two models part.
//!
//! The births are founded under the conformance profile with the outer
//! planets. A separation is the same in any zodiac at one instant, so
//! the profile's sidereal frame changes no aspect.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::Path;

use teistro::catalogue::Graha;
use teistro::{AspectRequest, Context, WesternAspect, WesternAspectRow};

use crate::births::{CHARTS, births, conformance};
use crate::generated::{Output, check, write};
use crate::measure::{Claim, count, fill, table};

const PAGE: &str = "docs/03-design/western-aspects-measured.md";

/// A pair and its aspect, the two bodies in catalogue order.
type Held = (String, Graha, Graha, WesternAspect);

/// Each birth's table under both models.
struct Measured {
    births: usize,
    leo: Vec<(String, Vec<WesternAspectRow>)>,
    lilly: Vec<(String, Vec<WesternAspectRow>)>,
}

fn measure(root: &Path, sdk: &Context) -> Result<Measured, String> {
    let births = births(root, sdk)?;
    if births.is_empty() {
        return Err(format!("{CHARTS} records no chart"));
    }
    let mut measured = Measured {
        births: births.len(),
        leo: Vec::new(),
        lilly: Vec::new(),
    };
    for birth in &births {
        let named = |why: teistro::Error| format!("{}: {why}", birth.name);
        let chart = birth.with_outer_planets(sdk)?;
        let leo = sdk
            .chart()
            .western_aspects(&chart, &AspectRequest::default())
            .map_err(named)?;
        // Lilly gives the outer three no orb, so his five are read over
        // the chart as founded without them.
        let lilly = sdk
            .chart()
            .western_aspects(&birth.document, &AspectRequest::lilly())
            .map_err(named)?;
        measured.leo.push((birth.name.clone(), leo));
        measured.lilly.push((birth.name.clone(), lilly));
    }
    Ok(measured)
}

/// The rows a model found among the seven at the Ptolemaic five, as a set.
fn ptolemaic_among_the_seven(tables: &[(String, Vec<WesternAspectRow>)]) -> BTreeSet<Held> {
    let seven = |graha: Graha| !matches!(graha, Graha::Uranus | Graha::Neptune | Graha::Pluto);
    tables
        .iter()
        .flat_map(|(name, rows)| {
            rows.iter()
                .filter(move |row| {
                    WesternAspect::PTOLEMAIC.contains(&row.aspect)
                        && seven(row.first)
                        && seven(row.second)
                })
                .map(move |row| (name.clone(), row.first, row.second, row.aspect))
        })
        .collect()
}

fn claims(measured: &Measured) -> Vec<Claim> {
    let rows = measured
        .leo
        .iter()
        .chain(&measured.lilly)
        .flat_map(|(_, rows)| rows);
    let (outside, all) = rows.fold((0, 0), |(outside, all), row| {
        (
            outside + usize::from(row.from_exact_deg > row.orb_deg),
            all + 1,
        )
    });
    let leo_rows: Vec<&WesternAspectRow> = measured.leo.iter().flat_map(|(_, rows)| rows).collect();
    let pairs: BTreeSet<(usize, Graha, Graha)> = measured
        .leo
        .iter()
        .enumerate()
        .flat_map(|(at, (_, rows))| rows.iter().map(move |row| (at, row.first, row.second)))
        .collect();
    let leo = ptolemaic_among_the_seven(&measured.leo);
    let lilly = ptolemaic_among_the_seven(&measured.lilly);
    vec![
        Claim::counted(
            "every row stands inside the orb its model allowed",
            outside,
            all,
        ),
        Claim::counted(
            "Leo's orbs never put one pair at two aspects, since no two of his nine overlap",
            leo_rows.len() - pairs.len(),
            leo_rows.len(),
        ),
        Claim::counted(
            "Lilly's moieties find every Ptolemaic aspect among the seven that Leo's orbs find",
            leo.difference(&lilly).count(),
            leo.len(),
        ),
        Claim::counted(
            "Leo's orbs find every Ptolemaic aspect among the seven that Lilly's moieties find",
            lilly.difference(&leo).count(),
            lilly.len(),
        ),
    ]
}

/// Each aspect's count under each model.
fn by_aspect(measured: &Measured) -> String {
    let leo = ptolemaic_among_the_seven(&measured.leo);
    let lilly = ptolemaic_among_the_seven(&measured.lilly);
    let mut out = String::from(
        "| aspect | Leo, every planet | Leo, the seven | Lilly, the seven | both | Leo only | Lilly only |\n\
         |---|---|---|---|---|---|---|\n",
    );
    for aspect in WesternAspect::ALL {
        let every = measured
            .leo
            .iter()
            .flat_map(|(_, rows)| rows)
            .filter(|row| row.aspect == aspect)
            .count();
        let of = |set: &BTreeSet<Held>| set.iter().filter(|held| held.3 == aspect).count();
        let ptolemaic = WesternAspect::PTOLEMAIC.contains(&aspect);
        let cell = |value: usize| {
            if ptolemaic {
                count(value)
            } else {
                String::from("—")
            }
        };
        let both = leo
            .intersection(&lilly)
            .filter(|held| held.3 == aspect)
            .count();
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} |",
            aspect.key(),
            count(every),
            cell(of(&leo)),
            cell(of(&lilly)),
            cell(both),
            cell(of(&leo) - both),
            cell(of(&lilly) - both),
        );
    }
    out
}

fn page(root: &Path) -> Result<String, String> {
    let sdk = conformance()?;
    let measured = measure(root, &sdk)?;
    let mut out = String::from(
        "# The Western aspects, measured\n\n\
         Status: `generated` by `cargo xtask western-aspects` from the \
         corpus's recorded births, 2026-10-03. Do not edit: \
         `check-western-aspects` regenerates this page and fails on any \
         difference.\n\n",
    );
    let _ = write!(
        out,
        "Each of the {} births is read twice (`western-aspects.md`, C240). \
         Leo's nine aspects are read under his orbs over the ten planets, the \
         chart founded with the outer three. Lilly's five are read under his \
         moieties over the seven, since he gives the outer three no orb. \
         Both are founded under `conformance-baseline`; a separation is the \
         same in any zodiac.\n\n",
        count(measured.births)
    );
    out.push_str(&table(&claims(&measured)));
    out.push_str(
        "\n## By aspect\n\n\
         How many rows each model finds, summed over the births. The last \
         three columns compare the two over the Ptolemaic five among the \
         seven planets, the only ground both models read; a dash marks an \
         aspect Lilly does not read.\n\n",
    );
    out.push_str(&by_aspect(&measured));
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
        Ok(text) => i32::from(
            check(
                root,
                &[Output::new(PAGE, text)],
                "cargo xtask western-aspects",
            ) != 0,
        ),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}
