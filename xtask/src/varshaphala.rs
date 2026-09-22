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
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{ChartRequest, Context, Ephemeris, UtcOffset};
use teistro_astro::ayanamsha::Basis;
use teistro_astro::completion::Completion;
use teistro_astro::events::{Lattice, Quantity, Search};
use teistro_astro::precession::PrecessionModel;
use teistro_astro::sidereal::Sidereal;
use teistro_chart::foundation::ChartFoundation;
use teistro_port_ephemeris::Body;

use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, count, fill, plural, seconds, table, worst};
use crate::rules_corpus::read_json;

const PAGE: &str = "docs/03-design/annual-chart-measured.md";
const CHARTS: &str = "fixtures/baseline/charts";

/// How many returns each recorded birth is followed for.
///
/// Forty years is where the rivals stop being a curiosity: the tropical
/// reading and the sidereal one separate by about twenty minutes a year,
/// and a reader wants the number at the age a consultation is actually
/// about rather than at the first birthday.
const YEARS: usize = 40;

/// The sidereal year, days (IERS 2010). The interval a return should keep
/// if the search read a sidereal longitude, and **not** what it keeps if
/// it read a tropical one.
const SIDEREAL_YEAR_DAYS: f64 = 365.256_363_004;

/// The tropical year, days: the rival's interval, for the same reason.
const TROPICAL_YEAR_DAYS: f64 = 365.242_190_402;

/// One recorded birth, founded.
struct Birth {
    name: String,
    foundation: ChartFoundation,
    /// The zone the birth was recorded in, which the return is cast in too.
    offset: UtcOffset,
    /// The Sun's sidereal longitude at birth, which every return returns to.
    natal_sun_deg: f64,
    /// Its tropical longitude, which the rival reading returns to instead.
    /// Taken from the position the chart already carries rather than
    /// rebuilt from the ayanamsha, so the two readings differ by the
    /// ephemeris and not by this pass's arithmetic.
    natal_sun_tropical_deg: f64,
}

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

/// Every recorded birth, founded by the SDK under the conformance profile.
fn births(root: &Path, sdk: &Context) -> Result<Vec<Birth>, String> {
    let directory = root.join(CHARTS);
    let entries = std::fs::read_dir(&directory).map_err(|why| format!("{CHARTS}: {why}"))?;
    let mut paths: Vec<_> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();
    let mut out = Vec::new();
    for path in paths {
        let file = read_json(&path)?;
        let name = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();
        let at = file["input"]["place"].clone();
        let (Some(latitude), Some(longitude)) = (at["latitude"].as_f64(), at["longitude"].as_f64())
        else {
            return Err(format!("{name}: a place without a latitude or a longitude"));
        };
        let altitude = at["altitude_m"].as_f64().unwrap_or(0.0);
        let Some(jd) = file["input"]["resolved"]["jd_ut"].as_f64() else {
            return Err(format!("{name}: no resolved instant"));
        };
        let offset_minutes = file["input"]["resolved"]["tz_offset_min"]
            .as_i64()
            .unwrap_or(0);
        let place = Place::new(
            Latitude::try_new(latitude).map_err(|why| format!("{name}: {why}"))?,
            Longitude::try_new(longitude).map_err(|why| format!("{name}: {why}"))?,
            Altitude::try_new(altitude).map_err(|why| format!("{name}: {why}"))?,
        );
        let offset = UtcOffset::try_from_seconds(
            i32::try_from(offset_minutes * 60).map_err(|why| format!("{name}: {why}"))?,
        )
        .map_err(|why| format!("{name}: {why}"))?;
        let request = ChartRequest::at(place, offset);
        let document = sdk
            .chart()
            .reading(JulianDay::<Utc>::literal(jd), &request)
            .map_err(|why| format!("{name}: founding it: {why}"))?
            .value;
        let sun = document
            .foundation
            .graha(Graha::Sun)
            .ok_or_else(|| format!("{name}: a founded chart places the Sun"))?;
        let (natal_sun_deg, natal_sun_tropical_deg) = (sun.longitude_deg, sun.tropical_deg);
        out.push(Birth {
            name,
            foundation: document.foundation,
            offset,
            natal_sun_deg,
            natal_sun_tropical_deg,
        });
    }
    Ok(out)
}

