//! Rectification: which parts of a birth window survive the verses that
//! test a birth time (`03-design/rectification.md`).
//!
//! A birth record gives a window, not an instant. [`narrow`] cuts the
//! window where a clause can change, judges each run once, and answers
//! the runs a bar left standing and the runs it removed, each with the
//! clauses that held and the verse each reads. It never answers a single
//! minute: the interval is the answer.
//!
//! This kernel reads no ephemeris. [`Sky`] is what it asks of one: the
//! lagna, the Sun and the Moon at an instant, and the day that instant
//! belongs to. The SDK implements it over the chart crossing; a test
//! implements it over straight lines.
//!
//! The stage built so far is **the purifier** of BPHS ch. 2 vv. 71–78 in
//! the Subodhini prints (X1): a human birth's lagna stands in the sign of
//! the pranapada, of Gulika or of the Moon, or in a trine of one (v. 77);
//! a lagna no purifier holds is "the birth of a plant" (v. 75), which bars
//! it by default (X8).
//!
//! ```
//! use teistro_core::catalogue::Vara;
//! use teistro_core::quantity::{JulianDay, Utc};
//! use teistro_rectification::{Day, Rules, Sky, Window, narrow};
//!
//! // A sky whose lagna turns once a day and whose Sun and Moon stand still.
//! struct Line;
//! impl Sky for Line {
//!     fn ascendant_deg(&self, at: JulianDay<Utc>) -> Result<f64, teistro_core::Error> {
//!         Ok(((at.get() - 2_460_000.25) * 360.0).rem_euclid(360.0))
//!     }
//!     fn sun_deg(&self, _: JulianDay<Utc>) -> Result<f64, teistro_core::Error> { Ok(10.0) }
//!     fn moon_deg(&self, _: JulianDay<Utc>) -> Result<f64, teistro_core::Error> { Ok(100.0) }
//!     fn day(&self, _: JulianDay<Utc>) -> Result<Day, teistro_core::Error> {
//!         Ok(Day::literal(2_460_000.25, 2_460_000.75, 2_460_001.25, Vara::Budhavara))
//!     }
//! }
//!
//! let window = Window::between(JulianDay::literal(2_460_000.30), JulianDay::literal(2_460_000.40))?;
//! let found = narrow(window, &Line, &Rules::default())?;
//! // Every interval's ends are clause edges, and every run is judged.
//! assert!(!found.intervals.is_empty());
//! for run in &found.intervals {
//!     assert!(run.verdict.pure);
//! }
//! # Ok::<(), teistro_core::Error>(())
//! ```

#![doc(html_no_source)]

mod edges;
pub mod purifier;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Vara;
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};

pub use purifier::{
    Clause, GulikaAt, Native, PranapadaRule, PranapadaWorking, Purifier, PurifyAs, Reference,
    Verdict, gulika_instant, pranapada_deg, pranapada_working,
};

/// The longest window a rectification takes, in hours: the bound the
/// clock-driven lagnas already set, a day and a half.
pub const LONGEST_HOURS: f64 = teistro_points::lagna::LONGEST_HOURS;

/// What the kernel asks of an ephemeris.
pub trait Sky {
    /// The lagna at an instant, in the chart's own zodiac, degrees.
    ///
    /// # Errors
    ///
    /// Whatever the implementation cannot answer, unchanged.
    fn ascendant_deg(&self, at: JulianDay<Utc>) -> Result<f64, Error>;

    /// The Sun's longitude at an instant, in the chart's own zodiac, degrees.
    ///
    /// # Errors
    ///
    /// Whatever the implementation cannot answer, unchanged.
    fn sun_deg(&self, at: JulianDay<Utc>) -> Result<f64, Error>;

    /// The Moon's longitude at an instant, in the chart's own zodiac, degrees.
    ///
    /// # Errors
    ///
    /// Whatever the implementation cannot answer, unchanged.
    fn moon_deg(&self, at: JulianDay<Utc>) -> Result<f64, Error>;

    /// The day an instant belongs to, by the chart's own rule: an instant
    /// before sunrise is in the night of the day before (X9).
    ///
    /// # Errors
    ///
    /// Whatever the implementation cannot answer, unchanged: a polar day
    /// it will not reckon among them.
    fn day(&self, at: JulianDay<Utc>) -> Result<Day, Error>;

