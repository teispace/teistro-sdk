//! An iCalendar feed (RFC 5545), written the same way twice.
//!
//! A pure function from a [`Feed`] to text: no clock and no I/O, so two
//! runs over one feed write the same bytes, and every binding that asks
//! the façade for a feed reads those bytes back
//! (`03-design/icalendar.md`).
//!
//! - Every line ends in CRLF, and a line longer than 75 octets is folded
//!   by CRLF and one space, never inside a UTF-8 sequence: a Devanagari
//!   name is three octets a character (§3.1).
//! - TEXT escapes `\`, `;` and `,`, writes a newline as `\n`, and drops
//!   the control characters the format forbids (§3.3.11).
//! - An event is a whole day, `DTSTART;VALUE=DATE` and the next day as
//!   `DTEND`, which is exclusive (§3.6.1), or a span of time in UTC,
//!   `DTSTART` and `DTEND` as `YYYYMMDDTHHMMSSZ` (§3.3.5's form 2). A
//!   lunar date does not recur in Gregorian terms, so there is no
//!   `RRULE`: each occurrence is its own event.
//! - `DTSTAMP` is the feed's own stamp, which the caller gives.
//! - An event's `UID` is unique in its feed (§3.8.4.7): a second of the
//!   same `UID` is refused, not written for a client to merge at will.
//! - [`Hints`] add `CATEGORIES` (§3.8.1.2), `COLOR` (RFC 7986 §5.9) and a
//!   `VALARM` (§3.6.6); a feed takes the first two for itself (RFC 7986
//!   §5.6, §5.9).
//!
//! ```
//! use teistro_calendar::{CalendarSystem, Gregorian};
//! use teistro_calendar::ical::{write, Event, Feed, Hints, When};
//!
//! let feed = Feed {
//!     product: "-//Example//Almanac//EN".to_owned(),
//!     name: "Festivals".to_owned(),
//!     refresh_minutes: Some(1440),
//!     stamp_unix_seconds: 1_767_225_600,
//!     categories: Vec::new(),
//!     color: None,
//!     events: vec![Event {
//!         uid: "a1@example".to_owned(),
//!         summary: "Vijaya Dashami".to_owned(),
//!         description: None,
//!         when: When::Day(Gregorian.to_fixed_ymd(2026, 10, 21)?),
//!         hints: Hints::default(),
//!     }],
//! };
//! let text = write(&feed)?;
//! assert!(text.starts_with("BEGIN:VCALENDAR\r\nVERSION:2.0\r\n"));
//! assert!(text.contains("DTSTART;VALUE=DATE:20261021\r\nDTEND;VALUE=DATE:20261022\r\n"));
//! assert!(text.contains("DTSTAMP:20260101T000000Z\r\n"));
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

mod color;

use std::collections::BTreeSet;
use std::fmt::Write as _;

pub use color::Color;

use crate::{CalendarSystem, FixedDay, Gregorian};
use teistro_core::error::{Error, Status};

/// A feed: the calendar's own properties and its events.
///
/// ```
/// use teistro_calendar::ical::{Feed, write};
///
/// // A feed with no events is still a calendar a subscriber can add.
/// let feed = Feed {
///     product: "-//Example//Almanac//EN".to_owned(),
///     name: "Festivals".to_owned(),
///     refresh_minutes: Some(1440),
///     stamp_unix_seconds: 1_767_225_600,
///     categories: Vec::new(),
///     color: None,
///     events: Vec::new(),
/// };
/// let text = write(&feed)?;
/// assert!(text.contains("REFRESH-INTERVAL;VALUE=DURATION:P1D\r\n"));
/// assert!(text.ends_with("END:VCALENDAR\r\n"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Feed {
    /// `PRODID`, as `-//Owner//Product//EN`.
    pub product: String,
    /// The calendar's name, written as `NAME` (RFC 7986) and as
    /// `X-WR-CALNAME`, which Apple and Google read.
    pub name: String,
    /// How often a subscriber should fetch the feed again, written as
    /// `REFRESH-INTERVAL` (RFC 7986) and as `X-PUBLISHED-TTL`, which
    /// Outlook reads; none, or zero, writes neither.
    pub refresh_minutes: Option<u32>,
    /// Every event's `DTSTAMP`, in seconds since 1970-01-01T00:00:00Z.
    pub stamp_unix_seconds: i64,
    /// The feed's own `CATEGORIES` (RFC 7986 §5.6); none writes no line.
    pub categories: Vec<String>,
    /// The feed's `COLOR` (RFC 7986 §5.9), and as
    /// `X-APPLE-CALENDAR-COLOR`, which Apple's clients read.
    pub color: Option<Color>,
    /// The events, written in this order.
    pub events: Vec<Event>,
}

