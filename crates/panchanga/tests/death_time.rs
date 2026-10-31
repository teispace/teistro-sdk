//! The death-time check held to the texts' clauses
//! (`03-design/death-time.md`).

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the answers they read"
)]

use teistro_core::catalogue::{Nakshatra, Rashi, Tithi, Vara};
use teistro_core::quantity::JulianDay;
use teistro_panchanga::death_time::{
    DeathInstant, DeathRiteKind, DeathTime, DeathTimeTable, NakshatraFeet, Pushkara, owed,
};

fn read(nakshatra: Nakshatra, moon: Rashi) -> DeathInstant {
    DeathInstant::read(
        &DeathTimeTable::dharmasindhu(),
        JulianDay::literal(2_461_051.5),
        nakshatra,
        Tithi::ShuklaPanchami,
        Vara::Somavara,
        moon,
    )
}

fn kinds(death: &DeathInstant, cremation: Option<&DeathInstant>) -> Vec<DeathRiteKind> {
    owed(death, cremation).iter().map(|one| one.rite).collect()
}

/// *Muhūrta-cintāmaṇi* II.49 comm., p. 86: a nakshatra is tripada when a
/// sign ends after its first or third foot, dvipada after its second.
#[test]
fn mc_249_the_lists_are_the_sign_arithmetic() {
    let table = DeathTimeTable::dharmasindhu();
    for (index, &nakshatra) in Nakshatra::ALL.iter().enumerate() {
        // In padas from Aries 0; a sign is nine padas, a nakshatra four.
        let start = index * 4;
        let feet = (1..4).find(|foot| (start + foot) % 9 == 0);
        let want = match feet {
            Some(1 | 3) => Some(NakshatraFeet::Tripada),
            Some(2) => Some(NakshatraFeet::Dvipada),
            _ => None,
        };
        assert_eq!(table.feet(nakshatra), want, "{nakshatra:?}");
    }
}

#[test]
fn ds_334_the_panchaka_is_the_moon_in_aquarius_or_pisces() {
    // Dhanishtha's first half is not in it, its second half is.
    assert!(!read(Nakshatra::Dhanishtha, Rashi::Capricorn).panchaka);
    assert!(read(Nakshatra::Dhanishtha, Rashi::Aquarius).panchaka);
    assert!(read(Nakshatra::Revati, Rashi::Pisces).panchaka);
    assert!(!read(Nakshatra::Ashwini, Rashi::Aries).panchaka);
}

#[test]
fn ds_335_the_four_cases_of_death_and_burning() {
    use DeathRiteKind::{ClosingShanti, PanchakaFigures};
    let before = read(Nakshatra::Shravana, Rashi::Capricorn);
    let inside = read(Nakshatra::Shatabhisha, Rashi::Aquarius);
    let after = read(Nakshatra::Ashwini, Rashi::Aries);
    assert_eq!(kinds(&before, Some(&before)), []);
    assert_eq!(kinds(&before, Some(&inside)), [PanchakaFigures]);
    assert_eq!(kinds(&inside, Some(&after)), [ClosingShanti]);
    assert_eq!(
        kinds(&inside, Some(&inside)),
        [PanchakaFigures, ClosingShanti]
    );
    // With no cremation given, it is taken to fall as the death did.
    assert_eq!(kinds(&inside, None), [PanchakaFigures, ClosingShanti]);
    assert!(owed(&before, Some(&inside))[0].clause.contains("no shanti"));
    assert!(owed(&inside, Some(&after))[0].clause.contains("no figures"));
}

#[test]
fn ds_335_a_tripada_nakshatra_owes_the_shanti_and_a_dvipada_alone_nothing() {
    let tripada = read(Nakshatra::Vishakha, Rashi::Libra);
    assert_eq!(tripada.feet, Some(NakshatraFeet::Tripada));
    assert_eq!(kinds(&tripada, None), [DeathRiteKind::ClosingShanti]);
    let dvipada = read(Nakshatra::Chitra, Rashi::Libra);
    assert_eq!(dvipada.feet, Some(NakshatraFeet::Dvipada));
    assert_eq!(kinds(&dvipada, None), []);
}

