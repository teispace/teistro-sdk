//! Every number a source prints, held to its page, and every rival
//! reading pinned beside the default it differs from.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::trivially_copy_pass_by_ref,
    reason = "tests fail by panicking, index their own tables and pass rules as the API does"
)]

use super::*;

fn name(text: &str, system: System, rules: &NumerologyRules) -> NameNumber {
    name_number(text, system, rules).unwrap()
}

fn word_numbers(number: &NameNumber) -> Vec<(u32, u32)> {
    number
        .words
        .iter()
        .map(|word| (word.total, word.reduction.number))
        .collect()
}

#[test]
fn each_table_is_its_page() {
    // Balliett p. 17: the cycle of nine.
    for (index, value) in BALLIETT.iter().enumerate() {
        assert_eq!(usize::from(*value), index % 9 + 1);
    }
    // Cheiro p. 70, letter by letter.
    let cheiro: Vec<(char, u8)> = ('A'..='Z').zip(CHEIRO).collect();
    assert_eq!(
        cheiro,
        [
            ('A', 1),
            ('B', 2),
            ('C', 3),
            ('D', 4),
            ('E', 5),
            ('F', 8),
            ('G', 3),
            ('H', 5),
            ('I', 1),
            ('J', 1),
            ('K', 2),
            ('L', 3),
            ('M', 4),
            ('N', 5),
            ('O', 7),
            ('P', 8),
            ('Q', 1),
            ('R', 2),
            ('S', 3),
            ('T', 4),
            ('U', 6),
            ('V', 6),
            ('W', 6),
            ('X', 5),
            ('Y', 1),
            ('Z', 7),
        ]
    );
    assert!(!CHEIRO.contains(&9), "no letter is 9 in Cheiro's table");
    // Sepharial's Hebraic key (p. 30) differs in three cells, C 2, H 8 and
    // X 6; it is a different key, so a "correction" towards it is a fault.
    assert_eq!((CHEIRO[2], CHEIRO[7], CHEIRO[23]), (3, 5, 5));
}

#[test]
fn balliett_s_examples_read_as_printed() {
    let rules = NumerologyRules::default();
    // pp. 18-19: Henry 34, so 7; Elder 26, so 8; the name 15, so 6.
    let henry = name("Henry Elder", System::Pythagorean, &rules);
    assert_eq!(word_numbers(&henry), [(34, 7), (26, 8)]);
    assert_eq!((henry.total, henry.reduction.number), (15, 6));
    // p. 13: John and Sarah are both 2.
    assert_eq!(
        name("John", System::Pythagorean, &rules).reduction.number,
        2
    );
    assert_eq!(
        name("Sarah", System::Pythagorean, &rules).reduction.number,
        2
    );
    // pp. 30-31: White 29 = 11, Cream 22, Orange 33 = 6.
    assert_eq!(
        word_numbers(&name("White", System::Pythagorean, &rules)),
        [(29, 11)]
    );
    assert_eq!(
        name("Cream", System::Pythagorean, &rules).reduction.number,
        22
    );
    assert_eq!(
        word_numbers(&name("Orange", System::Pythagorean, &rules)),
        [(33, 6)]
    );
    // p. 48: Mary 3 and Patterson 38, so 11, which stands.
    let mary = name("Mary Patterson", System::Pythagorean, &rules);
    assert_eq!(word_numbers(&mary), [(21, 3), (38, 11)]);
    // p. 90: John 2 and Wanamaker 33, so 6; the name 8.
    let wanamaker = name("John Wanamaker", System::Pythagorean, &rules);
    assert_eq!(word_numbers(&wanamaker), [(20, 2), (33, 6)]);
    assert_eq!(wanamaker.reduction.number, 8);
}

