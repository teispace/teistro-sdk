//! Nepal's gazetted holidays (`03-design/nepal-gazette-holidays.md`, SR-123):
//! the set each Nepal pack's listing carries, every year's festivals named
//! in the catalogue, and every year's days held to the days
//! `NEPAL_PANCHANGA` reckons for the festivals each holiday names, or to a
//! named exception (SR-157).

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a test fails by panicking and indexes what it built"
)]

use std::collections::BTreeMap;

use teistro::catalogue::{Calendar, HolidayScope};
use teistro::festival::{Holiday, HolidaySet};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, FestivalPack, FestivalRequest, UtcOffset};

/// The Bikram Sambat days each key of `NEPAL_PANCHANGA` falls on over a
/// VS year, at Kathmandu over the committee's sky.
fn reckoned(year: i32) -> BTreeMap<String, Vec<(u8, u8)>> {
    let place = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    let sdk = Context::builder()
        .profile("nepali-committee")
        .ephemeris([Ephemeris::SuryaSiddhanta])
        .build()
        .unwrap();
    let request = FestivalRequest::from(FestivalPack::NepalPanchanga);
    let gregorian = |(year, month, day): (i32, u8, u8)| {
        CalendarDate::defined(Calendar::Gregorian, year, month, day)
    };
    let mut days: BTreeMap<String, Vec<(u8, u8)>> = BTreeMap::new();
    // A VS year begins in mid-April: read from the 10th to the 16th a year
    // on, in three spans, and keep the days of that year.
    let ad = year - 57;
    for (from, to) in [
        ((ad, 4, 10), (ad, 8, 16)),
        ((ad, 8, 17), (ad, 12, 15)),
        ((ad, 12, 16), (ad + 1, 4, 16)),
    ] {
        let found = sdk
            .almanac()
            .festivals(
                &gregorian(from),
                &gregorian(to),
                &place,
                UtcOffset::literal(5, 45, 0),
                &request,
            )
            .unwrap()
            .value;
        let on = found
            .observances
            .iter()
            .map(|day| (&day.rule, &day.day))
            .chain(found.ekadashis.iter().map(|day| (&day.rule, &day.day)))
            .chain(found.dated.iter().map(|day| (&day.rule, &day.day)))
            .chain(
                found
                    .anniversaries
                    .iter()
                    .map(|day| (&day.key, &day.observance.day)),
            )
            .chain(found.added.iter().map(|day| (&day.key, &day.day)));
        for (key, day) in on {
            let bs = sdk.calendar().convert(day, Calendar::BikramSambat).unwrap();
            if bs.year == year {
                days.entry(key.clone())
                    .or_default()
                    .push((bs.month, bs.day));
            }
        }
    }
    days
}

#[test]
fn every_nepal_pack_lists_the_gazette_and_no_other_pack_does() {
    // Every year's, VS 2076 to 2083.
    let every = HolidaySet::NepalGazette.holidays().len();
    assert_eq!(every, 347);
    for pack in FestivalPack::ALL {
        let listed = pack.listing();
        let nepal = matches!(pack, FestivalPack::Nepal | FestivalPack::NepalPanchanga);
        assert_eq!(
            listed.holidays.len(),
            if nepal { every } else { 0 },
            "{}",
            pack.key()
        );
    }
    let json = serde_json::to_value(FestivalPack::NepalPanchanga.listing()).unwrap();
    let of_2083: Vec<&serde_json::Value> = json["holidays"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|holiday| holiday["year"] == 2083)
        .collect();
    assert_eq!(of_2083.len(), 46);
    assert_eq!(of_2083[8]["festivals"][3], "JYAPU_DIWAS");
    assert_eq!(of_2083[8]["scope"], "NATIONAL");
    assert_eq!(of_2083[8]["first"]["month"], 9);
}

#[test]
fn every_festival_a_holiday_names_is_in_the_catalogue() {
    let sdk = Context::builder().build().unwrap();
    let set = HolidaySet::NepalGazette;
    let cancelled = set.years().into_iter().flat_map(|year| set.cancelled(year));
    let mut named = 0;
    for holiday in set.holidays().into_iter().chain(cancelled) {
        for key in &holiday.festivals {
            assert!(
                sdk.intl().entity(&format!("festival.{key}")).is_ok(),
                "{key}, named by {}, is no festival of the catalogue",
                holiday.source
            );
            named += 1;
        }
    }
    assert!(named > 300, "{named}");
}

