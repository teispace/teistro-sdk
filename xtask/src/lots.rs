//! The lots, measured (cruxes C221–C222): the shapes Valens's text gives
//! them held on every birth, what each reading of Fortune by night moves
//! on the corpus's night births, and the premise the third reading's
//! horizon rests on counted rather than believed.
//!
//! The worked examples that are the acceptance test live in
//! `crates/hellenistic` as unit tests; this page asks how often the
//! choices they could not decide fall on real skies.

use std::fmt::Write as _;
use std::path::Path;

use teistro::catalogue::{Graha, Rashi};
use teistro::{FortuneRule, Lot, LotReading, LotRequest, Sect};

use crate::births::{CHARTS, births};
use crate::fortitudes::tropical;
use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, count, fill, table};

const PAGE: &str = "docs/03-design/lots-measured.md";

/// How far two longitudes may stand apart and still be one point: the
/// arithmetic is exact but for rounding.
const SAME_DEG: f64 = 1e-9;

/// One birth's lots under each reading of Fortune by night, with the two
/// longitudes the hemisphere is read from.
struct Read {
    ascendant_deg: f64,
    moon_deg: f64,
    /// Valens II.22, the default: reversed by night.
    reversed: LotReading,
    /// Lilly's: the same by day and night.
    lilly: LotReading,
    /// Valens III.11: reversed by night while the Moon is up.
    moon_up: LotReading,
}

impl Read {
    fn readings(&self) -> [&LotReading; 3] {
        [&self.reversed, &self.lilly, &self.moon_up]
    }

    fn night(&self) -> bool {
        self.reversed.sect == Sect::Night
    }
}

/// Where a reading put a lot; every reading here asked for all of them.
fn at(read: &LotReading, lot: Lot) -> f64 {
    read.lots
        .iter()
        .find(|placed| placed.lot == lot)
        .map_or(f64::NAN, |placed| placed.place.longitude_deg)
}

fn sign(read: &LotReading, lot: Lot) -> Rashi {
    Rashi::of_longitude(at(read, lot))
}

/// Whether two longitudes are one point, either side of 0°.
fn same(one: f64, other: f64) -> bool {
    let apart = (one - other).rem_euclid(360.0);
    apart < SAME_DEG || apart > 360.0 - SAME_DEG
}

/// The shapes the text gives, held on every birth under every reading.
fn structural_claims(reads: &[Read]) -> Vec<Claim> {
    let (mut mirrors, mut mirror_wrong) = (0, 0);
    let mut basis_wrong = 0;
    for read in reads {
        for reading in read.readings() {
            mirrors += 1;
            let reflected = 2.0 * read.ascendant_deg - at(reading, Lot::Fortune);
            mirror_wrong += usize::from(!same(at(reading, Lot::Daimon), reflected));
            let basis = (at(reading, Lot::Basis) - read.ascendant_deg).rem_euclid(360.0);
            basis_wrong += usize::from(basis > 180.0 + SAME_DEG);
        }
    }
    vec![
        Claim::counted(
            "Daimon is Fortune reflected in the ascendant, under every rule for Fortune by night",
            mirror_wrong,
            mirrors,
        ),
        Claim::counted(
            "Basis stands no more than half the circle past the ascendant, \"from the nearest Lot to the other\" (II.22)",
            basis_wrong,
            mirrors,
        ),
    ]
}

/// What each reading of Fortune by night moves, and the premise III.11's
/// horizon rests on.
fn night_claims(reads: &[Read]) -> Vec<Claim> {
    let nights: Vec<&Read> = reads.iter().filter(|read| read.night()).collect();
    let (mut set, mut lilly_moves, mut moon_up_moves) = (0, 0, 0);
    let (mut love_moves, mut hemisphere_wrong) = (0, 0);
    for read in &nights {
        let moon_up = read.moon_up.fortune_reversed;
        set += usize::from(!moon_up);
        lilly_moves +=
            usize::from(sign(&read.lilly, Lot::Fortune) != sign(&read.reversed, Lot::Fortune));
        moon_up_moves +=
            usize::from(sign(&read.moon_up, Lot::Fortune) != sign(&read.reversed, Lot::Fortune));
        love_moves += usize::from(sign(&read.lilly, Lot::Love) != sign(&read.reversed, Lot::Love));
        // Above the earth by the ecliptic: in the half of the zodiac that
        // runs from the descendant up to the ascendant.
        let by_ecliptic = (read.moon_deg - read.ascendant_deg).rem_euclid(360.0) > 180.0;
        hemisphere_wrong += usize::from(by_ecliptic != moon_up);
    }
    let of = count(nights.len());
    vec![
        Claim::stated(
            "C221: night births whose Fortune changes sign under Lilly's rule instead of Valens's reversal (II.22)",
            Verdict::Holds,
            format!("{} of {of}", count(lilly_moves)),
        ),
        Claim::stated(
            "C221: night births whose Fortune changes sign under III.11 instead of II.22, which can happen only with the Moon set",
            Verdict::Holds,
            format!(
                "{} of {of}; the Moon set at {}",
                count(moon_up_moves),
                count(set)
            ),
        ),
        Claim::stated(
            "night births whose Love changes sign under Lilly's Fortune, the two reversals no longer cancelling",
            Verdict::Holds,
            format!("{} of {of}", count(love_moves)),
        ),
        Claim::counted(
            "C221's premise: the Moon's ecliptic hemisphere from the ascendant says whether it is up, as its altitude does",
            hemisphere_wrong,
            nights.len(),
        )
        .with_note("its latitude carries it across the horizon the ecliptic does not"),
    ]
}

