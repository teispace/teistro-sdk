//! A lunar rule held to a calendar's month: the rule judges its day as it
//! always does, and only a day falling in the named month is kept
//! (`03-design/festival-rule-language.md`, C1320).
//!
//! Kerala keeps Sreekrishna Jayanthi on *Dharmasindhu*'s day for the dark
//! 8th, the one in the solar month Chingam. In most years that is the
//! amanta Shravana's 8th; when Chingam begins after it (2025, Chingam 1 on
//! 17 August), the notice keeps the next month's, on 14 September. A rule
//! naming its lunar month cannot say this, so a rule held to a calendar's
//! month names none and is judged in every month.

use serde::{Deserialize, Serialize};
use teistro_calendar::{CalendarDate, shipped};
use teistro_core::catalogue::Calendar;
use teistro_core::error::Error;

use super::{Observance, Which};

/// The month of a calendar a lunar rule's day must fall in, and which of
/// two days in one such month is kept.
///
/// ```
/// use teistro_core::catalogue::Calendar;
/// use teistro_panchanga::festival::{InMonth, Which};
///
/// let chingam = InMonth { calendar: Calendar::MalayalamSolar, month: 1, twice: Which::Earlier };
/// assert_eq!(
///     serde_json::to_value(chingam)?,
///     serde_json::json!({"calendar": "MALAYALAM_SOLAR", "month": 1, "twice": "EARLIER"}),
/// );
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InMonth {
    /// The calendar the month is in, a solar one as a rule.
    pub calendar: Calendar,
    /// The month, 1 for the first.
    pub month: u8,
    /// Which of two days the rule keeps in one such month is kept.
    pub twice: Which,
}

impl InMonth {
    /// Refuses a calendar the SDK does not ship, month 0, and a rule that
    /// also names its lunar month, which the calendar's month replaces.
    pub(super) fn check(self, key: &str, lunar_month: bool) -> Result<(), Error> {
        if lunar_month {
            return Err(Error::invalid_arg(format!(
                "{key}: a rule held to a calendar's month is judged in every lunar month; leave `month` out"
            ))
            .with_field("inMonth"));
        }
        if shipped(self.calendar).is_none() {
            return Err(Error::invalid_arg(format!(
                "{key}: the SDK does not ship the `{}` calendar",
                self.calendar.key()
            ))
            .with_field("inMonth.calendar"));
        }
        if self.month == 0 {
            return Err(
                Error::invalid_arg(format!("{key}: months are counted from 1"))
                    .with_field("inMonth.month"),
            );
        }
        Ok(())
    }

    /// The year and month of `day` in this calendar.
    fn of(self, day: &CalendarDate) -> Result<(i32, u8), Error> {
        let fixed = super::dated::system(day.calendar)?.fixed_of(day)?;
        let on = super::dated::system(self.calendar)?.date_of(fixed)?;
        Ok((on.year, on.month))
    }
}

/// Keeps, of `kept` (in the order of their tithis), each observance whose
/// rule names no [`InMonth`], and of one that does, the observance its
/// `twice` picks among those whose day falls in its month. Two days in one
/// month are compared only when both are among `kept`: a range's edge may
/// cut a month's other day off, as it does for a dated rule's.
///
/// # Errors
///
/// A day a calendar cannot convert.
pub(super) fn hold(
    rules: &[super::FestivalRule],
    kept: Vec<Observance>,
) -> Result<Vec<Observance>, Error> {
    if rules.iter().all(|rule| rule.in_month.is_none()) {
        return Ok(kept);
    }
    let mut placed: Vec<(Observance, Option<(i32, u8)>)> = Vec::with_capacity(kept.len());
    for observance in kept {
        let held = rules
            .iter()
            .find(|rule| rule.key == observance.rule)
            .and_then(|rule| rule.in_month);
        match held {
            None => placed.push((observance, None)),
            Some(held) => {
                let (year, month) = held.of(&observance.day)?;
                if month == held.month {
                    placed.push((observance, Some((year, month))));
                }
            }
        }
    }
    // One of two in the same month gives way: the later to the earlier
    // under `Earlier`, the earlier to the later under `Later`.
    let gives_way: Vec<bool> = placed
        .iter()
        .enumerate()
        .map(|(index, (observance, month))| {
            month.is_some_and(|month| {
                let twice = rules
                    .iter()
                    .find(|rule| rule.key == observance.rule)
                    .and_then(|rule| rule.in_month)
                    .map_or(Which::Earlier, |held| held.twice);
                let same = |(other, at): &&(Observance, Option<(i32, u8)>)| {
                    other.rule == observance.rule && *at == Some(month)
                };
                match twice {
                    Which::Earlier => placed.iter().take(index).any(|one| same(&one)),
                    Which::Later => placed.iter().skip(index + 1).any(|one| same(&one)),
                }
            })
        })
        .collect();
    Ok(placed
        .into_iter()
        .zip(gives_way)
        .filter_map(|((observance, _), gives_way)| (!gives_way).then_some(observance))
        .collect())
}
