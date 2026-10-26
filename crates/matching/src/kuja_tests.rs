//! The Kuja dosha held to *Manasagari*'s jāyābhāva v. 4.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index their own results"
)]

use teistro_core::catalogue::Rashi;

use crate::{
    KUJA_REFERENCES, KujaFrom, KujaHouses, KujaLifts, KujaNative, KujaReference, KujaRules,
    KujaSky, kuja, kuja_side,
};

/// A native with every sign given, Mars last.
fn native(lagna: Rashi, moon: Rashi, venus: Rashi, mars: Rashi) -> KujaNative {
    KujaNative {
        lagna,
        moon,
        venus,
        mars,
        sky: None,
    }
}

#[test]
fn the_verse_names_the_lagna_the_12th_4th_7th_and_8th() {
    // lagne vyaye ca pātāle yāmitre cāṣṭame kujaḥ: the verse's order.
    let mut named = [1, 12, 4, 7, 8];
    named.sort_unstable();
    assert_eq!(KujaHouses::Manasagari.houses(), named);
    assert_eq!(KujaHouses::WithSecond.houses(), [1, 2, 4, 7, 8, 12]);
}

#[test]
fn a_house_is_counted_by_sign_from_its_reference() {
    for lagna in Rashi::ALL {
        for mars in Rashi::ALL {
            let one = native(lagna, Rashi::Aries, Rashi::Aries, mars);
            let house = one.mars_house(KujaReference::Lagna);
            // Counting the house on from the lagna lands on Mars.
            let landed = Rashi::ALL[usize::from((lagna.id() + u16::from(house) - 1) % 12)];
            assert_eq!(landed, mars, "{lagna:?} {mars:?}");
            assert!((1..=12).contains(&house));
        }
    }
    // Mars in the lagna's own sign is the 1st, the verse's *lagne*.
    let one = native(Rashi::Leo, Rashi::Leo, Rashi::Leo, Rashi::Leo);
    assert!(
        KUJA_REFERENCES
            .iter()
            .all(|&from| one.mars_house(from) == 1)
    );
}

#[test]
fn five_of_twelve_places_carry_the_dosha_and_the_2nd_adds_one() {
    let count = |rules: KujaRules| {
        Rashi::ALL
            .iter()
            .filter(|&&mars| {
                kuja_side(
                    native(Rashi::Aries, Rashi::Aries, Rashi::Aries, mars),
                    rules,
                )
                .dosha
            })
            .count()
    };
    assert_eq!(count(KujaRules::default()), 5);
    let with_second = KujaRules {
        houses: KujaHouses::WithSecond,
        ..KujaRules::default()
    };
    assert_eq!(count(with_second), 6);
}

#[test]
fn the_moon_and_venus_count_only_when_the_rules_ask() {
    // Mars in the 2nd from the lagna, the 7th from the Moon, the 1st from
    // Venus.
    let one = native(Rashi::Aries, Rashi::Scorpio, Rashi::Taurus, Rashi::Taurus);
    let verse = kuja_side(one, KujaRules::default());
    let houses: Vec<_> = verse.readings.iter().map(|reading| reading.house).collect();
    assert_eq!(houses, [2, 7, 1]);
    let order: Vec<_> = verse.readings.iter().map(|reading| reading.from).collect();
    assert_eq!(order, KUJA_REFERENCES);
    // The facts are reported whatever the rules count.
    assert!(!verse.readings[0].in_houses && verse.readings[1].in_houses);
    assert!(!verse.dosha, "the lagna alone counts by default");
    let all = KujaRules {
        from: KujaFrom::LagnaMoonVenus,
        ..KujaRules::default()
    };
    assert!(kuja_side(one, all).dosha);
}

#[test]
fn both_is_a_clause_and_lifts_nothing() {
    let seventh = native(Rashi::Aries, Rashi::Aries, Rashi::Aries, Rashi::Libra);
    let third = native(Rashi::Aries, Rashi::Aries, Rashi::Aries, Rashi::Gemini);
    let rules = KujaRules::default();
    let both = kuja(seventh, seventh, rules);
    assert!(both.bride.dosha && both.groom.dosha && both.both);
    let one = kuja(seventh, third, rules);
    assert!(one.bride.dosha && !one.groom.dosha && !one.both);
}

