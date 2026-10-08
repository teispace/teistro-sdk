//! Rectification's premises, measured (`rectification.md`, X2 and X8):
//! how much of a day the purifier keeps under each reading, and how far
//! the verse's pranapada stands from the SDK's point.
//!
//! Each recorded birth's whole day, sunrise to the next sunrise, is
//! narrowed under `conformance-baseline` with the purifier as a weight, so
//! every run comes back with every clause. A reading that drops a
//! purifier or v. 76's extension is then read from the same runs: its
//! clauses are a subset, so its edges are among them. The readings that
//! change a clause's value (the printed pranapada, the SDK's point,
//! Gulika's start) are narrowed again.
//!
//! The corpus's instants are synthetic, not trusted birth times, so the
//! page says how often a recorded instant falls in a kept run and no
//! more: whether trusted times survive the bar waits for a corpus of them.

use std::fmt::Write as _;
use std::path::Path;

use teistro::catalogue::{Graha, Rashi};
use teistro::rectification::{
    Answer, Clause, GulikaAt, PranapadaRule, Purifier, PurifyAs, Reference, Rules, Window,
    pranapada_deg,
};

use crate::births::{Birth, CHARTS, births, conformance};
use crate::generated::{Output, check, write};
use crate::measure::{count, fill, median, share};

const PAGE: &str = "docs/03-design/rectification-measured.md";

/// Which clauses a reading reads.
type Reads = fn(&Clause) -> bool;

/// The readings read from the default run's clauses.
const SUBSETS: [(&str, Reads); 6] = [
    ("all three, with v. 76", |_| true),
    ("all three, without v. 76", |c| {
        c.reference == Reference::Itself
    }),
    ("the pranapada alone", |c| c.purifier == Purifier::Pranapada),
    ("Gulika alone, with v. 76", |c| {
        c.purifier == Purifier::Gulika
    }),
    ("Gulika alone, without v. 76", |c| {
        c.purifier == Purifier::Gulika && c.reference == Reference::Itself
    }),
    ("the Moon alone", |c| c.purifier == Purifier::Moon),
];

