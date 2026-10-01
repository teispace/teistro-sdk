//! When a chart is diurnal, measured (crux C209): Valens's horizon, as
//! shipped in `Sect::from_horizon`, against the chart's own sunrise and
//! sunset and against the day birth the corpus records.
//!
//! Valens reckons the hemisphere above the earth from the Ascendant and
//! the Descendant, so the horizon decides; but his text cannot part the
//! geometric horizon from the apparent one. The chart's own day runs from
//! an apparent sunrise, the Sun's limb lifted by refraction, so the two
//! readings disagree for a few minutes at each end of the day. This pass
//! finds those minutes at every recorded birth's own place and day: the
//! instant the horizon turns is bisected through the façade, founding a
//! chart at each step, and set against the chart's sunrise and sunset.

use std::fmt::Write as _;
use std::path::Path;

use teistro::quantity::{JulianDay, Utc};
use teistro::{ChartRequest, Context, DayState, DignityRequest, Ephemeris, PolarKind, Sect};

use crate::births::{Birth, CHARTS, births};
use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, count, fill, spaced, table};
use crate::rules_corpus::read_json;

const PAGE: &str = "docs/03-design/sect-measured.md";

/// How far either side of a sunrise or sunset the horizon's turn is
/// sought, in days: an hour, far wider than refraction and the limb.
const REACH_DAYS: f64 = 1.0 / 24.0;

/// The births whose day cannot be bracketed, each with its reason. The
/// list fails both ways: a birth that fails and is not here, and one here
/// that no longer fails, are both refused.
const UNBRACKETED: [(&str, &str); 1] = [(
    "c047-london-1800-01-02",
    "an hour before its sunrise is founded on 1 January 1800, whose own day reads the Sun on 31 December 1799, before the built-in ephemeris begins",
)];

/// The births whose recorded day birth the chart's own daylight does not
/// match, each with its reason; exact both ways, as [`UNBRACKETED`] is.
const RECORDED_OTHERWISE: [(&str, &str); 3] = [
    (
        "c022-honolulu-1941-12-07",
        "the recording engine placed the birth, at 07:48 local, in the previous Hindu day: its sunrise is a day early and its ishtakaal 63 ghatis (`shadbala-measured.md`)",
    ),
    (
        "c028-troms-1988-06-21",
        "a midnight-sun day: the conformance profile's `NEAREST_EVENT` policy synthesises the chart's sunrise and sunset and noon falls outside them, where the recording engine took the civil day (`01-golden-vectors.md` §3)",
    ),
    (
        "c039-mexico-city-1985-09-19",
        "the recording engine placed the birth, at 07:17 local, in the previous Hindu day: its sunrise is a day early and its ishtakaal 62 ghatis (`shadbala-measured.md`)",
    ),
];

/// What an exception list gets wrong: a failing name it does not list,
/// and a listed name that no longer fails.
fn inexact(failing: &[&str], listed: &[(&str, &str)]) -> Vec<String> {
    let unlisted = failing
        .iter()
        .filter(|name| !listed.iter().any(|(at, _)| at == *name))
        .map(|name| format!("{name} (fails, unlisted)"));
    let stale = listed
        .iter()
        .filter(|(at, _)| !failing.contains(at))
        .map(|(at, _)| format!("{at} (listed, but holds)"));
    unlisted.chain(stale).collect()
}

/// The polar circles' latitude, degrees: beyond it part of the zodiac
/// rises backwards, and the Ascendant no longer bounds the sky above.
const POLAR_CIRCLE_DEG: f64 = 66.56;

/// How finely it is found, in days: one second.
const RESOLUTION_DAYS: f64 = 1.0 / 86_400.0;

fn conformance() -> Result<Context, String> {
    Context::builder()
        .profile("conformance-baseline")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .map_err(|why| format!("the conformance profile: {why}"))
}

/// One birth's day, measured.
struct Measured {
    name: String,
    latitude: f64,
    /// The corpus's recorded day birth.
    recorded_day: bool,
    /// The chart's own daylight.
    daylight: bool,
    /// The sect by the horizon at the birth.
    horizon: Sect,
    /// The sect by Valens's own reckoning, in degrees from the Ascendant:
    /// a day chart when the Sun's longitude has already risen past it.
    by_degrees: Sect,
    /// Minutes from the chart's sunrise to the horizon's rising, and from
    /// its sunset to the horizon's setting, positive later; `None` on a
    /// polar day, whose bounds the polar policy synthesised.
    turns: Option<(f64, f64)>,
    /// Why the turns could not be bracketed, when they could not.
    unbracketed: Option<String>,
    /// Which polar day it is, when it is one.
    polar: Option<PolarKind>,
}

