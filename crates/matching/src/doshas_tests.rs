//! The marriage doshas gathered from the three readings, and nothing else.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index their own results"
)]

use teistro_core::catalogue::{Koota, Rashi};

use crate::{
    DoshaSystem, KootaReading, KootaRules, KujaNative, KujaRules, MarriageDosha, MatchRole, Native,
    PoruthamRules, ashta_koota, kuja, marriage_doshas, porutham,
};

/// A native's Mars in a sign, everything else in Aries.
fn mars_in(sign: Rashi) -> KujaNative {
    KujaNative {
        lagna: Rashi::Aries,
        moon: Rashi::Aries,
        venus: Rashi::Aries,
        mars: sign,
    }
}

/// Whether a koota's reading says its dosha is lifted.
fn lifted(reading: KootaReading) -> bool {
    match reading {
        KootaReading::GrahaMaitri { lifted, .. }
        | KootaReading::Gana { lifted, .. }
        | KootaReading::Bhakoot { lifted, .. }
        | KootaReading::Nadi { lifted, .. } => lifted,
        _ => false,
    }
}

fn doshas(bride: Native, groom: Native, mars: (KujaNative, KujaNative)) -> Vec<MarriageDosha> {
    marriage_doshas(
        &ashta_koota(bride, groom, KootaRules::default()),
        &porutham(bride, groom, PoruthamRules::default()),
        &kuja(mars.0, mars.1, KujaRules::default()),
    )
}

#[test]
fn a_birth_with_itself_carries_one_nadi_and_the_disagreeing_considerations() {
    let moon = Native::of_moon(45.0).unwrap();
    let quiet = mars_in(Rashi::Gemini);
    let read = doshas(moon, moon, (quiet, quiet));
    let ashta: Vec<_> = read
        .iter()
        .filter(|one| one.system == DoshaSystem::AshtaKoota)
        .collect();
    // One pada on both sides: the shared nadi VI.36 does not lift, and
    // nothing else.
    assert_eq!(ashta.len(), 1);
    assert_eq!(
        (ashta[0].koota, ashta[0].lifted),
        (Some(Koota::Nadi), false)
    );
    // The ten report each that disagrees or agrees only by the exception.
    let ten = porutham(moon, moon, PoruthamRules::default());
    let expected: Vec<_> = ten
        .considerations
        .iter()
        .filter(|row| !row.agrees || row.lifted)
        .map(|row| (Some(row.reading.koota()), row.lifted))
        .collect();
    let listed: Vec<_> = read
        .iter()
        .filter(|one| one.system == DoshaSystem::Porutham)
        .map(|one| (one.koota, one.lifted))
        .collect();
    assert_eq!(listed, expected);
    assert!(read.iter().all(|one| one.system != DoshaSystem::Kuja));
}

#[test]
fn every_ashta_entry_is_a_dosha_its_reading_names() {
    let quiet = (mars_in(Rashi::Gemini), mars_in(Rashi::Gemini));
    for bride in (0..108).map(|pada| Native::of_moon(f64::from(pada) * 10.0 / 3.0 + 1.0)) {
        let bride = bride.unwrap();
        for groom in (0..108).map(|pada| Native::of_moon(f64::from(pada) * 10.0 / 3.0 + 1.0)) {
            let groom = groom.unwrap();
            let read = ashta_koota(bride, groom, KootaRules::default());
            let listed = doshas(bride, groom, quiet);
            let mut kootas = listed
                .iter()
                .filter(|one| one.system == DoshaSystem::AshtaKoota)
                .map(|one| one.koota.unwrap());
            // In the verse's order, and only the four the verses name.
            let order: Vec<Koota> = kootas.by_ref().collect();
            let mut sorted = order.clone();
            sorted.sort_by_key(|koota| crate::ASHTA_KOOTA.iter().position(|k| k == koota));
            assert_eq!(order, sorted);
            assert!(order.iter().all(|koota| matches!(
                koota,
                Koota::GrahaMaitri | Koota::Gana | Koota::Bhakoot | Koota::Nadi
            )));
            // Each entry's lift is its reading's.
            for one in listed
                .iter()
                .filter(|one| one.system == DoshaSystem::AshtaKoota)
            {
                let row = read.row(one.koota.unwrap()).unwrap();
                assert_eq!(one.lifted, lifted(row.reading), "{:?}", row.reading);
            }
        }
    }
}

#[test]
fn each_side_with_the_kuja_dosha_is_listed_bride_first() {
    let moon = Native::of_moon(45.0).unwrap();
    let seventh = mars_in(Rashi::Libra);
    let third = mars_in(Rashi::Gemini);
    let sides = |mars| -> Vec<Option<MatchRole>> {
        doshas(moon, moon, mars)
            .into_iter()
            .filter(|one| one.system == DoshaSystem::Kuja)
            .map(|one| {
                assert_eq!((one.koota, one.lifted), (None, false));
                one.side
            })
            .collect()
    };
    assert_eq!(
        sides((seventh, seventh)),
        [Some(MatchRole::Bride), Some(MatchRole::Groom)]
    );
    assert_eq!(sides((third, seventh)), [Some(MatchRole::Groom)]);
    assert_eq!(sides((third, third)), []);
}

#[test]
fn an_entry_keeps_its_wire_spelling() {
    let moon = Native::of_moon(45.0).unwrap();
    let seventh = mars_in(Rashi::Libra);
    let read = doshas(moon, moon, (seventh, seventh));
    let last = serde_json::to_value(read.last().unwrap()).unwrap();
    assert_eq!(last["system"], "KUJA");
    assert_eq!(last["side"], "GROOM");
    assert!(last["koota"].is_null());
    let first = serde_json::to_value(read[0]).unwrap();
    assert_eq!(first["system"], "ASHTA_KOOTA");
    assert!(first["side"].is_null());
}