#[test]
fn balliett_s_birth_numbers_read_as_printed() {
    let rules = NumerologyRules::default();
    // p. 19: 17 January 1872 is 1 + 8 + 9 = 18, so 9.
    let elder = pythagorean_birth(BirthDate::new(1872, 1, 17).unwrap(), &rules);
    assert_eq!(
        (elder.month.number, elder.day.number, elder.year.number),
        (1, 8, 9)
    );
    assert_eq!(elder.sum.map(|sum| sum.steps), Some(vec![18, 9]));
    assert_eq!(elder.apart, Vec::<u32>::new());
    // p. 90: 11 July 1838 is printed "9, 11": the day's 11 stands apart
    // from the month's 7 and the year's 2.
    let wanamaker = pythagorean_birth(BirthDate::new(1838, 7, 11).unwrap(), &rules);
    assert_eq!(wanamaker.sum.map(|sum| sum.number), Some(9));
    assert_eq!(wanamaker.apart, [11]);
}

#[test]
fn cheiro_s_examples_read_as_printed() {
    let rules = NumerologyRules::default();
    // pp. 71-72: Lloyd 18, so 9; George 25, so 7; the compound 16.
    let lloyd = name("Lloyd George", System::Chaldean, &rules);
    assert_eq!(word_numbers(&lloyd), [(18, 9), (25, 7)]);
    assert_eq!(lloyd.compound, Some(16));
    assert_eq!(lloyd.reduction.number, 7);
    // p. 72: David Lloyd George 23; Baldwin 22, which is a 4.
    assert_eq!(
        name("David Lloyd George", System::Chaldean, &rules).compound,
        Some(23)
    );
    let baldwin = name("Baldwin", System::Chaldean, &rules);
    assert_eq!((baldwin.compound, baldwin.reduction.number), (Some(22), 4));
    // p. 87: John 9 and Smith 8, 17, so 8.
    let smith = name("John Smith", System::Chaldean, &rules);
    assert_eq!(smith.compound, Some(17));
    assert_eq!(smith.reduction.number, 8);
    // p. 35: no master stands; an 11 is a 2.
    assert_eq!(reduce(11, Masters::None).number, 2);
}

#[test]
fn every_rival_reading_is_pinned_against_the_default() {
    let source = NumerologyRules::default();
    let baseline = NumerologyRules::baseline();
    // 33 stands only under the baseline.
    assert_eq!(
        name("Orange", System::Pythagorean, &source)
            .reduction
            .number,
        6
    );
    assert_eq!(
        name("Orange", System::Pythagorean, &baseline)
            .reduction
            .number,
        33
    );
    // By word 1 + 1 = 2; whole 47, so 11.
    assert_eq!(
        name("Peter Stone", System::Pythagorean, &source)
            .reduction
            .number,
        2
    );
    assert_eq!(
        name("Peter Stone", System::Pythagorean, &baseline)
            .reduction
            .number,
        11
    );
    // Cheiro's compound 16, the letter total 43.
    assert_eq!(
        name("Lloyd George", System::Chaldean, &source).compound,
        Some(16)
    );
    assert_eq!(
        name("Lloyd George", System::Chaldean, &baseline).compound,
        Some(43)
    );
    // Balliett's "9, 11", the baseline's digit sum 2 + 7 + 20 = 29, so 11.
    let date = BirthDate::new(1838, 7, 11).unwrap();
    let by_part = pythagorean_birth(date, &source);
    let digit_sum = pythagorean_birth(date, &baseline);
    assert_eq!(
        (by_part.sum.map(|s| s.number), by_part.apart),
        (Some(9), vec![11])
    );
    assert_eq!(digit_sum.sum.map(|s| s.steps), Some(vec![29, 11]));
}

#[test]
fn the_baseline_profile_appears_only_under_the_baseline() {
    let date = BirthDate::new(1872, 1, 17).unwrap();
    let source = profile("Henry Elder", date, &NumerologyRules::default()).unwrap();
    assert!(source.baseline.is_none());
    let baseline = profile("Henry Elder", date, &NumerologyRules::baseline()).unwrap();
    let numbers = baseline.baseline.unwrap();
    // Vowels E, E, E: 15, so 6. Y is not a vowel (Balliett p. 98), and
    // with it the sum would be 22.
    assert_eq!(numbers.soul.steps, [15, 6]);
    // Consonants H N R Y L D R: 8 + 5 + 9 + 7 + 3 + 4 + 9 = 45, so 9.
    assert_eq!(numbers.personality.steps, [45, 9]);
    // 17 + 1 + 1872: 8 + 1 + 18 = 27, so 9, with no master kept.
    assert_eq!(numbers.chaldean_destiny.number, 9);
}

