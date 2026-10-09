//! The baseline engine's rectification, reproduced: a posterior over a
//! grid of candidate times, from a prior on the reported time and the
//! tattva of the birth's sex, and from how well each candidate's
//! Vimshottari periods fit dated life events (`03-design/rectification.md`,
//! step 6, X12, X14, X20 to X23).
//!
//! **Rank 2, and unsourced.** No text read gives either stage: the event
//! fit is the baseline's own scoring, and its tattva cycle a modern
//! exposition of the Shiva Svarodaya. So this is its own call, reached only
//! when asked, and never summed with the classical stages: a purification
//! removes times, and a likelihood only ranks them (C89's precedent).
//!
//! It reproduces the baseline as it computes, its departures included,
//! because a consumer migrating from it needs the same intervals: the
//! first mahadasha's sub-periods compressed into the balance
//! ([`BirthPeriod::Compressed`]), a search that descends only into the
//! best-fitting period, intervals joined by grid index across a refined
//! grid's gaps. Each is a crux on the design page, and the dasha's rules
//! are the request's to change.

use serde::{Deserialize, Serialize};
use teistro_core::angle::Nas;
use teistro_core::catalogue::{Graha, Nakshatra, Rashi};
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::math;
use teistro_core::quantity::{Degrees, JulianDay, Utc};
use teistro_core::settings::{
    AfterCycle, AshtottariGrouping, Balance, BirthPeriod, SeedOverflow, YearLength,
};
use teistro_dasha::{Birth, Dasha, Period, Timeline, VIMSHOTTARI};

use crate::conception::ConceptionSky;
use crate::purifier::{house, nth};

// ── the request ────────────────────────────────────────────────────────

/// How far the reported time is trusted, which weighs its prior.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Accuracy {
    /// A time read off a clock at the birth.
    Exact,
    /// A time remembered to the hour or so.
    #[default]
    Approximate,
    /// A time already rectified once.
    Rectified,
    /// No time worth weighing: the prior on it is flat.
    Unknown,
}

impl Accuracy {
    /// The weight of the reported time's prior.
    #[must_use]
    pub const fn weight(self) -> f64 {
        match self {
            Accuracy::Exact => 3.0,
            Accuracy::Approximate => 1.2,
            Accuracy::Rectified => 1.0,
            Accuracy::Unknown => 0.0,
        }
    }
}

/// The child's sex, which the tattva prior reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Sex {
    /// Male.
    Male,
    /// Female.
    Female,
}

/// What happened, which decides the houses and karakas a period's lord is
/// scored against.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventKind {
    /// A marriage.
    Marriage,
    /// An engagement.
    Engagement,
    /// A child born.
    ChildBirth,
    /// A miscarriage.
    Miscarriage,
    /// A first job.
    FirstJob,
    /// A change of job.
    JobChange,
    /// A job lost.
    JobLoss,
    /// A promotion.
    Promotion,
    /// A business started.
    BusinessStart,
    /// An education begun or finished.
    Education,
    /// A move of home.
    Relocation,
    /// Travel abroad.
    ForeignTravel,
    /// Property bought.
    Property,
    /// A vehicle bought.
    Vehicle,
    /// Surgery.
    Surgery,
    /// An accident.
    Accident,
    /// An illness.
    Illness,
    /// A death in the family.
    DeathInFamily,
    /// A lawsuit.
    Litigation,
    /// A spiritual initiation.
    SpiritualInitiation,
    /// Anything else.
    Other,
}

/// What a kind of event is scored against: the baseline's table.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Signification {
    /// The houses that signify it most.
    pub primary: &'static [u8],
    /// The houses that support it.
    pub supporting: &'static [u8],
    /// The houses that afflict it.
    pub afflicting: &'static [u8],
    /// Its karakas.
    pub karakas: &'static [Graha],
    /// How sharply its date pins a period, 0 to 1.
    pub sharpness: f64,
}

const fn signified(
    primary: &'static [u8],
    supporting: &'static [u8],
    afflicting: &'static [u8],
    karakas: &'static [Graha],
    sharpness: f64,
) -> Signification {
    Signification {
        primary,
        supporting,
        afflicting,
        karakas,
        sharpness,
    }
}

impl EventKind {
    /// The baseline's table for the kind.
    #[must_use]
    pub const fn signification(self) -> Signification {
        use Graha::{Jupiter, Ketu, Mars, Mercury, Moon, Rahu, Saturn, Sun, Venus};
        match self {
            EventKind::Marriage => signified(&[7], &[2, 11], &[], &[Venus, Jupiter], 1.0),
            EventKind::Engagement => signified(&[7, 5], &[2, 11], &[], &[Venus, Jupiter], 0.8),
            EventKind::ChildBirth => signified(&[5], &[2, 11, 9], &[], &[Jupiter], 1.0),
            EventKind::Miscarriage => signified(&[5], &[], &[8, 12, 6], &[Mars, Saturn, Ketu], 0.9),
            EventKind::FirstJob => signified(&[10, 6], &[2, 11], &[], &[Sun, Saturn, Mercury], 0.9),
            EventKind::JobChange => {
                signified(&[10, 6], &[3, 11], &[12], &[Saturn, Mercury, Rahu], 0.85)
            }
            EventKind::JobLoss => signified(&[10], &[], &[8, 12, 6], &[Saturn, Ketu], 0.85),
            EventKind::Promotion => signified(&[10, 11], &[2, 6], &[], &[Sun, Jupiter], 0.75),
            EventKind::BusinessStart => {
                signified(&[7, 10], &[3, 11, 2], &[], &[Mercury, Jupiter], 0.6)
            }
            EventKind::Education => signified(&[4, 5], &[9, 2], &[], &[Mercury, Jupiter], 0.7),
            EventKind::Relocation => signified(&[4], &[3, 11], &[12], &[Moon, Mars], 0.85),
            EventKind::ForeignTravel => signified(&[12, 9], &[3, 7], &[], &[Rahu, Moon], 0.9),
            EventKind::Property => signified(&[4], &[2, 11], &[], &[Mars, Venus, Saturn], 0.9),
            EventKind::Vehicle => signified(&[4], &[2, 11], &[], &[Venus], 0.8),
            EventKind::Surgery => signified(&[1, 6], &[], &[8, 12], &[Mars, Saturn], 1.0),
            EventKind::Accident => signified(&[1], &[], &[8, 6, 12], &[Mars, Saturn, Rahu], 1.0),
            EventKind::Illness => signified(&[1, 6], &[], &[8, 12], &[Saturn, Mars], 0.6),
            EventKind::DeathInFamily => signified(&[8], &[2, 7], &[12], &[Saturn, Ketu], 1.0),
            EventKind::Litigation => signified(&[6], &[7], &[8, 12], &[Mars, Saturn, Rahu], 0.75),
            EventKind::SpiritualInitiation => {
                signified(&[9, 12], &[5, 8], &[], &[Jupiter, Ketu], 0.5)
            }
            EventKind::Other => signified(&[1, 10], &[], &[], &[], 0.3),
        }
    }
}