#[test]
fn a_longitude_that_is_not_a_number_is_named() {
    let error = KujaNative::of_longitudes(0.0, 0.0, f64::NAN, 0.0).unwrap_err();
    assert_eq!(error.field(), Some("venus"));
    let wrapped = KujaNative::of_longitudes(-30.0, 365.0, 0.0, 719.0).unwrap();
    assert_eq!(
        (wrapped.lagna, wrapped.moon, wrapped.mars),
        (Rashi::Pisces, Rashi::Aries, Rashi::Pisces)
    );
}

#[test]
fn the_rules_and_the_answer_keep_their_wire_spelling() {
    let rules: KujaRules =
        serde_json::from_str(r#"{"houses":"WITH_SECOND","from":"LAGNA_MOON_VENUS"}"#).unwrap();
    assert_eq!(
        rules,
        KujaRules {
            houses: KujaHouses::WithSecond,
            from: KujaFrom::LagnaMoonVenus,
            lifts: KujaLifts::NONE,
        }
    );
    assert!(serde_json::from_str::<KujaRules>(r#"{"house":"WITH_SECOND"}"#).is_err());
    let one = native(Rashi::Aries, Rashi::Aries, Rashi::Aries, Rashi::Libra);
    let read = serde_json::to_value(kuja(one, one, KujaRules::default())).unwrap();
    assert_eq!(read["bride"]["readings"][0]["from"], "LAGNA");
    assert_eq!(read["bride"]["readings"][0]["inHouses"], true);
    assert_eq!(read["both"], true);
}

/// Signs as longitudes, at a sign's middle.
fn at(sign: Rashi) -> f64 {
    f64::from(sign.id()) * 30.0 + 15.0
}

/// A sky with the Sun in `sun`, a Moon `ahead` signs past it, Mercury,
/// Jupiter, Saturn and Rahu where given.
fn sky(
    sun: Rashi,
    ahead: u16,
    mercury: Rashi,
    jupiter: Rashi,
    saturn: Rashi,
    rahu: Rashi,
) -> KujaSky {
    let moon = at(sun) + f64::from(ahead) * 30.0;
    KujaSky::of_longitudes(
        at(sun),
        moon,
        at(mercury),
        at(jupiter),
        at(saturn),
        at(rahu),
    )
    .unwrap()
}

/// Mars in the 7th from an Aries lagna, the Moon in Taurus and Venus in
/// Gemini, under a sky that lifts nothing: the Sun and Mercury in Pisces,
/// the Moon waning, Jupiter in Cancer, Saturn in Leo, Rahu in Taurus.
fn bare() -> (KujaNative, KujaSky) {
    let native = native(Rashi::Aries, Rashi::Taurus, Rashi::Gemini, Rashi::Libra);
    (
        native,
        sky(
            Rashi::Pisces,
            8,
            Rashi::Pisces,
            Rashi::Cancer,
            Rashi::Leo,
            Rashi::Taurus,
        ),
    )
}

#[test]
fn a_sky_that_lifts_nothing_holds_no_lift() {
    let (native, sky) = bare();
    let side = kuja_side(native.with_sky(sky), KujaRules::default());
    assert!(side.dosha);
    assert_eq!(side.held, Some(KujaLifts::NONE));
    assert!(!side.lifted);
    // Without a sky nothing is read, which is not the same answer.
    assert_eq!(kuja_side(native, KujaRules::default()).held, None);
}

#[test]
fn each_benefic_lifts_by_its_aspect_or_its_conjunction() {
    let (native, bare_sky) = bare();
    let held = |native: KujaNative, sky: KujaSky| {
        kuja_side(native.with_sky(sky), KujaRules::default())
            .held
            .unwrap()
    };
    // Jupiter in Aquarius aspects Libra by its 9th: a benefic's aspect
    // and Jupiter's own clause.
    let jupiter = held(
        native,
        KujaSky {
            jupiter: Rashi::Aquarius,
            ..bare_sky
        },
    );
    assert!(jupiter.benefic && jupiter.jupiter_aspect);
    assert_eq!(jupiter.count(), 2);
    // Venus with Mars is a benefic's conjunction, and no Jupiter clause.
    let venus = held(
        KujaNative {
            venus: Rashi::Libra,
            ..native
        },
        bare_sky,
    );
    assert_eq!(
        venus,
        KujaLifts {
            benefic: true,
            ..KujaLifts::NONE
        }
    );
    // Mercury opposite Mars aspects it alone, not beside the Sun.
    let mercury = KujaSky {
        mercury: Rashi::Aries,
        ..bare_sky
    };
    assert!(held(native, mercury).benefic);
    assert!(
        !held(
            native,
            KujaSky {
                sun: Rashi::Aries,
                ..mercury
            }
        )
        .benefic
    );
    // The Moon opposite Mars lifts it only while it waxes.
    let opposite = KujaNative {
        moon: Rashi::Aries,
        ..native
    };
    let waning = KujaSky {
        moon_waxing: false,
        ..bare_sky
    };
    assert!(
        held(
            opposite,
            KujaSky {
                moon_waxing: true,
                ..bare_sky
            }
        )
        .benefic
    );
    assert!(!held(opposite, waning).benefic);
}

#[test]
fn the_almanacs_five_hold_where_their_clauses_read() {
    let (native, bare_sky) = bare();
    let held = |native: KujaNative, sky: KujaSky| {
        kuja_side(native.with_sky(sky), KujaRules::default())
            .held
            .unwrap()
    };
    // The almanacs' five.
    for (house, holds) in [
        (1, true),
        (4, true),
        (7, true),
        (9, true),
        (12, true),
        (10, false),
    ] {
        let saturn = Rashi::of_longitude(f64::from(house - 1) * 30.0 + 15.0);
        assert_eq!(
            held(native, KujaSky { saturn, ..bare_sky }).saturn,
            holds,
            "Saturn in {house}"
        );
    }
    let second = KujaNative {
        moon: Rashi::Taurus,
        venus: Rashi::Taurus,
        ..native
    };
    assert!(held(second, bare_sky).moon_venus_second);
    assert!(
        !held(
            KujaNative {
                venus: Rashi::Gemini,
                ..second
            },
            bare_sky
        )
        .moon_venus_second
    );
    assert!(
        held(
            native,
            KujaSky {
                rahu: Rashi::Capricorn,
                ..bare_sky
            }
        )
        .rahu_kendra
    );
    let with_rahu = held(
        native,
        KujaSky {
            rahu: Rashi::Libra,
            ..bare_sky
        },
    );
    assert!(with_rahu.with_rahu && with_rahu.rahu_kendra && !with_rahu.benefic);
}

#[test]
fn a_lift_lifts_only_where_the_rules_count_it() {
    let (native, sky) = bare();
    let native = native.with_sky(KujaSky {
        jupiter: Rashi::Aquarius,
        ..sky
    });
    let under = |lifts| {
        kuja_side(
            native,
            KujaRules {
                lifts,
                ..KujaRules::default()
            },
        )
    };
    assert!(!under(KujaLifts::NONE).lifted, "the verse lifts nothing");
    assert!(under(KujaLifts::PARASHARA).lifted);
    assert!(under(KujaLifts::PRINTED_IN_NEPAL).lifted);
    // Mars in the 3rd carries no dosha, so nothing is lifted.
    let free = KujaNative {
        mars: Rashi::Gemini,
        ..native
    };
    assert!(
        !kuja_side(
            free,
            KujaRules {
                lifts: KujaLifts::PARASHARA,
                ..KujaRules::default()
            }
        )
        .lifted
    );
}

#[test]
fn both_charts_carrying_it_lift_each_under_parashara() {
    let (native, sky) = bare();
    let native = native.with_sky(sky);
    let rules = KujaRules {
        lifts: KujaLifts::PARASHARA,
        ..KujaRules::default()
    };
    let read = kuja(native, native, rules);
    assert!(read.both && read.bride.lifted && read.groom.lifted);
    assert!(read.bride.held.unwrap().mutual);
    // One side alone: no mutual lift.
    let free = KujaNative {
        mars: Rashi::Gemini,
        ..native
    };
    let one = kuja(native, free, rules);
    assert!(!one.both && !one.bride.lifted && !one.bride.held.unwrap().mutual);
    // Mutual needs no sky.
    let bare_native = KujaNative {
        sky: None,
        ..native
    };
    assert!(kuja(bare_native, bare_native, rules).bride.lifted);
}

#[test]
fn the_lifts_cross_as_bits_and_back() {
    for bits in 0..0x80_u8 {
        assert_eq!(KujaLifts::from_bits(bits).bits(), bits);
    }
    let json = serde_json::to_string(&KujaLifts::PARASHARA).unwrap();
    assert_eq!(
        json,
        r#"{"benefic":true,"mutual":true,"saturn":false,"moonVenusSecond":false,"jupiterAspect":false,"rahuKendra":false,"withRahu":false}"#
    );
    let rules: KujaRules = serde_json::from_str(r#"{"lifts":{"jupiterAspect":true}}"#).unwrap();
    assert!(rules.lifts.jupiter_aspect && rules.lifts.count() == 1);
    assert!(serde_json::from_str::<KujaLifts>(r#"{"jupiter":true}"#).is_err());
}
