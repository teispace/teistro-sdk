//! Rectification of a founded chart, as every binding asks for it: the
//! chart is the birth on record, and each reading is taken around its
//! instant, at its place, on the request's clock
//! (`03-design/rectification.md`, step 8).
//!
//! The four readings are the façade's own calls, made with the chart's
//! instant and place, so a binding's answer is the Rust consumer's to the
//! bit. Each is optional, and one left out is not read.

use serde::{Deserialize, Serialize};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::settings::{
    AfterCycle, AshtottariGrouping, Balance, BirthPeriod, SeedOverflow, YearLength,
};
use teistro_core::time::UtcOffset;
use teistro_rectification::baseline::{
    Accuracy, BaselineAnswer, BaselineRequest, LifeEvent, Sex, baseline_dasha_rules,
};
use teistro_rectification::{
    Answer, Circumstance, CircumstanceRules, Conception, ConceptionRules, Facts, Rules, Window,
};
use teistro_serial::Document;

use crate::ChartArea;

/// The record's name where a binding sends it, which a refusal is named
/// under.
const RECTIFICATION: &str = "rectification";

/// The purifier over the minutes either side of the chart's instant.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Purify {
    /// How far either side of the chart's instant the window runs,
    /// minutes: more than none, and at most half of
    /// [`teistro_rectification::LONGEST_HOURS`].
    pub minutes: f64,
    /// The purifier's readings; the texts' own when left out.
    #[serde(default)]
    pub rules: Rules,
}

/// What the family remembers of the birth, and the readings it is
/// weighed under.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct CircumstanceAsked {
    /// The facts given; one left out is read and not weighed.
    pub facts: Facts,
    /// The readings; the texts' own when left out.
    pub rules: CircumstanceRules,
}

/// The baseline engine's cascade around the chart's instant: everything
/// [`BaselineRequest`] takes but the reported time, which is the chart's.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BaselineAsked {
    /// The window's half-width, minutes, 1 to 720.
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
    /// The dasha the event fit reads, the baseline engine's where left
    /// out.
    #[serde(default)]
    pub dasha: DashaAsked,
}

/// The dasha the baseline's event fit reads: each member the baseline
/// engine's own ([`baseline_dasha_rules`]) where left out, so a request
/// names only what it changes.
///
/// ```
/// use teistro::DashaAsked;
/// use teistro::settings::{BirthPeriod, YearLength};
///
/// let asked: DashaAsked = serde_json::from_str(r#"{"yearLength": "SAVANA_360"}"#)?;
/// let rules = asked.rules();
/// assert_eq!(rules.year_length, YearLength::Savana360);
/// assert_eq!(rules.birth_period, BirthPeriod::Compressed);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct DashaAsked {
    /// How the balance is measured.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance: Option<Balance>,
    /// The length of a year.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year_length: Option<YearLength>,
    /// How the birth period is divided among its sub-periods.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_period: Option<BirthPeriod>,
    /// What is answered past the end of the cycle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_cycle: Option<AfterCycle>,
    /// What a seed outside a conditional cycle does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed_overflow: Option<SeedOverflow>,
    /// How Ashtottari's lords share the nakshatras.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ashtottari_grouping: Option<AshtottariGrouping>,
}

impl DashaAsked {
    /// The rules asked for, the baseline engine's in each member left out.
    #[must_use]
    pub fn rules(&self) -> teistro_dasha::Rules {
        let own = baseline_dasha_rules();
        teistro_dasha::Rules {
            balance: self.balance.unwrap_or(own.balance),
            year_length: self.year_length.unwrap_or(own.year_length),
            birth_period: self.birth_period.unwrap_or(own.birth_period),
            after_cycle: self.after_cycle.unwrap_or(own.after_cycle),
            seed_overflow: self.seed_overflow.unwrap_or(own.seed_overflow),
            ashtottari_grouping: self.ashtottari_grouping.unwrap_or(own.ashtottari_grouping),
        }
    }
}

fn default_coverage() -> f64 {
    BaselineRequest::around(JulianDay::literal(0.0), 1.0).coverage
}

impl BaselineAsked {
    /// The request around a reported time.
    #[must_use]
    pub fn at(&self, reported: JulianDay<Utc>) -> BaselineRequest {
        BaselineRequest {
            reported,
            uncertainty_minutes: self.uncertainty_minutes,
            accuracy: self.accuracy,
            events: self.events.clone(),
            sex: self.sex,
            coverage: self.coverage,
            dasha: self.dasha.rules(),
        }
    }
}

