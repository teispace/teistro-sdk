//! A name's syllable held to the printed table, and naam milan.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index their own tables"
)]

use teistro_core::catalogue::Nakshatra;

use crate::{
    AbhijitPada, KootaRules, LatinName, NaamRules, NameRules, NameSyllable, NameVarga, Native,
    VargaRelation, ashta_koota, naam_milan, name_syllable, varga_koota,
};

/// *Muhurta Chintamani*'s śatapada table as printed (1954, p. 173, leaf
/// n185, read on the image), from Ashvini, Abhijit after Uttarashadha,
/// with the print's vowel lengths.
const PRINTED: [[&str; 4]; 28] = [
    ["चू", "चे", "चो", "ला"],
    ["ली", "लू", "ले", "लो"],
    ["आ", "ई", "उ", "ए"],
    ["ओ", "वा", "वी", "वू"],
    ["वे", "वो", "का", "की"],
    ["कू", "घ", "ङ", "छा"],
    ["के", "को", "हा", "ही"],
    ["हू", "हे", "हो", "डा"],
    ["डी", "डू", "डे", "डो"],
    ["मा", "मी", "मू", "मे"],
    ["मो", "टा", "टी", "टू"],
    ["टे", "टो", "पा", "पी"],
    ["पू", "ष", "णा", "ठा"],
    ["पे", "पो", "रा", "री"],
    ["रू", "रे", "रो", "ता"],
    ["ती", "तू", "ते", "तो"],
    ["ना", "नी", "नू", "ने"],
    ["नो", "या", "यी", "यू"],
    ["ये", "यो", "भा", "भी"],
    ["भू", "धा", "फा", "ढा"],
    ["भे", "भो", "जा", "जी"],
    ["जू", "जे", "जो", "खा"],
    ["खी", "खू", "खे", "खो"],
    ["गा", "गी", "गू", "गे"],
    ["गो", "सा", "सी", "सू"],
    ["से", "सो", "दा", "दी"],
    ["दू", "थ", "झ", "ञा"],
    ["दे", "दो", "चा", "ची"],
];

/// The printed row's star, `None` for Abhijit's.
fn star_of(row: usize) -> Option<Nakshatra> {
    match row {
        21 => None,
        _ => Some(Nakshatra::ALL[row - usize::from(row > 21)]),
    }
}

fn read(name: &str) -> NameSyllable {
    name_syllable(name, NameRules::default()).unwrap()
}

#[test]
fn the_rule_gives_every_printed_syllable_its_own_cell() {
    let mut cells = Vec::new();
    for (row, syllables) in PRINTED.iter().enumerate() {
        for (at, syllable) in syllables.iter().enumerate() {
            let one = read(syllable);
            assert_eq!(
                (one.nakshatra, usize::from(one.quarter)),
                (star_of(row), at + 1),
                "{syllable}"
            );
            cells.push(one.cell);
        }
    }
    cells.sort_unstable();
    assert_eq!(cells, (0..112).collect::<Vec<u8>>());
}

#[test]
fn a_conjunct_reads_its_first_consonant_with_the_clusters_vowel() {
    let cases = [
        // pi, Uttara Phalguni's 4th; kṛ as ki, Mrigashira's 4th (C293).
        ("प्रिया", Some(Nakshatra::UttaraPhalguni), 4, NameVarga::Rat),
        ("कृष्ण", Some(Nakshatra::Mrigashira), 4, NameVarga::Cat),
        ("क्षमा", Some(Nakshatra::Mrigashira), 3, NameVarga::Cat),
        // ś read as s, b as v (C294); the varga is the letter written.
        ("श्याम", Some(Nakshatra::Shatabhisha), 2, NameVarga::Sheep),
        ("बलराम", Some(Nakshatra::Rohini), 2, NameVarga::Rat),
        // The inherent a past an anusvara, a nukta read as its base.
        ("संजय", Some(Nakshatra::Shatabhisha), 2, NameVarga::Sheep),
        ("ज़ोया", None, 3, NameVarga::Lion),
    ];
    for (name, star, quarter, varga) in cases {
        let one = read(name);
        assert_eq!((one.nakshatra, one.quarter), (star, quarter), "{name}");
        assert_eq!(one.varga, varga, "{name}");
    }
}