/// The sect the horizon reads at an instant, at a birth's place.
fn horizon_at(sdk: &Context, request: &ChartRequest, jd: f64) -> Result<Sect, String> {
    let chart = sdk
        .chart()
        .reading(JulianDay::<Utc>::literal(jd), request)
        .map_err(|why| format!("founding at {jd}: {why}"))?
        .value;
    let read = sdk
        .chart()
        .dignities(&chart, &DignityRequest::default())
        .map_err(|why| format!("reading at {jd}: {why}"))?;
    Ok(read.sect)
}

/// The instant within [`REACH_DAYS`] of `edge` at which the horizon turns
/// from `before` to its other sect, to [`RESOLUTION_DAYS`].
fn turn_near(
    sdk: &Context,
    request: &ChartRequest,
    edge: f64,
    before: Sect,
) -> Result<f64, String> {
    let (mut low, mut high) = (edge - REACH_DAYS, edge + REACH_DAYS);
    if horizon_at(sdk, request, low)? != before || horizon_at(sdk, request, high)? == before {
        return Err(format!(
            "the horizon does not turn from {before:?} within an hour of {edge}"
        ));
    }
    while high - low > RESOLUTION_DAYS {
        let middle = f64::midpoint(low, high);
        if horizon_at(sdk, request, middle)? == before {
            low = middle;
        } else {
            high = middle;
        }
    }
    Ok(f64::midpoint(low, high))
}

fn measure(root: &Path, sdk: &Context, birth: &Birth) -> Result<Measured, String> {
    let file = read_json(&root.join(CHARTS).join(format!("{}.json", birth.name)))?;
    let recorded_day = file["foundation"]["is_day_birth"]
        .as_bool()
        .ok_or_else(|| format!("{}: no recorded day birth", birth.name))?;
    let foundation = &birth.document.foundation;
    let request = birth.request();
    let day = &foundation.day.day;
    let polar = match day.state {
        DayState::Polar { kind, .. } => Some(kind),
        DayState::Normal => None,
    };
    let bracket = || -> Result<(f64, f64), String> {
        let (sunrise, sunset) = (day.sunrise.get(), day.sunset.get());
        let rising = turn_near(sdk, &request, sunrise, Sect::Night)?;
        let setting = turn_near(sdk, &request, sunset, Sect::Day)?;
        Ok(((rising - sunrise) * 1440.0, (setting - sunset) * 1440.0))
    };
    let (turns, unbracketed) = match (polar, bracket()) {
        (Some(_), _) => (None, None),
        (None, Ok(turns)) => (Some(turns), None),
        (None, Err(why)) => (None, Some(why)),
    };
    let horizon = sdk
        .chart()
        .dignities(&birth.document, &DignityRequest::default())
        .map_err(|why| format!("{}: {why}", birth.name))?
        .sect;
    let sun = birth.natal_sun_deg;
    let from_ascendant = (sun - foundation.lagna_deg).rem_euclid(360.0);
    let by_degrees = if from_ascendant > 180.0 {
        Sect::Day
    } else {
        Sect::Night
    };
    Ok(Measured {
        name: birth.name.clone(),
        by_degrees,
        latitude: foundation.place.latitude.get(),
        recorded_day,
        daylight: foundation.day.part.is_daylight(),
        horizon,
        turns,
        unbracketed,
        polar,
    })
}

/// The births of a set, by name.
fn names<'a>(set: impl IntoIterator<Item = &'a Measured>) -> Vec<&'a str> {
    set.into_iter().map(|at| at.name.as_str()).collect()
}

