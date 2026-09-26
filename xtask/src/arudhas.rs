//! The falsification pass over the arudha padas, which the points module
//! and the sign-based dashas read (Phase 5).
//!
//! The corpus records all twelve under `houses.arudha_padas`, each with the
//! sign it first counts to and whether an exception moved it, and the arudha
//! lagna again under `foundation`; every input — the lagna's sign and each
//! graha's — is recorded beside them. So a rule is right or wrong, and the
//! page measures it beside the readings that differ from it: Scorpio's and
//! Aquarius's Jaimini lords instead of Mars and Saturn, no exception, and
//! the seventh-house exception counted from the house instead of from the
//! pada.
//!
//! `cargo xtask arudhas` writes the page; `check-arudhas` regenerates it in
//! memory and fails on any difference.

use std::fmt::Write as _;
use std::path::Path;

use serde_json::Value;
use teistro::dasha::jaimini::pada_lord;
use teistro::dasha::rashi::RashiChart;
use teistro::points::arudha::arudha_by;
use teistro::settings::NodeCoLordship;
use teistro_core::catalogue::{Catalogued, Dignity, Graha, Rashi};

use crate::generated::{Output, check, write};
use crate::measure::{Claim, count, fill, table};

const PAGE: &str = "docs/03-design/arudhas-measured.md";
const ROOT: &str = "fixtures/baseline";

/// One recorded chart: its lagna, each graha's sign, and its padas.
struct Chart {
    lagna: usize,
    signs: Vec<(Graha, usize)>,
    /// Each house's `(sign, first counted sign, exception applied)`.
    padas: Vec<(usize, usize, bool)>,
    arudha_lagna: usize,
}

fn whole(value: &Value, what: &str) -> Result<usize, String> {
    value
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| format!("{what} is not a whole number"))
}

fn charts(root: &Path) -> Result<Vec<Chart>, String> {
    let mut out = Vec::new();
    for dir in ["charts", "variants"] {
        let directory = root.join(ROOT).join(dir);
        let mut paths: Vec<_> = std::fs::read_dir(&directory)
            .map_err(|err| format!("{}: {err}", directory.display()))?
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
            .collect();
        paths.sort();
        for path in paths {
            let at = |err: String| format!("{}: {err}", path.display());
            let fixture: Value = serde_json::from_str(
                &std::fs::read_to_string(&path).map_err(|err| at(err.to_string()))?,
            )
            .map_err(|err| at(err.to_string()))?;
            let Some(padas) = fixture["houses"]["arudha_padas"].as_array() else {
                continue;
            };
            if fixture["positions"].is_null() || fixture["foundation"].is_null() {
                continue;
            }
            let signs = Graha::all()
                .iter()
                .take(9)
                .map(|graha| {
                    whole(
                        &fixture["positions"]["bodies"][graha.key()]["sign_index"],
                        graha.key(),
                    )
                    .map(|sign| (*graha, sign))
                })
                .collect::<Result<Vec<_>, String>>()
                .map_err(&at)?;
            let padas = padas
                .iter()
                .map(|pada| {
                    Ok((
                        whole(&pada["sign_index"], "sign_index")?,
                        whole(&pada["original_sign_index"], "original_sign_index")?,
                        pada["exception_applied"]
                            .as_bool()
                            .ok_or("exception_applied")?,
                    ))
                })
                .collect::<Result<Vec<_>, String>>()
                .map_err(&at)?;
            out.push(Chart {
                lagna: whole(&fixture["foundation"]["lagna"]["sign_index"], "lagna")
                    .map_err(&at)?,
                signs,
                padas,
                arudha_lagna: whole(
                    &fixture["foundation"]["arudha_lagna_sign_index"],
                    "arudha_lagna",
                )
                .map_err(&at)?,
            });
        }
    }
    Ok(out)
}

