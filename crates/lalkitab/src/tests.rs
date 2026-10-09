//! The worked examples of `docs/03-design/lalkitab.md` (E1 to E11, the
//! varshphal list's own E1 left to a supplied list), and the tables held
//! to the rules the book states for them.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index their own tables"
)]

use teistro_core::catalogue::Graha;

use crate::aspects::{Relation, Strength, YogDrishti, ahead, looks_at, yog_drishti};
use crate::cycle::{CYCLE, CycleStart, periods, ruler};
use crate::tables::{
    PLANETS, Regard, Rin, cycle_years, debilitated, exalted, mahadasha_years, matures_at, own,
    regard,
};
use crate::{Dignity, Masnui, Teva, VarshphalTable, read};

/// A teva from each planet's house, in Lal Kitab's order.
fn teva(houses: [u8; 9]) -> Teva {
    Teva::from_houses(PLANETS.into_iter().zip(houses)).unwrap()
}

#[test]
fn the_tables_follow_the_rules_the_book_states() {
    // Exaltation: the classical signs read as houses; the nodes the book's.
    let expected = [
        (Graha::Sun, vec![1], vec![7]),
        (Graha::Moon, vec![2], vec![8]),
        (Graha::Mars, vec![10], vec![4]),
        (Graha::Mercury, vec![6], vec![12]),
        (Graha::Jupiter, vec![4], vec![10]),
        (Graha::Venus, vec![12], vec![6]),
        (Graha::Saturn, vec![7], vec![1]),
        (Graha::Rahu, vec![3, 6], vec![9, 12]),
        (Graha::Ketu, vec![9, 12], vec![3, 6]),
    ];
    for (graha, up, down) in expected {
        assert_eq!(exalted(graha), up, "{graha:?}");
        assert_eq!(debilitated(graha), down, "{graha:?}");
    }
    assert_eq!(own(Graha::Sun), [5]);
    assert_eq!(own(Graha::Mars), [1, 8]);
    assert_eq!(own(Graha::Saturn), [10, 11]);
    assert_eq!(own(Graha::Rahu), Vec::<u8>::new());
    // The regard is directed (p. 31): the Moon counts Venus equal, Venus
    // counts the Moon an enemy.
    assert_eq!(regard(Graha::Moon, Graha::Venus), Some(Regard::Equal));
    assert_eq!(regard(Graha::Venus, Graha::Moon), Some(Regard::Enemy));
    // E5: the mahadashas make 120 and the cycle 35.
    let total = |years: fn(Graha) -> u8| PLANETS.iter().map(|g| u16::from(years(*g))).sum::<u16>();
    assert_eq!(total(mahadasha_years), 120);
    assert_eq!(total(cycle_years), CYCLE);
}

#[test]
fn the_waking_ages_are_where_the_second_round_of_the_general_cycle_starts() {
    // A table that checks itself: p. 99's ages are p. ~219's starts, from
    // the 16th year on.
    let starts = periods(16, 50, CycleStart::GENERAL);
    for planet in PLANETS {
        let first = starts
            .iter()
            .find(|period| period.planet == planet)
            .unwrap();
        assert_eq!(
            Some(first.from),
            matures_at(planet).map(u16::from),
            "{planet:?}"
        );
    }
}

#[test]
fn the_cycle_runs_from_the_general_start_or_the_readers() {
    // E4: the general table.
    for (year, planet) in [
        (1, Graha::Saturn),
        (7, Graha::Rahu),
        (16, Graha::Jupiter),
        (24, Graha::Moon),
        (36, Graha::Saturn),
        (120, Graha::Ketu),
    ] {
        assert_eq!(ruler(year, CycleStart::GENERAL), planet, "year {year}");
    }
    // E3: Venus from the 17th year runs back as well as on.
    let venus = CycleStart::new(Graha::Venus, 17).unwrap();
    let spans = |planet: Graha| -> Vec<(u16, u16)> {
        periods(1, 110, venus)
            .into_iter()
            .filter(|period| period.planet == planet)
            .map(|period| (period.from, period.to))
            .collect()
    };
    assert_eq!(spans(Graha::Jupiter), [(8, 13), (43, 48), (78, 83)]);
    assert_eq!(spans(Graha::Saturn), [(28, 33), (63, 68), (98, 103)]);
    assert_eq!(
        CycleStart::new(Graha::Venus, 0).unwrap_err().field(),
        Some("year")
    );
}

