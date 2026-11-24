//! Festival descriptions (`03-design/festival-about.md`, SR-153): every
//! described festival renders whole in English and Nepali, each fact the
//! data names is worded in both, and every community and region a shipped
//! pack's meta names has a name.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "tests fail by panicking and index what they asked for"
)]

use std::collections::BTreeSet;

use teistro::festival::FestivalFacts;
use teistro::{Context, FestivalPack, PackRequest};

const TAGS: [&str; 2] = ["en-Latn", "ne-Deva-NP"];

fn described(pack: FestivalPack) -> teistro::PackListing {
    let sdk = Context::builder().build().unwrap();
    sdk.almanac()
        .festival_pack(&PackRequest { pack, about: true })
}

/// A context with the festival corpus loaded, as a consumer loads the
/// release's `teistro-festivals-<version>-<tag>.tbundle`: stage (a)'s
/// subjects are embedded, the rest are words this corpus carries
/// (`03-design/festival-about.md` §7).
fn with_corpus() -> Context {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../packs/festivals");
    let tree = teistro_intl::source::Tree::load(std::path::Path::new(root)).unwrap();
    let sdk = Context::builder().build().unwrap();
    for locale in tree.locales.values() {
        let bundle = teistro_intl::pack::build_bundle(locale).unwrap();
        sdk.intl().load_pack(&bundle).unwrap();
    }
    sdk
}

/// Every subject a shipped pack describes, each once, with the listing
/// that described it first.
fn every_described() -> Vec<teistro::FestivalAbout> {
    let mut seen = BTreeSet::new();
    FestivalPack::ALL
        .into_iter()
        .flat_map(|pack| described(pack).about)
        .filter(|about| seen.insert(about.key.clone()))
        .collect()
}

/// Whether `text` is written in Devanagari, as a Nepali form must be.
fn devanagari(text: &str) -> bool {
    text.chars().any(|c| ('\u{0900}'..='\u{097F}').contains(&c))
}

#[test]
fn a_listing_describes_only_when_asked() {
    let sdk = Context::builder().build().unwrap();
    let plain = sdk.almanac().festival_pack(&PackRequest {
        pack: FestivalPack::NepalPanchanga,
        about: false,
    });
    assert!(plain.about.is_empty() && plain.unwritten.is_empty());
    let listed = described(FestivalPack::NepalPanchanga);
    assert_eq!(
        listed.rules, plain.rules,
        "describing leaves the rows alone"
    );
    let keys: BTreeSet<&str> = listed.about.iter().map(|a| a.key.as_str()).collect();
    assert_eq!(keys.len(), listed.about.len(), "each subject once");
    assert!(
        listed
            .unwritten
            .iter()
            .all(|key| !keys.contains(key.as_str())),
        "a subject is described or unwritten, never both"
    );
    let every: BTreeSet<String> = FestivalFacts::all().iter().map(|f| f.key.clone()).collect();
    let reached: BTreeSet<String> = every_described().into_iter().map(|a| a.key).collect();
    assert_eq!(
        reached, every,
        "a shipped pack lists every subject the data describes"
    );
}

#[test]
fn every_subject_renders_whole_in_english_and_nepali() {
    let sdk = with_corpus();
    let engine = sdk.locale_engine();
    for about in &every_described() {
        assert!(!about.sources.is_empty(), "{} cites a source", about.key);
        let items = std::iter::once(&about.about)
            .chain(&about.observance)
            .chain(std::iter::once(&about.who));
        for item in items {
            for tag in TAGS {
                let rendered = engine.render_in(tag, &item.key, &item.params);
                assert!(
                    rendered.warnings.is_empty(),
                    "{tag} {}: {rendered:?}",
                    about.key
                );
                assert_eq!(
                    rendered.resolved_from.as_deref(),
                    Some(tag),
                    "{tag} {}",
                    about.key
                );
                assert!(!rendered.is_draft, "{tag} {} is final", about.key);
                assert!(!rendered.text.contains('{'), "{tag}: {}", rendered.text);
                assert!(!rendered.text.is_empty(), "{tag} {}", about.key);
            }
        }
        // The `about` form is the locale's own, not the name standing in.
        let key = format!("festival.{}", about.key);
        let en = engine.entity_from("en-Latn", &key).unwrap();
        let ne = engine.entity_from("ne-Deva-NP", &key).unwrap();
        let (en_about, ne_about) = (&en.forms["about"], &ne.forms["about"]);
        let sentences = en_about.matches(". ").count() + 1;
        assert!((2..=4).contains(&sentences), "{key}: {sentences} sentences");
        assert!(devanagari(ne_about) && !devanagari(en_about), "{key}");
    }
}

#[test]
fn every_act_is_worded_in_both_locales_and_listed_by_a_subject() {
    let sdk = Context::builder().build().unwrap();
    let engine = sdk.locale_engine();
    let listed: BTreeSet<&str> = FestivalFacts::all()
        .iter()
        .flat_map(|facts| facts.observance.iter().map(String::as_str))
        .collect();
    for act in &listed {
        let key = format!("observance.{act}");
        let en = engine
            .entity_from("en-Latn", &key)
            .unwrap_or_else(|| panic!("{key}"));
        let ne = engine
            .entity_from("ne-Deva-NP", &key)
            .unwrap_or_else(|| panic!("{key}"));
        assert!(devanagari(ne.name()) && !devanagari(en.name()), "{key}");
    }
    let tree = teistro_intl::source::Tree::load(&teistro_intl::sdk_root()).unwrap();
    for tag in TAGS {
        let entities = &tree.locales[tag].namespaces[teistro_intl::ENTITY_NAMESPACE];
        let unlisted: Vec<String> = entities
            .in_source_order()
            .filter_map(|(key, _)| key.strip_prefix("observance.").map(str::to_owned))
            .filter(|act| !listed.contains(act.as_str()))
            .collect();
        assert!(
            unlisted.is_empty(),
            "{tag} words acts no subject lists: {unlisted:?}"
        );
    }
}

#[test]
fn every_community_and_region_a_shipped_pack_names_has_a_name() {
    let sdk = Context::builder().build().unwrap();
    let engine = sdk.locale_engine();
    let mut keys = BTreeSet::new();
    for pack in FestivalPack::ALL {
        for row in pack.listing().rules {
            keys.extend(
                row.meta
                    .communities
                    .iter()
                    .map(|c| format!("community.{c}")),
            );
            keys.extend(
                row.meta
                    .regions
                    .iter()
                    .map(|r| format!("region.{}", r.replace('-', "_"))),
            );
        }
    }
    let engine = &*engine;
    let missing: Vec<String> = keys
        .iter()
        .flat_map(|key| {
            TAGS.iter().filter_map(move |tag| {
                let named = engine
                    .entity_from(tag, key)
                    .is_some_and(|e| devanagari(e.name()) == (*tag == "ne-Deva-NP"));
                (!named).then(|| format!("{tag} {key}"))
            })
        })
        .collect();
    assert!(missing.is_empty(), "{} unnamed: {missing:?}", missing.len());
}