/// One event, a whole day or a span of time.
///
/// ```
/// use teistro_calendar::ical::{Event, Feed, Hints, When, write};
/// use teistro_calendar::{CalendarSystem, Gregorian};
///
/// let event = Event {
///     uid: "dashain-10@example".to_owned(),
///     summary: "Dashain, day 10".to_owned(),
///     description: Some("Tika; then the visits".to_owned()),
///     when: When::Day(Gregorian.to_fixed_ymd(2026, 10, 21)?),
///     hints: Hints::default(),
/// };
/// let feed = Feed { product: "-//Example//EN".to_owned(), name: "Days".to_owned(), refresh_minutes: None, stamp_unix_seconds: 0, categories: Vec::new(), color: None, events: vec![event] };
/// let text = write(&feed)?;
/// // TEXT escapes commas and semicolons.
/// assert!(text.contains("SUMMARY:Dashain\\, day 10\r\n"));
/// assert!(text.contains("DESCRIPTION:Tika\\; then the visits\r\n"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    /// `UID`: the same for the same occurrence, so a feed published again
    /// updates the event rather than adding a second.
    pub uid: String,
    /// `SUMMARY`.
    pub summary: String,
    /// `DESCRIPTION`, when there is one.
    pub description: Option<String>,
    /// When it is.
    pub when: When,
    /// Its categories, colour and alarm, each optional.
    pub hints: Hints,
}

/// What an event may say beyond when it is: `CATEGORIES`, `COLOR` and a
/// `VALARM`. A client may ignore any of them, so each is a hint.
///
/// ```
/// use teistro_calendar::ical::{Alarm, Color, Event, Feed, Hints, When, write};
/// use teistro_calendar::{CalendarSystem, Gregorian};
///
/// let hints = Hints {
///     categories: vec!["HOLIDAY".to_owned(), "Sun, Moon".to_owned()],
///     color: Some(Color::named("gold")?),
///     alarm: Some(Alarm { trigger_minutes: -60, description: None }),
/// };
/// let event = Event {
///     uid: "u@example".to_owned(),
///     summary: "Dashain".to_owned(),
///     description: None,
///     when: When::Day(Gregorian.to_fixed_ymd(2026, 10, 21)?),
///     hints,
/// };
/// let feed = Feed { product: "-//Example//EN".to_owned(), name: "Days".to_owned(), refresh_minutes: None, stamp_unix_seconds: 0, categories: Vec::new(), color: None, events: vec![event] };
/// let text = write(&feed)?;
/// // A comma inside a category is escaped; the one between them is not.
/// assert!(text.contains("CATEGORIES:HOLIDAY,Sun\\, Moon\r\n"));
/// assert!(text.contains("COLOR:gold\r\n"));
/// assert!(text.contains("BEGIN:VALARM\r\nACTION:DISPLAY\r\nTRIGGER:-PT1H\r\nDESCRIPTION:Dashain\r\nEND:VALARM\r\n"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Hints {
    /// `CATEGORIES`, in this order.
    pub categories: Vec<String>,
    /// `COLOR`.
    pub color: Option<Color>,
    /// A `VALARM`.
    pub alarm: Option<Alarm>,
}

/// A display alarm (§3.6.6), set relative to the event's start.
///
/// ```
/// use teistro_calendar::ical::Alarm;
///
/// // 09:00 on a whole day, whose start is the day's first moment.
/// let morning = Alarm { trigger_minutes: 540, description: Some("Today".to_owned()) };
/// assert_eq!(morning.trigger_minutes, 540);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Alarm {
    /// Minutes from the event's start to the alarm: negative before it,
    /// positive after.
    pub trigger_minutes: i32,
    /// `DESCRIPTION`, which a display alarm requires; the event's
    /// `SUMMARY` when none.
    pub description: Option<String>,
}

