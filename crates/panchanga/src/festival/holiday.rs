//! A government's holidays as its notices list them
//! (`03-design/nepal-gazette-holidays.md`, SR-123): each clause's words,
//! the festivals it names by their keys, whom it is for, and the days it
//! prints, year by year.
//!
//! A clause's words are kept apart from its festivals' names. Nepal's VS
//! 2083 notice gives Dhanya Purnima's day as "धान्य पूर्णिमा - किराँत
//! समुदायको उधौली पर्व, योमरी पुन्हि, ज्यापू दिवस": one holiday naming four
//! festivals, each of which has its own name in every locale. A reader
//! shows the notice as the notice, and each festival by its key. A clause
//! naming no festival the SDK keys (a state visit, a day of mourning) is
//! listed with none, its words carrying it.
//!
//! The days are the notice's own, in the Bikram Sambat calendar. A clause
//! that gives none ("on the day of Eid") has none: the festival's rule
//! gives the day, and the notice moves nothing.
//!
//! Each year's notice is data, `data/np-holidays/<year>.json`: its
//! citation, its clauses in its order, and the later notices amending it
//! (a holiday added, a clause's day fixed, a clause cancelled). One reader
//! makes every year's list, and a unit test holds every file to its shape.

use std::sync::LazyLock;

use serde::{Deserialize, Serialize};
use teistro_calendar::{BikramSambat, CalendarDay, CalendarSystem};
use teistro_core::catalogue::{Calendar, HolidayScope};

/// A clause's words, as printed and in English.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HolidayNotice {
    /// As the notice prints it, in Nepali.
    pub ne: String,
    /// In English.
    pub en: String,
}

/// One holiday of a notice.
///
/// ```
/// use teistro_core::catalogue::HolidayScope;
/// use teistro_panchanga::festival::HolidaySet;
///
/// let holidays = HolidaySet::NepalGazette.holidays_of(2083);
/// let poush_9 = holidays.iter().find(|holiday| holiday.festivals.len() == 4).ok_or("listed")?;
/// assert_eq!(poush_9.festivals, ["DHANYA_PURNIMA", "UDHAULI", "YOMARI_PUNHI", "JYAPU_DIWAS"]);
/// assert_eq!(poush_9.scope, HolidayScope::National);
/// assert_eq!(poush_9.year, 2083);
/// assert_eq!(poush_9.first.as_ref().map(|day| (day.year, day.month, day.day)), Some((2083, 9, 9)));
/// # Ok::<(), &str>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Holiday {
    /// The set it is of ([`HolidaySet::key`]).
    pub set: String,
    /// The Bikram Sambat year whose notice lists it.
    pub year: i32,
    /// Its clause's words.
    pub notice: HolidayNotice,
    /// The festivals the clause names, by key, in its order; empty for a
    /// clause naming none the SDK keys, which its notice names.
    pub festivals: Vec<String>,
    /// Whom it is for.
    pub scope: HolidayScope,
    /// Where it is kept, for a holiday the clause limits to a part of the
    /// country no ISO code names (`KATHMANDU_VALLEY`, `HILLS`, `TERAI`);
    /// empty when the clause names districts no key does, which its
    /// notice lists.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub regions: Vec<String>,
    /// For a holiday of a community or of a religion's followers, the
    /// community the clause names (`NEWAR`, `KIRAT`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub communities: Vec<String>,
    /// The first day the notice prints; absent when it gives the day as
    /// the festival's own.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first: Option<CalendarDay>,
    /// The last day, for a holiday of several days.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last: Option<CalendarDay>,
    /// The notice and its clause, and the later notice that dated or
    /// added it.
    pub source: String,
}

/// A government's list of holidays the SDK ships.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HolidaySet {
    /// Nepal's public, festival and substitute holidays, as the Ministry
    /// of Home Affairs' yearly notice in the Nepal Gazette (part 5) lists
    /// them, for every Bikram Sambat year from 2076 to 2083, with the
    /// later notices amending a year's.
    NepalGazette,
}

