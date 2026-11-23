//! Patterns a message author could get wrong, rendered over the SDK's own
//! sources: an option the function does not know, an operand of the
//! wrong kind, a key the catalogue lacks, a pattern that does not parse.
//! Each renders what it can and says what it could not, rather than
//! failing the whole message or passing in silence.

#![allow(clippy::unwrap_used, reason = "a test fails by panicking")]

use teistro_intl::source::Tree;
use teistro_intl::{Intl, Value, params, sdk_root};

fn english() -> Intl {
    let tree = Tree::load(&sdk_root()).unwrap();
    let mut intl = Intl::from_tree(&tree).unwrap();
    intl.set_locale("en-Latn").unwrap();
    intl
}

/// The text `source` renders with `$v` as `value`, and its warnings.
fn rendered(intl: &Intl, source: &str, value: Value) -> (String, Vec<String>) {
    let rendered = intl.render_source(source, &params([("v", value)])).unwrap();
    (rendered.text, rendered.warnings)
}

fn warns(warnings: &[String], saying: &str) -> bool {
    warnings.iter().any(|warning| warning.contains(saying))
}

#[test]
fn a_duration_unit_it_does_not_know_is_reported() {
    let intl = english();
    let (_, warnings) = rendered(&intl, "{$v :duration unit=fortnight}", Value::Num(2.0));
    assert!(
        warns(&warnings, "`unit=fortnight` is not a duration unit"),
        "{warnings:?}"
    );
    let (text, warnings) = rendered(
        &intl,
        "{$v :duration unit=hour into=|fortnight,minute|}",
        Value::Num(1.5),
    );
    assert!(
        warns(&warnings, "`into=` names `fortnight`"),
        "{warnings:?}"
    );
    assert!(text.contains("90"), "the known unit alone: {text}");
}

/// A year and a month have no fixed length, so a length is never broken
/// into them nor rounded to them: each is reported and left out, and the
/// fixed units named still read.
#[test]
fn a_calendar_unit_is_not_a_length() {
    let intl = english();
    let (text, warnings) = rendered(
        &intl,
        "{$v :duration unit=day into=|year,week,day|}",
        Value::Int(400),
    );
    assert!(
        warns(&warnings, "`into=` names `year`, which has no fixed length"),
        "{warnings:?}"
    );
    assert_eq!(text, "57 weeks and 1 day");
    let (text, warnings) = rendered(
        &intl,
        "{$v :duration unit=hour into=|hour,minute| round=month}",
        Value::Num(1.5),
    );
    assert!(
        warns(&warnings, "`round=month` has no fixed length"),
        "{warnings:?}"
    );
    assert_eq!(text, "1 hour and 30 minutes");
    let (text, warnings) = rendered(&intl, "{$v :duration unit=month into=|day|}", Value::Int(3));
    assert!(
        warns(&warnings, "`unit=month` has no fixed length"),
        "{warnings:?}"
    );
    assert_eq!(text, "3 months", "a count of the unit, as without `into=`");
}

/// A span given as counts: a calendar reckons an age or the span between
/// two dates, and `:duration` names it, dropping a part that is zero.
#[test]
fn a_span_given_as_counts_names_each_unit_once() {
    let intl = english();
    let span = |years: i64, months: i64, days: i64| {
        let message = "{:duration year=$years month=$months day=$days list=unit}";
        let rendered = intl
            .render_source(
                message,
                &params([
                    ("years", Value::Int(years)),
                    ("months", Value::Int(months)),
                    ("days", Value::Int(days)),
                ]),
            )
            .unwrap();
        assert!(rendered.warnings.is_empty(), "{:?}", rendered.warnings);
        rendered.text
    };
    assert_eq!(span(25, 3, 2), "25 years, 3 months, 2 days");
    assert_eq!(span(1, 0, 1), "1 year, 1 day");
    assert_eq!(span(0, 1, 0), "1 month");
    assert_eq!(
        span(0, 0, 0),
        "0 days",
        "nothing reads in the shortest unit"
    );

    // The options may come in any order, and a literal counts too.
    let (text, warnings) = rendered(&intl, "{:duration day=$v week=2 year=1}", Value::Int(3));
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(text, "1 year, 2 weeks and 3 days");

    // What a span given as counts cannot read is reported.
    let (text, warnings) = rendered(&intl, "{:duration unit=day}", Value::Int(3));
    assert!(warns(&warnings, "`unit=` reads an operand"), "{warnings:?}");
    assert!(warns(&warnings, "names none"), "{warnings:?}");
    assert_eq!(text, "{:duration}");
    let (_, warnings) = rendered(&intl, "{:duration year=many}", Value::Int(3));
    assert!(
        warns(&warnings, "`year=many` is not a number"),
        "{warnings:?}"
    );
    let (text, warnings) = rendered(&intl, "{$v :duration unit=day year=2}", Value::Int(3));
    assert!(
        warns(
            &warnings,
            "`year=` counts a unit only when `:duration` has no operand"
        ),
        "{warnings:?}"
    );
    assert_eq!(text, "3 days");
}