/// Whose lordship a sign's lord is read from.
#[derive(Clone, Copy)]
enum Lords {
    /// The catalogue's: Mars for Scorpio, Saturn for Aquarius.
    Parashari,
    /// Ketu for Scorpio, Rahu for Aquarius.
    Jaimini,
    /// BPHS ch. 29 v. 7 with the nodes as co-lords: the stronger of the two
    /// by ch. 46's ladder (crux C135), as the façade counts under
    /// `jaimini.node_co_lordship = BOTH`.
    StrongerOfTwo,
}

/// What a pada in the house or the seventh from it becomes.
#[derive(Clone, Copy)]
enum Exception {
    /// The tenth from the pada.
    TenthFromPada,
    /// The tenth from the house.
    TenthFromHouse,
    /// It stays.
    None,
}

fn rashi(index: usize) -> Rashi {
    Rashi::ALL[index % 12]
}

/// The chart as ch. 46's ladder reads it. The corpus records signs and not
/// dignities, and the ladder's only dignity step is exaltation, which the
/// sign decides, so a graha in its exaltation sign is exalted and every
/// other one neutral.
fn ladder_chart(chart: &Chart) -> RashiChart {
    let sign_of = |graha: Graha| {
        chart
            .signs
            .iter()
            .find(|(each, _)| *each == graha)
            .map_or(rashi(chart.lagna), |(_, at)| rashi(*at))
    };
    let grahas: [Graha; 9] = std::array::from_fn(|k| Graha::ALL[k]);
    RashiChart {
        lagna: rashi(chart.lagna),
        arudha_lagna: rashi(chart.lagna),
        navamsa_lagna: rashi(chart.lagna),
        signs: grahas.map(sign_of),
        dignities: grahas.map(|graha| {
            let exalted = graha
                .attributes()
                .exaltation
                .is_some_and(|at| at.sign == sign_of(graha));
            if exalted {
                Dignity::Exalted
            } else {
                Dignity::Neutral
            }
        }),
        brahma: None,
    }
}

/// A house's pada under a reading of the lords and of the exception, as
/// `(sign, first counted sign, exception applied)`: the count is the
/// shipped one (`points::arudha::arudha_by`), so a change to it moves this
/// page, and the exceptions it does not ship are read off its first count.
fn pada(chart: &Chart, house: usize, lords: Lords, exception: Exception) -> (usize, usize, bool) {
    let ladder = ladder_chart(chart);
    let lord_of = |sign: Rashi| match lords {
        Lords::Parashari => pada_lord(&ladder, sign, NodeCoLordship::None),
        Lords::StrongerOfTwo => pada_lord(&ladder, sign, NodeCoLordship::Both),
        Lords::Jaimini => match sign {
            Rashi::Scorpio => Graha::Ketu,
            Rashi::Aquarius => Graha::Rahu,
            other => other.attributes().lord,
        },
    };
    let shipped = arudha_by(
        rashi(chart.lagna),
        u8::try_from(house + 1).unwrap_or(1),
        lord_of,
        |graha| ladder.signs[graha as usize],
    );
    let sign = (chart.lagna + house) % 12;
    let first = shipped.counted as usize;
    let moves = matches!((first + 12 - sign) % 12, 0 | 6);
    match (moves, exception) {
        (true, Exception::TenthFromPada) => (shipped.sign as usize, first, shipped.moved),
        (true, Exception::TenthFromHouse) => ((sign + 9) % 12, first, true),
        _ => (first, first, false),
    }
}

fn wrong(charts: &[Chart], lords: Lords, exception: Exception) -> usize {
    charts
        .iter()
        .flat_map(|chart| (0..12).map(move |house| (chart, house)))
        .filter(|(chart, house)| pada(chart, *house, lords, exception) != chart.padas[*house])
        .count()
}

