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
use teistro_core::catalogue::{Catalogued, Graha, Rashi};

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

fn lord(sign: usize, lords: Lords) -> Graha {
    match (lords, sign) {
        (Lords::Jaimini, 7) => Graha::Ketu,
        (Lords::Jaimini, 10) => Graha::Rahu,
        _ => Rashi::ALL[sign].attributes().lord,
    }
}

/// A house's pada: the count from its sign to its lord, counted again from
/// the lord; and whether the exception moved it.
fn pada(chart: &Chart, house: usize, lords: Lords, exception: Exception) -> (usize, usize, bool) {
    let sign = (chart.lagna + house) % 12;
    let lord = lord(sign, lords);
    let at = chart
        .signs
        .iter()
        .find(|(graha, _)| *graha == lord)
        .map_or(sign, |(_, at)| *at);
    let first = (at + (at + 12 - sign) % 12) % 12;
    let from_house = (first + 12 - sign) % 12;
    if from_house == 0 || from_house == 6 {
        match exception {
            Exception::TenthFromPada => ((first + 9) % 12, first, true),
            Exception::TenthFromHouse => ((sign + 9) % 12, first, true),
            Exception::None => (first, first, false),
        }
    } else {
        (first, first, false)
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
         to the seventh.\n",
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