/// How finely an event's date is known, which sets the window it is
/// fitted over.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DatePrecision {
    /// The day: a window of one day.
    #[default]
    Day,
    /// The month: 31 days.
    Month,
    /// The year: 366 days.
    Year,
}

impl DatePrecision {
    /// The window's length, days, from its start.
    #[must_use]
    pub const fn days(self) -> f64 {
        match self {
            DatePrecision::Day => 1.0,
            DatePrecision::Month => 31.0,
            DatePrecision::Year => 366.0,
        }
    }
}

/// How sure the family is of an event, which weighs it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Confidence {
    /// Certain.
    #[default]
    Certain,
    /// Probable.
    Probable,
    /// Uncertain.
    Uncertain,
}

impl Confidence {
    /// The event's weight.
    #[must_use]
    pub const fn weight(self) -> f64 {
        match self {
            Confidence::Certain => 1.0,
            Confidence::Probable => 0.7,
            Confidence::Uncertain => 0.4,
        }
    }
}

/// A dated life event.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct LifeEvent {
    /// The caller's name for it, which its notes carry back.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// What happened.
    pub kind: EventKind,
    /// When it began, as an instant (UTC).
    pub on: JulianDay<Utc>,
    /// When it ended, where it ran over days.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<JulianDay<Utc>>,
    /// How finely `on` is known, where `until` is not given.
    #[serde(default)]
    pub precision: DatePrecision,
    /// How sure the family is.
    #[serde(default)]
    pub confidence: Confidence,
    /// Kept out of the fit and tested against it afterwards instead.
    #[serde(default)]
    pub held_out: bool,
}

impl LifeEvent {
    /// An event on a day, certain, in the fit.
    #[must_use]
    pub fn on(kind: EventKind, on: JulianDay<Utc>) -> LifeEvent {
        LifeEvent {
            id: None,
            kind,
            on,
            until: None,
            precision: DatePrecision::Day,
            confidence: Confidence::Certain,
            held_out: false,
        }
    }
}

/// The dasha the event fit reads: Vimshottari, under the rules the
/// baseline computes it with unless asked otherwise.
#[must_use]
pub const fn baseline_dasha_rules() -> teistro_dasha::Rules {
    teistro_dasha::Rules {
        balance: Balance::Spatial,
        year_length: YearLength::Julian36525,
        birth_period: BirthPeriod::Compressed,
        after_cycle: AfterCycle::End,
        seed_overflow: SeedOverflow::WrapToStart,
        ashtottari_grouping: AshtottariGrouping::ThreeEach,
    }
}

/// What the baseline rectification is asked.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct BaselineRequest {
    /// The time on record.
    pub reported: JulianDay<Utc>,
    /// The window's half-width, minutes: the search runs this far either
    /// side of the reported time. At least a minute, at most twelve hours.
    pub uncertainty_minutes: f64,
    /// How far the reported time is trusted.
    #[serde(default)]
    pub accuracy: Accuracy,
    /// The dated events.
    #[serde(default)]
    pub events: Vec<LifeEvent>,
    /// The child's sex, for the tattva prior; none skips it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sex: Option<Sex>,
    /// The share of the posterior the intervals hold, 0.5 to 0.99.
    #[serde(default = "default_coverage")]
    pub coverage: f64,
    /// The dasha the event fit reads.
    #[serde(default = "baseline_dasha_rules")]
    pub dasha: teistro_dasha::Rules,
}

const fn default_coverage() -> f64 {
    0.8
}

impl BaselineRequest {
    /// A request around a reported time, `uncertainty_minutes` either side:
    /// remembered approximately, no events, no sex, the intervals holding
    /// 80% of the posterior, the baseline's dasha.
    #[must_use]
    pub fn around(reported: JulianDay<Utc>, uncertainty_minutes: f64) -> BaselineRequest {
        BaselineRequest {
            reported,
            uncertainty_minutes,
            accuracy: Accuracy::default(),
            events: Vec::new(),
            sex: None,
            coverage: default_coverage(),
            dasha: baseline_dasha_rules(),
        }
    }

    /// Whether the request can be answered, each refusal naming its field.
    ///
    /// # Errors
    ///
    /// An uncertainty outside a minute to twelve hours or a coverage
    /// outside 0.5 to 0.99, as the baseline's caller refuses them; an
    /// event that ends before it begins.
    pub fn check(&self) -> Result<(), Error> {
        if !(1.0..=720.0).contains(&self.uncertainty_minutes) {
            return Err(Error::invalid_arg(format!(
                "the uncertainty is {} minutes; give 1 to 720",
                self.uncertainty_minutes
            ))
            .with_field("uncertaintyMinutes"));
        }
        if !(0.5..=0.99).contains(&self.coverage) {
            return Err(Error::invalid_arg(format!(
                "the coverage is {}; give 0.5 to 0.99",
                self.coverage
            ))
            .with_field("coverage"));
        }
        for (index, event) in self.events.iter().enumerate() {
            if event
                .until
                .is_some_and(|until| until.get() < event.on.get())
            {
                return Err(Error::invalid_arg("an event ends before it begins")
                    .with_field(format!("events[{index}].until")));
            }
        }
        Ok(())
    }

    /// The window searched.
    #[must_use]
    pub fn window(&self) -> Interval {
        let half = self.uncertainty_minutes / MINUTES_PER_DAY;
        Interval::literal(self.reported.get() - half, self.reported.get() + half)
    }
}

// ── the constants ──────────────────────────────────────────────────────

