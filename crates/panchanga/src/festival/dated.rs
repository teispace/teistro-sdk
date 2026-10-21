//! Observances kept on a date of a calendar rather than a tithi
//! (`03-design/festival-rule-language.md` §3.6).
//!
//! Nepal's committee prints the sankrantis on the first day of each
//! Bikram Sambat month, whose months are the Sun's signs under the punya
//! convention the calendar already applies (C619), and the national days
//! on fixed Bikram Sambat dates: पृथ्वीजयन्ती on Poush 27, the earthquake
//! safety day on Magh 2 (VS 2082 p. 20). A solar day is so a civil date in
//! a solar calendar, and two forms say every such rule: a day of a month,
//! counted back from its end when negative, and the nth weekday of a
//! month.

use serde::{Deserialize, Serialize};
use teistro_calendar::{CalendarDate, CalendarSystem, shipped};
use teistro_core::catalogue::{Calendar, Vara};
use teistro_core::error::Error;

use super::{FestivalDay, Observances};

/// An observance on a date of a calendar.
///
/// ```
/// use teistro_panchanga::festival::{Dated, DatedRule};
/// use teistro_core::catalogue::Calendar;
///
/// let maghe = DatedRule::nepal_panchanga().into_iter().find(|rule| rule.key == "MAKARA_SANKRANTI").ok_or("shipped")?;
/// assert_eq!(maghe.on, Dated::CivilDate { calendar: Calendar::BikramSambat, month: 10, day: 1 });
/// # Ok::<(), &str>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct DatedRule {
    /// The rule's key in its pack, `"MAKARA_SANKRANTI"`.
    pub key: String,
    /// Where the rule is stated.
    pub source: String,
    /// The date, written as `civilDate` or `civilWeekday`.
    #[serde(flatten)]
    pub on: Dated,
}

/// The date a [`DatedRule`] keeps.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum Dated {
    /// A day of a month of `calendar`, 1 for the first; a negative day
    /// counts back from the month's end, -1 for its last.
    CivilDate {
        /// The calendar the date is in.
        calendar: Calendar,
        /// The month, 1 for the first.
        month: u8,
        /// The day; never 0.
        day: i8,
    },
    /// The `nth` `weekday` of a month of `calendar`; a negative `nth`
    /// counts back from the month's end, -1 for its last.
    CivilWeekday {
        /// The calendar the month is in.
        calendar: Calendar,
        /// The month, 1 for the first.
        month: u8,
        /// The weekday.
        weekday: Vara,
        /// Which of them; never 0, at most 5 either way.
        nth: i8,
    },
}

impl Dated {
    fn calendar(self) -> Calendar {
        match self {
            Dated::CivilDate { calendar, .. } | Dated::CivilWeekday { calendar, .. } => calendar,
        }
    }

    fn month(self) -> u8 {
        match self {
            Dated::CivilDate { month, .. } | Dated::CivilWeekday { month, .. } => month,
        }
    }

    /// Whether `date`, of a month `length` days long, falling on `vara`,
    /// is the date this names.
    fn holds(self, date: &CalendarDate, length: u8, vara: Vara) -> bool {
        if date.month != self.month() {
            return false;
        }
        let (day, length) = (i16::from(date.day), i16::from(length));
        match self {
            Dated::CivilDate { day: wanted, .. } => {
                let wanted = i16::from(wanted);
                day == if wanted > 0 {
                    wanted
                } else {
                    length + 1 + wanted
                }
            }
            Dated::CivilWeekday {
                weekday,
                nth: wanted,
                ..
            } => {
                let nth = if wanted > 0 {
                    (day - 1) / 7 + 1
                } else {
                    -((length - day) / 7 + 1)
                };
                vara == weekday && nth == i16::from(wanted)
            }
        }
    }
}

impl DatedRule {
    /// Refuses a rule that could not be kept: an empty key, a calendar
    /// the SDK does not ship, month 0, a day of 0 or past 32 either way,
    /// or an nth of 0 or past 5 either way.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG`, naming the field that is wrong.
    pub fn check(&self) -> Result<(), Error> {
        if self.key.trim().is_empty() {
            return Err(Error::invalid_arg("a festival rule needs a key").with_field("key"));
        }
        let (form, calendar) = match self.on {
            Dated::CivilDate { calendar, .. } => ("civilDate", calendar),
            Dated::CivilWeekday { calendar, .. } => ("civilWeekday", calendar),
        };
        if shipped(calendar).is_none() {
            return Err(Error::invalid_arg(format!(
                "{}: the SDK does not ship the `{}` calendar",
                self.key,
                calendar.key()
            ))
            .with_field(format!("{form}.calendar")));
        }
        if self.on.month() == 0 {
            return Err(
                Error::invalid_arg(format!("{}: months are counted from 1", self.key))
                    .with_field(format!("{form}.month")),
            );
        }
        match self.on {
            Dated::CivilDate { day, .. } if day == 0 || day.unsigned_abs() > 32 => {
                Err(Error::invalid_arg(format!(
                    "{}: a day is counted from 1, or back from the month's end from -1, not {day}",
                    self.key
                ))
                .with_field("civilDate.day"))
            }
            Dated::CivilWeekday { nth, .. } if nth == 0 || nth.unsigned_abs() > 5 => {
                Err(Error::invalid_arg(format!(
                    "{}: a month holds a weekday at most five times; count from 1, or back from -1, not {nth}",
                    self.key
                ))
                .with_field("civilWeekday.nth"))
            }
            _ => Ok(()),
        }
    }

