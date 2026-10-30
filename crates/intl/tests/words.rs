//! Numbers and amounts in words (`docs/03-design/amounts-in-words.md` §6):
//! every group boundary of every rule set spelt and read back by a reader
//! built from the tables alone, CLDR's own words for 0 to 99, the worked
//! examples, the rounding rules and every refusal.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "a test fails by panicking"
)]

use std::collections::BTreeMap;

use teistro_core::Status;
use teistro_intl::source::Tree;
use teistro_intl::words::rbnf::{Part, parse};
use teistro_intl::words::{InWords, InWordsRequest, SOURCES};
use teistro_intl::{Intl, sdk_root};

fn intl() -> Intl {
    Intl::from_tree(&Tree::load(&sdk_root()).unwrap()).unwrap()
}

fn asked(intl: &Intl, json: &str) -> Result<InWords, teistro_core::error::Error> {
    intl.in_words(&InWordsRequest::from_json(json).unwrap())
}

fn number(intl: &Intl, n: i64, locale: &str, grouping: &str) -> InWords {
    asked(
        intl,
        &format!(
            r#"{{"number": {n}, "locale": "{locale}", "grouping": "{grouping}", "digits": "LATN"}}"#
        ),
    )
    .unwrap_or_else(|e| panic!("{n} in {locale} {grouping}: {e}"))
}

/// A reader of a file's words, from its tables alone: a word below 100
/// is its rule's base, a scale word multiplies everything since the last
/// larger scale, figures are read as they are, and the minus word, `and`
/// and commas are not numbers.
struct Reader {
    small: BTreeMap<String, u128>,
    scale: BTreeMap<String, u128>,
    minus: String,
}

impl Reader {
    fn of(file: &str) -> Reader {
        let source = SOURCES.iter().find(|s| s.file == file).unwrap();
        let sets = parse(source.rules).unwrap();
        let (mut small, mut scale) = (BTreeMap::new(), BTreeMap::new());
        let mut minus = String::new();
        for set in &sets {
            if let Some(Part::Text(word)) = set.negative.as_ref().and_then(|n| n.first()) {
                word.trim().clone_into(&mut minus);
            }
            for rule in &set.rules {
                let words: Vec<&str> = rule
                    .body
                    .iter()
                    .filter_map(|part| match part {
                        Part::Text(text) => Some(text.as_str()),
                        _ => None,
                    })
                    .collect();
                for word in words.iter().flat_map(|w| w.split([' ', '-', ','])) {
                    if word.is_empty() {
                        continue;
                    }
                    let table = if rule.base < 100 {
                        &mut small
                    } else {
                        &mut scale
                    };
                    table
                        .entry(word.to_owned())
                        .or_insert(u128::from(rule.base));
                }
            }
        }
        Reader {
            small,
            scale,
            minus,
        }
    }

    fn read(&self, text: &str) -> i128 {
        let mut sign = 1;
        let mut items: Vec<(u128, u128)> = Vec::new(); // (value, scale)
        for token in text.split([' ', '-']).filter(|t| !t.is_empty()) {
            let token = token.trim_end_matches(',');
            if token == self.minus || token == "minus" {
                sign = -1;
            } else if token == "and" || token.is_empty() {
                // A joining word, not a number.
            } else if let Some(v) = self.small.get(token) {
                items.push((*v, 1));
            } else if let Some(&s) = self.scale.get(token) {
                // Everything since the last larger scale is counted in this one.
                let mut run = 0;
                while let Some(&(v, _)) = items.last().filter(|(_, at)| *at < s) {
                    run += v;
                    items.pop();
                }
                items.push((run * s, s));
            } else if token.chars().all(|c| c.is_ascii_digit() || c == ',') {
                items.push((token.replace(',', "").parse().unwrap(), 1));
            } else {
                panic!("`{token}` in `{text}` is no word the tables have");
            }
        }
        sign * i128::try_from(items.iter().map(|(v, _)| v).sum::<u128>()).unwrap()
    }
}

