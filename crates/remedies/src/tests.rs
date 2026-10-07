#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the answers they read"
)]

use teistro_core::catalogue::{Graha, Rashi};

use crate::{ClauseKind, Functional, FunctionalRules, Nature, Scheme, functional};

use Graha::{
    Jupiter as JU, Mars as MA, Mercury as ME, Moon as MO, Saturn as SA, Sun as SU, Venus as VE,
};
use Nature::{Benefic as B, Malefic as M, Neutral as N, Yogakaraka as YK};

fn nature(read: &Functional, graha: Graha) -> Nature {
    read.row(graha).expect("a graha that owns a sign").nature
}

/// Each lagna's seven natures under *Laghu Parashari*, typed from the rules
/// (vv. 6 to 11 and 20) lagna by lagna, with the houses each graha owns,
/// rather than computed: a second reading of the same verses.
/// A graha, the houses it owns and its nature, as one lagna's row reads.
type Row = (Graha, &'static [u8], Nature);

const PARASHARI: [(Rashi, [Row; 7]); 12] = [
    (
        Rashi::Aries,
        [
            (MA, &[1, 8], B),
            (SU, &[5], B),
            (JU, &[9, 12], B),
            (ME, &[3, 6], M),
            (SA, &[10, 11], M),
            (VE, &[2, 7], N),
            (MO, &[4], N),
        ],
    ),
    (
        Rashi::Taurus,
        [
            (VE, &[1, 6], B),
            (ME, &[2, 5], B),
            (SA, &[9, 10], YK),
            (MO, &[3], M),
            (JU, &[8, 11], M),
            (SU, &[4], N),
            (MA, &[7, 12], N),
        ],
    ),
    (
        Rashi::Gemini,
        [
            (ME, &[1, 4], B),
            (VE, &[5, 12], B),
            (SA, &[8, 9], B),
            (SU, &[3], M),
            (MA, &[6, 11], M),
            (JU, &[7, 10], N),
            (MO, &[2], N),
        ],
    ),
    (
        Rashi::Cancer,
        [
            (MO, &[1], B),
            (MA, &[5, 10], YK),
            (JU, &[6, 9], B),
            (ME, &[3, 12], M),
            (VE, &[4, 11], M),
            (SA, &[7, 8], M),
            (SU, &[2], N),
        ],
    ),
    (
        Rashi::Leo,
        [
            (SU, &[1], B),
            (MA, &[4, 9], YK),
            (JU, &[5, 8], B),
            (ME, &[2, 11], M),
            (VE, &[3, 10], M),
            (SA, &[6, 7], M),
            (MO, &[12], N),
        ],
    ),
    (
        Rashi::Virgo,
        [
            (ME, &[1, 10], B),
            (VE, &[2, 9], B),
            (SA, &[5, 6], B),
            (MA, &[3, 8], M),
            (MO, &[11], M),
            (JU, &[4, 7], N),
            (SU, &[12], N),
        ],
    ),
    (
        Rashi::Libra,
        [
            (VE, &[1, 8], B),
            (SA, &[4, 5], YK),
            (ME, &[9, 12], B),
            (JU, &[3, 6], M),
            (SU, &[11], M),
            (MA, &[2, 7], N),
            (MO, &[10], N),
        ],
    ),
    (
        Rashi::Scorpio,
        [
            (MA, &[1, 6], B),
            (JU, &[2, 5], B),
            (MO, &[9], B),
            (SA, &[3, 4], M),
            (ME, &[8, 11], M),
            (VE, &[7, 12], N),
            (SU, &[10], N),
        ],
    ),
    (
        Rashi::Sagittarius,
        [
            (JU, &[1, 4], B),
            (MA, &[5, 12], B),
            (SU, &[9], B),
            (SA, &[2, 3], M),
            (VE, &[6, 11], M),
            (ME, &[7, 10], N),
            (MO, &[8], N),
        ],
    ),
    (
        Rashi::Capricorn,
        [
            (SA, &[1, 2], B),
            (VE, &[5, 10], YK),
            (ME, &[6, 9], B),
            (JU, &[3, 12], M),
            (MA, &[4, 11], M),
            (MO, &[7], N),
            (SU, &[8], N),
        ],
    ),
    (
        Rashi::Aquarius,
        [
            (SA, &[1, 12], B),
            (VE, &[4, 9], YK),
            (ME, &[5, 8], B),
            (JU, &[2, 11], M),
            (MA, &[3, 10], M),
            (MO, &[6], M),
            (SU, &[7], N),
        ],
    ),
    (
        Rashi::Pisces,
        [
            (JU, &[1, 10], B),
            (MA, &[2, 9], B),
            (MO, &[5], B),
            (VE, &[3, 8], M),
            (SU, &[6], M),
            (SA, &[11, 12], M),
            (ME, &[4, 7], N),
        ],
    ),
];

#[test]
fn every_lagna_reads_as_laghu_parashari_gives_it() {
    for (lagna, expected) in PARASHARI {
        let read = functional(lagna, FunctionalRules::default());
        assert_eq!(read.rows.len(), 7, "{lagna:?}");
        for (graha, houses, want) in expected {
            let row = read.row(graha).unwrap();
            assert_eq!(row.houses, houses, "{lagna:?} {graha:?}");
            assert_eq!(row.nature, want, "{lagna:?} {graha:?}");
        }
    }
}

#[test]
fn the_six_yogakarakas_and_no_other() {
    let found: Vec<(Rashi, Vec<Graha>)> = Rashi::ALL
        .into_iter()
        .map(|lagna| {
            (
                lagna,
                functional(lagna, FunctionalRules::default()).yogakarakas,
            )
        })
        .filter(|(_, found)| !found.is_empty())
        .collect();
    assert_eq!(
        found,
        [
            (Rashi::Taurus, vec![SA]),
            (Rashi::Cancer, vec![MA]),
            (Rashi::Leo, vec![MA]),
            (Rashi::Libra, vec![SA]),
            (Rashi::Capricorn, vec![VE]),
            (Rashi::Aquarius, vec![VE]),
        ]
    );
}

#[test]
fn the_lagna_lord_is_never_malefic_and_never_the_yogakaraka() {
    // BPHS ch. 13 v. 12 voids the lagnesha's own 6th or 8th: Mars for Aries
    // and Scorpio, Venus for Taurus and Libra.
    for lagna in Rashi::ALL {
        let read = functional(lagna, FunctionalRules::default());
        let lord = lagna.attributes().lord;
        assert_eq!(nature(&read, lord), B, "{lagna:?}");
    }
    for (lagna, lord) in [(Rashi::Aries, MA), (Rashi::Libra, VE)] {
        let row = functional(lagna, FunctionalRules::default())
            .row(lord)
            .cloned()
            .unwrap();
        assert!(row.holds(ClauseKind::RandhreshaVoidLagnesha), "{lagna:?}");
        assert!(!row.holds(ClauseKind::Randhresha), "{lagna:?}");
    }
}

#[test]
fn the_luminaries_carry_no_eighth_lord_blemish() {
    // LP v. 11: the Sun for Capricorn, the Moon for Sagittarius.
    for (lagna, graha) in [(Rashi::Capricorn, SU), (Rashi::Sagittarius, MO)] {
        let read = functional(lagna, FunctionalRules::default());
        let row = read.row(graha).unwrap();
        assert!(row.holds(ClauseKind::RandhreshaVoidLuminary), "{lagna:?}");
        assert_eq!(row.nature, N, "{lagna:?}");
        // The rival reading: the baseline makes either malefic.
        assert_eq!(
            nature(&functional(lagna, FunctionalRules::baseline()), graha),
            M
        );
    }
}

#[test]
fn the_lords_of_three_and_eleven_are_evil_and_the_twelfth_is_not() {
    // LP v. 6 against the baseline's 6, 8 and 12: Saturn for Aries owns the
    // 11th, the Sun for Libra the 11th; the Moon for Leo owns only the 12th.
    for (lagna, graha, parashari, baseline) in [
        (Rashi::Aries, SA, M, N),
        (Rashi::Libra, SU, M, N),
        (Rashi::Gemini, SU, M, N),
        (Rashi::Leo, MO, N, M),
        (Rashi::Virgo, SU, N, M),
    ] {
        assert_eq!(
            nature(&functional(lagna, FunctionalRules::default()), graha),
            parashari,
            "{lagna:?} {graha:?}"
        );
        assert_eq!(
            nature(&functional(lagna, FunctionalRules::baseline()), graha),
            baseline,
            "{lagna:?} {graha:?}"
        );
    }
}

#[test]
fn a_kendra_lord_loses_its_natural_nature() {
    // LP v. 7, the baseline's own four kendradhipati cases.
    for (lagna, graha) in [
        (Rashi::Gemini, JU),
        (Rashi::Virgo, JU),
        (Rashi::Sagittarius, ME),
        (Rashi::Pisces, ME),
    ] {
        let read = functional(lagna, FunctionalRules::default());
        let row = read.row(graha).unwrap();
        assert!(
            row.holds(ClauseKind::KendraBeneficLoses),
            "{lagna:?} {graha:?}"
        );
        assert_eq!(row.nature, N, "{lagna:?} {graha:?}");
    }
    // And a natural malefic owning only a kendra is not evil: the Sun for
    // Taurus owns the 4th.
    let taurus = functional(Rashi::Taurus, FunctionalRules::default());
    assert!(
        taurus
            .row(SU)
            .unwrap()
            .holds(ClauseKind::KendraMaleficLoses)
    );
    assert_eq!(nature(&taurus, SU), N);
}

#[test]
fn the_badhaka_is_by_the_lagnas_modality() {
    // The baseline's own table: the house and its lord, lagna by lagna.
    let expected = [
        (Rashi::Aries, 11, SA),
        (Rashi::Taurus, 9, SA),
        (Rashi::Gemini, 7, JU),
        (Rashi::Cancer, 11, VE),
        (Rashi::Leo, 9, MA),
        (Rashi::Virgo, 7, JU),
        (Rashi::Libra, 11, SU),
        (Rashi::Scorpio, 9, MO),
        (Rashi::Sagittarius, 7, ME),
        (Rashi::Capricorn, 11, MA),
        (Rashi::Aquarius, 9, VE),
        (Rashi::Pisces, 7, ME),
    ];
    for (lagna, house, lord) in expected {
        let read = functional(lagna, FunctionalRules::default());
        assert_eq!(
            (read.badhaka.house, read.badhaka.lord),
            (house, lord),
            "{lagna:?}"
        );
        assert!(
            read.row(lord).unwrap().holds(ClauseKind::Badhakesha),
            "{lagna:?}"
        );
        assert_ne!(lord, lagna.attributes().lord, "{lagna:?}");
    }
}

#[test]
fn the_marakas_are_the_lords_of_two_and_seven() {
    // LP v. 23. Aries: Venus owns both; Cancer: the Sun the 2nd, Saturn the 7th.
    assert_eq!(
        functional(Rashi::Aries, FunctionalRules::default()).marakas,
        [VE]
    );
    assert_eq!(
        functional(Rashi::Cancer, FunctionalRules::default()).marakas,
        [SU, SA]
    );
}

#[test]
fn the_baseline_scheme_reproduces_the_baseline_engines_sets() {
    // Its malefics, lagna by lagna: a lord of 6, 8 or 12 that owns no
    // trikona. Its yogakarakas are the six; nothing else differs in kind.
    let malefics: [(Rashi, &[Graha]); 12] = [
        (Rashi::Aries, &[ME]),
        (Rashi::Taurus, &[MA, JU]),
        (Rashi::Gemini, &[MA]),
        (Rashi::Cancer, &[ME, SA]),
        (Rashi::Leo, &[MO, SA]),
        (Rashi::Virgo, &[SU, MA]),
        (Rashi::Libra, &[JU]),
        (Rashi::Scorpio, &[ME, VE]),
        (Rashi::Sagittarius, &[MO, VE]),
        (Rashi::Capricorn, &[SU, JU]),
        (Rashi::Aquarius, &[MO]),
        (Rashi::Pisces, &[SU, VE, SA]),
    ];
    for (lagna, expected) in malefics {
        let read = functional(lagna, FunctionalRules::baseline());
        assert_eq!(read.scheme, Scheme::Baseline);
        let found: Vec<Graha> = read
            .rows
            .iter()
            .filter(|row| row.nature == M)
            .map(|row| row.graha)
            .collect();
        assert_eq!(found, expected, "{lagna:?}");
        assert_eq!(
            read.yogakarakas,
            functional(lagna, FunctionalRules::default()).yogakarakas,
            "{lagna:?}"
        );
    }
}

#[test]
fn every_clause_names_its_verse_and_a_key_is_what_serde_writes() {
    for (kind, key) in [
        (ClauseKind::TrikonaLord, "TRIKONA_LORD"),
        (
            ClauseKind::RandhreshaVoidLuminary,
            "RANDHRESHA_VOID_LUMINARY",
        ),
        (ClauseKind::KendraBeneficLoses, "KENDRA_BENEFIC_LOSES"),
        (ClauseKind::Badhakesha, "BADHAKESHA"),
    ] {
        assert_eq!(serde_json::to_value(kind).unwrap(), key);
        assert_ne!(kind.source(), "", "{kind:?}");
    }
    assert_eq!(
        serde_json::to_value(Nature::Yogakaraka).unwrap(),
        "YOGAKARAKA"
    );
    assert_eq!(
        serde_json::to_value(Scheme::LaghuParashari).unwrap(),
        "LAGHU_PARASHARI"
    );
    let rules: FunctionalRules = serde_json::from_str(r#"{"scheme": "BASELINE"}"#).unwrap();
    assert_eq!(rules, FunctionalRules::baseline());
    assert!(serde_json::from_str::<FunctionalRules>(r#"{"schema": "BASELINE"}"#).is_err());
    let read = serde_json::to_value(functional(Rashi::Taurus, FunctionalRules::default())).unwrap();
    assert_eq!(read["lagna"], "TAURUS");
    assert_eq!(
        read["rows"][6]["clauses"][0],
        serde_json::json!({"kind": "TRIKONA_LORD", "house": 9})
    );
    assert_eq!(
        read["badhaka"],
        serde_json::json!({"house": 9, "lord": "SATURN"})
    );
}
