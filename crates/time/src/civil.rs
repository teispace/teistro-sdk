//! A civil time of day, and a civil date-time in any calendar: what a
//! consumer types, before a zone turns it into an instant.

use core::fmt;
use core::str::FromStr;

use teistro_calendar::CalendarDate;
use teistro_core::quantity::InvalidValue;

/// Nanoseconds in a second.
pub const NANOS_PER_SECOND: u32 = 1_000_000_000;
/// Seconds in a civil day.
pub const SECONDS_PER_DAY: u32 = 86_400;

/// A time of day: hours, minutes, seconds and nanoseconds. The second
/// may be 60 for a leap second; whether the day had one is checked when
/// the time is resolved in a zone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct CivilTime {
    hour: u8,
    minute: u8,
    second: u8,
    nanos: u32,
}

impl CivilTime {
    /// 00:00:00.
    pub const MIDNIGHT: CivilTime = CivilTime {
        hour: 0,
        minute: 0,
        second: 0,
        nanos: 0,
    };
    /// 12:00:00.
    pub const NOON: CivilTime = CivilTime {
        hour: 12,
        minute: 0,
        second: 0,
        nanos: 0,
    };

    /// A time from its parts.
    ///
    /// # Errors
    ///
    /// An hour above 23, a minute above 59 or a second above 60, naming
    /// the part.
    pub fn new(hour: u8, minute: u8, second: u8) -> Result<CivilTime, InvalidValue> {
        let check = |value: u8, max: u8, quantity: &'static str, accepted: &'static str| {
            if value <= max {
                Ok(())
            } else {
                Err(InvalidValue {
                    quantity,
                    value: value.to_string(),
                    accepted,
                    field: Some(quantity.to_string()),
                })
            }
        };
        check(hour, 23, "hour", "0 to 23")?;
        check(minute, 59, "minute", "0 to 59")?;
        check(second, 60, "second", "0 to 59, or 60 for a leap second")?;
        Ok(CivilTime {
            hour,
            minute,
            second,
            nanos: 0,
        })
    }

    /// The same time with nanoseconds within the second.
    ///
    /// # Errors
    ///
    /// Nanoseconds of a thousand million or more.
    pub fn with_nanos(self, nanos: u32) -> Result<CivilTime, InvalidValue> {
        if nanos < NANOS_PER_SECOND {
            Ok(CivilTime { nanos, ..self })
        } else {
            Err(InvalidValue {
                quantity: "nanoseconds",
                value: nanos.to_string(),
                accepted: "below one thousand million",
                field: Some(String::from("nanos")),
            })
        }
    }

    /// A time from the seconds elapsed in the day, with nanoseconds.
    ///
    /// # Errors
    ///
    /// Seconds of 86 400 or more, or nanoseconds of a thousand million
    /// or more.
    pub fn from_seconds_of_day(seconds: u32, nanos: u32) -> Result<CivilTime, InvalidValue> {
        if seconds >= SECONDS_PER_DAY {
            return Err(InvalidValue {
                quantity: "seconds of the day",
                value: seconds.to_string(),
                accepted: "0 to 86399",
                field: Some(String::from("time")),
            });
        }
        let hour = u8::try_from(seconds / 3600).unwrap_or(23);
        let minute = u8::try_from(seconds / 60 % 60).unwrap_or(59);
        let second = u8::try_from(seconds % 60).unwrap_or(59);
        CivilTime::new(hour, minute, second)?.with_nanos(nanos)
    }

    /// The hour, 0 to 23.
    #[must_use]
    pub const fn hour(self) -> u8 {
        self.hour
    }

    /// The minute, 0 to 59.
    #[must_use]
    pub const fn minute(self) -> u8 {
        self.minute
    }

    /// The second, 0 to 60.
    #[must_use]
    pub const fn second(self) -> u8 {
        self.second
    }

    /// The nanoseconds within the second.
    #[must_use]
    pub const fn nanos(self) -> u32 {
        self.nanos
    }

    /// Whether the time names a leap second (a second of 60).
    #[must_use]
    pub const fn is_leap_second(self) -> bool {
        self.second == 60
    }

    /// The whole seconds elapsed in the day; 86 400 for 23:59:60.
    #[must_use]
    pub const fn seconds_of_day(self) -> u32 {
        self.hour as u32 * 3600 + self.minute as u32 * 60 + self.second as u32
    }

    /// The fraction of the day elapsed, for Julian-day arithmetic.
    #[must_use]
    pub fn day_fraction(self) -> f64 {
        (f64::from(self.seconds_of_day()) + f64::from(self.nanos) / f64::from(NANOS_PER_SECOND))
            / f64::from(SECONDS_PER_DAY)
    }
}

impl fmt::Display for CivilTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}:{:02}", self.hour, self.minute, self.second)?;
        if self.nanos != 0 {
            if self.nanos % 1_000_000 == 0 {
                write!(f, ".{:03}", self.nanos / 1_000_000)?;
            } else {
                write!(f, ".{:09}", self.nanos)?;
            }
        }
        Ok(())
    }
}

impl FromStr for CivilTime {
    type Err = InvalidValue;

