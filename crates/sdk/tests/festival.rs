//! Festival rules through the façade (`03-design/festival-rules.md` §4.3):
//! a year's observances over days founded once, each rule once, nothing
//! left unjudged, the widening declared, and a bad rule refused by the
//! place it holds in the request.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::Calendar;
use teistro::festival::{Edge, FestivalRule, FollowingRule, Guard, Predicate, Which, Window};
use teistro::muhurta::ActivityRules;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{
    AlmanacRequest, CalendarDate, Context, Ephemeris, FestivalPack, FestivalRequest,
    MuhurtaRequest, UtcOffset,
};

fn context() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

fn delhi() -> Place {
    Place::new(
        Latitude::literal(28.6139),
        Longitude::literal(77.209),
        Altitude::literal(216.0),
    )
}

fn year(year: i32) -> (CalendarDate, CalendarDate) {
    (
        CalendarDate::defined(Calendar::Gregorian, year, 1, 1),
        CalendarDate::defined(Calendar::Gregorian, year, 12, 31),
    )
}

#[test]
fn a_year_holds_each_shipped_rule_once_in_its_season() {
    let (from, to) = year(2026);
    let found = context()
        .almanac()
        .festivals(
            &from,
            &to,
            &delhi(),
            UtcOffset::literal(5, 30, 0),
            &FestivalRequest::from(FestivalPack::Nepal),
        )
        .unwrap();
    assert!(
        found.value.unjudged.is_empty(),
        "{:?}",
        found.value.unjudged
    );
    // Each in the Gregorian months its lunar month reaches.
    // The text's rules less the chaturthis at moonrise, which Nepal's
    // pack leaves out as its panchanga does.
    let (shipped, counted) = (
        FestivalRule::dharmasindhu()
            .into_iter()
            .filter(|rule| rule.at != Window::Moonrise)
            .collect::<Vec<_>>(),
        FollowingRule::nepal(),
    );
    let seasons = [
        ("RAMA_NAVAMI", 3..=4),
        ("JANMASHTAMI", 8..=9),
        ("VIJAYA_DASHAMI", 9..=10),
        ("LAKSHMI_PUJA", 10..=11),
        ("HARITALIKA", 8..=9),
        ("NAVARATRA_ARAMBHA", 9..=10),
        ("YAMA_DWITIYA", 10..=11),
        ("SHIVARATRI", 2..=3),
        ("RAKSHABANDHAN", 7..=8),
        ("BALI_PRATIPADA", 10..=11),
        ("HOLIKA", 2..=3),
        ("UPAKARMA_MADHYANDINA", 7..=8),
        ("UPAKARMA_TAITTIRIYA", 7..=8),
        ("UPAKARMA_SAMAVEDI", 8..=9),
        ("HOLI_HILLS", 2..=3),
        ("HOLI_TERAI", 2..=3),
    ];
    assert_eq!(
        seasons.iter().map(|(rule, _)| *rule).collect::<Vec<_>>(),
        shipped
            .iter()
            .map(|rule| rule.key.as_str())
            .chain(counted.iter().map(|rule| rule.key.as_str()))
            .collect::<Vec<_>>(),
        "every shipped rule has its season here"
    );
    for (rule, months) in seasons {
        let days: Vec<_> = found
            .value
            .observances
            .iter()
            .filter(|observance| observance.rule == rule)
            .collect();
        assert_eq!(days.len(), 1, "{rule}: {days:?}");
        assert!(
            months.contains(&days[0].day.month),
            "{rule} on {}",
            days[0].day
        );
    }
    // Two Ekadashis a lunar month, so 24 to 26 a year, the same number
    // under each observer.
    for rule in [
        "EKADASHI_VAISHNAVA",
        "EKADASHI_SMARTA",
        "EKADASHI_SMARTA_RENUNCIANT",
    ] {
        let fasts = found
            .value
            .ekadashis
            .iter()
            .filter(|fast| fast.rule == rule)
            .count();
        assert!((24..=26).contains(&fasts), "{rule}: {fasts}");
    }
    // The hills keep Holi on the fire's day, the Terai on the day after.
    let day = |rule: &str| {
        found
            .value
            .observances
            .iter()
            .find(|observance| observance.rule == rule)
            .map(|observance| observance.day.clone())
            .unwrap()
    };
    assert_eq!(day("HOLI_HILLS"), day("HOLIKA"));
    assert_eq!(day("HOLI_TERAI").day, day("HOLIKA").day + 1);
    let widened = found
        .provenance
        .applied_conventions
        .iter()
        .find(|convention| convention.knob == "festival.days")
        .unwrap();
    assert_eq!(widened.value, "GREGORIAN 2025-12-29..GREGORIAN 2027-01-03");
}

