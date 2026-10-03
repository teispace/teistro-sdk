//! The ten considerations held to *Kalaprakasika* XIII's printed lists.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index their own results"
)]

use teistro_core::catalogue::{Graha, Koota, Nakshatra, Rashi, Yoni};

use crate::porutham::{calls_friend, concordant, pierce, yoni_of};
use crate::{
    DeerghaBeyond, DhinamRule, LordsFriendship, Native, PORUTHAM, PoruthamReading, PoruthamRow,
    PoruthamRules, Rajju, TwoSignStar, porutham,
};

use Nakshatra as N;

/// A Moon in the middle of a star's pada.
fn at(star: Nakshatra, pada: u16) -> Native {
    let quarter = star.id() * 4 + pada - 1;
    Native::of_moon((f64::from(quarter) + 0.5) * 360.0 / 108.0).unwrap()
}

fn row(bride: Native, groom: Native, koota: Koota, rules: PoruthamRules) -> PoruthamRow {
    *porutham(bride, groom, rules).row(koota).unwrap()
}

fn dhinam(bride: Native, groom: Native, rules: PoruthamRules) -> (DhinamRule, bool) {
    let found = row(bride, groom, Koota::Tara, rules);
    let PoruthamReading::Tara { rule, .. } = found.reading else {
        unreachable!()
    };
    (rule, found.agrees)
}

#[test]
fn the_rajju_rule_is_the_printed_list() {
    // p. 75.
    let printed: [(Rajju, &[Nakshatra]); 5] = [
        (
            Rajju::Padha,
            &[
                N::Ashwini,
                N::Ashlesha,
                N::Magha,
                N::Jyeshtha,
                N::Mula,
                N::Revati,
            ],
        ),
        (
            Rajju::Ooru,
            &[
                N::Bharani,
                N::Pushya,
                N::PurvaPhalguni,
                N::Anuradha,
                N::PurvaAshadha,
                N::UttaraBhadrapada,
            ],
        ),
        (
            Rajju::Nabhi,
            &[
                N::Krittika,
                N::Punarvasu,
                N::UttaraPhalguni,
                N::Vishakha,
                N::UttaraAshadha,
                N::PurvaBhadrapada,
            ],
        ),
        (
            Rajju::Kanta,
            &[
                N::Rohini,
                N::Ardra,
                N::Hasta,
                N::Swati,
                N::Shravana,
                N::Shatabhisha,
            ],
        ),
        (Rajju::Siro, &[N::Mrigashira, N::Chitra, N::Dhanishtha]),
    ];
    assert_eq!(
        printed.iter().map(|(_, stars)| stars.len()).sum::<usize>(),
        27
    );
    for (rajju, stars) in printed {
        for &star in stars {
            assert_eq!(Rajju::of(star), rajju, "{star:?}");
        }
    }
}

#[test]
fn the_vedhai_pierce_both_ways_and_only_as_printed() {
    let mut pierced = 0;
    for a in Nakshatra::ALL {
        for b in Nakshatra::ALL {
            assert_eq!(pierce(a, b), pierce(b, a), "{a:?} {b:?}");
            pierced += usize::from(pierce(a, b));
        }
        assert!(!pierce(a, a));
    }
    // Twelve pairs and a triple's three, each counted both ways.
    assert_eq!(pierced, 2 * (12 + 3));
    assert!(pierce(N::Mrigashira, N::Dhanishtha));
    // C276: Ardra and Hasta sum as the first four do, and are not named.
    assert!(!pierce(N::Ardra, N::Hasta));
}

#[test]
fn the_yoni_is_the_chapters_own() {
    // p. 73: the cow is Uttara Phalguni, Uttarashadha and Uttara
    // Bhadrapada; no star is the mongoose.
    let cows: Vec<_> = Nakshatra::ALL
        .into_iter()
        .filter(|star| yoni_of(*star) == Yoni::Cow)
        .collect();
    assert_eq!(
        cows,
        [N::UttaraPhalguni, N::UttaraAshadha, N::UttaraBhadrapada]
    );
    assert!(
        Nakshatra::ALL
            .into_iter()
            .all(|star| yoni_of(star) != Yoni::Mongoose)
    );
    // Deer and elephant are enemies here, lion and elephant are not (C278).
    let hostile = |bride, groom| {
        !row(
            at(bride, 1),
            at(groom, 1),
            Koota::Yoni,
            PoruthamRules::default(),
        )
        .agrees
    };
    assert!(hostile(N::Jyeshtha, N::Bharani));
    assert!(!hostile(N::Dhanishtha, N::Bharani));
    assert!(hostile(N::UttaraAshadha, N::Chitra));
}

