//! The sighted Hijri calendar against the Eids printed in Nepal and India
//! (`03-design/hijri-sighted-measured.md`): every criterion and zone over
//! the SDK's built-in sky, at Kathmandu and at each Indian list's capital.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "tests fail by panicking, index what they asked for, and the sweep prints its table"
)]

use std::collections::BTreeMap;

use teistro::catalogue::Calendar;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Ut1};
use teistro::{CalendarDate, Context, Ephemeris, FestivalPack, FestivalRequest, UtcOffset};
use teistro_astro::completion::Completion;
use teistro_astro::crescent::{Crescent, Sighting};
use teistro_astro::delta_t::DeltaTModel;
use teistro_calendar::gregorian::{fixed_from_gregorian, gregorian_from_fixed};
use teistro_calendar::hijri::fixed_from_hijri;
use teistro_calendar::hijri_sighted::SightedMonths;
use teistro_calendar::{CalendarSystem, FixedDay};
use teistro_core::settings::{CrescentCriterion, CrescentZone, OverridePolicy};
use teistro_panchanga::festival::{PublishedEntry, PublishedSet};
use teistro_panchanga::hijri_sighted::sighted_months;

/// The days observed by a sighted calendar: Eid al-Fitr, Eid al-Adha,
/// Ashura (India's "Muharram") and the Mawlid, as (key, month, day).
const DAYS: [(&str, u8, u8); 4] = [
    ("EID_AL_FITR", 10, 1),
    ("EID_AL_ADHA", 12, 10),
    ("MUHARRAM", 1, 10),
    ("MAWLID", 3, 12),
];

/// The three current Nepali calendars' Eids, VS 2076 to 2084, as the
/// days they print less the tabular calendar's: (key, AH year, days).
/// The three agree every year (the maintainer's measurement, in
/// aggregate; the calendars' rows stay outside the repository).
const NEPAL: [(&str, i32, i64); 12] = [
    ("EID_AL_FITR", 1440, 0),
    ("EID_AL_FITR", 1441, 1),
    ("EID_AL_FITR", 1442, 1),
    ("EID_AL_FITR", 1443, 0),
    ("EID_AL_FITR", 1444, 0),
    ("EID_AL_FITR", 1445, 1),
    ("EID_AL_FITR", 1446, 0),
    ("EID_AL_FITR", 1447, 1),
    ("EID_AL_ADHA", 1443, 0),
    ("EID_AL_ADHA", 1444, 0),
    ("EID_AL_ADHA", 1445, 0),
    ("EID_AL_ADHA", 1447, 1),
];

fn place(latitude: f64, longitude: f64, altitude: f64) -> Place {
    Place::new(
        Latitude::literal(latitude),
        Longitude::literal(longitude),
        Altitude::literal(altitude),
    )
}

fn kathmandu() -> Place {
    place(27.7172, 85.324, 1400.0)
}

/// Each Indian list and the capital its days are reckoned at.
fn lists() -> Vec<(&'static str, PublishedSet, Place)> {
    vec![
        (
            "central",
            PublishedSet::india_central(),
            place(28.6139, 77.209, 216.0),
        ),
        (
            "andhra_pradesh",
            PublishedSet::india_andhra_pradesh(),
            place(16.5131, 80.5165, 30.0),
        ),
        (
            "assam",
            PublishedSet::india_assam(),
            place(26.1433, 91.7898, 55.0),
        ),
        (
            "bihar",
            PublishedSet::india_bihar(),
            place(25.5941, 85.1376, 53.0),
        ),
        (
            "gujarat",
            PublishedSet::india_gujarat(),
            place(23.2156, 72.6369, 81.0),
        ),
        (
            "karnataka",
            PublishedSet::india_karnataka(),
            place(12.9716, 77.5946, 920.0),
        ),
        (
            "kerala",
            PublishedSet::india_kerala(),
            place(8.5241, 76.9366, 10.0),
        ),
        (
            "maharashtra",
            PublishedSet::india_maharashtra(),
            place(19.076, 72.8777, 14.0),
        ),
        (
            "odisha",
            PublishedSet::india_odisha(),
            place(20.2961, 85.8245, 45.0),
        ),
        (
            "tamil_nadu",
            PublishedSet::india_tamil_nadu(),
            place(13.0827, 80.2707, 6.0),
        ),
        (
            "telangana",
            PublishedSet::india_telangana(),
            place(17.385, 78.4867, 542.0),
        ),
        (
            "uttar_pradesh",
            PublishedSet::india_uttar_pradesh(),
            place(26.8467, 80.9462, 123.0),
        ),
        (
            "west_bengal",
            PublishedSet::india_west_bengal(),
            place(22.5726, 88.3639, 9.0),
        ),
    ]
}

