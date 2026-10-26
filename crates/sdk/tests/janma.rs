//! BPHS ch. 85's and Phaladeepika ch. 13 v. 8's inauspicious births read from
//! charts the SDK computes: an hour at a time through twenty days, in which a
//! Krishna Chaturdashi passes through its six parts, the Sun enters
//! Capricorn and the Moon crosses some twenty nakshatras.
//!
//! The rules crate's corpus records no limb's ghatikas and no sankranti, so
//! the parts and the sankranti days are held here, where the chart carries
//! both.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use std::collections::BTreeSet;

use teistro::catalogue::Calendar;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::rules::shipped;
use teistro::{
    CalendarDate, ChartRequest, Context, Ephemeris, RuleRequest, ShippedRules, UtcOffset,
};

/// The packs' rules that a moment's limbs decide, read at each hour of the
/// twenty days from JD 2 451 545 (2000 January 1, noon) at Kathmandu, with
/// the evils at birth among which v. 8's three that kill the child stand:
/// the keys present at each.
fn hours() -> Vec<BTreeSet<String>> {
    let sdk = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap();
    let pack: Vec<_> = shipped::nabhasas()
        .iter()
        .filter(|rule| rule.category == "ashubha-janma")
        .cloned()
        .collect();
    assert_eq!(pack.len(), 96);
    let set = RuleRequest::shipped([ShippedRules::Arishtas])
        .with_rules(pack)
        .rule_set()
        .unwrap();
    let place = Place::new(
        Latitude::try_new(27.7172).unwrap(),
        Longitude::try_new(85.324).unwrap(),
        Altitude::try_new(0.0).unwrap(),
    );
    let instants: Vec<JulianDay<Utc>> = (0..20 * 24)
        .map(|hour| JulianDay::literal(2_451_545.0 + f64::from(hour) / 24.0))
        .collect();
    sdk.chart()
        .readings_with_rules(&instants, &ChartRequest::at(place, UtcOffset::UTC), &set)
        .unwrap()
        .value
        .iter()
        .map(|(_, reading)| {
            reading
                .present
                .iter()
                .map(|present| present.rule.key.clone())
                .collect()
        })
        .collect()
}

#[test]
fn chaturdashi_s_parts_follow_one_another_and_a_sankranti_names_its_day() {
    let hours = hours();
    let parts: Vec<u8> = hours
        .iter()
        .map(|present| {
            let held: Vec<u8> = (1..=6)
                .filter(|part| present.contains(&format!("BPHS_KRISHNA_CHATURDASHI_PART_{part}")))
                .collect();
            // Ch. 87's six parts divide the tithi: at most one holds.
            assert!(held.len() <= 1, "{held:?}");
            held.first().copied().unwrap_or(0)
        })
        .collect();
    // The Krishna Chaturdashi of 4 January walks its parts from the first
    // to the sixth without skipping or returning.
    let mut runs: Vec<Vec<u8>> = Vec::new();
    for (at, part) in parts.iter().enumerate() {
        if *part == 0 {
            continue;
        }
        if at == 0 || parts[at - 1] == 0 {
            runs.push(Vec::new());
        }
        let run = runs.last_mut().unwrap();
        if run.last() != Some(part) {
            run.push(*part);
        }
    }
    assert_eq!(runs.len(), 1, "{runs:?}");
    for run in &runs {
        assert_eq!(run.as_slice(), [1, 2, 3, 4, 5, 6]);
    }

    // The Sun enters Capricorn: the sankranti day holds one named
    // sankranti, the one its weekday names, and no other day does.
    let sankranti: Vec<&str> = hours
        .iter()
        .flat_map(|present| present.iter())
        .filter(|key| key.ends_with("_SANKRANTI"))
        .map(String::as_str)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    // The Sun entered sidereal Capricorn on Friday 14 January 2000 at
    // Kathmandu: ch. 90 v. 1 names a Friday's sankranti Mishra.
    assert_eq!(sankranti, ["BPHS_BIRTH_ON_THE_MISHRA_SANKRANTI"]);
    let days = hours
        .iter()
        .filter(|present| present.iter().any(|key| key.ends_with("_SANKRANTI")))
        .count();
    assert!(
        (20..=28).contains(&days),
        "{days} hours on a sankranti's day"
    );
    for present in &hours {
        assert!(
            present
                .iter()
                .filter(|key| key.ends_with("_SANKRANTI"))
                .count()
                <= 1
        );
    }

    // The Amavasya of 6 January passes too, about a day long.
    let amavasya = hours
        .iter()
        .filter(|present| present.contains("BPHS_BIRTH_ON_AMAVASYA"))
        .count();
    assert!(
        (18..=28).contains(&amavasya),
        "{amavasya} hours of Amavasya"
    );
}

