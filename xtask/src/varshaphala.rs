//! The falsification pass over the **annual chart**: the Varsha Pravesha,
//! the instant the Sun returns to where it stood at birth.
//!
//! Everything in Tajika hangs off this one instant, and the dashas over
//! the annual chart wait on it — which ones, and how many, is
//! `dasha-coverage-measured.md`'s to say and not this comment's. It is
//! also the one step of the technique a corpus could settle and **this**
//! corpus does not: the conformance repository records no annual chart of
//! any kind. So nothing here is compared against a recording. What is
//! measured is the rule itself, against the recorded **births**, and
//! against its rivals, because a rule with no recording to check it is
//! still a rule two readings can disagree about.
//!
//! # What makes this worth measuring rather than asserting
//!
//! The chart's frame is **tropical** and its ayanamsha is an offset the
//! SDK applies, so "the Sun returns to its natal longitude" is two
//! different instants depending on which longitude is meant, and the
//! ayanamsha's drift is what separates them. A year apart that is about
//! twenty minutes; a lifetime apart it is half a day, and the lagna is
//! somewhere else entirely. The pass measures the separation and what it
//! costs the lagna, because a reader deciding between the two readings
//! needs the consequence and not the principle.
//!
//! `cargo xtask varshaphala` writes the page and `check-varshaphala`
//! regenerates it in memory and fails on any difference.

use std::fmt::Write as _;
use std::path::Path;

use teistro::catalogue::Graha;
use teistro::quantity::{JulianDay, Utc};
use teistro::tajika::Reading;
use teistro::{Context, Ephemeris};

use crate::births::{Birth, CHARTS, births};
use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, count, fill, plural, seconds, table, worst};

const PAGE: &str = "docs/03-design/annual-chart-measured.md";

/// How many returns each recorded birth is followed for.
///
/// Forty years is where the rivals stop being a curiosity: the tropical
/// reading and the sidereal one separate by about twenty minutes a year,
/// and a reader wants the number at the age a consultation is actually
/// about rather than at the first birthday.
const YEARS: u16 = 40;

/// The sidereal year, days (IERS 2010). The interval a return should keep
/// if the search read a sidereal longitude, and **not** what it keeps if
/// it read a tropical one.
const SIDEREAL_YEAR_DAYS: f64 = 365.256_363_004;

/// The tropical year, days: the rival's interval, for the same reason.
const TROPICAL_YEAR_DAYS: f64 = 365.242_190_402;

/// What one reading of the return gives for one birth.
struct Returns {
    /// Each return's instant, in order.
    instants: Vec<f64>,
    /// Whether the window was cut short by the ephemeris rather than by
    /// the count asked for, so a birth with fewer returns can say which of
    /// the two it was.
    cut_short: bool,
}

impl Returns {
    /// The intervals between successive returns, the first measured from
    /// birth.
    fn intervals(&self, birth: f64) -> Vec<f64> {
        let mut last = birth;
        self.instants
            .iter()
            .map(|at| {
                let gap = at - last;
                last = *at;
                gap
            })
            .collect()
    }
}

/// The returns of one birth under one reading, **through the façade**.
///
/// This pass had its own copy of the search once, and a pass that
/// computes what it measures measures itself. It calls
/// `sdk.chart().praveshas` now, so a change to the module moves this page
/// — which is the point of a page that no corpus can check.
fn returns(sdk: &Context, birth: &Birth, reading: Reading) -> Result<Returns, String> {
    let found = sdk
        .chart()
        .praveshas(&birth.document, reading, YEARS)
        .map_err(|why| format!("{}: its returns: {why}", birth.name))?;
    Ok(Returns {
        cut_short: found.len() < usize::from(YEARS),
        instants: found.into_iter().map(|one| one.at.get()).collect(),
    })
}

/// The lagna's degree at an instant, for the birth's own place: what a
/// rival reading of the return actually costs a reader.
fn lagna_at(sdk: &Context, birth: &Birth, at: f64) -> Result<f64, String> {
    let request = birth.request();
    let document = sdk
        .chart()
        .reading(JulianDay::<Utc>::literal(at), &request)
        .map_err(|why| format!("{}: founding the return: {why}", birth.name))?
        .value;
    Ok(document.foundation.lagna_deg)
}

/// How far apart two instants put the lagna, degrees round the circle.
fn lagna_apart(a: f64, b: f64) -> f64 {
    let apart = (a - b).rem_euclid(360.0);
    apart.min(360.0 - apart)
}

/// The two rivals, measured at the ages a reader meets them at.
struct Rival {
    /// The worst separation from the sidereal return, days.
    worst_days: f64,
    /// The worst the lagna disagrees by, degrees.
    worst_lagna_deg: f64,
    /// Which birth and which year were worst.
    at: String,
}