#[test]
fn cheiro_s_date_numbers_stand_apart() {
    // p. 93: born 6 June 1866, the birth number 6 and the year 21, so 3,
    // never added together.
    let date = chaldean_birth(BirthDate::new(1866, 6, 6).unwrap());
    assert_eq!(date.birth.number, 6);
    assert_eq!(date.year.steps, [1866, 21, 3]);
}

#[test]
fn a_compound_s_meaning_follows_cheiro_s_rule() {
    assert_eq!(compound_class(9), None);
    assert_eq!(compound_class(16), Some(16));
    assert_eq!(compound_class(33), Some(24));
    assert_eq!(compound_class(52), Some(43));
    for own in [37, 43, 51] {
        assert_eq!(compound_class(own), Some(own));
    }
    assert_eq!(compound_class(53), None);
}

#[test]
fn a_reduction_keeps_the_number_s_residue_mod_nine() {
    for masters in [
        Masters::None,
        Masters::ElevenTwentyTwo,
        Masters::ElevenTwentyTwoThirtyThree,
    ] {
        for n in 1..=1_000_000u32 {
            let number = reduce(n, masters).number;
            assert_eq!(number % 9, n % 9, "{n} under {masters:?}");
            assert!(number <= 9 || masters.stops(number));
        }
    }
    assert_eq!(reduce(0, Masters::None).number, 0);
}

#[test]
fn the_planets_and_days_of_a_number_are_cheiro_s() {
    assert_eq!(planets_of(4), [Graha::Uranus, Graha::Sun]);
    assert_eq!(planets_of(7), [Graha::Neptune, Graha::Moon]);
    assert_eq!(planets_of(0), &[] as &[Graha]);
    assert_eq!(baseline_anka(Graha::Rahu), Some(4));
    assert_eq!(baseline_anka(Graha::Uranus), None);
    assert_eq!(days_of(5), [5, 14, 23]);
}

#[test]
fn a_name_outside_latin_letters_is_refused_and_named() {
    let source = NumerologyRules::default();
    let refused = name_number("Kṛṣṇa", System::Pythagorean, &source).unwrap_err();
    assert_eq!(refused.field(), Some("name"));
    assert!(refused.to_string().contains("character 2"), "{refused}");
    // The baseline leaves out ṛ, ṣ and ṇ alike, and reads K and A.
    let skipped = name("Kṛṣṇa", System::Pythagorean, &NumerologyRules::baseline());
    assert_eq!(skipped.words[0].total, 2 + 1);
    for empty in ["", "  ", "--'"] {
        assert!(
            name_number(empty, System::Chaldean, &source).is_err(),
            "{empty:?}"
        );
    }
    // Separators split words and are never refused.
    let hyphened = name("Jean-Paul O'Neil", System::Pythagorean, &source);
    assert_eq!(hyphened.words.len(), 4);
}

#[test]
fn a_date_is_held_to_the_calendar() {
    assert!(BirthDate::new(2023, 2, 29).is_err());
    assert!(BirthDate::new(2024, 2, 29).is_ok());
    assert!(BirthDate::new(2024, 13, 1).is_err());
    assert!(BirthDate::new(0, 1, 1).is_err());
}

#[test]
fn the_rules_read_their_documented_spellings() {
    let rules: NumerologyRules = serde_json::from_str(
        r#"{"masters":"ELEVEN_TWENTY_TWO_THIRTY_THREE","nameReduction":"WHOLE",
            "chaldeanCompound":"LETTER_TOTAL","birthReduction":"DIGIT_SUM","nonLatin":"SKIP"}"#,
    )
    .unwrap();
    assert_eq!(rules, NumerologyRules::baseline());
    assert!(serde_json::from_str::<NumerologyRules>(r#"{"master":"NONE"}"#).is_err());
}
