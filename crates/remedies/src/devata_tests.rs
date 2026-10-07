//! The ishṭa-devatā against BPHS (1923) ch. 9 vv. 70–76 and the baseline's
//! table.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the skies they build"
)]

use teistro_core::catalogue::{Graha, Rashi};

use crate::subjects::NINE;
use crate::{Deity, DevataRules, IshtaDevata, SunWithKetu, baseline_ishta_devata, ishta_devata};

/// The kārakāṁśa every test counts from: Leo, so the 12th is Cancer.
const KARAKAMSHA: Rashi = Rashi::Leo;

/// A sky that can happen with `there` in Cancer: the nodes opposite each
/// other, Rahu in Libra and Ketu in Aries unless one of them is placed,
/// and everything else in Aries.
fn sky(there: &[Graha]) -> [Rashi; 9] {
    let mut signs = [Rashi::Aries; 9];
    signs[7] = Rashi::Libra;
    for &graha in there {
        let at = NINE.iter().position(|&one| one == graha).unwrap();
        signs[at] = Rashi::Cancer;
    }
    for (node, other) in [(7, 8), (8, 7)] {
        if signs[node] == Rashi::Cancer {
            signs[other] = Rashi::Cancer.opposite();
        }
    }
    signs
}

fn reading(there: &[Graha]) -> IshtaDevata {
    ishta_devata(KARAKAMSHA, &sky(there), DevataRules::default())
}

#[test]
fn each_graha_alone_names_its_verse_and_deity() {
    for (graha, deities, verse) in [
        (Graha::Sun, vec![Deity::Shiva], 70),
        (Graha::Moon, vec![Deity::Gauri], 71),
        (Graha::Venus, vec![Deity::Lakshmi], 72),
        (Graha::Mars, vec![Deity::Skanda], 73),
        (Graha::Mercury, vec![Deity::Vishnu], 73),
        (Graha::Saturn, vec![Deity::Vishnu], 73),
        (Graha::Jupiter, vec![Deity::Shiva], 73),
        (Graha::Rahu, vec![Deity::Durga], 74),
        (Graha::Ketu, vec![Deity::Heramba, Deity::Skanda], 74),
    ] {
        let read = reading(&[graha]);
        assert_eq!(read.sign, Rashi::Cancer);
        assert_eq!(read.devotions.len(), 1, "{graha:?}");
        let one = &read.devotions[0];
        assert_eq!(
            (one.graha, &one.deities, one.verse),
            (graha, &deities, verse)
        );
        assert!(!one.with_ketu, "{graha:?} stands without Ketu");
    }
}

#[test]
fn the_prints_ravi_bhakti_is_a_reading() {
    let signs = sky(&[Graha::Sun, Graha::Ketu]);
    let rules = DevataRules {
        sun_with_ketu: SunWithKetu::Surya,
    };
    let read = ishta_devata(KARAKAMSHA, &signs, rules);
    assert_eq!(read.devotions[0].deities, [Deity::Surya]);
    assert_eq!(read.rules, rules);
    // The Moon's Gaurī stands under either reading (C354).
    let moon = ishta_devata(KARAKAMSHA, &sky(&[Graha::Moon, Graha::Ketu]), rules);
    assert_eq!(moon.devotions[0].deities, [Deity::Gauri]);
}

#[test]
fn ketu_is_reported_beside_each_graha_never_beside_the_nodes() {
    let read = reading(&[Graha::Moon, Graha::Jupiter, Graha::Ketu]);
    let with: Vec<(Graha, bool)> = read
        .devotions
        .iter()
        .map(|one| (one.graha, one.with_ketu))
        .collect();
    assert_eq!(
        with,
        [
            (Graha::Moon, true),
            (Graha::Jupiter, true),
            (Graha::Ketu, false)
        ]
    );
    // Rahu stands opposite Ketu, so a 12th holding Rahu never holds Ketu:
    // the series cannot be read as all "with Ketu" (C355).
    let rahu = reading(&[Graha::Rahu, Graha::Mars]);
    assert!(rahu.devotions.iter().all(|one| !one.with_ketu));
}

#[test]
fn saturn_or_venus_in_a_malefics_sign_serves_minor_deities() {
    // The kārakāṁśa in Aquarius: the 12th is Capricorn, Saturn's.
    let mut signs = sky(&[]);
    signs[6] = Rashi::Capricorn;
    signs[5] = Rashi::Capricorn;
    let read = ishta_devata(Rashi::Aquarius, &signs, DevataRules::default());
    assert_eq!(read.minor, [Graha::Saturn, Graha::Venus]);
    // In Taurus, Venus's own, neither does.
    let mut signs = sky(&[]);
    signs[6] = Rashi::Taurus;
    signs[5] = Rashi::Taurus;
    let read = ishta_devata(Rashi::Gemini, &signs, DevataRules::default());
    assert_eq!(read.minor, []);
    assert_eq!(read.devotions.len(), 2);
    // Leo is the Sun's: a malefic's sign.
    let mut signs = sky(&[]);
    signs[5] = Rashi::Leo;
    assert_eq!(
        ishta_devata(Rashi::Virgo, &signs, DevataRules::default()).minor,
        [Graha::Venus]
    );
}

#[test]
fn an_empty_twelfth_names_no_deity() {
    let read = reading(&[]);
    assert_eq!(read.devotions, []);
    assert_eq!(read.minor, []);
}

#[test]
fn the_baseline_keys_twelve_signs_by_the_moon() {
    assert_eq!(
        baseline_ishta_devata(Rashi::Aries),
        [Deity::Surya, Deity::Skanda]
    );
    assert_eq!(
        baseline_ishta_devata(Rashi::Gemini),
        baseline_ishta_devata(Rashi::Virgo)
    );
    assert_eq!(
        baseline_ishta_devata(Rashi::Aquarius),
        [Deity::Shani, Deity::Shiva]
    );
    assert_eq!(
        baseline_ishta_devata(Rashi::Pisces),
        [Deity::Vishnu, Deity::Brihaspati]
    );
}

#[test]
fn keys_are_what_serde_writes_and_rules_read_strictly() {
    assert_eq!(
        serde_json::to_string(&Deity::Heramba).unwrap(),
        "\"HERAMBA\""
    );
    let rules: DevataRules = serde_json::from_str(r#"{"sunWithKetu":"SURYA"}"#).unwrap();
    assert_eq!(rules.sun_with_ketu, SunWithKetu::Surya);
    assert!(serde_json::from_str::<DevataRules>(r#"{"sunWithKetus":"SURYA"}"#).is_err());
}
