//! A rule's day barred and kept on an earlier tithi instead
//! (`03-design/festival-rule-language.md` §3.8, C608).
//!
//! *Dharmasindhu* p. 56 bars the Durva fast of Bhadrapada's bright 8th
//! once Agastya has risen, and its note keeps it then on the dark 8th
//! before: "पूर्वकृष्णाष्टम्यामेव कुर्यात्". Agastya rises on the 21st day
//! after the Sun enters Simha (p. 58), so a bar is a day of a calendar's
//! year, read by the calendar that counts its days from the ingress. The
//! committee moves the bright 5th and 7th of the Gaura parva with the 8th,
//! so a rule may also be barred with another's day.

use serde::{Deserialize, Serialize};
use teistro_calendar::shipped;
use teistro_core::catalogue::{Calendar, Tithi};
use teistro_core::error::Error;

use super::{FestivalDay, FestivalRule, Judged, Observance, Rites, judge};
use crate::month::LunarMonth;
use crate::span::Span;

/// The longest a barred day's own tithi and the one kept in its place lie
/// apart, in days: a paksha's, with room for a long one.
const NEAREST_DAYS: f64 = 30.0;

/// When a rule's day is barred, and the tithi kept in its place: the last
/// occurrence of `instead` before the barred one, judged by the rule's own
/// window and guards.
///
/// ```
/// use teistro_core::catalogue::{Calendar, Tithi};
/// use teistro_panchanga::festival::Barred;
///
/// let agastya = Barred::From {
///     calendar: Calendar::BikramSambat,
///     month: 5,
///     day: 22,
///     instead: Tithi::KrishnaAshtami,
/// };
/// assert_eq!(
///     serde_json::to_value(&agastya)?,
///     serde_json::json!({
///         "kind": "FROM", "calendar": "BIKRAM_SAMBAT", "month": 5, "day": 22,
///         "instead": "KRISHNA_ASHTAMI",
///     }),
/// );
/// let with = Barred::With { rule: String::from("DURVA_ASHTAMI"), instead: Tithi::KrishnaSaptami };
/// assert_eq!(
///     serde_json::to_value(&with)?,
///     serde_json::json!({"kind": "WITH", "rule": "DURVA_ASHTAMI", "instead": "KRISHNA_SAPTAMI"}),
/// );
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Barred {
    /// From this day of a calendar's year on: a day falling on it or
    /// later in its year is barred, as the Durva fast is once Agastya has
    /// risen, Bhadra 22 of the Bikram Sambat (C608).
    From {
        /// The calendar the day is counted in.
        calendar: Calendar,
        /// Its month, 1 for the first.
        month: u8,
        /// Its day of the month, 1 for the first.
        day: u8,
        /// The tithi kept in the barred day's place.
        instead: Tithi,
    },
    /// When the named rule's day in the same lunar month is barred `FROM` a
    /// day, as the Gaura parva's bright 5th and 7th are with its 8th
    /// (C686). The named rule is asked for too; one barred only `WITH`
    /// another never moves this one.
    With {
        /// The rule's key.
        rule: String,
        /// The tithi kept in the barred day's place.
        instead: Tithi,
    },
}

impl Barred {
    /// The tithi kept in the barred day's place.
    #[must_use]
    pub const fn instead(&self) -> Tithi {
        match *self {
            Barred::From { instead, .. } | Barred::With { instead, .. } => instead,
        }
    }

    /// Whether the rule waits for another's day, judged in the pass after.
    pub(super) const fn waits(&self) -> bool {
        matches!(self, Barred::With { .. })
    }