#[test]
fn a_rule_is_refused_by_its_place_in_the_request() {
    let (from, to) = year(2026);
    let mut rules = FestivalRule::dharmasindhu();
    rules[2].decide.push(Guard::new(
        [Predicate::Lasts {
            day: Which::Later,
            from: Edge::Sunset,
            ghatis: 0,
        }],
        teistro::festival::Choice::Later,
    ));
    let error = context()
        .almanac()
        .festivals(
            &from,
            &to,
            &delhi(),
            UtcOffset::literal(5, 30, 0),
            &FestivalRequest::new(rules),
        )
        .unwrap_err();
    assert_eq!(error.field(), Some("rules[2].decide.when.ghatis"));
}

#[test]
fn the_answer_alone_and_beside_its_days_agree_and_the_days_are_the_almanacs() {
    let sdk = context();
    let (from, to) = (
        CalendarDate::defined(Calendar::Gregorian, 2026, 10, 15),
        CalendarDate::defined(Calendar::Gregorian, 2026, 11, 10),
    );
    let clock = UtcOffset::literal(5, 30, 0);
    let asked = FestivalRequest::from(FestivalPack::Dharmasindhu);
    let alone = sdk
        .almanac()
        .festivals(&from, &to, &delhi(), clock, &asked)
        .unwrap();
    let beside = sdk
        .almanac()
        .festivals_with_days(&from, &to, &delhi(), clock, &asked)
        .unwrap();
    assert_eq!(beside.answer, alone);
    let rules: Vec<&str> = alone
        .value
        .observances
        .iter()
        .map(|o| o.rule.as_str())
        .collect();
    assert_eq!(
        rules,
        [
            "VIJAYA_DASHAMI",
            "SANKASHTA_CHATURTHI",
            "KARAKA_CHATURTHI",
            "LAKSHMI_PUJA",
            "BALI_PRATIPADA"
        ]
    );
    let (days, each) = sdk.almanac().of_each(&from, &to, &delhi(), clock).unwrap();
    assert_eq!(beside.days, days);
    assert_eq!(beside.day_hashes, each);

    // The input hash covers the rules: another rule is another input.
    let mut earlier = FestivalRule::dharmasindhu().remove(3);
    earlier.decide.clear();
    let other = sdk
        .almanac()
        .festivals(&from, &to, &delhi(), clock, &asked.with_rule(earlier))
        .unwrap();
    assert_ne!(other.provenance.input_hash, alone.provenance.input_hash);
}

#[test]
fn muhurta_and_festivals_asked_together_answer_as_each_alone_over_one_run_of_days() {
    let sdk = context();
    let (from, to) = (
        CalendarDate::defined(Calendar::Gregorian, 2026, 10, 15),
        CalendarDate::defined(Calendar::Gregorian, 2026, 11, 10),
    );
    let clock = UtcOffset::literal(5, 30, 0);
    let muhurta = MuhurtaRequest::new(ActivityRules::raman_marriage()).with_windows_on(2);
    let festivals = FestivalRequest::from(FestivalPack::Dharmasindhu);
    let asked = AlmanacRequest::new()
        .with_muhurta(muhurta.clone())
        .with_festivals(festivals.clone());
    let both = sdk
        .almanac()
        .asked(&from, &to, &delhi(), clock, &asked)
        .unwrap();
    let (days, each) = sdk.almanac().of_each(&from, &to, &delhi(), clock).unwrap();
    assert_eq!(both.days, days);
    assert_eq!(both.day_hashes, each);
    assert_eq!(
        both.muhurta,
        Some(
            sdk.almanac()
                .muhurta(&from, &to, &delhi(), clock, &muhurta)
                .unwrap()
        )
    );
    assert_eq!(
        both.festivals,
        Some(
            sdk.almanac()
                .festivals(&from, &to, &delhi(), clock, &festivals)
                .unwrap()
        )
    );

    // Asked for neither, it is the days alone.
    let neither = sdk
        .almanac()
        .asked(&from, &to, &delhi(), clock, &AlmanacRequest::new())
        .unwrap();
    assert_eq!(
        (neither.days, neither.muhurta, neither.festivals),
        (days, None, None)
    );
}

