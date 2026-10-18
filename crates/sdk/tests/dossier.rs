//! The dossier over a founded chart: what it writes, that it writes the
//! same bytes twice, and what it refuses (`docs/03-design/dossier.md`).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::{DashaSystem, Varga};
use teistro::dossier::{DossierOptions, DossierPreset, dossier};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{ChartRecords, ChartRequest, Context, Document, Ephemeris, UtcOffset};

const BIRTH: f64 = 2_447_995.489_583_333_5;
const CLOCK: i32 = 20_700;

fn chart(request: &ChartRequest) -> Document {
    let sdk = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("a built context");
    sdk.chart()
        .reading(JulianDay::<Utc>::literal(BIRTH), request)
        .unwrap()
        .value
}

fn kathmandu() -> ChartRequest {
    let place = Place::new(
        Latitude::try_new(27.7172).unwrap(),
        Longitude::try_new(85.324).unwrap(),
        Altitude::try_new(1400.0).unwrap(),
    );
    ChartRequest::at(place, UtcOffset::try_from_seconds(CLOCK).unwrap())
}

fn full() -> Document {
    chart(
        &kathmandu()
            .with_panchanga()
            .with_state()
            .with_vargas([Varga::D9, Varga::D10])
            .with_dashas([DashaSystem::Vimshottari]),
    )
}

fn on_clock() -> DossierOptions {
    DossierOptions {
        utc_offset_seconds: Some(CLOCK),
        ..DossierOptions::default()
    }
}

/// The banners in order.
fn banners(text: &str) -> Vec<&str> {
    text.lines()
        .filter_map(|line| line.strip_prefix("== ")?.strip_suffix(" =="))
        .collect()
}

#[test]
fn a_dossier_writes_every_section_the_chart_carries_and_the_same_bytes_twice() {
    let document = full();
    let text = dossier(&document, &on_clock(), &[]).unwrap();
    assert!(text.starts_with("# teistro dossier 1\n== LEGEND ==\n"));
    assert_eq!(
        banners(&text),
        ["LEGEND", "CHART", "GRAHAS", "VARGAS", "DASHAS", "PANCHANGA"]
    );
    assert_eq!(text, dossier(&document, &on_clock(), &[]).unwrap());
    // The instant on the chart's clock; the lagna, the nine and their state.
    assert!(
        text.contains("instant|1990-04-14 05:30 +05:45 2447995.489583\n"),
        "{text}"
    );
    let grahas: Vec<&str> = text
        .lines()
        .skip_while(|line| *line != "== GRAHAS ==")
        .skip(2)
        .take_while(|line| !line.starts_with("=="))
        .collect();
    assert_eq!(grahas.len(), 10, "the lagna and the nine");
    assert!(grahas[0].starts_with("LAGNA|"));
    assert!(grahas[1].starts_with("SUN|"));
    assert!(
        grahas.iter().skip(1).all(|row| !row.ends_with("|-|-|-|-")),
        "the state read"
    );
    // Every row has a field a column.
    let mut columns = 0;
    for line in text.lines().skip_while(|line| *line != "== CHART ==") {
        if line.starts_with("== ") {
            columns = 0;
        } else if let Some(header) = line.strip_prefix("# ") {
            columns = header.split('|').count();
        } else {
            assert_eq!(line.split('|').count(), columns, "{line}");
        }
    }
    // A date-free timeline: no period marked.
    assert!(!text.contains("|*\n"));
}

#[test]
fn every_section_is_written_when_the_chart_carries_it() {
    let document = chart(
        &kathmandu()
            .with_panchanga()
            .with_state()
            .with_aspects()
            .with_points()
            .with_houses()
            .with_ashtakavarga()
            .with_vimshopaka()
            .with_shadbala()
            .with_bhava_bala()
            .with_jaimini()
            .with_avakahada()
            .with_vargas([Varga::D9])
            .with_dashas([DashaSystem::Vimshottari]),
    );
    let text = dossier(
        &document,
        &DossierOptions {
            dasha_depth: Some(2),
            ..on_clock()
        },
        &[],
    )
    .unwrap();
    std::fs::write(
        std::env::temp_dir().join("teistro-dossier-every.txt"),
        &text,
    )
    .unwrap();
    assert_eq!(
        banners(&text),
        [
            "LEGEND",
            "CHART",
            "GRAHAS",
            "HOUSES",
            "VARGAS",
            "DASHAS",
            "PANCHANGA",
            "ASHTAKAVARGA",
            "SHADBALA",
            "BHAVA_BALA",
            "VIMSHOPAKA",
            "JAIMINI",
            "POINTS",
            "ASPECTS",
            "AVAKAHADA",
        ]
    );
    // The sarva row sums to the 337 bindus every chart has.
    let sarva = text
        .lines()
        .find(|line| line.starts_with("SARVA|"))
        .unwrap();
    assert!(sarva.ends_with("|337"), "{sarva}");
    // Smaller than the document's own JSON, which says the same.
    let json = serde_json::to_string(&document).unwrap();
    assert!(
        text.len() < json.len(),
        "{} against {}",
        text.len(),
        json.len()
    );
}

