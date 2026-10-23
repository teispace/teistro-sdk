//! The bounds a request is held to (`03-design/mcp-server.md` §7, P5): a
//! message's text, an array's length, a range of days and a window of
//! years. A bound passed
//! is a refusal naming the field, the bound and the option that moves
//! it, never a slow answer: a server a model drives must not be made to
//! compute a century of days by one typo.

use core::fmt::Write as _;

use serde_json::{Map, Value};
use teistro::Error;

/// How large a request may be; `None` leaves that measure unbounded.
///
/// ```
/// use teistro_mcp::{Engine, Limits, Server};
///
/// // Two years of days in one call, the rest as shipped.
/// let limits = Limits { days: Some(732), ..Limits::default() };
/// let server = Server::new(Engine::Builtin).with_limits(limits);
/// # let _ = server;
/// // An operator who bounds nothing.
/// assert_eq!(Limits::UNBOUNDED.days, None);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// The most bytes one JSON-RPC message carries.
    pub message_bytes: Option<usize>,
    /// The most members of any one array inside a tool's record: a
    /// batch's instants, a corpus's charts, a list of subjects.
    pub items: Option<usize>,
    /// The most days a record's ranges span, summed: each object holding
    /// `first` and `last` as `{year, month, day}` is a range of days,
    /// counted inclusive, and a transit window (a `hits` section's `from`
    /// to `to`, searched day by day) its days begun. A window of life, a
    /// progression's, is not one: its cost is a day a year.
    pub days: Option<u64>,
    /// The most years the windows of Saturn's spells span, summed: a
    /// `sadeSati` section's `from` to `to`, its years begun. The search
    /// follows Saturn alone, so it costs a few hundredths of a second a
    /// year rather than a day's, and runs as far as the ephemeris reaches.
    pub years: Option<u64>,
}

impl Limits {
    /// Nothing bounded, for a host that bounds its callers itself.
    pub const UNBOUNDED: Limits = Limits {
        message_bytes: None,
        items: None,
        days: None,
        years: None,
    };
}

impl Default for Limits {
    /// The shipped bounds: a message of 16 MiB, an array of a thousand
    /// (a thousand charts found in a few seconds), a year of days (a
    /// leap year's almanac answers in seconds; a century would take the
    /// better part of half an hour), and a century and a half of Saturn's
    /// spells (a long life and more, found in about four seconds, where
    /// the built-in ephemeris's six centuries take six).
    fn default() -> Limits {
        Limits {
            message_bytes: Some(16 << 20),
            items: Some(1_000),
            days: Some(366),
            years: Some(150),
        }
    }
}

/// The command-line option moving each bound, which a refusal names.
pub(crate) const MESSAGE_OPTION: &str = "--max-message-bytes";
const ITEMS_OPTION: &str = "--max-items";
const DAYS_OPTION: &str = "--max-days";
const YEARS_OPTION: &str = "--max-years";

/// What a request has spent of the bounds that sum over it.
#[derive(Default)]
struct Spent {
    days: u64,
    years: u64,
}

impl Limits {
    /// Whether `request`, a tool's record, is inside the bounds; a
    /// refusal is `LIMIT`, its field the record's own spelling.
    ///
    /// Public for a host that hands callers' records to the SDK some
    /// other way and holds them to the server's bounds: the SDK itself
    /// computes whatever it is asked, so bounding a caller is the host's.
    ///
    /// ```
    /// use serde_json::json;
    /// use teistro_mcp::Limits;
    ///
    /// let century = json!({"periods": [{
    ///     "first": {"year": 1937, "month": 1, "day": 1},
    ///     "last": {"year": 2036, "month": 12, "day": 31},
    /// }]});
    /// let refused = Limits::default().check(&century).unwrap_err();
    /// assert_eq!(refused.field(), Some("periods[0].last"));
    /// assert!(Limits::UNBOUNDED.check(&century).is_ok());
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `LIMIT` naming the array or the range's `last` that passed a
    /// bound, the bound, and the option that moves it.
    pub fn check(&self, request: &Value) -> Result<(), Error> {
        self.walk(request, &mut String::new(), &mut Spent::default())
    }

