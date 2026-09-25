//! The date, time, datetime, ghati and duration functions over the SDK's
//! own sources: the patterns and names a locale declares in
//! `sdk.calendar`, the conversion between calendars, the built-in defaults
//! when a locale declares nothing, and the number options the patterns
//! lean on.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]

use std::sync::Arc;

use teistro_calendar::{CalendarDate, shipped};
use teistro_core::catalogue::{Calendar, Era};
use teistro_core::quantity::JulianDay;
use teistro_intl::analysis::{ParamType, signature};
use teistro_intl::source::{DayPeriod, Entry, Tree};
use teistro_intl::{ClockTime, Ghati, Intl, Value, params, sdk_root};
use teistro_time::EmbeddedTzdb;

fn engine(locale: &str) -> Intl {
    let tree = Tree::load(&sdk_root()).unwrap_or_else(|e| panic!("{e}"));
    let mut intl = Intl::from_tree(&tree).unwrap_or_else(|e| panic!("{e}"));
    intl.set_locale(locale).unwrap_or_else(|e| panic!("{e}"));
    intl
}

fn text(intl: &Intl, source: &str, value: Value) -> String {
    let rendered = intl
        .render_source(source, &params([("v", value)]))
        .unwrap_or_else(|e| panic!("{source}: {e}"));
    assert!(
        rendered.warnings.is_empty(),
        "{source}: {:?}",
        rendered.warnings
    );
    rendered.text
}

#[test]
fn dates_render_through_the_locales_patterns_in_its_calendar_and_digits() {
    let english = engine("en-Latn");
    let nepali = engine("ne-Deva-NP");
    // The architecture page's example: a Bikram Sambat date in Nepali.
    let bs = CalendarDate::defined(Calendar::BikramSambat, 2081, 5, 19);
    assert_eq!(
        text(&nepali, "{$v :date}", Value::Date(bs.clone())),
        "२०८१/०५/१९ गते"
    );
    assert_eq!(
        text(&nepali, "{$v :date style=long}", Value::Date(bs.clone())),
        "२०८१ भदौ १९ गते"
    );
    assert_eq!(
        text(&english, "{$v :date style=long}", Value::Date(bs)),
        "19 Bhadra 2081 BS"
    );
    // A Gregorian date, numeric, long and full, in both locales.
    let gregorian = CalendarDate::defined(Calendar::Gregorian, 2024, 9, 4);
    assert_eq!(
        text(&english, "{$v :date}", Value::Date(gregorian.clone())),
        "2024-09-04"
    );
    assert_eq!(
        text(
            &english,
            "{$v :date style=long}",
            Value::Date(gregorian.clone())
        ),
        "4 September 2024"
    );
    assert_eq!(
        text(
            &english,
            "{$v :date style=full}",
            Value::Date(gregorian.clone())
        ),
        "Wednesday, 4 September 2024"
    );
    assert_eq!(
        text(&nepali, "{$v :date style=full}", Value::Date(gregorian)),
        "बुधबार, २०२४ सेप्टेम्बर ४"
    );
    // A Julian date links to the Gregorian patterns; an era's year and its
    // short form are the pattern's when the date carries an era; a
    // `pattern=` names any message.
    let old = CalendarDate::defined(Calendar::Julian, 1582, 10, 4);
    assert_eq!(
        text(&english, "{$v :date style=long}", Value::Date(old)),
        "4 October 1582"
    );
    let dated =
        CalendarDate::defined(Calendar::Gregorian, -43, 3, 15).with_era(Era::BeforeCommonEra, 44);
    assert_eq!(
        text(
            &english,
            "{$v :date style=long} {$v :date pattern=sdk.reason.appName}",
            Value::Date(dated)
        ),
        "15 March 44 Teistro"
    );
}

