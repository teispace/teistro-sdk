//! Composite and Davison charts, measured (`western-composites.md`, C247,
//! C248): every pair of the corpus's recorded births made one chart both
//! ways, how often the composite lagna is turned, and how far each
//! planet of the composite stands from the same planet of the Davison
//! chart.
//!
//! The births are founded under the conformance profile with the outer
//! planets; the composite is read in the tropical zodiac, its default,
//! and the Davison chart's planets are turned tropical to meet it.

use std::fmt::Write as _;
use std::path::Path;

use teistro::catalogue::Graha;
use teistro::{Composite, Context, Document, Partner, SynastryZodiac};

use crate::births::{Birth, CHARTS, births, conformance};
use crate::generated::{Output, check, write};
use crate::measure::{Claim, count, fill, median, share, table, worst};

const PAGE: &str = "docs/03-design/composites-measured.md";

/// How far from the composite's Sun, or from the point opposite it, the
/// Davison chart's Sun is held to stand, degrees.
const SUN_DEG: f64 = 10.0;

/// One pair of births read.
struct Pair {
    /// Their composite, the first birth first.
    composite: Composite,
    /// Their composite, the second birth first.
    reversed: Composite,
    /// Whether their Davison birth is the same either way round.
    davison_agrees: bool,
    /// Each planet's arc from the composite's to the Davison chart's,
    /// degrees, in the composite's order.
    apart: Vec<(Graha, f64)>,
}

/// The shorter arc between two longitudes, degrees.
fn arc(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d)
}

fn partner(birth: &Birth) -> Partner {
    let foundation = &birth.document.foundation;
    Partner {
        instant: foundation.instant,
        place: foundation.place,
        utc_offset: birth.offset,
    }
}

fn read_pair(
    sdk: &Context,
    (a, first): (&Birth, &Document),
    (b, second): (&Birth, &Document),
) -> Result<Pair, String> {
    let named = |why: teistro::Error| format!("{} and {}: {why}", a.name, b.name);
    let area = sdk.chart();
    let composite = area
        .composite(first, second, SynastryZodiac::Tropical)
        .map_err(named)?;
    let reversed = area
        .composite(second, first, SynastryZodiac::Tropical)
        .map_err(named)?;
    let (one, other) = (partner(a), partner(b));
    let between = one.davison(&other).map_err(named)?;
    let davison_agrees = other.davison(&one).map_err(named)? == between;
    let chart = area
        .reading(
            between.instant,
            &teistro::ChartRequest::at(between.place, between.utc_offset).with_outer_planets(),
        )
        .map_err(named)?
        .value;
    let foundation = &chart.foundation;
    let apart = composite
        .planets
        .iter()
        .map(|at| {
            foundation
                .graha(at.graha)
                .map(|there| (at.graha, arc(at.longitude_deg, there.tropical_deg)))
                .ok_or_else(|| {
                    format!(
                        "{} and {}: the Davison chart places no {}",
                        a.name,
                        b.name,
                        at.graha.key()
                    )
                })
        })
        .collect::<Result<_, _>>()?;
    Ok(Pair {
        composite,
        reversed,
        davison_agrees,
        apart,
    })
}

fn measure(root: &Path, sdk: &Context) -> Result<(usize, Vec<Pair>), String> {
    let births = births(root, sdk)?;
    if births.len() < 2 {
        return Err(format!("{CHARTS} records fewer than two charts"));
    }
    let charts = births
        .iter()
        .map(|birth| birth.with_outer_planets(sdk))
        .collect::<Result<Vec<_>, _>>()?;
    let mut pairs = Vec::new();
    for (at, a) in births.iter().zip(&charts).enumerate() {
        for b in births.iter().zip(&charts).skip(at + 1) {
            pairs.push(read_pair(sdk, a, b)?);
        }
    }
    Ok((births.len(), pairs))
}