/// Each lot's sign under the default reading, over the births, so that a
/// lot that never moves is seen to.
fn sign_spread(reads: &[Read]) -> String {
    let mut out =
        String::from("| lot | signs it falls in | most births in one sign |\n|---|---|---|\n");
    for lot in Lot::ALL {
        let mut seen = [0_usize; 12];
        for read in reads {
            let index = Rashi::ALL
                .iter()
                .position(|one| *one == sign(&read.reversed, lot))
                .unwrap_or_default();
            if let Some(slot) = seen.get_mut(index) {
                *slot += 1;
            }
        }
        let signs = seen.iter().filter(|&&births| births > 0).count();
        let most = seen.iter().copied().max().unwrap_or_default();
        let _ = writeln!(out, "| `{lot:?}` | {signs} | {} |", count(most));
    }
    out
}

fn page(root: &Path) -> Result<String, String> {
    let sdk = tropical()?;
    let births = births(root, &sdk)?;
    if births.is_empty() {
        return Err(format!("{CHARTS} records no chart"));
    }
    let reads = births
        .iter()
        .map(|birth| {
            let read = |request: LotRequest| {
                sdk.chart()
                    .lots_with_request(&birth.document, &Lot::ALL, request)
                    .map_err(|why| format!("{}: {why}", birth.name))
            };
            let ascendant_deg = sdk
                .chart()
                .angles(&birth.document)
                .map_err(|why| format!("{}: {why}", birth.name))?
                .ascendant_deg;
            let moon_deg = birth
                .document
                .foundation
                .graha(Graha::Moon)
                .ok_or_else(|| format!("{}: no Moon", birth.name))?
                .longitude_deg;
            Ok(Read {
                ascendant_deg,
                moon_deg,
                reversed: read(LotRequest::VALENS)?,
                lilly: read(LotRequest::VALENS.with_fortune(FortuneRule::DayAndNight))?,
                moon_up: read(LotRequest::VALENS.with_fortune(FortuneRule::ReversedWhileMoonUp))?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let claims: Vec<Claim> = structural_claims(&reads)
        .into_iter()
        .chain(night_claims(&reads))
        .collect();
    let mut out = String::from(
        "# The lots, measured\n\n\
         Status: `generated` by `cargo xtask lots` from the corpus's \
         recorded births, 2026-10-02. Do not edit: `check-lots` \
         regenerates this page and fails on any difference.\n\n\
         Valens's lots (`hellenistic-lots.md`) are held to his two worked \
         examples of Fortune by night by the unit tests of \
         `crates/hellenistic`. Those examples give signs and no degrees, \
         and both have the Moon up, so they cannot decide between his \
         readings; this page counts how often the choice falls on real \
         skies. ",
    );
    let _ = write!(
        out,
        "It reads each of the corpus's {} births in the tropical zodiac \
         through `ChartArea::lots_with_request`, every lot at once, under \
         each `FortuneRule`; {} of them are night births by Valens's \
         horizon.\n\n",
        count(reads.len()),
        count(reads.iter().filter(|read| read.night()).count())
    );
    out.push_str(&table(&claims));
    out.push_str("\n## How many signs each lot falls in\n\n");
    out.push_str(&sign_spread(&reads));
    out.push_str(
        "\n## What it means\n\n\
         The first two rows hold the shapes the text gives on every birth \
         and every reading. The next three count what the readings of \
         Fortune by night move: Lilly's against Valens's II.22, which \
         every lot built on Fortune inherits, and III.11's, which parts \
         from II.22 only on a night whose Moon has set. The last row is \
         the premise of reading III.11's \"above the earth\" by the \
         Moon's altitude rather than by the ecliptic from the ascendant: \
         where it is falsified, the two disagree on a real sky, and the \
         altitude is the one that says whether the Moon has set.\n",
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
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask lots") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}