#[test]
fn the_lords_are_the_chapters_friends() {
    // pp. 74–75, each lord's line.
    let printed: [(Graha, &[Graha]); 7] = [
        (Graha::Mars, &[Graha::Mercury, Graha::Venus]),
        (
            Graha::Venus,
            &[Graha::Mars, Graha::Mercury, Graha::Jupiter, Graha::Saturn],
        ),
        (
            Graha::Mercury,
            &[
                Graha::Moon,
                Graha::Mars,
                Graha::Jupiter,
                Graha::Venus,
                Graha::Saturn,
            ],
        ),
        (Graha::Moon, &[Graha::Mercury, Graha::Jupiter]),
        (Graha::Sun, &[Graha::Jupiter]),
        (
            Graha::Jupiter,
            &[
                Graha::Sun,
                Graha::Moon,
                Graha::Mercury,
                Graha::Venus,
                Graha::Saturn,
            ],
        ),
        (
            Graha::Saturn,
            &[
                Graha::Sun,
                Graha::Moon,
                Graha::Mars,
                Graha::Mercury,
                Graha::Venus,
            ],
        ),
    ];
    for (lord, friends) in printed {
        for (other, _) in printed {
            if other != lord {
                assert_eq!(
                    calls_friend(lord, other),
                    friends.contains(&other),
                    "{lord:?} {other:?}"
                );
            }
        }
    }
    // Leo (the Sun) and Sagittarius (Jupiter): the Sun calls Jupiter a
    // friend and Jupiter the Sun, so the lords agree either way.
    let (leo, sagittarius) = (at(N::Magha, 1), at(N::Mula, 1));
    assert!(
        row(
            leo,
            sagittarius,
            Koota::GrahaMaitri,
            PoruthamRules::default()
        )
        .agrees
    );
    // Aries (Mars) and Taurus (Venus): Mars calls Venus a friend, Venus
    // calls Mars one; Aries and Cancer (the Moon) agree only one way.
    let (aries, cancer) = (at(N::Ashwini, 1), at(N::Pushya, 1));
    let one_way = PoruthamRules {
        lords_friendship: LordsFriendship::OneWay,
        ..PoruthamRules::default()
    };
    assert!(!row(aries, cancer, Koota::GrahaMaitri, PoruthamRules::default()).agrees);
    assert!(!row(aries, cancer, Koota::GrahaMaitri, one_way).agrees);
    let (sun, moon) = (leo, cancer);
    assert!(!row(sun, moon, Koota::GrahaMaitri, PoruthamRules::default()).agrees);
    assert!(row(moon, at(N::Ardra, 1), Koota::GrahaMaitri, one_way).agrees);
}

#[test]
fn vasyam_is_p75_and_never_a_sign_to_itself() {
    let printed: [(Rashi, &[Rashi]); 12] = [
        (Rashi::Aries, &[Rashi::Leo, Rashi::Scorpio]),
        (Rashi::Taurus, &[Rashi::Cancer, Rashi::Leo]),
        (Rashi::Gemini, &[Rashi::Virgo]),
        (Rashi::Cancer, &[Rashi::Scorpio, Rashi::Sagittarius]),
        (Rashi::Leo, &[Rashi::Libra]),
        (Rashi::Virgo, &[Rashi::Gemini, Rashi::Pisces]),
        (Rashi::Libra, &[Rashi::Capricorn]),
        (Rashi::Scorpio, &[Rashi::Virgo, Rashi::Cancer]),
        (Rashi::Sagittarius, &[Rashi::Pisces]),
        (Rashi::Capricorn, &[Rashi::Aquarius, Rashi::Aries]),
        (Rashi::Aquarius, &[Rashi::Aries]),
        (Rashi::Pisces, &[Rashi::Capricorn]),
    ];
    for (to, signs) in printed {
        for sign in Rashi::ALL {
            assert_eq!(
                concordant(sign, to),
                signs.contains(&sign),
                "{sign:?} to {to:?}"
            );
        }
    }
}

