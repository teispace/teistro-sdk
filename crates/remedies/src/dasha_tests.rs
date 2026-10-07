//! The antardaśā table held to the counts read off the pages, and to the
//! order the chapters themselves follow.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the table they read"
)]

use teistro_core::catalogue::Graha;

use crate::{Condition, Remedy, dasha_shanti, dasha_shantis};

/// The Vimśottarī order, which both the chapters (37 to 45) and the
/// antardaśās within each follow.
const ORDER: [Graha; 9] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Rahu,
    Graha::Jupiter,
    Graha::Saturn,
    Graha::Mercury,
    Graha::Ketu,
    Graha::Venus,
];

#[test]
fn every_chapter_runs_its_nine_antardashas_in_vimshottari_order() {
    let rows = dasha_shantis();
    assert_eq!(rows.len(), 81);
    for (at, row) in rows.iter().enumerate() {
        let (chapter, within) = (at / 9, at % 9);
        assert_eq!(row.mahadasha, ORDER[chapter], "row {at}");
        assert_eq!(row.antardasha, ORDER[(chapter + within) % 9], "row {at}");
        assert_eq!(usize::from(row.chapter), 37 + chapter, "row {at}");
    }
    assert!(rows.windows(2).all(|pair| pair[0].page <= pair[1].page));
    assert_eq!(dasha_shanti(Graha::Uranus, Graha::Sun), None);
}

#[test]
fn the_counts_are_the_pages_counts() {
    let rows = dasha_shantis();
    let with = |condition: Condition| {
        rows.iter()
            .filter(|row| row.conditions.first() == Some(&condition))
            .count()
    };
    assert_eq!(with(Condition::LordOfSecondOrSeventh), 53);
    assert_eq!(with(Condition::LordOfSeventh), 4);
    assert_eq!(with(Condition::InSecondOrSeventh), 10);
    assert_eq!(
        rows.iter().filter(|row| !row.remedies.is_empty()).count(),
        79
    );

    let naming = |remedy: Remedy| {
        rows.iter()
            .filter(|row| row.remedies.contains(&remedy))
            .count()
    };
    assert_eq!(naming(Remedy::MrityunjayaJapa), 17);
    assert_eq!(naming(Remedy::MahaMrityunjayaJapa), 3);
    assert_eq!(naming(Remedy::ChagaDana), 12);
    assert_eq!(naming(Remedy::ShvetaGoMahishi), 11);
    assert_eq!(naming(Remedy::KrishnaGoMahishi), 7);
    assert_eq!(naming(Remedy::VishnuSahasranama), 10);
    assert_eq!(naming(Remedy::AnadvanDana), 8);
    assert_eq!(naming(Remedy::ShivaSahasranama), 6);
    assert_eq!(naming(Remedy::Shanti), 10);
}

#[test]
fn what_the_page_leaves_out_stays_out() {
    // Venus/Moon prints neither a condition nor a rite; Venus/Mars names
    // the evil and moves on to Rahu without one.
    let venus_moon = dasha_shanti(Graha::Venus, Graha::Moon).unwrap();
    assert_eq!(
        (venus_moon.conditions, venus_moon.remedies),
        (&[][..], &[][..])
    );
    let venus_mars = dasha_shanti(Graha::Venus, Graha::Mars).unwrap();
    assert_eq!(venus_mars.conditions, [Condition::LordOfSecondOrSeventh]);
    assert_eq!(venus_mars.remedies, []);
    // Ketu/Sun's darśa-śānti stands as both prints read it, not as a
    // Sun śānti (C350); Moon/Venus gives silver, not a she-buffalo.
    assert!(
        dasha_shanti(Graha::Ketu, Graha::Sun)
            .unwrap()
            .remedies
            .contains(&Remedy::DarshaShanti)
    );
    assert_eq!(
        dasha_shanti(Graha::Moon, Graha::Venus).unwrap().remedies,
        [Remedy::RudraJapa, Remedy::ShvetaGoRajata]
    );
    // Jupiter/Moon is the 1923 print's 2nd-and-6th lord, not a 7th lord
    // read into it (C349).
    assert_eq!(
        dasha_shanti(Graha::Jupiter, Graha::Moon)
            .unwrap()
            .conditions,
        [Condition::LordOfSecondAndSixth]
    );
}

#[test]
fn the_keys_are_what_serde_writes() {
    let read = serde_json::to_value(dasha_shanti(Graha::Sun, Graha::Rahu).unwrap()).unwrap();
    assert_eq!(read["mahadasha"], "SUN");
    assert_eq!(read["conditions"][0], "IN_SECOND_OR_SEVENTH_WITH_ITS_LORD");
    assert_eq!(
        read["remedies"],
        serde_json::json!(["DURGA_JAPA", "CHAGA_DANA", "KRISHNA_GO_MAHISHI"])
    );
    assert_eq!(read["verses"], "48–50");
}