fn claims(pairs: &[Pair]) -> Vec<Claim> {
    let behind = pairs
        .iter()
        .filter(|pair| {
            let c = &pair.composite;
            !(0.0..=180.0).contains(&(c.lagna_deg - c.midheaven_deg).rem_euclid(360.0))
        })
        .count();
    let asymmetric = pairs
        .iter()
        .filter(|pair| pair.composite != pair.reversed)
        .count();
    let davison = pairs.iter().filter(|pair| !pair.davison_agrees).count();
    let sun = pairs
        .iter()
        .flat_map(|pair| &pair.apart)
        .filter(|&&(graha, arc)| graha == Graha::Sun && arc > SUN_DEG && arc < 180.0 - SUN_DEG)
        .count();
    vec![
        Claim::counted(
            "every composite lagna stands in the half of the zodiac after its midheaven",
            behind,
            pairs.len(),
        ),
        Claim::counted(
            "a composite is the same whichever birth is first",
            asymmetric,
            pairs.len(),
        ),
        Claim::counted(
            "a Davison birth is the same whichever birth is first",
            davison,
            pairs.len(),
        ),
        Claim::counted(
            format!(
                "the Davison chart's Sun stands within {SUN_DEG}° of the composite's, or of the point opposite it"
            ),
            sun,
            pairs.len(),
        ),
    ]
}

/// Each planet's arc between the two charts.
fn by_planet(pairs: &[Pair]) -> String {
    let mut out = String::from(
        "| planet | pairs | median arc | within 1° | share | within 10° | share | widest |\n|---|---|---|---|---|---|---|---|\n",
    );
    let Some(first) = pairs.first() else {
        return out;
    };
    for &(graha, _) in &first.apart {
        let arcs: Vec<f64> = pairs
            .iter()
            .flat_map(|pair| &pair.apart)
            .filter(|(one, _)| *one == graha)
            .map(|&(_, arc)| arc)
            .collect();
        let within = |limit: f64| arcs.iter().filter(|&&arc| arc <= limit).count();
        let _ = writeln!(
            out,
            "| {} | {} | {:.2}° | {} | {} | {} | {} | {:.1}° |",
            graha.key().to_lowercase(),
            count(arcs.len()),
            median(arcs.iter().copied()),
            count(within(1.0)),
            share(within(1.0), arcs.len()),
            count(within(10.0)),
            share(within(10.0), arcs.len()),
            worst(arcs.iter().copied()),
        );
    }
    out
}

fn page(root: &Path) -> Result<String, String> {
    let sdk = conformance()?;
    let (births, pairs) = measure(root, &sdk)?;
    let turned = pairs
        .iter()
        .filter(|pair| pair.composite.lagna_turned)
        .count();
    let mut out = String::from(
        "# Composite and Davison charts, measured\n\n\
         Status: `generated` by `cargo xtask composites` from the corpus's \
         recorded births, 2026-10-03. Do not edit: `check-composites` \
         regenerates this page and fails on any difference.\n\n",
    );
    let _ = write!(
        out,
        "Each of the {} pairs of the corpus's {} births is made one chart \
         both ways (`western-composites.md`): the composite of their \
         positions, read in the tropical zodiac, and the Davison chart, a \
         chart founded under `conformance-baseline` with the outer planets \
         at the midpoint of the two births and turned tropical.\n\n",
        count(pairs.len()),
        count(births),
    );
    out.push_str(&table(&claims(&pairs)));
    let _ = write!(
        out,
        "\n## The turned lagna\n\n\
         The near midpoint of two lagnas stood before the composite \
         midheaven, and was turned by 180° to stand after it (C247), in \
         {} of the {} pairs ({}). Without the turn, those composites would \
         set the lagna in the western half of the chart.\n",
        count(turned),
        count(pairs.len()),
        share(turned, pairs.len()),
    );
    out.push_str(
        "\n## The composite against the Davison chart\n\n\
         The two methods are often offered as alternatives. The table \
         gives each planet's arc between the composite's place and the \
         Davison chart's, over every pair. A planet that moved evenly \
         would stand, at the midpoint in time, half way along all it \
         travelled between the two births: at the near midpoint of its two \
         places, or opposite it when it made an odd number of whole turns \
         on the way. So the Sun, which moves almost evenly, stands near the composite's \
         Sun or opposite it in every pair, by whether the births are an \
         even or an odd number of years apart. Every other planet's \
         motion is uneven, retrograde or both, and the table shows how \
         far that carries it.\n\n",
    );
    out.push_str(&by_planet(&pairs));
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
            i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask composites") != 0)
        }
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}