#[test]
fn a_kshaya_eleventh_pierced_at_sunrise_splits_the_three_observers() {
    // 2023's bright 11th of Bhadrapada at Delhi began after the 25th's
    // sunrise and ended before the 26th's: pierced at sunrise, holding no
    // sunrise of its own. The published almanac gives the Smarta fast the
    // 25th and the renunciant's and the Vaishnava's the 26th, as the text's
    // S8 does.
    let (from, to) = (
        CalendarDate::defined(Calendar::Gregorian, 2023, 9, 20),
        CalendarDate::defined(Calendar::Gregorian, 2023, 9, 30),
    );
    let found = context()
        .almanac()
        .festivals(
            &from,
            &to,
            &delhi(),
            UtcOffset::literal(5, 30, 0),
            &FestivalRequest::from(FestivalPack::Dharmasindhu),
        )
        .unwrap()
        .value;
    let days: Vec<(&str, u8)> = found
        .ekadashis
        .iter()
        .map(|fast| (fast.rule.as_str(), fast.day.day))
        .collect();
    assert_eq!(
        days,
        [
            ("EKADASHI_VAISHNAVA", 26),
            ("EKADASHI_SMARTA", 25),
            ("EKADASHI_SMARTA_RENUNCIANT", 26),
        ]
    );
    let fast = &found.ekadashis[0];
    assert_eq!(fast.pierced_at, Some(teistro::festival::Vedha::Sunrise));
    assert_eq!(fast.days.each_ref().map(|day| day.day), [25, 26]);
}

/// *Dharmasindhu* p. 9's Sankashta chaturthi, the dark 4th at moonrise,
/// over 2025 at Delhi: the twelve days a published almanac lists for New
/// Delhi (rank 3), and p. 76's Karaka chaturthi on Ashvina's, Karva Chauth
/// on 10 October. Each moonrise read is the day's panchanga's first, to the
/// bit.
#[test]
fn the_dark_fourth_at_moonrise_is_the_almanacs_sankashta_every_month() {
    let sdk = context();
    let (from, to) = year(2025);
    let clock = UtcOffset::literal(5, 30, 0);
    let found = sdk
        .almanac()
        .festivals(
            &from,
            &to,
            &delhi(),
            clock,
            &FestivalRequest::from(FestivalPack::Dharmasindhu),
        )
        .unwrap()
        .value;
    let on = |rule: &str| -> Vec<(u8, u8)> {
        found
            .observances
            .iter()
            .filter(|observance| observance.rule == rule)
            .map(|observance| (observance.day.month, observance.day.day))
            .collect()
    };
    assert!(found.unjudged.is_empty(), "{:?}", found.unjudged);
    assert_eq!(
        on("SANKASHTA_CHATURTHI"),
        [
            (1, 17),
            (2, 16),
            (3, 17),
            (4, 16),
            (5, 16),
            (6, 14),
            (7, 14),
            (8, 12),
            (9, 10),
            (10, 10),
            (11, 8),
            (12, 7)
        ]
    );
    assert_eq!(on("KARAKA_CHATURTHI"), [(10, 10)]);

    // The moonrise each day was judged by is its panchanga's first.
    let day = CalendarDate::defined(Calendar::Gregorian, 2025, 3, 17);
    let panchanga = sdk
        .almanac()
        .of(&day, &day, &delhi(), clock)
        .unwrap()
        .value
        .remove(0);
    let bhalachandra = found
        .observances
        .iter()
        .find(|observance| {
            observance.rule == "SANKASHTA_CHATURTHI"
                && (observance.day.month, observance.day.day) == (3, 17)
        })
        .unwrap();
    assert_eq!(bhalachandra.extents[0].window.from, panchanga.moon.rises[0]);
}

