//! The dossier over a document carrying every section it writes
//! (`docs/03-design/dossier.md`): the fixture is the corpus's first birth
//! as the SDK founds it, written and held by `cargo xtask dossier`, since
//! this crate cannot found a chart with its strengths.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro_core::error::Error;
use teistro_serial::Document;
use teistro_serial::canonical::from_hash_form;
use teistro_serial::dossier::{
    Column, DOSSIER_FORMAT, DossierOptions, DossierPreset, DossierSection, View, built_in, column,
    dossier, dossier_legend,
};

/// The clock the birth was recorded on, Nepal's.
const CLOCK: i32 = 20_700;

fn chart() -> Document {
    from_hash_form(include_str!("dossier-chart.json")).expect("the fixture reads back")
}

fn on_clock() -> DossierOptions {
    DossierOptions {
        utc_offset_seconds: Some(CLOCK),
        ..DossierOptions::default()
    }
}

fn asking(sections: &[&str]) -> DossierOptions {
    DossierOptions {
        sections: Some(sections.iter().map(|one| (*one).to_owned()).collect()),
        ..on_clock()
    }
}

/// The banners in order.
fn banners(text: &str) -> Vec<&str> {
    text.lines()
        .filter_map(|line| line.strip_prefix("== ")?.strip_suffix(" =="))
        .collect()
}

/// A section's rows, split into fields: the lines under its header.
fn rows<'t>(text: &'t str, key: &str) -> Vec<Vec<&'t str>> {
    let banner = format!("== {key} ==");
    text.lines()
        .skip_while(|line| *line != banner)
        .skip(2)
        .take_while(|line| !line.starts_with("== "))
        .map(|line| line.split('|').collect())
        .collect()
}

#[test]
fn every_section_is_written_in_order_each_row_a_field_a_column() {
    let document = chart();
    let text = dossier(&document, &on_clock(), &[]).unwrap();
    assert!(text.starts_with(&format!(
        "# teistro dossier {DOSSIER_FORMAT}\n== LEGEND ==\n"
    )));
    let every: Vec<&str> = built_in()
        .iter()
        .map(|one| one.key())
        .filter(|key| *key != "NOW")
        .collect();
    let mut expected = vec!["LEGEND"];
    expected.extend(&every);
    assert_eq!(banners(&text), expected, "date-free, so no NOW");
    for section in built_in().iter().filter(|one| one.key() != "NOW") {
        let key = section.key();
        let names: Vec<&str> = section.columns().iter().map(|one| one.name).collect();
        let header = format!("# {}", names.join("|"));
        let at = text.find(&format!("== {key} ==\n{header}\n"));
        assert!(at.is_some(), "{key}: its header under its banner");
        assert!(
            text.contains(&format!("\n# {key}\n")),
            "{key}: in the legend"
        );
        for one in section.columns() {
            assert!(
                text.contains(&format!("#   {}: {}\n", one.name, one.meaning)),
                "{key}.{}: documented",
                one.name
            );
        }
        let written = rows(&text, key);
        assert!(!written.is_empty(), "{key}: rows");
        for row in written {
            assert_eq!(row.len(), names.len(), "{key}: {row:?}");
            assert!(row.iter().all(|field| !field.is_empty()), "{key}: {row:?}");
        }
    }
    assert_eq!(
        dossier(&document, &on_clock(), &[]).unwrap(),
        text,
        "the same bytes"
    );
    // The clock is read, and UTC when none is named.
    assert!(
        rows(&text, "CHART")
            .iter()
            .any(|row| row[1].contains("+05:45"))
    );
    let utc = dossier(&document, &DossierOptions::default(), &[]).unwrap();
    assert!(
        rows(&utc, "CHART")
            .iter()
            .any(|row| row[1].contains("+00:00"))
    );
}

