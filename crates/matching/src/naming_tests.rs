//! Nāmakaraṇa held to its sources: every cell read both ways, the signs'
//! syllables, the patro's glosses, the schemes, `CASUAL`, and the clauses
//! with the texts' own examples (`03-design/namakarana.md` §5).

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index their own tables"
)]

use std::collections::BTreeSet;

use teistro_core::catalogue::{Deity, Nakshatra, Rashi};
use teistro_intl::translit::{Scheme, devanagari_to_iast, to_devanagari};

use crate::{
    Birth, CASUAL, Clause, ClauseSource, FinalA, GrhyaSutra, InitialBy, LatinName, NaamRules,
    NameRules, NamingRules, NasalInitial, Native, PadaSyllable, SyllablesOf, birth_syllable,
    naam_milan, name_check, name_clauses, name_syllable, names_of, read_name, syllables,
};

fn under(latin: LatinName) -> NamingRules {
    NamingRules::from(NameRules {
        latin,
        ..NameRules::default()
    })
}

fn casual() -> NamingRules {
    under(LatinName::Casual)
}

/// Every cell of the cakra, the 27 stars' and Abhijit's, by sign.
fn every_cell() -> Vec<PadaSyllable> {
    let mut all: Vec<PadaSyllable> = Rashi::ALL
        .into_iter()
        .flat_map(|rashi| syllables(SyllablesOf::Rashi { rashi }, NamingRules::default()).unwrap())
        .collect();
    all.sort_by_key(|syllable| syllable.cell);
    all
}

fn cell_of(name: &str, rules: NamingRules) -> u8 {
    let read = read_name(name, rules).unwrap();
    assert!(!read.ambiguous, "{name}");
    read.readings[0].syllable.cell
}

/// IAST with its marks decomposed, as a keyboard without them types it.
fn decomposed(iast: &str) -> String {
    iast.chars()
        .map(|letter| match letter {
            'ā' => "a\u{304}".to_owned(),
            'ī' => "i\u{304}".to_owned(),
            'ū' => "u\u{304}".to_owned(),
            'ē' => "e\u{304}".to_owned(),
            'ō' => "o\u{304}".to_owned(),
            'ṅ' => "n\u{307}".to_owned(),
            'ñ' => "n\u{303}".to_owned(),
            'ṭ' => "t\u{323}".to_owned(),
            'ḍ' => "d\u{323}".to_owned(),
            'ṇ' => "n\u{323}".to_owned(),
            'ś' => "s\u{301}".to_owned(),
            'ṣ' => "s\u{323}".to_owned(),
            other => other.to_string(),
        })
        .collect()
}

/// IAST as ISO 15919 writes the letters a printed syllable uses.
fn iso(iast: &str) -> String {
    iast.replace('e', "ē").replace('o', "ō")
}

#[test]
fn every_cell_reads_back_to_itself_in_every_spelling() {
    let cells = every_cell();
    assert_eq!(cells.len(), 112);
    for (at, syllable) in cells.iter().enumerate() {
        assert_eq!(usize::from(syllable.cell), at);
        for (spelled, rules) in [
            (syllable.devanagari.clone(), NamingRules::default()),
            (syllable.iast.clone(), under(LatinName::Iast)),
            (decomposed(&syllable.iast), under(LatinName::Iast)),
            (iso(&syllable.iast), under(LatinName::Iso15919)),
            (decomposed(&iso(&syllable.iast)), under(LatinName::Iso15919)),
        ] {
            assert_eq!(cell_of(&spelled, rules), syllable.cell, "{spelled}");
        }
        // Every initial reaches it, but a substitute, which reads in its
        // own cell.
        for initial in &syllable.initials {
            let reached = cell_of(&initial.devanagari, NamingRules::default());
            assert_eq!(
                reached == syllable.cell,
                initial.by != InitialBy::Substitute,
                "{} in {}",
                initial.devanagari,
                syllable.devanagari
            );
            assert_eq!(
                cell_of(&initial.iast, under(LatinName::Iast)),
                reached,
                "{}",
                initial.iast
            );
        }
        assert_eq!(syllable.initials[0].by, InitialBy::Cell);
        assert_eq!(syllable.initials[0].devanagari, syllable.devanagari);
    }
}

