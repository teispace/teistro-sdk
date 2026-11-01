//! `intl.dates`: Bikram Sambat and Gregorian dates found in free text
//! (`docs/03-design/lexicon.md` §2, SR-61).
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]

use teistro_core::Status;
use teistro_core::catalogue::Calendar;
use teistro_intl::dates::{DateOrder, DatesRequest, FoundDates};
use teistro_intl::source::Tree;
use teistro_intl::{Intl, sdk_root};

fn intl() -> Intl {
    Intl::from_tree(&Tree::load(&sdk_root()).expect("the sources")).expect("the engine")
}

fn dates(intl: &Intl, text: &str) -> FoundDates {
    intl.dates(&DatesRequest {
        text: text.to_owned(),
        calendars: None,
    })
    .unwrap()
}

/// Each span's text and its readings as (calendar, y, m, d, order).
type Read = (String, Vec<(Calendar, i32, u8, u8, DateOrder)>);

fn read(answer: &FoundDates) -> Vec<Read> {
    answer
        .dates
        .iter()
        .map(|found| {
            (
                found.text.clone(),
                found
                    .readings
                    .iter()
                    .map(|r| {
                        (
                            r.date.calendar,
                            r.date.year,
                            r.date.month,
                            r.date.day,
                            r.order,
                        )
                    })
                    .collect(),
            )
        })
        .collect()
}

const BS: Calendar = Calendar::BikramSambat;
const AD: Calendar = Calendar::Gregorian;

#[test]
fn a_bikram_sambat_day_in_any_script_and_spelling() {
    let intl = intl();
    for text in [
        "२०८२ कात्तिक १५ गते",
        "2082 Kartik 15",
        "15 Kattik 2082",
        "Kartik 15, 2082",
        "२०८२ साल कार्तिक १५ गते",
        "kartik 15th 2082",
        "वि.सं. २०८२ कार्तिक १५",
    ] {
        let answer = dates(&intl, text);
        assert_eq!(answer.dates.len(), 1, "{text}");
        let readings = &answer.dates[0].readings;
        assert_eq!(readings.len(), 1, "{text}: {readings:?}");
        let r = &readings[0];
        assert_eq!(
            (r.date.calendar, r.date.year, r.date.month, r.date.day),
            (BS, 2082, 7, 15),
            "{text}"
        );
        let g = r.gregorian.as_ref().unwrap();
        assert_eq!((g.year, g.month, g.day), (2025, 11, 1), "{text}");
        assert_eq!(answer.dates[0].text, text, "the whole span, its marker too");
    }
}

#[test]
fn every_month_is_named_in_every_spelling_listed() {
    let intl = intl();
    for (month, spellings) in [
        (1, &["Baishakh", "Baisakh", "वैशाख", "बैशाख"][..]),
        (2, &["Jestha", "Jeth", "जेठ", "ज्येष्ठ"][..]),
        (3, &["Asar", "Ashadh", "असार", "आषाढ़"][..]),
        (4, &["Shrawan", "Saun", "साउन", "श्रावण"][..]),
        (5, &["Bhadra", "Bhadau", "भदौ", "भाद्रपद"][..]),
        (6, &["Ashwin", "Asoj", "असोज", "आश्विन"][..]),
        (7, &["Kartik", "Kattik", "कात्तिक", "कार्तिक"][..]),
        (8, &["Mangsir", "Mansir", "मंसिर", "मार्गशीर्ष"][..]),
        (9, &["Poush", "Push", "पुष", "पौष"][..]),
        (10, &["Magh", "माघ"][..]),
        (11, &["Falgun", "Phalgun", "फागुन", "फाल्गुन"][..]),
        (12, &["Chaitra", "Chait", "चैत", "चैत्र"][..]),
    ] {
        for spelt in spellings {
            let text = format!("2081 {spelt} 1");
            assert_eq!(
                read(&dates(&intl, &text)),
                [(text.clone(), vec![(BS, 2081, month, 1, DateOrder::Ymd)])],
                "{spelt}"
            );
        }
    }
}