const MINUTES_PER_DAY: f64 = 1440.0;
/// Candidates in the first pass across the window.
pub const CANDIDATES_PER_PASS: f64 = 240.0;
/// The coarse step's bounds, minutes.
pub const COARSE_STEP_MINUTES: (f64, f64) = (0.25, 6.0);
/// How much finer each refining pass steps.
pub const REFINE_FACTOR: f64 = 4.0;
/// The passes at most.
pub const MAX_PASSES: usize = 3;
/// The posterior a refining pass keeps.
pub const REFINE_COVERAGE: f64 = 0.995;
/// The candidates an answer lists.
pub const MAX_CANDIDATES: usize = 20;
/// The prior's resolution, minutes.
pub const PRIOR_RESOLUTION_MINUTES: f64 = 6.0;
/// The event fit's resolution, minutes.
pub const DASHA_RESOLUTION_MINUTES: f64 = 2.0;
/// The tattva prior's penalty on a candidate of the wrong sex's tattva.
pub const TATTVA_SEX_PENALTY: f64 = 2.5;
/// The weight of each level of the dasha a fit descends through.
pub const LEVEL_WEIGHT: [f64; 3] = [0.8, 1.4, 0.9];
/// The event fit's weight.
pub const DASHA_WEIGHT: f64 = 14.0;
/// The score a likelihood never falls below, so no candidate is vetoed.
pub const SCORE_FLOOR: f64 = 0.02;
/// How far over its baseline a held-out event must score to be supported.
pub const HOLD_OUT_MARGIN: f64 = 0.05;
/// How many candidates, about, a held-out event's baseline is the mean of:
/// every `n / HOLD_OUT_SPREAD`-th.
pub const HOLD_OUT_SPREAD: usize = 24;

// ── the tattva cycle (the baseline's, X12) ─────────────────────────────

/// The five tattvas, in the cycle's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Tattva {
    /// Earth.
    Prithvi,
    /// Water.
    Jala,
    /// Fire.
    Agni,
    /// Air.
    Vayu,
    /// Ether.
    Akasha,
}

impl Tattva {
    /// The cycle's order.
    pub const SEQUENCE: [Tattva; 5] = [
        Tattva::Prithvi,
        Tattva::Jala,
        Tattva::Agni,
        Tattva::Vayu,
        Tattva::Akasha,
    ];

    /// Its minutes in each 90-minute turn.
    #[must_use]
    pub const fn minutes(self) -> f64 {
        match self {
            Tattva::Prithvi => 6.0,
            Tattva::Jala => 12.0,
            Tattva::Agni => 18.0,
            Tattva::Vayu => 24.0,
            Tattva::Akasha => 30.0,
        }
    }

    /// The sex of a birth in it.
    #[must_use]
    pub const fn sex(self) -> Sex {
        match self {
            Tattva::Jala | Tattva::Vayu => Sex::Female,
            Tattva::Prithvi | Tattva::Agni | Tattva::Akasha => Sex::Male,
        }
    }

    /// The tattva a weekday's cycle starts from, Sunday 0.
    #[must_use]
    pub const fn starting(weekday: u8) -> Tattva {
        match weekday % 7 {
            0 | 2 => Tattva::Agni,
            1 | 5 => Tattva::Jala,
            3 => Tattva::Prithvi,
            4 => Tattva::Akasha,
            _ => Tattva::Vayu,
        }
    }
}

/// One turn of the cycle: 90 minutes.
pub const TATTVA_TURN_MINUTES: f64 = 90.0;

/// A turn's order: from the weekday's starting tattva in the cycle's
/// order, reversed in every odd turn (negative ones included).
#[must_use]
pub fn turn_order(weekday: u8, turn: i64) -> [Tattva; 5] {
    let start = Tattva::SEQUENCE
        .iter()
        .position(|t| *t == Tattva::starting(weekday))
        .unwrap_or_default();
    let mut order = Tattva::SEQUENCE;
    order.rotate_left(start);
    if turn.rem_euclid(2) == 1 {
        order.reverse();
    }
    order
}

/// The tattva running `minutes` after sunrise, on a weekday, with the
/// minutes it runs from and to.
#[must_use]
pub fn tattva_at(minutes: f64, weekday: u8) -> (Tattva, f64, f64) {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a turn count within a day and a half of sunrise"
    )]
    let turn = (minutes / TATTVA_TURN_MINUTES).floor() as i64;
    #[expect(
        clippy::cast_precision_loss,
        reason = "a turn count within a day and a half of sunrise"
    )]
    let mut cursor = turn as f64 * TATTVA_TURN_MINUTES;
    let order = turn_order(weekday, turn);
    for tattva in order {
        let end = cursor + tattva.minutes();
        if minutes < end {
            return (tattva, cursor, end);
        }
        cursor = end;
    }
    // Only a rounding past the turn's last minute reaches here: the turn's
    // last tattva holds it.
    let last = order[4];
    (last, cursor - last.minutes(), cursor)
}

/// The weekday of an instant on a civil clock, Sunday 0: counted from the
/// clock's local midnight.
#[must_use]
pub fn civil_weekday(at: JulianDay<Utc>, utc_offset_minutes: i32) -> u8 {
    let days = (at.get() + 1.5 + f64::from(utc_offset_minutes) / MINUTES_PER_DAY).floor();
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a day count taken mod 7, which is 0 to 6"
    )]
    let weekday = days.rem_euclid(7.0) as u8;
    weekday
}

// ── the likelihood ─────────────────────────────────────────────────────

/// A score in 0 to 1 as a log-likelihood, floored so nothing is vetoed.
#[must_use]
pub fn from_score(score: f64, weight: f64) -> f64 {
    math::ln(SCORE_FLOOR + (1.0 - SCORE_FLOOR) * score.clamp(0.0, 1.0)) * weight
}

/// Whether every value is the first one.
#[expect(
    clippy::float_cmp,
    reason = "flat means equal to the bit, as the baseline's"
)]
fn flat(curve: &[f64]) -> bool {
    curve
        .first()
        .is_none_or(|first| curve.iter().all(|v| v == first))
}

/// The posterior's probabilities, or none where nothing is finite.
#[must_use]
pub fn probabilities(log_posterior: &[f64]) -> Vec<f64> {
    let max = log_posterior
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    if !max.is_finite() {
        return Vec::new();
    }
    let weights: Vec<f64> = log_posterior.iter().map(|v| math::exp(v - max)).collect();
    let sum: f64 = weights.iter().sum();
    if sum <= 0.0 {
        return Vec::new();
    }
    weights.into_iter().map(|w| w / sum).collect()
}

/// Cells in descending probability, ties in ascending index (a stable
/// sort, as the baseline's).
fn most_probable_first<T>(cells: &mut [T], probability: impl Fn(&T) -> f64) {
    cells.sort_by(|a, b| probability(b).total_cmp(&probability(a)));
}