#[test]
fn a_doubled_jyeshtha_keeps_dashahara_in_its_adhika_month_however_short_the_range() {
    let sdk = Context::builder().nepal_official([]).build().unwrap();
    let kathmandu = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    // The range alone never leaves the month it asks about: the SDK reads
    // the lunation beside it, so a short range answers as a year does.
    let dashahara = |from: (i32, u8, u8), to: (i32, u8, u8)| {
        let answer = sdk
            .almanac()
            .festivals(
                &CalendarDate::defined(Calendar::Gregorian, from.0, from.1, from.2),
                &CalendarDate::defined(Calendar::Gregorian, to.0, to.1, to.2),
                &kathmandu,
                UtcOffset::literal(5, 45, 0),
                &FestivalRequest::from(FestivalPack::NepalPanchanga),
            )
            .unwrap()
            .value;
        assert!(
            answer
                .unjudged
                .iter()
                .all(|row| row.rule != "GANGA_DASHAHARA"),
            "{:?}",
            answer.unjudged
        );
        answer
            .observances
            .into_iter()
            .filter(|kept| kept.rule == "GANGA_DASHAHARA")
            .map(|kept| (kept.day.year, kept.day.month, kept.day.day))
            .collect::<Vec<_>>()
    };
    // VS 2083 p. 5 prints it on 26 May, the adhika Jyeshtha's bright 10th;
    // the nija month's, a lunation later, keeps none.
    assert_eq!(dashahara((2026, 5, 24), (2026, 5, 28)), [(2026, 5, 26)]);
    assert_eq!(dashahara((2026, 6, 20), (2026, 6, 28)), []);
    // VS 2082 doubles no month, so the nija Jyeshtha keeps it.
    assert_eq!(dashahara((2025, 6, 1), (2025, 6, 10)).len(), 1);
}

/// The rules `NEPAL_PANCHANGA` keeps in an adhika month alone, at
/// Kathmandu: none in VS 2082, which has no adhika month, and in VS 2083's
/// adhika Jyeshtha Padmini ekadashi (p. 5), Anala punhi (p. 5) and Parama
/// ekadashi (p. 6) on the printed days (C607, C694).
#[test]
fn the_adhika_month_alone_keeps_its_own_rules() {
    let sdk = Context::builder().nepal_official([]).build().unwrap();
    let kathmandu = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    let adhika = |from: (i32, u8, u8), to: (i32, u8, u8)| -> Vec<(String, (i32, u8, u8))> {
        sdk.almanac()
            .festivals(
                &CalendarDate::defined(Calendar::Gregorian, from.0, from.1, from.2),
                &CalendarDate::defined(Calendar::Gregorian, to.0, to.1, to.2),
                &kathmandu,
                UtcOffset::literal(5, 45, 0),
                &FestivalRequest::from(FestivalPack::NepalPanchanga),
            )
            .unwrap()
            .value
            .observances
            .into_iter()
            .filter(|kept| {
                ["PADMINI_EKADASHI", "ANALA_PUNHI", "PARAMA_EKADASHI"].contains(&kept.rule.as_str())
            })
            .map(|kept| (kept.rule, (kept.day.year, kept.day.month, kept.day.day)))
            .collect()
    };
    assert_eq!(adhika((2025, 4, 14), (2026, 4, 13)), []);
    assert_eq!(
        adhika((2026, 4, 14), (2027, 4, 13)),
        [
            ("PADMINI_EKADASHI".to_owned(), (2026, 5, 27)),
            ("ANALA_PUNHI".to_owned(), (2026, 5, 31)),
            ("PARAMA_EKADASHI".to_owned(), (2026, 6, 11)),
        ]
    );
}

/// A Gregorian day, as a test writes it.
type Day = (i32, u8, u8);

/// The days `NEPAL_PANCHANGA` keeps on a Bikram Sambat date at Kathmandu
/// under the committee's setup, each with its rule.
fn kept_dated(from: Day, to: Day) -> Vec<(String, Day)> {
    let sdk = Context::builder().nepal_official([]).build().unwrap();
    let kathmandu = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    sdk.almanac()
        .festivals(
            &CalendarDate::defined(Calendar::Gregorian, from.0, from.1, from.2),
            &CalendarDate::defined(Calendar::Gregorian, to.0, to.1, to.2),
            &kathmandu,
            UtcOffset::literal(5, 45, 0),
            &FestivalRequest::from(FestivalPack::NepalPanchanga),
        )
        .unwrap()
        .value
        .dated
        .into_iter()
        .map(|kept| (kept.rule, (kept.day.year, kept.day.month, kept.day.day)))
        .collect()
}