impl HolidaySet {
    /// Every set the SDK ships.
    pub const ALL: [HolidaySet; 1] = [HolidaySet::NepalGazette];

    /// Its key.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            HolidaySet::NepalGazette => "NEPAL_GAZETTE",
        }
    }

    /// The years of the calendar its days are in that it lists, in order.
    ///
    /// ```
    /// use teistro_panchanga::festival::HolidaySet;
    ///
    /// assert_eq!(HolidaySet::NepalGazette.years(), (2076..=2083).collect::<Vec<_>>());
    /// ```
    #[must_use]
    pub fn years(self) -> Vec<i32> {
        self.notices().iter().map(|year| year.year).collect()
    }

    /// Its holidays, year by year, each year's in its notice's order with
    /// the holidays later notices added after them.
    ///
    /// ```
    /// use teistro_panchanga::festival::HolidaySet;
    ///
    /// let holidays = HolidaySet::NepalGazette.holidays();
    /// assert_eq!(holidays.first().map(|holiday| holiday.year), Some(2076));
    /// assert_eq!(holidays.last().map(|holiday| holiday.year), Some(2083));
    /// ```
    #[must_use]
    pub fn holidays(self) -> Vec<Holiday> {
        self.notices()
            .iter()
            .flat_map(|year| year.holidays(self, &year.listed))
            .collect()
    }

    /// The holidays of one year, in its notice's order with the holidays
    /// later notices added after them; none for a year it does not list.
    ///
    /// ```
    /// use teistro_core::catalogue::HolidayScope;
    /// use teistro_panchanga::festival::HolidaySet;
    ///
    /// let holidays = HolidaySet::NepalGazette.holidays_of(2083);
    /// assert_eq!(holidays.len(), 45);
    /// assert!(holidays.iter().any(|holiday| holiday.scope == HolidayScope::Women));
    /// assert!(HolidaySet::NepalGazette.holidays_of(2050).is_empty());
    /// ```
    #[must_use]
    pub fn holidays_of(self, year: i32) -> Vec<Holiday> {
        self.notice(year)
            .map(|notice| notice.holidays(self, &notice.listed))
            .unwrap_or_default()
    }

    /// The holidays of one year a later notice cancelled, each with its
    /// clause's source and the cancelling notice's.
    ///
    /// ```
    /// use teistro_panchanga::festival::HolidaySet;
    ///
    /// let cancelled = HolidaySet::NepalGazette.cancelled(2080);
    /// assert_eq!(cancelled.len(), 1);
    /// assert_eq!(cancelled[0].notice.ne, "जनयुद्ध दिवस");
    /// assert!(cancelled[0].source.contains("cancelled by Nepal Gazette vol. 73 no. 54"));
    /// ```
    #[must_use]
    pub fn cancelled(self, year: i32) -> Vec<Holiday> {
        self.notice(year)
            .map(|notice| notice.holidays(self, &notice.cancelled))
            .unwrap_or_default()
    }

    /// The days of the festivals its holidays of several days in `year`
    /// are granted as, by key: the days the notice prints inside a span
    /// (Phulpati, Lakshmi Puja, Tihar's other days in VS 2078 to 2081),
    /// and for VS 2083,
    /// whose notice prints only the span's ends, Dashain's and Tihar's
    /// days the pack keys apart from the festivals the clause names, as
    /// the pack reckons them that year. A day inside its holiday's span is
    /// granted as the festival's.
    ///
    /// ```
    /// use teistro_panchanga::festival::HolidaySet;
    ///
    /// let days = HolidaySet::NepalGazette.festival_days(2083);
    /// assert!(days.iter().any(|(key, day)| key == "KUKUR_TIHAR" && (day.month, day.day) == (7, 22)));
    /// let printed = HolidaySet::NepalGazette.festival_days(2080);
    /// assert!(printed.iter().any(|(key, day)| key == "KIJA_PUJA" && (day.month, day.day) == (7, 29)));
    /// ```
    #[must_use]
    pub fn festival_days(self, year: i32) -> Vec<(String, CalendarDay)> {
        self.notice(year)
            .map(|notice| {
                notice
                    .listed
                    .iter()
                    .flat_map(|listed| &listed.row.days_of)
                    .map(|(key, on)| (key.clone(), bikram_sambat(year, *on)))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Every year's notice, in order.
    fn notices(self) -> &'static [Listing] {
        match self {
            HolidaySet::NepalGazette => &NEPAL_GAZETTE,
        }
    }

    /// One year's notice.
    fn notice(self, year: i32) -> Option<&'static Listing> {
        self.notices().iter().find(|notice| notice.year == year)
    }
}