    /// Whether an engine operation's arguments are inside the bounds,
    /// each field named as the operation spells it.
    pub(crate) fn check_fields(&self, fields: &Map<String, Value>) -> Result<(), Error> {
        self.fields(fields, &mut String::new(), &mut Spent::default())
    }

    fn walk(&self, value: &Value, path: &mut String, spent: &mut Spent) -> Result<(), Error> {
        match value {
            Value::Array(members) => {
                if let Some(most) = self.items.filter(|most| members.len() > *most) {
                    return Err(Error::limit(format!(
                        "`{path}` holds {} members; the server reads at most {most}",
                        members.len()
                    ))
                    .with_field(path.clone())
                    .with_hint(format!(
                        "split the batch into calls of {most} or fewer, or start the server \
                         with `{ITEMS_OPTION}`"
                    )));
                }
                for (at, member) in members.iter().enumerate() {
                    let length = path.len();
                    let _ = write!(path, "[{at}]");
                    self.walk(member, path, spent)?;
                    path.truncate(length);
                }
            }
            Value::Object(fields) => self.fields(fields, path, spent)?,
            _ => {}
        }
        Ok(())
    }

    fn fields(
        &self,
        fields: &Map<String, Value>,
        path: &mut String,
        spent: &mut Spent,
    ) -> Result<(), Error> {
        let key = path.rsplit('.').next().unwrap_or_default();
        let at = |end: &str| {
            if path.is_empty() {
                String::from(end)
            } else {
                format!("{path}.{end}")
            }
        };
        if key == SPELLS {
            spent.years = spent.years.saturating_add(begun(fields, YEAR));
            if let Some(most) = self.years.filter(|most| spent.years > *most) {
                let to = at("to");
                return Err(Error::limit(format!(
                    "the request's windows of Saturn's spells span {} years to `{to}`; the \
                     server searches at most {most} in one call",
                    spent.years
                ))
                .with_field(to)
                .with_hint(format!(
                    "ask in windows of {most} years or fewer, or start the server with \
                     `{YEARS_OPTION}`"
                )));
            }
        } else {
            let (spanned, end) = if WINDOWS.contains(&key) {
                (begun(fields, 1.0), "to")
            } else {
                (span(fields), "last")
            };
            spent.days = spent.days.saturating_add(spanned);
            if let Some(most) = self.days.filter(|most| spent.days > *most) {
                let last = at(end);
                return Err(Error::limit(format!(
                    "the request spans {} days to `{last}`; the server answers at most \
                     {most} in one call",
                    spent.days
                ))
                .with_field(last)
                .with_hint(format!(
                    "ask in ranges of {most} days or fewer, or start the server with \
                     `{DAYS_OPTION}`"
                )));
            }
        }
        for (key, member) in fields {
            let length = path.len();
            if !path.is_empty() {
                path.push('.');
            }
            path.push_str(key);
            self.walk(member, path, spent)?;
            path.truncate(length);
        }
        Ok(())
    }
}

/// The sections whose `from` to `to` is a transit window, searched day by
/// day: a year of one chart's hits takes about a second and a half
/// (`03-design/transit-hit-list.md`), so a century would hold a worker for
/// minutes.
const WINDOWS: &[&str] = &["hits"];

/// The section whose `from` to `to` is a window of Saturn's spells,
/// counted in years: Saturn alone is followed, a year in about three
/// hundredths of a second, and the search runs on to the ephemeris's
/// edge, so a window of millennia on an ephemeris that reaches them holds
/// a worker for minutes. The fuzzer's 670,000 years ran the built-in's
/// whole six centuries past its timeout.
const SPELLS: &str = "sadeSati";

/// A Julian year, the unit a window of Saturn's spells is counted in.
const YEAR: f64 = 365.25;