fn printed(rows: &[(&str, Day)]) -> Vec<(String, Day)> {
    rows.iter()
        .map(|(rule, day)| ((*rule).to_owned(), *day))
        .collect()
}

/// The days `NEPAL_PANCHANGA` keeps on a Bikram Sambat date, over VS 2082
/// at Kathmandu: exactly the rows read off the committee's print, each
/// sankranti on its month's first day (pp. 2-24), the six national days,
/// Tamu Lhosar, Walke parva, the sixteen festivals of a solar date
/// (Bisket and the Baisakh jatras, Shrawan's, Bhadra's, Asoj's and Magh's
/// firsts), and Bhumiraj and the Bhumi puja on and three days after the
/// Sun's entry into Ardra (p. 6).
#[test]
fn the_dated_days_are_the_committees_printed_rows() {
    let kept = kept_dated;
    assert_eq!(
        kept((2025, 4, 14), (2026, 4, 13)),
        printed(&[
            ("MESHA_SANKRANTI", (2025, 4, 14)),
            ("TOKHA_SAPAN_TIRTHA_SNAN", (2025, 4, 14)),
            ("YOSIN_PATAN", (2025, 4, 14)),
            ("BHAKTAPUR_MAHAKALI_MAHALAXMI_JATRA", (2025, 4, 15)),
            ("TOKHA_CHANDESHWARI_JATRA", (2025, 4, 15)),
            ("THIMI_BALKUMARI_JATRA", (2025, 4, 15)),
            ("BHAKTAPUR_BRAHMAYANI_JATRA", (2025, 4, 16)),
            ("BHAKTAPUR_CHHUMA_GANESH_JATRA", (2025, 4, 17)),
            ("BISKET_JATRA_ENDS", (2025, 4, 18)),
            ("LOKTANTRA_DIWAS", (2025, 4, 24)),
            ("VRISHABHA_SANKRANTI", (2025, 5, 15)),
            ("MITHUNA_SANKRANTI", (2025, 6, 15)),
            ("BHUMIRAJ", (2025, 6, 22)),
            ("BHUMI_PUJA", (2025, 6, 25)),
            ("KARKATA_SANKRANTI", (2025, 7, 17)),
            ("KANDAKARAK_PUJA", (2025, 7, 17)),
            ("KHEER_KHANE_DIN", (2025, 7, 31)),
            ("SIMHA_SANKRANTI", (2025, 8, 17)),
            ("WALKE_PARVA", (2025, 8, 17)),
            ("KIRTIPUR_BAGH_BHAIRAV_JATRA", (2025, 8, 17)),
            ("KANYA_SANKRANTI", (2025, 9, 17)),
            ("VISHWAKARMA_PUJA", (2025, 9, 17)),
            ("CONSTITUTION_DAY", (2025, 9, 19)),
            ("TULA_SANKRANTI", (2025, 10, 18)),
            ("VRISHCHIKA_SANKRANTI", (2025, 11, 17)),
            ("DHANU_SANKRANTI", (2025, 12, 16)),
            ("TAMU_LHOSAR", (2025, 12, 30)),
            ("PRITHVI_JAYANTI", (2026, 1, 11)),
            ("MAKARA_SANKRANTI", (2026, 1, 15)),
            ("MAGHI_THARU", (2026, 1, 15)),
            ("MAKAR_SNAN", (2026, 1, 15)),
            ("DEVGHAT_MAKAR_SNAN", (2026, 1, 15)),
            ("EARTHQUAKE_SAFETY_DAY", (2026, 1, 16)),
            ("MARTYRS_DAY", (2026, 1, 30)),
            ("KUMBHA_SANKRANTI", (2026, 2, 13)),
            ("DEMOCRACY_DAY", (2026, 2, 19)),
            ("MINA_SANKRANTI", (2026, 3, 15)),
            ("BISKET_JATRA_BEGINS", (2026, 4, 10)),
        ])
    );
}