/// What the readings make of each birth: Valens's degrees against the
/// altitude, the chart's daylight against the record and the horizon, and
/// the polar days.
fn reading_claims(measured: &[Measured]) -> Vec<Claim> {
    let day_of = |sect| sect == Sect::Day;
    let polar_circle = |at: &&Measured| at.latitude.abs() > POLAR_CIRCLE_DEG;
    let parting = |at: &&Measured| at.by_degrees != at.horizon;
    let outside = measured.iter().filter(|at| !polar_circle(at)).count();
    let degrees_outside = names(
        measured
            .iter()
            .filter(|at| !polar_circle(at))
            .filter(parting),
    );
    let inside = measured.iter().filter(polar_circle).count();
    let degrees_inside = names(measured.iter().filter(polar_circle).filter(parting));
    let recorded = names(measured.iter().filter(|at| at.daylight != at.recorded_day));
    let recorded_wrong = inexact(&recorded, &RECORDED_OTHERWISE);
    let partings = names(
        measured
            .iter()
            .filter(|at| at.polar.is_none() && day_of(at.horizon) != at.daylight),
    );
    let polar: Vec<_> = measured
        .iter()
        .filter_map(|at| at.polar.map(|kind| (at, kind == PolarKind::Day)))
        .collect();
    let polar_wrong = polar
        .iter()
        .filter(|(at, sun_up)| day_of(at.horizon) != *sun_up)
        .count();
    let polar_daylight = polar
        .iter()
        .filter(|(at, sun_up)| at.daylight != *sun_up)
        .count();
    vec![
        Claim::counted(
            "outside the polar circles, Valens's degrees from the Ascendant read each birth as the Sun's altitude (`SectRule::Horizon`) does",
            degrees_outside.len(),
            outside,
        )
        .with_note(if degrees_outside.is_empty() {
            String::from("they are one rule there")
        } else {
            degrees_outside.join(", ")
        }),
        Claim::stated(
            "inside them, the degrees part from the altitude where the zodiac rises backwards",
            Verdict::Holds,
            format!(
                "{} of {inside} births inside part{}",
                degrees_inside.len(),
                if degrees_inside.is_empty() {
                    String::new()
                } else {
                    format!(": {}", degrees_inside.join(", "))
                }
            ),
        ),
        Claim::counted(
            "the chart's own daylight is the day birth the corpus records, but for the births listed below with their reasons, and every listed birth still differs",
            recorded_wrong.len(),
            measured.len(),
        )
        .with_note(format!(
            "{} differ{}",
            recorded.len(),
            if recorded_wrong.is_empty() {
                String::from(", and the list is exact")
            } else {
                format!("; {}", recorded_wrong.join("; "))
            }
        )),
        Claim::counted(
            "Valens's horizon (`SectRule::Horizon`) reads each birth on a day with a sunrise and a sunset as the chart's daylight (`SectRule::Daylight`) does",
            partings.len(),
            measured.len() - polar.len(),
        )
        .with_note(if partings.is_empty() {
            String::from("no birth falls in the minutes they part")
        } else {
            partings.join(", ")
        }),
        Claim::counted(
            "on a polar day the horizon reads the birth by the sky the day has: day where the Sun does not set, night where it does not rise",
            polar_wrong,
            polar.len(),
        )
        .with_note(format!(
            "the chart's daylight, inside the bounds its polar policy synthesised, reads {polar_daylight} of them otherwise"
        )),
    ]
}

/// Where the horizon turns against the chart's apparent sunrise and
/// sunset, at every birth's place and day that has them.
fn turn_claims(measured: &[Measured]) -> Vec<Claim> {
    let turns: Vec<(f64, f64)> = measured.iter().filter_map(|at| at.turns).collect();
    let rises_after = turns.iter().filter(|(rising, _)| *rising <= 0.0).count();
    let sets_before = turns.iter().filter(|(_, setting)| *setting >= 0.0).count();
    let span = |values: &mut dyn Iterator<Item = f64>| {
        values.fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), value| {
            (low.min(value), high.max(value))
        })
    };
    let rising = span(&mut turns.iter().map(|(rising, _)| *rising));
    let setting = span(&mut turns.iter().map(|(_, setting)| setting.abs()));
    let failed: Vec<&str> = measured
        .iter()
        .filter(|at| at.unbracketed.is_some())
        .map(|at| at.name.as_str())
        .collect();
    let exceptions = inexact(&failed, &UNBRACKETED);
    vec![
        Claim::counted(
            "the horizon rises after the chart's apparent sunrise, at every birth's place and day the Sun rises",
            rises_after,
            turns.len(),
        ),
        Claim::counted(
            "the horizon sets before the chart's apparent sunset, at every birth's place and day the Sun sets",
            sets_before,
            turns.len(),
        ),
        Claim::counted(
            "every birth whose turns cannot be bracketed is listed below with its reason, and every listed birth still cannot be",
            exceptions.len(),
            measured.len(),
        )
        .with_note(if exceptions.is_empty() {
            String::from("the list is exact")
        } else {
            exceptions.join("; ")
        }),
        Claim::stated(
            "how long the two readings part at each end of the day",
            Verdict::Holds,
            format!(
                "{} to {} min at sunrise, {} to {} min at sunset",
                spaced(rising.0, 1),
                spaced(rising.1, 1),
                spaced(setting.0, 1),
                spaced(setting.1, 1),
            ),
        ),
    ]
}