#[test]
fn a_date_converts_to_the_calendar_the_option_names() {
    let nepali = engine("ne-Deva-NP");
    let gregorian = CalendarDate::defined(Calendar::Gregorian, 2024, 9, 4);
    let fixed = shipped(Calendar::Gregorian)
        .unwrap()
        .fixed_of(&gregorian)
        .unwrap();
    let expected = shipped(Calendar::BikramSambat)
        .unwrap()
        .date_of(fixed)
        .unwrap();
    let rendered = text(
        &nepali,
        "{$v :date calendar=BIKRAM_SAMBAT}",
        Value::Date(gregorian.clone()),
    );
    let digits = |n: i32| -> String {
        format!("{n:02}")
            .chars()
            .map(|c| char::from_u32(0x966 + c.to_digit(10).unwrap()).unwrap())
            .collect()
    };
    assert_eq!(
        rendered,
        format!(
            "{}/{}/{} गते",
            digits(expected.year),
            digits(i32::from(expected.month)),
            digits(i32::from(expected.day))
        )
    );
    // An unknown target calendar warns and leaves the date in its own.
    let warned = nepali
        .render_source(
            "{$v :date calendar=MARTIAN}",
            &params([("v", Value::Date(gregorian))]),
        )
        .unwrap();
    assert!(warned.warnings.iter().any(|w| w.contains("MARTIAN")));
    assert_eq!(warned.text, "२०२४-०९-०४");
}

#[test]
fn a_twelve_hour_clock_reads_as_each_locale_reads_it() {
    let english = engine("en-Latn");
    let nepali = engine("ne-Deva-NP");
    let morning = ClockTime::new(6, 15, 0);
    let evening = ClockTime::new(18, 5, 30);
    let midnight = ClockTime::new(0, 0, 0);
    let noon = ClockTime::new(12, 0, 0);

    // English reads by am and pm.
    assert_eq!(
        text(&english, "{$v :time hour12=true}", Value::Time(morning)),
        "6:15 am"
    );
    assert_eq!(
        text(&english, "{$v :time hour12=true}", Value::Time(evening)),
        "6:05 pm"
    );
    assert_eq!(
        text(
            &english,
            "{$v :time hour12=true style=long}",
            Value::Time(evening)
        ),
        "6:05:30 pm"
    );
    assert_eq!(
        text(&english, "{$v :time hour12=true}", Value::Time(midnight)),
        "12:00 am",
        "midnight is twelve, not zero"
    );
    assert_eq!(
        text(&english, "{$v :time hour12=true}", Value::Time(noon)),
        "12:00 pm"
    );

    // Nepali reads by the part of the day, in its own digits.
    assert_eq!(
        text(&nepali, "{$v :time hour12=true}", Value::Time(morning)),
        "बिहान ६:१५"
    );
    assert_eq!(
        text(&nepali, "{$v :time hour12=true}", Value::Time(evening)),
        "साँझ ६:०५"
    );
    assert_eq!(
        text(&nepali, "{$v :time hour12=true}", Value::Time(midnight)),
        "राति १२:००"
    );
    assert_eq!(
        text(
            &nepali,
            "{$v :time hour12=true}",
            Value::Time(ClockTime::new(14, 30, 0))
        ),
        "दिउँसो २:३०"
    );

    // The clock the option does not ask for is unchanged.
    assert_eq!(text(&english, "{$v :time}", Value::Time(evening)), "18:05");

    // A date and time takes the option too.
    let date = CalendarDate::defined(Calendar::Gregorian, 2024, 9, 4);
    assert_eq!(
        text(
            &english,
            "{$v :datetime hour12=true}",
            Value::DateTime(date.clone(), morning)
        ),
        "2024-09-04, 6:15 am"
    );

    // A pattern of the caller's own reads the same parameters.
    assert_eq!(
        text(
            &english,
            "{$v :time pattern=sdk.calendar.time.numeric12}",
            Value::Time(morning)
        ),
        "6:15 am"
    );
}