/// A holiday's festivals and its first day, month and day.
type Named<'a> = (Vec<&'a str>, Option<(u8, u8)>);

#[test]
fn every_entry_naming_more_than_one_festival_keeps_them_apart_from_its_notice() {
    let several: Vec<Holiday> = HolidaySet::NepalGazette
        .holidays_of(2083)
        .into_iter()
        .filter(|holiday| holiday.festivals.len() > 1)
        .collect();
    let named: Vec<Named<'_>> = several
        .iter()
        .map(|holiday| {
            (
                holiday.festivals.iter().map(String::as_str).collect(),
                holiday.first.as_ref().map(|day| (day.month, day.day)),
            )
        })
        .collect();
    assert_eq!(
        named,
        [
            (
                vec!["CHANDI_PURNIMA", "BUDDHA_JAYANTI", "UBHAULI"],
                Some((1, 18))
            ),
            (vec!["VIJAYA_DASHAMI", "PHULPATI"], Some((6, 31))),
            (vec!["LAKSHMI_PUJA", "YAMA_DWITIYA"], Some((7, 22))),
            (
                vec!["DHANYA_PURNIMA", "UDHAULI", "YOMARI_PUNHI", "JYAPU_DIWAS"],
                Some((9, 9))
            ),
            (vec!["MAGHI_THARU", "MAKARA_SANKRANTI"], Some((10, 1))),
        ]
    );
    // 9 Poush 2083 is 24 December 2026, one holiday of four festivals.
    let sdk = Context::builder().build().unwrap();
    let poush_9 = several[3].first.as_ref().unwrap().date();
    let gregorian = sdk
        .calendar()
        .convert(&poush_9, Calendar::Gregorian)
        .unwrap();
    assert_eq!(
        (gregorian.year, gregorian.month, gregorian.day),
        (2026, 12, 24)
    );
    assert_eq!(several[3].scope, HolidayScope::National);
    assert!(several[3].notice.ne.starts_with("धान्य पूर्णिमा"));
}

/// The gazette's days the pack's rules do not give, each with its reason:
/// `(VS year, festival)`. A rule change that moves another year's day, or
/// mends one of these, fails the test below until this list says so.
const EXCEPTIONS: [(i32, &str, &str); 4] = [
    (
        2076,
        "KIJA_PUJA",
        "the cabinet's added holiday of Kartik 13, the day after Bhai tika, where the committee prints किजापूजा in Yama dwitiya's row, Kartik 12 (VS 2076 p. 15, C2304, C2423)",
    ),
    (
        2079,
        "HOLI_HILLS",
        "the Holika fire a day early, Falgun 22 (6 March 2023): the rule takes the later day when the later pradosha holds part of the full moon, standing the earlier day's bhadra against it; whether that bhadra stood is not encoded (C199, C2420)",
    ),
    (
        2079,
        "HOLI_TERAI",
        "the day after HOLI_HILLS, Falgun 23, so a day early with it (C2420)",
    ),
    (
        2083,
        "DURA_MHAIPRU_NAKUMA",
        "the Dura community's day, Poush 15, which no rule of the pack reckons (C2420)",
    ),
];

#[test]
fn every_printed_day_holds_the_day_the_pack_reckons_for_its_festivals() {
    let listed = FestivalPack::NepalPanchanga.listing().rules;
    let mut held = 0;
    let mut excepted = Vec::new();
    for year in HolidaySet::NepalGazette.years() {
        let days = reckoned(year);
        for holiday in HolidaySet::NepalGazette.holidays_of(year) {
            let Some(first) = &holiday.first else {
                continue;
            };
            let first = (first.month, first.day);
            let last = holiday
                .last
                .as_ref()
                .map_or(first, |day| (day.month, day.day));
            for key in &holiday.festivals {
                let kept = days
                    .get(key)
                    .is_some_and(|on| on.iter().any(|&day| (first..=last).contains(&day)));
                if EXCEPTIONS
                    .iter()
                    .any(|&(y, k, _)| (y, k) == (year, key.as_str()))
                {
                    assert!(!kept, "VS {year} {key} is held now: drop its exception");
                    excepted.push((year, key.clone()));
                    continue;
                }
                assert!(
                    days.contains_key(key) || listed.iter().all(|row| row.key != *key),
                    "VS {year}: {key} is in the pack and has no day"
                );
                if !days.contains_key(key) {
                    continue;
                }
                assert!(
                    kept,
                    "VS {year} {key}: the pack gives {:?}, {} prints {first:?} to {last:?}",
                    days[key], holiday.source
                );
                held += 1;
            }
        }
        // The festival days a span is read with are the days the pack reckons.
        for (key, day) in HolidaySet::NepalGazette.festival_days(year) {
            assert!(
                days.get(&key)
                    .is_some_and(|on| on.contains(&(day.month, day.day))),
                "VS {year} {key}: the notice is read with {day:?}, the pack gives {:?}",
                days.get(&key)
            );
        }
    }
    assert_eq!(excepted.len(), EXCEPTIONS.len(), "{excepted:?}");
    // Every festival named by a dated holiday of VS 2076 to 2083 the pack
    // reckons, but the exceptions above (SR-157).
    assert_eq!(held, 0, "the count of held days, pinned once measured");
}