/// Every printed day of the four a list holds: (key, the day, whether a
/// declaration after the sighting set it).
fn printed(set: &PublishedSet) -> Vec<(String, FixedDay, bool)> {
    set.entries
        .iter()
        .filter_map(|entry| match entry {
            PublishedEntry::Add {
                key,
                day,
                subject_to,
                ..
            } if DAYS.iter().any(|(wanted, ..)| wanted == key) => Some((
                key.clone(),
                fixed_from_gregorian(day.year, day.month, day.day),
                subject_to.is_none(),
            )),
            _ => None,
        })
        .collect()
}

fn months(
    at: Place,
    sighting: Sighting,
    (from, to): ((i32, u8, u8), (i32, u8, u8)),
) -> SightedMonths {
    let provider = teistro_ephemeris_builtin::provider::Builtin::new();
    let sky = sky(&provider);
    sighted_months(
        &sky,
        at,
        DeltaTModel::TableThenModel,
        sighting,
        (
            fixed_from_gregorian(from.0, from.1, from.2),
            fixed_from_gregorian(to.0, to.1, to.2),
        ),
    )
    .unwrap()
}

/// The sighted day of `key` nearest `near`, as a fixed day.
fn sighted_day(table: &SightedMonths, key: &str, near: FixedDay) -> FixedDay {
    let (_, month, day) = *DAYS.iter().find(|(wanted, ..)| *wanted == key).unwrap();
    let year = table.date_of(near).unwrap().year;
    let mut best = None::<FixedDay>;
    for year in [year - 1, year, year + 1] {
        let Ok(found) = table.fixed_of(&CalendarDate::defined(
            Calendar::HijriSighted,
            year,
            month,
            day,
        )) else {
            continue;
        };
        if best.is_none_or(|best| near.days_until(found).abs() < near.days_until(best).abs()) {
            best = Some(found);
        }
    }
    best.unwrap()
}

/// The Nepali events as (key, AH year, printed day).
fn nepal_days() -> Vec<(&'static str, i32, FixedDay)> {
    NEPAL
        .iter()
        .map(|&(key, year, offset)| {
            let (_, month, day) = *DAYS.iter().find(|(wanted, ..)| *wanted == key).unwrap();
            (
                key,
                year,
                fixed_from_hijri(year, month, day).plus_days(offset),
            )
        })
        .collect()
}

/// Every printed Eid of the three Nepali calendars, at Kathmandu, under
/// the default: Odeh's criterion to zone A.
#[test]
fn the_default_gives_nepals_printed_eids() {
    let default = Sighting::new(CrescentCriterion::Odeh2006, CrescentZone::A).unwrap();
    let table = months(kathmandu(), default, ((2019, 5, 1), (2026, 6, 30)));
    let parted: Vec<_> = nepal_days()
        .into_iter()
        .filter_map(|(key, year, day)| {
            let found = sighted_day(&table, key, day);
            (found != day).then(|| (key, year, gregorian_from_fixed(day), found.days_until(day)))
        })
        .collect();
    println!(
        "Kathmandu, Odeh A: {} of {} parted: {parted:?}",
        parted.len(),
        NEPAL.len()
    );
    // As measured: the default parts from none of the twelve.
    assert!(parted.is_empty(), "{parted:?}");
}