#[test]
fn times_datetimes_ghatis_and_durations_render_and_pluralise() {
    let english = engine("en-Latn");
    let nepali = engine("ne-Deva-NP");
    let time = ClockTime::new(5, 30, 7);
    assert_eq!(text(&english, "{$v :time}", Value::Time(time)), "05:30");
    assert_eq!(
        text(&english, "{$v :time style=long}", Value::Time(time)),
        "05:30:07"
    );
    assert_eq!(text(&nepali, "{$v :time}", Value::Time(time)), "०५:३०");
    let date = CalendarDate::defined(Calendar::Gregorian, 2024, 9, 4);
    assert_eq!(
        text(
            &english,
            "{$v :datetime style=long}",
            Value::DateTime(date, time)
        ),
        "4 September 2024, 05:30:07"
    );
    let ghati = Ghati::new(12, 5, 0);
    assert_eq!(text(&english, "{$v :ghati}", Value::Ghati(ghati)), "12-05");
    assert_eq!(
        text(&english, "{$v :ghati style=long}", Value::Ghati(ghati)),
        "12 ghatis 5 pala"
    );
    assert_eq!(
        text(&nepali, "{$v :ghati style=long}", Value::Ghati(ghati)),
        "१२ घडी ५ पला"
    );
    assert_eq!(
        text(&english, "{$v :duration unit=day}", Value::Int(1)),
        "1 day"
    );
    assert_eq!(
        text(&english, "{$v :duration unit=day}", Value::Int(3)),
        "3 days"
    );
    assert_eq!(
        text(&english, "{$v :duration}", Value::Num(2.5)),
        "2.5 minutes"
    );
    assert_eq!(
        text(&nepali, "{$v :duration unit=day}", Value::Int(3)),
        "३ दिन"
    );
    // A bare parameter renders its default text too.
    assert_eq!(text(&english, "{$v}", Value::Ghati(ghati)), "12-05");
    assert_eq!(text(&english, "{$v}", Value::Time(time)), "05:30");
}

#[test]
fn a_locale_without_calendar_messages_falls_back_to_the_defaults_with_a_warning() {
    let tree = Tree::load(&sdk_root()).unwrap_or_else(|e| panic!("{e}"));
    let mut locales = tree.locales.clone();
    for locale in locales.values_mut() {
        locale.namespaces.remove("sdk.calendar");
    }
    let mut intl = Intl::new(locales).unwrap_or_else(|e| panic!("{e}"));
    intl.set_locale("ne-Deva-NP").unwrap();
    let date = CalendarDate::defined(Calendar::BikramSambat, 2081, 5, 19);
    let rendered = intl
        .render_source("{$v :date style=long}", &params([("v", Value::Date(date))]))
        .unwrap();
    assert_eq!(rendered.text, "२०८१-०५-१९");
    assert!(
        rendered
            .warnings
            .iter()
            .any(|w| w.contains("default pattern"))
    );
    let rendered = intl
        .render_source(
            "{$v :ghati style=long}",
            &params([("v", Value::Ghati(Ghati::new(12, 5, 30)))]),
        )
        .unwrap();
    assert_eq!(rendered.text, "१२-०५-३०");
    // A value of the wrong kind is refused with a warning and the fallback text.
    let wrong = intl
        .render_source("{$v :date}", &params([("v", Value::Int(7))]))
        .unwrap();
    assert!(wrong.warnings.iter().any(|w| w.contains("needs a date")));
}

#[test]
fn integers_take_padding_and_grouping_options_and_the_analysis_types_the_functions() {
    let english = engine("en-Latn");
    let nepali = engine("ne-Deva-NP");
    assert_eq!(text(&english, "{$v :integer}", Value::Int(2024)), "2,024");
    assert_eq!(
        text(
            &english,
            "{$v :integer useGrouping=false}",
            Value::Int(2024)
        ),
        "2024"
    );
    assert_eq!(
        text(
            &english,
            "{$v :integer minimumIntegerDigits=3}",
            Value::Int(7)
        ),
        "007"
    );
    assert_eq!(
        text(&nepali, "{$v :integer useGrouping=false}", Value::Int(2081)),
        "२०८१"
    );
    assert_eq!(
        text(
            &english,
            "{$v :integer minimumIntegerDigits=2}",
            Value::Int(-5)
        ),
        "-05"
    );
    let tree = Tree::load(&sdk_root()).unwrap_or_else(|e| panic!("{e}"));
    let meta = &tree.base().unwrap().meta;
    let sig = signature(
        &teistro_intl::mf2::parse(
            "{$d :date calendar=GREGORIAN pattern=sdk.calendar.GREGORIAN.date.long} {$t :time} {$dt :datetime} {$g :ghati} {$n :duration unit=day}",
        )
        .unwrap(),
        meta,
    );
    assert_eq!(sig.params["d"], ParamType::Date);
    assert_eq!(sig.params["t"], ParamType::Time);
    assert_eq!(sig.params["dt"], ParamType::DateTime);
    assert_eq!(sig.params["g"], ParamType::Ghati);
    assert_eq!(sig.params["n"], ParamType::Number);
    assert_eq!(
        sig.links,
        vec![String::from("sdk.calendar.GREGORIAN.date.long")]
    );
}

