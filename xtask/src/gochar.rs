//! The measurement pass over **gochar**, the transits read from the natal
//! Moon (`03-design/gochar.md`).
//!
//! Nothing records a transit's verdict — the corpus has no gochar and
//! `PyJHora` no vedha — so this pass measures what the table *does* over
//! the real sky rather than holding it to a recording. Two things only the
//! sky can show:
//!
//! - over the recorded births, through `sdk.chart().gochar`, how the
//!   verdicts fall, and how many each fork moves: the reference (C139) and
//!   the nodes' readings (C136, C137, C140);
//! - over sixty years of days read from every reference sign, **who
//!   obstructs whom**: each pair of grahas either obstructs whenever it
//!   stands in the other's vedha house, is spared by a named exemption, or
//!   never stands there because the sky forbids it. Each of the three
//!   lists is written here on its own and held both ways, so a pair the
//!   code spares that the verses do not, or an elongation bound that is
//!   wrong, fails the pass.
//!
//! `cargo xtask gochar` writes the page; `check-gochar` regenerates it in
//! memory and fails on any difference.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::Path;

use teistro::catalogue::{Graha, Rashi};
use teistro::gochar::{GRAHAS, GocharReading, GocharRules, Transit, Verdict, gochar};
use teistro::quantity::{JulianDay, Utc};
use teistro::settings::{NodeObstruction, NodeVedha};
use teistro::{Context, Ephemeris, GocharFrom, GocharRequest};

use crate::births::{Birth, births};
use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict as Decided, capitalised, count, spelled, table};

const PAGE: &str = "docs/03-design/gochar-measured.md";

/// The profile the corpus was recorded under, which the births are
/// founded in.
const PROFILE: &str = "conformance-baseline";

/// The first of each month of 2026, at 0h UT, which the births are read
/// over.
const YEAR_FROM: f64 = 2_461_041.5;
const MONTH_DAYS: f64 = 30.4375;

/// The sky's span: every day from 1 January 1960 to 1 January 2020, two
/// cycles of Saturn and three of the nodes.
const SKY_FROM: f64 = 2_436_934.5;
const SKY_DAYS: u32 = 21_915;

/// Who the verses spare, written out from them rather than read back from
/// the code: the Sun and Saturn each other (vv. 3, 5), the Moon and
/// Mercury each other (vv. 4, 6), and the nodes each other (C140).
const SPARED: [(Graha, Graha, &str); 6] = [
    (
        Graha::Sun,
        Graha::Saturn,
        "v. 3: no vedha between father and son",
    ),
    (
        Graha::Saturn,
        Graha::Sun,
        "v. 5: the Sun does not obstruct Saturn",
    ),
    (
        Graha::Moon,
        Graha::Mercury,
        "v. 4: Mercury does not obstruct the Moon",
    ),
    (
        Graha::Mercury,
        Graha::Moon,
        "v. 6: the Moon does not obstruct Mercury",
    ),
    (
        Graha::Rahu,
        Graha::Ketu,
        "C140: the nodes do not obstruct each other",
    ),
    (
        Graha::Ketu,
        Graha::Rahu,
        "C140: the nodes do not obstruct each other",
    ),
];

/// How many signs apart two grahas that never part far can stand: the
/// greatest elongation in degrees crosses at most that many sign
/// boundaries, rounded up. Mercury stays within about 28° of the Sun and
/// Venus within about 47°. Each bound must explain a pair the sky never
/// tests, or it is a list entry nothing needs and the pass says so;
/// Mercury and Venus, within 75° of each other, explain none, since
/// every vedha offset between them is within three signs.
const BOUNDS: [(Graha, Graha, u8, &str); 2] = [
    (
        Graha::Sun,
        Graha::Mercury,
        1,
        "Mercury stays within 28° of the Sun",
    ),
    (
        Graha::Sun,
        Graha::Venus,
        2,
        "Venus stays within 47° of the Sun",
    ),
];

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
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask gochar") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

fn index(graha: Graha) -> usize {
    graha as usize
}