    /// Parses `HH:MM`, `HH:MM:SS` or `HH:MM:SS.fraction`.
    fn from_str(text: &str) -> Result<CivilTime, InvalidValue> {
        let invalid = || InvalidValue {
            quantity: "time",
            value: text.to_string(),
            accepted: "HH:MM, HH:MM:SS or HH:MM:SS.fraction",
            field: Some(String::from("time")),
        };
        let mut parts = text.split(':');
        let hour: u8 = parts
            .next()
            .and_then(|p| p.parse().ok())
            .ok_or_else(invalid)?;
        let minute: u8 = parts
            .next()
            .and_then(|p| p.parse().ok())
            .ok_or_else(invalid)?;
        let (second, nanos) = match parts.next() {
            None => (0u8, 0u32),
            Some(rest) => {
                let (whole, fraction) = rest.split_once('.').unwrap_or((rest, ""));
                let second: u8 = whole.parse().map_err(|_| invalid())?;
                let nanos = if fraction.is_empty() {
                    0
                } else {
                    if fraction.len() > 9 || !fraction.bytes().all(|b| b.is_ascii_digit()) {
                        return Err(invalid());
                    }
                    let padded = format!("{fraction:0<9}");
                    padded.parse::<u32>().map_err(|_| invalid())?
                };
                (second, nanos)
            }
        };
        if parts.next().is_some() {
            return Err(invalid());
        }
        CivilTime::new(hour, minute, second)?.with_nanos(nanos)
    }
}

impl<'de> serde::Deserialize<'de> for CivilTime {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<CivilTime, D::Error> {
        #[derive(serde::Deserialize)]
        struct Parts {
            hour: u8,
            minute: u8,
            second: u8,
            #[serde(default)]
            nanos: u32,
        }
        let parts = Parts::deserialize(deserializer)?;
        CivilTime::new(parts.hour, parts.minute, parts.second)
            .and_then(|t| t.with_nanos(parts.nanos))
            .map_err(serde::de::Error::custom)
    }
}

/// A civil date in a calendar and, when known, the time of day.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CivilDateTime {
    /// The date, in its calendar.
    pub date: CalendarDate,
    /// The time, when known; the unknown-time policy supplies one.
    pub time: Option<CivilTime>,
}

impl CivilDateTime {
    /// A date at a time.
    #[must_use]
    pub const fn at(date: CalendarDate, time: CivilTime) -> CivilDateTime {
        CivilDateTime {
            date,
            time: Some(time),
        }
    }

    /// A date whose time is not known.
    #[must_use]
    pub const fn date_only(date: CalendarDate) -> CivilDateTime {
        CivilDateTime { date, time: None }
    }
}

impl fmt::Display for CivilDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.date)?;
        match self.time {
            Some(time) => write!(f, " {time}"),
            None => f.write_str(" (time unknown)"),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_core::catalogue::Calendar;

    use super::*;

    #[test]
    fn times_validate_render_and_parse() {
        let t = CivilTime::new(5, 30, 0).unwrap();
        assert_eq!((t.hour(), t.minute(), t.second(), t.nanos()), (5, 30, 0, 0));
        assert_eq!(t.seconds_of_day(), 19_800);
        assert!((t.day_fraction() - 19_800.0 / 86_400.0).abs() < 1e-15);
        assert_eq!(t.to_string(), "05:30:00");
        assert_eq!("05:30".parse::<CivilTime>().unwrap(), t);
        assert_eq!("05:30:00".parse::<CivilTime>().unwrap(), t);
        let fine = "23:59:59.25".parse::<CivilTime>().unwrap();
        assert_eq!(fine.nanos(), 250_000_000);
        assert_eq!(fine.to_string(), "23:59:59.250");
        let finer = t.with_nanos(123_456_789).unwrap();
        assert_eq!(finer.to_string(), "05:30:00.123456789");
        assert_eq!(
            CivilTime::from_seconds_of_day(19_800, 5).unwrap(),
            t.with_nanos(5).unwrap()
        );
        let leap = CivilTime::new(23, 59, 60).unwrap();
        assert!(leap.is_leap_second());
        assert_eq!(leap.seconds_of_day(), 86_400);
        assert_eq!(
            CivilTime::new(24, 0, 0).unwrap_err().field.as_deref(),
            Some("hour")
        );
        assert_eq!(
            CivilTime::new(0, 60, 0).unwrap_err().field.as_deref(),
            Some("minute")
        );
        assert!(CivilTime::new(0, 0, 61).is_err());
        assert!(t.with_nanos(NANOS_PER_SECOND).is_err());
        assert!(CivilTime::from_seconds_of_day(86_400, 0).is_err());
        for bad in [
            "5",
            "05:xx",
            "05:30:00:00",
            "05:30:00.1234567890",
            "05:30:00.x",
        ] {
            assert!(bad.parse::<CivilTime>().is_err(), "{bad}");
        }
        assert!(CivilTime::MIDNIGHT < CivilTime::NOON);
        let json = serde_json::to_string(&fine).unwrap();
        assert_eq!(serde_json::from_str::<CivilTime>(&json).unwrap(), fine);
        assert!(
            serde_json::from_str::<CivilTime>("{\"hour\":25,\"minute\":0,\"second\":0}").is_err()
        );
    }

    #[test]
    fn a_civil_date_time_carries_its_calendar() {
        let date = CalendarDate::defined(Calendar::Gregorian, 1990, 4, 14);
        let civil = CivilDateTime::at(date.clone(), CivilTime::new(5, 30, 0).unwrap());
        assert_eq!(civil.to_string(), "GREGORIAN 1990-04-14 05:30:00");
        let unknown = CivilDateTime::date_only(date);
        assert!(unknown.to_string().ends_with("(time unknown)"));
        let json = serde_json::to_string(&civil).unwrap();
        assert_eq!(serde_json::from_str::<CivilDateTime>(&json).unwrap(), civil);
    }
}