/// An engine whose base locale renders a time as the part of the day it
/// falls in, with the division the caller gives, so the whole path can be
/// walked: the metadata, the walk over the parts, the message the key
/// resolves to, and the pattern that places it.
fn day_period_engine(periods: Vec<DayPeriod>) -> Intl {
    let mut tree = Tree::load(&sdk_root()).unwrap_or_else(|e| panic!("{e}"));
    let base = tree
        .locales
        .get_mut("en-Latn")
        .unwrap_or_else(|| panic!("the base locale"));
    if !periods.is_empty() {
        base.meta.day_periods = periods;
    }
    base.namespaces
        .get_mut("sdk.calendar")
        .unwrap_or_else(|| panic!("sdk.calendar"))
        .insert(
            String::from("time.numeric"),
            Entry::Message(String::from("{$dayPeriod}")),
        );
    let mut intl = Intl::from_tree(&tree).unwrap_or_else(|e| panic!("{e}"));
    intl.set_locale("en-Latn").unwrap_or_else(|e| panic!("{e}"));
    intl
}

#[test]
fn the_part_of_the_day_comes_from_the_locales_own_division() {
    // What every locale takes when it states nothing.
    let default = day_period_engine(Vec::new());
    for (hour, expected) in [
        (0, "at night"),
        (3, "at night"),
        (4, "in the morning"),
        (11, "in the morning"),
        (12, "in the afternoon"),
        (15, "in the afternoon"),
        (16, "in the evening"),
        (19, "in the evening"),
        (20, "at night"),
        (23, "at night"),
    ] {
        assert_eq!(
            text(
                &default,
                "{$v :time}",
                Value::Time(ClockTime::new(hour, 0, 0))
            ),
            expected,
            "hour {hour}"
        );
    }

    // A language whose day divides in two rather than four. Nothing but
    // `_meta.json` changes: the same keys, the same messages, the same
    // pattern.
    let halves = day_period_engine(vec![
        DayPeriod {
            from: 6,
            key: String::from("morning"),
        },
        DayPeriod {
            from: 18,
            key: String::from("night"),
        },
    ]);
    for (hour, expected) in [
        (5, "at night"),
        (6, "in the morning"),
        (17, "in the morning"),
        (18, "at night"),
    ] {
        assert_eq!(
            text(
                &halves,
                "{$v :time}",
                Value::Time(ClockTime::new(hour, 0, 0))
            ),
            expected,
            "hour {hour}"
        );
    }
}