/// When an event is.
///
/// ```
/// use teistro_calendar::ical::{Event, Feed, Hints, When, write};
///
/// let span = |from_unix_seconds, to_unix_seconds| Feed {
///     product: "-//Example//EN".to_owned(),
///     name: "Hours".to_owned(),
///     refresh_minutes: None,
///     stamp_unix_seconds: 0,
///     categories: Vec::new(),
///     color: None,
///     events: vec![Event {
///         uid: "u@example".to_owned(),
///         summary: "An hour".to_owned(),
///         description: None,
///         when: When::Span { from_unix_seconds, to_unix_seconds },
///         hints: Hints::default(),
///     }],
/// };
/// let text = write(&span(1_767_225_600, 1_767_229_200))?;
/// assert!(text.contains("DTSTART:20260101T000000Z\r\nDTEND:20260101T010000Z\r\n"));
/// // A span must end after it starts.
/// assert!(write(&span(1_767_225_600, 1_767_225_600)).is_err());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum When {
    /// A whole day, ending at the next day's start.
    Day(FixedDay),
    /// A span of time, each end in seconds since 1970-01-01T00:00:00Z and
    /// written in UTC; the end is exclusive.
    Span {
        /// The start.
        from_unix_seconds: i64,
        /// The end, after the start.
        to_unix_seconds: i64,
    },
}

/// The longest a line may be, in octets, before its CRLF (§3.1).
const LINE_OCTETS: usize = 75;

/// The fixed day of 1970-01-01, the Unix epoch.
const UNIX_EPOCH: FixedDay = FixedDay::new(719_163);

/// The feed as RFC 5545 text.
///
/// ```
/// use teistro_calendar::ical::{Feed, write};
///
/// let feed = Feed {
///     product: "-//Example//EN".to_owned(),
///     name: "Days".to_owned(),
///     refresh_minutes: None,
///     stamp_unix_seconds: 0,
///     categories: Vec::new(),
///     color: None,
///     events: Vec::new(),
/// };
/// // The same feed writes the same bytes, every line ended by CRLF.
/// let text = write(&feed)?;
/// assert_eq!(text, write(&feed)?);
/// assert!(text.split_inclusive('\n').all(|line| line.ends_with("\r\n")));
/// // The format's four-digit year cannot write a stamp in the year 10000.
/// assert!(write(&Feed { stamp_unix_seconds: 253_402_300_800, ..feed }).is_err());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// `OUT_OF_RANGE` for a day, a span's end or a stamp outside the years 1
/// to 9999, which the format's four-digit year cannot write;
/// `INVALID_ARG` for a span that does not end after it starts, an empty
/// `UID` or category, or a `UID` two events share.
pub fn write(feed: &Feed) -> Result<String, Error> {
    let stamp = date_time(feed.stamp_unix_seconds)?;
    let mut uids = BTreeSet::new();
    let mut out = String::new();
    line(&mut out, "BEGIN:VCALENDAR");
    line(&mut out, "VERSION:2.0");
    line(&mut out, &format!("PRODID:{}", text(&feed.product)));
    line(&mut out, "CALSCALE:GREGORIAN");
    line(&mut out, "METHOD:PUBLISH");
    line(&mut out, &format!("NAME:{}", text(&feed.name)));
    line(&mut out, &format!("X-WR-CALNAME:{}", text(&feed.name)));
    if let Some(minutes) = feed.refresh_minutes.filter(|&minutes| minutes > 0) {
        let duration = duration(minutes);
        line(
            &mut out,
            &format!("REFRESH-INTERVAL;VALUE=DURATION:{duration}"),
        );
        line(&mut out, &format!("X-PUBLISHED-TTL:{duration}"));
    }
    categories(&mut out, &feed.categories)?;
    if let Some(color) = feed.color {
        line(&mut out, &format!("COLOR:{}", color.name()));
        line(&mut out, &format!("X-APPLE-CALENDAR-COLOR:{}", color.hex()));
    }
    for event in &feed.events {
        if event.uid.is_empty() {
            return Err(Error::invalid_arg(format!(
                "the event {:?} has no UID, which §3.6.1 requires",
                event.summary
            )));
        }
        if !uids.insert(event.uid.as_str()) {
            return Err(Error::invalid_arg(format!(
                "two events have the UID {}, which §3.8.4.7 makes unique",
                event.uid
            )));
        }
        line(&mut out, "BEGIN:VEVENT");
        line(&mut out, &format!("UID:{}", text(&event.uid)));
        line(&mut out, &format!("DTSTAMP:{stamp}"));
        match event.when {
            When::Day(day) => {
                line(&mut out, &format!("DTSTART;VALUE=DATE:{}", date(day)?));
                line(
                    &mut out,
                    &format!("DTEND;VALUE=DATE:{}", date(day.plus_days(1))?),
                );
            }
            When::Span {
                from_unix_seconds,
                to_unix_seconds,
            } => {
                if to_unix_seconds <= from_unix_seconds {
                    return Err(Error::invalid_arg(format!(
                        "the event {} ends at {to_unix_seconds}, not after its start at {from_unix_seconds}",
                        event.uid
                    )));
                }
                line(
                    &mut out,
                    &format!("DTSTART:{}", date_time(from_unix_seconds)?),
                );
                line(&mut out, &format!("DTEND:{}", date_time(to_unix_seconds)?));
            }
        }
        line(&mut out, &format!("SUMMARY:{}", text(&event.summary)));
        if let Some(description) = &event.description {
            line(&mut out, &format!("DESCRIPTION:{}", text(description)));
        }
        categories(&mut out, &event.hints.categories)?;
        if let Some(color) = event.hints.color {
            line(&mut out, &format!("COLOR:{}", color.name()));
        }
        line(&mut out, "TRANSP:TRANSPARENT");
        if let Some(alarm) = &event.hints.alarm {
            line(&mut out, "BEGIN:VALARM");
            line(&mut out, "ACTION:DISPLAY");
            line(
                &mut out,
                &format!("TRIGGER:{}", trigger(alarm.trigger_minutes)),
            );
            let said = alarm.description.as_ref().unwrap_or(&event.summary);
            line(&mut out, &format!("DESCRIPTION:{}", text(said)));
            line(&mut out, "END:VALARM");
        }
        line(&mut out, "END:VEVENT");
    }
    line(&mut out, "END:VCALENDAR");
    Ok(out)
}