#[test]
fn ds_316_335_the_pushkara_yogas() {
    let table = DeathTimeTable::dharmasindhu();
    let of = |tithi, vara, nakshatra| table.pushkara(tithi, vara, nakshatra);
    let (s2, sat) = (Tithi::ShuklaDvitiya, Vara::Shanivara);
    assert_eq!(
        of(s2, sat, Nakshatra::Punarvasu),
        Some(Pushkara::Tripushkara)
    );
    assert_eq!(
        of(s2, sat, Nakshatra::Mrigashira),
        Some(Pushkara::Dvipushkara)
    );
    assert_eq!(of(s2, sat, Nakshatra::Ashwini), None);
    assert_eq!(
        of(Tithi::KrishnaDwadashi, Vara::Ravivara, Nakshatra::Chitra),
        Some(Pushkara::Dvipushkara)
    );
    assert_eq!(of(Tithi::ShuklaTritiya, sat, Nakshatra::Punarvasu), None);
    assert_eq!(of(s2, Vara::Guruvara, Nakshatra::Punarvasu), None);
    // The other reading: Thursday for Sunday (DS p. 316), the two alone
    // a dvipushkara (DS p. 316, Garga in NS p. 434).
    let other = DeathTimeTable {
        pushkara_varas: vec![Vara::Guruvara, Vara::Mangalavara, Vara::Shanivara],
        two_make_dvipushkara: true,
        ..table.clone()
    };
    assert_eq!(
        other.pushkara(s2, Vara::Guruvara, Nakshatra::Ashwini),
        Some(Pushkara::Dvipushkara)
    );
    assert_eq!(
        other.pushkara(s2, Vara::Ravivara, Nakshatra::Punarvasu),
        None
    );
}

#[test]
fn ds_335_a_pushkara_at_the_death_or_the_burning_owes_the_figures_and_the_shanti() {
    use DeathRiteKind::{ClosingShanti, PanchakaFigures, PushkaraFigures};
    let table = DeathTimeTable::dharmasindhu();
    let pushkara = DeathInstant::read(
        &table,
        JulianDay::literal(2_461_051.5),
        Nakshatra::Chitra,
        Tithi::ShuklaSaptami,
        Vara::Shanivara,
        Rashi::Libra,
    );
    let plain = read(Nakshatra::Rohini, Rashi::Taurus);
    assert_eq!(kinds(&pushkara, None), [PushkaraFigures, ClosingShanti]);
    assert_eq!(
        kinds(&plain, Some(&pushkara)),
        [PushkaraFigures, ClosingShanti]
    );
    let inside = read(Nakshatra::Revati, Rashi::Pisces);
    assert_eq!(
        kinds(&inside, Some(&pushkara)),
        [PushkaraFigures, ClosingShanti]
    );
    let burnt_inside = DeathInstant {
        pushkara: pushkara.pushkara,
        ..inside
    };
    assert_eq!(
        kinds(&burnt_inside, None),
        [PanchakaFigures, PushkaraFigures, ClosingShanti]
    );
}

#[test]
fn an_answer_writes_its_members_in_full() {
    let death = read(Nakshatra::Revati, Rashi::Pisces);
    let answer = DeathTime {
        death,
        cremation: Some(read(Nakshatra::Ashwini, Rashi::Aries)),
        rites: owed(&death, None),
        table: String::from("DHARMASINDHU"),
    };
    let written = answer.in_full().unwrap();
    assert_eq!(written["death"]["nakshatra"], "nakshatra.REVATI");
    assert_eq!(written["cremation"]["moon"], "rashi.ARIES");
    assert_eq!(written["rites"].as_array().unwrap().len(), 2);
}