/// Each graha's verdicts, the Sun to Ketu: good, obstructed, not good.
type Verdicts = [[usize; 3]; 9];

fn tally(verdicts: &mut Verdicts, reading: &GocharReading) {
    for read in &reading.grahas {
        let column = match read.verdict {
            Verdict::Good => 0,
            Verdict::Obstructed => 1,
            Verdict::NotGood => 2,
        };
        if let Some(cell) = verdicts
            .get_mut(index(read.graha))
            .and_then(|row| row.get_mut(column))
        {
            *cell += 1;
        }
    }
}

/// How many grahas' verdicts differ between two readings of one instant.
fn moved(a: &GocharReading, b: &GocharReading) -> usize {
    a.grahas
        .iter()
        .zip(&b.grahas)
        .filter(|(a, b)| a.verdict != b.verdict)
        .count()
}

fn transits_of(reading: &GocharReading) -> [Transit; 9] {
    reading.grahas.each_ref().map(|read| read.transit)
}

/// The readings of the nodes the settings offer beside the text's.
const FORKS: [(&str, GocharRules); 3] = [
    (
        "`node_vedha = NONE` (C136)",
        GocharRules {
            node_vedha: NodeVedha::None,
            ..GocharRules::TEXT
        },
    ),
    (
        "`node_obstruction = EACH_OTHER_TOO` (C140)",
        GocharRules {
            node_obstruction: NodeObstruction::EachOtherToo,
            ..GocharRules::TEXT
        },
    ),
    (
        "`node_obstruction = NONE` (C137)",
        GocharRules {
            node_obstruction: NodeObstruction::None,
            ..GocharRules::TEXT
        },
    ),
];

/// What the births say, read through the façade.
struct OverBirths {
    births: usize,
    readings: usize,
    verdicts: Verdicts,
    by_lagna: usize,
    forks: [usize; 3],
}

fn over_births(sdk: &Context, born: &[Birth]) -> Result<OverBirths, String> {
    let year = GocharRequest::over(
        (0_u32..12)
            .map(|month| JulianDay::<Utc>::literal(YEAR_FROM + MONTH_DAYS * f64::from(month))),
    );
    let mut out = OverBirths {
        births: born.len(),
        readings: 0,
        verdicts: [[0; 3]; 9],
        by_lagna: 0,
        forks: [0; 3],
    };
    for birth in born {
        let read = |from| {
            sdk.chart()
                .gochar(&birth.document, &year.clone().counted_from(from))
                .map(|read| read.value)
                .map_err(|why| format!("{}: {why}", birth.name))
        };
        let moon = read(GocharFrom::Moon)?;
        let lagna = read(GocharFrom::Lagna)?;
        for (moon, lagna) in moon.iter().zip(&lagna) {
            out.readings += 1;
            tally(&mut out.verdicts, moon);
            out.by_lagna += moved(moon, lagna);
            let transits = transits_of(moon);
            for (slot, (_, rules)) in out.forks.iter_mut().zip(FORKS) {
                *slot += moved(moon, &gochar(moon.reference, &transits, rules));
            }
        }
    }
    Ok(out)
}

/// What the sky says, read from every reference sign.
struct OverSky {
    readings: usize,
    verdicts: Verdicts,
    /// `[graha][other]`: how often `other` stood in `graha`'s vedha house
    /// while `graha` stood in a good house, and how often it obstructed.
    stood: [[usize; 9]; 9],
    obstructed: [[usize; 9]; 9],
    /// Each graha's vedha offsets, the vedha house less the good house,
    /// as the readings gave them.
    offsets: [BTreeSet<u8>; 9],
    /// Under the literal reading of the nodes (C140): a node in a good
    /// house, and how many of those the other node obstructed.
    literal_good: usize,
    literal_by_the_other: usize,
}