/// One content line, folded and ended by CRLF. A continuation begins with
/// one space, which counts towards its 75 octets.
fn line(out: &mut String, content: &str) {
    let mut room = LINE_OCTETS;
    let mut used = 0;
    for character in content.chars() {
        let octets = character.len_utf8();
        if used + octets > room {
            out.push_str("\r\n ");
            room = LINE_OCTETS - 1;
            used = 0;
        }
        out.push(character);
        used += octets;
    }
    out.push_str("\r\n");
}

/// A `CATEGORIES` line, when there are any: each value escaped, the commas
/// between them not (§3.8.1.2).
fn categories(out: &mut String, categories: &[String]) -> Result<(), Error> {
    if categories.is_empty() {
        return Ok(());
    }
    if categories.iter().any(String::is_empty) {
        return Err(Error::invalid_arg(
            "an empty category names nothing a client can show",
        ));
    }
    let values: Vec<String> = categories.iter().map(|value| text(value)).collect();
    line(out, &format!("CATEGORIES:{}", values.join(",")));
    Ok(())
}

/// A TEXT value, escaped (§3.3.11). The format allows no control
/// character but a tab, so the rest are dropped.
fn text(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\\' => out.push_str("\\\\"),
            ';' => out.push_str("\\;"),
            ',' => out.push_str("\\,"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push('\t'),
            // A CR alone or before a newline says nothing a reader keeps.
            control if control.is_ascii_control() => {}
            other => out.push(other),
        }
    }
    out
}

/// A DATE value, `YYYYMMDD`.
fn date(day: FixedDay) -> Result<String, Error> {
    let gregorian = Gregorian.date_of(day)?;
    if !(1..=9999).contains(&gregorian.year) {
        return Err(Error::new(
            Status::OutOfRange,
            format!(
                "an iCalendar date has a four-digit year, and {} is not one",
                gregorian.year
            ),
        ));
    }
    Ok(format!(
        "{:04}{:02}{:02}",
        gregorian.year, gregorian.month, gregorian.day
    ))
}

/// A DATE-TIME value in UTC, `YYYYMMDDTHHMMSSZ`.
fn date_time(unix_seconds: i64) -> Result<String, Error> {
    let day = UNIX_EPOCH.plus_days(unix_seconds.div_euclid(86_400));
    let second = unix_seconds.rem_euclid(86_400);
    let mut out = date(day)?;
    let _ = write!(
        out,
        "T{:02}{:02}{:02}Z",
        second / 3_600,
        second / 60 % 60,
        second % 60
    );
    Ok(out)
}