/// The grid points holding a share of the mass, taken most probable first,
/// then put back in index order and grouped into runs of consecutive index:
/// each run's first and last point.
fn runs(grid: &[f64], p: &[f64], share: f64) -> Vec<(f64, f64)> {
    let mut cells: Vec<(usize, f64, f64)> = grid
        .iter()
        .zip(p)
        .enumerate()
        .map(|(index, (at, p))| (index, *at, *p))
        .collect();
    most_probable_first(&mut cells, |cell| cell.2);
    let mut mass = 0.0;
    let mut taken = Vec::new();
    for (index, at, p) in cells {
        if mass >= share {
            break;
        }
        mass += p;
        taken.push((index, at));
    }
    taken.sort_unstable_by_key(|cell| cell.0);
    let mut runs: Vec<(usize, f64, f64)> = Vec::new();
    for (index, at) in taken {
        match runs.last_mut() {
            Some(run) if run.0 + 1 == index => (run.0, run.2) = (index, at),
            _ => runs.push((index, at, at)),
        }
    }
    runs.into_iter()
        .map(|(_, first, last)| (first, last))
        .collect()
}

/// The index of the first largest value, and the value.
fn first_max(values: &[f64]) -> Option<(usize, f64)> {
    values
        .iter()
        .copied()
        .enumerate()
        .reduce(|best, cell| if cell.1 > best.1 { cell } else { best })
}

// ── the grid ───────────────────────────────────────────────────────────

/// The candidates over segments: each half-open, stepped from its start,
/// at least one point; all of them sorted, duplicates kept, as the
/// baseline's refined grid keeps the points its overlapping segments
/// share.
fn grid(segments: &[(f64, f64)], step: f64) -> Vec<f64> {
    let mut points = Vec::new();
    for (start, end) in segments {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a count of minutes in a day, at least one"
        )]
        let count = (((end - start) / step).round().max(1.0)) as usize;
        #[expect(clippy::cast_precision_loss, reason = "a count of grid points")]
        points.extend((0..count).map(|i| start + i as f64 * step));
    }
    points.sort_by(f64::total_cmp);
    points
}

// ── the sky at a candidate ─────────────────────────────────────────────

/// The nine grahas the fit reads, in the order a candidate holds them.
pub const NINE: [Graha; 9] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
    Graha::Rahu,
    Graha::Ketu,
];

/// What the stages read of one candidate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Candidate {
    /// The instant.
    pub at: f64,
    /// The lagna, degrees.
    pub lagna_deg: f64,
    /// The nine grahas' longitudes, in [`NINE`]'s order.
    pub grahas_deg: [f64; 9],
    /// The civil weekday, Sunday 0.
    pub weekday: u8,
}

impl Candidate {
    /// A graha's sign, if it is one of [`NINE`].
    fn sign(&self, graha: Graha) -> Option<Rashi> {
        NINE.iter()
            .zip(self.grahas_deg)
            .find_map(|(g, deg)| (*g == graha).then(|| Rashi::of_longitude(deg)))
    }

    fn lagna(&self) -> Rashi {
        Rashi::of_longitude(self.lagna_deg)
    }

    fn moon_deg(&self) -> f64 {
        self.grahas_deg[1]
    }
}

fn candidate(
    sky: &dyn ConceptionSky,
    at: f64,
    utc_offset_minutes: i32,
) -> Result<Candidate, Error> {
    let instant = JulianDay::try_new(at)?;
    let lagna_deg = sky.ascendant_deg(instant)?;
    let grahas_deg: [f64; 9] = sky
        .grahas_deg(&NINE, instant)?
        .try_into()
        .map_err(|_| Error::internal("nine grahas asked, and not nine answered"))?;
    Ok(Candidate {
        at,
        lagna_deg,
        grahas_deg,
        weekday: civil_weekday(instant, utc_offset_minutes),
    })
}

// ── the stages ─────────────────────────────────────────────────────────

/// Which stage an outcome is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BaselineStage {
    /// The reported time and the tattva of the sex.
    Prior,
    /// The events against the dasha's periods.
    DashaBoundary,
}

/// What a stage says it did.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(
    tag = "kind",
    rename_all = "SCREAMING_SNAKE_CASE",
    rename_all_fields = "camelCase"
)]
pub enum Note {
    /// The tattva prior: how many candidates a tattva of the other sex
    /// penalised.
    TattvaSex {
        /// The sex.
        sex: Sex,
        /// The minutes of every 90 that admit it.
        admitted_minutes: f64,
        /// The candidates penalised, not excluded.
        penalised: usize,
        /// The candidates.
        of: usize,
    },
    /// The reported time's prior.
    ReportedTime {
        /// How far it is trusted.
        accuracy: Accuracy,
        /// Its uncertainty, minutes.
        uncertainty_minutes: f64,
    },
    /// One event's fit.
    EventFit {
        /// Its index in the request.
        event: usize,
        /// The caller's name for it.
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        /// What happened.
        event_kind: EventKind,
        /// The period lords that fit it, mahadasha first, where any did.
        lords: Vec<Graha>,
        /// Its best score over every candidate, weighted.
        contribution: f64,
    },
}

/// One stage's outcome.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct StageOutcome {
    /// The stage.
    pub stage: BaselineStage,
    /// Whether it ran.
    pub applied: bool,
    /// Whether it told no candidate from another.
    pub flat: bool,
    /// The finest it can tell, minutes.
    pub resolution_minutes: f64,
    /// What it says it did.
    pub notes: Vec<Note>,
}

/// An event's window, weighted.
struct Window {
    event: usize,
    kind: EventKind,
    from: f64,
    to: f64,
    weight: f64,
}

fn windows(events: &[LifeEvent], held_out: bool, weighted: bool) -> Vec<Window> {
    events
        .iter()
        .enumerate()
        .filter(|(_, e)| e.held_out == held_out)
        .map(|(event, e)| {
            let span = e.until.map_or_else(
                || e.precision.days(),
                |until| (until.get() - e.on.get()).max(1.0),
            );
            Window {
                event,
                kind: e.kind,
                from: e.on.get(),
                to: e.on.get() + span,
                weight: if weighted {
                    e.kind.signification().sharpness * e.confidence.weight()
                } else {
                    1.0
                },
            }
        })
        .collect()
}

/// How strongly a period's lord signifies an event at a candidate, 0 to 1.
#[must_use]
pub fn signifier_score(lord: Graha, kind: EventKind, candidate: &Candidate) -> f64 {
    let table = kind.signification();
    let lagna = candidate.lagna();
    let placed = candidate.sign(lord).map(|sign| house(lagna, sign));
    let mut score: f64 = 0.0;
    if let Some(placed) = placed {
        if table.primary.contains(&placed) {
            score += 0.5;
        } else if table.afflicting.contains(&placed) {
            score += 0.35;
        } else if table.supporting.contains(&placed) {
            score += 0.15;
        }
    }
    let rules = |houses: &[u8]| {
        houses
            .iter()
            .any(|h| nth(lagna, h - 1).attributes().lord == lord)
    };
    // The nodes own no sign.
    let owner = !matches!(lord, Graha::Rahu | Graha::Ketu);
    if owner && rules(table.primary) {
        score += 0.35;
    }
    if owner && rules(table.afflicting) {
        score += 0.2;
    }
    if table.karakas.contains(&lord) {
        score += 0.25;
    }
    score.min(1.0)
}

