//! A locale tree as a translator might break it: each malformed directory,
//! meta file, namespace file and record is refused with the file it is in
//! and what is wrong, and the names the tree's files and keys take are
//! held to their spelling at each edge.

#![allow(clippy::unwrap_used, reason = "a test fails by panicking")]

use std::path::{Path, PathBuf};

use teistro_intl::sdk_root;
use teistro_intl::source::{
    Entity, META_FILE, Tree, is_form_name, is_key_segment, is_locale_tag, is_member_key,
    is_namespace_name,
};
use teistro_intl::validate::validate;

/// A fresh directory of the test's own.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("intl-sources")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A tree holding one locale, `tag`, with the base locale's meta file
/// (its `locale` set to `locale`) and each of `files`.
fn tree(name: &str, tag: &str, locale: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = scratch(name);
    let dir = root.join(tag);
    std::fs::create_dir_all(&dir).unwrap();
    let meta = std::fs::read_to_string(sdk_root().join("en-Latn").join(META_FILE)).unwrap();
    std::fs::write(
        dir.join(META_FILE),
        meta.replace(
            "\"locale\": \"en-Latn\"",
            &format!("\"locale\": \"{locale}\""),
        ),
    )
    .unwrap();
    for (file, text) in files {
        std::fs::write(dir.join(file), text).unwrap();
    }
    root
}

/// The refusal of the tree `files` make, which must name `file` and say
/// `saying`.
fn refused(name: &str, files: &[(&str, &str)], file: &str, saying: &str) {
    let root = tree(name, "en-Latn", "en-Latn", files);
    let error = Tree::load(&root).unwrap_err();
    assert!(error.path.ends_with(file), "{name}: {error}");
    assert!(error.detail.contains(saying), "{name}: {error}");
}