/// Whom Nepal's notice of `year` gives `key` a holiday for, and where it
/// says so: the broadest of the clauses naming it (a nationwide clause
/// over one for some), `None` for a key no clause names or a year no
/// notice is shipped for. With no year, the latest notice is read.
///
/// ```
/// use teistro_core::catalogue::HolidayScope;
/// use teistro_panchanga::festival::nepal_gazette_scope;
///
/// let (scope, source) = nepal_gazette_scope("TAMU_LHOSAR", None).ok_or("named")?;
/// assert_eq!(scope, HolidayScope::National);
/// assert!(source.ends_with("clause 2.1 (ट)"));
/// assert_eq!(nepal_gazette_scope("GAURA_PARVA", None).map(|(scope, _)| scope), Some(HolidayScope::Concerned));
/// assert_eq!(nepal_gazette_scope("NO_SUCH_DAY", None), None);
/// // VS 2077's notice gave Tamu Lhosar only to those it concerns.
/// assert_eq!(nepal_gazette_scope("TAMU_LHOSAR", Some(2077)).map(|(scope, _)| scope), Some(HolidayScope::Concerned));
/// assert_eq!(nepal_gazette_scope("TAMU_LHOSAR", Some(2050)), None);
/// # Ok::<(), &str>(())
/// ```
#[must_use]
pub fn nepal_gazette_scope(key: &str, year: Option<i32>) -> Option<(HolidayScope, String)> {
    let notices = HolidaySet::NepalGazette.notices();
    let notice = match year {
        Some(year) => notices.iter().find(|notice| notice.year == year),
        None => notices.last(),
    }?;
    let granted = |listed: &&Listed| {
        listed.row.festivals.iter().any(|held| held == key)
            || listed.row.grants_as_its_festival(key)
    };
    let naming = notice.listed.iter().filter(granted);
    let listed = naming
        .clone()
        .find(|listed| listed.row.scope == HolidayScope::National)
        .or_else(|| naming.clone().next())?;
    let within = if listed.row.festivals.iter().any(|held| held == key) {
        ""
    } else {
        ", a day of the festival it grants"
    };
    Some((listed.row.scope, format!("{}{within}", listed.source)))
}

/// Each year's notice as shipped, in order.
const NEPAL_GAZETTE_FILES: [&str; 8] = [
    include_str!("../../data/np-holidays/2076.json"),
    include_str!("../../data/np-holidays/2077.json"),
    include_str!("../../data/np-holidays/2078.json"),
    include_str!("../../data/np-holidays/2079.json"),
    include_str!("../../data/np-holidays/2080.json"),
    include_str!("../../data/np-holidays/2081.json"),
    include_str!("../../data/np-holidays/2082.json"),
    include_str!("../../data/np-holidays/2083.json"),
];

/// Nepal's notices, read once.
static NEPAL_GAZETTE: LazyLock<Vec<Listing>> = LazyLock::new(|| {
    #[allow(
        clippy::expect_used,
        reason = "embedded data, read by a test on every build"
    )]
    NEPAL_GAZETTE_FILES
        .into_iter()
        .map(Listing::read)
        .collect::<Result<Vec<Listing>, String>>()
        .expect("the shipped notices read")
});

/// A day of the notice: month and day.
type On = (u8, u8);