/// The same through the façade: a request's rules on `HIJRI_SIGHTED` are
/// kept on the sighted days, and the answer names the crescent test.
#[test]
fn a_request_keeps_its_rules_on_the_sighted_days() {
    let context = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap();
    let request = FestivalRequest::from_json(
        r#"{"rules": [
            {"key":"EID_AL_FITR","source":"1 Shawwal","civilDate":{"calendar":"HIJRI_SIGHTED","month":10,"day":1}}
        ]}"#,
    )
    .unwrap();
    let found = context
        .almanac()
        .festivals(
            &CalendarDate::defined(Calendar::Gregorian, 2025, 3, 1),
            &CalendarDate::defined(Calendar::Gregorian, 2025, 4, 30),
            &kathmandu(),
            UtcOffset::literal(5, 45, 0),
            &request,
        )
        .unwrap();
    let kept: Vec<_> = found
        .value
        .dated
        .iter()
        .map(|kept| (kept.rule.as_str(), kept.day.month, kept.day.day))
        .collect();
    let default = Sighting::new(CrescentCriterion::Odeh2006, CrescentZone::A).unwrap();
    let table = months(kathmandu(), default, ((2025, 3, 1), (2025, 4, 30)));
    let eid = gregorian_from_fixed(
        table
            .fixed_of(&CalendarDate::defined(Calendar::HijriSighted, 1446, 10, 1))
            .unwrap(),
    );
    assert_eq!(kept, [("EID_AL_FITR", eid.1, eid.2)]);
    assert!(
        found
            .value
            .dated
            .iter()
            .all(|kept| kept.on.calendar == Calendar::HijriSighted)
    );
    assert!(
        found
            .provenance
            .applied_conventions
            .iter()
            .any(|applied| applied.knob == "calendars.hijri_criterion"
                && applied.value == "ODEH_2006 to zone A"),
        "{:?}",
        found.provenance.applied_conventions
    );
    // Odeh's zone D begins no month, and says which knob.
    let refused = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .settings_json(r#"{"calendars": {"hijri_zone": "D"}}"#)
        .build()
        .unwrap()
        .almanac()
        .festivals(
            &CalendarDate::defined(Calendar::Gregorian, 2025, 3, 1),
            &CalendarDate::defined(Calendar::Gregorian, 2025, 3, 31),
            &kathmandu(),
            UtcOffset::literal(5, 45, 0),
            &request,
        )
        .unwrap_err();
    assert_eq!(refused.field(), Some("calendars.hijri_zone"));
}

/// Nepal's own pack keeps the gazette's two Eids on the sighted days at
/// Kathmandu: each Eid from Eid al-Fitr 1445 (April 2024) to Eid al-Adha
/// 1447 (May 2026) once, every printed one on its printed day.
#[test]
fn the_nepal_pack_keeps_its_printed_eids() {
    let context = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap();
    let request = FestivalRequest::from(FestivalPack::NepalPanchanga);
    // A range holds at most a year: three requests over the span.
    let spans = [
        ((2024, 4, 1), (2025, 3, 31)),
        ((2025, 4, 1), (2026, 3, 31)),
        ((2026, 4, 1), (2026, 6, 30)),
    ];
    let mut kept = Vec::new();
    for ((y0, m0, d0), (y1, m1, d1)) in spans {
        let found = context
            .almanac()
            .festivals(
                &CalendarDate::defined(Calendar::Gregorian, y0, m0, d0),
                &CalendarDate::defined(Calendar::Gregorian, y1, m1, d1),
                &kathmandu(),
                UtcOffset::literal(5, 45, 0),
                &request,
            )
            .unwrap();
        kept.extend(
            found
                .value
                .dated
                .iter()
                .filter(|kept| kept.rule.starts_with("EID_AL_"))
                .map(|kept| {
                    let day = &kept.day;
                    (
                        kept.rule.clone(),
                        fixed_from_gregorian(day.year, day.month, day.day),
                    )
                }),
        );
    }
    // Six Eids fall in the span; the calendars measured print five of
    // them (Eid al-Adha 1446 is not among the twelve), each on its day.
    assert_eq!(kept.len(), 6, "{kept:?}");
    let missed: Vec<_> = nepal_days()
        .into_iter()
        .filter(|&(key, year, day)| year >= 1445 && !kept.contains(&(key.to_owned(), day)))
        .collect();
    assert!(missed.is_empty(), "{missed:?} in {kept:?}");
}