/// An event's fit at one candidate: down the dasha from its mahadashas,
/// at each level the overlapping period that signifies it best (the
/// first, on a tie), and into that period only.
fn fit(dasha: &Dasha, window: &Window, candidate: &Candidate, levels: usize) -> (f64, Vec<Graha>) {
    let mut pool: Vec<Period> = dasha.mahadashas().collect();
    let (mut total, mut weights) = (0.0, 0.0);
    let mut lords = Vec::new();
    for weight in LEVEL_WEIGHT.iter().take(levels) {
        let mut best: Option<(Period, f64)> = None;
        for period in pool
            .iter()
            .filter(|p| p.interval.to.get() > window.from && p.interval.from.get() < window.to)
        {
            let score = signifier_score(period.lord, window.kind, candidate);
            if best.as_ref().is_none_or(|(_, b)| score > *b) {
                best = Some((*period, score));
            }
        }
        let Some((period, score)) = best else {
            break;
        };
        total += score * weight;
        weights += weight;
        if score > 0.0 {
            lords.push(period.lord);
        }
        pool = dasha.children(&period).collect();
    }
    (if weights > 0.0 { total / weights } else { 0.0 }, lords)
}

fn dasha_at(candidate: &Candidate, rules: teistro_dasha::Rules) -> Result<Dasha, Error> {
    let birth = Birth {
        instant: JulianDay::try_new(candidate.at)?,
        moon: Nas::from_degrees(Degrees::try_new(candidate.moon_deg().rem_euclid(360.0))?),
        moon_span: None,
    };
    Dasha::new(&VIMSHOTTARI, &birth, rules)
}

/// The prior: the reported time's Gaussian, and the tattva of the sex.
fn prior(
    request: &BaselineRequest,
    sunrise: f64,
    candidates: &[Candidate],
) -> (Vec<f64>, Vec<Note>) {
    let n = candidates.len();
    let mut curve = vec![0.0; n];
    let mut notes = Vec::new();
    if let Some(sex) = request.sex {
        let allowed: Vec<bool> = candidates
            .iter()
            .map(|c| {
                tattva_at((c.at - sunrise) * MINUTES_PER_DAY, c.weekday)
                    .0
                    .sex()
                    == sex
            })
            .collect();
        let admitted = allowed.iter().filter(|a| **a).count();
        if admitted < n {
            for (value, ok) in curve.iter_mut().zip(&allowed) {
                if !ok {
                    *value -= TATTVA_SEX_PENALTY;
                }
            }
            notes.push(Note::TattvaSex {
                sex,
                admitted_minutes: Tattva::SEQUENCE
                    .iter()
                    .filter(|t| t.sex() == sex)
                    .map(|t| t.minutes())
                    .sum(),
                penalised: n - admitted,
                of: n,
            });
        }
    }
    let weight = request.accuracy.weight();
    if weight > 0.0 {
        let sigma = request.uncertainty_minutes / 2.0 / MINUTES_PER_DAY;
        for (value, c) in curve.iter_mut().zip(candidates) {
            let z = (c.at - request.reported.get()) / sigma;
            *value += -0.5 * z * z * weight;
        }
        notes.push(Note::ReportedTime {
            accuracy: request.accuracy,
            uncertainty_minutes: request.uncertainty_minutes,
        });
    }
    (curve, notes)
}

/// The event fit over every candidate.
struct Boundary {
    /// Each candidate's log-likelihood.
    curve: Vec<f64>,
    /// Each window's best fit.
    notes: Vec<Note>,
    /// The dashas read, which the hold-out test reuses.
    dashas: Vec<Dasha>,
}

/// The event fit over every candidate.
fn dasha_boundary(
    request: &BaselineRequest,
    candidates: &[Candidate],
    levels: usize,
) -> Result<Boundary, Error> {
    let windows = windows(&request.events, false, true);
    let dashas = candidates
        .iter()
        .map(|c| dasha_at(c, request.dasha))
        .collect::<Result<Vec<_>, _>>()?;
    if windows.is_empty() {
        return Ok(Boundary {
            curve: vec![0.0; candidates.len()],
            notes: Vec::new(),
            dashas,
        });
    }
    let total: f64 = windows.iter().map(|w| w.weight).sum();
    let mut best: Vec<(f64, Vec<Graha>)> = vec![(f64::NEG_INFINITY, Vec::new()); windows.len()];
    let mut curve = Vec::with_capacity(candidates.len());
    for (candidate, dasha) in candidates.iter().zip(&dashas) {
        let mut earned = 0.0;
        for (window, best) in windows.iter().zip(&mut best) {
            let (score, lords) = fit(dasha, window, candidate, levels);
            earned += score * window.weight;
            if score > best.0 {
                *best = (score, lords);
            }
        }
        let score = if total > 0.0 { earned / total } else { 0.0 };
        curve.push(from_score(score, DASHA_WEIGHT));
    }
    let mut notes: Vec<Note> = windows
        .iter()
        .zip(best)
        .map(|(w, (score, lords))| Note::EventFit {
            event: w.event,
            id: request.events.get(w.event).and_then(|e| e.id.clone()),
            event_kind: w.kind,
            lords,
            contribution: score.max(0.0) * w.weight * DASHA_WEIGHT,
        })
        .collect();
    notes.sort_by(|a, b| contribution(b).total_cmp(&contribution(a)));
    Ok(Boundary {
        curve,
        notes,
        dashas,
    })
}

fn contribution(note: &Note) -> f64 {
    match note {
        Note::EventFit { contribution, .. } => *contribution,
        _ => 0.0,
    }
}

// ── the answer ─────────────────────────────────────────────────────────

/// One candidate, ranked.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Ranked {
    /// The instant.
    pub at: JulianDay<Utc>,
    /// Its share of the posterior.
    pub probability: f64,
    /// Its log-posterior, the stages summed.
    pub log_posterior: f64,
    /// Its lagna's sign.
    pub lagna: Rashi,
    /// Its lagna's nakshatra.
    pub lagna_nakshatra: Nakshatra,
}