/// The Bikram Sambat day `on` of `year`.
const fn bikram_sambat(year: i32, (month, day): On) -> CalendarDay {
    CalendarDay {
        calendar: Calendar::BikramSambat,
        year,
        month,
        day,
        era: None,
        leap: None,
    }
}

/// One year's notice as its file writes it.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct File {
    /// The Bikram Sambat year it lists.
    year: i32,
    /// Its citation.
    notice: String,
    /// Its clauses, in its order.
    holidays: Vec<Row>,
    /// The later notices amending it, in their order.
    #[serde(default)]
    amendments: Vec<Amendment>,
}

/// One holiday as a file writes it.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Row {
    /// Its clause; absent for a holiday a later notice added.
    #[serde(default)]
    clause: Option<String>,
    scope: HolidayScope,
    festivals: Vec<String>,
    #[serde(default)]
    regions: Vec<String>,
    #[serde(default)]
    communities: Vec<String>,
    #[serde(default)]
    first: Option<On>,
    #[serde(default)]
    last: Option<On>,
    /// For a holiday of several days, the day of each festival inside it
    /// the notice prints, or for one it does not print that the pack keys
    /// apart, the day the pack reckons in the notice's year: a day inside
    /// `first` to `last` is granted as the festival's.
    #[serde(default)]
    days_of: Vec<(String, On)>,
    ne: String,
    en: String,
}

/// A later notice's change to a year's list, keyed as a member is.
#[derive(Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum Amendment {
    /// A holiday the notice adds.
    Adds(Added),
    /// The day the notice fixes for a clause that gave none.
    Dates(Dated),
    /// A clause the notice cancels.
    Cancels(Cancelled),
}

/// A holiday a later notice adds.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Added {
    notice: String,
    holiday: Row,
}

/// The day a later notice fixes for a clause.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Dated {
    notice: String,
    clause: String,
    on: On,
}

/// A clause a later notice cancels.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Cancelled {
    notice: String,
    clause: String,
}

/// One year's holidays, its amendments applied.
#[derive(Debug)]
struct Listing {
    year: i32,
    listed: Vec<Listed>,
    cancelled: Vec<Listed>,
}

/// A holiday with the source it is listed by.
#[derive(Debug)]
struct Listed {
    row: Row,
    source: String,
}

impl Listing {
    /// A notice's file read and its amendments applied, its days and
    /// clauses checked.
    fn read(text: &str) -> Result<Listing, String> {
        let file: File = serde_json::from_str(text).map_err(|error| error.to_string())?;
        let year = file.year;
        let mut listed = file
            .holidays
            .into_iter()
            .map(|row| {
                let source = match &row.clause {
                    Some(clause) => format!("{}, clause {clause}", file.notice),
                    None => return Err(format!("VS {year}: `{}` has no clause", row.ne)),
                };
                Ok(Listed { row, source })
            })
            .collect::<Result<Vec<Listed>, String>>()?;
        let mut cancelled = Vec::new();
        for amendment in file.amendments {
            match amendment {
                Amendment::Adds(Added { notice, holiday }) => listed.push(Listed {
                    row: holiday,
                    source: notice,
                }),
                Amendment::Dates(Dated { notice, clause, on }) => {
                    let at = only(&listed, year, &clause)?;
                    let entry = listed
                        .get_mut(at)
                        .ok_or_else(|| format!("VS {year}: clause {clause} is not listed"))?;
                    if entry.row.first.replace(on).is_some() {
                        return Err(format!("VS {year}: clause {clause} already prints its day"));
                    }
                    entry.source = format!("{}; its day by {notice}", entry.source);
                }
                Amendment::Cancels(Cancelled { notice, clause }) => {
                    let at = only(&listed, year, &clause)?;
                    let mut entry = listed.remove(at);
                    entry.source = format!("{}; cancelled by {notice}", entry.source);
                    cancelled.push(entry);
                }
            }
        }
        let read = Listing {
            year,
            listed,
            cancelled,
        };
        read.check()?;
        Ok(read)
    }

