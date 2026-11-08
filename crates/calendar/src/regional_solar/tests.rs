#![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

//! The printed month beginnings each calendar is held to: individual days
//! of the *Rashtriya Panchang*'s regional tables (Positional Astronomy
//! Centre, Kolkata; the edition is named with each), chosen where the rule
//! decides, with the sankranti's printed IST beside them. The misses the
//! page records (`docs/calendars/regional-solar.md`) are asserted as the
//! SDK's day, with the printed day in the comment.

use teistro_core::catalogue::{Calendar, Era};
use teistro_core::envelope::CalendarResolution;
use teistro_core::error::Status;

use super::*;
use crate::CalendarSystem;
use crate::gregorian::{fixed_from_gregorian, gregorian_from_fixed};

fn calendar(id: Calendar) -> &'static RegionalSolar {
    RegionalSolar::shipped(id).unwrap()
}

/// The Gregorian day a month of the year that opens in a Gregorian year
/// begins on: the year's month counted from 1 at its first sign.
fn month_start(id: Calendar, gregorian_year: i32, month: u8) -> (i32, u8, u8) {
    let system = calendar(id);
    let year = gregorian_year - system.region().year_start.gregorian_offset;
    gregorian_from_fixed(system.to_fixed_ymd(year, month, 1).unwrap())
}

/// Asserts each `(sign, opening Gregorian year, printed start)`: the
/// sign's month in the year that opened in that Gregorian year.
fn assert_starts(id: Calendar, cases: &[(u8, i32, (i32, u8, u8))]) {
    let region = calendar(id).region();
    for &(sign, year, printed) in cases {
        let month = (1..=12)
            .find(|month| region.sign_of_month(*month) == Some(sign))
            .unwrap();
        assert_eq!(
            month_start(id, year, month),
            printed,
            "{}: sign {sign} of the year opening in {year}",
            region.name
        );
    }
}

#[test]
fn tamil_months_begin_before_sunset_at_chennai() {
    assert_starts(
        Calendar::TamilSolar,
        &[
            // Saka 1914: Chittirai, 13 April 1992.
            (0, 1992, (1992, 4, 13)),
            // Saka 1940: Tula at 18:44 and Vrishchika at 18:32, after
            // sunset, begin the next day.
            (6, 2018, (2018, 10, 18)),
            (7, 2018, (2018, 11, 17)),
            // Saka 1942: Simha at 19:11, the next day.
            (4, 2020, (2020, 8, 17)),
            // Saka 1945: Mithuna at 18:16, before sunset, the same day.
            (2, 2023, (2023, 6, 15)),
            // Saka 1946: Vrisha at 17:54, the same day.
            (1, 2024, (2024, 5, 14)),
            // Saka 1946: Meena at 18:51 on 14 March 2025, the next day.
            (11, 2024, (2025, 3, 15)),
            // Saka 1947: Karka at 17:32, the same day.
            (3, 2025, (2025, 7, 16)),
            // Saka 1942: Meena at 18:04 on 14 March 2021, the same day.
            (11, 2020, (2021, 3, 14)),
            // Saka 1948: Mesha, 14 April 2026.
            (0, 2026, (2026, 4, 14)),
        ],
    );
}

#[test]
fn bengal_months_begin_the_civil_day_after() {
    assert_starts(
        Calendar::BengalSolar,
        &[
            // Saka 1914: Vaisakha, 14 April 1992.
            (0, 1992, (1992, 4, 14)),
            // Saka 1940: Vrisha at 05:03 on 15 May 2018, after midnight,
            // begins the day after next by the Hindu day: 16 May.
            (1, 2018, (2018, 5, 16)),
            // Saka 1942: Mithuna at 23:54 on 14 June 2020, before
            // midnight: 15 June (Assam printed 16 June, C765).
            (2, 2020, (2020, 6, 15)),
            // Saka 1943: Vrisha at 23:25 on 14 May 2021: 15 May by the
            // rule; the column prints 16 May, most likely Assam's day.
            (1, 2021, (2021, 5, 15)),
            // Saka 1943: Meena at 00:16 on 15 March 2022: 16 March.
            (11, 2021, (2022, 3, 16)),
            // Saka 1946: Mithuna at 00:27 on 15 June 2024: 16 June.
            (2, 2024, (2024, 6, 16)),
            // Saka 1948: Karka at 23:39 on 16 July 2026: 17 July.
            (3, 2026, (2026, 7, 17)),
            // Saka 1946: Bengali San 1431 opened on 14 April 2024.
            (0, 2024, (2024, 4, 14)),
        ],
    );
}