#[test]
fn a_dated_dossier_says_now_and_follows_the_running_branch() {
    let document = chart();
    let date_free = dossier(&document, &asking(&["DASHAS"]), &[]).unwrap();
    let at = 2_460_676.5;
    let dated = dossier(
        &document,
        &DossierOptions {
            at: Some(at),
            ..asking(&["NOW", "DASHAS"])
        },
        &[],
    )
    .unwrap();
    assert_eq!(banners(&dated), ["LEGEND", "NOW", "DASHAS"]);
    let now = rows(&dated, "NOW");
    assert_eq!(now.len(), 1);
    assert_eq!(now[0][0], "2025-01-01 05:45");
    let age = (at - document.foundation.instant.get()) / 365.25;
    assert_eq!(now[0][2], format!("{age:.2}"));
    let periods = rows(&dated, "DASHAS");
    let running: Vec<&Vec<&str>> = periods.iter().filter(|row| row[6] == "*").collect();
    assert!(!running.is_empty(), "a period runs");
    let mut depths: Vec<usize> = running
        .iter()
        .map(|row| row[1].split('/').count())
        .collect();
    depths.sort_unstable();
    depths.dedup();
    assert_eq!(
        depths,
        (1..=depths.len()).collect::<Vec<_>>(),
        "one a level"
    );
    assert!(
        rows(&dated, "DASHAS").len() < rows(&date_free, "DASHAS").len(),
        "the branch only"
    );
    assert!(rows(&date_free, "DASHAS").iter().all(|row| row[6] == "-"));
}

#[test]
fn a_preset_chooses_and_a_depth_cuts() {
    let document = chart();
    let family = dossier(
        &document,
        &DossierOptions {
            preset: DossierPreset::Family,
            ..on_clock()
        },
        &[],
    )
    .unwrap();
    assert_eq!(
        banners(&family),
        ["LEGEND", "CHART", "GRAHAS", "DASHAS", "PANCHANGA"]
    );
    let shallow = dossier(
        &document,
        &DossierOptions {
            dasha_depth: Some(1),
            ..asking(&["DASHAS"])
        },
        &[],
    )
    .unwrap();
    assert!(
        rows(&shallow, "DASHAS")
            .iter()
            .all(|row| !row[1].contains('/'))
    );
    // Named out of order and twice, written once in the dossier's order.
    let named = dossier(&document, &asking(&["SHADBALA", "GRAHAS", "SHADBALA"]), &[]).unwrap();
    assert_eq!(banners(&named), ["LEGEND", "GRAHAS", "SHADBALA"]);
}