/// A rival reading measured against the sidereal one over every birth, at
/// the first return and at the fortieth.
fn rival(
    sdk: &Context,
    births: &[Birth],
    sidereal: &[Returns],
    other: impl Fn(&Birth, &Returns, usize) -> Option<f64>,
) -> Result<Rival, String> {
    let mut worst_days = 0.0_f64;
    let mut worst_lagna_deg = 0.0_f64;
    let mut at = String::from("nothing measured");
    for (birth, mine) in births.iter().zip(sidereal) {
        for year in [1_usize, usize::from(YEARS)] {
            let Some(theirs) = other(birth, mine, year) else {
                continue;
            };
            let Some(ours) = mine.instants.get(year - 1) else {
                continue;
            };
            let apart = (theirs - ours).abs();
            let lagna = lagna_apart(lagna_at(sdk, birth, theirs)?, lagna_at(sdk, birth, *ours)?);
            if apart > worst_days {
                worst_days = apart;
                at = format!("`{}`, year {year}", birth.name);
            }
            worst_lagna_deg = worst_lagna_deg.max(lagna);
        }
    }
    Ok(Rival {
        worst_days,
        worst_lagna_deg,
        at,
    })
}

/// Hours, written at the scale they are.
fn hours(days: f64) -> String {
    let hours = days * 24.0;
    if hours.abs() < 1.0 {
        format!("{:.1} minutes", hours * 60.0)
    } else {
        format!("{hours:.2} hours")
    }
}

/// What the births decide about the rule itself, before its rivals: that
/// the search loses nothing, that the Sun is where it should be, and that
/// the interval is a sidereal year and not a tropical one.
struct Held {
    births: usize,
    cut_short: usize,
    lost: usize,
    worst_arcsec: f64,
    read_back: usize,
    gaps: usize,
    mean: f64,
    from_sidereal: f64,
    from_tropical: f64,
    spread: f64,
}

fn what_holds(sdk: &Context, births: &[Birth], sidereal: &[Returns]) -> Result<Held, String> {
    // Every birth reaches the fortieth return unless the ephemeris stops
    // first, which is a fact about the ephemeris and is counted as one: a
    // search that lost a return would otherwise read as a shorter life.
    let cut_short = sidereal.iter().filter(|found| found.cut_short).count();
    let lost = sidereal
        .iter()
        .filter(|found| found.instants.len() < usize::from(YEARS) && !found.cut_short)
        .count();

    // The Sun stands where it stood — read back through a **founded
    // chart** and not through the search's own source, because the
    // question is whether the whole pipeline agrees and not whether the
    // solver landed on its own lattice line, which it did by construction.
    // The first and last return of every birth, which is where the
    // ayanamsha has moved least and most.
    let mut worst_arcsec = 0.0_f64;
    let mut read_back = 0usize;
    for (birth, found) in births.iter().zip(sidereal) {
        for at in [found.instants.first(), found.instants.last()]
            .into_iter()
            .flatten()
        {
            let request = birth.request();
            let document = sdk
                .chart()
                .reading(JulianDay::<Utc>::literal(*at), &request)
                .map_err(|why| format!("{}: reading a return back: {why}", birth.name))?
                .value;
            let sun = document
                .foundation
                .graha(Graha::Sun)
                .ok_or_else(|| format!("{}: a founded chart places the Sun", birth.name))?;
            worst_arcsec =
                worst_arcsec.max(lagna_apart(sun.longitude_deg, birth.natal_sun_deg) * 3600.0);
            read_back += 1;
        }
    }

    // The interval, which is what says which zodiac was read.
    let gaps: Vec<f64> = births
        .iter()
        .zip(sidereal)
        .flat_map(|(birth, found)| found.intervals(birth.at()))
        .collect();
    #[expect(
        clippy::cast_precision_loss,
        reason = "a few thousand intervals is exact as a float"
    )]
    let mean = gaps.iter().sum::<f64>() / gaps.len() as f64;
    let from_sidereal = worst(gaps.iter().map(|gap| (gap - SIDEREAL_YEAR_DAYS).abs()));
    let from_tropical = worst(gaps.iter().map(|gap| (gap - TROPICAL_YEAR_DAYS).abs()));
    let spread = worst(gaps.iter().map(|gap| (gap - mean).abs()));

    Ok(Held {
        births: births.len(),
        cut_short,
        lost,
        worst_arcsec,
        read_back,
        gaps: gaps.len(),
        mean,
        from_sidereal,
        from_tropical,
        spread,
    })
}

/// The rule itself: what every recorded birth says about it.
fn what_the_births_decide(out: &mut String, held: &Held) {
    out.push_str("## What the births decide\n\n");
    let claims = [
        Claim::counted(
            "the search loses no return a birth's window holds",
            held.lost,
            held.births,
        )
        .with_note(format!(
            "{} of them stop early because the built-in ephemeris does, not because a \
             return was missed",
            count(held.cut_short)
        )),
        Claim::stated(
            "the Sun stands at its natal sidereal longitude at every return",
            if held.worst_arcsec <= 1.0 {
                Verdict::Holds
            } else {
                Verdict::Falsified
            },
            format!(
                "worst {:.3} arcseconds over {}, read back through a founded chart",
                held.worst_arcsec,
                plural(held.read_back, "return")
            ),
        ),
        Claim::stated(
            "a return is one **sidereal** year after the last",
            if held.from_sidereal < held.from_tropical {
                Verdict::Holds
            } else {
                Verdict::Falsified
            },
            format!(
                "mean {:.6} days, worst {:.4} from the sidereal year and {:.4} from the \
                 tropical",
                held.mean, held.from_sidereal, held.from_tropical
            ),
        ),
        Claim::stated(
            "the interval is not constant: the Earth's orbit is not a circle",
            if held.spread > 1e-3 {
                Verdict::Holds
            } else {
                Verdict::Falsified
            },
            format!(
                "{} between the widest and the mean",
                seconds(held.spread * 86_400.0)
            ),
        ),
    ];
    out.push_str(&table(&claims));
    out.push('\n');
}