#[test]
fn a_duration_breaks_into_the_units_a_message_names() {
    let english = engine("en-Latn");
    let nepali = engine("ne-Deva-NP");

    // 3725 seconds is an hour, two minutes and five seconds, joined by
    // the locale's own `and` pattern.
    assert_eq!(
        text(
            &english,
            "{$v :duration unit=second into=|hour,minute,second|}",
            Value::Int(3725)
        ),
        "1 hour, 2 minutes and 5 seconds"
    );

    // The order the units are named in does not change the reading.
    assert_eq!(
        text(
            &english,
            "{$v :duration unit=second into=|second,hour,minute|}",
            Value::Int(3725)
        ),
        "1 hour, 2 minutes and 5 seconds"
    );

    // A part that is zero is dropped: nobody says "and no minutes".
    assert_eq!(
        text(
            &english,
            "{$v :duration unit=second into=|hour,minute,second|}",
            Value::Int(3605)
        ),
        "1 hour and 5 seconds"
    );
    assert_eq!(
        text(
            &english,
            "{$v :duration unit=second into=|hour,minute|}",
            Value::Int(7200)
        ),
        "2 hours"
    );

    // Unless every part is zero, when the shortest unit named keeps it.
    assert_eq!(
        text(
            &english,
            "{$v :duration unit=second into=|hour,minute,second|}",
            Value::Int(0)
        ),
        "0 seconds"
    );

    // The last unit keeps the remainder rather than rounding it away.
    assert_eq!(
        text(
            &english,
            "{$v :duration unit=second into=|minute,second|}",
            Value::Num(90.5)
        ),
        "1 minute and 30.5 seconds"
    );

    // Whole days out of minutes, which is what a dasha period arrives as.
    assert_eq!(
        text(
            &english,
            "{$v :duration into=|day,hour,minute|}",
            Value::Int(1_501)
        ),
        "1 day, 1 hour and 1 minute"
    );

    // The whole length is negative, not each part of it.
    assert_eq!(
        text(
            &english,
            "{$v :duration unit=second into=|hour,minute|}",
            Value::Int(-3720)
        ),
        "-1 hour and 2 minutes"
    );

    // The locale's digits, its plural rules and its own list pattern.
    assert_eq!(
        text(
            &nepali,
            "{$v :duration unit=second into=|hour,minute,second|}",
            Value::Int(3725)
        ),
        "१ घण्टा, २ मिनेट र ५ सेकेन्ड"
    );

    // Without `into=`, nothing changes.
    assert_eq!(
        text(&english, "{$v :duration unit=hour}", Value::Int(3)),
        "3 hours"
    );
}

/// The civil value an instant should read as, in the Gregorian calendar.
fn civil(year: i32, month: u8, day: u8, time: ClockTime) -> Value {
    Value::DateTime(
        CalendarDate::defined(Calendar::Gregorian, year, month, day),
        time,
    )
}

/// An instant, the SDK's own, is read in the zone a `timeZone` option
/// names and then exactly as the civil date and time it was there: the
/// same text the civil value gives, through the same patterns and the same
/// calendar conversion.
#[test]
fn an_instant_reads_as_the_civil_time_it_was_in_the_zone_named() {
    let english = engine("en-Latn");
    let noon_utc = Value::Instant(JulianDay::literal(2_460_000.0));
    for (zone, expected) in [
        ("UTC", civil(2023, 2, 24, ClockTime::new(12, 0, 0))),
        ("+05:45", civil(2023, 2, 24, ClockTime::new(17, 45, 0))),
        ("-0300", civil(2023, 2, 24, ClockTime::new(9, 0, 0))),
        ("+14", civil(2023, 2, 25, ClockTime::new(2, 0, 0))),
    ] {
        for function in ["date", "time", "datetime"] {
            assert_eq!(
                text(
                    &english,
                    &format!("{{$v :{function} timeZone=|{zone}|}}"),
                    noon_utc.clone()
                ),
                text(&english, &format!("{{$v :{function}}}"), expected.clone()),
                "{function} in {zone}"
            );
        }
    }
    // Moved first and then converted: the Bikram Sambat date of the day
    // it was in Kathmandu, not of the day it was in Greenwich.
    let nepali = engine("ne-Deva-NP");
    assert_eq!(
        text(
            &nepali,
            "{$v :date timeZone=|+14| calendar=BIKRAM_SAMBAT}",
            noon_utc.clone()
        ),
        text(
            &nepali,
            "{$v :date calendar=BIKRAM_SAMBAT}",
            civil(2023, 2, 25, ClockTime::new(2, 0, 0))
        ),
    );
    // A zone given by the caller at render time.
    let rendered = english
        .render_source(
            "{$v :time timeZone=$zone}",
            &params([("v", noon_utc), ("zone", Value::from("+05:45"))]),
        )
        .unwrap();
    assert!(rendered.warnings.is_empty(), "{:?}", rendered.warnings);
    assert_eq!(
        rendered.text,
        text(
            &english,
            "{$v :time}",
            Value::Time(ClockTime::new(17, 45, 0))
        )
    );
}

