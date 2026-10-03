//! The tables held to their verses, and the kootas' symmetries.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    reason = "tests fail by panicking, index their own results and compare exact halves"
)]

use teistro_core::catalogue::{Gana, Graha, Koota, Nadi, Nakshatra, Rashi, Yoni};

use crate::{
    ASHTA_KOOTA, ASHTA_KOOTA_POINTS, BhakootDosha, BhakootLift, DevaBride, EqualVarna,
    KootaReading, KootaRules, MaitriRelation, NadiDosha, Native, VashyaRelation, YoniRelation,
    ashta_koota, maitri_relation, vashya_relation, yoni_relation,
};

use Nakshatra as N;

/// A Moon in the middle of a nakshatra's first pada.
fn moon(star: Nakshatra) -> Native {
    Native::of_moon(f64::from(star.id()) * 360.0 / 27.0 + 1.0).unwrap()
}

fn reading(
    bride: Nakshatra,
    groom: Nakshatra,
    koota: Koota,
    rules: KootaRules,
) -> (f64, KootaReading) {
    let row = *ashta_koota(moon(bride), moon(groom), rules)
        .row(koota)
        .unwrap();
    (row.points, row.reading)
}

#[test]
fn the_ganas_are_the_verses() {
    // VI.29.
    let rakshasa = [
        N::Magha,
        N::Ashlesha,
        N::Dhanishtha,
        N::Jyeshtha,
        N::Mula,
        N::Shatabhisha,
        N::Krittika,
        N::Chitra,
        N::Vishakha,
    ];
    let manushya = [
        N::PurvaPhalguni,
        N::PurvaAshadha,
        N::PurvaBhadrapada,
        N::UttaraPhalguni,
        N::UttaraAshadha,
        N::UttaraBhadrapada,
        N::Rohini,
        N::Bharani,
        N::Ardra,
    ];
    for star in Nakshatra::ALL {
        let gana = star.attributes().gana;
        let want = if rakshasa.contains(&star) {
            Gana::Rakshasa
        } else if manushya.contains(&star) {
            Gana::Manushya
        } else {
            Gana::Deva
        };
        assert_eq!(gana, want, "{star:?}");
    }
}

#[test]
fn the_nadis_are_the_verses() {
    // VI.34.
    let adi = [
        N::Jyeshtha,
        N::Mula,
        N::UttaraPhalguni,
        N::Hasta,
        N::Ardra,
        N::Punarvasu,
        N::Shatabhisha,
        N::PurvaBhadrapada,
        N::Ashwini,
    ];
    let madhya = [
        N::Pushya,
        N::Mrigashira,
        N::Chitra,
        N::Anuradha,
        N::Bharani,
        N::Dhanishtha,
        N::PurvaAshadha,
        N::PurvaPhalguni,
        N::UttaraBhadrapada,
    ];
    for star in Nakshatra::ALL {
        let want = if adi.contains(&star) {
            Nadi::Aadi
        } else if madhya.contains(&star) {
            Nadi::Madhya
        } else {
            Nadi::Antya
        };
        assert_eq!(star.attributes().nadi, want, "{star:?}");
    }
}

#[test]
fn the_yonis_are_the_verses() {
    // VI.25–26, the commentary's aja for the verse's mesha.
    let pairs: [(Yoni, &[Nakshatra]); 14] = [
        (Yoni::Horse, &[N::Ashwini, N::Shatabhisha]),
        (Yoni::Buffalo, &[N::Swati, N::Hasta]),
        (Yoni::Lion, &[N::Dhanishtha, N::PurvaBhadrapada]),
        (Yoni::Elephant, &[N::Bharani, N::Revati]),
        (Yoni::Goat, &[N::Pushya, N::Krittika]),
        (Yoni::Monkey, &[N::Shravana, N::PurvaAshadha]),
        (Yoni::Mongoose, &[N::UttaraAshadha]),
        (Yoni::Serpent, &[N::Mrigashira, N::Rohini]),
        (Yoni::Deer, &[N::Jyeshtha, N::Anuradha]),
        (Yoni::Dog, &[N::Mula, N::Ardra]),
        (Yoni::Cat, &[N::Punarvasu, N::Ashlesha]),
        (Yoni::Rat, &[N::Magha, N::PurvaPhalguni]),
        (Yoni::Tiger, &[N::Vishakha, N::Chitra]),
        (Yoni::Cow, &[N::UttaraPhalguni, N::UttaraBhadrapada]),
    ];
    for (yoni, stars) in pairs {
        for star in stars {
            assert_eq!(star.attributes().yoni, yoni, "{star:?}");
        }
    }
    assert_eq!(
        pairs.iter().map(|(_, stars)| stars.len()).sum::<usize>(),
        27
    );
    assert_eq!(
        yoni_relation(Yoni::Cow, Yoni::Tiger),
        YoniRelation::GreatEnemy
    );
    assert_eq!(yoni_relation(Yoni::Cow, Yoni::Horse), YoniRelation::Neutral);
}