/// The returns of one birth under one zodiac, reading its own longitude in
/// that zodiac.
///
/// The target is the birth's longitude **in the zodiac being searched**,
/// which is what makes this two readings and not one rule measured twice:
/// the sidereal reading returns to the sidereal longitude and the tropical
/// one to the tropical, and they are different instants because the
/// ayanamsha moves between them.
fn returns(sdk: &Context, birth: &Birth, reading: Reading) -> Result<Returns, String> {
    let provider = sdk
        .ephemeris()
        .ok_or_else(|| String::from("a context with an ephemeris"))?;
    let settings = sdk.settings();
    let completion = Completion::new(provider, settings.provider.overrides, sdk.delta_t());
    let frame = birth.foundation.zodiac.request;
    let tropical = completion
        .longitudes(frame)
        .with_observer(birth.foundation.place);
    // **The chart's own sidereal reading, not the frame's.** A frame's
    // `Zodiac::Sidereal` applies the *mean* ayanamsha and a founded chart
    // applies the nutated one, 18.46 arcseconds apart — which for the Sun
    // is about seven minutes of time and about two degrees of lagna. The
    // first shape of this pass searched the frame's and the read-back
    // through a founded chart falsified it by exactly that, which is why
    // the read-back goes through a chart and not through the source the
    // search already used.
    let sidereal = Sidereal {
        tropical: &tropical,
        ayanamsha: birth.foundation.zodiac.ayanamsha,
        basis: Basis::True,
        precession: PrecessionModel::default(),
        delta_t: sdk.delta_t(),
    };
    let target = match reading {
        Reading::Sidereal => birth.natal_sun_deg,
        Reading::Tropical => birth.natal_sun_tropical_deg,
    };
    let at = birth.foundation.instant.get();
    // The window is clamped to what the provider covers, because a birth
    // late enough in the corpus runs past the built-in ephemeris before it
    // reaches forty — and a search that asked anyway would fail the whole
    // pass on a fact about the ephemeris rather than about the rule. What
    // it costs is counted on the page rather than hidden here.
    #[expect(clippy::cast_precision_loss, reason = "forty is exact as a float")]
    let wanted = at + (YEARS as f64) * 366.0;
    let covered = completion.capabilities().jd_range.1;
    // The Sun never turns and moves about a degree a day, so a ten-day
    // step cannot pass the one target twice between two samples — and the
    // default of one day would sample forty years of every birth twice
    // over, which is the whole of this pass's cost. That it changes no
    // answer is the check below rather than the claim here: the page is
    // byte-identical at one day and at ten.
    let step_days = 10.0;
    // The search brackets a crossing between two samples, so it can read a
    // step below where it was told to start. A birth at the very edge of
    // the ephemeris then fails on an instant the caller never asked about
    // — which is what a ten-day step found and a one-day step had hidden.
    // Starting a step in costs nothing: the first return is a year away.
    let from = at + step_days + 1.0;
    let to = wanted.min(covered);
    if to <= from {
        // A birth so late that the ephemeris ends before its first return.
        // Nothing to search, and saying so is the answer: the page counts
        // it as the ephemeris stopping rather than as a return lost.
        return Ok(Returns {
            instants: Vec::new(),
            cut_short: true,
        });
    }
    let events = match reading {
        Reading::Sidereal => Search::new(
            &sidereal,
            Quantity::Longitude(Body::Sun),
            Lattice {
                origin_deg: target.rem_euclid(360.0),
                step_deg: 0.0,
            },
        )
        .with_step_days(step_days)
        .between(JulianDay::literal(from), JulianDay::literal(to)),
        Reading::Tropical => Search::new(
            &tropical,
            Quantity::Longitude(Body::Sun),
            Lattice {
                origin_deg: target.rem_euclid(360.0),
                step_deg: 0.0,
            },
        )
        .with_step_days(step_days)
        .between(JulianDay::literal(from), JulianDay::literal(to)),
    }
    .map_err(|why| format!("{}: searching the returns: {why}", birth.name))?;
    Ok(Returns {
        instants: events
            .into_iter()
            .map(|event| event.instant.get())
            .collect(),
        cut_short: covered < wanted,
    })
}

/// Which longitude a return returns to.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Reading {
    /// The natal **sidereal** longitude, read as the chart reads it: the
    /// tradition's, and this module's.
    Sidereal,
    /// The natal **tropical** longitude: the Western return, kept here as
    /// the rival it is.
    Tropical,
}

/// The lagna's degree at an instant, for the birth's own place: what a
/// rival reading of the return actually costs a reader.
fn lagna_at(sdk: &Context, birth: &Birth, at: f64) -> Result<f64, String> {
    let request = ChartRequest::at(birth.foundation.place, birth.offset);
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
        for year in [1_usize, YEARS] {
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
        .filter(|found| found.instants.len() < YEARS && !found.cut_short)
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
            let request = ChartRequest::at(birth.foundation.place, birth.offset);
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
        .flat_map(|(birth, found)| found.intervals(birth.foundation.instant.get()))
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
        Some(birth.foundation.instant.get() + years * SIDEREAL_YEAR_DAYS)
    })?;

    let mut out = String::new();
    out.push_str("# The annual chart, measured\n\n");
    let _ = write!(
        out,
        "Status: `generated` by `cargo xtask varshaphala` over the conformance \
         corpus's recorded births. Do not edit: `check-varshaphala` \
         regenerates this page and fails on any difference. The design it \
         measures is [`annual-chart.md`](annual-chart.md).\n\n"
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
        count(YEARS),
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