/// Every boundary of a rule set: each rule's base, the value before and
/// after it, and the round values every grouping turns on.
fn boundaries(file: &str) -> Vec<i64> {
    let source = SOURCES.iter().find(|s| s.file == file).unwrap();
    let mut out: Vec<i64> = vec![
        0,
        1,
        9,
        10,
        11,
        19,
        20,
        21,
        99,
        100,
        101,
        999,
        1000,
        1001,
        99_999,
        100_000,
        100_001,
        999_999,
        1_000_000,
        9_999_999,
        10_000_000,
        99_999_999,
        999_999_999,
        1_000_000_000,
        123_456_789,
        4_567_890_123,
    ];
    for set in parse(source.rules).unwrap() {
        for rule in set.rules {
            if let Ok(base) = i64::try_from(rule.base) {
                out.extend([
                    base - 1,
                    base,
                    base + 1,
                    base.saturating_mul(7).saturating_add(42),
                ]);
            }
        }
    }
    out.extend([i64::MAX, i64::MAX - 1]);
    out.retain(|n| *n >= 0);
    out.sort_unstable();
    out.dedup();
    out
}

#[test]
fn every_group_boundary_reads_back_to_its_number() {
    let intl = intl();
    let mut checked = 0;
    for (locale, grouping, file) in [
        ("ne-Deva-NP", "INDIAN", "ne"),
        ("hi-Deva-IN", "INDIAN", "hi"),
        ("en-Latn", "INDIAN", "en_IN"),
        ("en-Latn", "INTERNATIONAL", "en"),
    ] {
        let reader = Reader::of(file);
        for n in boundaries(file) {
            let words = number(&intl, n, locale, grouping);
            assert_eq!(
                reader.read(&words.text),
                i128::from(n),
                "{n} in {file}: {}",
                words.text
            );
            let negative = number(&intl, -n, locale, grouping);
            assert_eq!(
                reader.read(&negative.text),
                -i128::from(n),
                "-{n} in {file}: {}",
                negative.text
            );
            assert_eq!(words.rules, format!("cldr-48/{file}/spellout-cardinal"));
            checked += 1;
        }
    }
    assert!(checked > 150, "{checked} boundaries");
}