#[test]
fn a_pada_stands_in_its_natives_sign_and_abhijit_in_makara() {
    for syllable in every_cell() {
        match (syllable.nakshatra, syllable.pada) {
            (Some(nakshatra), Some(pada)) => {
                let native = Native::of_pada(nakshatra, pada).unwrap();
                assert_eq!(syllable.rashi, native.rashi, "{nakshatra:?} {pada}");
                // The built birth syllable is a view of the cell.
                let born = birth_syllable(nakshatra, pada).unwrap();
                assert_eq!(
                    (born.cell, born.devanagari, born.iast, born.varga),
                    (
                        syllable.cell,
                        syllable.devanagari,
                        syllable.iast,
                        syllable.varga
                    )
                );
            }
            (None, None) => {
                assert_eq!(syllable.rashi, Rashi::Capricorn);
                assert!((76..80).contains(&syllable.cell));
            }
            other => assert_eq!(other.0.is_some(), other.1.is_some()),
        }
    }
}

#[test]
fn each_sign_names_its_nine_padas_and_makara_abhijits_four() {
    let mut seen = BTreeSet::new();
    for rashi in Rashi::ALL {
        let named = syllables(SyllablesOf::Rashi { rashi }, NamingRules::default()).unwrap();
        let padas = named.iter().filter(|s| s.pada.is_some()).count();
        let abhijit = named.iter().filter(|s| s.pada.is_none()).count();
        assert_eq!(padas, 9, "{rashi:?}");
        assert_eq!(abhijit, if rashi == Rashi::Capricorn { 4 } else { 0 });
        assert!(named.windows(2).all(|pair| pair[0].cell < pair[1].cell));
        for syllable in named {
            assert!(seen.insert(syllable.cell), "{rashi:?} {}", syllable.cell);
        }
    }
    assert_eq!(seen, (0..112).collect());
    // Makara's thirteen, as the patro heads it: bho ja ji, Abhijit's ju
    // je jo kha, khi khu khe kho, ga gi (vowel length aside).
    let makara: Vec<String> = syllables(
        SyllablesOf::Rashi {
            rashi: Rashi::Capricorn,
        },
        NamingRules::default(),
    )
    .unwrap()
    .into_iter()
    .map(|s| s.devanagari)
    .collect();
    assert_eq!(
        makara,
        [
            "भो", "जा", "जी", "जू", "जे", "जो", "खा", "खी", "खू", "खे", "खो", "गा", "गी"
        ]
    );
}

#[test]
fn a_star_names_its_four_and_a_pada_its_one() {
    for nakshatra in Nakshatra::ALL {
        let four = syllables(SyllablesOf::Nakshatra { nakshatra }, NamingRules::default()).unwrap();
        assert_eq!(four.len(), 4);
        for (at, syllable) in four.iter().enumerate() {
            assert_eq!(syllable.nakshatra, Some(nakshatra));
            let pada = u8::try_from(at + 1).unwrap();
            assert_eq!(syllable.pada, Some(pada));
            let one = syllables(
                SyllablesOf::Pada { nakshatra, pada },
                NamingRules::default(),
            )
            .unwrap();
            assert_eq!(one, std::slice::from_ref(syllable));
        }
    }
    for pada in [0, 5] {
        let refused = syllables(
            SyllablesOf::Pada {
                nakshatra: Nakshatra::Ashwini,
                pada,
            },
            NamingRules::default(),
        )
        .unwrap_err();
        assert_eq!(refused.field(), Some("pada"));
    }
}