    /// Refuses a rule kept on no single tithi, a calendar the SDK does not
    /// ship, a month or day 0, and a rule barred with itself or no rule.
    pub(super) fn check(&self, rule: &FestivalRule) -> Result<(), Error> {
        let key = &rule.key;
        if rule.occurs.tithi().is_none() {
            return Err(Error::invalid_arg(format!(
                "{key}: a bar keeps another tithi in the day's place, and this rule is kept on no single tithi"
            ))
            .with_field("barred"));
        }
        match self {
            Barred::From {
                calendar,
                month,
                day,
                ..
            } => {
                if shipped(*calendar).is_none() {
                    return Err(Error::invalid_arg(format!(
                        "{key}: the SDK does not ship the `{}` calendar",
                        calendar.key()
                    ))
                    .with_field("barred.calendar"));
                }
                for (count, field) in [(month, "barred.month"), (day, "barred.day")] {
                    if *count == 0 {
                        return Err(Error::invalid_arg(format!(
                            "{key}: months and days are counted from 1"
                        ))
                        .with_field(field));
                    }
                }
            }
            Barred::With { rule: other, .. } => {
                if other.trim().is_empty() || other == key {
                    return Err(Error::invalid_arg(format!(
                        "{key}: a rule is barred with another rule's day; name one"
                    ))
                    .with_field("barred.rule"));
                }
            }
        }
        Ok(())
    }

    /// Refuses a rule barred with one no rule asked for keeps. A named rule
    /// a caller replaced with one no day bars leaves this one's day
    /// standing: only a day barred `FROM` one moves it.
    pub(super) fn check_among(&self, key: &str, rules: &[FestivalRule]) -> Result<(), Error> {
        match self {
            Barred::With { rule: other, .. } if !rules.iter().any(|rule| rule.key == *other) => {
                Err(Error::invalid_arg(format!(
                    "{key}: it is barred with {other}, which no rule asked for keeps; ask for that rule too"
                ))
                .with_field("barred.rule"))
            }
            _ => Ok(()),
        }
    }

    /// Whether `kept` is barred: by its day's place in the calendar's
    /// year, or by the named rule's barred day in the same month.
    fn holds(&self, kept: &Observance, rites: &Rites<'_>) -> Result<bool, Error> {
        match self {
            Barred::From {
                calendar,
                month,
                day,
                ..
            } => {
                let fixed = super::dated::system(kept.day.calendar)?.fixed_of(&kept.day)?;
                let on = super::dated::system(*calendar)?.date_of(fixed)?;
                Ok((on.month, on.day) >= (*month, *day))
            }
            Barred::With { rule, .. } => Ok(rites.kept.iter().any(|other| {
                other.rule == *rule
                    && other.barred
                    && other.month == kept.month
                    && other.adhika == kept.adhika
                    && (other.tithi.from.get() - kept.tithi.from.get()).abs() < NEAREST_DAYS
            })),
        }
    }
}

/// What `rule` keeps of `judged`, its occurrence `whole` in `month`: the
/// judged day, or when its bar holds, the day judged for the last
/// occurrence of the bar's tithi before `whole`, marked
/// [`Observance::barred`] and named in the barred day's month.
///
/// # Errors
///
/// A day a calendar cannot convert, and what judging refuses.
pub(super) fn keep(
    rule: &FestivalRule,
    judged: Judged,
    whole: teistro_core::interval::Interval,
    month: LunarMonth,
    tithis: &[Span<Tithi>],
    days: &[FestivalDay],
    rites: &Rites<'_>,
) -> Result<Judged, Error> {
    let (Some(bar), Ok(Some(kept))) = (&rule.barred, &judged) else {
        return Ok(judged);
    };
    if !bar.holds(kept, rites)? {
        return Ok(judged);
    }
    let instead = bar.instead();
    let Some(occurrence) = tithis.iter().rev().find(|held| {
        held.member == instead
            && held.whole.to.get() <= whole.from.get()
            && whole.from.get() - held.whole.from.get() < NEAREST_DAYS
    }) else {
        return Ok(Err(format!(
            "its day is barred, and the days do not reach the {} before it",
            instead.key()
        )));
    };
    Ok(
        judge(rule, occurrence.whole, month, days, rites)?.map(|judged| {
            judged.map(|observance| Observance {
                barred: true,
                ..observance
            })
        }),
    )
}