#[test]
fn rasi_reads_the_brides_sign_odd_for_the_2nd_and_6th() {
    // p. 74's six felicitous 6ths, bride's sign first.
    let felicitous = [
        (Rashi::Aries, Rashi::Virgo),
        (Rashi::Sagittarius, Rashi::Taurus),
        (Rashi::Libra, Rashi::Pisces),
        (Rashi::Aquarius, Rashi::Cancer),
        (Rashi::Leo, Rashi::Capricorn),
        (Rashi::Gemini, Rashi::Scorpio),
    ];
    // A Moon in each sign: the 1st pada of the star whose start it holds.
    let sign = |rashi: Rashi| {
        let moon = Native::of_moon(f64::from(rashi.id()) * 30.0 + 15.0).unwrap();
        assert_eq!(moon.rashi, rashi);
        moon
    };
    for bride in Rashi::ALL {
        for apart in 1..=12_u16 {
            let groom = Rashi::ALL[usize::from((bride.id() + apart - 1) % 12)];
            let found = row(
                sign(bride),
                sign(groom),
                Koota::Bhakoot,
                PoruthamRules::default(),
            );
            let own = found.agrees && !found.lifted;
            let expected = match apart {
                2 => groom.id() % 2 == 1,
                6 => felicitous.contains(&(bride, groom)),
                3..=5 => false,
                _ => true,
            };
            assert_eq!(own, expected, "{bride:?} {groom:?}");
        }
    }
}

#[test]
fn dhinam_reads_the_count_and_its_rounds() {
    let rules = PoruthamRules::default();
    let bride = at(N::Ashwini, 1);
    let agrees = |star: Nakshatra, pada: u16| dhinam(bride, at(star, pada), rules);
    // First nine: 3, 5 and 7 disagree; 9 agrees (C267).
    assert_eq!(agrees(N::Krittika, 2), (DhinamRule::Count, false));
    assert_eq!(agrees(N::Ardra, 1), (DhinamRule::Count, true));
    assert_eq!(agrees(N::Ashlesha, 1), (DhinamRule::Count, true));
    // Second nine: the 12th's 1st quarter only (C268).
    assert_eq!(
        agrees(N::UttaraPhalguni, 1),
        (DhinamRule::SecondRoundQuarter, false)
    );
    assert_eq!(agrees(N::UttaraPhalguni, 2), (DhinamRule::Count, true));
    assert_eq!(
        agrees(N::Chitra, 4),
        (DhinamRule::SecondRoundQuarter, false)
    );
    assert_eq!(
        agrees(N::Vishakha, 3),
        (DhinamRule::SecondRoundQuarter, false)
    );
    // Third nine: the 22nd and the 27th across signs.
    assert_eq!(agrees(N::Shravana, 1), (DhinamRule::VadhaVainasika, false));
    assert_eq!(agrees(N::Dhanishtha, 1), (DhinamRule::Count, true));
    assert_eq!(agrees(N::Revati, 1), (DhinamRule::TwentySeventh, false));
}

#[test]
fn p70s_pairs_are_7th_counts() {
    // The six unhappy pairs disagree; the four happy ones agree either way
    // round (C269).
    let rules = PoruthamRules::default();
    for (a, b) in [
        (N::Krittika, N::Ashlesha),
        (N::Ashlesha, N::Swati),
        (N::Chitra, N::PurvaAshadha),
        (N::Anuradha, N::Dhanishtha),
        (N::Dhanishtha, N::Bharani),
        (N::Shatabhisha, N::Krittika),
    ] {
        assert_eq!(
            dhinam(at(a, 1), at(b, 1), rules),
            (DhinamRule::Count, false),
            "{a:?} {b:?}"
        );
    }
    for (a, b) in [
        (N::Ardra, N::UttaraPhalguni),
        (N::PurvaPhalguni, N::Anuradha),
        (N::Chitra, N::Pushya),
        (N::Punarvasu, N::Hasta),
    ] {
        for (bride, groom) in [(a, b), (b, a)] {
            assert_eq!(
                dhinam(at(bride, 2), at(groom, 2), rules),
                (DhinamRule::HappyPair, true)
            );
        }
    }
}