#[test]
fn the_friendships_are_the_verses() {
    // VI.27–28: each lord's friends and enemies, the rest neutral.
    use Graha as G;
    let verse: [(Graha, &[Graha], &[Graha]); 7] = [
        (
            G::Sun,
            &[G::Mars, G::Jupiter, G::Moon],
            &[G::Venus, G::Saturn],
        ),
        (G::Moon, &[G::Mercury, G::Sun], &[]),
        (G::Mars, &[G::Moon, G::Jupiter, G::Sun], &[G::Mercury]),
        (G::Mercury, &[G::Venus, G::Sun], &[G::Moon]),
        (
            G::Jupiter,
            &[G::Sun, G::Mars, G::Moon],
            &[G::Mercury, G::Venus],
        ),
        (G::Venus, &[G::Mercury, G::Saturn], &[G::Moon, G::Sun]),
        (
            G::Saturn,
            &[G::Venus, G::Mercury],
            &[G::Moon, G::Sun, G::Mars],
        ),
    ];
    for (graha, friends, enemies) in verse {
        for (other, _, _) in verse {
            if other == graha {
                continue;
            }
            let attributes = graha.attributes();
            assert_eq!(
                attributes.friends.contains(&other),
                friends.contains(&other),
                "{graha:?} of {other:?}"
            );
            assert_eq!(
                attributes.enemies.contains(&other),
                enemies.contains(&other),
                "{graha:?} of {other:?}"
            );
        }
    }
    assert_eq!(
        maitri_relation(G::Sun, G::Saturn),
        MaitriRelation::MutualEnemies
    );
    assert_eq!(
        maitri_relation(G::Moon, G::Saturn),
        MaitriRelation::NeutralEnemy
    );
    assert_eq!(maitri_relation(G::Mars, G::Mars), MaitriRelation::OneLord);
}

#[test]
fn the_verse_decides_vashya_where_it_speaks() {
    // VI.23: every sign but Leo is vashya to the human signs, the water
    // signs their food; every sign but Scorpio is vashya to Leo.
    assert_eq!(
        vashya_relation(Rashi::Gemini, Rashi::Pisces),
        VashyaRelation::Food
    );
    assert_eq!(
        vashya_relation(Rashi::Leo, Rashi::Scorpio),
        VashyaRelation::Neither
    );
    assert_eq!(
        vashya_relation(Rashi::Virgo, Rashi::Libra),
        VashyaRelation::Mutual
    );
    // Kalaprakasika's table: Scorpio is vashya to Aries, not Aries to Scorpio.
    assert_eq!(
        vashya_relation(Rashi::Aries, Rashi::Scorpio),
        VashyaRelation::OneWay
    );
    for sign in Rashi::ALL {
        assert_eq!(
            vashya_relation(sign, sign),
            VashyaRelation::Mutual,
            "{sign:?}"
        );
    }
}

#[test]
fn a_symmetric_koota_is_symmetric_over_every_pair() {
    let rules = KootaRules::default();
    for bride in Nakshatra::ALL {
        for groom in Nakshatra::ALL {
            let there = ashta_koota(moon(bride), moon(groom), rules);
            let back = ashta_koota(moon(groom), moon(bride), rules);
            for koota in [
                Koota::Vashya,
                Koota::Tara,
                Koota::Yoni,
                Koota::GrahaMaitri,
                Koota::Bhakoot,
                Koota::Nadi,
            ] {
                assert_eq!(
                    there.row(koota).unwrap().points,
                    back.row(koota).unwrap().points,
                    "{koota:?} {bride:?} {groom:?}"
                );
            }
            assert!(there.total <= ASHTA_KOOTA_POINTS);
            assert!(
                there
                    .kootas
                    .iter()
                    .map(|row| row.reading.koota())
                    .eq(ASHTA_KOOTA)
            );
            let maxima: f64 = there.kootas.iter().map(|row| row.max_points).sum();
            assert_eq!(maxima, ASHTA_KOOTA_POINTS);
        }
    }
}