    /// That every day it prints is a day of its year, in order, and every
    /// key and scope is written as a key.
    fn check(&self) -> Result<(), String> {
        let bs = BikramSambat::shipped();
        let year = self.year;
        let real = |(month, day): On| {
            bs.month_length(year, month)
                .is_ok_and(|length| (1..=length).contains(&day))
        };
        let key = |text: &str| {
            !text.is_empty()
                && text
                    .bytes()
                    .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
        };
        for listed in self.listed.iter().chain(&self.cancelled) {
            let row = &listed.row;
            let named = |held: bool, what: &str| {
                if held {
                    Ok(())
                } else {
                    Err(format!("VS {year}, {}: {what}", listed.source))
                }
            };
            let forward = match (row.first, row.last) {
                (_, None) => true,
                (Some(first), Some(last)) => first < last,
                (None, Some(_)) => false,
            };
            named(
                row.first.iter().chain(&row.last).all(|&on| real(on)),
                "a day the year does not have",
            )?;
            named(forward, "a span that does not run forward")?;
            named(
                row.days_of.iter().all(|&(_, on)| real(on)),
                "a festival's day the year does not have",
            )?;
            named(
                row.festivals
                    .iter()
                    .chain(&row.regions)
                    .chain(&row.communities)
                    .chain(row.days_of.iter().map(|(held, _)| held))
                    .all(|held| key(held)),
                "a key not written as one",
            )?;
            named(!row.ne.is_empty() && !row.en.is_empty(), "no words")?;
        }
        Ok(())
    }

    /// The holidays `listed` of this year, as a set answers them.
    fn holidays(&self, set: HolidaySet, listed: &[Listed]) -> Vec<Holiday> {
        listed
            .iter()
            .map(|listed| listed.row.holiday(set, self.year, &listed.source))
            .collect()
    }
}

/// The one holiday of `listed` printed under `clause`.
fn only(listed: &[Listed], year: i32, clause: &str) -> Result<usize, String> {
    let mut under = listed
        .iter()
        .enumerate()
        .filter(|(_, listed)| listed.row.clause.as_deref() == Some(clause))
        .map(|(at, _)| at);
    match (under.next(), under.next()) {
        (Some(at), None) => Ok(at),
        _ => Err(format!(
            "VS {year}: an amendment names clause {clause}, which is not one holiday"
        )),
    }
}

impl Row {
    /// Whether the holiday grants `key`'s day as its festival's: one of
    /// [`Row::days_of`] whose day lies between its first and last.
    fn grants_as_its_festival(&self, key: &str) -> bool {
        let (Some(first), Some(last)) = (self.first, self.last) else {
            return false;
        };
        self.days_of
            .iter()
            .any(|(held, day)| held == key && (first..=last).contains(day))
    }

    fn holiday(&self, set: HolidaySet, year: i32, source: &str) -> Holiday {
        Holiday {
            set: set.key().to_owned(),
            year,
            notice: HolidayNotice {
                ne: self.ne.clone(),
                en: self.en.clone(),
            },
            festivals: self.festivals.clone(),
            scope: self.scope,
            regions: self.regions.clone(),
            communities: self.communities.clone(),
            first: self.first.map(|on| bikram_sambat(year, on)),
            last: self.last.map(|on| bikram_sambat(year, on)),
            source: source.to_owned(),
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::indexing_slicing,
        clippy::unwrap_used,
        reason = "a test indexes what it built and fails by panicking"
    )]

    use super::*;

    #[test]
    fn every_notice_reads_and_checks() {
        let mut years = Vec::new();
        for text in NEPAL_GAZETTE_FILES {
            let read = Listing::read(text);
            assert!(read.is_ok(), "{read:?}");
            years.push(read.unwrap().year);
        }
        assert_eq!(years, (2076..=2083).collect::<Vec<_>>());
    }