/// A held-out event tested against the fit.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct HoldOut {
    /// Its index in the request.
    pub event: usize,
    /// What happened.
    pub kind: EventKind,
    /// Its score at the posterior's mode.
    pub score_at_fit: f64,
    /// Its mean score over candidates spread across the grid.
    pub baseline: f64,
    /// Whether the mode fits it by more than [`HOLD_OUT_MARGIN`] over the
    /// spread.
    pub supported: bool,
}

/// What the baseline rectification answers.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct BaselineAnswer {
    /// The window searched.
    pub window: Interval,
    /// The sunrise the tattva cycle counted from.
    pub sunrise: JulianDay<Utc>,
    /// The intervals holding [`BaselineRequest::coverage`] of the
    /// posterior.
    pub intervals: Vec<Interval>,
    /// Their width in all, minutes, never finer than the resolution.
    pub interval_width_minutes: f64,
    /// The finest the stages that told candidates apart can tell, minutes.
    pub resolution_minutes: f64,
    /// The posterior's mode.
    pub suggested: JulianDay<Utc>,
    /// How concentrated the posterior is, 0 (flat) to 1.
    pub concentration: f64,
    /// The most probable candidates, at most [`MAX_CANDIDATES`].
    pub candidates: Vec<Ranked>,
    /// Each stage's outcome.
    pub stages: Vec<StageOutcome>,
    /// The events fitted.
    pub events_used: usize,
    /// The events held out.
    pub events_held_out: usize,
    /// The held-out events, tested.
    pub hold_out: Vec<HoldOut>,
}

/// One pass over a grid.
struct Pass {
    grid: Vec<f64>,
    candidates: Vec<Candidate>,
    log_posterior: Vec<f64>,
    outcomes: Vec<StageOutcome>,
    dashas: Vec<Dasha>,
    levels: usize,
}

/// The civil day a search is read on: the clock's offset, which gives each
/// candidate its weekday, and the sunrise of the reported time's civil
/// date, which the tattva cycle counts from.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Civil {
    utc_offset_minutes: i32,
    sunrise: f64,
}

impl Civil {
    /// The reported time's civil date, as the baseline's caller takes it:
    /// its sunrise is the one opening the day that holds the date's noon.
    fn of(
        sky: &dyn ConceptionSky,
        reported: JulianDay<Utc>,
        utc_offset_minutes: i32,
    ) -> Result<Civil, Error> {
        let offset = f64::from(utc_offset_minutes) / MINUTES_PER_DAY;
        let midnight = (reported.get() + offset - 0.5).floor() + 0.5 - offset;
        let day = sky.day(JulianDay::try_new(midnight + 0.5)?)?;
        Ok(Civil {
            utc_offset_minutes,
            sunrise: day.sunrise.get(),
        })
    }
}

fn pass(
    sky: &dyn ConceptionSky,
    request: &BaselineRequest,
    civil: Civil,
    grid: Vec<f64>,
    step_minutes: f64,
) -> Result<Pass, Error> {
    let candidates = grid
        .iter()
        .map(|at| candidate(sky, *at, civil.utc_offset_minutes))
        .collect::<Result<Vec<_>, _>>()?;
    let (prior_curve, prior_notes) = prior(request, civil.sunrise, &candidates);
    let mut log_posterior = prior_curve.clone();
    let mut outcomes = vec![StageOutcome {
        stage: BaselineStage::Prior,
        applied: true,
        flat: flat(&prior_curve),
        resolution_minutes: PRIOR_RESOLUTION_MINUTES,
        notes: prior_notes,
    }];
    let applies = request.events.iter().any(|e| !e.held_out);
    let levels = if step_minutes <= DASHA_RESOLUTION_MINUTES {
        3
    } else {
        2
    };
    let mut dashas = Vec::new();
    if applies {
        let Boundary {
            curve,
            notes,
            dashas: read,
        } = dasha_boundary(request, &candidates, levels)?;
        for (sum, value) in log_posterior.iter_mut().zip(&curve) {
            *sum += value;
        }
        dashas = read;
        outcomes.push(StageOutcome {
            stage: BaselineStage::DashaBoundary,
            applied: true,
            flat: flat(&curve),
            resolution_minutes: DASHA_RESOLUTION_MINUTES,
            notes,
        });
    } else {
        outcomes.push(StageOutcome {
            stage: BaselineStage::DashaBoundary,
            applied: false,
            flat: true,
            resolution_minutes: DASHA_RESOLUTION_MINUTES,
            notes: Vec::new(),
        });
    }
    Ok(Pass {
        grid,
        candidates,
        log_posterior,
        outcomes,
        dashas,
        levels,
    })
}

/// The finest resolution of the stages that told candidates apart.
fn discriminating(outcomes: &[StageOutcome]) -> Option<f64> {
    outcomes
        .iter()
        .filter(|o| o.applied && !o.flat)
        .map(|o| o.resolution_minutes)
        .reduce(f64::min)
}

/// The baseline rectification over a sky, on a civil clock
/// `utc_offset_minutes` east of UTC.
///
/// # Errors
///
/// What [`BaselineRequest::check`] refuses, and whatever the sky cannot
/// answer at a candidate or for the reported time's day.
pub fn rectify(
    sky: &dyn ConceptionSky,
    request: &BaselineRequest,
    utc_offset_minutes: i32,
) -> Result<BaselineAnswer, Error> {
    request.check()?;
    let civil = Civil::of(sky, request.reported, utc_offset_minutes)?;
    let window = request.window();
    let mut step = (2.0 * request.uncertainty_minutes / CANDIDATES_PER_PASS)
        .clamp(COARSE_STEP_MINUTES.0, COARSE_STEP_MINUTES.1);
    let mut segments = vec![(window.from.get(), window.to.get())];
    let mut current = pass(
        sky,
        request,
        civil,
        grid(&segments, step / MINUTES_PER_DAY),
        step,
    )?;
    for _ in 1..MAX_PASSES {
        let Some(floor) = discriminating(&current.outcomes) else {
            break;
        };
        if step <= floor / 2.0 {
            break;
        }
        let p = probabilities(&current.log_posterior);
        let old = step / MINUTES_PER_DAY;
        segments = runs(&current.grid, &p, REFINE_COVERAGE)
            .into_iter()
            .map(|(first, last)| (first - old, last + 2.0 * old))
            .collect();
        step = (floor / 2.0).max(step / REFINE_FACTOR);
        current = pass(
            sky,
            request,
            civil,
            grid(&segments, step / MINUTES_PER_DAY),
            step,
        )?;
    }
    Ok(assemble(request, window, civil, &current, step))
}