/// A DURATION value of whole minutes, in the largest units that write it
/// exactly: `P1D`, `PT6H`, `PT90M`.
fn duration(minutes: u32) -> String {
    if minutes % 1_440 == 0 {
        format!("P{}D", minutes / 1_440)
    } else if minutes % 60 == 0 {
        format!("PT{}H", minutes / 60)
    } else {
        format!("PT{minutes}M")
    }
}

/// A TRIGGER value: a DURATION from the start, signed; zero is `PT0S`.
fn trigger(minutes: i32) -> String {
    if minutes == 0 {
        return "PT0S".to_owned();
    }
    let sign = if minutes < 0 { "-" } else { "" };
    format!("{sign}{}", duration(minutes.unsigned_abs()))
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    fn feed(events: Vec<Event>) -> Feed {
        Feed {
            product: "-//Teispace//Teistro//EN".to_owned(),
            name: "Festivals".to_owned(),
            refresh_minutes: None,
            stamp_unix_seconds: 0,
            categories: Vec::new(),
            color: None,
            events,
        }
    }

    fn event(summary: &str) -> Event {
        Event {
            uid: "u@teistro".to_owned(),
            summary: summary.to_owned(),
            description: None,
            when: When::Day(Gregorian.to_fixed_ymd(2026, 1, 1).unwrap()),
            hints: Hints::default(),
        }
    }

    #[test]
    fn the_epoch_is_the_unix_epoch() {
        assert_eq!(Gregorian.to_fixed_ymd(1970, 1, 1).unwrap(), UNIX_EPOCH);
        assert_eq!(date_time(0).unwrap(), "19700101T000000Z");
        assert_eq!(date_time(-1).unwrap(), "19691231T235959Z");
        assert_eq!(
            date_time(1_792_454_400 + 3_723).unwrap(),
            "20261020T010203Z"
        );
    }

    #[test]
    fn every_line_ends_in_crlf_and_no_bare_newline_is_written() {
        let text = write(&feed(vec![event("a\nb")])).unwrap();
        assert!(text.ends_with("END:VCALENDAR\r\n"));
        assert_eq!(text.matches('\n').count(), text.matches("\r\n").count());
        assert!(text.contains("SUMMARY:a\\nb\r\n"));
    }

    #[test]
    fn text_escapes_the_four_it_must() {
        assert_eq!(text(r"a\b;c,d"), r"a\\b\;c\,d");
        assert_eq!(text("a\r\nb"), "a\\nb");
        // A colon needs no escape in a value.
        assert_eq!(text("a:b"), "a:b");
    }

    #[test]
    fn a_long_line_folds_at_75_octets_and_never_inside_a_character() {
        // Devanagari is three octets a character: 30 of them are 90.
        let name = "क".repeat(30);
        let text = write(&feed(vec![event(&name)])).unwrap();
        for line in text.split("\r\n") {
            assert!(line.len() <= LINE_OCTETS, "{} octets: {line}", line.len());
        }
        // Unfolded, the summary is whole again.
        let unfolded = text.replace("\r\n ", "");
        assert!(unfolded.contains(&format!("SUMMARY:{name}\r\n")));
        // "SUMMARY:" is 8 octets, and 22 characters (66) fill the first
        // line to 74: a 23rd would make 77.
        let first = text.lines().find(|l| l.starts_with("SUMMARY:")).unwrap();
        assert_eq!(first.trim_end().len(), 8 + 22 * 3);
    }

    #[test]
    fn a_line_of_exactly_75_octets_is_not_folded() {
        let mut out = String::new();
        line(&mut out, &"x".repeat(75));
        assert_eq!(out, format!("{}\r\n", "x".repeat(75)));
        let mut out = String::new();
        line(&mut out, &"x".repeat(76));
        assert_eq!(out, format!("{}\r\n x\r\n", "x".repeat(75)));
    }

    #[test]
    fn an_event_is_a_whole_day_and_its_end_is_the_next() {
        let mut last = event("New Year's Eve");
        last.when = When::Day(Gregorian.to_fixed_ymd(2026, 12, 31).unwrap());
        let text = write(&feed(vec![last])).unwrap();
        assert!(text.contains("DTSTART;VALUE=DATE:20261231\r\nDTEND;VALUE=DATE:20270101\r\n"));
    }

    #[test]
    fn a_span_is_written_in_utc_and_one_that_does_not_end_after_it_starts_is_refused() {
        let mut meal = event("Parana");
        // 2026-01-01T01:02:03Z to 03:04:05Z.
        let from = 1_767_229_323;
        meal.when = When::Span {
            from_unix_seconds: from,
            to_unix_seconds: from + 7_322,
        };
        let text = write(&feed(vec![meal.clone()])).unwrap();
        assert!(text.contains("DTSTART:20260101T010203Z\r\nDTEND:20260101T030405Z\r\n"));
        assert!(!text.contains("VALUE=DATE"));
        meal.when = When::Span {
            from_unix_seconds: from,
            to_unix_seconds: from,
        };
        let empty = write(&feed(vec![meal])).unwrap_err();
        assert_eq!(empty.status, Status::InvalidArg);
    }

    #[test]
    fn a_refresh_interval_is_written_both_ways_in_its_largest_unit() {
        let mut asked = feed(Vec::new());
        asked.refresh_minutes = Some(1_440);
        let text = write(&asked).unwrap();
        assert!(text.contains("REFRESH-INTERVAL;VALUE=DURATION:P1D\r\nX-PUBLISHED-TTL:P1D\r\n"));
        assert_eq!(duration(360), "PT6H");
        assert_eq!(duration(90), "PT90M");
        assert!(!write(&feed(Vec::new())).unwrap().contains("REFRESH"));
        // A client told to fetch again every zero minutes would poll.
        asked.refresh_minutes = Some(0);
        assert!(!write(&asked).unwrap().contains("REFRESH"));
    }

    #[test]
    fn the_same_feed_writes_the_same_bytes() {
        let mut second = event("b");
        second.uid = "v@teistro".to_owned();
        let asked = feed(vec![event("a"), second]);
        assert_eq!(write(&asked).unwrap(), write(&asked).unwrap());
    }

    #[test]
    fn a_control_character_is_dropped_and_a_tab_kept() {
        assert_eq!(text("a\u{7}b\tc\u{7f}"), "ab\tc");
    }

    #[test]
    fn hints_are_written_inside_the_event_and_the_alarm_ends_it() {
        let mut held = event("Dashain");
        held.hints = Hints {
            categories: vec!["a,b".to_owned(), "c".to_owned()],
            color: Some(Color::named("Gold").unwrap()),
            alarm: Some(Alarm {
                trigger_minutes: 540,
                description: Some("Today; rest".to_owned()),
            }),
        };
        let text = write(&feed(vec![held])).unwrap();
        assert!(text.contains("CATEGORIES:a\\,b,c\r\nCOLOR:gold\r\nTRANSP:TRANSPARENT\r\nBEGIN:VALARM\r\nACTION:DISPLAY\r\nTRIGGER:PT9H\r\nDESCRIPTION:Today\\; rest\r\nEND:VALARM\r\nEND:VEVENT\r\n"));
    }

    #[test]
    fn a_trigger_is_signed_and_zero_is_the_start() {
        assert_eq!(trigger(0), "PT0S");
        assert_eq!(trigger(-90), "-PT90M");
        assert_eq!(trigger(-1_440), "-P1D");
        assert_eq!(trigger(i32::MIN), "-PT2147483648M");
    }

    #[test]
    fn a_feed_says_its_categories_and_colour_both_ways() {
        let mut asked = feed(Vec::new());
        asked.categories = vec!["Festivals".to_owned()];
        asked.color = Some(Color::named("navy").unwrap());
        let text = write(&asked).unwrap();
        assert!(
            text.contains(
                "CATEGORIES:Festivals\r\nCOLOR:navy\r\nX-APPLE-CALENDAR-COLOR:#000080\r\n"
            )
        );
    }

    #[test]
    fn a_uid_is_unique_and_present_and_a_category_says_something() {
        let twice = write(&feed(vec![event("a"), event("b")])).unwrap_err();
        assert_eq!(twice.status, Status::InvalidArg);
        let mut bare = event("a");
        bare.uid = String::new();
        assert!(write(&feed(vec![bare])).is_err());
        let mut blank = feed(Vec::new());
        blank.categories = vec![String::new()];
        assert!(write(&blank).is_err());
    }

    #[test]
    fn a_year_the_format_cannot_write_is_refused() {
        let mut far = event("far");
        // Year 0 is astronomical 1 BCE: the calendar has it, the format not.
        far.when = When::Day(Gregorian.to_fixed_ymd(0, 6, 1).unwrap());
        assert!(write(&feed(vec![far])).is_err());
    }
}