fn page(root: &Path) -> Result<String, String> {
    let sdk = conformance()?;
    let births = births(root, &sdk)?;
    if births.is_empty() {
        return Err(format!("{CHARTS} records no chart"));
    }
    let measured = births
        .iter()
        .map(|birth| measure(root, &sdk, birth))
        .collect::<Result<Vec<_>, _>>()?;
    let claims: Vec<Claim> = reading_claims(&measured)
        .into_iter()
        .chain(turn_claims(&measured))
        .collect();
    let convention = births
        .first()
        .map(|birth| format!("{:?}", birth.document.foundation.day.day.convention))
        .unwrap_or_default();
    let mut out = String::from(
        "# When a chart is diurnal, measured\n\n\
         Status: `generated` by `cargo xtask sect` from the corpus's \
         recorded births and Valens's *Anthologies* (Riley's translation), \
         2026-10-02. Do not edit: `check-sect` regenerates this page and \
         fails on any difference.\n\n\
         Crux C209 asks when a chart is a day chart. Valens reckons the \
         hemisphere above the earth in degrees from the Ascendant and the \
         Descendant, so the horizon decides (`essential-dignities.md` \
         §Sect). The SDK reads it as the Sun's geometric altitude \
         (`Sect::from_altitude`), which is Valens's reckoning wherever the \
         zodiac rises in order and the only one that still says whether the \
         Sun is up inside the polar circles. The chart's own day is \
         reckoned otherwise: ",
    );
    let _ = write!(
        out,
        "its sunrise and sunset are `{convention}`, the Sun's limb lifted by \
         refraction. So the two readings part for a few minutes at each end \
         of every day the Sun rises and sets, and this page measures those \
         minutes at each of the corpus's {} births, at its own place and on \
         its own day. The instant the horizon turns is bisected to a second \
         through the façade's `dignities`, founding a chart at each \
         step.\n\n",
        count(measured.len())
    );
    out.push_str(&table(&claims));
    out.push_str("\n## Recorded otherwise\n\n");
    for (name, why) in RECORDED_OTHERWISE {
        let _ = writeln!(out, "- `{name}`: {why}.");
    }
    out.push_str("\n## Unbracketed\n\n");
    for (name, why) in UNBRACKETED {
        let _ = writeln!(out, "- `{name}`: {why}.");
    }
    out.push_str(
        "\n## What it means\n\n\
         The readings are one rule but for the horizon they take: the \
         geometric one, where the Sun's centre stands on the Ascendant, or \
         the apparent one, where its limb first shows. Every minute they \
         part, the chart's daylight says day and the horizon says night. \
         Nothing in Valens decides between them, so both ship and the \
         answer names the one it used. The horizon is the default because \
         it is the rule as Valens computes it, in degrees from the \
         Ascendant.\n\n\
         ## Birth by birth\n\n\
         Minutes from the chart's sunrise to the horizon's rising, and from \
         its sunset to the horizon's setting, on the birth's own day.\n\n\
         | birth | latitude | day birth (recorded) | daylight | horizon | rising | setting |\n\
         |---|---|---|---|---|---|---|\n",
    );
    let yes = |day: bool| if day { "day" } else { "night" };
    let minutes = |value: Option<f64>, at: &Measured| match (value, at.polar) {
        (Some(value), _) => format!("{} min", spaced(value, 1)),
        (None, Some(_)) => String::from("polar"),
        (None, None) => String::from("unbracketed"),
    };
    for at in &measured {
        let _ = writeln!(
            out,
            "| {} | {}° | {} | {} | {} | {} | {} |",
            at.name,
            spaced(at.latitude, 2),
            yes(at.recorded_day),
            yes(at.daylight),
            yes(at.horizon == Sect::Day),
            minutes(at.turns.map(|(rising, _)| rising), at),
            minutes(at.turns.map(|(_, setting)| setting), at),
        );
    }
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
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask sect") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}