    /// The days Nepal's national panchanga prints on a Bikram Sambat date:
    /// the twelve sankrantis on the first of each month, the national days
    /// it names, and the festivals it keeps on a date.
    #[must_use]
    pub fn nepal_panchanga() -> Vec<DatedRule> {
        let bs = |key: &str, month: u8, day: i8, source: &str| DatedRule {
            key: key.to_owned(),
            source: source.to_owned(),
            on: Dated::CivilDate {
                calendar: Calendar::BikramSambat,
                month,
                day,
            },
        };
        let sankranti = |key: &str, month: u8, printed: &str| {
            bs(
                key,
                month,
                1,
                &format!(
                    "Nepal's national panchanga, VS 2082: \"{printed}\" on the first of the Bikram Sambat month, the day the Sun enters its sign under the punya convention (C619)"
                ),
            )
        };
        vec![
            sankranti("MESHA_SANKRANTI", 1, "वैशाखसंक्रान्ति, नववर्षारम्भ"),
            sankranti("VRISHABHA_SANKRANTI", 2, "ज्येष्ठसंक्रान्ति"),
            sankranti("MITHUNA_SANKRANTI", 3, "आषाढसंक्रान्ति"),
            sankranti("KARKATA_SANKRANTI", 4, "श्रावणसङ्क्रान्ति, दक्षिणायनारम्भ"),
            sankranti("SIMHA_SANKRANTI", 5, "भाद्रसङ्क्रान्ति"),
            sankranti("KANYA_SANKRANTI", 6, "आश्विनसङ्क्रान्ति"),
            sankranti("TULA_SANKRANTI", 7, "कार्तिकसङ्क्रान्ति"),
            sankranti("VRISHCHIKA_SANKRANTI", 8, "मार्गसङ्क्रान्ति"),
            sankranti("DHANU_SANKRANTI", 9, "पौषसङ्क्रान्ति"),
            sankranti("MAKARA_SANKRANTI", 10, "माघसङ्क्रान्ति, उत्तरायण सुरु"),
            sankranti("KUMBHA_SANKRANTI", 11, "फाल्गुनसङ्क्रान्ति"),
            sankranti("MINA_SANKRANTI", 12, "चैत्रसङ्क्रान्ति"),
            bs(
                "LOKTANTRA_DIWAS",
                1,
                11,
                "Nepal's national panchanga, VS 2082 p. 2: \"लोकतन्त्र दिवस\" on Baisakh 11",
            ),
            bs(
                "CONSTITUTION_DAY",
                6,
                3,
                "Nepal's national panchanga, VS 2082 p. 12: \"संविधानदिवस\" on Asoj 3",
            ),
            bs(
                "PRITHVI_JAYANTI",
                9,
                27,
                "Nepal's national panchanga, VS 2082 p. 20 and VS 2083 p. 21: \"पृथ्वीजयन्ती, राष्ट्रिय एकता दिवस\" on Poush 27",
            ),
            bs(
                "EARTHQUAKE_SAFETY_DAY",
                10,
                2,
                "Nepal's national panchanga, VS 2082 p. 20 and VS 2083 p. 21: \"राष्ट्रिय भूकम्प सुरक्षा दिवस\" on Magh 2",
            ),
            bs(
                "TAMU_LHOSAR",
                9,
                15,
                "Nepal's national panchanga, VS 2082 p. 19 and VS 2083 p. 20: \"तमुल्होछार\" on Poush 15",
            ),
            bs(
                "MARTYRS_DAY",
                10,
                16,
                "Nepal's national panchanga, VS 2082 p. 21 and VS 2083 p. 22: \"सहिद दिवस\" on Magh 16",
            ),
            bs(
                "DEMOCRACY_DAY",
                11,
                7,
                "Nepal's national panchanga, VS 2082 p. 23 and VS 2083 p. 23: \"राष्ट्रिय प्रजातन्त्र दिवस\" on Phalgun 7",
            ),
            bs(
                "WALKE_PARVA",
                5,
                1,
                "Nepal's national panchanga, VS 2082 p. 10 and VS 2083 p. 11: \"वल्केपर्व\" on Bhadra 1, the day of भाद्रसङ्क्रान्ति",
            ),
        ]
    }
}

/// An observance a [`DatedRule`] gave.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DatedObservance {
    /// The rule's key.
    pub rule: String,
    /// The day, in the range's calendar.
    pub day: CalendarDate,
    /// The same day in the rule's calendar.
    pub on: CalendarDate,
}

