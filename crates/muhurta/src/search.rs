//! The search: the windows of a range an activity may be held in, judged
//! and ordered (`03-design/muhurta.md` §4.4).
//!
//! Three passes, with the baseline engine's cost property — the work
//! tracks the size of the answer, not of the range:
//!
//! 1. **The season**, once for the range. A day the rite's heeded
//!    blackouts cover whole is closed, and named with what closed it, so
//!    "why is November empty" has an answer; its almanac is never built.
//! 2. **The days** that remain, each judged by its day clauses (limbs,
//!    vara, month, periods, and against a native its taras) and ordered.
//! 3. **The windows** of the best `days_with_windows` of them: each day cut
//!    wherever any clause changes — the day's own spans, the lagna's
//!    navamsa, the grahas' ingresses, a blackout's edge — and each piece
//!    judged once, at its middle, for all of it.
//!
//! What the search reads it asks of a [`Sources`], so the orchestration is
//! the same over any sky; [`crate::sources::ProviderSources`] answers it
//! over an ephemeris provider.

use serde::{Deserialize, Serialize};
use teistro_calendar::CalendarDate;
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_panchanga::Panchanga;

use crate::activity::{ActivityRules, Unjudged};
use crate::clause::Clause;
use crate::day::{Native, clauses as day_clauses};
use crate::instant::{Limbs, Sky, clauses as instant_clauses};
use crate::judge::{Judgement, Ranking};
use crate::season::{Blackout, BlackoutKind};
use crate::window::{day_cuts, lagna_cuts, tyajya, windows};

/// What the search reads.
pub trait Sources {
    /// The civil day a date names, local midnight to local midnight.
    ///
    /// # Errors
    ///
    /// A date the calendar does not have.
    fn civil_day(&self, date: &CalendarDate) -> Result<Interval, Error>;

    /// The date after one.
    ///
    /// # Errors
    ///
    /// A date the calendar does not have.
    fn next_date(&self, date: &CalendarDate) -> Result<CalendarDate, Error>;

    /// The almanac of a date.
    ///
    /// # Errors
    ///
    /// As `Almanac::day`.
    fn day(&self, date: &CalendarDate) -> Result<Panchanga, Error>;

    /// The blackouts of the kinds asked for over a range.
    ///
    /// # Errors
    ///
    /// The provider's or the visibility reckoner's refusal.
    fn season(&self, range: Interval, kinds: &[BlackoutKind]) -> Result<Vec<Blackout>, Error>;

    /// The lagna's longitude at an instant, degrees.
    ///
    /// # Errors
    ///
    /// As `Founder::ascendant_at`.
    fn lagna_at(&self, at: JulianDay<Utc>) -> Result<f64, Error>;

    /// The lagna and the nine grahas at an instant.
    ///
    /// # Errors
    ///
    /// The provider's refusal.
    fn sky_at(&self, at: JulianDay<Utc>) -> Result<Sky, Error>;

    /// Every instant inside a window at which a graha other than the Sun
    /// and the Moon changes sign, or the Moon changes navamsa — the cuts
    /// an almanac day does not already hold.
    ///
    /// # Errors
    ///
    /// The provider's refusal.
    fn ingresses(&self, within: Interval) -> Result<Vec<JulianDay<Utc>>, Error>;
}

/// The most days one search may span: a year and a day, as the almanac's
/// own range.
pub const MOST_DAYS: usize = teistro_panchanga::almanac::MOST_DAYS;

/// How far either side of a day the lagna is followed, days: three hours,
/// so the signs rising at the day's two ends are whole and their tyajya
/// can be judged.
const LAGNA_MARGIN_DAYS: f64 = 0.125;

/// What to search for.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Request {
    /// The activity's rules.
    pub rules: ActivityRules,
    /// The first date.
    pub from: CalendarDate,
    /// The last date, inclusive.
    pub to: CalendarDate,
    /// The native the day is read against, if any.
    pub native: Option<Native>,
    /// The order of the answer.
    pub ranking: Ranking,
    /// How many of the best days are cut into windows.
    pub days_with_windows: usize,
    /// How many windows the answer holds at most.
    pub most: usize,
}

/// A day the season closed.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ClosedDay {
    /// The date.
    pub date: CalendarDate,
    /// The heeded blackouts over it, each kind once, in the order found.
    pub by: Vec<BlackoutKind>,
}