/// The words CLDR's own file gives a number, read from the XML directly:
/// the first `n: word;` rule naming it.
fn cldr_word(file: &str, n: u32) -> String {
    let xml = std::fs::read_to_string(format!(
        "{}/cldr/rbnf/{file}.xml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let prefix = format!("{n}: ");
    xml.lines()
        .filter_map(|line| line.strip_prefix(&prefix)?.strip_suffix(';'))
        .find(|word| !word.contains(['=', '<', '>', '[']))
        .unwrap()
        .to_owned()
}

#[test]
fn nepali_and_hindi_say_cldrs_own_word_for_every_number_below_a_hundred() {
    let intl = intl();
    for (locale, file) in [("ne-Deva-NP", "ne"), ("hi-Deva-IN", "hi")] {
        for n in 0..100 {
            assert_eq!(
                number(&intl, i64::from(n), locale, "INDIAN").text,
                cldr_word(file, n),
                "{n} in {file}"
            );
        }
    }
}

#[test]
fn the_scale_words_are_cldrs() {
    let intl = intl();
    let say = |n: i64, locale: &str, grouping: &str| number(&intl, n, locale, grouping).text;
    assert_eq!(say(100_000, "ne-Deva-NP", "INDIAN"), "एक लाख");
    assert_eq!(say(10_000_000, "ne-Deva-NP", "INDIAN"), "एक करोड");
    assert_eq!(say(1_000_000_000, "ne-Deva-NP", "INDIAN"), "एक अरब");
    assert_eq!(say(100_000_000_000, "ne-Deva-NP", "INDIAN"), "एक खरब");
    assert_eq!(say(10_000_000_000_000, "ne-Deva-NP", "INDIAN"), "एक शंख");
    assert_eq!(say(1_000, "ne-Deva-NP", "INDIAN"), "एक हजार");
    assert_eq!(say(1_000, "hi-Deva-IN", "INDIAN"), "एक हज़ार");
    assert_eq!(say(100_000_000_000, "hi-Deva-IN", "INDIAN"), "एक खरब");
    assert_eq!(say(10_000_000, "en-Latn", "INDIAN"), "one crore");
    // CLDR's Indian English counts crores to a lakh crore, then trillions (C804).
    assert_eq!(say(1_000_000_000, "en-Latn", "INDIAN"), "one hundred crore");
    assert_eq!(say(1_000_000_000_000, "en-Latn", "INDIAN"), "one trillion");
    assert_eq!(say(1_000_000, "en-Latn", "INTERNATIONAL"), "one million");
    // Past the last word, the rules write figures in the Indian grouping.
    let past = number(&intl, 1_000_000_000_000_000, "ne-Deva-NP", "INDIAN");
    assert_eq!(past.text, "1,00,00,00,00,00,00,000");
    let devanagari = asked(
        &intl,
        r#"{"number": 1000000000000000, "locale": "ne-Deva-NP"}"#,
    )
    .unwrap();
    assert_eq!(devanagari.text, "१,००,००,००,००,००,००,०००");
}

#[test]
fn the_cheque_guides_examples() {
    let intl = intl();
    // Vedantu, "How to write rupees and paise in words": 10,000.50.
    let vedantu = asked(
        &intl,
        r#"{"amount": {"currency": "NPR", "value": {"decimal": "10000.50"}, "only": true}, "grouping": "INDIAN"}"#,
    )
    .unwrap();
    assert_eq!(vedantu.text, "ten thousand rupees and fifty paisa only");
    assert_eq!(vedantu.figure, "10,000.50");
    let nepali = asked(
        &intl,
        r#"{"amount": {"currency": "NPR", "value": {"minor": 123456}, "only": true}, "locale": "ne-Deva-NP"}"#,
    )
    .unwrap();
    assert_eq!(nepali.text, "एक हजार दुई सय चौँतिस रुपैयाँ र छपन्न पैसा मात्र");
    assert_eq!(nepali.figure, "१,२३४.५६");
    let hindi = asked(
        &intl,
        r#"{"amount": {"currency": "INR", "value": {"minor": 123456}, "only": true}, "locale": "hi-Deva-IN"}"#,
    )
    .unwrap();
    assert_eq!(hindi.text, "एक हज़ार दो सौ चौंतीस रुपये और छप्पन पैसे मात्र");
    let lakh = asked(
        &intl,
        r#"{"amount": {"currency": "INR", "value": {"minor": 1250000000}}, "digits": "DEVA", "grouping": "INDIAN"}"#,
    )
    .unwrap();
    assert_eq!(lakh.text, "one crore twenty-five lakh rupees");
    assert_eq!(lakh.figure, "१,२५,००,०००.००");
    let verbose = asked(&intl, r#"{"number": 105, "style": "VERBOSE"}"#).unwrap();
    assert_eq!(verbose.text, "one hundred and five");
    assert_eq!(verbose.rules, "cldr-48/en/spellout-cardinal-verbose");
}

#[test]
fn an_amounts_shape_and_units_follow_its_counts() {
    let intl = intl();
    let say = |json: &str| asked(&intl, json).unwrap();
    let paisa = say(r#"{"amount": {"currency": "INR", "value": {"minor": 1}}}"#);
    assert_eq!(paisa.text, "one paisa");
    assert_eq!(
        say(r#"{"amount": {"currency": "INR", "value": {"minor": 2}}}"#).text,
        "two paise"
    );
    assert_eq!(
        say(r#"{"amount": {"currency": "INR", "value": {"minor": 100}}}"#).text,
        "one rupee"
    );
    assert_eq!(
        say(r#"{"amount": {"currency": "NPR", "value": {"minor": 0}}}"#).text,
        "zero rupees"
    );
    let whole = say(r#"{"amount": {"currency": "NPR", "value": {"minor": 7}, "minorDigits": 0}}"#);
    assert_eq!(whole.text, "seven rupees");
    assert_eq!(whole.figure, "7");
    let parts = whole.amount.unwrap();
    assert_eq!((parts.minor_words, parts.minor_unit), (None, None));
    assert_eq!(
        say(r#"{"amount": {"currency": "NPR", "value": {"decimal": "0.5"}}, "locale": "hi-Deva-IN"}"#).text,
        "पचास पैसे"
    );
}

#[test]
fn rounding_is_half_up_unless_asked() {
    let intl = intl();
    let minor = |decimal: &str, rounding: &str| {
        let json = format!(
            r#"{{"amount": {{"currency": "NPR", "value": {{"decimal": "{decimal}"}}, "rounding": "{rounding}"}}}}"#
        );
        let parts = asked(&intl, &json).unwrap().amount.unwrap();
        (parts.major * 100 + parts.minor, parts.rounded)
    };
    assert_eq!(minor("0.005", "HALF_UP"), (1, true));
    assert_eq!(minor("0.015", "HALF_UP"), (2, true));
    assert_eq!(minor("0.004", "HALF_UP"), (0, true));
    assert_eq!(minor("0.005", "HALF_EVEN"), (0, true));
    assert_eq!(minor("0.015", "HALF_EVEN"), (2, true));
    assert_eq!(minor("0.0051", "HALF_EVEN"), (1, true));
    assert_eq!(minor("0.025", "HALF_EVEN"), (2, true));
    assert_eq!(minor("1.10", "EXACT"), (110, false));
    assert_eq!(minor("1.1000", "EXACT"), (110, false));
    assert_eq!(minor(".5", "HALF_UP"), (50, false));
    assert_eq!(minor("7.", "HALF_UP"), (700, false));
    assert_eq!(minor("0.995", "HALF_UP"), (100, true));
    let exact = asked(
        &intl,
        r#"{"amount": {"currency": "NPR", "value": {"decimal": "1.005"}, "rounding": "EXACT"}}"#,
    )
    .unwrap_err();
    assert_eq!(exact.status, Status::InvalidArg);
    assert_eq!(exact.field(), Some("inWords.amount.value.decimal"));
}

#[test]
fn every_refusal_is_typed_and_named() {
    let intl = intl();
    for (json, status, field) in [
        (
            r#"{"number": 1, "locale": "sa-Deva"}"#,
            Status::Unsupported,
            "inWords.locale",
        ),
        (
            r#"{"number": 1, "locale": "sa-Latn"}"#,
            Status::Unsupported,
            "inWords.locale",
        ),
        (
            r#"{"number": 1, "locale": "xx-Latn"}"#,
            Status::Unsupported,
            "inWords.locale",
        ),
        (
            r#"{"number": 1, "locale": "ne-Deva-NP", "grouping": "INTERNATIONAL"}"#,
            Status::Unsupported,
            "inWords.grouping",
        ),
        (
            r#"{"number": 1, "locale": "hi-Deva-IN", "style": "VERBOSE"}"#,
            Status::Unsupported,
            "inWords.style",
        ),
        (r"{}", Status::InvalidArg, "inWords.number"),
        (
            r#"{"number": 1, "amount": {"currency": "NPR", "value": {"minor": 1}}}"#,
            Status::InvalidArg,
            "inWords.amount",
        ),
        (
            r#"{"amount": {"currency": "npr", "value": {"minor": 1}}}"#,
            Status::InvalidArg,
            "inWords.amount.currency",
        ),
        (
            r#"{"amount": {"currency": "USD", "value": {"minor": 1}}}"#,
            Status::Unsupported,
            "inWords.amount.currency",
        ),
        (
            r#"{"amount": {"currency": "NPR", "value": {"minor": 1}, "minorDigits": 5}}"#,
            Status::InvalidArg,
            "inWords.amount.minorDigits",
        ),
        (
            r#"{"amount": {"currency": "NPR", "value": {"decimal": "-1"}}}"#,
            Status::InvalidArg,
            "inWords.amount.value.decimal",
        ),
        (
            r#"{"amount": {"currency": "NPR", "value": {"decimal": "1e3"}}}"#,
            Status::InvalidArg,
            "inWords.amount.value.decimal",
        ),
        (
            r#"{"amount": {"currency": "NPR", "value": {"decimal": "."}}}"#,
            Status::InvalidArg,
            "inWords.amount.value.decimal",
        ),
        (
            r#"{"amount": {"currency": "NPR", "value": {"decimal": "1.2.3"}}}"#,
            Status::InvalidArg,
            "inWords.amount.value.decimal",
        ),
        (
            r#"{"amount": {"currency": "NPR", "value": {"decimal": "99999999999999999999"}}}"#,
            Status::InvalidArg,
            "inWords.amount.value.decimal",
        ),
        (
            r#"{"amount": {"currency": "NPR", "value": {"decimal": "184467440737095516.155"}}}"#,
            Status::InvalidArg,
            "inWords.amount.value.decimal",
        ),
    ] {
        let refused = asked(&intl, json).unwrap_err();
        assert_eq!(
            (refused.status, refused.field()),
            (status, Some(field)),
            "{json}: {refused}"
        );
    }
    let sanskrit = asked(&intl, r#"{"number": 1, "locale": "sa-Deva"}"#).unwrap_err();
    assert!(sanskrit.message.contains("unreviewed"), "{sanskrit}");
    // A negative amount cannot be written: its count is unsigned.
    assert!(
        InWordsRequest::from_json(r#"{"amount": {"currency": "NPR", "value": {"minor": -1}}}"#)
            .is_err()
    );
}

#[test]
fn a_currency_a_pack_names_is_spelt_and_a_fallback_unit_is_refused() {
    let mut intl = intl();
    // Hindi has no `sdk.amount.USD`, and English's would be the wrong language.
    intl.set_override(
        "en-Latn",
        "sdk.amount.USD.major",
        ".input {$count :integer} .match $count one {{dollar}} * {{dollars}}",
    )
    .unwrap();
    intl.set_override(
        "en-Latn",
        "sdk.amount.USD.minor",
        ".input {$count :integer} .match $count one {{cent}} * {{cents}}",
    )
    .unwrap();
    let dollars = asked(
        &intl,
        r#"{"amount": {"currency": "USD", "value": {"minor": 101}}}"#,
    )
    .unwrap();
    assert_eq!(dollars.text, "one dollar and one cent");
    let hindi = asked(
        &intl,
        r#"{"amount": {"currency": "USD", "value": {"minor": 101}}, "locale": "hi-Deva-IN"}"#,
    )
    .unwrap_err();
    assert_eq!(hindi.status, Status::Unsupported);
    assert!(
        hindi
            .hint()
            .unwrap_or_default()
            .contains("sdk.amount.USD.major"),
        "{hindi}"
    );
}

#[test]
fn the_tables_are_the_pinned_files() {
    use sha2::Digest;
    for source in SOURCES {
        let xml = std::fs::read(format!(
            "{}/cldr/rbnf/{}.xml",
            env!("CARGO_MANIFEST_DIR"),
            source.file
        ))
        .unwrap();
        let sha = sha2::Sha256::digest(&xml)
            .iter()
            .fold(String::new(), |mut hex, b| {
                use std::fmt::Write as _;
                let _ = write!(hex, "{b:02x}");
                hex
            });
        assert_eq!(sha, source.sha256, "{}", source.file);
        assert!(parse(source.rules).is_ok(), "{}", source.file);
    }
    assert_eq!(SOURCES.len(), 4);
}
