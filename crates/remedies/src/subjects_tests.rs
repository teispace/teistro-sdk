//! Whom a remedy is for, and the printed antardaśā conditions judged
//! where their predicate can be, on skies worked out by hand.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the skies they build"
)]

use teistro_core::catalogue::{Graha, Rashi};

use crate::{Condition, FunctionalRules, Reason, RemedySky, Running, functional, holds, subjects};

/// Every graha in `sign` but those `moved`, nothing combust, a waxing
/// Moon.
fn sky(lagna: Rashi, sign: Rashi, moved: &[(usize, Rashi)]) -> RemedySky {
    let mut signs = [sign; 9];
    for &(at, to) in moved {
        signs[at] = to;
    }
    RemedySky {
        lagna,
        signs,
        combust: [false; 9],
        moon_waxing: true,
        running: None,
    }
}

const SUN: usize = 0;
const MARS: usize = 2;
const SATURN: usize = 6;
const RAHU: usize = 7;

fn run(mahadasha: Graha, antardasha: Graha) -> Running {
    Running {
        mahadasha,
        antardasha,
    }
}

#[test]
fn every_reason_is_reported_and_none_is_ranked() {
    // Taurus rising, everything in Taurus but the Sun, debilitated in
    // Libra, the 6th. Laghu Parashari makes the Moon (the 3rd) and
    // Jupiter (the 8th and 11th) malefic; Mercury (the 2nd) and Mars (the
    // 7th) are marakas; a fixed lagna's badhaka is the 9th, Saturn's; and
    // the catalogue debilitates Ketu in Taurus.
    let taurus = sky(Rashi::Taurus, Rashi::Taurus, &[(SUN, Rashi::Libra)]);
    let read = subjects(
        &taurus,
        &functional(Rashi::Taurus, FunctionalRules::default()),
    );
    let found: Vec<(Graha, Vec<Reason>)> = read
        .subjects
        .into_iter()
        .map(|subject| (subject.graha, subject.reasons))
        .collect();
    assert_eq!(
        found,
        [
            (Graha::Sun, vec![Reason::Debilitated, Reason::Dusthana]),
            (Graha::Moon, vec![Reason::FunctionalMalefic]),
            (Graha::Mars, vec![Reason::Maraka]),
            (Graha::Mercury, vec![Reason::Maraka]),
            (Graha::Jupiter, vec![Reason::FunctionalMalefic]),
            (Graha::Saturn, vec![Reason::Badhakesha]),
            (Graha::Ketu, vec![Reason::Debilitated]),
        ]
    );
    assert_eq!(read.antardasha, None);
}

#[test]
fn the_running_dasha_brings_its_printed_shanti() {
    // Leo rising: Saturn owns the 6th and the 7th, so Sun/Saturn's
    // condition (37.65) holds; under Taurus he owns the 9th and 10th.
    let mut leo = sky(Rashi::Leo, Rashi::Leo, &[]);
    leo.running = Some(run(Graha::Sun, Graha::Saturn));
    let read = subjects(&leo, &functional(Rashi::Leo, FunctionalRules::default()));
    let shanti = read.antardasha.unwrap();
    assert_eq!(shanti.shanti.verses, "65–66");
    assert_eq!(shanti.holds, [Some(true)]);
    let saturn = read
        .subjects
        .iter()
        .find(|subject| subject.graha == Graha::Saturn)
        .unwrap();
    assert_eq!(saturn.reasons[0], Reason::Antardasha);
    assert_eq!(
        holds(
            Condition::LordOfSecondOrSeventh,
            &sky(Rashi::Taurus, Rashi::Leo, &[]),
            run(Graha::Sun, Graha::Saturn)
        ),
        Some(false)
    );
}