/// A chart read as a birth time to rectify.
///
/// ```
/// use teistro::RectificationRequest;
///
/// let asked = RectificationRequest::from_json(
///     r#"{"purify": {"minutes": 30}, "circumstance": {"facts": {"fatherPresent": false}}}"#,
/// )?;
/// assert_eq!(asked.purify.map(|p| p.minutes), Some(30.0));
/// assert!(asked.baseline.is_none());
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct RectificationRequest {
    /// The purifier of BPHS ch. 2 vv. 67–78 over a window around the
    /// chart's instant ([`ChartArea::rectify`]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purify: Option<Purify>,
    /// The pranapada's house, the nisheka and the conception Moon at the
    /// chart's instant ([`ChartArea::conception`]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conception: Option<ConceptionRules>,
    /// *Brihat Jataka* ch. V's circumstances at the chart's instant
    /// ([`ChartArea::circumstance`]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub circumstance: Option<CircumstanceAsked>,
    /// The baseline engine's cascade around the chart's instant
    /// ([`ChartArea::rectify_baseline`]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline: Option<BaselineAsked>,
}

impl RectificationRequest {
    /// The record a binding sends, as JSON: `purify` (`minutes`,
    /// `rules`), `conception`, `circumstance` (`facts`, `rules`) and
    /// `baseline`, every member optional.
    ///
    /// ```
    /// use teistro::RectificationRequest;
    ///
    /// let typo = RectificationRequest::from_json(r#"{"purify": {"minutes": 30, "rule": {}}}"#).unwrap_err();
    /// assert_eq!(typo.field(), Some("rectification.purify.rule"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, or a key it does not
    /// read, named under `rectification`.
    pub fn from_json(text: &str) -> Result<RectificationRequest, Error> {
        teistro_core::strict::read(text, RECTIFICATION)
    }
}

/// A chart's rectification: each reading the request asked for.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Rectification {
    /// What the purifier leaves standing of the window.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purified: Option<Answer>,
    /// The conception reports at the chart's instant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conception: Option<Conception>,
    /// The circumstances at the chart's instant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub circumstance: Option<Circumstance>,
    /// The baseline engine's cascade around it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline: Option<BaselineAnswer>,
}

impl ChartArea<'_> {
    /// A founded chart read as a birth time to rectify
    /// (`03-design/rectification.md`, step 8): each reading the request
    /// names, around the chart's instant and at its place, on the clock
    /// `offset` east of UTC.
    ///
    /// # Errors
    ///
    /// What each reading refuses, named under `rectification`: a purifier
    /// window of no minutes or longer than a day and a half, a baseline
    /// uncertainty outside 1 to 720 minutes, and whatever founding a chart
    /// at an instant read refuses.
    pub fn rectification(
        self,
        document: &Document,
        offset: UtcOffset,
        asked: &RectificationRequest,
    ) -> Result<Rectification, Error> {
        let at = document.foundation.instant;
        let place = &document.foundation.place;
        let read = || -> Result<Rectification, Error> {
            let purified = match asked.purify {
                Some(purify) => {
                    purify
                        .rules
                        .check()
                        .map_err(|error| rebased(error, "purify.rules"))?;
                    let window = window_around(at, purify.minutes)?;
                    let read = self.rectify(window, place, offset, &purify.rules);
                    Some(read.map_err(|error| rebased(error, "purify"))?.value)
                }
                None => None,
            };
            let conception = match asked.conception {
                Some(rules) => {
                    let read = self.conception(at, place, offset, &rules);
                    Some(read.map_err(|error| rebased(error, "conception"))?.value)
                }
                None => None,
            };
            let circumstance = match asked.circumstance {
                Some(asked) => {
                    let read = self.circumstance(at, place, offset, &asked.facts, &asked.rules);
                    Some(read.map_err(|error| rebased(error, "circumstance"))?.value)
                }
                None => None,
            };
            let baseline = match &asked.baseline {
                Some(asked) => {
                    let read = self.rectify_baseline(place, offset, &asked.at(at));
                    Some(read.map_err(|error| rebased(error, "baseline"))?.value)
                }
                None => None,
            };
            Ok(Rectification {
                purified,
                conception,
                circumstance,
                baseline,
            })
        };
        read().map_err(|error| error.under(RECTIFICATION))
    }
}

/// A kernel refusal, named by the field a binding sent: the kernel names
/// its own fields under `rectification`, which the record holds under the
/// member that asked.
fn rebased(error: Error, member: &str) -> Error {
    let inner = error.field().map(|field| {
        field
            .strip_prefix("rectification.")
            .unwrap_or(field)
            .to_owned()
    });
    let error = match inner {
        Some(inner) if inner != "rectification" => error.with_field(inner),
        _ => error.with_field(""),
    };
    error.under(member)
}

/// The window `minutes` either side of an instant, its refusal named by
/// the field a binding sent.
fn window_around(at: JulianDay<Utc>, minutes: f64) -> Result<Window, Error> {
    let refused = |error: Error| error.with_field("purify.minutes");
    if !(minutes.is_finite() && minutes > 0.0) {
        return Err(refused(Error::invalid_arg(format!(
            "a purifier window runs some minutes either side of the chart, and {minutes} is none"
        ))));
    }
    let half = minutes / 1440.0;
    Window::between(
        JulianDay::try_new(at.get() - half)?,
        JulianDay::try_new(at.get() + half)?,
    )
    .map_err(refused)
}
