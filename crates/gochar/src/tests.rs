#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they built"
)]

use super::*;

/// v. 2's good houses, written out from the verse on their own so the
/// vedha table's first column is checked against it rather than itself.
const GOOD: [(Graha, &[u8]); 9] = [
    (Graha::Sun, &[6, 3, 10, 11]),
    (Graha::Moon, &[3, 10, 6, 7, 1, 11]),
    (Graha::Mars, &[6, 3, 11]),
    (Graha::Mercury, &[6, 2, 4, 10, 8, 11]),
    (Graha::Jupiter, &[7, 9, 2, 5, 11]),
    (Graha::Venus, &[1, 2, 3, 4, 5, 8, 9, 11, 12]),
    (Graha::Saturn, &[6, 3, 11]),
    (Graha::Rahu, &[6, 3, 10, 11]),
    (Graha::Ketu, &[6, 3, 10, 11]),
];

/// The sign `house` houses from `reference`.
fn in_house(reference: Rashi, house: u8) -> Rashi {
    Rashi::ALL[(reference as usize + usize::from(house) - 1) % 12]
}

/// Every graha parked in `park`, a house nobody's table pairs with the
/// others' houses in these tests, but `placed`.
fn transits(reference: Rashi, placed: &[(Graha, u8)], park: u8) -> [Transit; 9] {
    let mut out = [Transit::new(in_house(reference, park), 15.0); 9];
    for (graha, house) in placed {
        out[*graha as usize] = Transit::new(in_house(reference, *house), 15.0);
    }
    out
}

#[test]
fn every_good_house_is_v_2_s_and_no_other() {
    let reference = Rashi::Leo;
    for (graha, good) in GOOD {
        for house in 1..=12_u8 {
            // Park the others where they can obstruct nothing in question:
            // the graha's own house, which no pair uses as its vedha.
            let reading = gochar(
                reference,
                &transits(reference, &[(graha, house)], house),
                GocharRules::TEXT,
            );
            let read = &reading.grahas[graha as usize];
            assert_eq!(read.house, house);
            assert_eq!(
                read.good_house,
                good.contains(&house),
                "{graha:?} in {house}"
            );
            if !read.good_house {
                assert_eq!(read.verdict, Verdict::NotGood, "{graha:?} in {house}");
                assert_eq!(read.vedha_house, None);
            }
        }
    }
}

#[test]
fn every_vedha_pair_obstructs_and_every_exemption_does_not() {
    let reference = Rashi::Capricorn;
    for graha in GRAHAS {
        for (good, vedha) in pairs(graha, GocharRules::TEXT) {
            let vedha = vedha.unwrap();
            for other in GRAHAS.into_iter().filter(|other| *other != graha) {
                // `graha` in its good house, `other` in the vedha house, the
                // rest in the good house beside `graha`, which no vedha is.
                let reading = gochar(
                    reference,
                    &transits(reference, &[(graha, good), (other, vedha)], good),
                    GocharRules::TEXT,
                );
                let read = &reading.grahas[graha as usize];
                assert_eq!(read.vedha_house, Some(vedha), "{graha:?} in {good}");
                if exempt(graha) == Some(other) {
                    assert_eq!(
                        read.verdict,
                        Verdict::Good,
                        "{other:?} exempt for {graha:?}"
                    );
                    assert!(read.obstructed_by.is_empty());
                } else {
                    assert_eq!(read.verdict, Verdict::Obstructed, "{other:?} for {graha:?}");
                    assert_eq!(read.obstructed_by, vec![other]);
                }
            }
        }
    }
}

#[test]
fn the_exemptions_are_mutual_as_the_verses_state_them() {
    for (a, b) in [(Graha::Sun, Graha::Saturn), (Graha::Moon, Graha::Mercury)] {
        assert_eq!(exempt(a), Some(b));
        assert_eq!(exempt(b), Some(a));
    }
    for graha in [
        Graha::Mars,
        Graha::Jupiter,
        Graha::Venus,
        Graha::Rahu,
        Graha::Ketu,
    ] {
        assert_eq!(exempt(graha), None, "{graha:?}");
    }
}

/// v. 8's translation phrases Venus's houses as "bad effects … if he is
/// marred"; its Sanskrit makes an obstructed Venus inauspicious in them,
/// which is the same rule: good in nine houses unless obstructed.
#[test]
fn venus_is_good_in_nine_houses_unless_obstructed() {
    let reference = Rashi::Aries;
    let reading = gochar(
        reference,
        &transits(reference, &[(Graha::Venus, 2), (Graha::Mars, 7)], 4),
        GocharRules::TEXT,
    );
    let venus = &reading.grahas[Graha::Venus as usize];
    assert_eq!(venus.vedha_house, Some(7));
    assert_eq!(venus.verdict, Verdict::Obstructed);
    assert_eq!(venus.obstructed_by, vec![Graha::Mars]);
}

#[test]
fn the_nodes_follow_the_settings() {
    let reference = Rashi::Aries;
    // Rahu in the 3rd, Mars in its vedha house the 9th; Ketu in the 9th
    // too, standing where it could obstruct the Sun in the 3rd.
    let placed = transits(
        reference,
        &[
            (Graha::Rahu, 3),
            (Graha::Mars, 9),
            (Graha::Sun, 3),
            (Graha::Ketu, 9),
        ],
        5,
    );
    let text = gochar(reference, &placed, GocharRules::TEXT);
    assert_eq!(
        text.grahas[Graha::Rahu as usize].verdict,
        Verdict::Obstructed
    );
    assert_eq!(
        text.grahas[Graha::Sun as usize].obstructed_by,
        vec![Graha::Mars, Graha::Ketu]
    );
    let unobstructed = GocharRules {
        node_vedha: NodeVedha::None,
        ..GocharRules::TEXT
    };
    let rahu = &gochar(reference, &placed, unobstructed).grahas[Graha::Rahu as usize];
    assert_eq!((rahu.verdict, rahu.vedha_house), (Verdict::Good, None));
    let seven = GocharRules {
        node_obstruction: NodeObstruction::None,
        ..GocharRules::TEXT
    };
    let sun = &gochar(reference, &placed, seven).grahas[Graha::Sun as usize];
    assert_eq!(sun.obstructed_by, vec![Graha::Mars]);
}

#[test]
fn a_transit_bears_fruit_in_its_decanate() {
    for (graha, fruitful, barren) in [
        (Graha::Sun, 0.0, 10.0),
        (Graha::Mars, 9.999, 10.0),
        (Graha::Jupiter, 10.0, 20.0),
        (Graha::Venus, 19.999, 9.999),
        (Graha::Moon, 20.0, 19.999),
        (Graha::Saturn, 29.999, 0.0),
    ] {
        assert!(Fruition::of(graha).holds(fruitful), "{graha:?} {fruitful}");
        assert!(!Fruition::of(graha).holds(barren), "{graha:?} {barren}");
    }
    for graha in [Graha::Mercury, Graha::Rahu, Graha::Ketu] {
        assert_eq!(Fruition::of(graha), Fruition::Throughout);
    }
}

#[test]
fn a_longitude_splits_into_its_sign_and_degrees() {
    let at = Transit::at_longitude(47.5);
    assert_eq!((at.sign, at.degrees), (Rashi::Taurus, 17.5));
    assert_eq!(Transit::at_longitude(-0.5).sign, Rashi::Pisces);
    assert_eq!(Transit::at_longitude(360.0).sign, Rashi::Aries);
}