/// The answer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Answer {
    /// The windows judged, best first under the ranking, at most `most`.
    pub windows: Vec<Judgement>,
    /// The days the season closed.
    pub closed: Vec<ClosedDay>,
    /// How many days were judged whole, and how many of them were cut.
    pub days_judged: usize,
    /// How many of those were cut into windows.
    pub days_cut: usize,
    /// How many windows fell inside a heeded blackout that did not cover
    /// their whole day, and were left out.
    pub windows_blacked_out: usize,
    /// The ranking the windows are in.
    pub ranking: Ranking,
    /// What the rules ask that was not judged.
    pub unjudged: Vec<Unjudged>,
}

/// Runs a search.
///
/// # Errors
///
/// A range that ends before it begins or spans more than [`MOST_DAYS`]
/// days (`INVALID_ARG`), and whatever the sources refuse.
pub fn search<S: Sources + ?Sized>(sources: &S, request: &Request) -> Result<Answer, Error> {
    let rules = &request.rules;
    let dates = dates(sources, request)?;
    let civil: Vec<Interval> = dates
        .iter()
        .map(|date| sources.civil_day(date))
        .collect::<Result<_, _>>()?;
    let (Some(first), Some(last)) = (civil.first(), civil.last()) else {
        return Err(Error::internal("a search with no days"));
    };
    // A day's almanac runs sunrise to sunrise, past its civil day; a day
    // either side holds it.
    let range = Interval::new(first.from.plus_days(-1.0)?, last.to.plus_days(1.0)?)?;
    let season = sources.season(range, &rules.heeds)?;

    let mut closed = Vec::new();
    let mut days = Vec::new();
    for (date, civil) in dates.iter().zip(&civil) {
        if covered(*civil, &season) {
            closed.push(ClosedDay {
                date: date.clone(),
                by: kinds_over(*civil, &season),
            });
            continue;
        }
        let day = sources.day(date)?;
        let mut held = day_clauses(&day, request.native.as_ref(), &rules.day);
        held.extend(rules.month_clauses(&day));
        let judged = Judgement::of(day.window, &held, rules);
        days.push((day, held, judged));
    }
    let days_judged = days.len();
    days.sort_by(|a, b| request.ranking.compare(&a.2, &b.2));

    let mut judged = Vec::new();
    let mut windows_blacked_out = 0;
    let days_cut = days.len().min(request.days_with_windows);
    for (day, held, _) in days.iter().take(days_cut) {
        let cut = cut_day(sources, request, day, held, &season)?;
        windows_blacked_out += cut.blacked_out;
        judged.extend(cut.judged);
    }
    judged.sort_by(|a, b| request.ranking.compare(a, b));
    judged.truncate(request.most);
    Ok(Answer {
        windows: judged,
        closed,
        days_judged,
        days_cut,
        windows_blacked_out,
        ranking: request.ranking,
        unjudged: rules.unjudged.clone(),
    })
}

/// The dates of a request, in order.
fn dates<S: Sources + ?Sized>(sources: &S, request: &Request) -> Result<Vec<CalendarDate>, Error> {
    let mut dates = vec![request.from.clone()];
    while !dates.last().is_some_and(|last| same_day(last, &request.to)) {
        if dates.len() >= MOST_DAYS {
            return Err(Error::invalid_arg(format!(
                "a muhurta search spans at most {MOST_DAYS} days, and {} to {} is more, or ends before it begins",
                request.from, request.to
            ))
            .with_field("to"));
        }
        let next = match dates.last() {
            Some(date) => sources.next_date(date)?,
            None => break,
        };
        dates.push(next);
    }
    Ok(dates)
}

/// Whether two dates name the same day. The whole dates may differ where
/// the day does not: a calendar's own `date_of` attaches the era view and
/// its resolution, which a caller's date need not carry.
fn same_day(a: &CalendarDate, b: &CalendarDate) -> bool {
    (a.calendar, a.year, a.month, a.day) == (b.calendar, b.year, b.month, b.day)
}

/// A day's windows, judged.
struct Cut {
    judged: Vec<Judgement>,
    blacked_out: usize,
}