#[test]
fn a_dated_dossier_marks_the_running_periods_and_expands_their_branch() {
    let document = full();
    let at = BIRTH + 365.25 * 30.0;
    let dated = dossier(
        &document,
        &DossierOptions {
            at: Some(at),
            ..on_clock()
        },
        &[],
    )
    .unwrap();
    let free = dossier(&document, &on_clock(), &[]).unwrap();
    assert!(banners(&dated).contains(&"NOW"));
    assert!(dated.contains("|30.00\n"), "the age");
    let running: Vec<&str> = dated
        .lines()
        .filter(|line| line.starts_with("VIMSHOTTARI|") && line.ends_with("|*"))
        .collect();
    assert!(!running.is_empty(), "{dated}");
    assert!(
        running.windows(2).all(|pair| pair[1]
            .split('|')
            .nth(1)
            .unwrap()
            .starts_with(pair[0].split('|').nth(1).unwrap())),
        "one branch"
    );
    let rows = |text: &str| {
        text.lines()
            .filter(|line| line.starts_with("VIMSHOTTARI|"))
            .count()
    };
    assert!(
        rows(&dated) < rows(&free),
        "the branch alone below the first level"
    );
}

#[test]
fn a_preset_and_a_depth_choose_without_refusing() {
    let document = full();
    let family = dossier(
        &document,
        &DossierOptions {
            preset: DossierPreset::Family,
            dasha_depth: Some(1),
            ..on_clock()
        },
        &[],
    )
    .unwrap();
    assert_eq!(
        banners(&family),
        ["LEGEND", "CHART", "GRAHAS", "DASHAS", "PANCHANGA"]
    );
    assert!(!family.contains("VARGAS"), "the legend too is scoped");
    assert!(
        family
            .lines()
            .filter(|line| line.starts_with("VIMSHOTTARI|"))
            .all(|line| !line.split('|').nth(1).unwrap().contains('/'))
    );
    // A bare chart: the preset writes what there is.
    let bare = dossier(&chart(&kathmandu()), &on_clock(), &[]).unwrap();
    assert_eq!(banners(&bare), ["LEGEND", "CHART", "GRAHAS"]);
}

#[test]
fn a_section_the_chart_lacks_or_does_not_know_is_refused_by_its_place() {
    let bare = chart(&kathmandu());
    let asked = |sections: &[&str]| DossierOptions {
        sections: Some(sections.iter().map(|one| (*one).to_owned()).collect()),
        ..on_clock()
    };
    let lacking = dossier(&bare, &asked(&["GRAHAS", "DASHAS"]), &[]).unwrap_err();
    assert_eq!(lacking.field(), Some("sections[1]"));
    assert!(lacking.to_string().contains("dashas"), "{lacking}");
    let unknown = dossier(&bare, &asked(&["PLANETS"]), &[]).unwrap_err();
    assert_eq!(unknown.field(), Some("sections[0]"));
    // Named in any order, written in the dossier's.
    let text = dossier(&bare, &asked(&["GRAHAS", "CHART"]), &[]).unwrap();
    assert_eq!(banners(&text), ["LEGEND", "CHART", "GRAHAS"]);
    let zero = dossier(
        &bare,
        &DossierOptions {
            dasha_depth: Some(0),
            ..on_clock()
        },
        &[],
    )
    .unwrap_err();
    assert_eq!(zero.field(), Some("dashaDepth"));
}

#[test]
fn a_record_widens_the_request_and_writes_on_its_clock() {
    let sdk = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("a built context");
    let instants = [
        JulianDay::<Utc>::literal(BIRTH),
        JulianDay::<Utc>::literal(BIRTH + 1.0),
    ];
    let compose = |json: &str| {
        let mut records = ChartRecords::default();
        records.read("dossier", json)?;
        let records = records.checked()?;
        sdk.chart()
            .compose(&instants, &kathmandu(), &records)
            .map(|composed| (composed.dossier, composed.founded.value))
    };
    let (texts, documents) = compose(r#"{"sections": ["GRAHAS", "SHADBALA"]}"#).unwrap();
    assert_eq!(texts.len(), 2);
    for (text, document) in texts.iter().zip(&documents) {
        assert!(document.shadbala.is_some(), "the record asked for it");
        assert_eq!(banners(text), ["LEGEND", "GRAHAS", "SHADBALA"]);
        // The request's own clock.
        assert_eq!(
            text,
            &dossier(
                document,
                &DossierOptions {
                    sections: Some(vec!["GRAHAS".into(), "SHADBALA".into()]),
                    ..on_clock()
                },
                &[],
            )
            .unwrap()
        );
    }
    let (texts, _) = compose("{}").unwrap();
    assert!(texts[0].contains("+05:45"), "the request's clock");
    let lacking = compose(r#"{"sections": ["DASHAS"]}"#).unwrap_err();
    assert_eq!(lacking.field(), Some("dossier.sections[0]"));
    let unasked = ChartRecords::default();
    let none = sdk
        .chart()
        .compose(&instants, &kathmandu(), &unasked)
        .unwrap();
    assert_eq!(none.dossier, Vec::<String>::new());
}
