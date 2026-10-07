//! Each rule held to its verse, on skies whose clauses are worked out by
//! hand in the comments.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the skies they build"
)]

use teistro_core::catalogue::{Graha, Rashi};
use teistro_tajika::{KshinaRule, MoonClause, MoonRules};

use crate::{
    Answer, Change, ClauseKind, FactorKind, Favour, MookRule, Outcome, Person, PiscesRising,
    Placed, PrashnaRules, PrashnaSky, Question, Score, ScoreRule, Thought, TimingRule, Unit,
    number_sign, read,
};

/// A sky with the lagna at `lagna_deg` and the nine grahas, Sun to Ketu,
/// at `at`; every navāṁśa Aries unless a test sets it, and strengths
/// rising from the Sun so none ties.
fn sky(lagna_deg: f64, lagna_navamsha: Rashi, at: [f64; 9]) -> PrashnaSky {
    PrashnaSky {
        lagna_deg,
        lagna_navamsha,
        grahas: at.map(|longitude_deg| Placed {
            longitude_deg,
            navamsha: Rashi::Aries,
            retrograde: false,
            combust: false,
        }),
        strength: [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0],
    }
}

/// Gemini rising, Jupiter, Venus and a waxing Moon in it, the rising
/// navāṁśa Sagittarius (Jupiter's); the Sun, Mars, Saturn and Mercury
/// in Taurus, none of which aspects Gemini: every clause is for.
fn gemini() -> PrashnaSky {
    sky(
        75.0,
        Rashi::Sagittarius,
        [40.0, 70.0, 45.0, 50.0, 80.0, 85.0, 55.0, 42.0, 222.0],
    )
}

fn rules() -> PrashnaRules {
    PrashnaRules::default()
}

#[test]
fn every_clause_for_succeeds_and_every_clause_against_fails() {
    let read_gemini = read(&gemini(), Question::default(), rules()).unwrap();
    assert_eq!(read_gemini.verdict.outcome, Outcome::Succeeds);
    let in_lagna: Vec<Graha> = read_gemini
        .verdict
        .clauses
        .iter()
        .filter(|clause| clause.kind == ClauseKind::InLagna)
        .filter_map(|clause| clause.graha)
        .collect();
    assert_eq!(in_lagna, [Graha::Moon, Graha::Jupiter, Graha::Venus]);

    // Aries rising (by the back), a waning Moon, Mars and Saturn in it, the
    // rising navāṁśa Aquarius (Saturn's), the benefics in Taurus: every
    // clause is against.
    let aries = sky(
        10.0,
        Rashi::Aquarius,
        [100.0, 10.0, 15.0, 40.0, 45.0, 50.0, 20.0, 160.0, 340.0],
    );
    let read_aries = read(&aries, Question::default(), rules()).unwrap();
    assert_eq!(read_aries.verdict.outcome, Outcome::Fails);
    assert!(
        read_aries
            .verdict
            .clauses
            .iter()
            .all(|clause| clause.favour == Favour::Against)
    );
}

#[test]
fn pisces_rises_both_ways_unless_the_rules_say_by_the_head() {
    // Pisces rising with Jupiter, Venus and a waxing Moon in it; the
    // malefics and Mercury in Aquarius, which aspects nothing in Pisces.
    let pisces = sky(
        340.0,
        Rashi::Pisces,
        [
            320.0, 345.0, 325.0, 330.0, 350.0, 355.0, 315.0, 100.0, 280.0,
        ],
    );
    let default = read(&pisces, Question::default(), rules()).unwrap();
    assert_eq!(default.verdict.clauses[0].favour, Favour::Both);
    assert_eq!(default.verdict.outcome, Outcome::WithDifficulty);
    let by_the_head = PrashnaRules {
        pisces: PiscesRising::Shirshodaya,
        ..rules()
    };
    let read = read(&pisces, Question::default(), by_the_head).unwrap();
    assert_eq!(read.verdict.outcome, Outcome::Succeeds);
}