/// What SR-163 asked of a notice's record (`nepal-gazette-holidays.md` §6):
/// the clause apart, a later decision typed with its document, each
/// year's documents, the weekly rest days, and every region, district and
/// community a holiday names said in every Devanagari locale.
#[test]
fn a_notice_carries_its_clauses_changes_documents_and_rest_days() {
    let json = serde_json::to_value(FestivalPack::NepalPanchanga.listing()).unwrap();
    let holidays = json["holidays"].as_array().unwrap();
    let gen_z = holidays
        .iter()
        .find(|holiday| holiday["year"] == 2083 && holiday["clause"] == "8 (ग)")
        .unwrap();
    assert_eq!(gen_z["scope"], "NATIONAL");
    assert_eq!(gen_z["changes"][0]["kind"], "RESCOPED");
    assert_eq!(gen_z["changes"][0]["scopeBefore"], "OFFICES_OPEN");
    assert_eq!(gen_z["changes"][0]["document"]["decided"]["month"], 5);
    let mourning = holidays
        .iter()
        .find(|holiday| holiday["year"] == 2083 && holiday["changes"][0]["kind"] == "ADDED")
        .unwrap();
    assert_eq!(
        mourning["scope"], "OFFICES_OPEN",
        "a day of mourning, offices open"
    );
    let terai = holidays
        .iter()
        .find(|holiday| holiday["year"] == 2083 && holiday["festivals"][0] == "HOLI_TERAI")
        .unwrap();
    assert_eq!(terai["districts"].as_array().unwrap().len(), 21);
    let years = json["holidayYears"].as_array().unwrap();
    assert_eq!(years.len(), 8);
    let latest = years.last().unwrap();
    assert_eq!(latest["notice"]["sha256"].as_str().unwrap().len(), 64);
    assert_eq!(latest["notice"]["retrieved"], "2026-09-14");
    let rest = json["restDays"].as_array().unwrap();
    assert_eq!(rest[0]["vara"], "SHANIVARA");
    assert_eq!(rest[1]["scope"], "OFFICES_AND_SCHOOLS");

    let set = HolidaySet::NepalGazette;
    let cancelled = set.years().into_iter().flat_map(|year| set.cancelled(year));
    let named: std::collections::BTreeSet<String> = set
        .holidays()
        .into_iter()
        .chain(cancelled)
        .flat_map(|holiday| {
            let regions = holiday
                .regions
                .iter()
                .chain(&holiday.districts)
                .map(|key| format!("region.{key}"));
            let communities = holiday
                .communities
                .iter()
                .map(|key| format!("community.{key}"));
            regions.chain(communities).collect::<Vec<_>>()
        })
        .chain([format!(
            "holiday_scope.{}",
            HolidayScope::OfficesAndSchools.key()
        )])
        .collect();
    assert!(named.len() > 30, "{named:?}");
    let devanagari = |name: &str| name.chars().any(|c| ('\u{0900}'..='\u{097F}').contains(&c));
    for locale in ["hi-Deva-IN", "ne-Deva-NP", "sa-Deva"] {
        let reader = Context::builder().locale(locale).build().unwrap();
        let latin: Vec<String> = named
            .iter()
            .filter_map(|key| {
                let entity = reader.intl().entity(key).unwrap();
                let name = entity.form("name").unwrap().to_owned();
                (!devanagari(&name)).then(|| format!("{key}: {name}"))
            })
            .collect();
        assert!(latin.is_empty(), "{locale} names in Latin: {latin:?}");
    }
}
