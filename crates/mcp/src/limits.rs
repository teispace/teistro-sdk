//! The bounds a request is held to (`03-design/mcp-server.md` §7, P5): a
//! message's text, an array's length and a range of days. A bound passed
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
    /// counted inclusive.
    pub days: Option<u64>,
}

impl Limits {
    /// Nothing bounded, for a host that bounds its callers itself.
    pub const UNBOUNDED: Limits = Limits {
        message_bytes: None,
        items: None,
        days: None,
    };
}

impl Default for Limits {
    /// The shipped bounds: a message of 16 MiB, an array of a thousand
    /// (a thousand charts found in a few seconds), and a year of days (a
    /// leap year's almanac answers in seconds; a century would take the
    /// better part of half an hour).
    fn default() -> Limits {
        Limits {
            message_bytes: Some(16 << 20),
            items: Some(1_000),
            days: Some(366),
        }
    }
}

/// The command-line option moving each bound, which a refusal names.
pub(crate) const MESSAGE_OPTION: &str = "--max-message-bytes";
const ITEMS_OPTION: &str = "--max-items";
const DAYS_OPTION: &str = "--max-days";

impl Limits {
    /// Whether `request`, a tool's record, is inside the bounds; a
    /// refusal is `LIMIT`, its field the record's own spelling.
    pub(crate) fn check(&self, request: &Value) -> Result<(), Error> {
        self.walk(request, &mut String::new(), &mut 0)
    }

    /// Whether an engine operation's arguments are inside the bounds,
    /// each field named as the operation spells it.
    pub(crate) fn check_fields(&self, fields: &Map<String, Value>) -> Result<(), Error> {
        self.fields(fields, &mut String::new(), &mut 0)
    }

    fn walk(&self, value: &Value, path: &mut String, days: &mut u64) -> Result<(), Error> {
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
                    self.walk(member, path, days)?;
                    path.truncate(length);
                }
            }
            Value::Object(fields) => self.fields(fields, path, days)?,
            _ => {}
        }
        Ok(())
    }

    fn fields(
        &self,
        fields: &Map<String, Value>,
        path: &mut String,
        days: &mut u64,
    ) -> Result<(), Error> {
        *days = days.saturating_add(span(fields));
        if let Some(most) = self.days.filter(|most| *days > *most) {
            let last = if path.is_empty() {
                String::from("last")
            } else {
                format!("{path}.last")
            };
            return Err(Error::limit(format!(
                "the request spans {days} days to `{last}`; the server answers at most \
                 {most} in one call"
            ))
            .with_field(last)
            .with_hint(format!(
                "ask in ranges of {most} days or fewer, or start the server with \
                 `{DAYS_OPTION}`"
            )));
        }
        for (key, member) in fields {
            let length = path.len();
            if !path.is_empty() {
                path.push('.');
            }
            path.push_str(key);
            self.walk(member, path, days)?;
            path.truncate(length);
        }
        Ok(())
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
