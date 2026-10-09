//! The rule held to every printed cell, and the worked examples of
//! `docs/03-design/pakshi.md` §4.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    reason = "tests fail by panicking, index their own tables, and hold tiled spans to the bit"
)]

use teistro_core::catalogue::{Nakshatra, Paksha, Vara};

use crate::tables::{activity, death_bird, relation, sequence, sub_share, subs};
use crate::{
    Activity, Bird, BirthBird, Clock, Day, Half, Relation, Relations, Rules, SubLengths,
    birth_bird, now, read_day,
};

/// Ayyar's four tables as `pakshi.md` §2.7 transcribes them, his one slip
/// corrected (crux P7): per half, the weekday groups' columns and each
/// bird's row of activities over the five yamas.
const TABLES: [(Paksha, Half, [&[Vara]; 5], [[&str; 5]; 5]); 4] = {
    use Vara::{
        Budhavara as Wed, Guruvara as Thu, Mangalavara as Tue, Ravivara as Sun, Shanivara as Sat,
        Shukravara as Fri, Somavara as Mon,
    };
    const BRIGHT: [&[Vara]; 5] = [&[Sun, Tue], &[Mon, Wed], &[Thu], &[Fri], &[Sat]];
    const DARK: [&[Vara]; 5] = [&[Sun, Tue], &[Mon, Sat], &[Wed], &[Thu], &[Fri]];
    [
        (
            Paksha::Shukla,
            Half::Day,
            BRIGHT,
            [
                ["EWRSD", "DEWRS", "SDEWR", "RSDEW", "WRSDE"],
                ["WRSDE", "EWRSD", "DEWRS", "SDEWR", "RSDEW"],
                ["RSDEW", "WRSDE", "EWRSD", "DEWRS", "SDEWR"],
                ["SDEWR", "RSDEW", "WRSDE", "EWRSD", "DEWRS"],
                ["DEWRS", "SDEWR", "RSDEW", "WRSDE", "EWRSD"],
            ],
        ),
        (
            Paksha::Shukla,
            Half::Night,
            BRIGHT,
            [
                ["DWSER", "WSERD", "SERDW", "ERDWS", "RDWSE"],
                ["RDWSE", "DWSER", "WSERD", "SERDW", "ERDWS"],
                ["ERDWS", "RDWSE", "DWSER", "WSERD", "SERDW"],
                ["SERDW", "ERDWS", "RDWSE", "DWSER", "WSERD"],
                ["WSERD", "SERDW", "ERDWS", "RDWSE", "DWSER"],
            ],
        ),
        (
            Paksha::Krishna,
            Half::Day,
            DARK,
            [
                ["WEDSR", "SRWED", "DSRWE", "RWEDS", "EDSRW"],
                ["DSRWE", "WEDSR", "RWEDS", "EDSRW", "SRWED"],
                ["RWEDS", "DSRWE", "EDSRW", "SRWED", "WEDSR"],
                ["EDSRW", "RWEDS", "SRWED", "WEDSR", "DSRWE"],
                ["SRWED", "EDSRW", "WEDSR", "DSRWE", "RWEDS"],
            ],
        ),
        (
            Paksha::Krishna,
            Half::Night,
            DARK,
            [
                ["ESWDR", "DRESW", "SWDRE", "WDRES", "RESWD"],
                ["RESWD", "WDRES", "ESWDR", "SWDRE", "DRESW"],
                ["DRESW", "SWDRE", "RESWD", "ESWDR", "WDRES"],
                ["WDRES", "ESWDR", "DRESW", "RESWD", "SWDRE"],
                ["SWDRE", "RESWD", "WDRES", "DRESW", "ESWDR"],
            ],
        ),
    ]
};

fn letter(activity: Activity) -> char {
    match activity {
        Activity::Eating => 'E',
        Activity::Walking => 'W',
        Activity::Ruling => 'R',
        Activity::Sleeping => 'S',
        Activity::Dying => 'D',
    }
}

#[test]
fn the_rule_gives_every_printed_cell() {
    for (paksha, half, groups, rows) in TABLES {
        for (bird, row) in Bird::ALL.into_iter().zip(rows) {
            for (varas, printed) in groups.into_iter().zip(row) {
                for vara in varas {
                    let ruled: String = (0..5)
                        .map(|yama| letter(activity(bird, paksha, half, *vara, yama)))
                        .collect();
                    assert_eq!(ruled, printed, "{paksha:?} {half:?} {vara:?} {bird:?}");
                }
            }
        }
    }
}

#[test]
fn ayyars_slip_is_the_one_cell_the_rule_refuses() {
    // AY p. 107, a bright Saturday night's fifth watch: he prints the first
    // watch again (the vulture ruling twice and never eating); the rule,
    // and PUL p. 45, give the vulture eating.
    let fifth: String = Bird::ALL
        .into_iter()
        .map(|bird| {
            letter(activity(
                bird,
                Paksha::Shukla,
                Half::Night,
                Vara::Shanivara,
                4,
            ))
        })
        .collect();
    assert_eq!(fifth, "ESWDR");
    assert_ne!(fifth, "RESWD");
}