/// A zone's name is answered by the zone database the engine is given, the
/// SDK's embedded one here, daylight saving and all; without one it warns
/// by name rather than guessing.
#[test]
fn a_named_zone_is_answered_by_the_database_and_refused_without_one() {
    let mut english = engine("en-Latn");
    let july = Value::Instant(JulianDay::literal(2_460_127.0));
    let january = Value::Instant(JulianDay::literal(2_459_946.0));
    let unanswered = english
        .render_source(
            "{$v :time timeZone=|America/New_York|}",
            &params([("v", july.clone())]),
        )
        .unwrap();
    assert!(
        unanswered
            .warnings
            .iter()
            .any(|w| w.contains("no zone database")),
        "{:?}",
        unanswered.warnings
    );
    english.set_time_zones(Arc::new(EmbeddedTzdb::shared()));
    let hour = |value: Value| text(&english, "{$v :time timeZone=|America/New_York|}", value);
    assert_eq!(
        hour(july),
        text(&english, "{$v :time}", Value::Time(ClockTime::new(8, 0, 0))),
        "daylight saving"
    );
    assert_eq!(
        hour(january),
        text(&english, "{$v :time}", Value::Time(ClockTime::new(7, 0, 0))),
        "standard time"
    );
    let misspelt = english
        .render_source(
            "{$v :time timeZone=|Asia/Kathmandoo|}",
            &params([("v", Value::Instant(JulianDay::literal(2_460_000.0)))]),
        )
        .unwrap();
    assert!(
        misspelt
            .warnings
            .iter()
            .any(|w| w.contains("Asia/Kathmandu")),
        "the nearest name is suggested: {:?}",
        misspelt.warnings
    );
}

/// What is not an instant in a zone warns and says why: an instant with
/// no zone (read in UTC), a zone on a value already civil (left as it is),
/// and an offset that is not one.
#[test]
fn an_instant_without_a_zone_and_a_zone_without_an_instant_warn() {
    let english = engine("en-Latn");
    let instant = Value::Instant(JulianDay::literal(2_460_000.0));
    let unzoned = english
        .render_source("{$v :time}", &params([("v", instant.clone())]))
        .unwrap();
    assert!(
        unzoned
            .warnings
            .iter()
            .any(|w| w.contains("names no `timeZone`"))
    );
    assert_eq!(
        unzoned.text,
        text(
            &english,
            "{$v :time}",
            Value::Time(ClockTime::new(12, 0, 0))
        )
    );
    let civil_time = Value::Time(ClockTime::new(6, 15, 0));
    let moved = english
        .render_source(
            "{$v :time timeZone=|+05:45|}",
            &params([("v", civil_time.clone())]),
        )
        .unwrap();
    assert!(moved.warnings.iter().any(|w| w.contains("already a civil")));
    assert_eq!(moved.text, text(&english, "{$v :time}", civil_time));
    for bad in ["+5:4", "+2500", "+05:60"] {
        let refused = english
            .render_source(
                &format!("{{$v :time timeZone=|{bad}|}}"),
                &params([("v", instant.clone())]),
            )
            .unwrap();
        assert!(!refused.warnings.is_empty(), "{bad}");
    }
}

/// A date or time function that names a zone takes an instant, and the
/// typed accessors every binding generates say so.
#[test]
fn a_zone_types_the_value_as_an_instant() {
    let tree = Tree::load(&sdk_root()).unwrap_or_else(|e| panic!("{e}"));
    let meta = &tree.base().unwrap().meta;
    let sig = signature(
        &teistro_intl::mf2::parse(
            "{$at :time timeZone=$zone} {$on :date timeZone=|UTC|} {$when :datetime timeZone=|+05:45|} {$civil :time}",
        )
        .unwrap(),
        meta,
    );
    assert_eq!(sig.params["at"], ParamType::Instant);
    assert_eq!(sig.params["on"], ParamType::Instant);
    assert_eq!(sig.params["when"], ParamType::Instant);
    assert_eq!(sig.params["zone"], ParamType::String);
    assert_eq!(sig.params["civil"], ParamType::Time);
}