#[test]
fn odia_months_begin_on_the_sunrise_day_at_bhubaneswar() {
    assert_starts(
        Calendar::OdiaSolar,
        &[
            // Saka 1914: Vaisakha, 13 April 1992.
            (0, 1992, (1992, 4, 13)),
            // Saka 1940: Vrisha at 05:03 on 15 May 2018, before sunrise:
            // the day before.
            (1, 2018, (2018, 5, 14)),
            // Saka 1945: Karka at 05:07 on 17 July 2023: 16 July.
            (3, 2023, (2023, 7, 16)),
            // Saka 1944: Vrisha at 05:30 on 15 May 2022, after sunrise:
            // the same day.
            (1, 2022, (2022, 5, 15)),
            // Saka 1943: Mithuna at 06:01 on 15 June 2021: the same day.
            (2, 2021, (2021, 6, 15)),
            // Saka 1942: Mithuna at 23:54 on 14 June 2020: the same day.
            (2, 2020, (2020, 6, 14)),
            // Saka 1947: Kumbha at 04:09 on 13 February 2026: the 12th.
            (10, 2025, (2026, 2, 12)),
            // Saka 1941 printed Vrishchika (00:51 on 17 November 2019) on
            // the 17th, against every other edition's reckoning; the SDK
            // keeps the sunrise day, the 16th.
            (7, 2019, (2019, 11, 16)),
        ],
    );
}

#[test]
fn malayalam_months_begin_before_three_fifths_of_the_daylight() {
    assert_starts(
        Calendar::MalayalamSolar,
        &[
            // Kollam 1168: Chingam, 17 August 1992.
            (4, 1992, (1992, 8, 17)),
            // Kollam 1195: Simha at 13:02 and Kanya at 13:03, before the
            // mark: the same day.
            (4, 2019, (2019, 8, 17)),
            (5, 2019, (2019, 9, 17)),
            // Kollam 1197: Tula at 13:12, the same day.
            (6, 2021, (2021, 10, 17)),
            // Kollam 1199: Kanya at 13:31, after the mark: the next day.
            (5, 2023, (2023, 9, 18)),
            // Kollam 1201: Tula at 13:46, the next day.
            (6, 2025, (2025, 10, 18)),
            // Kollam 1201: Chingam, 17 August 2025.
            (4, 2025, (2025, 8, 17)),
            // Kollam 1199: Simha at 13:33 on 17 August 2023, a minute and
            // a half from the mark: the SDK's sankranti falls before it
            // (the 17th), the column prints the 18th.
            (4, 2023, (2023, 8, 17)),
            // Kollam 1167: Mesha 1992 by the Malabar rule, as printed.
            (0, 1991, (1992, 4, 14)),
            // Kollam 1194: Mesha at 14:09 on 14 April 2019, after the
            // mark: 15 April by the Malabar rule; the 2018 and later
            // editions print the 14th (C764).
            (0, 2018, (2019, 4, 15)),
        ],
    );
}

#[test]
fn every_date_names_its_era_and_frame() {
    for (id, era) in [
        (Calendar::TamilSolar, Era::Shaka),
        (Calendar::BengalSolar, Era::Bengali),
        (Calendar::OdiaSolar, Era::Shaka),
        (Calendar::MalayalamSolar, Era::Kollam),
    ] {
        let system = calendar(id);
        assert_eq!(system.id(), id);
        let date = system.date_of(fixed_from_gregorian(2025, 1, 1)).unwrap();
        assert_eq!(date.era.map(|e| (e.era, e.year)), Some((era, date.year)));
        let CalendarResolution::Computed { model } = &date.resolution else {
            panic!("{id:?}: {:?}", date.resolution);
        };
        assert_eq!(model, system.table().frame);
        assert!(model.contains("ayanamsha LAHIRI (true basis)"), "{model}");
        assert!(model.contains(&system.region().rule.key()), "{model}");
        assert!(!model.contains(env!("CARGO_PKG_VERSION")), "{model}");
        let capabilities = system.capabilities();
        assert!(!capabilities.needs_ephemeris && !capabilities.needs_place);
        assert_eq!(capabilities.eras, [era]);
    }
}