fn claims(charts: &[Chart]) -> Vec<Claim> {
    let padas = charts.len() * 12;
    let (in_house, in_seventh) = charts
        .iter()
        .flat_map(|chart| (0..12).map(move |house| (chart, house)))
        .fold((0, 0), |(one, seven), (chart, house)| {
            let (_, first, _) = pada(chart, house, Lords::Parashari, Exception::None);
            let sign = (chart.lagna + house) % 12;
            match (first + 12 - sign) % 12 {
                0 => (one + 1, seven),
                6 => (one, seven + 1),
                _ => (one, seven),
            }
        });
    vec![
        Claim::counted(
            "a house's pada is the count from its sign to the sign's lord, counted again from the lord, with the catalogue's lords; one that falls in the house or the seventh from it moves to the tenth from itself",
            wrong(charts, Lords::Parashari, Exception::TenthFromPada),
            padas,
        )
        .with_note(format!(
            "the exception moves {} in the house and {} in the seventh",
            count(in_house),
            count(in_seventh)
        )),
        Claim::counted(
            "the same with Ketu for Scorpio and Rahu for Aquarius",
            wrong(charts, Lords::Jaimini, Exception::TenthFromPada),
            padas,
        ),
        Claim::counted(
            "the same with the stronger of the two lords by ch. 46's ladder, the nodes as co-lords (BPHS ch. 29 v. 7, `jaimini.node_co_lordship = BOTH`)",
            wrong(charts, Lords::StrongerOfTwo, Exception::TenthFromPada),
            padas,
        ),
        Claim::counted(
            "the same with the exception moving a pada to the tenth from the **house**",
            wrong(charts, Lords::Parashari, Exception::TenthFromHouse),
            padas,
        ),
        Claim::counted(
            "the same with no exception",
            wrong(charts, Lords::Parashari, Exception::None),
            padas,
        ),
        Claim::counted(
            "the arudha lagna is the first house's pada",
            charts
                .iter()
                .filter(|chart| chart.padas[0].0 != chart.arudha_lagna)
                .count(),
            charts.len(),
        ),
    ]
}

fn page(root: &Path) -> Result<String, String> {
    let charts = charts(root)?;
    if charts.is_empty() {
        return Err(String::from("the corpus records no arudha padas"));
    }
    let mut out = String::new();
    let _ = write!(
        out,
        "# The arudha padas, measured\n\n\
         Status: `generated` by `cargo xtask arudhas` over the conformance corpus's \
         `houses.arudha_padas`, 2026-09-15. Do not edit: `check-arudhas` regenerates this \
         page and fails on any difference.\n\n\
         The corpus records the twelve padas of {charts} charts, each with the sign it first \
         counts to and whether an exception moved it, beside the lagna and every graha's sign \
         they are computed from: {padas} padas.\n\n\
         ## What the corpus decides\n\n{table}\n",
        charts = count(charts.len()),
        padas = count(charts.len() * 12),
        table = table(&claims(&charts)),
    );
    out.push_str(
        "## What it means for the module\n\n\
         **The padas are a function of the lagna's sign and the grahas' signs**, so they are \
         points and not a school's module: `teistro-points` computes them and the sign-based \
         dashas read the first. The lords are the catalogue's, which is the reading the \
         recording engine takes for the padas even though its Jaimini dashas count Scorpio \
         and Aquarius to Ketu and Rahu first; that difference is the engine's, measured here \
         and in `rashi-dashas-measured.md`.\n\n\
         **The exception counts from the pada**, so a pada in the seventh lands in the \
         fourth house; the tenth from the house is refused wherever the exception applies \
         to the seventh.\n\n\
         **v. 7 reaches the padas only through the co-lordship.** Its द्विनाथ, a sign with two \
         lords, counted up to the stronger, is a question only where a node co-lords Scorpio or \
         Aquarius. Under `jaimini.node_co_lordship = NONE`, the default, each has one lord and it \
         is the catalogue's, so the shipped padas are the verse's as well as the corpus's; a \
         consumer who makes the nodes co-lords gets the stronger lord, and the row above counts \
         what that moves against a corpus that never does (crux C135). The count itself is the \
         shipped `points::arudha::arudha_by`, so a change to it moves this page.\n",
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
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask arudhas") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}