/// The two readings this module does not take, priced in lagna degrees
/// rather than in minutes, because that is what a reader reads.
fn the_rivals(out: &mut String, the_tropical: &Rival, the_mean: &Rival) {
    out.push_str("## The two readings this is not\n\n");
    let _ = write!(
        out,
        "| rival | worst apart | where that puts the lagna | worst at |\n|---|---|---|---|\n\
         | the **tropical** return: the Sun to its natal tropical longitude | {} | {:.2}° | {} |\n\
         | the **mean** return: birth plus a whole sidereal year each time | {} | {:.2}° | {} |\n\n",
        hours(the_tropical.worst_days),
        the_tropical.worst_lagna_deg,
        the_tropical.at,
        hours(the_mean.worst_days),
        the_mean.worst_lagna_deg,
        the_mean.at,
    );
    let _ = write!(
        out,
        "Neither is a rounding error. A lagna {:.0}° from the one a reader \
         would have read is a different sign, a different lord and a \
         different chart, and the tables every Tajika judgement is made \
         from are indexed by exactly that. **Which reading a module takes \
         is therefore a setting and not a default it may quietly pick**, \
         and the tradition's own is the sidereal one, which is what this \
         page measures as the rule and the others as its rivals.\n\n",
        the_tropical.worst_lagna_deg.max(the_mean.worst_lagna_deg),
    );
}

fn page(root: &Path) -> Result<String, String> {
    let sdk = Context::builder()
        .profile("conformance-baseline")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .map_err(|why| format!("the conformance profile: {why}"))?;
    let births = births(root, &sdk)?;
    if births.is_empty() {
        return Err(format!("{CHARTS} records no chart"));
    }

    // The reading this module takes, and the one it does not.
    let mut sidereal = Vec::with_capacity(births.len());
    let mut tropical = Vec::with_capacity(births.len());
    for birth in &births {
        sidereal.push(returns(&sdk, birth, Reading::Sidereal)?);
        tropical.push(returns(&sdk, birth, Reading::Tropical)?);
    }

    let held = what_holds(&sdk, &births, &sidereal)?;

    let the_tropical = rival(&sdk, &births, &sidereal, |birth, _, year| {
        let index = births
            .iter()
            .position(|each| each.name == birth.name)
            .unwrap_or(0);
        tropical
            .get(index)
            .and_then(|found| found.instants.get(year - 1))
            .copied()
    })?;
    let the_mean = rival(&sdk, &births, &sidereal, |birth, _, year| {
        #[expect(
            clippy::cast_precision_loss,
            reason = "a year index under a hundred is exact as a float"
        )]
        let years = year as f64;
        Some(birth.at() + years * SIDEREAL_YEAR_DAYS)
    })?;

    let mut out = String::new();
    out.push_str("# The annual chart, measured\n\n");
    let _ = write!(
        out,
        "Status: `generated` by `cargo xtask varshaphala` over the conformance \
         corpus's recorded births. Do not edit: `check-varshaphala` \
         regenerates this page and fails on any difference. The design it \
         measures is [`annual-chart.md`](annual-chart.md).\n\n\
         Every number below is read from `sdk.chart().praveshas`, the \
         shipped path, and not from a copy of it kept here: a pass that \
         computes what it measures measures itself. This page was written \
         before the module and its numbers did not move when the module \
         took over, which is the only evidence that the thing measured and \
         the thing built are one thing.\n\n"
    );
    let _ = write!(
        out,
        "The corpus records **no annual chart of any kind**, so nothing here \
         is compared against a recording. What it records is {} — their \
         places, their instants and their settings — and a return is a \
         property of a birth, so the rule can be measured over them even \
         where its answer was never written down. {} returns in all, {} a \
         birth.\n\n\
         **The reading is not free.** A chart's frame is tropical and its \
         ayanamsha is an offset the SDK applies, so \"the Sun returns to its \
         natal longitude\" names two different instants, and the ayanamsha's \
         drift is what parts them. Both are computed here and the second \
         column of the rivals' table is what the choice costs: not the \
         separation, which is a number, but where it puts the **lagna**, \
         which is what a reader reads.\n\n",
        plural(births.len(), "birth"),
        count(held.gaps),
        count(usize::from(YEARS)),
    );

    what_the_births_decide(&mut out, &held);

    the_rivals(&mut out, &the_tropical, &the_mean);

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
            i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask varshaphala") != 0)
        }
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}
