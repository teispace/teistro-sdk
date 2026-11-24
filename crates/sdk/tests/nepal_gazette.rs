//! Nepal's gazetted holidays (`03-design/nepal-gazette-holidays.md`, SR-123):
//! the set each Nepal pack's listing carries, every year's festivals named
//! in the catalogue, and VS 2083's days held to the days `NEPAL_PANCHANGA`
//! reckons for the festivals each holiday names.

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

/// The Bikram Sambat days each key of `NEPAL_PANCHANGA` falls on over VS
/// 2083, at Kathmandu over the committee's sky.
fn reckoned() -> BTreeMap<String, Vec<(u8, u8)>> {
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
    // VS 2083 is 14 April 2026 to 13 April 2027, in three spans.
    for (from, to) in [
        ((2026, 4, 14), (2026, 8, 16)),
        ((2026, 8, 17), (2026, 12, 15)),
        ((2026, 12, 16), (2027, 4, 13)),
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
            if bs.year == 2083 {
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
    assert_eq!(every, 346);
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

#[test]
fn every_printed_day_holds_the_day_the_pack_reckons_for_its_festivals() {
    let days = reckoned();
    let mut held = 0;
    for holiday in HolidaySet::NepalGazette.holidays_of(2083) {
        let Some(first) = &holiday.first else {
            continue;
        };
        let first = (first.month, first.day);
        let last = holiday
            .last
            .as_ref()
            .map_or(first, |day| (day.month, day.day));
        for key in &holiday.festivals {
            let Some(on) = days.get(key) else {
                assert!(
                    FestivalPack::NepalPanchanga
                        .listing()
                        .rules
                        .iter()
                        .all(|row| row.key != *key),
                    "{key} is in the pack and has no day in VS 2083"
                );
                continue;
            };
            assert!(
                on.iter().any(|&day| (first..=last).contains(&day)),
                "{key}: the pack gives {on:?}, {} prints {first:?} to {last:?}",
                holiday.source
            );
            held += 1;
        }
    }
    // Every festival named by a holiday with a printed day, 47, but Dura
    // Mhaipru Nakuma, which the pack has no rule for. The six holidays the
    // notice gives "on the day of" (the two Eids, Mawlid, Guru Nanak
    // Jayanti, Bhoto Jatra, Sirua Pavani) print no day to hold.
    assert_eq!(held, 46);
    // The festival days a span is read with are the days the pack reckons.
    for (key, day) in HolidaySet::NepalGazette.festival_days(2083) {
        assert!(
            days.get(&key)
                .is_some_and(|on| on.contains(&(day.month, day.day))),
            "{key}: the notice is read with {day:?}, the pack gives {:?}",
            days.get(&key)
        );
    }
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