    /// The ishtakaal of an instant in its day, in hours of twenty-four-
    /// minute ghatis: what the pranapada counts its palas from (v. 71).
    /// The chart's own ghati reckoning decides it (X9), so a chart that
    /// gives the daylight thirty ghatis whatever its length answers that.
    /// By default the clock's hours since the day's sunrise.
    ///
    /// # Errors
    ///
    /// Whatever the implementation cannot answer, unchanged.
    fn ishtakaal_hours(&self, at: JulianDay<Utc>, day: &Day) -> Result<f64, Error> {
        Ok((at.get() - day.sunrise.get()) * 24.0)
    }
}

/// The day an instant belongs to: its sunrise, sunset and the next
/// sunrise, and its weekday.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Day {
    /// The sunrise that opens the day.
    pub sunrise: JulianDay<Utc>,
    /// The sunset that ends its daylight.
    pub sunset: JulianDay<Utc>,
    /// The sunrise that ends its night.
    pub next_sunrise: JulianDay<Utc>,
    /// Its weekday, which gives the eighths their lords.
    pub vara: Vara,
}

impl Day {
    /// A day from bare Julian days, for tests and examples.
    #[must_use]
    pub const fn literal(sunrise: f64, sunset: f64, next_sunrise: f64, vara: Vara) -> Day {
        Day {
            sunrise: JulianDay::literal(sunrise),
            sunset: JulianDay::literal(sunset),
            next_sunrise: JulianDay::literal(next_sunrise),
            vara,
        }
    }

    /// Whether an instant falls in this day, sunrise inclusive.
    #[must_use]
    pub fn holds(&self, at: JulianDay<Utc>) -> bool {
        self.sunrise.get() <= at.get() && at.get() < self.next_sunrise.get()
    }

    /// The arc holding an instant, and whether it is the daylight.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for a day whose sunset precedes its sunrise or whose
    /// next sunrise precedes its sunset, which no [`Sky`] should answer.
    pub fn arc(&self, at: JulianDay<Utc>) -> Result<(Interval, bool), Error> {
        if at.get() < self.sunset.get() {
            Ok((Interval::new(self.sunrise, self.sunset)?, true))
        } else {
            Ok((Interval::new(self.sunset, self.next_sunrise)?, false))
        }
    }
}

/// The window a birth record gives.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Window {
    /// Its start, a UTC Julian day.
    pub from: JulianDay<Utc>,
    /// Its end, after its start.
    pub to: JulianDay<Utc>,
}

impl Window {
    /// A window between two instants.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for an end that is not after the start or an instant
    /// that is not finite, and `OUT_OF_RANGE` for a window longer than
    /// [`LONGEST_HOURS`]; each names its field under `rectification`.
    pub fn between(from: JulianDay<Utc>, to: JulianDay<Utc>) -> Result<Window, Error> {
        if !from.get().is_finite() || !to.get().is_finite() {
            return Err(
                Error::invalid_arg("a window's ends must be finite Julian days")
                    .with_field("rectification.window"),
            );
        }
        if to.get() <= from.get() {
            return Err(Error::invalid_arg(format!(
                "a window ends after it starts, and {} is not after {}",
                to.get(),
                from.get()
            ))
            .with_field("rectification.window.to"));
        }
        let hours = (to.get() - from.get()) * 24.0;
        if hours > LONGEST_HOURS {
            return Err(Error::new(
                teistro_core::error::Status::OutOfRange,
                format!("a window is at most {LONGEST_HOURS} hours, and this one is {hours:.2}"),
            )
            .with_field("rectification.window"));
        }
        Ok(Window { from, to })
    }

    /// The window's length in days.
    #[must_use]
    pub fn days(&self) -> f64 {
        self.to.get() - self.from.get()
    }
}

/// What a rectification is asked to apply: the purifier's knobs, one per
/// crux, and the step that seeds the edge search.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "one switch per purifier and one for v. 76, each a crux of its own (X6, X7)"
)]
pub struct Rules {
    /// The species v. 77 and v. 78 name, which decides the houses that
    /// purify: a human's are the sign and its trines.
    pub native: Native,
    /// Whether the pranapada may purify (v. 74).
    pub pranapada: bool,
    /// Whether Gulika may purify (v. 75).
    pub gulika: bool,
    /// Whether the Moon may purify (v. 75).
    pub moon: bool,
    /// Whether Gulika's 7th, its navamsha and that navamsha's 7th also
    /// purify, as v. 76 attaches them to Gulika alone (X6).
    pub gulika_extension: bool,
    /// Whether a lagna no purifier holds is removed or only weighed (X8).
    pub purify_as: PurifyAs,
    /// How the pranapada is reckoned (X2, X3).
    pub pranapada_rule: PranapadaRule,
    /// Which end of Saturn's eighth is Gulika (X5).
    pub gulika_at: GulikaAt,
    /// The seed step, in minutes: the window is sampled this often and
    /// every change between two samples is found exactly (X15). It must be
    /// shorter than the briefest run a clause holds, which the verse's
    /// pranapada sets at six minutes a sign.
    pub seed_minutes: f64,
}

