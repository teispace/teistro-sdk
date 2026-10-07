//! The graha-śānti table held to the verses, each reading the earlier
//! translations got wrong pinned beside the verse word.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the answers they read"
)]

use teistro_core::catalogue::{Direction, Graha};

use crate::{
    Dakshina, Gem, ImageMaterial, MandalaPlace, OFFERINGS, RikSource, ShantiRules, Substance,
    shanti,
};

const NINE: [Graha; 9] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
    Graha::Rahu,
    Graha::Ketu,
];

/// The maṇḍala's place toward a direction.
fn place(direction: Direction) -> MandalaPlace {
    match direction {
        Direction::East => MandalaPlace::East,
        Direction::Southeast => MandalaPlace::Southeast,
        Direction::South => MandalaPlace::South,
        Direction::Southwest => MandalaPlace::Southwest,
        Direction::West => MandalaPlace::West,
        Direction::Northwest => MandalaPlace::Northwest,
        Direction::North => MandalaPlace::North,
        _ => MandalaPlace::Northeast,
    }
}

fn read(graha: Graha) -> crate::Shanti {
    shanti(graha, ShantiRules::default()).unwrap()
}

#[test]
fn the_japa_counts_are_the_verse_words_read_right_to_left() {
    // BPHS 84.19: sapta, rudrāḥ, diśaḥ, nandāḥ, nava-candrāḥ, nṛpāḥ,
    // tri-pakṣāḥ, aṣṭa-candrāḥ, sapta-candrāḥ.
    let counts: Vec<u8> = NINE
        .iter()
        .map(|&graha| read(graha).japa_thousands)
        .collect();
    assert_eq!(counts, [7, 11, 10, 9, 19, 16, 23, 18, 17]);
    // The English print's 11 000 for Mars misreads diśaḥ, the ten
    // directions; the print's own Hindi gloss says 10 (C344).
    assert_ne!(read(Graha::Mars).japa_thousands, 11);
}

#[test]
fn the_rite_is_the_verses_in_graha_order() {
    let images: Vec<ImageMaterial> = NINE.iter().map(|&graha| read(graha).image).collect();
    assert_eq!(
        images,
        [
            ImageMaterial::Copper,
            ImageMaterial::Crystal,
            ImageMaterial::RedSandalwood,
            ImageMaterial::Gold,
            ImageMaterial::Gold,
            ImageMaterial::Silver,
            ImageMaterial::Iron,
            ImageMaterial::Lead,
            ImageMaterial::Bronze,
        ]
    );
    // Ketu's fee is chāga, a goat, in both texts; "a sheep" is the 1918
    // English (C344).
    assert_eq!(read(Graha::Ketu).dakshina, Dakshina::Goat);
    assert_eq!(read(Graha::Rahu).dakshina, Dakshina::Iron);
    assert_eq!(OFFERINGS, [108, 28]);
}

#[test]
fn only_rahus_rik_depends_on_the_text() {
    let yajnavalkya = ShantiRules {
        rik: RikSource::Yajnavalkya,
    };
    for graha in NINE {
        let bphs = read(graha);
        let other = shanti(graha, yajnavalkya).unwrap();
        assert_eq!(bphs.rik == other.rik, graha != Graha::Rahu, "{graha:?}");
        assert_eq!(
            bphs,
            crate::Shanti {
                rik: bphs.rik,
                ..other
            }
        );
    }
    assert_eq!(read(Graha::Rahu).rik, "kayā naś citra");
}

#[test]
fn the_nodes_own_gems_and_no_substance() {
    assert_eq!(read(Graha::Rahu).gem, Gem::Hessonite);
    assert_eq!(read(Graha::Ketu).gem, Gem::CatsEye);
    for graha in NINE {
        let node = matches!(graha, Graha::Rahu | Graha::Ketu);
        assert_eq!(read(graha).substance.is_none(), node, "{graha:?}");
    }
    assert_eq!(read(Graha::Mercury).substance, Some(Substance::Alloy));
}

#[test]
fn brihat_jataka_gives_eight_directions_and_the_mandala_is_another_scheme() {
    let directions: Vec<Direction> = NINE
        .iter()
        .filter_map(|&graha| read(graha).direction)
        .collect();
    assert_eq!(directions.len(), 8);
    let mut distinct = directions.clone();
    distinct.sort();
    distinct.dedup();
    assert_eq!(distinct.len(), 8, "one graha to each direction");
    assert_eq!(read(Graha::Ketu).direction, None);
    assert_eq!(read(Graha::Ketu).mandala, MandalaPlace::Northwest);
    assert_eq!(read(Graha::Sun).mandala, MandalaPlace::Centre);
    // The maṇḍala agrees with Brihat Jataka for Mars, Saturn and Rahu
    // only, so it is a scheme of its own, not a reading of II.5.
    let agree: Vec<Graha> = NINE
        .into_iter()
        .filter(|&graha| {
            let row = read(graha);
            row.direction.map(place) == Some(row.mandala)
        })
        .collect();
    assert_eq!(agree, [Graha::Mars, Graha::Saturn, Graha::Rahu]);
}

#[test]
fn no_text_prescribes_for_the_outer_planets() {
    for graha in [Graha::Uranus, Graha::Neptune, Graha::Pluto] {
        assert_eq!(shanti(graha, ShantiRules::default()), None);
    }
}

#[test]
fn the_keys_are_what_serde_writes() {
    let sun = serde_json::to_value(read(Graha::Sun)).unwrap();
    assert_eq!(sun["japaThousands"], 7);
    assert_eq!(sun["dakshina"], "MILCH_COW");
    assert_eq!(sun["mandala"], "CENTRE");
    assert_eq!(sun["direction"], "EAST");
    assert_eq!(
        serde_json::to_value(read(Graha::Ketu)).unwrap()["direction"],
        serde_json::Value::Null
    );
    let rules: ShantiRules = serde_json::from_str(r#"{"rik": "YAJNAVALKYA"}"#).unwrap();
    assert_eq!(rules.rik, RikSource::Yajnavalkya);
    assert!(serde_json::from_str::<ShantiRules>(r#"{"rk": "BPHS"}"#).is_err());
}