#[test]
fn what_the_document_lacks_or_the_options_break_is_refused_by_its_place() {
    let document = chart();
    let bare = Document::of(document.foundation.clone());
    let full = dossier(&bare, &on_clock(), &[]).unwrap();
    assert_eq!(
        banners(&full),
        ["LEGEND", "CHART", "GRAHAS"],
        "what it carries"
    );
    let lacking = dossier(&bare, &asking(&["GRAHAS", "SHADBALA"]), &[]).unwrap_err();
    assert_eq!(lacking.field(), Some("sections[1]"));
    assert!(lacking.hint().is_some_and(|hint| hint.contains("shadbala")));
    let undated = dossier(&document, &asking(&["NOW"]), &[]).unwrap_err();
    assert_eq!(undated.field(), Some("sections[0]"));
    let unknown = dossier(&document, &asking(&["PLANETS"]), &[]).unwrap_err();
    assert_eq!(unknown.field(), Some("sections[0]"));
    let refused =
        |options: DossierOptions| -> Error { dossier(&document, &options, &[]).unwrap_err() };
    assert_eq!(
        refused(DossierOptions {
            dasha_depth: Some(0),
            ..on_clock()
        })
        .field(),
        Some("dashaDepth")
    );
    assert_eq!(
        refused(DossierOptions {
            utc_offset_seconds: Some(-86_400),
            ..on_clock()
        })
        .field(),
        Some("utcOffsetSeconds")
    );
    assert_eq!(
        refused(DossierOptions {
            at: Some(f64::NAN),
            ..on_clock()
        })
        .field(),
        Some("at")
    );
    // The record's reader names the field the caller wrote.
    let read = |text: &str| DossierOptions::from_json(text, &["OWN"]).unwrap_err();
    assert_eq!(
        read(r#"{"dashaDepth": 0}"#).field(),
        Some("dossier.dashaDepth")
    );
    assert_eq!(
        read(r#"{"sections": ["GRAHAS", "X"]}"#).field(),
        Some("dossier.sections[1]")
    );
    assert!(
        read("[]")
            .field()
            .is_some_and(|field| field.starts_with("dossier"))
    );
    assert!(DossierOptions::from_json(r#"{"sections": ["OWN"]}"#, &["OWN"]).is_ok());
}

/// A consumer's section: written after the built-in ones, its columns and
/// notes in the legend as theirs are.
struct Own;

impl DossierSection for Own {
    fn key(&self) -> &'static str {
        "OWN"
    }
    fn columns(&self) -> &[Column] {
        const COLUMNS: [Column; 2] = [column("what", "a word"), column("how", "another")];
        &COLUMNS
    }
    fn notes(&self) -> &[&str] {
        &["A consumer's note."]
    }
    fn asked_by(&self) -> &'static str {
        "the consumer's own call"
    }
    fn rows(&self, _: &Document, view: &View) -> Option<Vec<Vec<String>>> {
        view.at?;
        Some(vec![vec![String::from("a|b"), String::from("c\nd")]])
    }
}

#[test]
fn a_consumer_section_is_written_and_documented_as_a_built_in_one() {
    let document = chart();
    let dated = DossierOptions {
        at: Some(2_460_676.5),
        ..asking(&["OWN", "GRAHAS"])
    };
    let text = dossier(&document, &dated, &[&Own]).unwrap();
    assert_eq!(banners(&text), ["LEGEND", "GRAHAS", "OWN"]);
    assert!(text.contains("# OWN\n#   what: a word\n#   how: another\n#   A consumer's note.\n"));
    assert_eq!(
        rows(&text, "OWN"),
        [["a b", "c d"]],
        "no separator, no line break"
    );
    // Carried only when it has rows; refused by its hint when named.
    let full = dossier(&document, &on_clock(), &[&Own]).unwrap();
    assert!(!banners(&full).contains(&"OWN"));
    let undated = dossier(&document, &asking(&["OWN"]), &[&Own]).unwrap_err();
    assert!(
        undated
            .hint()
            .is_some_and(|hint| hint.contains("consumer's own"))
    );
    // One legend for a group's dossiers.
    let legend = dossier_legend(&[&Own]);
    assert!(legend.starts_with("== LEGEND ==\n") && legend.contains("# OWN\n"));
}

#[test]
fn every_built_in_names_what_carries_it() {
    let document = chart();
    let bare = Document::of(document.foundation.clone());
    for (at, section) in built_in().iter().enumerate() {
        assert!(!section.asked_by().is_empty(), "{}", section.key());
        let dated = DossierOptions {
            at: None,
            ..asking(&[section.key()])
        };
        if let Err(refused) = dossier(&bare, &dated, &[]) {
            assert_eq!(refused.field(), Some("sections[0]"), "{at}");
            assert_eq!(
                refused.hint(),
                Some(format!("ask with {}", section.asked_by()).as_str()),
                "{}",
                section.key()
            );
        }
    }
}

#[test]
fn the_document_is_built_section_by_section_as_the_fixture_reads() {
    let whole = chart();
    let mut built = Document::of(whole.foundation.clone())
        .with_panchanga(whole.panchanga.clone().unwrap())
        .with_state(whole.state.clone().unwrap())
        .with_aspects(whole.aspects.clone().unwrap())
        .with_points(whole.points.clone().unwrap())
        .with_houses(whole.houses.clone().unwrap())
        .with_ashtakavarga(whole.ashtakavarga.clone().unwrap())
        .with_vimshopaka(whole.vimshopaka.clone().unwrap())
        .with_shadbala(whole.shadbala.clone().unwrap())
        .with_bhava_bala(whole.bhava_bala.clone().unwrap())
        .with_jaimini(whole.jaimini.clone().unwrap())
        .with_avakahada(whole.avakahada.clone().unwrap());
    for varga in &whole.vargas {
        built = built.with_varga(varga.clone());
    }
    for dasha in &whole.dashas {
        built = built.with_dasha(dasha.clone());
    }
    assert_eq!(built, whole);
    for key in [
        "panchanga",
        "state",
        "ashtakavarga",
        "vimshopaka",
        "shadbala",
        "bhava_bala",
        "jaimini",
        "avakahada",
    ] {
        assert!(whole.sections().contains(&key), "{key}");
    }
}

#[test]
fn a_tropical_chart_says_so() {
    let mut document = chart();
    document.foundation.zodiac.ayanamsha = None;
    let text = dossier(&document, &asking(&["CHART"]), &[]).unwrap();
    assert!(
        rows(&text, "CHART")
            .iter()
            .any(|row| row.contains(&"TROPICAL"))
    );
}
