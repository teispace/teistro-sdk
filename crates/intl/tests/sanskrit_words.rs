//! Sanskrit numbers and amounts in words, by the SDK's drafted rules
//! (`crates/intl/drafts/rbnf/sa.txt`, `docs/03-design/amounts-in-words.md`
//! §5): each worked value asserted against the grammar that gives it, and
//! every answer marked unreviewed until the consumer's reviewer (Q42)
//! signs the forms off (C1310 to C1319).

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "a test fails by panicking"
)]

use teistro_intl::source::Tree;
use teistro_intl::words::rbnf::parse;
use teistro_intl::words::{DRAFTED, InWords, InWordsRequest};
use teistro_intl::{Intl, sdk_root};

fn intl() -> Intl {
    Intl::from_tree(&Tree::load(&sdk_root()).unwrap()).unwrap()
}

fn asked(intl: &Intl, json: &str) -> InWords {
    intl.in_words(&InWordsRequest::from_json(json).unwrap())
        .unwrap_or_else(|e| panic!("{json}: {e}"))
}

fn words(intl: &Intl, n: i64, locale: &str) -> String {
    let answer = asked(intl, &format!(r#"{{"number": {n}, "locale": "{locale}"}}"#));
    assert!(answer.is_draft, "{n} in {locale} is the SDK's draft");
    answer.text
}

/// The worked values, each with the section it is read from: M. R. Kale,
/// *A Higher Sanskrit Grammar* §§159–161 (K), W. D. Whitney, *A Sanskrit
/// Grammar* §§475–481 (W).
#[test]
fn the_grammars_worked_values() {
    let intl = intl();
    for (n, expected) in [
        (0, "शून्यम्"),
        // K §159; the neuter, as a number is counted (C1311).
        (1, "एकम्"),
        (2, "द्वे"),
        (3, "त्रीणि"),
        (4, "चत्वारि"),
        (5, "पञ्च"),
        (8, "अष्ट"),
        (10, "दश"),
        // K §159, W §476 f.
        (11, "एकादश"),
        (12, "द्वादश"),
        (13, "त्रयोदश"),
        (16, "षोडश"),
        (18, "अष्टादश"),
        // The first of K's four forms of 19 (C1312).
        (19, "नवदश"),
        (20, "विंशतिः"),
        (21, "एकविंशतिः"),
        (22, "द्वाविंशतिः"),
        (23, "त्रयोविंशतिः"),
        (26, "षड्विंशतिः"),
        (28, "अष्टाविंशतिः"),
        (33, "त्रयस्त्रिंशत्"),
        (36, "षट्त्रिंशत्"),
        // dvā and trayas, K's first form where Pāṇini VI.3.49 allows two (C1313).
        (42, "द्वाचत्वारिंशत्"),
        (43, "त्रयश्चत्वारिंशत्"),
        (44, "चतुश्चत्वारिंशत्"),
        (53, "त्रयःपञ्चाशत्"),
        (64, "चतुःषष्टिः"),
        (72, "द्वासप्ततिः"),
        (80, "अशीतिः"),
        (81, "एकाशीतिः"),
        // Unchanged before aśīti (K §160).
        (82, "द्व्यशीतिः"),
        (83, "त्र्यशीतिः"),
        (86, "षडशीतिः"),
        (88, "अष्टाशीतिः"),
        (90, "नवतिः"),
        (93, "त्रयोनवतिः"),
        // W §476 d: the n of navati assimilated to ṣaṣ.
        (96, "षण्णवतिः"),
        (99, "नवनवतिः"),
        // K §159.
        (100, "शतम्"),
        // K §161: एकाधिकं शतं or एकाधिकशतं; the compound (C1315).
        (101, "एकाधिकशतम्"),
        (112, "द्वादशाधिकशतम्"),
        (150, "पञ्चाशदधिकशतम्"),
        // K §159: द्विशत and त्रिशत, the compounds W §481 calls usual later.
        (200, "द्विशतम्"),
        (300, "त्रिशतम्"),
        (999, "नवनवत्यधिकनवशतम्"),
        (1000, "सहस्रम्"),
        (1001, "एकाधिकसहस्रम्"),
        // K §161, letter for letter.
        (1892, "द्वानवत्यधिकाष्टशताधिकसहस्रम्"),
        (8000, "अष्टसहस्रम्"),
        (8001, "एकाधिकाष्टसहस्रम्"),
        (25_000, "पञ्चविंशतिसहस्रम्"),
        (30_000, "त्रिंशत्सहस्रम्"),
        // K §159: लक्ष neuter, कोटि feminine.
        (100_000, "लक्षम्"),
        (200_000, "द्विलक्षम्"),
        (3_000_000, "त्रिंशल्लक्षम्"),
        (10_000_000, "कोटिः"),
        (40_000_000, "चतुष्कोटिः"),
        // A hundred crore, and a crore of crores (C1314).
        (1_000_000_000, "शतकोटिः"),
        (100_000_000_000_000, "कोटिकोटिः"),
        (-5, "ऋणम् पञ्च"),
    ] {
        assert_eq!(words(&intl, n, "sa-Deva"), expected, "{n}");
    }
}

/// K §161's second example, 177639, written with the two choices the
/// draft makes where Kale allows another: 39 as नवत्रिंशत् where Kale's
/// example writes एकोनचत्वारिंशत् (C1312), and the last joint compounded
/// where it writes `अधिकं लक्षम्` (C1315).
#[test]
fn kales_177639_with_the_drafts_two_choices() {
    let kale = "एकोनचत्वारिंशदधिकषट्शताधिकसप्तसप्ततिसहस्राधिकं लक्षम्";
    let drafted = kale
        .replace("एकोनचत्वारिंशद", "नवत्रिंशद")
        .replace("धिकं लक्षम्", "धिकलक्षम्");
    assert_eq!(words(&intl(), 177_639, "sa-Deva"), drafted);
}

/// W §475 d: "123,45,67,890, to signify 1-23 crores, 45 lakhs, 67
/// thousand, eight hundred and ninety": crores counted past ninety-nine.
#[test]
fn whitneys_indian_sum_counts_its_crores() {
    let intl = intl();
    let answer = asked(&intl, r#"{"number": 1234567890, "locale": "sa-Deva"}"#);
    assert_eq!(
        answer.text,
        "नवत्यधिकाष्टशताधिकसप्तषष्टिसहस्राधिकपञ्चचत्वारिंशल्लक्षाधिकत्रयोविंशत्यधिकशतकोटिः"
    );
    assert_eq!(answer.figure, "१,२३,४५,६७,८९०");
    assert_eq!(answer.rules, "draft/sa/spellout-cardinal");
    // Every i64 is spelt: crores count with a whole number.
    for n in [i64::MAX, i64::MIN + 1] {
        let text = words(&intl, n, "sa-Deva");
        assert!(text.ends_with("कोटिः"), "{n}: {text}");
        assert!(!text.contains('+'), "{n}: every joint closed");
    }
}

#[test]
fn the_latin_locale_reads_the_same_words_transliterated() {
    let intl = intl();
    for (n, expected) in [
        (0, "śūnyam"),
        (1, "ekam"),
        (19, "navadaśa"),
        (21, "ekaviṃśatiḥ"),
        (99, "navanavatiḥ"),
        (101, "ekādhikaśatam"),
        (1892, "dvānavatyadhikāṣṭaśatādhikasahasram"),
        (100_000, "lakṣam"),
        (10_000_000, "koṭiḥ"),
    ] {
        assert_eq!(words(&intl, n, "sa-Latn"), expected, "{n}");
    }
    let lakh = asked(&intl, r#"{"number": 150000, "locale": "sa-Latn"}"#);
    // The words count in lakhs, so the figures are grouped as they are,
    // though `sa-Latn` groups its other numbers by three.
    assert_eq!(lakh.figure, "1,50,000");
    assert_eq!(lakh.text, "pañcāśatsahasrādhikalakṣam");
    assert_eq!(lakh.rules, "draft/sa/spellout-cardinal/iast");
}

#[test]
fn an_amount_reads_in_rupees_and_paisa() {
    let intl = intl();
    let cheque = asked(
        &intl,
        r#"{"amount": {"currency": "INR", "value": {"minor": 123456}, "only": true}, "locale": "sa-Deva"}"#,
    );
    assert_eq!(
        cheque.text,
        "चतुस्त्रिंशदधिकद्विशताधिकसहस्रम् रूप्यकाणि षट्पञ्चाशत् पैसा च मात्रम्"
    );
    assert_eq!(cheque.figure, "१,२३४.५६");
    assert!(cheque.is_draft);
    let parts = cheque.amount.unwrap();
    assert_eq!(parts.major_unit, "रूप्यकाणि");
    assert_eq!(parts.minor_words.as_deref(), Some("षट्पञ्चाशत्"));

    // The unit agrees with one and two (W §482 d), and is plural past them.
    let say = |json: &str| asked(&intl, json).text;
    assert_eq!(
        say(r#"{"amount": {"currency": "INR", "value": {"minor": 100}}, "locale": "sa-Deva"}"#),
        "एकम् रूप्यकम्"
    );
    assert_eq!(
        say(r#"{"amount": {"currency": "INR", "value": {"minor": 200}}, "locale": "sa-Deva"}"#),
        "द्वे रूप्यके"
    );
    assert_eq!(
        say(
            r#"{"amount": {"currency": "NPR", "value": {"minor": 10000000}}, "locale": "sa-Deva"}"#
        ),
        "लक्षम् नेपालरूप्यकाणि"
    );
    assert_eq!(
        say(r#"{"amount": {"currency": "NPR", "value": {"decimal": "0.5"}}, "locale": "sa-Deva"}"#),
        "पञ्चाशत् पैसा"
    );

    let latin = asked(
        &intl,
        r#"{"amount": {"currency": "INR", "value": {"minor": 123456}, "only": true}, "locale": "sa-Latn"}"#,
    );
    assert_eq!(
        latin.text,
        "catustriṃśadadhikadviśatādhikasahasram rūpyakāṇi ṣaṭpañcāśat paisā ca mātram"
    );
    assert_eq!(latin.figure, "1,234.56");
    assert!(latin.is_draft);
}

/// Every other language's words are reviewed, CLDR's, and say so.
#[test]
fn only_the_draft_says_draft() {
    let intl = intl();
    for locale in ["en-Latn", "ne-Deva-NP", "hi-Deva-IN"] {
        let answer = asked(
            &intl,
            &format!(
                r#"{{"amount": {{"currency": "NPR", "value": {{"minor": 101}}}}, "locale": "{locale}"}}"#
            ),
        );
        assert!(!answer.is_draft, "{locale}");
        assert!(answer.rules.starts_with("cldr-48/"), "{locale}");
    }
}

#[test]
fn the_drafted_table_is_the_hand_kept_file() {
    use sha2::Digest;
    assert_eq!(DRAFTED.len(), 1);
    for source in DRAFTED {
        let text = std::fs::read(format!(
            "{}/drafts/rbnf/{}.txt",
            env!("CARGO_MANIFEST_DIR"),
            source.file
        ))
        .unwrap();
        let sha = sha2::Sha256::digest(&text)
            .iter()
            .fold(String::new(), |mut hex, b| {
                use std::fmt::Write as _;
                let _ = write!(hex, "{b:02x}");
                hex
            });
        assert_eq!(sha, source.sha256, "{}", source.file);
        assert!(parse(source.rules).is_ok(), "{}", source.file);
    }
}