/// An avama birth (BPHS ch. 85 v. 3, C759) is one in a tithi the almanac
/// of the same days calls kshaya, touching no sunrise, read back hour by
/// hour: the rule holds exactly where a kshaya tithi's span holds the hour.
#[test]
fn an_avama_birth_is_one_in_a_tithi_the_almanac_calls_kshaya() {
    let hours = hours();
    let sdk = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap();
    let place = Place::new(
        Latitude::try_new(27.7172).unwrap(),
        Longitude::try_new(85.324).unwrap(),
        Altitude::try_new(0.0).unwrap(),
    );
    let date = |day| CalendarDate::defined(Calendar::Gregorian, 2000, 1, day);
    let days = sdk
        .almanac()
        .of(&date(1), &date(22), &place, UtcOffset::UTC)
        .unwrap()
        .value;
    let kshaya: Vec<(f64, f64)> = days
        .iter()
        .flat_map(|day| {
            day.limbs
                .tithi
                .iter()
                .filter(|span| day.sunrises(span).is_kshaya())
                .map(|span| (span.whole.from.get(), span.whole.to.get()))
        })
        .collect();
    let expected: Vec<usize> = (0..hours.len())
        .filter(|hour| {
            let at = 2_451_545.0 + f64::from(u32::try_from(*hour).unwrap()) / 24.0;
            kshaya.iter().any(|(from, to)| *from <= at && at < *to)
        })
        .collect();
    let avama = held(&hours, "BPHS_BIRTH_IN_AVAMA");
    // Twenty days hold a kshaya tithi, so the comparison is not of nothing.
    assert!(!expected.is_empty(), "{kshaya:?}");
    assert_eq!(avama, expected);
}

/// The hours at which `key` holds, as indexes into the twenty days.
fn held(hours: &[BTreeSet<String>], key: &str) -> Vec<usize> {
    hours
        .iter()
        .enumerate()
        .filter(|(_, present)| present.contains(key))
        .map(|(at, _)| at)
        .collect()
}

/// The runs of consecutive hours in `at`.
fn runs(at: &[usize]) -> Vec<usize> {
    let mut runs: Vec<usize> = Vec::new();
    for (index, hour) in at.iter().enumerate() {
        if index > 0 && at[index - 1] + 1 == *hour {
            *runs.last_mut().unwrap() += 1;
        } else {
            runs.push(1);
        }
    }
    runs
}

#[test]
fn phaladeepika_s_eighth_verse_reads_the_moon_s_pada_and_the_star_s_poison() {
    let hours = hours();

    // Each star's four ghatis of poison are a fifteenth of it, under two
    // hours, once in every star the Moon crosses.
    let visha = held(&hours, "PHALADEEPIKA_BIRTH_IN_VISHAGHATI");
    let windows = runs(&visha);
    assert_eq!(windows.len(), 20, "{visha:?}");
    assert!(windows.iter().all(|hours| *hours <= 2), "{windows:?}");
    assert_eq!(visha.len(), 32);

    // Dinamrityu and Dinaroga count by day alone. The first hour is noon
    // UTC, 17:45 at Kathmandu, and January's day there runs from about
    // seven to a little after five.
    for key in [
        "PHALADEEPIKA_BIRTH_IN_DINAMRITYU",
        "PHALADEEPIKA_BIRTH_IN_DINAROGA",
    ] {
        let at = held(&hours, key);
        assert!(!at.is_empty(), "{key}");
        for hour in at {
            let local = (hour * 4 + 71) % 96;
            assert!((28..=68).contains(&local), "{key} at hour {hour}");
        }
    }

    // The padas of Pushya, Purva Ashadha and Chitra follow one another:
    // Purva Ashadha whole by the new Moon of 6 January, then Pushya to its
    // third by the full Moon as the days end.
    let padas: Vec<u8> = hours
        .iter()
        .map(|present| {
            (1..=4)
                .find(|pada| {
                    present.contains(&format!(
                        "PHALADEEPIKA_PUSHYA_PURVA_ASHADHA_CHITRA_PADA_{pada}"
                    ))
                })
                .unwrap_or(0)
        })
        .collect();
    let mut stars: Vec<Vec<u8>> = Vec::new();
    for (at, pada) in padas.iter().enumerate() {
        if *pada == 0 {
            continue;
        }
        if at == 0 || padas[at - 1] == 0 {
            stars.push(Vec::new());
        }
        let star = stars.last_mut().unwrap();
        if star.last() != Some(pada) {
            star.push(*pada);
        }
    }
    assert_eq!(stars, [vec![1, 2, 3, 4], vec![1, 2, 3]]);

    // Mula's four padas pass under a lagna no benefic touches; Ashlesha's
    // fall outside the twenty days.
    let mula: Vec<&str> = hours
        .iter()
        .flat_map(|present| present.iter())
        .filter(|key| {
            key.starts_with("PHALADEEPIKA_MULA") || key.starts_with("PHALADEEPIKA_ASHLESHA")
        })
        .map(String::as_str)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    assert_eq!(
        mula,
        [
            "PHALADEEPIKA_MULA_PADA_1_FATHER",
            "PHALADEEPIKA_MULA_PADA_2_MOTHER",
            "PHALADEEPIKA_MULA_PADA_3_LINE",
            "PHALADEEPIKA_MULA_PADA_4_PROSPERITY",
        ]
    );
}