#[test]
fn varna_and_gana_are_directional() {
    let rules = KootaRules::default();
    // Cancer (Brahmin) and Gemini (Shudra): a higher groom earns the point.
    let cancer = Native::of_moon(95.0).unwrap();
    let gemini = Native::of_moon(65.0).unwrap();
    let points = |b: Native, g: Native| ashta_koota(b, g, rules).row(Koota::Varna).unwrap().points;
    assert_eq!((points(gemini, cancer), points(cancer, gemini)), (1.0, 0.0));
    let half = KootaRules {
        equal_varna: EqualVarna::Half,
        ..rules
    };
    assert_eq!(
        ashta_koota(cancer, cancer, half)
            .row(Koota::Varna)
            .unwrap()
            .points,
        0.5
    );
    // Daivajna-manohara's gana points.
    let gana = |b, g, rules: KootaRules| reading(b, g, Koota::Gana, rules).0;
    // Rohini Manushya, Ashvini Deva, Magha Rakshasa.
    assert_eq!(gana(N::Rohini, N::Ashwini, rules), 5.0);
    assert_eq!(gana(N::Ashwini, N::Rohini, rules), 4.0);
    let three = KootaRules {
        deva_bride: DevaBride::Three,
        ..rules
    };
    assert_eq!(gana(N::Ashwini, N::Rohini, three), 3.0);
    assert_eq!(gana(N::Ashwini, N::Magha, rules), 2.0);
    assert_eq!(gana(N::Rohini, N::Magha, rules), 1.0);
    assert_eq!(gana(N::Magha, N::Ashwini, rules), 0.0);
    assert_eq!(gana(N::Magha, N::Rohini, rules), 0.0);
}

#[test]
fn tara_counts_both_ways_by_nines() {
    let rules = KootaRules::default();
    // Ashvini to Krittika is the 3rd, bad; back is the 26th, the 8th, good.
    let (points, read) = reading(N::Ashwini, N::Krittika, Koota::Tara, rules);
    assert_eq!(
        read,
        KootaReading::Tara {
            bride_to_groom: 3,
            groom_to_bride: 8
        }
    );
    assert_eq!(points, 1.5);
}

#[test]
fn a_bad_bhakoot_names_its_dosha_and_its_exceptions() {
    let rules = KootaRules::default();
    // Aries and Virgo, 6/8 under Mars and Mercury, enemies one way.
    let (points, read) = reading(N::Ashwini, N::Hasta, Koota::Bhakoot, rules);
    let KootaReading::Bhakoot {
        apart,
        dosha,
        exceptions,
        lifted,
    } = read
    else {
        unreachable!()
    };
    assert_eq!(
        (points, apart, dosha),
        (0.0, 6, Some(BhakootDosha::SixEight))
    );
    assert!(!exceptions.one_lord && !exceptions.lords_friends);
    // Ashvini and Hasta share the first nadi, so nothing lifts it.
    assert!(!lifted);
    // Aries and Scorpio: one lord, lifted under the verse; Garga asks
    // three of maitri, tara and vashya.
    let (_, read) = reading(N::Ashwini, N::Anuradha, Koota::Bhakoot, rules);
    let KootaReading::Bhakoot {
        exceptions, lifted, ..
    } = read
    else {
        unreachable!()
    };
    assert!(exceptions.one_lord && lifted);
    let garga = KootaRules {
        bhakoot_lift: BhakootLift::Garga,
        ..rules
    };
    let (_, read) = reading(N::Ashwini, N::Anuradha, Koota::Bhakoot, garga);
    let KootaReading::Bhakoot {
        exceptions, lifted, ..
    } = read
    else {
        unreachable!()
    };
    let held = [true, exceptions.tara_pure, exceptions.vashya]
        .iter()
        .filter(|h| **h)
        .count();
    assert_eq!(lifted, held >= 3);
}