#[test]
fn a_house_asked_about_is_judged_by_its_lord_and_who_aspects_it() {
    // Gemini rising, the 7th asked: Sagittarius, Jupiter's sign. Jupiter,
    // Venus and the Moon in Gemini aspect it fully (the 7th), and so does
    // Mars from Taurus (his 8th), a malefic.
    let question = Question::about(7);
    let answer = read(&gemini(), question, rules()).unwrap();
    let karya: Vec<(Option<Graha>, Favour)> = answer
        .verdict
        .clauses
        .iter()
        .filter(|clause| clause.kind == ClauseKind::KaryaHouse)
        .map(|clause| (clause.graha, clause.favour))
        .collect();
    assert_eq!(
        karya,
        [
            (Some(Graha::Moon), Favour::For),
            (Some(Graha::Mars), Favour::Against),
            (Some(Graha::Jupiter), Favour::For),
            (Some(Graha::Venus), Favour::For),
        ]
    );
    assert_eq!(ClauseKind::KaryaHouse.source(), "Shatpanchashika I.3");
}

#[test]
fn the_strongest_grahas_house_gives_months_by_its_navamsha() {
    // II.14–15: Saturn, strongest, in Taurus, the 12th from Gemini; a
    // fixed navāṁśa doubles the twelve months.
    let mut fixed = gemini();
    fixed.grahas[6].navamsha = Rashi::Leo;
    let timing = read(&fixed, Question::default(), rules()).unwrap().timing;
    assert_eq!(
        (timing.graha, timing.count, timing.multiplier),
        (Graha::Saturn, 12, 2)
    );
    assert_eq!((timing.amount, timing.unit), (Some(24), Unit::Months));
    assert!(!timing.tie);

    // Venus tied with Saturn: the earlier in weekday order is Venus, and
    // the answer says it broke a tie.
    let mut tied = gemini();
    tied.strength[5] = tied.strength[6];
    let timing = read(&tied, Question::default(), rules()).unwrap().timing;
    assert_eq!(
        (timing.graha, timing.count, timing.tie),
        (Graha::Venus, 1, true)
    );
}

#[test]
fn the_first_occupied_sign_counts_twelve_days_or_one_when_retrograde() {
    // V.5's example: Taurus rising, Virgo the first sign held, five on.
    let mut taurus = sky(
        40.0,
        Rashi::Leo,
        [
            160.0, 170.0, 200.0, 230.0, 260.0, 290.0, 320.0, 350.0, 170.0,
        ],
    );
    let first = PrashnaRules {
        timing: TimingRule::FirstOccupied,
        ..rules()
    };
    let timing = read(&taurus, Question::default(), first).unwrap().timing;
    // The Moon is the stronger of the two in Virgo; Ketu has no Shadbala.
    assert_eq!((timing.graha, timing.amount), (Graha::Moon, Some(60)));
    taurus.grahas[1].retrograde = true;
    let timing = read(&taurus, Question::default(), first).unwrap().timing;
    assert_eq!((timing.amount, timing.unit), (Some(5), Unit::Days));
}

#[test]
fn a_graha_between_the_lagna_and_the_moon_gives_no_time() {
    // II.17: Taurus rising, the Moon in Leo, four signs on.
    let moon_days = PrashnaRules {
        timing: TimingRule::MoonDays,
        ..rules()
    };
    let clear = sky(
        40.0,
        Rashi::Leo,
        [
            130.0, 130.0, 200.0, 230.0, 260.0, 290.0, 320.0, 350.0, 170.0,
        ],
    );
    let timing = read(&clear, Question::default(), moon_days).unwrap().timing;
    assert_eq!((timing.count, timing.amount), (4, Some(4)));
    assert_eq!(timing.between, []);
    let mut blocked = clear;
    blocked.grahas[2].longitude_deg = 70.0;
    let timing = read(&blocked, Question::default(), moon_days)
        .unwrap()
        .timing;
    assert_eq!(
        (timing.amount, timing.between.as_slice()),
        (None, &[Graha::Mars][..])
    );
}

