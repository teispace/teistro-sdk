//! A month view's title and a run of months (SR-86):
//! `sdk.calendar.<CALENDAR>.monthYear` and `monthYearRange`, through
//! `intl.date`'s `MONTH_YEAR` style and `until`
//! (`docs/03-design/intl-engine-and-packs.md`, "A month and its year").
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]

use teistro_core::Status;
use teistro_intl::civil::{CivilPartKind, CivilText, DateTextRequest};
use teistro_intl::source::{Entry, Tree};
use teistro_intl::{Intl, Value, params, sdk_root};

const LOCALES: [&str; 5] = ["en-Latn", "hi-Deva-IN", "ne-Deva-NP", "sa-Deva", "sa-Latn"];

/// Every calendar whose base locale names its months.
const NAMED: [&str; 9] = [
    "GREGORIAN",
    "JULIAN",
    "MIXED",
    "BIKRAM_SAMBAT",
    "INDIAN_NATIONAL",
    "HIJRI_TABULAR",
    "TIBETAN_PHUGPA",
    "BANGLADESHI",
    "ASSAM_SOLAR",
];

/// A day in a calendar `intl.date` holds, and a day some months on that
/// falls in the next year.
const DAYS: [(&str, i32, u8, u8, i32, u8); 8] = [
    ("GREGORIAN", 2026, 12, 31, 2027, 1),
    ("JULIAN", 2026, 12, 18, 2027, 1),
    ("MIXED", 2026, 12, 31, 2027, 1),
    ("BIKRAM_SAMBAT", 2083, 12, 14, 2084, 1),
    ("INDIAN_NATIONAL", 1948, 12, 9, 1949, 1),
    ("HIJRI_TABULAR", 1448, 12, 20, 1449, 1),
    ("TIBETAN_PHUGPA", 2153, 12, 20, 2154, 1),
    ("BANGLADESHI", 1433, 12, 15, 1434, 1),
];

fn intl() -> Intl {
    Intl::from_tree(&Tree::load(&sdk_root()).expect("the sources")).expect("the engine")
}

fn day(calendar: &str, year: i32, month: u8, day: u8) -> String {
    format!(
        r#"{{"calendar": "calendar.{calendar}", "year": {year}, "month": {month}, "day": {day}}}"#
    )
}