#[test]
fn every_day_reads_back_and_years_abut() {
    let ids = REGIONS.iter().map(|region| region.calendar);
    for id in ids.chain([Calendar::TamilVakya]) {
        let system = calendar(id);
        let region = system.region();
        let (first, last) = system.capabilities().range;
        let (first_year, last_year) = system.years();
        assert_eq!(
            usize::try_from(last_year - first_year + 1).unwrap(),
            system.table().month_lengths.len()
        );
        assert_eq!(
            gregorian_from_fixed(first).0,
            GREGORIAN_SPAN.0,
            "{}",
            region.name
        );
        let mut previous: Option<CalendarDate> = None;
        for day in first.get()..=last.get() {
            let fixed = FixedDay::new(day);
            let date = system.date_of(fixed).unwrap();
            assert_eq!(system.fixed_of(&date).unwrap(), fixed, "{date:?}");
            if let Some(before) = previous {
                let next_day = before.day + 1;
                let follows = (date.year, date.month, date.day)
                    == (before.year, before.month, next_day)
                    || (date.day == 1
                        && ((date.year, date.month) == (before.year, before.month + 1)
                            || (date.year, date.month, before.month) == (before.year + 1, 1, 12)));
                assert!(follows, "{before:?} then {date:?}");
            }
            previous = Some(date);
        }
        for year in first_year..=last_year {
            let days: u32 = (1..=12)
                .map(|month| u32::from(system.month_length(year, month).unwrap()))
                .sum();
            assert!(days == 365 || days == 366, "{} {year}: {days}", region.name);
            assert_eq!(system.is_leap(year), days == 366);
            assert!(
                (1..=12)
                    .all(|month| (29..=32).contains(&system.month_length(year, month).unwrap())),
                "{} {year}",
                region.name
            );
        }
    }
}

#[test]
fn a_date_outside_the_table_or_the_month_is_refused() {
    let tamil = calendar(Calendar::TamilSolar);
    let (first, last) = tamil.years();
    for year in [first - 1, last + 1] {
        let refused = tamil.to_fixed_ymd(year, 1, 1).unwrap_err();
        assert_eq!(refused.status, Status::OutOfRange, "{refused}");
        assert!(refused.message.contains("Tamil"), "{refused}");
        assert!(tamil.year_start(year).is_none());
    }
    assert!(tamil.to_fixed_ymd(1946, 13, 1).is_err());
    assert!(tamil.to_fixed_ymd(1946, 0, 1).is_err());
    let length = tamil.month_length(1946, 1).unwrap();
    assert!(tamil.to_fixed_ymd(1946, 1, length + 1).is_err());
    let marked =
        CalendarDate::defined(Calendar::TamilSolar, 1946, 1, 1).with_leap(Some(crate::Leap::Day));
    assert!(tamil.fixed_of(&marked).is_err());
    let (range_first, range_last) = tamil.capabilities().range;
    assert!(tamil.date_of(range_first.plus_days(-1)).is_err());
    assert!(tamil.date_of(range_last.plus_days(1)).is_err());
    assert!(RegionalSolar::shipped(Calendar::Gregorian).is_none());
    assert_eq!(MALAYALAM.sign_of_month(0), None);
}

#[test]
fn the_engine_reproduces_the_shipped_rows_under_the_tables_rule() {
    // The engine run over the Surya Siddhanta's Sun is a different
    // astronomy, but the year start and the rule are the calendar's own:
    // a Malayalam year opens at Simha and runs twelve months.
    let text = teistro_siddhanta::SuryaSiddhanta::text();
    let row = MALAYALAM.engine(&text).year(1201).unwrap();
    let (year, month, _) = gregorian_from_fixed(row.start);
    assert_eq!((year, month), (2025, 8));
    assert!(row.days() == 365 || row.days() == 366);
    assert_eq!(MALAYALAM.engine(&text).year_start(), MALAYALAM.year_start);
}

