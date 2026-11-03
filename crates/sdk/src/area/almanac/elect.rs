//! The election search (`03-design/election-search.md`, A2-1): the times a
//! muhurta rule set leaves over a range at a place, ranked, each with the
//! clauses that hold for it and those that fail it. The search is the
//! almanac's muhurta search ([`AlmanacArea::muhurta`]); this is its record.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use teistro_calendar::CalendarDate;
use teistro_core::catalogue::Calendar;
use teistro_core::envelope::Envelope;
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::Place;
use teistro_core::time::UtcOffset;
use teistro_muhurta::activity::Unjudged;
use teistro_muhurta::search::ClosedDay;
use teistro_muhurta::{Answer, Bar, Clause, Judgement, Ranking, Score};

use super::AlmanacArea;
use crate::asked::{DayAsked, offset_of, place_of};
use crate::muhurta_request::MuhurtaRequest;

/// The field an election request's refusals are named under.
const ELECT: &str = "elect";

/// An election search: a range of days at a place, and the muhurta rule
/// set its times are judged by.
///
/// ```
/// use teistro::finders::ElectRequest;
///
/// let asked = ElectRequest::from_json(
///     r#"{"first": {"year": 2026, "month": 11, "day": 20}, "last": {"year": 2026, "month": 11, "day": 30},
///         "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700,
///         "muhurta": {"rules": "RAMAN_MARRIAGE", "most": 5}}"#,
/// )?;
/// assert_eq!(asked.muhurta.most(), 5);
/// let wrong = ElectRequest::from_json(
///     r#"{"first": {"year": 2026, "month": 11, "day": 20}, "latitudeDeg": 27.7172,
///         "longitudeDeg": 85.324, "utcOffsetSeconds": 20700, "muhurta": {"rules": "RAMAN_MARIAGE"}}"#,
/// )
/// .unwrap_err();
/// assert_eq!(wrong.field(), Some("elect.muhurta.rules"));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ElectRequest {
    /// The first day searched.
    pub first: CalendarDate,
    /// The last day searched, inclusive; the first again for one day.
    pub last: CalendarDate,
    /// Where the days are reckoned.
    pub place: Place,
    /// The offset their civil days run at.
    pub offset: UtcOffset,
    /// The rule set, and how the windows are ranked and kept.
    pub muhurta: MuhurtaRequest,
}

/// The request as a binding writes it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "ElectRequest"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ElectAsked {
    /// The calendar `first` and `last` are in; Gregorian when left out.
    #[serde(default)]
    calendar: Option<Calendar>,
    /// The first day searched.
    first: DayAsked,
    /// The last day searched, inclusive; the first when left out.
    #[serde(default)]
    last: Option<DayAsked>,
    /// The place's latitude, degrees north positive.
    latitude_deg: f64,
    /// The place's longitude, degrees east positive.
    longitude_deg: f64,
    /// The place's height above sea level in metres, 0 when left out.
    #[serde(default)]
    altitude_m: f64,
    /// The offset from UTC the place's civil days run at, in seconds, east positive.
    utc_offset_seconds: i32,
    /// The rule set: an activity's name or rules spelt out, the native, the ranking, how many days are cut and how many windows kept.
    #[cfg_attr(feature = "schema", schemars(with = "crate::muhurta_request::Asked"))]
    muhurta: Value,
}

impl ElectRequest {
    /// The record a binding sends, as JSON: `first` and optional `last` as
    /// `{year, month, day}` in `calendar` (Gregorian when left out);
    /// `latitudeDeg`, `longitudeDeg`, `altitudeM` and `utcOffsetSeconds`;
    /// and `muhurta`, the rule set as `almanac.days` reads it.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, a place or offset out of range, or a rule set its own reader
    /// refuses; each named under `elect`.
    pub fn from_json(text: &str) -> Result<ElectRequest, Error> {
        let asked: ElectAsked = teistro_core::strict::read(text, ELECT)?;
        asked.request().map_err(|why| why.under(ELECT))
    }
}