    #[test]
    fn a_bad_file_is_refused_by_name() {
        let refused = |text: &str| Listing::read(text).unwrap_err();
        let file = |holiday: &str, amendments: &str| {
            format!(
                r#"{{"year": 2080, "notice": "N", "holidays": [{holiday}], "amendments": [{amendments}]}}"#
            )
        };
        let row = r#"{"clause": "1", "scope": "NATIONAL", "festivals": [], "ne": "न", "en": "N"}"#;
        assert!(
            refused(&file(
                r#"{"clause": "1", "scope": "NATIONAL", "festivals": [], "first": [2, 33], "ne": "न", "en": "N"}"#,
                ""
            ))
            .contains("a day the year does not have")
        );
        assert!(
            refused(&file(
                r#"{"clause": "1", "scope": "NATIONAL", "festivals": ["Not a key"], "ne": "न", "en": "N"}"#,
                ""
            ))
            .contains("a key not written as one")
        );
        assert!(
            refused(&file(
                r#"{"scope": "NATIONAL", "festivals": [], "ne": "न", "en": "N"}"#,
                ""
            ))
            .contains("has no clause")
        );
        assert!(
            refused(&file(row, r#"{"CANCELS": {"notice": "M", "clause": "2"}}"#))
                .contains("not one holiday")
        );
        assert!(
            refused(&file(
                r#"{"clause": "1", "scope": "NATIONAL", "festivals": [], "first": [1, 1], "ne": "न", "en": "N"}"#,
                r#"{"DATES": {"notice": "M", "clause": "1", "on": [1, 2]}}"#
            ))
            .contains("already prints its day")
        );
        assert!(refused(&file(row, r#"{"MOVES": {}}"#)).contains("unknown variant"));
    }

    #[test]
    fn each_year_lists_its_notice_s_holidays() {
        // Each year's clauses, a span of days one holiday, with the
        // holidays later notices added and without the one cancelled.
        let counts: Vec<(i32, usize)> = HolidaySet::NepalGazette
            .years()
            .into_iter()
            .map(|year| (year, HolidaySet::NepalGazette.holidays_of(year).len()))
            .collect();
        assert_eq!(
            counts,
            [
                (2076, 44),
                (2077, 40),
                (2078, 41),
                (2079, 44),
                (2080, 43),
                (2081, 45),
                (2082, 44),
                (2083, 45),
            ]
        );
        assert_eq!(
            HolidaySet::NepalGazette.holidays().len(),
            counts.iter().map(|(_, count)| count).sum::<usize>()
        );
    }

    #[test]
    fn every_year_s_dashain_holiday_holds_vijaya_dashami() {
        let spans: Vec<(i32, On, On)> = HolidaySet::NepalGazette
            .holidays()
            .into_iter()
            .filter(|holiday| holiday.festivals.contains(&"VIJAYA_DASHAMI".to_owned()))
            .map(|holiday| {
                let (first, last) = (holiday.first.unwrap(), holiday.last.unwrap());
                (
                    holiday.year,
                    (first.month, first.day),
                    (last.month, last.day),
                )
            })
            .collect();
        assert_eq!(
            spans,
            [
                (2076, (6, 18), (6, 22)),
                (2077, (7, 7), (7, 11)),
                (2078, (6, 26), (6, 30)),
                (2079, (6, 16), (6, 21)),
                (2080, (7, 4), (7, 9)),
                (2081, (6, 24), (6, 28)),
                (2082, (6, 13), (6, 18)),
                (2083, (6, 31), (7, 6)),
            ]
        );
    }

    #[test]
    fn a_later_notice_adds_dates_and_cancels() {
        let set = HolidaySet::NepalGazette;
        // VS 2078: Gyalpo Lhosar, the Sherpa community's in the notice,
        // given to the whole country by a later one.
        let (scope, source) = nepal_gazette_scope("GYALPO_LHOSAR", Some(2078)).unwrap();
        assert_eq!(scope, HolidayScope::National);
        assert!(source.contains("Falgun 18, 2078"), "{source}");
        assert!(set.holidays_of(2078).iter().any(|holiday| {
            holiday.festivals == ["GYALPO_LHOSAR"] && holiday.scope == HolidayScope::Community
        }));
        // VS 2080: Janayuddha Day's holiday cancelled, and Eid's day fixed.
        assert!(
            set.holidays_of(2080)
                .iter()
                .all(|holiday| holiday.notice.ne != "जनयुद्ध दिवस")
        );
        let eid = set
            .holidays_of(2080)
            .into_iter()
            .find(|holiday| holiday.festivals == ["EID_AL_FITR"])
            .unwrap();
        assert_eq!(eid.first.map(|day| (day.month, day.day)), Some((12, 29)));
        assert!(
            eid.source.contains("clause 2.1 (त); its day by"),
            "{}",
            eid.source
        );
        // VS 2079: a day of mourning, a holiday naming no festival.
        assert!(set.holidays_of(2079).iter().any(|holiday| {
            holiday.festivals.is_empty()
                && holiday.first.as_ref().map(|day| (day.month, day.day)) == Some((7, 1))
        }));
    }

    #[test]
    fn a_scope_beyond_the_country_says_whom_it_limits_to() {
        for holiday in HolidaySet::NepalGazette.holidays() {
            let limited = !holiday.regions.is_empty() || !holiday.communities.is_empty();
            let names = |held: bool| assert!(held, "{}", holiday.source);
            if matches!(
                holiday.scope,
                HolidayScope::Community | HolidayScope::Religion
            ) {
                names(!holiday.communities.is_empty());
            } else if holiday.scope == HolidayScope::Region {
                // Sirua Pavani's districts and Dhawathambu Chheuchik's have
                // no key; their notices name them.
                names(
                    !holiday.regions.is_empty()
                        || holiday.notice.ne.starts_with("सिरुवा पावनी")
                        || holiday.notice.ne.starts_with("धवथाम्बु छेउचिक"),
                );
            } else {
                names(!limited);
            }
        }
    }

    #[test]
    fn a_festival_s_day_inside_its_holiday_is_granted_and_one_outside_is_not() {
        // Kukur Tihar, Kartik 22, inside Tihar's Kartik 22 to 26.
        let (scope, source) = nepal_gazette_scope("KUKUR_TIHAR", Some(2083)).unwrap();
        assert_eq!(scope, HolidayScope::National);
        assert!(source.ends_with("clause 2.1 (छ), a day of the festival it grants"));
        // Kag Tihar, Kartik 21, the day before it.
        assert_eq!(nepal_gazette_scope("KAG_TIHAR", Some(2083)), None);
        // Mha puja is no day of the clause's festival in the pack's grouping.
        assert_eq!(nepal_gazette_scope("MHA_PUJA", Some(2083)), None);
        // VS 2080's notice names it among Tihar's days.
        assert_eq!(
            nepal_gazette_scope("MHA_PUJA", Some(2080)).map(|(scope, _)| scope),
            Some(HolidayScope::National)
        );
        // The latest notice is VS 2083's.
        assert_eq!(
            nepal_gazette_scope("KUKUR_TIHAR", None),
            nepal_gazette_scope("KUKUR_TIHAR", Some(2083))
        );
    }

    #[test]
    fn the_holidays_naming_more_than_one_festival_are_the_notices_five() {
        let several: Vec<Vec<String>> = HolidaySet::NepalGazette
            .holidays_of(2083)
            .into_iter()
            .filter(|holiday| holiday.festivals.len() > 1)
            .map(|holiday| holiday.festivals)
            .collect();
        assert_eq!(
            several,
            [
                vec!["CHANDI_PURNIMA", "BUDDHA_JAYANTI", "UBHAULI"],
                vec!["VIJAYA_DASHAMI", "PHULPATI"],
                vec!["LAKSHMI_PUJA", "YAMA_DWITIYA"],
                vec!["DHANYA_PURNIMA", "UDHAULI", "YOMARI_PUNHI", "JYAPU_DIWAS"],
                vec!["MAGHI_THARU", "MAKARA_SANKRANTI"],
            ]
        );
    }
}