#[test]
fn the_baseline_times_from_the_lagna_lord_by_the_lagnas_modality() {
    // Gemini, dual: Mercury in Taurus, the 12th; years.
    let timing = read(&gemini(), Question::default(), PrashnaRules::baseline())
        .unwrap()
        .timing;
    assert_eq!(
        (timing.graha, timing.amount, timing.unit),
        (Graha::Mercury, Some(12), Unit::Years)
    );
    assert_eq!(timing.rule, TimingRule::Baseline);
    assert_ne!(rules().timing, TimingRule::Baseline);
}

#[test]
fn an_unspoken_question_names_a_person_for_eight_houses_only() {
    let person_with_saturn_in = |longitude_deg: f64| {
        let mut moved = gemini();
        moved.grahas[6].longitude_deg = longitude_deg;
        let mook = read(&moved, Question::default(), rules()).unwrap().mook;
        (mook.house, mook.person)
    };
    // Saturn in Taurus is the 12th, which VII.7–8 does not list.
    assert_eq!(person_with_saturn_in(55.0), (12, Some(Person::Unspecified)));
    assert_eq!(person_with_saturn_in(265.0), (7, Some(Person::Wife)));
    assert_eq!(person_with_saturn_in(325.0), (9, Some(Person::Religious)));
    // In the lagna, read against the lagna navāṁśa's lord, Jupiter:
    // Saturn is his neutral, so one like the querent; Venus his enemy.
    assert_eq!(person_with_saturn_in(85.0).1, Some(Person::Querent));
    let mut venus = gemini();
    venus.strength[5] = 8.0;
    let mook = read(&venus, Question::default(), rules()).unwrap().mook;
    assert_eq!(
        (mook.graha, mook.person),
        (Graha::Venus, Some(Person::Enemy))
    );

    // The Moon's house, or the stronger lagna lord's: Mercury (4.0) is
    // stronger than the Moon (2.0), in Taurus, the 12th.
    let moon_house = PrashnaRules {
        mook: MookRule::MoonHouse,
        ..rules()
    };
    let mook = read(&gemini(), Question::default(), moon_house)
        .unwrap()
        .mook;
    assert_eq!(
        (mook.graha, mook.house, mook.person),
        (Graha::Mercury, 12, None)
    );
}

#[test]
fn the_thought_runs_mineral_root_living_from_an_odd_signs_start() {
    let thought_at = |lagna_deg: f64| {
        read(
            &sky(lagna_deg, Rashi::Aries, gemini_at()),
            Question::default(),
            rules(),
        )
        .unwrap()
        .mook
        .thought
    };
    // Aries, odd: the first three navāṁśas.
    assert_eq!(thought_at(1.0), Thought::Mineral);
    assert_eq!(thought_at(4.0), Thought::Root);
    assert_eq!(thought_at(7.0), Thought::Living);
    assert_eq!(thought_at(11.0), Thought::Mineral);
    // Taurus, even: the reverse.
    assert_eq!(thought_at(31.0), Thought::Living);
    assert_eq!(thought_at(34.0), Thought::Root);
    assert_eq!(thought_at(37.0), Thought::Mineral);
}

fn gemini_at() -> [f64; 9] {
    gemini().grahas.map(|placed| placed.longitude_deg)
}

#[test]
fn a_dual_lagna_stays_in_its_first_half_and_changes_in_its_second() {
    let change_at = |lagna_deg: f64| {
        read(
            &sky(lagna_deg, Rashi::Aries, gemini_at()),
            Question::default(),
            rules(),
        )
        .unwrap()
        .change
    };
    assert_eq!(change_at(10.0), Change::Changes);
    assert_eq!(change_at(40.0), Change::Stays);
    assert_eq!(change_at(64.0), Change::Stays);
    assert_eq!(change_at(76.0), Change::Changes);
}

#[test]
fn a_house_out_of_range_or_a_number_that_is_not_one_is_refused() {
    for house in [0, 13] {
        let error = read(&gemini(), Question::about(house), rules()).unwrap_err();
        assert_eq!(error.field(), Some("question.house"));
    }
    let mut broken = gemini();
    broken.lagna_deg = f64::NAN;
    let error = read(&broken, Question::default(), rules()).unwrap_err();
    assert_eq!(error.field(), Some("lagnaDeg"));
}