fn over_sky(sdk: &Context, born: &[Birth]) -> Result<OverSky, String> {
    let natal = born.first().ok_or("no recorded birth to read the sky at")?;
    let days = GocharRequest::over(
        (0..SKY_DAYS).map(|day| JulianDay::<Utc>::literal(SKY_FROM + f64::from(day))),
    );
    let read = sdk
        .chart()
        .gochar(&natal.document, &days)
        .map_err(|why| format!("the sky: {why}"))?
        .value;
    let literal = GocharRules {
        node_obstruction: NodeObstruction::EachOtherToo,
        ..GocharRules::TEXT
    };
    let mut out = OverSky {
        readings: 0,
        verdicts: [[0; 3]; 9],
        stood: [[0; 9]; 9],
        obstructed: [[0; 9]; 9],
        offsets: Default::default(),
        literal_good: 0,
        literal_by_the_other: 0,
    };
    for day in &read {
        let transits = transits_of(day);
        for reference in Rashi::ALL {
            let reading = gochar(reference, &transits, GocharRules::TEXT);
            out.readings += 1;
            tally(&mut out.verdicts, &reading);
            for read in &reading.grahas {
                let Some(vedha) = read.vedha_house else {
                    continue;
                };
                let at = index(read.graha);
                if let Some(offsets) = out.offsets.get_mut(at) {
                    offsets.insert((vedha + 12 - read.house) % 12);
                }
                for other in GRAHAS.into_iter().filter(|other| *other != read.graha) {
                    let there = reading.grahas.get(index(other)).map(|o| o.house);
                    if there != Some(vedha) {
                        continue;
                    }
                    if let Some(cell) = out.stood.get_mut(at).and_then(|r| r.get_mut(index(other)))
                    {
                        *cell += 1;
                    }
                    if read.obstructed_by.contains(&other)
                        && let Some(cell) = out
                            .obstructed
                            .get_mut(at)
                            .and_then(|r| r.get_mut(index(other)))
                    {
                        *cell += 1;
                    }
                }
            }
            let literally = gochar(reference, &transits, literal);
            for (node, other) in [(Graha::Rahu, Graha::Ketu), (Graha::Ketu, Graha::Rahu)] {
                if let Some(read) = literally.grahas.get(index(node))
                    && read.good_house
                {
                    out.literal_good += 1;
                    if read.obstructed_by.contains(&other) {
                        out.literal_by_the_other += 1;
                    }
                }
            }
        }
    }
    Ok(out)
}

/// Why a pair never stands in the other's vedha house, if an elongation
/// bound says it cannot: every one of the obstructed graha's vedha offsets
/// lies further round the circle than the pair ever parts.
fn bounded(graha: Graha, other: Graha, offsets: &BTreeSet<u8>) -> Option<&'static str> {
    let (bound, why) = BOUNDS.iter().find_map(|(a, b, bound, why)| {
        ((*a, *b) == (graha, other) || (*b, *a) == (graha, other)).then_some((*bound, *why))
    })?;
    let beyond = |offset: &u8| (*offset).min(12 - offset) > bound;
    (!offsets.is_empty() && offsets.iter().all(beyond)).then_some(why)
}

fn spared(graha: Graha, other: Graha) -> Option<&'static str> {
    SPARED
        .iter()
        .find_map(|(a, b, why)| ((*a, *b) == (graha, other)).then_some(*why))
}

fn name(graha: Graha) -> String {
    capitalised(&graha.key().to_lowercase())
}

fn verdict_rows(verdicts: &Verdicts) -> String {
    let mut rows = String::from(
        "| graha | good | obstructed | not good | obstructed, of the good houses |\n|---|---:|---:|---:|---:|\n",
    );
    for graha in GRAHAS {
        let [good, obstructed, not_good] = verdicts.get(index(graha)).copied().unwrap_or_default();
        let share = if good + obstructed == 0 {
            String::from("—")
        } else {
            #[allow(
                clippy::cast_precision_loss,
                reason = "counts far below 2^52 as a percentage"
            )]
            let percent = 100.0 * obstructed as f64 / (good + obstructed) as f64;
            format!("{percent:.1}%")
        };
        let _ = writeln!(
            rows,
            "| {} | {} | {} | {} | {share} |",
            name(graha),
            count(good),
            count(obstructed),
            count(not_good)
        );
    }
    rows
}