#[test]
fn each_yama_the_five_birds_do_the_five_things() {
    for paksha in Paksha::ALL {
        for half in [Half::Day, Half::Night] {
            for vara in Vara::ALL {
                for yama in 0..5 {
                    let mut done: Vec<Activity> = Bird::ALL
                        .into_iter()
                        .map(|bird| activity(bird, paksha, half, vara, yama))
                        .collect();
                    done.sort();
                    done.dedup();
                    assert_eq!(done.len(), 5);
                }
            }
        }
    }
}

#[test]
fn the_birth_birds_are_the_groups_and_the_dark_half_reverses_them() {
    let groups = [
        (0, 4, Bird::Vulture),
        (5, 10, Bird::Owl),
        (11, 15, Bird::Crow),
        (16, 20, Bird::Cock),
        (21, 26, Bird::Peacock),
    ];
    let dark = [
        Bird::Peacock,
        Bird::Cock,
        Bird::Crow,
        Bird::Owl,
        Bird::Vulture,
    ];
    for nakshatra in Nakshatra::ALL {
        let id = nakshatra.id();
        let group = groups
            .iter()
            .position(|(from, to, _)| (*from..=*to).contains(&id))
            .unwrap();
        let bright = birth_bird(nakshatra, Paksha::Shukla, BirthBird::ByPaksha);
        assert_eq!(bright, groups[group].2, "{nakshatra:?}");
        assert_eq!(
            birth_bird(nakshatra, Paksha::Krishna, BirthBird::ByPaksha),
            dark[group]
        );
        assert_eq!(
            birth_bird(nakshatra, Paksha::Krishna, BirthBird::Single),
            bright
        );
    }
}

/// A day from 6:00 to 18:00 to 6:00, as the books' examples assume, on
/// day 0.25 (6:00 UTC) of Julian day 0.
fn even(vara: Vara, paksha: Paksha) -> Day {
    Day::new(0.25, 0.75, 1.25, vara, paksha).unwrap()
}

/// The instant `hours:minutes` past midnight of `even`'s day.
fn clock(hours: u8, minutes: u8) -> f64 {
    (f64::from(hours) + f64::from(minutes) / 60.0) / 24.0
}

#[test]
fn the_books_worked_examples_come_back() {
    // PUL p. vii, 31 October 1984, a bright Wednesday: the cock (Uttara
    // Ashadha) sleeps from 8:24 to 10:48 and dies from 10:48.
    let wednesday = read_day(
        &even(Vara::Budhavara, Paksha::Shukla),
        Bird::Cock,
        &Rules::default(),
    );
    let at = now(&wednesday, clock(9, 30)).unwrap();
    assert_eq!(
        (at.yama.half, at.yama.yama, at.yama.activity),
        (Half::Day, 2, Activity::Sleeping)
    );
    assert!((at.yama.span.from - clock(8, 24)).abs() < 1e-9);
    assert_eq!(
        now(&wednesday, clock(11, 0)).unwrap().yama.activity,
        Activity::Dying
    );
    // 21 May 1991, a bright Tuesday night: the owl (Purva Phalguni) dies
    // from 20:24 to 22:48, the night's second yama.
    let tuesday = read_day(
        &even(Vara::Mangalavara, Paksha::Shukla),
        Bird::Owl,
        &Rules::default(),
    );
    let at = now(&tuesday, clock(22, 20)).unwrap();
    assert_eq!(
        (at.yama.half, at.yama.yama, at.yama.activity),
        (Half::Night, 2, Activity::Dying)
    );
    assert!((at.yama.span.to - clock(22, 48)).abs() < 1e-9);
}

#[test]
fn a_yamas_sub_periods_start_at_the_natives_own_and_are_owned_by_the_doers() {
    // PUL pp. 60–61: a bright Sunday's third yama. The vulture rules from
    // 10:48, then the owl's sleep, the crow's death, the cock's eating and
    // the peacock's walking, for 48, 18, 12, 30 and 36 minutes.
    let parts = subs(
        Bird::Vulture,
        Paksha::Shukla,
        Half::Day,
        Vara::Ravivara,
        2,
        SubLengths::Agastya,
    );
    let read: Vec<(Activity, Bird, u8)> = parts
        .iter()
        .map(|sub| (sub.activity, sub.owner, sub.share))
        .collect();
    assert_eq!(
        read,
        [
            (Activity::Ruling, Bird::Vulture, 48),
            (Activity::Sleeping, Bird::Owl, 18),
            (Activity::Dying, Bird::Crow, 12),
            (Activity::Eating, Bird::Cock, 30),
            (Activity::Walking, Bird::Peacock, 36),
        ]
    );
    // The peacock walks first, 10:48 to 11:24.
    let sunday = read_day(
        &even(Vara::Ravivara, Paksha::Shukla),
        Bird::Peacock,
        &Rules::default(),
    );
    let first = sunday.yamas[2].subs[0];
    assert_eq!(first.sub.activity, Activity::Walking);
    assert!((first.span.from - clock(10, 48)).abs() < 1e-9);
    assert!((first.span.to - clock(11, 24)).abs() < 1e-9);
    assert_eq!(first.owner_is, Relation::Friend, "a bird is its own friend");
}