/// Cuts one day into windows and judges each.
fn cut_day<S: Sources + ?Sized>(
    sources: &S,
    request: &Request,
    day: &Panchanga,
    held: &[Clause],
    season: &[Blackout],
) -> Result<Cut, Error> {
    let rules = &request.rules;
    let span = day.window;
    let followed = Interval::new(
        span.from.plus_days(-LAGNA_MARGIN_DAYS)?,
        span.to.plus_days(LAGNA_MARGIN_DAYS)?,
    )?;
    let lagna = lagna_cuts(followed, |at| sources.lagna_at(at))?;
    let mut cuts = day_cuts(day);
    cuts.extend(lagna.iter().map(|c| c.at));
    cuts.extend(sources.ingresses(span)?);
    cuts.extend(season.iter().flat_map(|b| [b.at.from, b.at.to]));
    let tyajya = tyajya(&lagna, span);

    let mut judged = Vec::new();
    let mut blacked_out = 0;
    for piece in windows(span, cuts) {
        if season.iter().any(|b| b.at.overlaps(piece)) {
            blacked_out += 1;
            continue;
        }
        let mid = JulianDay::literal(f64::midpoint(piece.from.get(), piece.to.get()));
        let sky = sources.sky_at(mid)?;
        let limbs = match (day.tithi_at(mid), day.nakshatra_at(mid)) {
            (Some(tithi), Some(nakshatra)) => Some(Limbs {
                tithi: tithi.member,
                vara: day.vara(),
                nakshatra: nakshatra.member,
            }),
            _ => None,
        };
        let native_lagna = request.native.and_then(|n| n.lagna);
        let mut instant = instant_clauses(&sky, limbs.as_ref(), native_lagna, piece);
        instant.extend(rules.instant_clauses(&sky, piece));
        judged.push(Judgement::of(
            piece,
            held.iter().chain(&tyajya).chain(&instant),
            rules,
        ));
    }
    Ok(Cut {
        judged,
        blacked_out,
    })
}

/// Whether the blackouts cover an interval whole, between them.
fn covered(span: Interval, season: &[Blackout]) -> bool {
    let mut over: Vec<Interval> = season
        .iter()
        .map(|b| b.at)
        .filter(|at| at.overlaps(span))
        .collect();
    over.sort_by(|a, b| a.from.get().total_cmp(&b.from.get()));
    let mut reached = span.from.get();
    for at in over {
        if at.from.get() > reached {
            return false;
        }
        reached = reached.max(at.to.get());
        if reached >= span.to.get() {
            return true;
        }
    }
    false
}

/// The kinds of blackout over an interval, each once.
fn kinds_over(span: Interval, season: &[Blackout]) -> Vec<BlackoutKind> {
    let mut kinds = Vec::new();
    for b in season.iter().filter(|b| b.at.overlaps(span)) {
        if !kinds.contains(&b.kind) {
            kinds.push(b.kind);
        }
    }
    kinds
}

#[cfg(test)]
mod tests {
    use super::{covered, same_day};
    use crate::season::{Blackout, BlackoutKind};
    use teistro_calendar::CalendarDate;
    use teistro_core::catalogue::{Calendar, Era};
    use teistro_core::interval::Interval;

    fn b(from: f64, to: f64) -> Blackout {
        Blackout {
            kind: BlackoutKind::Chaturmas,
            at: Interval::literal(from, to),
        }
    }

    #[test]
    fn a_day_is_covered_only_when_the_blackouts_leave_no_gap_in_it() {
        let day = Interval::literal(10.0, 11.0);
        assert!(covered(day, &[b(0.0, 20.0)]));
        assert!(covered(day, &[b(10.5, 12.0), b(9.0, 10.5)]));
        assert!(!covered(day, &[b(9.0, 10.4), b(10.5, 12.0)]));
        assert!(!covered(day, &[b(9.0, 10.99)]));
        assert!(!covered(day, &[]));
    }

    #[test]
    fn a_date_names_its_day_whatever_its_era_view() {
        // A caller's date is bare; the calendar's own `date_of` attaches
        // the era view, as the Gregorian's attaches CE.
        let bare = CalendarDate::defined(Calendar::Gregorian, 2026, 11, 30);
        let viewed = bare.clone().with_era(Era::CommonEra, 2026);
        assert_ne!(bare, viewed);
        assert!(same_day(&bare, &viewed));
        let next = CalendarDate {
            day: 29,
            ..bare.clone()
        };
        assert!(!same_day(&bare, &next));
    }
}