/// The obstruction matrix and the three classes' disagreements.
struct Pairs {
    rows: String,
    spared_wrong: usize,
    bounded_wrong: usize,
    partial: usize,
    compared: usize,
}

fn pairs(sky: &OverSky) -> Pairs {
    let mut rows = String::from("| obstructed ↓ by → |");
    for other in GRAHAS {
        let _ = write!(rows, " {} |", name(other));
    }
    rows.push_str("\n|---|");
    rows.push_str(&"---:|".repeat(GRAHAS.len()));
    rows.push('\n');
    let mut out = Pairs {
        rows: String::new(),
        spared_wrong: 0,
        bounded_wrong: 0,
        partial: 0,
        compared: 0,
    };
    let empty = BTreeSet::new();
    let mut bounds_used = BTreeSet::new();
    for graha in GRAHAS {
        let _ = write!(rows, "| {} |", name(graha));
        for other in GRAHAS {
            if other == graha {
                rows.push_str(" — |");
                continue;
            }
            out.compared += 1;
            let stood = sky.stood[index(graha)][index(other)];
            let obstructed = sky.obstructed[index(graha)][index(other)];
            let offsets = sky.offsets.get(index(graha)).unwrap_or(&empty);
            let is_spared = spared(graha, other).is_some();
            let is_bounded = bounded(graha, other, offsets).is_some();
            if let Some(why) = bounded(graha, other, offsets) {
                bounds_used.insert(why);
            }
            // Both ways: a listed exemption must be one the sky tested and
            // the code honoured, and a spared pair must be listed; a bound
            // must hold on the sky, and a pair the sky never tested must
            // have a bound.
            if is_spared != (stood > 0 && obstructed == 0) {
                out.spared_wrong += 1;
            }
            if is_bounded != (stood == 0) {
                out.bounded_wrong += 1;
            }
            if obstructed != 0 && obstructed != stood {
                out.partial += 1;
            }
            let cell = if stood == 0 {
                String::from("never")
            } else if obstructed == 0 {
                format!("spared ({})", count(stood))
            } else {
                count(obstructed)
            };
            let _ = write!(rows, " {cell} |");
        }
        rows.push('\n');
    }
    // A bound that explains no pair is an entry nothing needs.
    out.bounded_wrong += BOUNDS
        .iter()
        .filter(|(_, _, _, why)| !bounds_used.contains(why))
        .count();
    out.rows = rows;
    out
}