#[test]
fn every_sub_period_scheme_fills_its_yama() {
    for lengths in [SubLengths::Agastya, SubLengths::Pulippani] {
        for paksha in Paksha::ALL {
            for half in [Half::Day, Half::Night] {
                let total: u16 = sequence(paksha, half)
                    .into_iter()
                    .map(|done| u16::from(sub_share(done, paksha, half, lengths)))
                    .sum();
                assert_eq!(total, 144, "{lengths:?} {paksha:?} {half:?}");
            }
        }
    }
}

#[test]
fn the_death_birds_and_relations_are_the_printed_ones() {
    let bright: Vec<Bird> = Vara::ALL
        .into_iter()
        .map(|vara| death_bird(Paksha::Shukla, vara))
        .collect();
    use Bird::{Cock as K, Crow as C, Owl as O, Peacock as P, Vulture as V};
    assert_eq!(bright, [O, C, K, P, V, O, V]);
    let dark: Vec<Bird> = Vara::ALL
        .into_iter()
        .map(|vara| death_bird(Paksha::Krishna, vara))
        .collect();
    assert_eq!(dark, [C, O, V, P, K, P, K]);
    // Agastya's lists are directed: the owl counts the vulture a friend,
    // the vulture the owl an enemy.
    assert_eq!(
        relation(O, V, Paksha::Shukla, Relations::Agastya),
        Relation::Friend
    );
    assert_eq!(
        relation(V, O, Paksha::Shukla, Relations::Agastya),
        Relation::Enemy
    );
    // Pulippani's are symmetric, two friends and two enemies each.
    for paksha in Paksha::ALL {
        for of in Bird::ALL {
            let friends = Bird::ALL
                .into_iter()
                .filter(|to| {
                    *to != of && relation(of, *to, paksha, Relations::Pulippani) == Relation::Friend
                })
                .count();
            assert_eq!(friends, 2);
            for to in Bird::ALL {
                assert_eq!(
                    relation(of, to, paksha, Relations::Pulippani),
                    relation(to, of, paksha, Relations::Pulippani)
                );
            }
        }
    }
    // The bright cycle puts the crow beside the cock: PUL p. 48's "owl" is
    // its slip.
    assert_eq!(
        relation(K, C, Paksha::Shukla, Relations::Pulippani),
        Relation::Friend
    );
    assert_eq!(
        relation(K, O, Paksha::Shukla, Relations::Pulippani),
        Relation::Enemy
    );
}

#[test]
fn a_day_refuses_instants_out_of_order_and_the_clocks_cut_it() {
    let field = |error: teistro_core::error::Error| error.field().map(str::to_owned);
    let sunset = Day::new(1.0, 0.5, 2.0, Vara::Ravivara, Paksha::Shukla).unwrap_err();
    assert_eq!(field(sunset).as_deref(), Some("sunset"));
    let next = Day::new(0.0, 0.5, 0.4, Vara::Ravivara, Paksha::Shukla).unwrap_err();
    assert_eq!(field(next).as_deref(), Some("nextSunrise"));
    // A long summer day: stretched yamas follow it, nazhigai do not.
    let long = Day::new(0.0, 0.6, 1.0, Vara::Ravivara, Paksha::Shukla).unwrap();
    let stretched = read_day(&long, Bird::Crow, &Rules::default());
    assert!((stretched.yamas[5].span.from - 0.6).abs() < 1e-12);
    let nazhigai = Rules {
        clock: Clock::Nazhigai,
        ..Rules::default()
    };
    let fixed = read_day(&long, Bird::Crow, &nazhigai);
    assert!((fixed.yamas[5].span.from - 0.5).abs() < 1e-12);
    assert!((fixed.yamas[0].span.to - 0.1).abs() < 1e-12);
    // Sub-periods tile their yama.
    for yama in &stretched.yamas {
        assert_eq!(yama.subs.first().unwrap().span.from, yama.span.from);
        assert_eq!(yama.subs.last().unwrap().span.to, yama.span.to);
    }
    let outside = now(&stretched, 1.5).unwrap_err();
    assert_eq!(field(outside).as_deref(), Some("instant"));
}