/// Where the Sun does not set the sighted months cannot be reckoned: the
/// Nepal pack still answers its other days at Tromsø in its polar summer,
/// gives no Eid, and says why (`03-design/hijri-sighted.md` §5).
#[test]
fn a_place_without_an_evening_keeps_the_packs_other_days() {
    // A polar day has no sunrise either, so an app there names a policy
    // for the day's other rules; the crescent's evening still never comes.
    let context = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .settings_json(r#"{"day": {"polar_day_policy": "CIVIL_MIDNIGHT"}}"#)
        .build()
        .unwrap();
    let tromso = place(69.6492, 18.9553, 10.0);
    let found = context
        .almanac()
        .festivals(
            &CalendarDate::defined(Calendar::Gregorian, 2026, 5, 15),
            &CalendarDate::defined(Calendar::Gregorian, 2026, 6, 15),
            &tromso,
            UtcOffset::literal(2, 0, 0),
            &FestivalRequest::from(FestivalPack::NepalPanchanga),
        )
        .unwrap();
    assert!(!found.value.dated.is_empty() || !found.value.observances.is_empty());
    assert!(
        found
            .value
            .dated
            .iter()
            .all(|kept| !kept.rule.starts_with("EID_AL_"))
    );
    assert!(
        found
            .provenance
            .applied_conventions
            .iter()
            .any(|applied| applied.knob == "calendars.hijri_criterion"
                && applied.reason.contains("were not reckoned")),
        "{:?}",
        found.provenance.applied_conventions
    );
}

/// The sweep the measured page prints: every criterion and zone against
/// Nepal's twelve and every Indian list's moon-sighted days, and the depth
/// the chain of months opens at, held to twice it.
#[test]
#[ignore = "the measured page's sweep: cargo test -p teistro --test hijri_sighted -- --ignored --nocapture"]
fn sweep() {
    let span = ((2019, 1, 1), (2027, 12, 31));
    agreement(span);
    margins(span);
    depth();
}

/// Every criterion and zone against Nepal's twelve and the Indian lists.
fn agreement(span: ((i32, u8, u8), (i32, u8, u8))) {
    let zones: Vec<(CrescentCriterion, CrescentZone)> = [
        (CrescentCriterion::Odeh2006, CrescentZone::A),
        (CrescentCriterion::Odeh2006, CrescentZone::B),
        (CrescentCriterion::Odeh2006, CrescentZone::C),
        (CrescentCriterion::Yallop1997, CrescentZone::A),
        (CrescentCriterion::Yallop1997, CrescentZone::B),
        (CrescentCriterion::Yallop1997, CrescentZone::C),
        (CrescentCriterion::Yallop1997, CrescentZone::D),
    ]
    .to_vec();
    let lists = lists();
    for (criterion, zone) in zones {
        let sighting = Sighting::new(criterion, zone).unwrap();
        let table = months(kathmandu(), sighting, span);
        let mut parted = Vec::new();
        for (key, year, day) in nepal_days() {
            let found = sighted_day(&table, key, day);
            if found != day {
                parted.push(format!(
                    "{key} {year} {:?} {:+}",
                    gregorian_from_fixed(day),
                    found.days_until(day)
                ));
            }
        }
        println!(
            "NEPAL {sighting}: {}/{} agree; parted: {}",
            NEPAL.len() - parted.len(),
            NEPAL.len(),
            parted.join("; ")
        );
        let mut counts: BTreeMap<&str, (usize, usize, Vec<String>)> = BTreeMap::new();
        for (name, set, at) in &lists {
            let table = months(*at, sighting, span);
            for (key, day, declared) in printed(set) {
                let found = sighted_day(&table, &key, day);
                let entry = counts.entry(name).or_default();
                entry.1 += 1;
                if found == day {
                    entry.0 += 1;
                } else {
                    let mark = if declared { " (declared)" } else { "" };
                    entry.2.push(format!(
                        "{key} {:?} {:+}{mark}",
                        gregorian_from_fixed(day),
                        found.days_until(day)
                    ));
                }
            }
        }
        let (agree, total) = counts
            .values()
            .fold((0, 0), |sum, (a, t, _)| (sum.0 + a, sum.1 + t));
        println!("INDIA {sighting}: {agree}/{total} agree");
        for (name, (a, t, parts)) in &counts {
            println!("  {name}: {a}/{t}; {}", parts.join("; "));
        }
    }
}

/// The sky the margins and the depth read.
fn sky(
    provider: &teistro_ephemeris_builtin::provider::Builtin,
) -> Completion<'_, teistro_ephemeris_builtin::provider::Builtin> {
    Completion::new(
        provider,
        OverridePolicy::PreferNative,
        DeltaTModel::TableThenModel,
    )
}