/// VS 2083's rows checked against the same pack (pp. 2-3, 9-13, 20-23, 27).
#[test]
fn the_dated_days_are_the_second_editions_printed_rows() {
    let kept = kept_dated;
    // VS 2083 pp. 2-3: Bisket and the Baisakh jatras, Thimi's and Chhuma
    // Ganesh's continuing their rows after Ⓑ and Ⓒ.
    assert_eq!(
        kept((2026, 4, 14), (2026, 4, 18)),
        printed(&[
            ("MESHA_SANKRANTI", (2026, 4, 14)),
            ("TOKHA_SAPAN_TIRTHA_SNAN", (2026, 4, 14)),
            ("YOSIN_PATAN", (2026, 4, 14)),
            ("BHAKTAPUR_MAHAKALI_MAHALAXMI_JATRA", (2026, 4, 15)),
            ("TOKHA_CHANDESHWARI_JATRA", (2026, 4, 15)),
            ("THIMI_BALKUMARI_JATRA", (2026, 4, 15)),
            ("BHAKTAPUR_BRAHMAYANI_JATRA", (2026, 4, 16)),
            ("BHAKTAPUR_CHHUMA_GANESH_JATRA", (2026, 4, 17)),
            ("BISKET_JATRA_ENDS", (2026, 4, 18)),
        ])
    );
    // VS 2083 p. 7: भूमिरज on Asar 8, the day the Sun enters Ardra at
    // 21:11, and भूमिपूजा on Asar 11; a range opening after the entry
    // still finds the puja counted from it.
    assert_eq!(
        kept((2026, 6, 20), (2026, 6, 26)),
        printed(&[("BHUMIRAJ", (2026, 6, 22)), ("BHUMI_PUJA", (2026, 6, 25))])
    );
    assert_eq!(
        kept((2026, 6, 25), (2026, 6, 25)),
        printed(&[("BHUMI_PUJA", (2026, 6, 25))])
    );
    // VS 2083 pp. 9-13: Shrawan 1 and 15, Bhadra 1 (वल्केपर्व on
    // भाद्रसङ्क्रान्ति) and Asoj 1.
    assert_eq!(
        kept((2026, 7, 17), (2026, 9, 17))
            .into_iter()
            .filter(|(rule, _)| !rule.ends_with("SANKRANTI"))
            .collect::<Vec<_>>(),
        printed(&[
            ("KANDAKARAK_PUJA", (2026, 7, 17)),
            ("KHEER_KHANE_DIN", (2026, 7, 31)),
            ("WALKE_PARVA", (2026, 8, 17)),
            ("KIRTIPUR_BAGH_BHAIRAV_JATRA", (2026, 8, 17)),
            ("VISHWAKARMA_PUJA", (2026, 9, 17)),
        ])
    );
    assert_eq!(
        kept((2026, 12, 30), (2027, 2, 19)),
        printed(&[
            ("TAMU_LHOSAR", (2026, 12, 30)),
            ("PRITHVI_JAYANTI", (2027, 1, 11)),
            ("MAKARA_SANKRANTI", (2027, 1, 15)),
            ("MAGHI_THARU", (2027, 1, 15)),
            ("MAKAR_SNAN", (2027, 1, 15)),
            ("DEVGHAT_MAKAR_SNAN", (2027, 1, 15)),
            ("EARTHQUAKE_SAFETY_DAY", (2027, 1, 16)),
            ("MARTYRS_DAY", (2027, 1, 30)),
            ("KUMBHA_SANKRANTI", (2027, 2, 13)),
            ("DEMOCRACY_DAY", (2027, 2, 19)),
        ])
    );
    // VS 2083 p. 27: Bisket begins on Chaitra's fourth-last day.
    assert_eq!(
        kept((2027, 4, 10), (2027, 4, 10)),
        printed(&[("BISKET_JATRA_BEGINS", (2027, 4, 10))])
    );
}

/// A rule's key, the day it was kept, and that day's Tibetan month and day.
type Kept = (String, (i32, u8, u8), (u8, u8));