/// The month beginnings the vakya almanacs print, as days of the month
/// from Chittirai to Panguni, by the Gregorian year the Tamil year opens
/// in: the Manonmani Vilasam suddha vakya almanac (archive.org
/// `bqfm-manonmani-vilasa-suddha-vakya-panchangam-sarvajitt` for 1947 to
/// 1962, `calw-manonmani-vilasa-suddha-vakkiya-panchangam-isvara-` for
/// 1997 to 2021) and the Koviloor vakya almanac for 1978 (Tamil Digital
/// Library `KMM1082`). A zero is a month not read.
const VAKYA_PRINTED: [(i32, [u8; 12]); 42] = [
    (1947, [14, 15, 15, 17, 17, 17, 18, 16, 16, 14, 13, 14]),
    (1948, [13, 14, 14, 16, 16, 16, 17, 16, 15, 14, 12, 14]),
    (1949, [13, 14, 14, 16, 17, 17, 17, 16, 16, 14, 12, 14]),
    (1950, [13, 14, 15, 16, 17, 17, 17, 16, 16, 14, 13, 14]),
    (1951, [14, 15, 15, 17, 17, 17, 18, 17, 16, 14, 13, 14]),
    (1952, [13, 14, 14, 16, 16, 16, 17, 16, 15, 14, 12, 14]),
    (1953, [13, 14, 15, 16, 17, 17, 17, 16, 16, 14, 12, 14]),
    (1954, [13, 14, 15, 16, 17, 17, 17, 16, 16, 14, 13, 14]),
    (1955, [14, 15, 15, 17, 17, 17, 18, 17, 16, 14, 13, 14]),
    (1956, [13, 14, 14, 16, 16, 16, 17, 16, 15, 14, 12, 14]),
    (1957, [13, 14, 15, 16, 17, 17, 17, 16, 16, 14, 12, 14]),
    (1958, [14, 14, 15, 16, 17, 17, 17, 16, 16, 14, 13, 14]),
    (1959, [14, 15, 15, 17, 17, 17, 18, 17, 16, 14, 13, 14]),
    (1960, [13, 14, 14, 16, 16, 16, 17, 16, 15, 14, 12, 14]),
    (1961, [13, 14, 15, 16, 17, 17, 17, 16, 16, 14, 12, 14]),
    (1962, [14, 14, 15, 16, 0, 0, 0, 0, 0, 0, 0, 0]),
    (1978, [14, 15, 15, 17, 17, 17, 18, 17, 16, 14, 13, 15]),
    (1997, [14, 15, 15, 17, 17, 17, 18, 16, 16, 14, 13, 15]),
    (1998, [14, 15, 15, 17, 17, 17, 18, 17, 16, 15, 13, 15]),
    (1999, [14, 15, 15, 17, 18, 18, 18, 17, 16, 15, 13, 14]),
    (2000, [13, 14, 15, 16, 17, 17, 17, 16, 16, 14, 13, 14]),
    (2001, [14, 15, 15, 17, 17, 17, 18, 16, 16, 14, 13, 15]),
    (2002, [14, 15, 15, 17, 17, 17, 18, 17, 16, 15, 13, 15]),
    (2003, [14, 15, 15, 17, 18, 18, 18, 17, 16, 15, 13, 14]),
    (2004, [13, 14, 15, 16, 17, 17, 17, 16, 16, 14, 13, 14]),
    (2005, [14, 15, 15, 17, 17, 17, 18, 16, 16, 14, 13, 15]),
    (2006, [14, 15, 15, 17, 17, 17, 18, 17, 16, 15, 13, 15]),
    (2007, [14, 15, 15, 17, 18, 18, 18, 17, 17, 15, 13, 14]),
    (2008, [13, 14, 15, 16, 17, 17, 17, 16, 16, 14, 13, 14]),
    (2009, [14, 15, 15, 17, 17, 17, 18, 17, 16, 14, 13, 15]),
    (2010, [14, 15, 15, 17, 17, 17, 18, 17, 16, 15, 13, 15]),
    (2011, [14, 15, 16, 17, 18, 18, 18, 17, 17, 15, 13, 14]),
    (2012, [13, 14, 15, 16, 17, 17, 17, 16, 16, 14, 13, 14]),
    (2013, [14, 15, 15, 17, 17, 17, 18, 17, 16, 14, 13, 15]),
    (2014, [14, 15, 15, 17, 17, 17, 18, 17, 16, 15, 13, 15]),
    (2015, [14, 15, 16, 17, 18, 18, 18, 17, 17, 15, 13, 14]),
    (2016, [14, 14, 15, 16, 17, 17, 17, 16, 16, 14, 13, 14]),
    (2017, [14, 15, 15, 17, 17, 17, 18, 17, 16, 14, 13, 15]),
    (2018, [14, 15, 15, 17, 17, 17, 18, 17, 16, 15, 13, 15]),
    (2019, [14, 15, 16, 17, 18, 18, 18, 17, 17, 15, 13, 14]),
    (2020, [14, 14, 15, 16, 17, 17, 17, 16, 16, 14, 13, 14]),
    (2021, [14, 15, 15, 17, 17, 17, 18, 17, 16, 14, 13, 15]),
];

/// The Gregorian month each Tamil month begins in, Chittirai first.
const TAMIL_GREGORIAN_MONTHS: [u8; 12] = [4, 5, 6, 7, 8, 9, 10, 11, 12, 1, 2, 3];