#[test]
fn a_teva_is_the_whole_sign_house_from_the_lagna_and_refuses_what_is_not_one() {
    use teistro_core::catalogue::Rashi;
    // E6: a Libra lagna.
    let mut signs: Vec<(Graha, Rashi)> = PLANETS.iter().map(|g| (*g, Rashi::Leo)).collect();
    signs[1] = (Graha::Sun, Rashi::Libra);
    signs[2] = (Graha::Moon, Rashi::Aries);
    let teva = Teva::from_signs(Rashi::Libra, signs).unwrap();
    assert_eq!(teva.house_of(Graha::Sun), Some(1));
    assert_eq!(teva.house_of(Graha::Moon), Some(7));
    assert_eq!(teva.house_of(Graha::Jupiter), Some(11));
    let field = |placements: Vec<(Graha, u8)>| {
        Teva::from_houses(placements)
            .unwrap_err()
            .field()
            .map(str::to_owned)
    };
    let mut twice: Vec<(Graha, u8)> = PLANETS.iter().map(|g| (*g, 1)).collect();
    twice[3] = (Graha::Sun, 2);
    assert_eq!(field(twice).as_deref(), Some("placements[3]"));
    let mut outside: Vec<(Graha, u8)> = PLANETS.iter().map(|g| (*g, 1)).collect();
    outside[0].1 = 13;
    assert_eq!(field(outside).as_deref(), Some("placements[0]"));
    let short: Vec<(Graha, u8)> = PLANETS.iter().skip(1).map(|g| (*g, 1)).collect();
    assert_eq!(field(short).as_deref(), Some("placements"));
    let mut uranus: Vec<(Graha, u8)> = PLANETS.iter().map(|g| (*g, 1)).collect();
    uranus.push((Graha::Uranus, 3));
    assert_eq!(field(uranus).as_deref(), Some("placements[9]"));
}

#[test]
fn the_aspects_run_forward_and_the_yog_drishti_is_its_rule() {
    assert_eq!(looks_at(1), [(7, Strength::Full)]);
    assert_eq!(looks_at(8), [(2, Strength::Quarter)]);
    assert_eq!((looks_at(7), looks_at(11)), (&[][..], &[][..]));
    // E10.
    let fourth = yog_drishti(4);
    let pairs = [
        (fourth.help, (8, 12)),
        (fourth.general, (10, 10)),
        (fourth.collision, (11, 9)),
        (fourth.foundation, (12, 8)),
        (fourth.deceit, (1, 7)),
        (fourth.wall, (5, 3)),
    ];
    for (relation, (sees, seen_from)) in pairs {
        assert_eq!((relation.sees, relation.seen_from), (sees, seen_from));
    }
    // Each relation is one relation read from both ends: the house a house
    // sees under it sees that house back as where it is seen from.
    for house in 1..=12 {
        let seen = yog_drishti(house);
        let relations: [fn(YogDrishti) -> Relation; 6] = [
            |y| y.help,
            |y| y.general,
            |y| y.collision,
            |y| y.foundation,
            |y| y.deceit,
            |y| y.wall,
        ];
        for of in relations {
            let relation = of(seen);
            assert_eq!(of(yog_drishti(relation.sees)).seen_from, house);
        }
        assert_eq!(ahead(house, 12), house);
    }
}

#[test]
fn a_reading_reports_the_books_conditions() {
    // Jupiter Sun Moon Venus Mars Mercury Saturn Rahu Ketu
    let chart = teva([2, 10, 9, 7, 3, 4, 5, 12, 4]);
    let reading = read(&chart);
    let planet = |graha: Graha| reading.planets.iter().find(|p| p.graha == graha).unwrap();
    // E9: Venus in 7 and Mars in 3 are in their pakka ghar and awake.
    assert!(planet(Graha::Venus).awake && planet(Graha::Mars).awake);
    assert!(planet(Graha::Venus).dignities.contains(&Dignity::Pakka));
    // E7: Saturn in the Sun's 5 and the Sun in Saturn's pakka 10.
    assert!(reading.flags.sathi.contains(&[Graha::Sun, Graha::Saturn]));
    // E11: the Moon in 9 with Mercury in 4.
    assert_eq!(reading.pitri.len(), 1);
    assert_eq!(reading.pitri[0].ninth, Graha::Moon);
    // Ketu in the Moon's 4: the mother's debt; Ketu in 4 is a dharmi papi.
    let matri = reading
        .rinas
        .iter()
        .find(|debt| debt.rin == Rin::Matri)
        .unwrap();
    assert_eq!(matri.seated[0].enemy, Graha::Ketu);
    assert!(reading.flags.dharmi.contains(&Graha::Ketu));
    // Mercury and Ketu together make no artificial planet; none is formed.
    assert_eq!(reading.masnui, Vec::new());
    assert!(!reading.flags.ratandha);
    // Every house is read, its waker the p. 98 table's.
    assert_eq!(reading.houses.len(), 12);
    assert_eq!(reading.houses[0].waker, Graha::Mars);
    // House 1 is empty and no house looks at it.
    assert!(!reading.houses[0].awake);
}