#[test]
fn a_well_formed_tree_loads() {
    let root = tree(
        "fine",
        "en-Latn",
        "en-Latn",
        &[
            ("sdk.test.json", r#"{"greeting": {"hello": "Hello"}}"#),
            (
                "sdk.entity.json",
                r#"{"graha": {"SUN": {"name": "Sun", "glyph": "☉", "gender": "m"}}}"#,
            ),
            ("_overrides.json", "not json, and a tool's file"),
            ("notes.txt", "not a namespace"),
        ],
    );
    let tree = Tree::load(&root).unwrap();
    let base = tree.base().unwrap();
    assert_eq!(base.tag, "en-Latn");
    assert_eq!(base.namespaces.len(), 2);
}

#[test]
fn a_root_without_locales_and_a_directory_without_a_tag_are_refused() {
    let empty = scratch("empty");
    let error = Tree::load(&empty).unwrap_err();
    assert_eq!(error.detail, "no locale directories");

    let root = tree("untagged", "english", "english", &[]);
    let error = Tree::load(&root).unwrap_err();
    assert!(error.path.ends_with("english"), "{error}");
    assert!(
        error.detail.contains("named by a tag with a script"),
        "{error}"
    );
}

#[test]
fn a_meta_file_missing_malformed_or_for_another_locale_is_refused() {
    let root = tree("no-meta", "en-Latn", "en-Latn", &[]);
    std::fs::remove_file(root.join("en-Latn").join(META_FILE)).unwrap();
    let error = Tree::load(&root).unwrap_err();
    assert!(error.path.ends_with(META_FILE), "{error}");

    let root = tree("bad-meta", "en-Latn", "en-Latn", &[]);
    std::fs::write(root.join("en-Latn").join(META_FILE), "{\"locale\": 3}").unwrap();
    let error = Tree::load(&root).unwrap_err();
    assert!(error.path.ends_with(META_FILE), "{error}");

    let root = tree("other-meta", "en-Latn", "ne-Deva-NP", &[]);
    let error = Tree::load(&root).unwrap_err();
    assert_eq!(
        error.detail,
        "`locale` is ne-Deva-NP but the directory is en-Latn"
    );
}

#[test]
fn a_namespace_file_is_refused_for_its_name_its_json_and_its_shape() {
    refused(
        "namespace-name",
        &[("Sdk.Test.json", "{}")],
        "Sdk.Test.json",
        "lowercase dotted words",
    );
    refused(
        "namespace-json",
        &[("sdk.test.json", "{")],
        "sdk.test.json",
        "EOF",
    );
    refused(
        "namespace-array",
        &[("sdk.test.json", "[]")],
        "sdk.test.json",
        "must be an object of keys",
    );
    refused(
        "namespace-number",
        &[("sdk.test.json", r#"{"greeting": {"count": 3}}"#)],
        "sdk.test.json",
        "`greeting.count` must be a string message or an object",
    );
    refused(
        "namespace-segment",
        &[("sdk.test.json", r#"{"greeting": {"Hello-there": "Hi"}}"#)],
        "sdk.test.json",
        "key segment `Hello-there` under `greeting`",
    );
}

#[test]
fn the_entity_namespace_holds_records_with_forms_only() {
    refused(
        "entity-message",
        &[(
            "sdk.entity.json",
            r#"{"graha": {"SUN": {"name": "Sun"}, "note": "a message"}}"#,
        )],
        "sdk.entity.json",
        "`graha.note`: the entity namespace holds records, not messages",
    );
    refused(
        "entity-form",
        &[(
            "sdk.entity.json",
            r#"{"graha": {"SUN": {"name": "Sun", "Prose": "the Sun"}}}"#,
        )],
        "sdk.entity.json",
        "`graha.SUN.Prose`: a form name is a camelCase word",
    );
    refused(
        "entity-forms",
        &[(
            "sdk.entity.json",
            r#"{"graha": {"SUN": {"name": "Sun", "forms": "all"}}}"#,
        )],
        "sdk.entity.json",
        "a form name",
    );
}

#[test]
fn a_locale_tag_has_a_language_a_script_and_at_most_a_region() {
    for good in ["en-Latn", "ne-Deva-NP", "es-Latn-419", "haw-Latn"] {
        assert!(is_locale_tag(good), "{good}");
    }
    for bad in [
        "",
        "en",
        "e-Latn",
        "engl-Latn",
        "EN-Latn",
        "en-latn",
        "en-LATN",
        "en-Lat",
        "en-Latn-np",
        "en-Latn-N",
        "en-Latn-41",
        "en-Latn-4a9",
        "en-Latn-NP-x",
    ] {
        assert!(!is_locale_tag(bad), "{bad}");
    }
}

#[test]
fn names_and_keys_are_held_to_their_spelling() {
    assert!(is_namespace_name("sdk.entity") && is_namespace_name("sdk.v2"));
    for bad in [
        "",
        "sdk..entity",
        "sdk.Entity",
        "sdk.2nd",
        "sdk-entity",
        "sdk.",
    ] {
        assert!(!is_namespace_name(bad), "{bad}");
    }
    assert!(is_key_segment("avastha_baladi"), "a kind of two words");
    assert!(is_key_segment("greeting2") && is_key_segment("SUN_MOON"));
    for bad in ["", "hello_there", "SUN__MOON", "SUN_", "_sun", "9lives"] {
        assert!(!is_key_segment(bad), "{bad}");
    }
    assert!(is_form_name("phalaProse"));
    for bad in ["", "forms", "Name", "1st", "short-name"] {
        assert!(!is_form_name(bad), "{bad}");
    }
    assert!(!is_member_key("") && !is_member_key("SUN-MOON"));
}

#[test]
fn an_entity_keeps_what_an_overlay_leaves_out() {
    let mut sun = Entity::default();
    sun.forms.insert(String::from("name"), String::from("Sun"));
    assert_eq!(sun.form("glyph"), None, "no glyph is no form");
    let mut gendered = sun.clone();
    gendered.gender = Some(String::from("m"));
    assert!(gendered.kept_under(&sun), "a gender the overlay lacks");
    assert!(!sun.kept_under(&gendered));
    let mut glyphed = sun.clone();
    glyphed.glyph = Some(String::from("☉"));
    assert!(glyphed.kept_under(&sun), "a glyph the overlay lacks");
    assert!(!glyphed.kept_under(&glyphed));
}

/// A tree of the base locale and Nepali, each with its own files.
fn two_locales(name: &str, english: &[(&str, &str)], nepali: &[(&str, &str)]) -> PathBuf {
    let root = tree(name, "en-Latn", "en-Latn", english);
    let dir = root.join("ne-Deva-NP");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::copy(
        sdk_root().join("ne-Deva-NP").join(META_FILE),
        dir.join(META_FILE),
    )
    .unwrap();
    for (file, text) in nepali {
        std::fs::write(dir.join(file), text).unwrap();
    }
    root
}

fn findings(root: &Path) -> Vec<String> {
    let report = validate(&Tree::load(root).unwrap());
    assert!(
        report
            .markdown()
            .contains("| severity | locale | key | finding |")
    );
    report
        .diagnostics
        .iter()
        .map(|d| format!("{} {}: {}", d.locale, d.key, d.message))
        .collect()
}

fn found(findings: &[String], saying: &str) -> bool {
    findings.iter().any(|finding| finding.contains(saying))
}

#[test]
fn a_tree_without_its_base_locale_is_one_finding() {
    let root = scratch("no-base");
    let dir = root.join("ne-Deva-NP");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::copy(
        sdk_root().join("ne-Deva-NP").join(META_FILE),
        dir.join(META_FILE),
    )
    .unwrap();
    let report = validate(&Tree::load(&root).unwrap());
    assert!(!report.passed());
    assert_eq!(report.diagnostics.len(), 1);
    assert!(found(&findings(&root), "the base locale is missing"));
}

#[test]
fn an_entity_record_is_held_to_its_name_gender_key_and_base() {
    let root = two_locales(
        "entities",
        &[(
            "sdk.entity.json",
            r#"{"graha": {
                "SUN": {"name": "Sun", "prose": "the Sun", "gender": "m"},
                "MOON": {"prose": "the Moon"},
                "MARS": {"name": "Mars", "gender": "x"},
                "VULCAN": {"name": "Vulcan"}
            }}"#,
        )],
        &[(
            "sdk.entity.json",
            r#"{"graha": {"SUN": {"name": "सूर्य", "gender": "f"}, "MOON": {"glyph": "☽"}}}"#,
        )],
    );
    let findings = findings(&root);
    for saying in [
        "en-Latn sdk.entity.graha.MOON: an entity needs a non-empty `name`",
        "en-Latn sdk.entity.graha.MARS: gender `x` is not a value of the `gender` context",
        "en-Latn sdk.entity.graha.VULCAN: not a catalogue key",
        "ne-Deva-NP sdk.entity.graha.SUN: no `prose` form; `name` will stand in",
        "ne-Deva-NP sdk.entity.graha.SUN: gender differs from the base locale",
    ] {
        assert!(found(&findings, saying), "{saying}: {findings:#?}");
    }
}

#[test]
fn a_message_is_held_to_its_links_selectors_and_base_parameters() {
    let root = two_locales(
        "messages",
        &[
            (
                "sdk.entity.json",
                r#"{"graha": {"SUN": {"name": "Sun", "prose": "the Sun", "gender": "m"}}}"#,
            ),
            (
                "sdk.test.json",
                r#"{
                "link": "{|sdk.test.nothing| :msg}",
                "named": "{|graha.VULCAN| :entity} and {|rashi.ARIES| :entity}",
                "rank": ".input {$n :number select=ordinal} .match $n many {{many}} * {{other}}",
                "who": ".input {$g :entity} .match $g SUN {{sun}} m {{he}} zz {{zz}} * {{other}}",
                "count": "{$n :number} items"
            }"#,
            ),
        ],
        &[("sdk.test.json", r#"{"count": "{$n :entity} {$extra} वस्तु"}"#)],
    );
    let findings = findings(&root);
    for saying in [
        "`:msg` names `sdk.test.nothing`, which the base locale does not have",
        "`:entity` names `graha.VULCAN`, which is not a catalogue key",
        "`:entity` names `rashi.ARIES`, which the entity namespace does not describe yet",
        "`many` is never a ordinal category in en-Latn",
        "`zz` is neither an entity key nor a gender",
        "ne-Deva-NP sdk.test.count: uses `$n` as",
        "ne-Deva-NP sdk.test.count: uses `$extra`, which the base message does not declare",
    ] {
        assert!(found(&findings, saying), "{saying}: {findings:#?}");
    }
}
