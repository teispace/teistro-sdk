//! A day's observances in order of significance (`03-design/festival-metadata.md`
//! §7): Kartik 2083's days as Nepal's national panchanga sets them, over its
//! own sky at Kathmandu, under the default order, the rule order, a
//! request's own tier and a least tier.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a test fails by panicking and indexes what it built"
)]

use teistro::catalogue::Calendar;
use teistro::festival::{MetaFilter, MetaPatch, Significance};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{
    CalendarDate, CellsRequest, Context, DayClock, DayOrder, Ephemeris, FestivalPack,
    FestivalRequest, UtcOffset,
};

/// Each cell's festivals from Kartik 5 to 23, 2083 (22 October to 9
/// November 2026), by Gregorian day of the month.
fn kartik(festivals: FestivalRequest) -> Vec<(u8, Vec<String>)> {
    // The official reckoning as a consumer builds it: the Surya Siddhanta
    // with a modern ephemeris for the sunrise and the star planets.
    let sdk = Context::builder()
        .nepal_official([Ephemeris::Builtin])
        .build()
        .unwrap();
    let request = CellsRequest {
        first: CalendarDate::defined(Calendar::Gregorian, 2026, 10, 22),
        last: CalendarDate::defined(Calendar::Gregorian, 2026, 11, 9),
        place: Place::new(
            Latitude::literal(27.7172),
            Longitude::literal(85.324),
            Altitude::literal(1400.0),
        ),
        clock: DayClock::Offset(UtcOffset::literal(5, 45, 0)),
        festivals: Some(festivals),
    };
    sdk.almanac()
        .cells(&request)
        .unwrap()
        .value
        .cells
        .into_iter()
        .map(|cell| (cell.date.day, cell.festivals))
        .collect()
}

/// The festivals of one day of [`kartik`]'s.
fn on(cells: &[(u8, Vec<String>)], day: u8) -> Vec<&str> {
    let (_, keys) = cells.iter().find(|(held, _)| *held == day).unwrap();
    keys.iter().map(String::as_str).collect()
}

/// That `first` comes before `then` on a day.
fn before(keys: &[&str], first: &str, then: &str) -> bool {
    let at = |key| keys.iter().position(|held| *held == key);
    matches!((at(first), at(then)), (Some(a), Some(b)) if a < b)
}

#[test]
fn kartik_2083_comes_out_as_the_print_sets_it() {
    let cells = kartik(FestivalRequest::from(FestivalPack::NepalPanchanga));
    // Kartik 5: the Papankusha ekadashi's fast before the Annapurna jatra.
    assert_eq!(on(&cells, 22)[0], "PAPANKUSHA_EKADASHI");
    assert!(before(
        &on(&cells, 22),
        "PAPANKUSHA_EKADASHI",
        "ANNAPURNA_JATRA"
    ));
    // Kartik 7: Khanchi's sarai, kept there, before the UN day.
    assert_eq!(on(&cells, 24)[0], "KHANCHI_SARAI_PARVA");
    // Kartik 9: Kartik snan begins with the sky lamps (both open a
    // stretch), then Argha's sarai and Katin punhi, and Intersex Awareness
    // Day last (C2346).
    assert_eq!(
        on(&cells, 26),
        [
            "KARTIK_SNAN_BEGINS",
            "AKASHDEEPADAN_ARAMBHA",
            "ARGHA_SARAI_PARVA",
            "KATIN_PUNHI",
            "INTERSEX_AWARENESS_DAY"
        ]
    );
    // Kartik 19: Rama ekadashi before Govatsa dwadashi.
    assert_eq!(on(&cells, 5)[0], "RAMA_EKADASHI");
    assert!(before(&on(&cells, 5), "RAMA_EKADASHI", "GOVATSA_DWADASHI"));
    // Kartik 22: Lakshmi Puja and Kukur Tihar, both national inside the
    // notice's Tihar holiday, Lakshmi Puja first by Nepal's names'
    // rank, the maintainer's order over the print's (SR-145, C2303).
    assert_eq!(
        on(&cells, 8)[..3],
        ["LAKSHMI_PUJA", "KUKUR_TIHAR", "NARAK_CHATURDASHI"]
    );
    // Kartik 23: the cows' worship, a national day of the notice's Tihar
    // holiday, before Somabare aunsi.
    assert_eq!(on(&cells, 9)[0], "GAI_PUJA");
    assert!(before(&on(&cells, 9), "GAI_PUJA", "SOMAVATI_AMAVASYA"));
}

#[test]
fn the_rule_order_is_the_answer_s_own() {
    let cells =
        kartik(FestivalRequest::from(FestivalPack::NepalPanchanga).with_order(DayOrder::RuleOrder));
    assert_eq!(
        on(&cells, 26),
        [
            "ARGHA_SARAI_PARVA",
            "KARTIK_SNAN_BEGINS",
            "KATIN_PUNHI",
            "AKASHDEEPADAN_ARAMBHA",
            "INTERSEX_AWARENESS_DAY"
        ]
    );
}

#[test]
fn a_request_sets_its_own_tier_and_keeps_a_least_one() {
    let mine = FestivalRequest::from(FestivalPack::NepalPanchanga).with_meta(
        "LAKSHMI_PUJA",
        &MetaPatch {
            significance: Some(Significance::Minor),
            ..MetaPatch::default()
        },
    );
    // Lakshmi Puja, set minor, follows Kukur Tihar.
    let day = kartik(mine);
    assert_eq!(on(&day, 8)[0], "KUKUR_TIHAR");
    assert!(before(&on(&day, 8), "KUKUR_TIHAR", "LAKSHMI_PUJA"));

    let least = FestivalRequest::from(FestivalPack::NepalPanchanga).keeping(MetaFilter {
        min_significance: Some(Significance::Period),
        ..MetaFilter::default()
    });
    let kept = kartik(least);
    assert_eq!(
        on(&kept, 26),
        ["KARTIK_SNAN_BEGINS", "AKASHDEEPADAN_ARAMBHA"]
    );
    assert_eq!(on(&kept, 8), ["LAKSHMI_PUJA", "KUKUR_TIHAR"]);
}