#[test]
fn the_flags_hold_where_the_book_says() {
    // E8: the Sun in 4 and Saturn in 7, with the Sun and Venus together.
    let night_blind = read(&teva([2, 4, 9, 4, 3, 6, 7, 12, 6]));
    assert!(night_blind.flags.ratandha);
    let formed = &night_blind.masnui;
    assert!(
        formed
            .iter()
            .any(|f| f.counts_as == Masnui::Jupiter && f.house == 4)
    );
    // A minor's chart: nothing in 1, 4, 7 or 10.
    let minor = read(&teva([2, 3, 5, 6, 8, 9, 11, 12, 2]));
    assert!(minor.flags.nabalig);
    // One papi alone in the kendras still is; two planets there are not.
    assert!(read(&teva([2, 3, 5, 6, 8, 9, 10, 12, 2])).flags.nabalig);
    assert!(!read(&teva([1, 3, 5, 6, 8, 9, 10, 12, 2])).flags.nabalig);
}

#[test]
fn a_kayam_planet_stands_alone_in_a_dignity_and_unseen() {
    // The Sun alone in its pakka 1, nothing in a house that looks at 1.
    let reading = read(&teva([2, 1, 4, 7, 3, 6, 10, 12, 6]));
    let sun = reading
        .planets
        .iter()
        .find(|p| p.graha == Graha::Sun)
        .unwrap();
    assert_eq!(sun.dignities, [Dignity::Pakka, Dignity::Exalted]);
    assert!(sun.kayam);
    // Mars in 3 looks at 9 and 11 and is not kayam if Jupiter joins it.
    let joined = read(&teva([3, 1, 4, 7, 3, 6, 10, 12, 6]));
    let mars = joined
        .planets
        .iter()
        .find(|p| p.graha == Graha::Mars)
        .unwrap();
    assert!(!mars.kayam);
    assert_eq!(mars.casts.len(), 2);
}

/// A list with the book's structure and none of its numbers: year `y`
/// sends natal house `h` to `h + y − 1`, round the twelve.
fn synthetic() -> Vec<[u8; 12]> {
    (0..120_u8)
        .map(|year| {
            let mut row = [0_u8; 12];
            for (column, slot) in row.iter_mut().enumerate() {
                let column = u8::try_from(column).unwrap();
                *slot = (column + year) % 12 + 1;
            }
            row
        })
        .collect()
}

#[test]
fn a_varshphal_list_is_checked_and_moves_each_house_together() {
    let table = VarshphalTable::from_rows(synthetic()).unwrap();
    let natal = teva([2, 4, 9, 4, 3, 6, 7, 12, 6]);
    let second = table.annual(&natal, 2).unwrap();
    // The Sun and Venus share house 4 at birth and share 5 in year 2.
    assert_eq!(second.house_of(Graha::Sun), Some(5));
    assert_eq!(second.house_of(Graha::Venus), Some(5));
    assert_eq!(table.annual(&natal, 0).unwrap_err().field(), Some("year"));
    assert_eq!(table.annual(&natal, 121).unwrap_err().field(), Some("year"));
    // Refusals name the cell.
    let field = |rows: Vec<[u8; 12]>| {
        VarshphalTable::from_rows(rows)
            .unwrap_err()
            .field()
            .map(str::to_owned)
    };
    assert_eq!(field(synthetic()[..119].to_vec()).as_deref(), Some("rows"));
    let mut repeated = synthetic();
    repeated[5][3] = repeated[5][4];
    assert_eq!(field(repeated).as_deref(), Some("rows[5]"));
    // A row repeated inside a twelve-year block is a permutation and still
    // breaks the block's Latin square.
    let mut echoed = synthetic();
    echoed[13] = echoed[12];
    assert_eq!(field(echoed).as_deref(), Some("rows[13][0]"));
}
