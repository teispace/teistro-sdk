//! `calendar.in_era` (SR-96): a date viewed in a named era, the Kirat
//! Yele Sambat of a Bikram Sambat date under the context's epoch, and the
//! refusals by field (`calendars/bikram-sambat.md` §Kirat Yele Sambat).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::{Calendar, Era};
use teistro::{CalendarDate, Context, Status};

fn bs_of(sdk: &Context, year: i32, month: u8, day: u8) -> CalendarDate {
    sdk.calendar()
        .convert(
            &CalendarDate::defined(Calendar::Gregorian, year, month, day),
            Calendar::BikramSambat,
        )
        .unwrap()
}

fn yele_year(sdk: &Context, date: &CalendarDate) -> i32 {
    let viewed = sdk.calendar().in_era(date, Era::KiratYele).unwrap();
    let era = viewed.era.unwrap();
    assert_eq!(era.era, Era::KiratYele);
    era.year
}

/// *Kantipur*, 15 January 2026: Kirati Sambat 5086 began on Magh 1, 2082;
/// the day before closed 5085; Regmi's epoch from Yalambar counts 1281
/// fewer.
#[test]
fn magh_first_2082_opened_kirati_sambat_5086() {
    let sdk = Context::builder().build().unwrap();
    let magh = bs_of(&sdk, 2026, 1, 15);
    assert_eq!((magh.year, magh.month, magh.day), (2082, 10, 1));
    assert_eq!(yele_year(&sdk, &magh), 5086);
    let viewed = sdk.calendar().in_era(&magh, Era::KiratYele).unwrap();
    assert_eq!(
        (viewed.calendar, viewed.year, viewed.month, viewed.day),
        (Calendar::BikramSambat, 2082, 10, 1)
    );
    assert_eq!(viewed.resolution, magh.resolution);
    assert_eq!(yele_year(&sdk, &bs_of(&sdk, 2026, 1, 14)), 5085);

    let regmi = Context::builder()
        .settings_json(r#"{"calendars": {"kirat_yele": "YALAMBAR"}}"#)
        .build()
        .unwrap();
    assert_eq!(yele_year(&regmi, &magh), 3805);
    assert_ne!(regmi.settings_hash(), sdk.settings_hash());
    // Indigenous Voice, 2021: 5076 and 3795 from Magh 1, 2072 (January 2016).
    let magh_2072 = CalendarDate::defined(Calendar::BikramSambat, 2072, 10, 1);
    assert_eq!(yele_year(&sdk, &magh_2072), 5076);
    assert_eq!(yele_year(&regmi, &magh_2072), 3795);
}

/// An era the calendar attaches is its own reading; one it gives another
/// day, or never, is refused on `era` naming those it gives.
#[test]
fn a_calendars_own_era_is_its_reading_and_any_other_is_refused() {
    let sdk = Context::builder().build().unwrap();
    let magh = bs_of(&sdk, 2026, 1, 15);
    let vikrama = sdk.calendar().in_era(&magh, Era::Vikrama).unwrap();
    assert_eq!(vikrama, magh);

    let hijri = sdk.calendar().in_era(&magh, Era::Hijri).unwrap_err();
    assert_eq!(
        (hijri.status, hijri.field()),
        (Status::Unsupported, Some("era"))
    );
    let hint = hijri.hint().unwrap_or_default();
    assert!(
        hint.contains("era.VIKRAMA") && hint.contains("era.KIRAT_YELE"),
        "{hint}"
    );

    // Year 0 is 1 BCE: it reads in BEFORE_COMMON_ERA, not COMMON_ERA.
    let bce = CalendarDate::defined(Calendar::Gregorian, 0, 1, 1);
    let refused = sdk.calendar().in_era(&bce, Era::CommonEra).unwrap_err();
    assert_eq!(refused.field(), Some("era"));
    let before = sdk.calendar().in_era(&bce, Era::BeforeCommonEra).unwrap();
    assert_eq!(before.era.map(|era| era.year), Some(1));

    // A Gregorian date has no Yele year, and the Tibetan calendar no era.
    let gregorian = CalendarDate::defined(Calendar::Gregorian, 2026, 1, 15);
    assert_eq!(
        sdk.calendar()
            .in_era(&gregorian, Era::KiratYele)
            .unwrap_err()
            .field(),
        Some("era")
    );
    let losar = sdk
        .calendar()
        .convert(&gregorian, Calendar::TibetanPhugpa)
        .unwrap();
    let none = sdk.calendar().in_era(&losar, Era::Vikrama).unwrap_err();
    assert!(
        none.hint().unwrap_or_default().contains("no era"),
        "{none:?}"
    );

    // A day the calendar does not have is refused as `fixed_of` refuses it.
    let missing = CalendarDate::defined(Calendar::BikramSambat, 2082, 10, 31);
    assert_eq!(
        sdk.calendar()
            .in_era(&missing, Era::KiratYele)
            .unwrap_err()
            .status,
        Status::InvalidArg
    );
}

/// The record the MCP server serves answers the same date and its weekday.
#[test]
fn the_record_answers_the_era_and_the_weekday() {
    let sdk = Context::builder().build().unwrap();
    let record = teistro::records::record("calendar.in_era").expect("every build carries it");
    assert_eq!(record.boundary, "ts_calendar_in_era");
    let answered = record
        .answer(
            &sdk,
            r#"{"calendar": "calendar.BIKRAM_SAMBAT", "date": {"year": 2082, "month": 10, "day": 1}, "era": "era.KIRAT_YELE"}"#,
        )
        .unwrap();
    let value = &answered.value;
    assert_eq!(value["date"]["era"]["year"], 5086, "{value}");
    assert!(
        value["date"]["era"]["era"]
            .as_str()
            .is_some_and(|key| key.ends_with("KIRAT_YELE")),
        "{value}"
    );
    // 15 January 2026 was a Thursday.
    assert_eq!(value["weekday"], "THURSDAY", "{value}");
}