#[test]
fn a_shared_nadi_takes_its_points_and_the_middle_one_is_the_dosha_on_request() {
    let rules = KootaRules::default();
    // Ashvini and Ardra share the first nadi.
    let (points, read) = reading(N::Ashwini, N::Ardra, Koota::Nadi, rules);
    assert_eq!(
        (points, read),
        (
            0.0,
            KootaReading::Nadi {
                bride: Nadi::Aadi,
                groom: Nadi::Aadi,
                dosha: true,
                lifted: false
            }
        )
    );
    let middle = KootaRules {
        nadi_dosha: NadiDosha::MiddleOnly,
        ..rules
    };
    let (_, read) = reading(N::Ashwini, N::Ardra, Koota::Nadi, middle);
    assert_eq!(
        read,
        KootaReading::Nadi {
            bride: Nadi::Aadi,
            groom: Nadi::Aadi,
            dosha: false,
            lifted: false
        }
    );
    let (_, read) = reading(N::Bharani, N::Pushya, Koota::Nadi, middle);
    assert_eq!(
        read,
        KootaReading::Nadi {
            bride: Nadi::Madhya,
            groom: Nadi::Madhya,
            dosha: true,
            lifted: false
        }
    );
}

/// A Moon at a nakshatra's pada, mid-arc.
fn at(star: Nakshatra, pada: u16) -> Native {
    let quarter = star.id() * 4 + pada - 1;
    Native::of_moon((f64::from(quarter) + 0.5) * 360.0 / 108.0).unwrap()
}

#[test]
fn one_sign_or_one_star_lifts_the_nadi_and_the_gana_by_vi36() {
    let rules = KootaRules::default();
    let nadi = |bride: Native, groom: Native| match ashta_koota(bride, groom, rules)
        .row(Koota::Nadi)
        .unwrap()
        .reading
    {
        KootaReading::Nadi { dosha, lifted, .. } => (dosha, lifted),
        _ => unreachable!(),
    };
    // Krittika and Rohini in Taurus, both Antya: Keshavarka's example.
    assert_eq!(nadi(at(N::Krittika, 2), at(N::Rohini, 1)), (true, true));
    // Krittika across Aries and Taurus: one star, two signs.
    assert_eq!(nadi(at(N::Krittika, 1), at(N::Krittika, 3)), (true, true));
    // Bharani's two padas in one sign, the commentary's example.
    assert_eq!(nadi(at(N::Bharani, 1), at(N::Bharani, 2)), (true, true));
    // One pada of one star: nothing to lift.
    assert_eq!(nadi(at(N::Bharani, 1), at(N::Bharani, 1)), (true, false));
    // Two signs, two stars: the dosha stands.
    assert_eq!(nadi(at(N::Ashwini, 1), at(N::Ardra, 1)), (true, false));

    let gana = |bride: Native, groom: Native| match ashta_koota(bride, groom, rules)
        .row(Koota::Gana)
        .unwrap()
        .reading
    {
        KootaReading::Gana { dosha, lifted, .. } => (dosha, lifted),
        _ => unreachable!(),
    };
    // Krittika (Rakshasa) and Rohini (Manushya) in one sign.
    assert_eq!(gana(at(N::Krittika, 2), at(N::Rohini, 1)), (true, true));
    // Ashvini (Deva, Aries) and Magha (Rakshasa, Leo): Mars and the Sun are
    // friends both ways, the sign lords' friendship of VI.33.
    assert_eq!(gana(at(N::Ashwini, 1), at(N::Magha, 1)), (true, true));
    // The same gana is no dosha.
    assert_eq!(gana(at(N::Ashwini, 1), at(N::Pushya, 1)), (false, false));
}

#[test]
fn a_good_bhakoot_lifts_the_lords_enmity_by_vi33() {
    let rules = KootaRules::default();
    let maitri = |bride: Native, groom: Native| {
        let read = ashta_koota(bride, groom, rules);
        let good = matches!(
            read.row(Koota::Bhakoot).unwrap().reading,
            KootaReading::Bhakoot { dosha: None, .. }
        );
        match read.row(Koota::GrahaMaitri).unwrap().reading {
            KootaReading::GrahaMaitri {
                relation, lifted, ..
            } => (relation, good, lifted),
            _ => unreachable!(),
        }
    };
    // Every pair of padas: lifted exactly when an enmity meets a good
    // Bhakoot.
    for bride in 0..108_u16 {
        for groom in 0..108_u16 {
            let (b, g) = (
                at(Nakshatra::ALL[usize::from(bride / 4)], bride % 4 + 1),
                at(Nakshatra::ALL[usize::from(groom / 4)], groom % 4 + 1),
            );
            let (relation, good, lifted) = maitri(b, g);
            let enmity = matches!(
                relation,
                MaitriRelation::FriendEnemy
                    | MaitriRelation::NeutralEnemy
                    | MaitriRelation::MutualEnemies
            );
            assert_eq!(lifted, enmity && good);
        }
    }
}