#[test]
fn a_placement_is_counted_from_the_lagna_or_the_dasha_lord_as_printed() {
    let taurus = Rashi::Taurus;
    // Moon/Saturn (38.37): Saturn in the 2nd, 7th or 8th: Sagittarius is
    // the 8th from Taurus.
    let eighth = sky(taurus, taurus, &[(SATURN, Rashi::Sagittarius)]);
    assert_eq!(
        holds(
            Condition::InSecondSeventhOrEighth,
            &eighth,
            run(Graha::Moon, Graha::Saturn)
        ),
        Some(true)
    );
    assert_eq!(
        holds(
            Condition::InSecondOrSeventh,
            &eighth,
            run(Graha::Moon, Graha::Saturn)
        ),
        Some(false)
    );

    // Mars/Saturn (39.33): Saturn 8th from Mars and joined with a
    // malefic. Mars in Taurus, Saturn in Sagittarius with the Sun.
    let with_sun = sky(
        taurus,
        taurus,
        &[(SATURN, Rashi::Sagittarius), (SUN, Rashi::Sagittarius)],
    );
    let mars_saturn = run(Graha::Mars, Graha::Saturn);
    assert_eq!(
        holds(
            Condition::DusthanaFromDashaLordWithMalefic,
            &with_sun,
            mars_saturn
        ),
        Some(true)
    );
    assert_eq!(
        holds(
            Condition::DusthanaFromDashaLordWithMalefic,
            &eighth,
            mars_saturn
        ),
        Some(false)
    );
    // The same placement under Sun/Jupiter's "or" holds without one.
    let jupiter_eighth = sky(taurus, taurus, &[(4, Rashi::Sagittarius)]);
    assert_eq!(
        holds(
            Condition::SixthOrEighthFromDashaLordOrWeak,
            &jupiter_eighth,
            run(Graha::Sun, Graha::Jupiter)
        ),
        Some(true)
    );

    // Sun/Rahu (37.48): Rahu in the 7th, Scorpio, with its lord Mars.
    let rahu_seventh = sky(
        taurus,
        taurus,
        &[(RAHU, Rashi::Scorpio), (MARS, Rashi::Scorpio)],
    );
    assert_eq!(
        holds(
            Condition::InSecondOrSeventhWithItsLord,
            &rahu_seventh,
            run(Graha::Sun, Graha::Rahu)
        ),
        Some(true)
    );
    let alone = sky(taurus, taurus, &[(RAHU, Rashi::Scorpio)]);
    assert_eq!(
        holds(
            Condition::InSecondOrSeventhWithItsLord,
            &alone,
            run(Graha::Sun, Graha::Rahu)
        ),
        Some(false)
    );
}

#[test]
fn what_the_verse_leaves_open_is_not_decided() {
    let taurus = sky(Rashi::Taurus, Rashi::Taurus, &[]);
    // A node owns no sign here, so its lordship is open, not false (C351).
    assert_eq!(
        holds(
            Condition::LordOfSecondOrSeventh,
            &taurus,
            run(Graha::Sun, Graha::Ketu)
        ),
        None
    );
    // Jupiter/Moon's 2nd-and-6th lord, which the Moon cannot be (C349).
    assert_eq!(
        holds(
            Condition::LordOfSecondAndSixth,
            &taurus,
            run(Graha::Jupiter, Graha::Moon)
        ),
        None
    );
    assert_eq!(
        holds(
            Condition::JoinedOrAspected,
            &taurus,
            run(Graha::Jupiter, Graha::Venus)
        ),
        None
    );
}

#[test]
fn the_keys_are_what_serde_writes() {
    let mut leo = sky(Rashi::Leo, Rashi::Leo, &[]);
    leo.running = Some(run(Graha::Sun, Graha::Saturn));
    let read = serde_json::to_value(subjects(
        &leo,
        &functional(Rashi::Leo, FunctionalRules::default()),
    ))
    .unwrap();
    assert_eq!(read["antardasha"]["holds"], serde_json::json!([true]));
    assert_eq!(read["antardasha"]["shanti"]["antardasha"], "SATURN");
    assert_eq!(read["subjects"][0]["graha"], "SUN");
    assert_eq!(read["subjects"][0]["reasons"][0], "MAHADASHA");
    let input: RemedySky = serde_json::from_value(serde_json::to_value(leo).unwrap()).unwrap();
    assert_eq!(input, leo);
}

#[test]
fn five_ketu_antardashas_turn_on_a_lordship_no_text_gives_a_node() {
    let taurus = sky(Rashi::Taurus, Rashi::Taurus, &[]);
    let open: Vec<(Graha, Graha)> = crate::dasha_shantis()
        .iter()
        .filter(|row| matches!(row.antardasha, Graha::Rahu | Graha::Ketu))
        .filter(|row| row.conditions == [Condition::LordOfSecondOrSeventh])
        .inspect(|row| {
            let running = run(row.mahadasha, row.antardasha);
            assert_eq!(holds(row.conditions[0], &taurus, running), None);
        })
        .map(|row| (row.mahadasha, row.antardasha))
        .collect();
    assert_eq!(
        open,
        [
            (Graha::Sun, Graha::Ketu),
            (Graha::Rahu, Graha::Ketu),
            (Graha::Jupiter, Graha::Ketu),
            (Graha::Mercury, Graha::Ketu),
            (Graha::Venus, Graha::Ketu),
        ]
    );
}