#[expect(
    clippy::too_many_lines,
    reason = "a page reads top to bottom, and splitting it would scatter its prose"
)]
fn page(root: &Path) -> Result<String, String> {
    let sdk = Context::builder()
        .profile(PROFILE)
        .ephemeris([Ephemeris::Builtin])
        .build()
        .map_err(|why| format!("{PROFILE}: {why}"))?;
    let born = births(root, &sdk)?;
    let births = over_births(&sdk, &born)?;
    let sky = over_sky(&sdk, &born)?;
    let matrix = pairs(&sky);

    let mut out = String::new();
    let _ = writeln!(
        out,
        "# Gochar, measured\n\n\
         Status: `generated` by `cargo xtask gochar`. Do not edit:\n\
         `check-gochar` regenerates this page and fails on any difference.\n\n\
         The design this measures is `gochar.md`. **Nothing records a\n\
         transit's verdict** — the corpus has no gochar and PyJHora no\n\
         vedha — so the table is held cell by cell by the crate's tests, and\n\
         this page measures what it does over the real sky: how the verdicts\n\
         fall, how many each fork moves, and who obstructs whom.\n"
    );
    let _ = writeln!(
        out,
        "## 1. Over the recorded births\n\n\
         Each of the {} recorded births, founded under `{PROFILE}`, read\n\
         through `sdk.chart().gochar` on the first of each month of 2026:\n\
         {} readings of nine grahas, counted from the natal Moon (v. 1).\n\n{}",
        spelled(births.births),
        count(births.readings),
        verdict_rows(&births.verdicts)
    );
    let graha_readings = births.readings * GRAHAS.len();
    let mut forks = format!(
        "| counted or read otherwise | verdicts moved, of {} |\n|---|---:|\n| from the lagna instead of the Moon (C139) | {} |\n",
        count(graha_readings),
        count(births.by_lagna)
    );
    for ((fork, _), moved) in FORKS.iter().zip(births.forks) {
        let _ = writeln!(forks, "| {fork} | {} |", count(moved));
    }
    let _ = writeln!(
        out,
        "What each fork moves, over the same readings:\n\n{forks}"
    );
    let _ = writeln!(
        out,
        "## 2. Who obstructs whom, over sixty years of sky\n\n\
         Every day from 1960 to 2020 ({} days), each read from all twelve\n\
         reference signs: {} readings. The verdicts:\n\n{}",
        count(SKY_DAYS as usize),
        count(sky.readings),
        verdict_rows(&sky.verdicts)
    );
    let _ = writeln!(
        out,
        "Each cell counts the times the column's graha **obstructed** the\n\
         row's, standing in its vedha house while the row's graha stood in a\n\
         good house. `spared` is a pair that stood there and never\n\
         obstructed, with the times it stood; `never` is a pair the sky never\n\
         put there.\n\n{}",
        matrix.rows
    );
    let spared_list = SPARED
        .iter()
        .map(|(a, b, why)| format!("- {} by {}: {why}", name(*a), name(*b)))
        .collect::<Vec<_>>()
        .join("\n");
    let mut bound_list = Vec::new();
    for graha in GRAHAS {
        for other in GRAHAS {
            if let Some(offsets) = sky.offsets.get(index(graha))
                && let Some(why) = bounded(graha, other, offsets)
            {
                bound_list.push(format!("- {} by {}: {why}", name(graha), name(other)));
            }
        }
    }
    let _ = writeln!(
        out,
        "The pairs spared, as the verses name them:\n\n{spared_list}\n\n\
         The pairs the sky never puts there, each by the elongation it\n\
         never exceeds against every vedha offset of the obstructed graha:\n\n{}\n",
        bound_list.join("\n")
    );
    let _ = writeln!(
        out,
        "## 3. The nodes read literally (C140)\n\n\
         Read with every graha obstructing, the nodes each other too, a node\n\
         stood in a good house {} times over the sky, and the other node\n\
         obstructed it {} times: the nodes always stand opposite and the\n\
         Sun's vedha pairs, theirs under `LIKE_THE_SUN`, are opposite houses,\n\
         so the literal reading leaves v. 2's good houses for the nodes never\n\
         good. The default spares them each other.\n",
        count(sky.literal_good),
        count(sky.literal_by_the_other)
    );

    let claims = vec![
        Claim::counted(
            "a pair is spared exactly where the verses or C140 name it",
            matrix.spared_wrong,
            matrix.compared,
        ),
        Claim::counted(
            "a pair never stands in the other's vedha house exactly where an elongation bound forbids it",
            matrix.bounded_wrong,
            matrix.compared,
        ),
        Claim::counted(
            "a graha standing in another's vedha house obstructs it every time or never",
            matrix.partial,
            matrix.compared,
        ),
        Claim::counted(
            "under the literal reading the other node obstructs every node in a good house",
            sky.literal_good - sky.literal_by_the_other,
            sky.literal_good,
        ),
    ];
    let _ = writeln!(out, "## 4. What this pass decides\n");
    let _ = writeln!(out, "{}", table(&claims));
    let falsified = claims
        .iter()
        .filter(|claim| claim.verdict == Decided::Falsified)
        .count();
    let _ = writeln!(
        out,
        "{} The first three hold every ordered pair of grahas to one of\n\
         three lists written in this pass and not read back from the code,\n\
         both ways; the fourth is the measurement C140's default rests on.",
        match falsified {
            0 => format!("None of the {} claims is falsified.", spelled(claims.len())),
            1 => format!("One of the {} claims is falsified.", spelled(claims.len())),
            _ => format!(
                "{} of the {} claims are falsified.",
                capitalised(&spelled(falsified)),
                spelled(claims.len())
            ),
        }
    );
    Ok(crate::measure::fill(&out))
}