fn assemble(
    request: &BaselineRequest,
    window: Interval,
    civil: Civil,
    pass: &Pass,
    step: f64,
) -> BaselineAnswer {
    let resolution = discriminating(&pass.outcomes).unwrap_or(2.0 * request.uncertainty_minutes);
    let p = probabilities(&pass.log_posterior);
    let events_used = request.events.iter().filter(|e| !e.held_out).count();
    let events_held_out = request.events.len() - events_used;
    let Some((mode, _)) = first_max(&p) else {
        return BaselineAnswer {
            window,
            sunrise: JulianDay::literal(civil.sunrise),
            intervals: vec![window],
            interval_width_minutes: 2.0 * request.uncertainty_minutes,
            resolution_minutes: resolution,
            suggested: request.reported,
            concentration: 0.0,
            candidates: Vec::new(),
            stages: pass.outcomes.clone(),
            events_used,
            events_held_out,
            hold_out: Vec::new(),
        };
    };
    let step_days = step / MINUTES_PER_DAY;
    let intervals: Vec<Interval> = runs(&pass.grid, &p, request.coverage)
        .into_iter()
        .map(|(first, last)| Interval::literal(first, last + step_days))
        .collect();
    let width: f64 = intervals
        .iter()
        .map(|i| (i.to.get() - i.from.get()) * MINUTES_PER_DAY)
        .sum();
    let entropy: f64 = p
        .iter()
        .filter(|v| **v > 0.0)
        .map(|v| -v * math::ln(*v))
        .sum();
    #[expect(clippy::cast_precision_loss, reason = "a count of grid points")]
    let n = p.len() as f64;
    let concentration = if p.len() > 1 {
        (1.0 - math::exp(entropy) / n).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let mut cells: Vec<(&Candidate, f64, f64)> = pass
        .candidates
        .iter()
        .zip(&p)
        .zip(&pass.log_posterior)
        .map(|((c, p), log)| (c, *p, *log))
        .collect();
    most_probable_first(&mut cells, |cell| cell.1);
    let suggested = cells
        .first()
        .map_or(request.reported, |cell| JulianDay::literal(cell.0.at));
    let candidates = cells
        .into_iter()
        .take(MAX_CANDIDATES)
        .map(|(c, probability, log_posterior)| Ranked {
            at: JulianDay::literal(c.at),
            probability,
            log_posterior,
            lagna: c.lagna(),
            lagna_nakshatra: nakshatra(c.lagna_deg),
        })
        .collect();
    BaselineAnswer {
        window,
        sunrise: JulianDay::literal(civil.sunrise),
        intervals,
        interval_width_minutes: width.max(resolution),
        resolution_minutes: resolution,
        suggested,
        concentration,
        candidates,
        stages: pass.outcomes.clone(),
        events_used,
        events_held_out,
        hold_out: hold_out(request, pass, mode),
    }
}

fn nakshatra(longitude_deg: f64) -> Nakshatra {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a longitude's nakshatra, 0 to 26"
    )]
    let index = ((longitude_deg.rem_euclid(360.0) / (360.0 / 27.0)).floor() as u16).min(26);
    Nakshatra::from_id(index).unwrap_or(Nakshatra::Revati)
}