/// The observances each dated rule gives over `days`, in the order of
/// the days, then of the rules.
///
/// # Errors
///
/// A rule's own refusal ([`DatedRule::check`]), or a day its calendar
/// cannot convert.
pub fn dated(rules: &[DatedRule], days: &[FestivalDay]) -> Result<Observances, Error> {
    for rule in rules {
        rule.check()?;
    }
    let mut answer = Observances::default();
    for day in days {
        let fixed = system(day.date.calendar)?.fixed_of(&day.date)?;
        for rule in rules {
            let calendar = system(rule.on.calendar())?;
            let on = calendar.date_of(fixed)?;
            let length = calendar.month_length(on.year, on.month)?;
            if rule.on.holds(&on, length, day.vara) {
                answer.dated.push(DatedObservance {
                    rule: rule.key.clone(),
                    day: day.date.clone(),
                    on,
                });
            }
        }
    }
    Ok(answer)
}

fn system(calendar: Calendar) -> Result<&'static dyn CalendarSystem, Error> {
    shipped(calendar).ok_or_else(|| {
        Error::unsupported(format!(
            "the SDK does not ship the `{}` calendar",
            calendar.key()
        ))
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    fn date(month: u8, day: u8) -> CalendarDate {
        CalendarDate::defined(Calendar::BikramSambat, 2082, month, day)
    }

    fn civil(month: u8, day: i8) -> Dated {
        Dated::CivilDate {
            calendar: Calendar::BikramSambat,
            month,
            day,
        }
    }

    fn weekday(month: u8, weekday: Vara, nth: i8) -> Dated {
        Dated::CivilWeekday {
            calendar: Calendar::BikramSambat,
            month,
            weekday,
            nth,
        }
    }

    #[test]
    fn a_day_counts_from_the_first_or_back_from_the_last() {
        // A month of 30 days.
        assert!(civil(10, 1).holds(&date(10, 1), 30, Vara::Ravivara));
        assert!(!civil(10, 1).holds(&date(11, 1), 30, Vara::Ravivara));
        assert!(civil(12, -1).holds(&date(12, 30), 30, Vara::Ravivara));
        assert!(civil(12, -4).holds(&date(12, 27), 30, Vara::Ravivara));
        assert!(!civil(12, -4).holds(&date(12, 27), 31, Vara::Ravivara));
    }

    #[test]
    fn a_weekday_counts_by_the_weeks_from_either_end() {
        let saturday = Vara::Shanivara;
        // The 8th to the 14th hold each weekday's second.
        assert!(weekday(5, saturday, 2).holds(&date(5, 8), 31, saturday));
        assert!(weekday(5, saturday, 2).holds(&date(5, 14), 31, saturday));
        assert!(!weekday(5, saturday, 2).holds(&date(5, 15), 31, saturday));
        assert!(!weekday(5, saturday, 2).holds(&date(5, 8), 31, Vara::Ravivara));
        // In a 31-day month the 25th to the 31st hold each one's last.
        assert!(weekday(5, saturday, -1).holds(&date(5, 25), 31, saturday));
        assert!(!weekday(5, saturday, -1).holds(&date(5, 24), 31, saturday));
        assert!(weekday(5, saturday, -2).holds(&date(5, 24), 31, saturday));
    }

    #[test]
    fn a_rule_that_cannot_be_kept_is_refused_by_its_field() {
        let refused = |on| {
            DatedRule {
                key: "X".to_owned(),
                source: String::new(),
                on,
            }
            .check()
            .unwrap_err()
            .field()
            .map(str::to_owned)
        };
        assert_eq!(refused(civil(0, 1)).as_deref(), Some("civilDate.month"));
        assert_eq!(refused(civil(1, 0)).as_deref(), Some("civilDate.day"));
        assert_eq!(refused(civil(1, -33)).as_deref(), Some("civilDate.day"));
        assert_eq!(
            refused(weekday(1, Vara::Ravivara, 6)).as_deref(),
            Some("civilWeekday.nth")
        );
        assert_eq!(
            refused(weekday(1, Vara::Ravivara, 0)).as_deref(),
            Some("civilWeekday.nth")
        );
        for rule in DatedRule::nepal_panchanga() {
            rule.check().unwrap();
        }
    }

    #[test]
    fn a_rule_reads_as_its_json_record() {
        let rule: DatedRule = serde_json::from_str(
            r#"{"key": "SAFER_INTERNET_DAY", "source": "the organisers",
                "civilWeekday": {"calendar": "GREGORIAN", "month": 2, "weekday": "MANGALAVARA", "nth": 2}}"#,
        )
        .unwrap();
        assert_eq!(
            rule.on,
            Dated::CivilWeekday {
                calendar: Calendar::Gregorian,
                month: 2,
                weekday: Vara::Mangalavara,
                nth: 2
            }
        );
        let maghe = DatedRule::nepal_panchanga().into_iter().nth(9).unwrap();
        let text = serde_json::to_string(&maghe).unwrap();
        assert!(
            text.contains(r#""civilDate":{"calendar":"BIKRAM_SAMBAT","month":10,"day":1}"#),
            "{text}"
        );
    }
}