#[test]
fn one_star_and_one_sign_follow_p71() {
    let rules = PoruthamRules::default();
    // Common stars by their lists.
    assert_eq!(
        dhinam(at(N::Rohini, 1), at(N::Rohini, 3), rules),
        (DhinamRule::CommonExcellent, true)
    );
    assert_eq!(
        dhinam(at(N::Pushya, 1), at(N::Pushya, 1), rules),
        (DhinamRule::CommonNeutral, true)
    );
    assert_eq!(
        dhinam(at(N::Mula, 1), at(N::Mula, 1), rules),
        (DhinamRule::CommonAvoid, false)
    );
    // Krittika spans Aries (1st quarter) and Taurus: the groom's quarter
    // earlier by default, the bride's in the first sign by the knob (C270).
    let (aries, taurus) = (at(N::Krittika, 1), at(N::Krittika, 3));
    assert_eq!(dhinam(taurus, aries, rules), (DhinamRule::TwoSigns, true));
    assert_eq!(dhinam(aries, taurus, rules), (DhinamRule::TwoSigns, false));
    let bride_first = PoruthamRules {
        two_sign_star: TwoSignStar::BrideFirstSign,
        ..rules
    };
    assert_eq!(
        dhinam(aries, taurus, bride_first),
        (DhinamRule::TwoSigns, true)
    );
    // Chitra splits two and two: the groom first under either reading.
    let (virgo, libra) = (at(N::Chitra, 2), at(N::Chitra, 3));
    assert_eq!(
        dhinam(libra, virgo, bride_first),
        (DhinamRule::TwoSigns, true)
    );
    // Two stars in one sign: the groom's prior, or next after a named one.
    assert_eq!(
        dhinam(at(N::Bharani, 1), at(N::Ashwini, 1), rules),
        (DhinamRule::SameSign, true)
    );
    assert_eq!(
        dhinam(at(N::Ashwini, 1), at(N::Bharani, 1), rules),
        (DhinamRule::NextStar, true)
    );
    assert_eq!(
        dhinam(at(N::Rohini, 1), at(N::Mrigashira, 1), rules),
        (DhinamRule::SameSign, false)
    );
    // Bharani and Krittika are left out, read by the count.
    assert_eq!(
        dhinam(at(N::Bharani, 1), at(N::Krittika, 1), rules),
        (DhinamRule::Count, true)
    );
}

#[test]
fn the_exception_lifts_four_and_says_so() {
    // One star in one sign: the same Rajju, lifted by the one lord.
    let moon = at(N::Rohini, 1);
    let ten = porutham(moon, moon, PoruthamRules::default());
    assert!(ten.exception.one_lord && ten.exception.holds());
    let rajju = ten.row(Koota::Rajju).unwrap();
    assert!(rajju.agrees && rajju.lifted);
    // Never Dhinam, Mahendra or the rest: Sthree-Dheergham stays at the 1st.
    let deergha = ten.row(Koota::StreeDeergha).unwrap();
    assert!(!deergha.agrees && !deergha.lifted);
    // Ashvini and Jyeshtha pierce; Aries and Scorpio share Mars.
    let ten = porutham(
        at(N::Ashwini, 1),
        at(N::Jyeshtha, 1),
        PoruthamRules::default(),
    );
    assert!(ten.row(Koota::Vedha).unwrap().lifted);
}

#[test]
fn the_counts_read_the_rules() {
    let bride = at(N::Ashwini, 1);
    let rules = PoruthamRules::default();
    for star in Nakshatra::ALL {
        let ten = porutham(bride, at(star, 2), rules);
        assert!(
            ten.considerations
                .iter()
                .map(|row| row.reading.koota())
                .eq(PORUTHAM)
        );
        let count = star.id() + 1;
        assert_eq!(
            ten.row(Koota::Mahendra).unwrap().agrees,
            [4, 7, 10, 13, 16, 19, 22, 25].contains(&count)
        );
        assert_eq!(ten.row(Koota::StreeDeergha).unwrap().agrees, count > 13);
        let seventh = PoruthamRules {
            deergha_beyond: DeerghaBeyond::Seventh,
            ..rules
        };
        assert_eq!(
            row(bride, at(star, 2), Koota::StreeDeergha, seventh).agrees,
            count > 7
        );
        let agreeing = ten.considerations.iter().filter(|row| row.agrees).count();
        assert_eq!(usize::from(ten.agreeing), agreeing);
        assert!(ten.chief_agreeing <= ten.agreeing.min(5));
    }
}