#[test]
fn vakya_months_begin_on_the_almanacs_days() {
    let mut read = 0;
    let mut parting = 0;
    for (year, days) in VAKYA_PRINTED {
        for ((month, day), gregorian_month) in (1..=12).zip(days).zip(TAMIL_GREGORIAN_MONTHS) {
            if day == 0 {
                continue;
            }
            let gregorian_year = if month <= 9 { year } else { year + 1 };
            let printed = (gregorian_year, gregorian_month, day);
            assert_eq!(
                month_start(Calendar::TamilVakya, year, month),
                printed,
                "month {month} of {year}"
            );
            read += 1;
            parting += usize::from(month_start(Calendar::TamilSolar, year, month) != printed);
        }
    }
    // Every month read, and the drik calendar misses a fifth of them.
    assert_eq!((read, parting), (496, 89));
}

#[test]
fn vakya_months_turn_on_the_almanacs_daytime() {
    // Kali 5101: the Kumbha sankranti at 29 nadis 41 vinadis after the
    // sunrise of 12 February 2001, past that day's 29 nadis 7 vinadis of
    // daylight: Masi begins on the 13th, as printed, where a thirty-nadi
    // cutoff would give the 12th.
    let kumbha = teistro_siddhanta::vakya::sankranti(5101, 10).unwrap();
    assert_eq!(kumbha.since_sunrise().0, 29);
    assert_eq!(month_start(Calendar::TamilVakya, 2000, 11), (2001, 2, 13));
    // Saka 1869's Mesha sankranti at night: the year began on Monday.
    assert_eq!(
        vakya::month_start(1869, 0),
        Some(fixed_from_gregorian(1947, 4, 14))
    );
    assert_eq!(vakya::month_start(1869, 12), None);
}

#[test]
fn the_vakya_calendar_names_its_frame_and_keeps_the_tamil_year() {
    let system = calendar(Calendar::TamilVakya);
    assert_eq!(system.id(), Calendar::TamilVakya);
    assert_eq!(system.region().calendar, Calendar::TamilSolar);
    assert_eq!(system.years(), calendar(Calendar::TamilSolar).years());
    let date = system.date_of(fixed_from_gregorian(2025, 1, 1)).unwrap();
    assert_eq!(date.calendar, Calendar::TamilVakya);
    assert_eq!(
        date.era.map(|e| (e.era, e.year)),
        Some((Era::Shaka, date.year))
    );
    let CalendarResolution::Computed { model } = &date.resolution else {
        panic!("{:?}", date.resolution);
    };
    assert_eq!(model, vakya::FRAME);
    assert!(
        model.contains("Vakyakarana") && model.contains(&TAMIL.rule.key()),
        "{model}"
    );
    let capabilities = system.capabilities();
    assert!(!capabilities.needs_ephemeris && !capabilities.needs_place);
    assert_eq!(capabilities.eras, [Era::Shaka]);
    let (_, last) = system.years();
    let refused = system.to_fixed_ymd(last + 1, 1, 1).unwrap_err();
    assert!(refused.message.contains("Tamil vakya"), "{refused}");
    let early = system
        .date_of(system.capabilities().range.0.plus_days(-1))
        .unwrap_err();
    assert!(early.message.contains("Tamil vakya"), "{early}");
    let marked =
        CalendarDate::defined(Calendar::TamilVakya, 1946, 1, 1).with_leap(Some(crate::Leap::Day));
    assert!(system.fixed_of(&marked).is_err());
    assert!(system.to_fixed_ymd(1946, 13, 1).is_err());
}

#[test]
fn the_schools_part_in_a_fifth_of_the_months() {
    // Tamil years opening 1900 to 2100: the vakya month begins a day
    // after the drik one in 450 months and a day before it in 74.
    let (mut later, mut earlier) = (0, 0);
    for year in 1900..=2100 {
        for month in 1..=12 {
            let vakya = fixed_from_gregorian_tuple(month_start(Calendar::TamilVakya, year, month));
            let drik = fixed_from_gregorian_tuple(month_start(Calendar::TamilSolar, year, month));
            match drik.days_until(vakya) {
                1 => later += 1,
                -1 => earlier += 1,
                0 => {}
                other => panic!("{year} month {month}: {other} days apart"),
            }
        }
    }
    assert_eq!((later, earlier), (450, 74));
}

fn fixed_from_gregorian_tuple((year, month, day): (i32, u8, u8)) -> FixedDay {
    fixed_from_gregorian(year, month, day)
}