#[test]
fn a_gregorian_day_in_english_nepali_and_hindi() {
    let intl = intl();
    for text in [
        "31 October 2026",
        "October 31st, 2026",
        "2026 October 31",
        "31 Oct 2026",
        "३१ अक्टोबर २०२६",
        "31 अक्टूबर 2026",
        "सन् २०२६ अक्टोबर ३१",
    ] {
        assert_eq!(read(&dates(&intl, text)).len(), 1, "{text}");
        let r = &dates(&intl, text).dates[0].readings;
        assert_eq!(r.len(), 1, "{text}");
        assert_eq!(
            (
                r[0].date.calendar,
                r[0].date.year,
                r[0].date.month,
                r[0].date.day
            ),
            (AD, 2026, 10, 31),
            "{text}"
        );
        assert!(r[0].gregorian.is_none());
    }
}

#[test]
fn figures_alone_read_in_both_calendars() {
    let intl = intl();
    assert_eq!(
        read(&dates(&intl, "on 2082/07/15 we met")),
        [(
            String::from("2082/07/15"),
            vec![
                (BS, 2082, 7, 15, DateOrder::Ymd),
                (AD, 2082, 7, 15, DateOrder::Ymd)
            ]
        )]
    );
    // गते settles the Bikram Sambat; a marker settles either.
    assert_eq!(
        read(&dates(&intl, "२०८२/०७/१५ गते")),
        [(
            String::from("२०८२/०७/१५ गते"),
            vec![(BS, 2082, 7, 15, DateOrder::Ymd)]
        )]
    );
    assert_eq!(
        read(&dates(&intl, "2026-10-31 AD")),
        [(
            String::from("2026-10-31 AD"),
            vec![(AD, 2026, 10, 31, DateOrder::Ymd)]
        )]
    );
    // Day 32 is a Bikram Sambat day in some months (Shrawan 2081, not its
    // Asar) and never a Gregorian.
    assert_eq!(
        read(&dates(&intl, "2081.04.32")),
        [(
            String::from("2081.04.32"),
            vec![(BS, 2081, 4, 32, DateOrder::Ymd)]
        )]
    );
    assert_eq!(read(&dates(&intl, "2081.03.32")).len(), 0);
}

#[test]
fn a_year_last_reads_day_first_and_month_first() {
    let intl = intl();
    let asked =
        DatesRequest::from_json(r#"{"text": "05/06/2025", "calendars": ["calendar.GREGORIAN"]}"#)
            .unwrap();
    assert_eq!(
        read(&intl.dates(&asked).unwrap()),
        [(
            String::from("05/06/2025"),
            vec![
                (AD, 2025, 6, 5, DateOrder::Dmy),
                (AD, 2025, 5, 6, DateOrder::Mdy)
            ]
        )]
    );
    let only =
        DatesRequest::from_json(r#"{"text": "31/10/2026", "calendars": ["calendar.GREGORIAN"]}"#)
            .unwrap();
    assert_eq!(
        read(&intl.dates(&only).unwrap()),
        [(
            String::from("31/10/2026"),
            vec![(AD, 2026, 10, 31, DateOrder::Dmy)]
        )]
    );
}

#[test]
fn what_is_not_read() {
    let intl = intl();
    for text in [
        // No year, a two-digit year, a lunar date, a day no calendar has,
        // separators that differ, and no date at all.
        "Kartik 15",
        "15/10/26",
        "२०८२ कार्तिक कृष्ण १५",
        "कार्तिक शुक्ल १, २०८२",
        "2026 February 30",
        "2026/10-31",
        "the man went out",
        "",
    ] {
        assert!(
            dates(&intl, text).dates.is_empty(),
            "{text}: {:?}",
            read(&dates(&intl, text))
        );
    }
}

#[test]
fn several_dates_in_one_text_in_order() {
    let intl = intl();
    let answer = dates(
        &intl,
        "Bibaha 2082 Mangsir 5 (21 November 2025) ma ho, ra bratabandha 2082/11/02 BS ma.",
    );
    let texts: Vec<&str> = answer.dates.iter().map(|d| d.text.as_str()).collect();
    assert_eq!(
        texts,
        ["2082 Mangsir 5", "21 November 2025", "2082/11/02 BS"]
    );
    let starts: Vec<usize> = answer.dates.iter().map(|d| d.start).collect();
    assert_eq!(starts, [7, 23, 63]);
}

#[test]
fn another_calendar_is_refused_by_name() {
    let intl = intl();
    let asked =
        DatesRequest::from_json(r#"{"text": "x", "calendars": ["calendar.JULIAN"]}"#).unwrap();
    let refused = intl.dates(&asked).unwrap_err();
    assert_eq!(refused.status, Status::Unsupported);
    assert_eq!(refused.field(), Some("dates.calendars"));
}