/// Rules kept on a Tibetan date, read as a binding writes them, fall on
/// the days Dharamsala printed: Saga Dawa Duchen 2016 in leap month 4,
/// Chokhor Duchen 2024 on the first of leap month 6's two 4ths, and
/// Chotrul Duchen 2018, its 15th skipped, on the 14th (FPMT) or the 16th
/// (Tushita) as the rule asks (`calendars/tibetan.md`).
#[test]
fn a_rule_on_a_tibetan_date_keeps_the_printed_day() {
    let sdk = context();
    let request = FestivalRequest::from_json(
        r#"{"rules": [
            {"key": "SAGA_DAWA_DUCHEN", "source": "a test", "civilDate": {"calendar": "TIBETAN_PHUGPA", "month": 4, "day": 15}},
            {"key": "CHOKHOR_DUCHEN", "source": "a test", "civilDate": {"calendar": "calendar.TIBETAN_PHUGPA", "month": 6, "day": 4}},
            {"key": "CHOTRUL_DUCHEN", "source": "a test", "civilDate": {"calendar": "TIBETAN_PHUGPA", "month": 1, "day": 15}},
            {"key": "CHOTRUL_DUCHEN_AFTER", "source": "a test", "civilDate": {"calendar": "TIBETAN_PHUGPA", "month": 1, "day": 15, "skipped": "AFTER"}}
        ]}"#,
    )
    .unwrap();
    let kept = |from: (i32, u8, u8), to: (i32, u8, u8)| -> Vec<Kept> {
        sdk.almanac()
            .festivals(
                &CalendarDate::defined(Calendar::Gregorian, from.0, from.1, from.2),
                &CalendarDate::defined(Calendar::Gregorian, to.0, to.1, to.2),
                &delhi(),
                UtcOffset::literal(5, 30, 0),
                &request,
            )
            .unwrap()
            .value
            .dated
            .into_iter()
            .map(|kept| {
                (
                    kept.rule,
                    (kept.day.year, kept.day.month, kept.day.day),
                    (kept.on.month, kept.on.day),
                )
            })
            .collect()
    };
    assert_eq!(
        kept((2016, 5, 18), (2016, 6, 22)),
        [("SAGA_DAWA_DUCHEN".to_owned(), (2016, 5, 21), (4, 15))]
    );
    assert_eq!(
        kept((2024, 7, 5), (2024, 8, 10)),
        [("CHOKHOR_DUCHEN".to_owned(), (2024, 7, 9), (6, 4))]
    );
    assert_eq!(
        kept((2018, 2, 27), (2018, 3, 4)),
        [
            ("CHOTRUL_DUCHEN".to_owned(), (2018, 3, 1), (1, 14)),
            ("CHOTRUL_DUCHEN_AFTER".to_owned(), (2018, 3, 2), (1, 16)),
        ]
    );
    let refused = FestivalRequest::from_json(
        r#"{"rules": [{"key": "X", "source": "a test", "civilDate": {"calendar": "BIKRAM_SAMBAT", "month": 1, "day": 1, "skipped": "BEFORE"}}]}"#,
    )
    .unwrap_err();
    assert_eq!(
        refused.field(),
        Some("festivals.rules[0].civilDate.skipped")
    );
}

/// A rule on the Sun's entry into a nakshatra, written as a binding writes
/// it: Nautapa begins the day the Sun enters Rohini, which the panchangas
/// printed in India give as 25 May 2025 (about 03:27 IST).
#[test]
fn a_rule_on_the_suns_entry_reads_as_a_binding_writes_it() {
    let request = FestivalRequest::from_json(
        r#"{"rules": [{"key": "NAUTAPA", "source": "a test", "sunEnters": {"nakshatra": "nakshatra.ROHINI"}}]}"#,
    )
    .unwrap();
    let kept: Vec<(String, u8)> = context()
        .almanac()
        .festivals(
            &CalendarDate::defined(Calendar::Gregorian, 2025, 5, 20),
            &CalendarDate::defined(Calendar::Gregorian, 2025, 5, 30),
            &delhi(),
            UtcOffset::literal(5, 30, 0),
            &request,
        )
        .unwrap()
        .value
        .dated
        .into_iter()
        .map(|kept| (kept.rule, kept.day.day))
        .collect();
    assert_eq!(kept, [("NAUTAPA".to_owned(), 25)]);
    let refused = FestivalRequest::from_json(
        r#"{"rules": [{"key": "X", "source": "a test", "sunEnters": {"nakshatra": "ROHINI", "days": 16}}]}"#,
    )
    .unwrap_err();
    assert_eq!(refused.field(), Some("festivals.rules[0].sunEnters.days"));
}