/// The readings that change a clause, each narrowed with its purifier
/// alone.
fn variants() -> [(&'static str, Rules); 3] {
    let weight = Rules {
        purify_as: PurifyAs::Weight,
        ..Rules::default()
    };
    let pranapada = Rules {
        gulika: false,
        moon: false,
        ..weight
    };
    [
        (
            "the pranapada alone, the printed reading",
            Rules {
                pranapada_rule: PranapadaRule::PrintedExample,
                ..pranapada
            },
        ),
        (
            "the pranapada alone, the SDK's point",
            Rules {
                pranapada_rule: PranapadaRule::SdkPoint,
                ..pranapada
            },
        ),
        (
            "Gulika alone, with v. 76, at the start of Saturn's eighth",
            Rules {
                pranapada: false,
                moon: false,
                gulika_at: GulikaAt::Start,
                ..weight
            },
        ),
    ]
}

/// One birth's day, narrowed.
struct Read {
    /// The day's length, in days.
    days: f64,
    /// The recorded instant.
    at: f64,
    /// Under the default rules, as a weight.
    all: Answer,
    /// Under each of [`variants`], in order.
    variants: Vec<Answer>,
}

/// What a reading keeps of one day: the kept days, and whether the
/// recorded instant was kept.
fn kept(answer: &Answer, at: f64, reads: Reads) -> (f64, bool) {
    let mut days = 0.0;
    let mut instant = false;
    for run in &answer.intervals {
        if run.verdict.clauses.iter().any(|c| c.held && reads(c)) {
            days += run.to.get() - run.from.get();
            if run.from.get() <= at && at < run.to.get() {
                instant = true;
            }
        }
    }
    (days, instant)
}

/// The days narrowed, the births refused with why, and the births.
type Measured = (Vec<Read>, Vec<String>, Vec<Birth>);

fn measure(root: &Path) -> Result<Measured, String> {
    let sdk = conformance()?;
    let births = births(root, &sdk)?;
    if births.is_empty() {
        return Err(format!("{CHARTS} records no chart"));
    }
    let weight = Rules {
        purify_as: PurifyAs::Weight,
        ..Rules::default()
    };
    let mut read = Vec::new();
    let mut refused = Vec::new();
    for birth in &births {
        let day = &birth.document.foundation.day.day;
        let place = birth.document.foundation.place;
        let narrowed = Window::between(day.sunrise, day.next_sunrise).and_then(|window| {
            let area = sdk.chart();
            let all = area.rectify(window, &place, birth.offset, &weight)?.value;
            let variants = variants()
                .iter()
                .map(|(_, rules)| {
                    Ok(sdk
                        .chart()
                        .rectify(window, &place, birth.offset, rules)?
                        .value)
                })
                .collect::<Result<Vec<_>, teistro::Error>>()?;
            Ok((window, all, variants))
        });
        match narrowed {
            Ok((window, all, variants)) => read.push(Read {
                days: window.days(),
                at: birth.at(),
                all,
                variants,
            }),
            Err(why) => refused.push(format!("`{}` ({})", birth.name, why.message)),
        }
    }
    Ok((read, refused, births))
}

/// One reading's row: the share of all the days kept, the least and most
/// kept day, and how many recorded instants were kept.
fn row(label: &str, read: &[Read], of: impl Fn(&Read) -> (f64, bool)) -> String {
    let shares: Vec<f64> = read.iter().map(|one| of(one).0 / one.days).collect();
    let kept_days: f64 = read.iter().map(|one| of(one).0).sum();
    let all_days: f64 = read.iter().map(|one| one.days).sum();
    let instants = read.iter().filter(|one| of(one).1).count();
    let least = shares.iter().copied().fold(f64::INFINITY, f64::min);
    let most = shares.iter().copied().fold(0.0, f64::max);
    format!(
        "| {label} | {:.1}% | {:.1}% | {:.1}% | {} of {} ({}) |\n",
        100.0 * kept_days / all_days,
        100.0 * least,
        100.0 * most,
        count(instants),
        count(read.len()),
        share(instants, read.len()),
    )
}

/// The verse's pranapada against the SDK's point at each recorded instant
/// (X2): how many fall in the same sign, and their median separation.
fn x2(births: &[Birth]) -> Result<String, String> {
    let mut same = 0;
    let mut apart = Vec::new();
    let mut refused = Vec::new();
    for birth in births {
        let foundation = &birth.document.foundation;
        let sun = foundation
            .graha(Graha::Sun)
            .ok_or_else(|| format!("{}: no Sun", birth.name))?
            .longitude_deg;
        let hours = foundation.timing.ishtakaal.to_hours();
        let both = pranapada_deg(PranapadaRule::Verse, sun, hours)
            .and_then(|verse| Ok((verse, pranapada_deg(PranapadaRule::SdkPoint, sun, hours)?)));
        let (verse, point) = match both {
            Ok(both) => both,
            Err(why) => {
                refused.push(format!("`{}` ({})", birth.name, why.message));
                continue;
            }
        };
        if Rashi::of_longitude(verse) == Rashi::of_longitude(point) {
            same += 1;
        }
        let gap = (verse - point).rem_euclid(360.0);
        apart.push(gap.min(360.0 - gap));
    }
    let judged = apart.len();
    let mut out = format!(
        "At {} recorded instants the verse's pranapada and the SDK's \
         point stand in the same sign at {} ({}), {:.1}° apart at the \
         median. The verse moves 300° an hour and the point 60°, so the \
         two agree only where the shift and the hours happen to meet: the \
         rate is the finding X2 raised, and the stage computes the \
         verse's.",
        count(judged),
        count(same),
        share(same, judged),
        median(apart),
    );
    if refused.is_empty() {
        out.push('\n');
    } else {
        let _ = writeln!(out, " Neither is reckoned at {}.", refused.join(", "));
    }
    Ok(out)
}

fn page(root: &Path) -> Result<String, String> {
    let (read, refused, births) = measure(root)?;
    let mut out = String::from(
        "# Rectification's premises, measured\n\n\
         Status: `generated` by `cargo xtask rectification` from the \
         corpus's recorded births, 2026-10-08. Do not edit: \
         `check-rectification` regenerates this page and fails on any \
         difference.\n\n",
    );
    let _ = write!(
        out,
        "Each recorded birth's day, sunrise to the next sunrise, is \
         narrowed under `conformance-baseline` by the purifier of BPHS \
         ch. 2 vv. 67–78 (`rectification.md`). A reading keeps a run when \
         one of its clauses holds there. {} days were narrowed",
        count(read.len()),
    );
    if refused.is_empty() {
        out.push_str(".\n\n");
    } else {
        let _ = write!(out, "; refused: {}.\n\n", refused.join(", "));
    }
    out.push_str(
        "## How much of a day each reading keeps (X8)\n\n\
         The share of all the days kept, the least and the most kept of \
         one day, and how many recorded instants fall in a kept run. The \
         corpus's instants are synthetic, so the last column is no test \
         of the bar: a reading that keeps a share of each day keeps about \
         that share of instants by chance. Whether trusted birth times \
         survive the bar waits for a corpus of them.\n\n\
         | reading | kept | least kept day | most kept day | recorded instants kept |\n\
         |---|---|---|---|---|\n",
    );
    for (label, reads) in SUBSETS {
        out.push_str(&row(label, &read, |one| kept(&one.all, one.at, reads)));
    }
    for (index, (label, _)) in variants().iter().enumerate() {
        out.push_str(&row(label, &read, |one| {
            one.variants
                .get(index)
                .map_or((0.0, false), |answer| kept(answer, one.at, |_| true))
        }));
    }
    out.push_str("\n## The verse's pranapada and the SDK's point (X2)\n\n");
    out.push_str(&x2(&births)?);
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
                "cargo xtask rectification",
            ) != 0,
        ),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}