#[test]
fn a_request_for_syllables_reads_strictly() {
    let asked: SyllablesOf =
        serde_json::from_str(r#"{"of": "PADA", "nakshatra": "ASHWINI", "pada": 2}"#).unwrap();
    assert_eq!(
        asked,
        SyllablesOf::Pada {
            nakshatra: Nakshatra::Ashwini,
            pada: 2
        }
    );
    let star: SyllablesOf =
        serde_json::from_str(r#"{"of": "NAKSHATRA", "nakshatra": "ROHINI"}"#).unwrap();
    assert_eq!(
        star,
        SyllablesOf::Nakshatra {
            nakshatra: Nakshatra::Rohini
        }
    );
    assert!(serde_json::from_str::<SyllablesOf>(r#"{"of": "SIGN", "rashi": "ARIES"}"#).is_err());
    let rules: NamingRules = serde_json::from_str(
        r#"{"name": {"latin": "ISO_15919"}, "nasal": "NONE", "finalA": "SILENT"}"#,
    )
    .unwrap();
    assert_eq!(rules.name.latin, LatinName::Iso15919);
    assert_eq!(
        (rules.nasal, rules.final_a),
        (NasalInitial::None, FinalA::Silent)
    );
    for key in ["IAST", "HARVARD_KYOTO", "ITRANS", "CASUAL", "REFUSE"] {
        let latin: LatinName = serde_json::from_str(&format!("\"{key}\"")).unwrap();
        assert_eq!(serde_json::to_string(&latin).unwrap(), format!("\"{key}\""));
    }
}

#[test]
fn the_patro_glosses_each_nasal_with_its_vargas_third() {
    // C770: Ardra 3 ङ=ग, Hasta 3 ण=ड, Uttara Bhadrapada 4 ञ=ज.
    for (nakshatra, pada, third, name) in [
        (Nakshatra::Ardra, 3, "ग", "गणेश"),
        (Nakshatra::Hasta, 3, "ड", "डमरु"),
        (Nakshatra::UttaraBhadrapada, 4, "ज", "जय"),
    ] {
        let of = SyllablesOf::Pada { nakshatra, pada };
        let listed = &syllables(of, NamingRules::default()).unwrap()[0];
        let substitutes: Vec<&str> = listed
            .initials
            .iter()
            .filter(|i| i.by == InitialBy::Substitute)
            .map(|i| i.devanagari.as_str())
            .collect();
        assert_eq!(substitutes, [third]);
        // Each substitute is its varga's third: two letters before the
        // nasal.
        let nasal = listed.devanagari.chars().next().unwrap();
        assert_eq!(
            u32::from(third.chars().next().unwrap()),
            u32::from(nasal) - 2
        );
        let bare = NamingRules {
            nasal: NasalInitial::None,
            ..NamingRules::default()
        };
        let unlisted = &syllables(of, bare).unwrap()[0];
        assert!(
            unlisted
                .initials
                .iter()
                .all(|i| i.by != InitialBy::Substitute)
        );
        // A name beginning with the substitute reads in its own cell, and
        // a check says it took the substitute only while one is listed.
        let birth = Birth { nakshatra, pada };
        let check = name_check(name, birth, NamingRules::default()).unwrap();
        assert!(!check.same_pada && check.by_substitute, "{name}");
        assert!(!name_check(name, birth, bare).unwrap().by_substitute);
    }
    // ग reads as Dhanishtha's or Shatabhisha's, never Ardra's.
    let ga = name_syllable("गायत्री", NameRules::default()).unwrap();
    let go = name_syllable("गोपाल", NameRules::default()).unwrap();
    assert_eq!(
        (ga.nakshatra, ga.quarter, go.nakshatra, go.quarter),
        (
            Some(Nakshatra::Dhanishtha),
            1,
            Some(Nakshatra::Shatabhisha),
            1
        )
    );
    // The pada's own letter is no substitute, and another star's ग is not
    // named by one.
    let own = Birth {
        nakshatra: Nakshatra::Ardra,
        pada: 3,
    };
    assert!(
        !name_check("ङ", own, NamingRules::default())
            .unwrap()
            .by_substitute
    );
    let elsewhere = Birth {
        nakshatra: Nakshatra::Ashwini,
        pada: 1,
    };
    assert!(
        !name_check("गणेश", elsewhere, NamingRules::default())
            .unwrap()
            .by_substitute
    );
}

#[test]
fn the_paired_letters_are_listed_where_their_pair_stands() {
    let paired: Vec<(u8, String)> = every_cell()
        .into_iter()
        .flat_map(|s| {
            s.initials
                .into_iter()
                .filter(|i| i.by == InitialBy::Paired)
                .map(move |i| (s.cell, i.devanagari))
        })
        .collect();
    let spelled: Vec<&str> = paired.iter().map(|(_, d)| d.as_str()).collect();
    assert_eq!(
        spelled,
        ["बा", "बी", "बू", "बे", "बो", "शा", "शी", "शू", "शे", "शो"]
    );
}

#[test]
fn dharmasindhus_examples_read_as_it_gives_them() {
    // "Some" name by the avakahada: Cūḍāmaṇi, Cedīśa, Coleśa, Lakṣmaṇa
    // for Ashvini's four padas (C773).
    for (pada, name) in [(1, "चूडामणि"), (2, "चेदीश"), (3, "चोलेश"), (4, "लक्ष्मण")]
    {
        let birth = Birth {
            nakshatra: Nakshatra::Ashwini,
            pada,
        };
        assert!(
            name_check(name, birth, NamingRules::default())
                .unwrap()
                .same_pada,
            "{name}"
        );
    }
    for (iast, pada) in [
        ("Cūḍāmaṇi", 1),
        ("Cedīśa", 2),
        ("Coleśa", 3),
        ("Lakṣmaṇa", 4),
    ] {
        let birth = Birth {
            nakshatra: Nakshatra::Ashwini,
            pada,
        };
        assert!(
            name_check(iast, birth, under(LatinName::Iast))
                .unwrap()
                .same_pada
        );
    }
    // Deva and Hari meet every clause a boy's name is held to, under
    // either sutra.
    for sutra in [GrhyaSutra::Paraskara, GrhyaSutra::Ashvalayana] {
        let rules = NamingRules {
            sutra,
            ..NamingRules::default()
        };
        for name in ["देवः", "हरिः"] {
            let clauses = name_clauses(name, rules);
            assert_eq!(clauses.syllables, 2);
            for row in clauses.rows {
                let girls = matches!(row.clause, Clause::OddForAGirl | Clause::LongAForAGirl);
                assert_eq!(row.holds, !girls, "{name} {:?}", row.clause);
            }
        }
    }
    // Rudra fails the visarga and keeps the parity.
    let rudra = name_clauses(
        "रुद्र",
        NamingRules {
            sutra: GrhyaSutra::Ashvalayana,
            ..NamingRules::default()
        },
    );
    let holds = |clause| {
        rudra
            .rows
            .iter()
            .find(|row| row.clause == clause)
            .unwrap()
            .holds
    };
    assert!(!holds(Clause::VisargaFinal));
    assert!(holds(Clause::EvenForABoy) && holds(Clause::TwoOrFourSyllables));
    // Āgniśarman: a Krittika birth named for its deity, Agni.
    for pada in 1..=4 {
        let names = names_of(
            Birth {
                nakshatra: Nakshatra::Krittika,
                pada,
            },
            NamingRules::default(),
        )
        .unwrap();
        assert_eq!(names.deity, Deity::Agni);
        assert_eq!(names.syllable.pada, Some(pada));
    }
    assert_eq!(
        names_of(
            Birth {
                nakshatra: Nakshatra::Krittika,
                pada: 9
            },
            NamingRules::default()
        )
        .unwrap_err()
        .field(),
        Some("pada")
    );
}

#[test]
fn a_name_reads_alike_in_every_scheme() {
    for (deva, spellings) in [
        (
            "कृष्ण",
            [
                "Kṛṣṇa",
                "Kr\u{323}s\u{323}n\u{323}a",
                "Kr\u{325}ṣṇa",
                "kRSNa",
                "kRRiShNa",
            ],
        ),
        ("शिव", ["Śiva", "S\u{301}iva", "Śiva", "ziva", "shiva"]),
        (
            "गणेश",
            [
                "Gaṇeśa",
                "Gan\u{323}es\u{301}a",
                "Gaṇēśa",
                "gaNeza",
                "gaNesha",
            ],
        ),
        (
            "ज्ञान",
            ["Jñāna", "Jn\u{303}a\u{304}na", "Jñāna", "jJAna", "j~nAna"],
        ),
        (
            "षष्ठी",
            [
                "Ṣaṣṭhī",
                "S\u{323}as\u{323}t\u{323}hi\u{304}",
                "Ṣaṣṭhī",
                "SaSThI",
                "ShaShThii",
            ],
        ),
        (
            "ऋषि",
            ["Ṛṣi", "R\u{323}s\u{323}i", "R\u{325}ṣi", "RSi", "RRiShi"],
        ),
        ("ओम", ["Oma", "Oma", "Ōma", "oma", "oma"]),
    ] {
        let expected = name_syllable(deva, NameRules::default()).unwrap();
        for (spelled, latin) in spellings.into_iter().zip([
            LatinName::Iast,
            LatinName::Iast,
            LatinName::Iso15919,
            LatinName::HarvardKyoto,
            LatinName::Itrans,
        ]) {
            let rules = NameRules {
                latin,
                ..NameRules::default()
            };
            assert_eq!(
                name_syllable(spelled, rules).unwrap(),
                expected,
                "{spelled} {latin:?}"
            );
        }
    }
}

#[test]
fn a_latin_name_is_refused_where_its_scheme_has_no_letter() {
    for (spelled, latin) in [
        ("Wasim", LatinName::Iast),
        ("Deva", LatinName::Iso15919),
        ("Rama", LatinName::Itrans),
        ("xena", LatinName::HarvardKyoto),
        ("Cyrus", LatinName::Casual),
        ("Rāma", LatinName::Casual),
    ] {
        let refused = read_name(spelled, under(latin)).unwrap_err();
        assert_eq!(refused.field(), Some("name"), "{spelled}");
    }
    let refused = read_name("Rama", NamingRules::default()).unwrap_err();
    assert!(refused.hint().unwrap().contains("CASUAL"));
    let refused = read_name("Wasim", under(LatinName::Iast)).unwrap_err();
    assert!(refused.hint().unwrap().contains("IAST"));
}

/// A `CASUAL` reading as the Devanagari it spells, its final consonant
/// open.
fn spelled(reading: &str) -> String {
    to_devanagari(reading, Scheme::Iast)
        .unwrap()
        .trim_end_matches('्')
        .to_owned()
}

#[test]
fn the_casual_table_is_the_designs_list() {
    // `03-design/namakarana.md` §2.5, row by row.
    let design: [(&[&str], &[&str]); 13] = [
        (&["ch", "chh"], &["च", "छ"]),
        (&["t", "th"], &["त", "थ", "ट", "ठ"]),
        (&["d", "dh"], &["द", "ध", "ड", "ढ"]),
        (&["n"], &["न", "ण"]),
        (&["sh", "shh"], &["श", "ष"]),
        (&["s"], &["स", "श"]),
        (&["kh"], &["ख", "ष"]),
        (&["gy", "gya"], &["ग्य", "ज्ञ"]),
        (&["ksh", "x"], &["क्ष"]),
        (&["v", "w", "b"], &["व", "ब"]),
        (&["z"], &["\u{95b}"]),
        (&["f"], &["\u{95e}"]),
        (&["q"], &["\u{958}"]),
    ];
    let readings = |latin: &str| -> &[&str] {
        CASUAL
            .iter()
            .find(|(spelled, _)| *spelled == latin)
            .unwrap()
            .1
    };
    for (latins, devanagari) in design {
        let read: BTreeSet<String> = latins
            .iter()
            .flat_map(|latin| readings(latin).iter().map(|r| spelled(r)))
            .collect();
        let listed: BTreeSet<String> = devanagari.iter().map(|d| (*d).to_owned()).collect();
        assert_eq!(read, listed, "{latins:?}");
    }
    // ri and ru: रि and ऋ as the design lists them, and ru's own रु.
    let read: BTreeSet<String> = ["ri", "ru"]
        .iter()
        .flat_map(|latin| readings(latin).iter().map(|r| spelled(r)))
        .collect();
    assert_eq!(read, ["रि", "रु", "ऋ"].map(String::from).into());
    // The vowels, as IAST.
    for (latin, iast) in [
        ("aa", "ā"),
        ("ee", "ī"),
        ("oo", "ū"),
        ("ii", "ī"),
        ("uu", "ū"),
        ("ai", "ai"),
        ("au", "au"),
    ] {
        assert_eq!(readings(latin), [iast]);
    }
    // No spelling is hidden behind an earlier one it begins with, and each
    // reads as IAST.
    for (at, (spelled_as, reads)) in CASUAL.iter().enumerate() {
        assert!(
            CASUAL[..at]
                .iter()
                .all(|(earlier, _)| !spelled_as.starts_with(earlier)),
            "{spelled_as}"
        );
        for reading in *reads {
            assert!(to_devanagari(reading, Scheme::Iast).is_ok(), "{reading}");
        }
    }
}

#[test]
fn casual_answers_every_cell_and_never_chooses() {
    // Krishna: कृ or क्रि, both ki.
    let krishna = read_name("Krishna", casual()).unwrap();
    assert!(!krishna.ambiguous);
    assert_eq!(
        krishna.readings[0].syllable.nakshatra,
        Some(Nakshatra::Mrigashira)
    );
    assert_eq!(krishna.readings[0].spelled, "क्रिश्न");
    // Shiva: शि (Shatabhisha 3) or षि (Hasta 2).
    let shiva = read_name("Shiva", casual()).unwrap();
    let stars: Vec<_> = shiva
        .readings
        .iter()
        .map(|r| (r.syllable.nakshatra, r.syllable.quarter, r.spelled.as_str()))
        .collect();
    assert_eq!(
        stars,
        [
            (Some(Nakshatra::Hasta), 2, "षिव"),
            (Some(Nakshatra::Shatabhisha), 3, "शिव")
        ]
    );
    assert!(shiva.ambiguous);
    // Gyan: ग्य (Dhanishtha 1) or ज्ञ (Uttarashadha 3).
    let gyan = read_name("Gyan", casual()).unwrap();
    let stars: Vec<_> = gyan
        .readings
        .iter()
        .map(|r| (r.syllable.nakshatra, r.syllable.quarter))
        .collect();
    assert_eq!(
        stars,
        [
            (Some(Nakshatra::UttaraAshadha), 3),
            (Some(Nakshatra::Dhanishtha), 1)
        ]
    );
    // Sita: स or श, one cell, one reading.
    let sita = read_name("Sita", casual()).unwrap();
    assert_eq!(sita.readings.len(), 1);
    assert_eq!(sita.readings[0].spelled, "सित");
    // Vijay: व and ब share a cell and not a varga, so both are answered.
    let vijay = read_name("Vijay", casual()).unwrap();
    assert!(vijay.ambiguous);
    assert_eq!(
        vijay.readings[0].syllable.cell,
        vijay.readings[1].syllable.cell
    );
    // A name's words keep their spaces, and a word's last consonant its a.
    let two = read_name("Ram Prasad", casual()).unwrap();
    assert_eq!(two.readings[0].spelled, "रम प्रसद");
    assert_eq!(two.readings[0].clauses.syllables, 5);
    // A name of consonants alone is one akshara, read with its a.
    let shr = read_name("Shr", casual()).unwrap();
    assert_eq!(
        shr.readings
            .iter()
            .map(|r| r.spelled.as_str())
            .collect::<Vec<_>>(),
        ["ष्र", "श्र"]
    );
    // A hyphen ends the first akshara.
    let hyphen = read_name("Sh-ri", casual()).unwrap();
    assert_eq!(hyphen.readings.len(), 2);
    // Aishwarya begins with ai, which C293 refuses.
    assert_eq!(
        read_name("Aishwarya", casual()).unwrap_err().field(),
        Some("name")
    );
}

#[test]
fn a_match_or_a_check_refuses_a_name_read_two_ways() {
    let rules = NaamRules {
        name: NameRules {
            latin: LatinName::Casual,
            ..NameRules::default()
        },
        ..NaamRules::default()
    };
    let refused = naam_milan("Sita", "Shiva", rules).unwrap_err();
    assert_eq!(refused.field(), Some("groom.name"));
    assert!(refused.hint().unwrap().contains("षिव, शिव"));
    let answered = naam_milan("Sita", "Krishna", rules).unwrap();
    assert_eq!(
        answered,
        naam_milan("सीता", "कृष्ण", NaamRules::default()).unwrap()
    );
    let birth = Birth {
        nakshatra: Nakshatra::Hasta,
        pada: 2,
    };
    assert_eq!(
        name_check("Shiva", birth, casual()).unwrap_err().field(),
        Some("name")
    );
    assert_eq!(
        name_syllable("Gyan", casual().name).unwrap_err().field(),
        Some("name")
    );
    // The default still refuses a Latin name, as it always has.
    assert_eq!(
        naam_milan("Sita", "Krishna", NaamRules::default())
            .unwrap_err()
            .field(),
        Some("bride.name")
    );
}

#[test]
fn syllables_are_counted_by_their_vowels() {
    let silent = NamingRules {
        final_a: FinalA::Silent,
        ..NamingRules::default()
    };
    for (name, sounded, spoken) in [
        ("राम", 2, 1),
        // A conjunct counts once: la-kṣma-ṇa.
        ("लक्ष्मण", 3, 2),
        ("क्षमा", 2, 2),
        // A final virama counts nothing.
        ("राम्", 1, 1),
        // A word's only vowel stays.
        ("क", 1, 1),
        ("श्री", 1, 1),
        // Each word drops its own.
        ("राम प्रसाद", 5, 3),
        ("अरुण", 3, 2),
        ("ॐ", 1, 1),
        ("रामः", 2, 2),
        ("", 0, 0),
    ] {
        assert_eq!(
            name_clauses(name, NamingRules::default()).syllables,
            sounded,
            "{name}"
        );
        let counted = name_clauses(name, silent);
        assert_eq!(
            (counted.syllables, counted.counted),
            (spoken, FinalA::Silent),
            "{name}"
        );
    }
}

#[test]
fn each_clause_is_a_fact_about_the_letters() {
    let rules = NamingRules::default();
    let silent = NamingRules {
        final_a: FinalA::Silent,
        ..rules
    };
    let holds = |name: &str, rules: NamingRules, clause: Clause| {
        name_clauses(name, rules)
            .rows
            .iter()
            .find(|row| row.clause == clause)
            .unwrap()
            .holds
    };
    for (name, holds_it) in [
        ("गणेश", true),
        ("बलराम", true),
        ("हरि", true),
        ("राम", false),
        ("अरुण", false),
        ("सीता", false),
    ] {
        assert_eq!(
            holds(name, rules, Clause::SonantInitial),
            holds_it,
            "{name}"
        );
    }
    for (name, holds_it) in [("देव", true), ("अरुण", true), ("राम", false)] {
        assert_eq!(
            holds(name, rules, Clause::SemivowelWithin),
            holds_it,
            "{name}"
        );
    }
    assert!(holds("रामः", rules, Clause::VisargaFinal));
    assert!(!holds("राम", rules, Clause::VisargaFinal));
    assert!(holds("सुदर्शन", rules, Clause::TwoOrFourSyllables));
    assert!(!holds("कमल", rules, Clause::TwoOrFourSyllables));
    assert!(holds("सीता", rules, Clause::LongAForAGirl));
    assert!(holds("आ", rules, Clause::LongAForAGirl));
    assert!(!holds("राम", rules, Clause::LongAForAGirl));
    assert!(holds("कमल", rules, Clause::OddForAGirl));
    // The final l or r: under a virama always, bare only when speech
    // drops the a, and never a word's only vowel.
    assert!(!holds("कमल्", rules, Clause::NoFinalLOrR));
    assert!(holds("कमल", rules, Clause::NoFinalLOrR));
    assert!(!holds("कमल", silent, Clause::NoFinalLOrR));
    assert!(!holds("कुमार", silent, Clause::NoFinalLOrR));
    assert!(holds("ल", silent, Clause::NoFinalLOrR));
    assert!(holds("हरिः", silent, Clause::NoFinalLOrR));
    assert!(holds("राम", silent, Clause::NoFinalLOrR));
}

#[test]
fn each_sutra_reports_its_own_clauses_and_sources() {
    let rows = |sutra| {
        name_clauses(
            "देवः",
            NamingRules {
                sutra,
                ..NamingRules::default()
            },
        )
        .rows
        .into_iter()
        .map(|row| (row.clause, row.source))
        .collect::<Vec<_>>()
    };
    let [p, a, d] = [
        ClauseSource::Paraskara,
        ClauseSource::Ashvalayana,
        ClauseSource::ApastambaHiranyakeshin,
    ];
    assert_eq!(
        rows(GrhyaSutra::Paraskara),
        [
            (Clause::SonantInitial, p),
            (Clause::SemivowelWithin, p),
            (Clause::VisargaFinal, p),
            (Clause::TwoOrFourSyllables, p),
            (Clause::OddForAGirl, p),
            (Clause::LongAForAGirl, p),
            (Clause::NoFinalLOrR, d),
        ]
    );
    assert_eq!(
        rows(GrhyaSutra::Ashvalayana),
        [
            (Clause::SonantInitial, a),
            (Clause::SemivowelWithin, a),
            (Clause::VisargaFinal, a),
            (Clause::TwoOrFourSyllables, a),
            (Clause::EvenForABoy, a),
            (Clause::OddForAGirl, a),
            (Clause::NoFinalLOrR, d),
        ]
    );
}

#[test]
fn a_check_carries_the_names_clauses() {
    let birth = Birth {
        nakshatra: Nakshatra::Chitra,
        pada: 3,
    };
    let check = name_check("राम", birth, NamingRules::default()).unwrap();
    assert!(check.same_pada && !check.by_substitute);
    assert_eq!(check.clauses, name_clauses("राम", NamingRules::default()));
    assert_eq!(check.prescribed.iast, devanagari_to_iast("रा"));
    // A Latin name's clauses are its Devanagari spelling's.
    let iast = name_check("rāmaḥ", birth, under(LatinName::Iast)).unwrap();
    assert_eq!(iast.clauses, name_clauses("रामः", NamingRules::default()));
}

#[test]
fn a_nukta_letter_reads_as_its_base_and_a_diphthong_after_a_consonant_is_refused() {
    for (nukta, base) in [
        ('\u{958}', 'क'),
        ('\u{959}', 'ख'),
        ('\u{95a}', 'ग'),
        ('\u{95b}', 'ज'),
        ('\u{95c}', 'ड'),
        ('\u{95d}', 'ढ'),
        ('\u{95e}', 'फ'),
        ('\u{95f}', 'य'),
    ] {
        assert_eq!(
            name_syllable(&format!("{nukta}ा"), NameRules::default()).unwrap(),
            name_syllable(&format!("{base}ा"), NameRules::default()).unwrap(),
            "{nukta}"
        );
    }
    for name in ["कैलाश", "गौरी"] {
        let refused = name_syllable(name, NameRules::default()).unwrap_err();
        assert_eq!(refused.field(), Some("name"), "{name}");
    }
}

#[test]
fn a_charts_birth_is_its_avakahadas_moon() {
    let moon = crate::avakahada(Native::of_moon(45.0).unwrap()).unwrap();
    let birth = Birth::from(&moon);
    assert_eq!((birth.nakshatra, birth.pada), (Nakshatra::Rohini, 2));
    // The name the chart prints is the one the birth prescribes.
    let names = names_of(birth, NamingRules::default()).unwrap();
    assert_eq!(names.syllable.cell, moon.syllable.cell);
}