/// Each held-out event, unweighted, at the mode and at candidates spread
/// over the grid.
fn hold_out(request: &BaselineRequest, pass: &Pass, mode: usize) -> Vec<HoldOut> {
    let held = windows(&request.events, true, false);
    if held.is_empty() || pass.candidates.is_empty() {
        return Vec::new();
    }
    let dashas: Vec<Dasha> = if pass.dashas.len() == pass.candidates.len() {
        pass.dashas.clone()
    } else {
        match pass
            .candidates
            .iter()
            .map(|c| dasha_at(c, request.dasha))
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(read) => read,
            Err(_) => return Vec::new(),
        }
    };
    let cells: Vec<(&Dasha, &Candidate)> = dashas.iter().zip(&pass.candidates).collect();
    let Some(&fitted) = cells.get(mode) else {
        return Vec::new();
    };
    let spread: Vec<(&Dasha, &Candidate)> = cells
        .iter()
        .copied()
        .step_by((cells.len() / HOLD_OUT_SPREAD).max(1))
        .collect();
    held.iter()
        .map(|w| {
            let at =
                |(dasha, candidate): (&Dasha, &Candidate)| fit(dasha, w, candidate, pass.levels).0;
            let score_at_fit = at(fitted);
            #[expect(clippy::cast_precision_loss, reason = "a count of candidates")]
            let baseline = spread.iter().copied().map(at).sum::<f64>() / spread.len() as f64;
            HoldOut {
                event: w.event,
                kind: w.kind,
                score_at_fit,
                baseline,
                supported: score_at_fit > baseline + HOLD_OUT_MARGIN,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index what they built"
    )]

    use super::*;

    const WEDNESDAY: u8 = 3;
    const THURSDAY: u8 = 4;

    #[test]
    fn the_turn_is_ninety_minutes_of_five_distinct_tattvas() {
        let total: f64 = Tattva::SEQUENCE.iter().map(|t| t.minutes()).sum();
        assert!((total - TATTVA_TURN_MINUTES).abs() < 1e-12);
        assert!((1440.0 / TATTVA_TURN_MINUTES - 16.0).abs() < 1e-12);
    }

    #[test]
    fn a_weekday_starts_its_turn_from_its_own_tattva() {
        assert_eq!(Tattva::starting(WEDNESDAY), Tattva::Prithvi);
        assert_eq!(Tattva::starting(THURSDAY), Tattva::Akasha);
        assert_eq!(Tattva::starting(6), Tattva::Vayu);
        assert_eq!(tattva_at(0.0, WEDNESDAY).0, Tattva::Prithvi);
        assert_eq!(tattva_at(0.0, THURSDAY).0, Tattva::Akasha);
    }

    #[test]
    fn odd_turns_run_backwards_negative_ones_too() {
        let direct = turn_order(WEDNESDAY, 0);
        let mut reversed = direct;
        reversed.reverse();
        assert_eq!(turn_order(WEDNESDAY, 1), reversed);
        assert_eq!(turn_order(WEDNESDAY, 2), direct);
        assert_eq!(turn_order(WEDNESDAY, -1), reversed);
        assert_eq!(turn_order(WEDNESDAY, -2), direct);
        // Half a minute before sunrise is the last of the turn before.
        assert_eq!(tattva_at(-0.5, WEDNESDAY).0, turn_order(WEDNESDAY, -1)[4]);
    }

    #[test]
    fn every_minute_falls_inside_its_span() {
        for m in [
            -1.0, -45.0, -89.0, -90.0, -91.0, -200.0, -450.0, 0.0, 17.0, 200.0,
        ] {
            let (_, from, to) = tattva_at(m, WEDNESDAY);
            assert!(from <= m && m < to, "{m}: {from} to {to}");
        }
        for weekday in 0..7 {
            let mut cursor = -180.0;
            while cursor < 180.0 {
                let (_, from, to) = tattva_at(cursor, weekday);
                assert!((from - cursor).abs() < 1e-9, "{weekday} {cursor}");
                cursor = to;
            }
        }
    }

    #[test]
    fn a_female_birth_is_admitted_36_minutes_of_90() {
        let admitted = (0..90)
            .filter(|m| tattva_at(f64::from(*m), WEDNESDAY).0.sex() == Sex::Female)
            .count();
        assert_eq!(admitted, 36);
    }

    #[test]
    fn the_weekday_is_the_civil_clocks() {
        // JD 2451545.0 is 2000-01-01 12:00 UTC, a Saturday; at +05:45 it is
        // 17:45 the same day, and at 18:30 UTC it is Sunday in Kathmandu.
        assert_eq!(civil_weekday(JulianDay::literal(2_451_545.0), 345), 6);
        assert_eq!(
            civil_weekday(JulianDay::literal(2_451_545.0 + 6.5 / 24.0), 345),
            0
        );
    }

    #[test]
    fn a_flat_curve_stays_flat_and_a_peak_wins() {
        let peak: Vec<f64> = [0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0]
            .iter()
            .map(|s| from_score(*s, 10.0))
            .collect();
        let p = probabilities(&peak);
        assert_eq!(first_max(&p).map(|m| m.0), Some(4));
        assert!(p[4] > 0.9);
        let even: Vec<f64> = vec![from_score(0.7, 10.0); 9];
        assert!(flat(&even));
        assert!(from_score(0.0, 10.0).is_finite());
        assert!(
            from_score(0.0, 20.0) - from_score(1.0, 20.0)
                < from_score(0.0, 1.0) - from_score(1.0, 1.0)
        );
    }

    #[test]
    fn probabilities_sum_to_one_over_a_steep_curve() {
        let curve: Vec<f64> = (0..900)
            .map(|i| from_score(if i == 450 { 1.0 } else { 0.001 }, 40.0))
            .collect();
        let p = probabilities(&curve);
        assert!(p.iter().all(|v| v.is_finite()));
        assert!((p.iter().sum::<f64>() - 1.0).abs() < 1e-9);
        assert_eq!(probabilities(&[f64::NEG_INFINITY; 3]), [0.0_f64; 0]);
    }

    #[test]
    fn runs_join_by_index_and_split_at_a_gap() {
        // A gap between two segments' points, which a run by index joins.
        let grid = [0.0, 1.0, 2.0, 3.0, 4.0, 40.0, 41.0, 42.0, 43.0];
        let scores = [1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0];
        let curve: Vec<f64> = scores.iter().map(|s| from_score(*s, 12.0)).collect();
        assert_eq!(
            runs(&grid, &probabilities(&curve), 0.9),
            [(0.0, 1.0), (42.0, 43.0)]
        );
        let one = [0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0];
        let curve: Vec<f64> = one.iter().map(|s| from_score(*s, 12.0)).collect();
        assert_eq!(runs(&grid, &probabilities(&curve), 0.8), [(3.0, 40.0)]);
    }

    #[test]
    fn the_grid_is_half_open_and_keeps_duplicates() {
        assert_eq!(grid(&[(0.0, 4.0)], 1.0), [0.0, 1.0, 2.0, 3.0]);
        assert_eq!(grid(&[(0.0, 0.0)], 1.0), [0.0]);
        assert_eq!(
            grid(&[(2.0, 4.0), (0.0, 3.0)], 1.0),
            [0.0, 1.0, 2.0, 2.0, 3.0]
        );
    }

    fn at(lagna_deg: f64, grahas_deg: [f64; 9]) -> Candidate {
        Candidate {
            at: 2_451_545.0,
            lagna_deg,
            grahas_deg,
            weekday: 0,
        }
    }

    #[test]
    fn a_lord_scores_its_house_its_lordship_and_its_karakatva() {
        // Aries rising, Venus in Libra (the 7th) and lord of the 7th, a
        // marriage's karaka: 0.5 + 0.35 + 0.25, capped at one.
        let mut grahas = [0.0; 9];
        grahas[5] = 190.0;
        let c = at(10.0, grahas);
        assert!((signifier_score(Graha::Venus, EventKind::Marriage, &c) - 1.0).abs() < 1e-12);
        // Saturn in Aries, the 1st: an accident's primary house and karaka,
        // and lord of the 10th and 11th, neither of the accident's.
        assert!((signifier_score(Graha::Saturn, EventKind::Accident, &c) - 0.75).abs() < 1e-12);
        // Rahu in Aries owns nothing: the house and its karakatva only.
        assert!((signifier_score(Graha::Rahu, EventKind::Accident, &c) - 0.75).abs() < 1e-12);
        // The Moon in Aries for a marriage: nothing at all.
        assert!(signifier_score(Graha::Moon, EventKind::Marriage, &c).abs() < 1e-12);
    }

    #[test]
    fn a_refused_request_names_its_field() {
        let request = BaselineRequest {
            reported: JulianDay::literal(2_451_545.0),
            uncertainty_minutes: 0.0,
            accuracy: Accuracy::Approximate,
            events: Vec::new(),
            sex: None,
            coverage: 0.8,
            dasha: baseline_dasha_rules(),
        };
        let refused = request.check().expect_err("a zero uncertainty");
        assert_eq!(refused.field(), Some("uncertaintyMinutes"));
        let refused = BaselineRequest {
            uncertainty_minutes: 30.0,
            coverage: 1.0,
            ..request.clone()
        }
        .check()
        .expect_err("a whole coverage");
        assert_eq!(refused.field(), Some("coverage"));
        let refused = BaselineRequest {
            uncertainty_minutes: 30.0,
            events: vec![LifeEvent {
                id: None,
                kind: EventKind::Marriage,
                on: JulianDay::literal(2_460_000.0),
                until: Some(JulianDay::literal(2_459_999.0)),
                precision: DatePrecision::Day,
                confidence: Confidence::Certain,
                held_out: false,
            }],
            ..request
        }
        .check()
        .expect_err("an event backwards");
        assert_eq!(refused.field(), Some("events[0].until"));
    }
}
