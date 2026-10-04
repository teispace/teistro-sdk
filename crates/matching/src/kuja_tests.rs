//! The Kuja dosha held to *Manasagari*'s jāyābhāva v. 4.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index their own results"
)]

use teistro_core::catalogue::Rashi;

use crate::{
    KUJA_REFERENCES, KujaFrom, KujaHouses, KujaNative, KujaReference, KujaRules, kuja, kuja_side,
};

/// A native with every sign given, Mars last.
fn native(lagna: Rashi, moon: Rashi, venus: Rashi, mars: Rashi) -> KujaNative {
    KujaNative {
        lagna,
        moon,
        venus,
        mars,
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
            from: KujaFrom::LagnaMoonVenus
        }
    );
    assert!(serde_json::from_str::<KujaRules>(r#"{"house":"WITH_SECOND"}"#).is_err());
    let one = native(Rashi::Aries, Rashi::Aries, Rashi::Aries, Rashi::Libra);
    let read = serde_json::to_value(kuja(one, one, KujaRules::default())).unwrap();
    assert_eq!(read["bride"]["readings"][0]["from"], "LAGNA");
    assert_eq!(read["bride"]["readings"][0]["inHouses"], true);
    assert_eq!(read["both"], true);
}