fn asked(at: &str, until: Option<&str>, locale: &str) -> DateTextRequest {
    let until = until.map_or_else(String::new, |until| format!(r#", "until": {until}"#));
    DateTextRequest::from_json(&format!(
        r#"{{"at": {at}{until}, "style": "MONTH_YEAR", "locale": "{locale}"}}"#
    ))
    .unwrap()
}

fn title(intl: &Intl, at: &str, until: Option<&str>, locale: &str) -> CivilText {
    intl.date(&asked(at, until, locale)).unwrap()
}

fn text(intl: &Intl, at: &str, until: Option<&str>, locale: &str) -> String {
    title(intl, at, until, locale).text
}

#[test]
fn the_titles_read_as_cldr_orders_them() {
    let intl = intl();
    let october = day("GREGORIAN", 2026, 10, 1);
    let november = day("GREGORIAN", 2026, 11, 30);
    let december = day("GREGORIAN", 2026, 12, 5);
    let january = day("GREGORIAN", 2027, 1, 5);
    // CLDR 48 `en`: yMMMM `MMMM y`, its interval `MMMM – MMMM y` and
    // `MMMM y – MMMM y`, with thin spaces about the dash.
    assert_eq!(text(&intl, &october, None, "en-Latn"), "October 2026");
    assert_eq!(
        text(&intl, &october, Some(&november), "en-Latn"),
        "October\u{2009}–\u{2009}November 2026"
    );
    assert_eq!(
        text(&intl, &december, Some(&january), "en-Latn"),
        "December 2026\u{2009}–\u{2009}January 2027"
    );
    // `ne`: the year first, `y MMMM`, `y MMMM–MMMM`, `y MMMM – y MMMM`.
    assert_eq!(text(&intl, &october, None, "ne-Deva-NP"), "२०२६ अक्टोबर");
    assert_eq!(
        text(&intl, &october, Some(&november), "ne-Deva-NP"),
        "२०२६ अक्टोबर–नोभेम्बर"
    );
    assert_eq!(
        text(&intl, &december, Some(&january), "ne-Deva-NP"),
        "२०२६ डिसेम्बर\u{2009}–\u{2009}२०२७ जनवरी"
    );
    // `hi`: `MMMM y`, `MMMM – MMMM y`, `MMMM y – MMMM y`.
    assert_eq!(text(&intl, &october, None, "hi-Deva-IN"), "अक्टूबर २०२६");
    assert_eq!(
        text(&intl, &december, Some(&january), "hi-Deva-IN"),
        "दिसंबर २०२६ – जनवरी २०२७"
    );
    // The Bikram Sambat writes its era where the locale's long date does.
    let kartik = day("BIKRAM_SAMBAT", 2083, 7, 15);
    let chaitra = day("BIKRAM_SAMBAT", 2083, 12, 1);
    let baishakh = day("BIKRAM_SAMBAT", 2084, 1, 1);
    assert_eq!(text(&intl, &kartik, None, "ne-Deva-NP"), "२०८३ कात्तिक");
    assert_eq!(text(&intl, &kartik, None, "en-Latn"), "Kartik 2083 BS");
    assert_eq!(
        text(&intl, &kartik, None, "hi-Deva-IN"),
        "कार्तिक वि.सं. २०८३"
    );
    assert_eq!(
        text(&intl, &chaitra, Some(&baishakh), "ne-Deva-NP"),
        "२०८३ चैत\u{2009}–\u{2009}२०८४ बैशाख"
    );
    assert_eq!(
        text(&intl, &chaitra, Some(&baishakh), "en-Latn"),
        "Chaitra 2083\u{2009}–\u{2009}Baishakh 2084 BS"
    );
    assert_eq!(
        text(&intl, &chaitra, Some(&baishakh), "hi-Deva-IN"),
        "चैत्र २०८३ – वैशाख वि.सं. २०८४"
    );
}

#[test]
fn a_run_within_one_month_is_that_month() {
    let intl = intl();
    let first = day("GREGORIAN", 2026, 10, 1);
    let last = day("GREGORIAN", 2026, 10, 31);
    assert_eq!(
        title(&intl, &first, Some(&last), "en-Latn"),
        title(&intl, &first, None, "en-Latn")
    );
}

#[test]
fn every_calendar_and_locale_writes_a_title_and_a_run() {
    let intl = intl();
    for locale in LOCALES {
        for (calendar, year, month, on, next_year, next_month) in DAYS {
            let at = day(calendar, year, month, on);
            let same_year = day(calendar, year, month - 1, 1);
            let next = day(calendar, next_year, next_month, 1);
            for (from, until, years) in [
                (at.clone(), None, 1),
                (same_year, Some(at.clone()), 1),
                (at.clone(), Some(next), 2),
            ] {
                let answer = title(&intl, &from, until.as_deref(), locale);
                let what = format!("{locale} {calendar} {until:?}: {}", answer.text);
                let joined: String = answer.parts.iter().map(|p| p.text.as_str()).collect();
                assert_eq!(joined, answer.text, "{what}");
                assert!(answer.warnings.is_empty(), "{what}: {:?}", answer.warnings);
                assert!(!answer.is_fallback, "{what} fell back");
                assert!(!answer.text.contains('{'), "{what}");
                let count = |kind| answer.parts.iter().filter(|p| p.kind == kind).count();
                assert_eq!(count(CivilPartKind::Year), years, "{what}");
                assert_eq!(
                    count(CivilPartKind::Month),
                    if until.is_some() { 2 } else { 1 },
                    "{what}"
                );
                assert!(!kinds_contain_day(&answer), "{what}");
                // Nepali writes the Hijri months and era in Latin letters.
                let latin = calendar == "HIJRI_TABULAR" && locale == "ne-Deva-NP";
                if locale.contains("-Deva") && !latin {
                    assert!(
                        !answer.text.chars().any(|c| c.is_ascii_alphanumeric()),
                        "{what}"
                    );
                }
            }
        }
    }
}

fn kinds_contain_day(answer: &CivilText) -> bool {
    answer.parts.iter().any(|p| p.kind == CivilPartKind::Day)
}

/// The messages themselves, rendered with the parameters the engine
/// supplies, for every calendar that names its months: the regional solar
/// calendars `intl.date` cannot check a day of are held here.
#[test]
fn every_named_calendar_carries_both_patterns_in_every_locale() {
    let intl = intl();
    for locale in LOCALES {
        for calendar in NAMED {
            let single = params([
                ("year", Value::Int(1433)),
                ("month", Value::Int(7)),
                ("monthName", Value::Str(String::from("M7"))),
                ("leap", Value::Str(String::new())),
            ]);
            let rendered = intl.render_in(
                locale,
                &format!("sdk.calendar.{calendar}.monthYear"),
                &single,
            );
            assert!(!rendered.is_fallback, "{locale} {calendar}");
            assert!(
                rendered.warnings.is_empty(),
                "{locale} {calendar}: {:?}",
                rendered.warnings
            );
            assert!(
                rendered.text.contains("M7"),
                "{locale} {calendar}: {}",
                rendered.text
            );
            for (greatest, until_year) in [("MONTH", 1433), ("YEAR", 1434)] {
                let mut run = single.clone();
                run.insert(String::from("untilYear"), Value::Int(until_year));
                run.insert(String::from("untilMonth"), Value::Int(8));
                run.insert(
                    String::from("untilMonthName"),
                    Value::Str(String::from("M8")),
                );
                run.insert(String::from("greatest"), Value::Str(String::from(greatest)));
                let key = format!("sdk.calendar.{calendar}.monthYearRange");
                let rendered = intl.render_in(locale, &key, &run);
                let what = format!("{locale} {key} {greatest}: {}", rendered.text);
                assert!(!rendered.is_fallback, "{what}");
                assert!(
                    rendered.warnings.is_empty(),
                    "{what}: {:?}",
                    rendered.warnings
                );
                assert!(
                    rendered.text.contains("M7") && rendered.text.contains("M8"),
                    "{what}"
                );
                assert!(rendered.text.contains('–'), "{what}");
            }
        }
    }
}

#[test]
fn a_run_is_refused_where_it_cannot_be_written() {
    let intl = intl();
    let refused = |json: &str| {
        intl.date(&DateTextRequest::from_json(json).unwrap())
            .unwrap_err()
    };
    let october = day("GREGORIAN", 2026, 10, 1);
    let september = day("GREGORIAN", 2026, 9, 1);
    let kartik = day("BIKRAM_SAMBAT", 2083, 7, 1);
    let long = refused(&format!(
        r#"{{"at": {october}, "until": {october}, "style": "LONG"}}"#
    ));
    assert_eq!(
        (long.status, long.field()),
        (Status::InvalidArg, Some("date.until"))
    );
    let before = refused(&format!(
        r#"{{"at": {october}, "until": {september}, "style": "MONTH_YEAR"}}"#
    ));
    assert_eq!(
        (before.status, before.field()),
        (Status::InvalidArg, Some("date.until"))
    );
    let across = refused(&format!(
        r#"{{"at": {october}, "until": {kartik}, "style": "MONTH_YEAR"}}"#
    ));
    assert_eq!(
        (across.status, across.field()),
        (Status::InvalidArg, Some("date.until.calendar"))
    );
    let no_day = refused(&format!(
        r#"{{"at": {october}, "until": {}, "style": "MONTH_YEAR"}}"#,
        day("GREGORIAN", 2026, 11, 31)
    ));
    assert_eq!(no_day.status, Status::InvalidArg);
    assert!(
        no_day.field().is_some_and(|f| f.starts_with("date.until")),
        "{no_day:?}"
    );
    // An ISO week date has no month to title.
    let week = refused(&format!(
        r#"{{"at": {}, "style": "MONTH_YEAR"}}"#,
        day("ISO_WEEK", 2026, 44, 6)
    ));
    assert_eq!(week.status, Status::Unsupported);
}

#[test]
fn a_title_in_other_digits() {
    let intl = intl();
    let request = DateTextRequest::from_json(&format!(
        r#"{{"at": {}, "style": "MONTH_YEAR", "locale": "ne-Deva-NP", "digits": "LATN"}}"#,
        day("BIKRAM_SAMBAT", 2083, 7, 15)
    ))
    .unwrap();
    assert_eq!(intl.date(&request).unwrap().text, "2083 कात्तिक");
}

/// SR-85: every tithi, nakshatra and lunar month carries `abbrev` in
/// every shipped locale, no wider than its name, and the abbreviations of
/// one kind name one member each.
#[test]
fn an_abbreviation_is_never_wider_than_the_name() {
    let tree = Tree::load(&sdk_root()).unwrap();
    let width = |text: &str| text.chars().count();
    for tag in LOCALES {
        let entities = &tree.locales[tag].namespaces["sdk.entity"];
        for kind in ["tithi", "nakshatra", "masa"] {
            let mut abbrevs = Vec::new();
            for (key, entry) in &entities.entries {
                let Entry::Entity(entity) = entry else {
                    continue;
                };
                if key.split('.').next() != Some(kind) {
                    continue;
                }
                let abbrev = entity.form("abbrev");
                assert!(abbrev.is_some(), "{tag} {key} has no `abbrev`");
                let abbrev = abbrev.unwrap();
                assert!(!abbrev.is_empty(), "{tag} {key}");
                assert!(
                    width(abbrev) <= width(entity.name()),
                    "{tag} {key}: `{abbrev}` is wider than `{}`",
                    entity.name()
                );
                abbrevs.push(abbrev.to_owned());
            }
            let count = abbrevs.len();
            assert_eq!(
                count,
                if kind == "tithi" {
                    30
                } else if kind == "masa" {
                    12
                } else {
                    27
                },
                "{tag} {kind}"
            );
            // A tithi's name does not say its fortnight in Devanagari, so
            // the two fortnights share the fifteen; English writes it.
            abbrevs.sort();
            abbrevs.dedup();
            let distinct = if kind == "tithi" && tag != "en-Latn" {
                16
            } else {
                count
            };
            assert_eq!(abbrevs.len(), distinct, "{tag} {kind}: {abbrevs:?}");
        }
    }
}

#[test]
fn an_abbreviation_renders_where_a_message_asks_for_it() {
    let mut intl = intl();
    let render = |intl: &Intl, key: &str| {
        let rendered = intl
            .render_source(
                "{$m :entity form=abbrev}",
                &params([("m", Value::Entity(key.to_owned()))]),
            )
            .unwrap();
        assert!(rendered.warnings.is_empty(), "{:?}", rendered.warnings);
        rendered.text
    };
    assert_eq!(render(&intl, "tithi.SHUKLA_TRAYODASHI"), "Su. 13");
    assert_eq!(render(&intl, "tithi.AMAVASYA"), "Ba. 15");
    assert_eq!(render(&intl, "nakshatra.PURVA_PHALGUNI"), "P.Phal.");
    intl.set_locale("ne-Deva-NP").unwrap();
    assert_eq!(render(&intl, "masa.KARTIKA"), "का");
    assert_eq!(render(&intl, "tithi.KRISHNA_CHATURDASHI"), "चतुर्द.");
    assert_eq!(render(&intl, "nakshatra.UTTARA_BHADRAPADA"), "उ.भा.");
    intl.set_locale("hi-Deva-IN").unwrap();
    assert_eq!(render(&intl, "masa.MARGASHIRSHA"), "मार्ग.");
}