/// The almanac's varjyam and the arishta pack's vishaghati are one table:
/// Muhurta Chintamani's, which Ojha repeats (C165, C394). Each of the
/// pack's 27 windows is that star's start plus one to four sixtieths.
#[test]
fn the_packs_vishaghati_is_the_almanacs_varjyam() {
    use teistro::catalogue::Nakshatra;
    use teistro::omen::varjyam_start;
    use teistro::settings::Varjyam;

    let rule = shipped::arishtas()
        .iter()
        .find(|rule| rule.key == "PHALADEEPIKA_BIRTH_IN_VISHAGHATI")
        .unwrap();
    let json = serde_json::to_value(&rule.conditions).unwrap();
    let windows = json[0]["conditions"].as_array().unwrap();
    assert_eq!(windows.len(), 27);
    for window in windows {
        let star = window["conditions"][0]["nakshatras"][0].as_str().unwrap();
        let nakshatra = Nakshatra::ALL
            .iter()
            .copied()
            .find(|n| serde_json::to_value(n).unwrap() == star)
            .unwrap();
        let start = u64::from(varjyam_start(Varjyam::MuhurtaChintamani, nakshatra).unwrap());
        assert_eq!(
            window["conditions"][1]["parts"],
            serde_json::json!([start + 1, start + 2, start + 3, start + 4]),
            "{star}"
        );
    }
}

/// Every hour the pack finds a birth in the vishaghati falls in a varjyam
/// the almanac reports for the same days, and every hour inside one is
/// such a birth: the rule language's sixtieths and the almanac's windows
/// agree on the sky.
#[test]
fn every_vishaghati_hour_is_inside_an_almanac_varjyam() {
    use teistro::CalendarDate;
    use teistro::catalogue::Calendar;

    let hours = hours();
    let visha = held(&hours, "PHALADEEPIKA_BIRTH_IN_VISHAGHATI");
    let sdk = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap();
    let place = Place::new(
        Latitude::try_new(27.7172).unwrap(),
        Longitude::try_new(85.324).unwrap(),
        Altitude::try_new(0.0).unwrap(),
    );
    let days = sdk
        .almanac()
        .of(
            &CalendarDate::defined(Calendar::Gregorian, 1999, 12, 31),
            &CalendarDate::defined(Calendar::Gregorian, 2000, 1, 22),
            &place,
            UtcOffset::UTC,
        )
        .unwrap()
        .value;
    let windows: Vec<_> = days
        .iter()
        .flat_map(|day| day.omens.varjyam.iter().map(|span| span.whole))
        .collect();
    assert!(windows.len() >= 20, "{}", windows.len());
    for hour in 0..20 * 24 {
        let at = JulianDay::literal(2_451_545.0 + f64::from(hour) / 24.0);
        let inside = windows.iter().any(|window| window.contains(at));
        assert_eq!(
            inside,
            visha.contains(&usize::try_from(hour).unwrap()),
            "hour {hour}"
        );
    }
}