impl ElectAsked {
    fn request(self) -> Result<ElectRequest, Error> {
        let calendar = self.calendar.unwrap_or(Calendar::Gregorian);
        Ok(ElectRequest {
            first: self.first.in_calendar(calendar),
            last: self.last.unwrap_or(self.first).in_calendar(calendar),
            place: place_of(self.latitude_deg, self.longitude_deg, self.altitude_m)?,
            offset: offset_of(self.utc_offset_seconds)?,
            muhurta: MuhurtaRequest::from_json(&self.muhurta.to_string())?,
        })
    }
}

/// One window an election search kept, ranked, with its reasons.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ElectedWindow {
    /// Its place in the ranking, 1 the best.
    pub rank: usize,
    /// The window, UTC Julian days.
    pub at: Interval,
    /// Whether no bar of the rules holds over it.
    pub open: bool,
    /// The clauses that hold for the time over it, each with the span it
    /// held over.
    pub holds: Vec<Clause>,
    /// The clauses that hold against it.
    pub fails: Vec<Clause>,
    /// The rules' bars among them that close it, in the rules' order.
    pub barred_by: Vec<Bar>,
    /// Its score under the `BASELINE` ranking; absent under one that does
    /// not score.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<Score>,
}

impl ElectedWindow {
    /// A judgement as the `rank`-th window, its clauses split by whether
    /// each counts for the time.
    fn ranked(rank: usize, judgement: Judgement) -> ElectedWindow {
        let open = judgement.open();
        let (holds, fails) = judgement.clauses.into_iter().partition(Clause::favourable);
        ElectedWindow {
            rank,
            at: judgement.at,
            open,
            holds,
            fails,
            barred_by: judgement.barred_by,
            score: judgement.score,
        }
    }
}

/// What an election search answers: the windows best first, and what the
/// search did not judge.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Elections {
    /// The windows kept, best first under the ranking.
    pub windows: Vec<ElectedWindow>,
    /// The days the season closed whole, each with what closed it.
    pub closed: Vec<ClosedDay>,
    /// How many days were judged whole.
    pub days_judged: usize,
    /// How many of those were cut into windows.
    pub days_cut: usize,
    /// How many windows fell inside a heeded blackout that did not cover
    /// their whole day, and were left out.
    pub windows_blacked_out: usize,
    /// The ranking the windows are in.
    pub ranking: Ranking,
    /// What the rules ask that was not judged, said rather than dropped.
    pub unjudged: Vec<Unjudged>,
}

impl From<Answer> for Elections {
    fn from(answer: Answer) -> Elections {
        Elections {
            windows: answer
                .windows
                .into_iter()
                .zip(1..)
                .map(|(judgement, rank)| ElectedWindow::ranked(rank, judgement))
                .collect(),
            closed: answer.closed,
            days_judged: answer.days_judged,
            days_cut: answer.days_cut,
            windows_blacked_out: answer.windows_blacked_out,
            ranking: answer.ranking,
            unjudged: answer.unjudged,
        }
    }
}