#[test]
fn a_duration_is_broken_into_units_with_its_sign_on_the_first() {
    let intl = english();
    let (text, warnings) = rendered(
        &intl,
        "{$v :duration unit=hour into=|minute,hour|}",
        Value::Num(-1.5),
    );
    assert!(warnings.is_empty(), "{warnings:?}");
    assert!(text.contains('1') && text.contains("30"), "{text}");
    assert!(
        text.find('1') < text.find("30"),
        "longest first, whatever order they were named in: {text}"
    );
    assert!(!text.contains("−30") && !text.contains("-30"), "{text}");
    // Nothing at all is the shortest unit named, at nought.
    let (none, _) = rendered(
        &intl,
        "{$v :duration unit=hour into=|hour,minute|}",
        Value::Num(0.0),
    );
    assert!(none.contains('0') && none.contains("minute"), "{none}");
}

#[test]
fn an_operand_of_the_wrong_kind_falls_back_to_its_text() {
    let intl = english();
    for source in ["{$v :duration}", "{$v :number}", "{$v :entity}"] {
        let (text, _) = rendered(&intl, source, Value::List(Vec::new()));
        assert!(!text.is_empty(), "{source}: {text}");
    }
    let (_, warnings) = rendered(&intl, "{$v :entity}", Value::Num(3.0));
    assert!(
        warns(&warnings, "`:entity` needs a catalogue key"),
        "{warnings:?}"
    );
}

#[test]
fn an_entity_of_another_kind_or_no_kind_is_reported() {
    let intl = english();
    let (text, warnings) = rendered(
        &intl,
        "{$v :entity kind=rashi}",
        Value::Entity("graha.SUN".into()),
    );
    assert!(
        warns(&warnings, "entity `graha.SUN` is not a `rashi`"),
        "{warnings:?}"
    );
    assert_ne!(text, "");
    let (_, warnings) = rendered(&intl, "{$v :entity}", Value::Entity("graha.VULCAN".into()));
    assert!(
        warns(&warnings, "entity `graha.VULCAN` is not a catalogue key"),
        "{warnings:?}"
    );
    // A rule's key names a consumer's pack, which no table holds.
    let (_, warnings) = rendered(&intl, "{$v :entity}", Value::Entity("rule.MY_RULE".into()));
    assert!(!warns(&warnings, "not a catalogue key"), "{warnings:?}");
}

#[test]
fn a_number_without_grouping_is_written_whole() {
    let intl = english();
    let (grouped, _) = rendered(&intl, "{$v :number}", Value::Int(1_234_567));
    let (plain, _) = rendered(
        &intl,
        "{$v :number useGrouping=never}",
        Value::Int(1_234_567),
    );
    assert_eq!((grouped.as_str(), plain.as_str()), ("1,234,567", "1234567"));
}

#[test]
fn a_pattern_that_does_not_parse_is_refused() {
    let intl = english();
    assert!(intl.render_source("{$v :number", &params([])).is_err());
    assert!(intl.render_source("{{unclosed", &params([])).is_err());
}