impl Default for Rules {
    fn default() -> Rules {
        Rules {
            native: Native::Human,
            pranapada: true,
            gulika: true,
            moon: true,
            gulika_extension: true,
            purify_as: PurifyAs::Bar,
            pranapada_rule: PranapadaRule::Verse,
            gulika_at: GulikaAt::End,
            seed_minutes: DEFAULT_SEED_MINUTES,
        }
    }
}

/// The default seed step: a minute, a sixth of the verse's pranapada sign.
pub const DEFAULT_SEED_MINUTES: f64 = 1.0;

/// The finest seed step a rectification takes, in minutes: a second.
pub const FINEST_SEED_MINUTES: f64 = 1.0 / 60.0;

/// One run of the window between two edges, with what was judged in it.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Run {
    /// Where it starts: the window's start or a clause edge.
    pub from: JulianDay<Utc>,
    /// Where it ends: a clause edge or the window's end.
    pub to: JulianDay<Utc>,
    /// The verdict every instant of it shares.
    pub verdict: Verdict,
}

/// The grid that seeded the edges, so a run reproduces.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Grid {
    /// The seed step, in days.
    pub step_days: f64,
    /// How many cells the window was cut into.
    pub cells: u32,
}

/// What a rectification answers.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Answer {
    /// The maximal runs no bar removed, in order. Under
    /// [`PurifyAs::Weight`] every run is here, its verdict saying whether
    /// it is pure.
    pub intervals: Vec<Run>,
    /// The runs a bar removed, in order: each one's verdict names every
    /// clause that failed.
    pub removed: Vec<Run>,
    /// Every instant inside the window where a clause changes, found to
    /// within [`EDGE_TOLERANCE_DAYS`].
    pub edges: Vec<JulianDay<Utc>>,
    /// The seed grid.
    pub grid: Grid,
}

/// How close an edge is found: a hundredth of a second.
pub const EDGE_TOLERANCE_DAYS: f64 = 1e-7;

/// Narrows a window by the purifier.
///
/// # Errors
///
/// `INVALID_ARG` for a seed step that is not a finite number of minutes
/// from [`FINEST_SEED_MINUTES`] to an hour, or a set of rules under which
/// nothing may purify, naming the field; and whatever the [`Sky`] refuses.
pub fn narrow(window: Window, sky: &dyn Sky, rules: &Rules) -> Result<Answer, Error> {
    rules.check()?;
    let judge = purifier::Judge::new(sky, rules);
    let step_days = rules.seed_minutes / (24.0 * 60.0);
    let known = known_edges(window, &judge)?;
    let (runs, edges, cells) = edges::runs(window, step_days, &known, |at| judge.verdict(at))?;
    let (intervals, removed) = match rules.purify_as {
        PurifyAs::Bar => runs.into_iter().partition(|run| run.verdict.pure),
        PurifyAs::Weight => (runs, Vec::new()),
    };
    Ok(Answer {
        intervals,
        removed,
        edges,
        grid: Grid { step_days, cells },
    })
}

/// The sunrises and sunsets inside a window: the pranapada starts again at
/// one and Gulika moves to the other arc at the other (X9).
fn known_edges(window: Window, judge: &purifier::Judge<'_>) -> Result<Vec<f64>, Error> {
    let mut known = Vec::new();
    let mut at = window.from;
    loop {
        let day = judge.day(at)?;
        known.extend([day.sunrise.get(), day.sunset.get(), day.next_sunrise.get()]);
        if day.next_sunrise.get() >= window.to.get() {
            return Ok(known);
        }
        at = day.next_sunrise;
    }
}

impl Rules {
    /// Refuses rules no rectification can run under.
    ///
    /// # Errors
    ///
    /// As [`narrow`].
    pub fn check(&self) -> Result<(), Error> {
        if !self.seed_minutes.is_finite()
            || !(FINEST_SEED_MINUTES..=60.0).contains(&self.seed_minutes)
        {
            return Err(Error::invalid_arg(format!(
                "the seed step is {FINEST_SEED_MINUTES:.4} to 60 minutes, not {}",
                self.seed_minutes
            ))
            .with_field("rectification.seedMinutes"));
        }
        if !(self.pranapada || self.gulika || self.moon) {
            return Err(Error::invalid_arg(
                "no purifier may purify: allow at least one of the pranapada, Gulika and the Moon",
            )
            .with_field("rectification.purifiers"));
        }
        Ok(())
    }
}