/// The units of `unit` days a window's `from` to `to` (Julian days)
/// begins; none for a window its reader will refuse.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "positive and whole, and a float cast saturates past u64::MAX"
)]
fn begun(fields: &Map<String, Value>, unit: f64) -> u64 {
    let instant = |key: &str| fields.get(key).and_then(Value::as_f64);
    let (Some(from), Some(to)) = (instant("from"), instant("to")) else {
        return 0;
    };
    let between = ((to - from) / unit).ceil();
    if between.is_finite() && between > 0.0 {
        between as u64
    } else {
        0
    }
}

/// The days an object's `first` to `last` span, inclusive; one for a
/// `first` alone, and none for an object that is not a range or a range
/// its reader will refuse. Every calendar's dates are counted as
/// Gregorian ones, which their years, months and days approximate to a
/// few days in a bound of hundreds.
fn span(fields: &Map<String, Value>) -> u64 {
    let Some(first) = fields.get("first").and_then(civil_day) else {
        return 0;
    };
    match fields.get("last") {
        None | Some(Value::Null) => 1,
        Some(last) => civil_day(last)
            .and_then(|last| last.checked_sub(first))
            .and_then(|between| u64::try_from(between).ok())
            .map_or(0, |between| between.saturating_add(1)),
    }
}

/// `{year, month, day}` as the days since 1970-01-01 on the proleptic
/// Gregorian calendar (Hinnant's `days_from_civil`), or None for
/// anything else.
fn civil_day(date: &Value) -> Option<i64> {
    let part = |key: &str| date.get(key).and_then(Value::as_i64);
    let (year, month, day) = (part("year")?, part("month")?, part("day")?);
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) || year.unsigned_abs() > 1 << 40 {
        return None;
    }
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let of_era = year.rem_euclid(400);
    let shifted = if month > 2 { month - 3 } else { month + 9 };
    let of_year = (153 * shifted + 2) / 5 + day - 1;
    let of_era_days = of_era * 365 + of_era / 4 - of_era / 100 + of_year;
    Some(era * 146_097 + of_era_days - 719_468)
}

/// The refusal of a message longer than `most` bytes.
pub(crate) fn too_long(most: usize) -> String {
    format!(
        "the message is longer than {most} bytes, the most the server reads: start it with \
         `{MESSAGE_OPTION}` to read longer"
    )
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use serde_json::json;

    use super::*;

    #[test]
    fn a_civil_day_counts_leap_years_and_eras() {
        let day = |year, month, day| civil_day(&json!({"year": year, "month": month, "day": day}));
        assert_eq!(day(1970, 1, 1), Some(0));
        assert_eq!(day(2000, 3, 1).unwrap() - day(2000, 2, 28).unwrap(), 2);
        assert_eq!(day(1900, 3, 1).unwrap() - day(1900, 2, 28).unwrap(), 1);
        assert_eq!(day(2026, 1, 1).unwrap() - day(2025, 1, 1).unwrap(), 365);
        assert_eq!(day(1, 1, 1).unwrap() - day(0, 1, 1).unwrap(), 366);
        assert_eq!(day(-400, 1, 1).unwrap() - day(-800, 1, 1).unwrap(), 146_097);
        assert_eq!(day(2026, 13, 1), None);
        assert_eq!(civil_day(&json!({"year": 2026, "month": 1})), None);
    }

    #[test]
    fn a_backwards_or_unreadable_range_is_left_to_its_reader() {
        let first = json!({"year": 2026, "month": 1, "day": 2});
        let earlier = json!({"year": 2026, "month": 1, "day": 1});
        let range = |last: Value| {
            let mut fields = Map::new();
            fields.insert(String::from("first"), first.clone());
            fields.insert(String::from("last"), last);
            span(&fields)
        };
        assert_eq!(range(earlier), 0);
        assert_eq!(range(json!("tomorrow")), 0);
        assert_eq!(range(first.clone()), 1);
        assert_eq!(range(Value::Null), 1);
    }
}