#[test]
fn a_latin_name_is_read_only_as_declared_iast() {
    let refused = name_syllable("Chandra", NameRules::default()).unwrap_err();
    assert_eq!(refused.field(), Some("name"));
    let iast = NameRules {
        latin: LatinName::Iast,
        ..NameRules::default()
    };
    let as_iast = |name| name_syllable(name, iast).unwrap();
    assert_eq!(as_iast("Kṛṣṇa"), read("कृष्ण"));
    assert_eq!(as_iast("candra"), read("चन्द्र"));
    // English "ch" is च, IAST's is छ: Revati against Ardra.
    assert_eq!(as_iast("chandra").nakshatra, Some(Nakshatra::Ardra));
    assert_eq!(as_iast("Śyāma"), read("श्याम"));
    assert!(name_syllable("Wasim", iast).is_err());
}

#[test]
fn a_letter_the_cakra_does_not_read_is_refused_by_name() {
    for name in ["ऐश्वर्या", "औषधि", "ऌ", "ळ", "", "  ", "1राम"] {
        let refused = name_syllable(name, NameRules::default()).unwrap_err();
        assert_eq!(refused.field(), Some("name"), "{name}");
    }
    let iast = NameRules {
        latin: LatinName::Iast,
        ..NameRules::default()
    };
    assert!(name_syllable("aiśvarya", iast).is_err());
}

#[test]
fn abhijit_stands_where_the_knob_puts_it() {
    let jo = read("जोशी");
    assert_eq!((jo.nakshatra, jo.quarter), (None, 3));
    assert_eq!(
        jo.native(AbhijitPada::Refuse).unwrap_err().field(),
        Some("abhijit")
    );
    let (fourth, first) = (
        jo.native(AbhijitPada::UttaraAshadha).unwrap(),
        jo.native(AbhijitPada::Shravana).unwrap(),
    );
    assert_eq!(
        (fourth.nakshatra, fourth.pada),
        (Nakshatra::UttaraAshadha, 4)
    );
    assert_eq!((first.nakshatra, first.pada), (Nakshatra::Shravana, 1));
}

#[test]
fn the_fifth_varga_is_the_enemy_both_ways() {
    let vargas = [
        NameVarga::Garuda,
        NameVarga::Cat,
        NameVarga::Lion,
        NameVarga::Dog,
        NameVarga::Serpent,
        NameVarga::Rat,
        NameVarga::Deer,
        NameVarga::Sheep,
    ];
    let mut counts = [0; 3];
    for bride in vargas {
        for groom in vargas {
            let koota = varga_koota(bride, groom);
            assert_eq!(koota.relation, varga_koota(groom, bride).relation);
            let at = match koota.relation {
                VargaRelation::Same => 0,
                VargaRelation::Enemy => 1,
                VargaRelation::Neutral => 2,
            };
            counts[at] += 1;
        }
    }
    assert_eq!(counts, [8, 8, 48]);
    assert_eq!(
        varga_koota(read("अमर").varga, read("तारा").varga).relation,
        VargaRelation::Enemy
    );
}

#[test]
fn two_names_are_matched_star_to_star() {
    let rules = NaamRules::default();
    let read = naam_milan("सीता", "राम", rules).unwrap();
    let (sita, ram) = (
        read.bride.native(AbhijitPada::Refuse).unwrap(),
        read.groom.native(AbhijitPada::Refuse).unwrap(),
    );
    assert_eq!(read.ashta, ashta_koota(sita, ram, KootaRules::default()));
    assert_eq!(read.varga.relation, VargaRelation::Neutral);
    let refused = naam_milan("राम", "जोशी", rules).unwrap_err();
    assert_eq!(refused.field(), Some("groom.abhijit"));
    let refused = naam_milan("ऐश्वर्या", "राम", rules).unwrap_err();
    assert_eq!(refused.field(), Some("bride.name"));
}

#[test]
fn a_pada_founds_its_native() {
    for nakshatra in Nakshatra::ALL {
        for pada in 1..=4 {
            let one = Native::of_pada(nakshatra, pada).unwrap();
            assert_eq!((one.nakshatra, one.pada), (nakshatra, pada));
        }
    }
    for pada in [0, 5] {
        let refused = Native::of_pada(Nakshatra::Ashwini, pada).unwrap_err();
        assert_eq!(refused.field(), Some("pada"));
    }
}