/// Odeh's V on the two evenings around each Nepali month's beginning, at
/// Kathmandu: the evening before its first day and the evening before
/// that.
fn margins(span: ((i32, u8, u8), (i32, u8, u8))) {
    let provider = teistro_ephemeris_builtin::provider::Builtin::new();
    let sky = sky(&provider);
    let crescent = Crescent::new(&sky, kathmandu(), DeltaTModel::TableThenModel);
    let value = |day: FixedDay| {
        let noon = day.jd_at_midnight().unwrap().get() + 0.5 - 85.324 / 360.0;
        let evening = crescent.evening(JulianDay::<Ut1>::literal(noon)).unwrap();
        evening.geometry.map_or(f64::NAN, |at| at.odeh_v)
    };
    let default = Sighting::new(CrescentCriterion::Odeh2006, CrescentZone::A).unwrap();
    let table = months(kathmandu(), default, span);
    for (key, year, _) in nepal_days() {
        let month = if key == "EID_AL_FITR" { 10 } else { 12 };
        let first = table
            .fixed_of(&CalendarDate::defined(
                Calendar::HijriSighted,
                year,
                month,
                1,
            ))
            .unwrap();
        println!(
            "MARGIN {key} {year}: first {:?}; V {:.2} on the evening before, {:.2} the evening before that",
            gregorian_from_fixed(first),
            value(first.plus_days(-1)),
            value(first.plus_days(-2))
        );
    }
}

/// The chain's depth: months reckoned from twelve months back and from
/// twenty-four agree, 1900 to 2100, at Kathmandu, under the default.
fn depth() {
    let provider = teistro_ephemeris_builtin::provider::Builtin::new();
    let sky = sky(&provider);
    let default = Sighting::new(CrescentCriterion::Odeh2006, CrescentZone::A).unwrap();
    let mut differ = 0;
    let mut compared = 0;
    let mut clamped = 0;
    for year in (1900..2100).step_by(2) {
        let (first, last) = (
            fixed_from_gregorian(year, 1, 1),
            fixed_from_gregorian(year + 1, 12, 31),
        );
        let near = sighted_months(
            &sky,
            kathmandu(),
            DeltaTModel::TableThenModel,
            default,
            (first, last),
        )
        .unwrap();
        let far = sighted_months(
            &sky,
            kathmandu(),
            DeltaTModel::TableThenModel,
            default,
            (first.plus_days(-365), last),
        )
        .unwrap();
        for month in near.months() {
            compared += 1;
            if !month.seen {
                clamped += 1;
            }
            if far
                .months()
                .iter()
                .find(|other| (other.year, other.month) == (month.year, month.month))
                .map(|other| other.first)
                != Some(month.first)
            {
                differ += 1;
            }
        }
    }
    println!(
        "DEPTH: {differ} of {compared} months differ between a chain opened 12 and 24 months back, 1900-2101; {clamped} began after a 29th unseen"
    );
}