/// An engine over a lone `en-Latn` that declares no messages, no digit
/// grouping and no `or` list: every pattern falls back to its built-in
/// default.
///
/// Each call writes a directory of its own: two tests asking at once
/// removed and rewrote one directory under each other, and a write then
/// failed with `EINVAL`.
fn bare() -> Intl {
    static CALLS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let call = CALLS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("intl-patterns")
        .join(format!("bare-{}-{call}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let dir = root.join("en-Latn");
    std::fs::create_dir_all(&dir).unwrap();
    let meta = std::fs::read_to_string(sdk_root().join("en-Latn").join("_meta.json")).unwrap();
    let mut meta: serde_json::Value = serde_json::from_str(&meta).unwrap();
    if let Some(object) = meta.as_object_mut() {
        object.insert("grouping".into(), serde_json::json!([]));
        if let Some(lists) = object
            .get_mut("listPatterns")
            .and_then(serde_json::Value::as_object_mut)
        {
            lists.remove("or");
        }
    }
    std::fs::write(dir.join("_meta.json"), meta.to_string()).unwrap();
    let mut intl = Intl::from_tree(&Tree::load(&root).unwrap()).unwrap();
    intl.set_locale("en-Latn").unwrap();
    intl
}

#[test]
fn a_locale_is_chosen_only_from_those_loaded() {
    let mut intl = english();
    assert!(intl.set_locale("xx-Latn").is_err());
    assert!(intl.set_locale("ne-Deva-NP").is_ok());
}

#[test]
fn a_time_without_a_declared_pattern_takes_the_built_in_one() {
    use teistro_intl::ClockTime;
    let intl = bare();
    let afternoon = Value::from(ClockTime::new(13, 5, 9));
    let (numeric, _) = rendered(&intl, "{$v :time}", afternoon.clone());
    assert_eq!(numeric, "13:05");
    let (twelve, warnings) = rendered(
        &intl,
        "{$v :time hour12=true style=medium}",
        afternoon.clone(),
    );
    assert!(twelve.starts_with("1:05:09"), "{twelve}");
    assert!(!warnings.is_empty(), "the missing pattern is reported");
    let (twelve_short, _) = rendered(&intl, "{$v :time hour12=true}", afternoon.clone());
    assert!(
        twelve_short.starts_with("1:05") && !twelve_short.starts_with("1:05:09"),
        "{twelve_short}"
    );
    let (patterned, _) = rendered(
        &intl,
        "{$v :time pattern=sdk.calendar.time.mine}",
        afternoon,
    );
    assert_eq!(patterned, "13:05:09");
    let (_, warnings) = rendered(&intl, "{$v :time}", Value::Num(3.0));
    assert!(
        warns(&warnings, "`:time` needs a time of day"),
        "{warnings:?}"
    );
    let (_, warnings) = rendered(&intl, "{$v :ghati}", Value::Num(3.0));
    assert!(
        warns(&warnings, "`:ghati` needs a ghati-pala count"),
        "{warnings:?}"
    );
}

#[test]
fn a_locale_without_grouping_or_a_list_pattern_still_renders() {
    let intl = bare();
    let (grouped, _) = rendered(&intl, "{$v :number}", Value::Int(1_234_567));
    assert_eq!(grouped, "1,234,567", "groups of three by default");
    let items = Value::List(vec![Value::Str("a".into()), Value::Str("b".into())]);
    let (either, warnings) = rendered(&intl, "{$v :list type=or}", items);
    assert_eq!(either, "a, b");
    assert!(
        warns(&warnings, "en-Latn declares no `or` list pattern"),
        "{warnings:?}"
    );
    let (hours, _) = rendered(&intl, "{$v :duration unit=hour}", Value::Num(1.5));
    assert_eq!(hours, "1.5 hour", "the built-in count and unit");
}

#[test]
fn an_angle_is_written_to_the_precision_and_sign_asked() {
    let intl = english();
    for (source, value, want) in [
        ("{$v :dms signed=true}", -12.5, "-12°30′00″"),
        ("{$v :dms precision=deg}", 12.4, "12°"),
        ("{$v :dms precision=min}", 372.25, "12°15′"),
        ("{$v :dms}", -10.0, "350°00′00″"),
    ] {
        let (text, _) = rendered(&intl, source, Value::Num(value));
        assert_eq!(text, want, "{source} {value}");
    }
    let (zodiac, _) = rendered(&intl, "{$v :zodiac precision=sec}", Value::Num(45.5));
    assert!(zodiac.contains("15°30′00″"), "{zodiac}");
    for source in ["{$v :dms}", "{$v :zodiac}"] {
        let (text, _) = rendered(&intl, source, Value::Str("north".into()));
        assert!(!text.is_empty(), "{source}");
    }
}

/// SR-140: `en-NP` is English as Nepal writes it, `en-Latn-NP`: every word
/// English's, from `en-Latn`, and a number grouped by lakhs and crores in
/// Latin digits. `en` alone stays the base English, grouped by thousands.
#[test]
fn nepals_english_groups_by_lakhs_and_crores() {
    let tree = Tree::load(&sdk_root()).unwrap();
    let mut intl = Intl::from_tree(&tree).unwrap();
    assert_eq!(intl.set_locale("en-NP").unwrap(), "en-Latn-NP");
    for (value, expected) in [
        (Value::Int(271_700), "2,71,700"),
        (Value::Int(10_000_000), "1,00,00,000"),
        (Value::Int(999), "999"),
        (Value::Num(-1_234_567.5), "-12,34,567.5"),
    ] {
        let rendered = intl
            .render_source("{$v :number}", &params([("v", value)]))
            .unwrap();
        assert_eq!(rendered.text, expected);
    }
    let words = intl.render(
        "sdk.calendar.duration.year",
        &params([("n", Value::Int(2))]),
    );
    assert_eq!(words.text, "2 years");
    assert_eq!(words.resolved_from.as_deref(), Some("en-Latn"));
    assert!(words.warnings.is_empty(), "{:?}", words.warnings);
    assert!(words.is_fallback, "answered along the declared chain");

    assert_eq!(intl.set_locale("en").unwrap(), "en-Latn");
    let rendered = intl
        .render_source("{$v :number}", &params([("v", Value::Int(271_700))]))
        .unwrap();
    assert_eq!(rendered.text, "271,700");
}

/// SR-151 (C2332): Nepal's English names the twelve signs as its press
/// does in its rashifal columns, Mesh to Meen, where `en-Latn` names the
/// Western sign; an explicit form and a graha are English's, and every
/// sentence still comes from `en-Latn`.
#[test]
fn nepals_english_names_the_signs_as_its_press_does() {
    let tree = Tree::load(&sdk_root()).unwrap();
    let mut intl = Intl::from_tree(&tree).unwrap();
    let from = |intl: &Intl, rashi: &str| {
        intl.render(
            "sdk.reason.gochar.from",
            &params([
                ("from", Value::from("MOON")),
                ("rashi", Value::Entity(format!("rashi.{rashi}"))),
            ]),
        )
    };
    assert_eq!(
        from(&intl, "SAGITTARIUS").text,
        "Transits counted from the natal Moon, Sagittarius"
    );
    intl.set_locale("en-Latn-NP").unwrap();
    let dhanu = from(&intl, "SAGITTARIUS");
    assert_eq!(dhanu.text, "Transits counted from the natal Moon, Dhanu");
    assert_eq!(dhanu.resolved_from.as_deref(), Some("en-Latn"));
    assert!(dhanu.warnings.is_empty(), "{:?}", dhanu.warnings);
    let named: Vec<String> = [
        "ARIES",
        "TAURUS",
        "GEMINI",
        "CANCER",
        "LEO",
        "VIRGO",
        "LIBRA",
        "SCORPIO",
        "SAGITTARIUS",
        "CAPRICORN",
        "AQUARIUS",
        "PISCES",
    ]
    .iter()
    .map(|rashi| {
        intl.render_source(
            "{$v :entity kind=rashi}",
            &params([("v", Value::Entity(format!("rashi.{rashi}")))]),
        )
        .unwrap()
        .text
    })
    .collect();
    assert_eq!(
        named,
        [
            "Mesh",
            "Brish",
            "Mithun",
            "Karkat",
            "Singha",
            "Kanya",
            "Tula",
            "Brishchik",
            "Dhanu",
            "Makar",
            "Kumbha",
            "Meen"
        ]
    );
    // An explicit form is the record's, and a graha has no name of Nepal's.
    for (source, key, expected) in [
        ("{$v :entity kind=rashi form=iast}", "rashi.PISCES", "Mīna"),
        ("{$v :entity kind=graha}", "graha.SATURN", "Saturn"),
    ] {
        let rendered = intl
            .render_source(source, &params([("v", Value::Entity(key.into()))]))
            .unwrap();
        assert_eq!(rendered.text, expected, "{source}");
    }
}