#[test]
fn the_keys_are_what_serde_writes() {
    let answer =
        serde_json::to_value(read(&gemini(), Question::default(), rules()).unwrap()).unwrap();
    assert_eq!(answer["verdict"]["outcome"], "SUCCEEDS");
    assert_eq!(
        answer["verdict"]["clauses"][0],
        serde_json::json!({"kind": "LAGNA_RISING", "graha": null, "favour": "FOR"})
    );
    assert_eq!(answer["timing"]["rule"], "STRONGEST_GRAHA");
    assert_eq!(answer["mook"]["thought"], "ROOT");
    let rules: PrashnaRules = serde_json::from_str(r#"{"timing": "MOON_DAYS"}"#).unwrap();
    assert_eq!(rules.timing, TimingRule::MoonDays);
    assert!(serde_json::from_str::<PrashnaRules>(r#"{"time": "MOON_DAYS"}"#).is_err());
}

#[test]
fn the_prashna_tantras_nakta_example_is_read_as_printed() {
    // Tajika Nilakanthi, Prashna Tantra vv. 18–19: Virgo rising, a
    // question about a woman (the 7th, Pisces, Jupiter's). Mercury, the
    // lagna lord, in Leo; Jupiter in Pisces; the two do not aspect. The
    // fast Moon in Sagittarius has passed Mercury's trine and comes to
    // Jupiter's square, carrying the light: the woman is won through
    // another's hand.
    let virgo = sky(
        160.0,
        Rashi::Virgo,
        [165.0, 252.0, 280.0, 130.0, 350.0, 190.0, 310.0, 40.0, 220.0],
    );
    let links = read(&virgo, Question::about(7), rules())
        .unwrap()
        .links
        .unwrap();
    assert_eq!(
        (links.lagnesha, links.karyesha),
        (Graha::Mercury, Graha::Jupiter)
    );
    let nakta = links
        .held
        .iter()
        .find(|held| held.yoga == teistro_tajika::YearYoga::Nakta)
        .unwrap();
    assert_eq!(nakta.through, Some(Graha::Moon));
    assert_eq!(
        read(&virgo, Question::default(), rules()).unwrap().links,
        None
    );
}

#[test]
fn the_moon_is_read_in_a_query_as_at_birth() {
    // The nakta sky: the Moon 87° past the Sun, inside the gloss's dark
    // 8th to bright 8th but not the dark 11th to the new Moon.
    let virgo = sky(
        160.0,
        Rashi::Virgo,
        [165.0, 252.0, 280.0, 130.0, 350.0, 190.0, 310.0, 40.0, 220.0],
    );
    let kshina = |kshina: KshinaRule| {
        let rules = PrashnaRules {
            moon: MoonRules { kshina },
            ..rules()
        };
        read(&virgo, Question::default(), rules)
            .unwrap()
            .moon
            .clauses
            .contains(&MoonClause::Kshina)
    };
    assert!(kshina(KshinaRule::DarkEighthToBrightEighth));
    assert!(!kshina(KshinaRule::DarkEleventhToNewMoon));
}

/// Aries rising; in the order of [`crate::GRAHAS`]: the Sun and Mercury in
/// Cancer (4th), the Moon at `moon_deg` in Aries (1st), Mars in Capricorn
/// (10th), Jupiter in Leo (5th), Venus in Gemini (3rd), Saturn in Libra
/// (7th), Rahu in Pisces and Ketu in Virgo.
fn aries(moon_deg: f64) -> PrashnaSky {
    sky(
        10.0,
        Rashi::Aries,
        [
            100.0, moon_deg, 280.0, 110.0, 125.0, 60.0, 200.0, 331.0, 151.0,
        ],
    )
}

fn scored(sky: &PrashnaSky) -> Score {
    let rules = PrashnaRules {
        score: ScoreRule::Baseline,
        ..rules()
    };
    read(sky, Question::default(), rules)
        .unwrap()
        .score
        .unwrap()
}

#[test]
fn the_baseline_scores_what_it_runs() {
    // Mars, the lagna lord, in the 10th (+2); the Moon waning; Moon and
    // Mercury in kendras (+2), the Sun, Mars and Saturn (−3); the Moon at
    // 15° Aries meets Mercury's square at 20° before the sign ends, and
    // Venus is the nearest ahead of her (+1).
    let score = scored(&aries(15.0));
    let factors: Vec<(FactorKind, i8)> = score
        .factors
        .iter()
        .map(|factor| (factor.kind, factor.points))
        .collect();
    assert_eq!(
        factors,
        [
            (FactorKind::LagnaLordKendraOrTrikona, 2),
            (FactorKind::BeneficsInKendras, 2),
            (FactorKind::MaleficsInKendras, -3),
            (FactorKind::MoonApplyingToBenefic, 1),
        ]
    );
    assert_eq!((score.points, score.answer), (2, Answer::Yes));
    assert!(!score.void);
    // At 28° she perfects nothing held still before Taurus: −3, and
    // the answer turns uncertain.
    let late = scored(&aries(28.0));
    assert!(late.void);
    assert_eq!((late.points, late.answer), (-1, Answer::Uncertain));
    assert_eq!(late.applying_to, Some(Graha::Venus));
    // Off by default, and the sourced verdict is there either way.
    let plain = read(&aries(15.0), Question::default(), rules()).unwrap();
    assert_eq!(plain.score, None);
    assert_eq!(rules().score, ScoreRule::Off);
}

#[test]
fn the_baselines_topic_breaks_a_tie_in_its_own_order() {
    // The Moon alone scores 3 (the 1st is a kendra and a trikona).
    let topic = |sky: &PrashnaSky| {
        let rules = PrashnaRules {
            mook: MookRule::Baseline,
            ..rules()
        };
        read(sky, Question::default(), rules).unwrap().mook
    };
    let moon = topic(&aries(15.0));
    assert_eq!(
        (moon.graha, moon.house, moon.tie, moon.person),
        (Graha::Moon, 1, false, None)
    );
    // Everything in Gemini but Mercury (4th) and Mars (10th), 2 each: the
    // baseline lists Mercury before Mars, as the catalogue does not.
    let tied = sky(
        10.0,
        Rashi::Aries,
        [70.0, 75.0, 280.0, 100.0, 65.0, 66.0, 68.0, 331.0, 151.0],
    );
    let read_tied = topic(&tied);
    assert_eq!(
        (read_tied.graha, read_tied.house, read_tied.tie),
        (Graha::Mercury, 4, true)
    );
}

#[test]
fn the_baselines_number_counts_signs_from_aries() {
    assert_eq!(number_sign(1).unwrap(), Rashi::Aries);
    assert_eq!(number_sign(13).unwrap(), Rashi::Aries);
    assert_eq!(number_sign(108).unwrap(), Rashi::Pisces);
    assert!(number_sign(0).is_err() && number_sign(109).is_err());
    let asked = Question {
        number: Some(14),
        ..Question::default()
    };
    assert_eq!(
        read(&aries(15.0), asked, rules()).unwrap().number_sign,
        Some(Rashi::Taurus)
    );
    assert_eq!(
        read(&aries(15.0), Question::default(), rules())
            .unwrap()
            .number_sign,
        None
    );
}

#[test]
fn the_default_reaches_no_baseline_value_and_baseline_reaches_all() {
    let (default, baseline) = (PrashnaRules::default(), PrashnaRules::baseline());
    assert_ne!(default.timing, TimingRule::Baseline);
    assert_ne!(default.mook, MookRule::Baseline);
    assert_eq!(default.score, ScoreRule::Off);
    assert_eq!(
        (baseline.timing, baseline.mook, baseline.score),
        (
            TimingRule::Baseline,
            MookRule::Baseline,
            ScoreRule::Baseline
        )
    );
}