impl AlmanacArea<'_> {
    /// **The times a muhurta rule set leaves** over a range at a place,
    /// ranked best first, each window with the clauses that hold for it,
    /// those that fail it and the bars that close it
    /// (`03-design/election-search.md`). The search is
    /// [`AlmanacArea::muhurta`]'s, and so is the provenance.
    ///
    /// ```no_run
    /// use teistro::Context;
    /// use teistro::finders::ElectRequest;
    ///
    /// let sdk = Context::builder().build()?;
    /// let asked = ElectRequest::from_json(
    ///     r#"{"first": {"year": 2026, "month": 11, "day": 20}, "last": {"year": 2026, "month": 12, "day": 15},
    ///         "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700,
    ///         "muhurta": {"rules": "RAMAN_MARRIAGE", "most": 3}}"#,
    /// )?;
    /// for window in sdk.almanac().elect(&asked)?.value.windows {
    ///     println!("{}: {} for, {} against", window.rank, window.holds.len(), window.fails.len());
    /// }
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::muhurta`]: a context with no ephemeris, a range
    /// the wrong way round or longer than the search reads, or what the
    /// search refuses.
    pub fn elect(self, request: &ElectRequest) -> Result<Envelope<Elections>, Error> {
        let Envelope { value, provenance } = self.muhurta(
            &request.first,
            &request.last,
            &request.place,
            request.offset,
            &request.muhurta,
        )?;
        Ok(Envelope::sealing(Elections::from(value), provenance))
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use teistro_core::catalogue::Kaala;
    use teistro_muhurta::{ActivityRules, ClauseKind};

    use super::*;

    const T: f64 = 2_460_000.5;

    #[test]
    fn a_window_splits_its_clauses_by_what_each_counts_for() {
        let rules = ActivityRules::raman_marriage();
        let held = [
            Clause {
                kind: ClauseKind::MarsInEighth {},
                at: Interval::literal(T, T + 0.2),
            },
            Clause {
                kind: ClauseKind::Abhijit {},
                at: Interval::literal(T + 0.1, T + 0.3),
            },
            Clause {
                kind: ClauseKind::Kaala {
                    kaala: Kaala::RahuKaala,
                },
                at: Interval::literal(T + 0.15, T + 0.4),
            },
        ];
        let judged = Judgement::of(Interval::literal(T + 0.1, T + 0.2), &held, &rules);
        let window = ElectedWindow::ranked(3, judged.clone());
        assert_eq!(window.rank, 3);
        assert_eq!(window.open, judged.open());
        assert_eq!(
            window.holds.len() + window.fails.len(),
            judged.clauses.len()
        );
        assert!(window.holds.iter().all(Clause::favourable));
        assert!(!window.fails.iter().any(Clause::favourable));
        assert_eq!(window.barred_by, judged.barred_by);
    }

    #[test]
    fn an_answer_is_ranked_from_one_in_its_order() {
        let rules = ActivityRules::raman_marriage();
        let window =
            |from: f64| Judgement::of(Interval::literal(T + from, T + from + 0.1), &[], &rules);
        let answer = Answer {
            windows: vec![window(0.0), window(0.5)],
            closed: Vec::new(),
            days_judged: 2,
            days_cut: 1,
            windows_blacked_out: 0,
            ranking: Ranking::Texts,
            unjudged: Vec::new(),
        };
        let elections = Elections::from(answer);
        let ranks: Vec<usize> = elections.windows.iter().map(|w| w.rank).collect();
        assert_eq!(ranks, [1, 2]);
        assert_eq!(elections.windows[1].at.from.get(), T + 0.5);
        assert_eq!((elections.days_judged, elections.days_cut), (2, 1));
    }

    #[test]
    fn a_request_is_refused_under_its_own_name() {
        let refused = ElectRequest::from_json(r#"{"first": {"year": 2026, "month": 1, "day": 1}}"#)
            .unwrap_err();
        assert_eq!(refused.field(), Some("elect"));
        let far = ElectRequest::from_json(
            r#"{"first": {"year": 2026, "month": 1, "day": 1}, "latitudeDeg": 97,
                "longitudeDeg": 85.3, "utcOffsetSeconds": 20700, "muhurta": {"rules": "RAMAN_MARRIAGE"}}"#,
        )
        .unwrap_err();
        assert_eq!(far.field(), Some("elect.latitudeDeg"));
        let one = ElectRequest::from_json(
            r#"{"calendar": "BIKRAM_SAMBAT", "first": {"year": 2083, "month": 8, "day": 1}, "latitudeDeg": 27.7,
                "longitudeDeg": 85.3, "utcOffsetSeconds": 20700, "muhurta": {"rules": "RAMAN_MARRIAGE"}}"#,
        )
        .unwrap();
        assert_eq!(one.first, one.last);
        assert_eq!(one.first.calendar, Calendar::BikramSambat);
    }
}
