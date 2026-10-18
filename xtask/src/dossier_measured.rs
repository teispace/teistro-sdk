//! The dossier, measured (`03-design/dossier.md`, "What will be
//! measured"): how many bytes each section of a chart's dossier spends
//! against the canonical JSON of what it writes, over the conformance
//! corpus's births founded by the SDK, so "far fewer bytes" is a number
//! and not a sentence.
//!
//! A section's bytes are what adding it to a dossier costs: its legend
//! lines, its banner, its header and its rows, the dossier of nothing
//! subtracted. Its JSON is the canonical JSON of the document members it
//! is written from. `cargo xtask dossier` writes the page;
//! `check-dossier` regenerates it in memory and fails on any difference.
//!
//! The pass also writes one of those documents, the first birth's, as the
//! fixture `teistro-serial`'s own dossier tests read: the crate cannot
//! found a chart with every section, and a fixture held here is never a
//! document the SDK no longer writes.

use std::fmt::Write as _;
use std::path::Path;

use serde_json::Value;
use teistro::catalogue::{DashaSystem, Varga};
use teistro::dossier::{DossierOptions, dossier};
use teistro::{Document, Status, quantity::JulianDay};
use teistro_serial::canonical::to_hash_form;

use crate::births::{CHARTS, births, conformance};
use crate::generated::{Output, check, write};
use crate::measure::{Claim, count, fill, table};

const PAGE: &str = "docs/03-design/dossier-measured.md";
/// The full document `teistro-serial`'s dossier tests read.
const FIXTURE: &str = "crates/serial/tests/dossier-chart.json";
/// The birth the fixture is, the corpus's first.
const FIXTURE_BIRTH: &str = "c001-kathmandu-1990-04-14";

/// The dossier's sections, grouped by the document members they are
/// written from: `CHART` and `GRAHAS` both read the foundation, so they
/// are measured together against it and the state.
const GROUPS: [(&[&str], &[&str]); 13] = [
    (&["CHART", "GRAHAS"], &["foundation", "state"]),
    (&["HOUSES"], &["houses"]),
    (&["VARGAS"], &["vargas"]),
    (&["DASHAS"], &["dashas"]),
    (&["PANCHANGA"], &["panchanga"]),
    (&["ASHTAKAVARGA"], &["ashtakavarga"]),
    (&["SHADBALA"], &["shadbala"]),
    (&["BHAVA_BALA"], &["bhava_bala"]),
    (&["VIMSHOPAKA"], &["vimshopaka"]),
    (&["JAIMINI"], &["jaimini"]),
    (&["POINTS"], &["points"]),
    (&["ASPECTS"], &["aspects"]),
    (&["AVAKAHADA"], &["avakahada"]),
];

/// Bytes over the corpus: a group's, or the whole dossiers'.
#[derive(Default)]
struct Total {
    charts: usize,
    dossier: usize,
    json: usize,
    larger: usize,
}

impl Total {
    /// One chart's dossier bytes and JSON bytes.
    fn add(&mut self, dossier: usize, json: usize) {
        self.charts += 1;
        self.dossier += dossier;
        self.json += json;
        self.larger += usize::from(dossier >= json);
    }
}

/// A birth founded with every section a dossier writes; the points left
/// out of one whose sunrise the day does not have (a polar summer), which
/// the points need and the corpus holds.
fn whole(sdk: &teistro::Context, birth: &crate::births::Birth) -> Result<Document, teistro::Error> {
    let at = JulianDay::literal(birth.at());
    let request = birth
        .request()
        .with_panchanga()
        .with_state()
        .with_houses()
        .with_ashtakavarga()
        .with_shadbala()
        .with_bhava_bala()
        .with_vimshopaka()
        .with_jaimini()
        .with_aspects()
        .with_avakahada()
        .with_vargas([Varga::D9, Varga::D10])
        .with_dashas([DashaSystem::Vimshottari]);
    sdk.chart()
        .reading(at, &request.clone().with_points())
        .or_else(|why| match why.field() {
            Some(field) if field.starts_with("points") => sdk.chart().reading(at, &request),
            _ => Err(why),
        })
        .map(|read| read.value)
}

/// The dossier of `sections` alone, on the birth's clock.
fn written(document: &Document, sections: &[&str], clock: i32) -> Result<String, String> {
    let options = DossierOptions {
        sections: Some(sections.iter().map(|one| (*one).to_owned()).collect()),
        utc_offset_seconds: Some(clock),
        ..DossierOptions::default()
    };
    dossier(document, &options, &[]).map_err(|why| format!("{sections:?}: {why}"))
}

/// The canonical JSON bytes of the document members `keys`, as one object.
fn json_of(document: &Value, keys: &[&str]) -> usize {
    let picked: serde_json::Map<String, Value> = keys
        .iter()
        .filter_map(|key| Some(((*key).to_owned(), document.get(*key)?.clone())))
        .collect();
    to_hash_form(&picked).len()
}

/// `dossier / json` to two decimals.
fn ratio(dossier: usize, json: usize) -> String {
    #[allow(
        clippy::cast_precision_loss,
        reason = "byte counts of a corpus, far under 2^52"
    )]
    let value = dossier as f64 / json.max(1) as f64;
    format!("{value:.2}")
}

/// What the corpus measured: each group's total, the whole dossiers',
/// the lowest and highest ratio of one chart, and the births set aside.
struct Measured {
    fixture: Option<String>,
    births: usize,
    beyond: Vec<String>,
    groups: Vec<Total>,
    whole: Total,
    extremes: (f64, f64),
}

fn measure(root: &Path) -> Result<Measured, String> {
    let sdk = conformance()?;
    let corpus = births(root, &sdk)?;
    let mut groups: Vec<Total> = GROUPS.iter().map(|_| Total::default()).collect();
    let mut whole_total = Total::default();
    let mut extremes: Option<(f64, f64)> = None;
    let mut beyond = Vec::new();
    let mut fixture = None;
    for birth in &corpus {
        let document = match whole(&sdk, birth) {
            Ok(document) => document,
            // A section reads a day the ephemeris does not reach.
            Err(why) if why.status == Status::OutOfRange => {
                beyond.push(birth.name.clone());
                continue;
            }
            Err(why) => return Err(format!("{}: {why}", birth.name)),
        };
        if birth.name == FIXTURE_BIRTH {
            fixture = Some(to_hash_form(&document) + "\n");
        }
        let clock = birth.offset.seconds();
        let value = serde_json::to_value(&document).map_err(|why| why.to_string())?;
        let empty = written(&document, &[], clock)?.len();
        for ((sections, keys), total) in GROUPS.iter().zip(&mut groups) {
            if keys.iter().any(|key| value.get(*key).is_none()) {
                continue;
            }
            let bytes = written(&document, sections, clock)?.len() - empty;
            total.add(bytes, json_of(&value, keys));
        }
        let options = DossierOptions {
            utc_offset_seconds: Some(clock),
            ..DossierOptions::default()
        };
        let text =
            dossier(&document, &options, &[]).map_err(|why| format!("{}: {why}", birth.name))?;
        let json = to_hash_form(&document).len();
        whole_total.add(text.len(), json);
        #[allow(
            clippy::cast_precision_loss,
            reason = "byte counts of one chart, far under 2^52"
        )]
        let one = text.len() as f64 / json as f64;
        extremes = Some(extremes.map_or((one, one), |(low, high)| (low.min(one), high.max(one))));
    }
    Ok(Measured {
        fixture,
        births: corpus.len(),
        beyond,
        groups,
        whole: whole_total,
        extremes: extremes.ok_or_else(|| format!("{CHARTS} holds no births"))?,
    })
}

fn outputs(root: &Path) -> Result<Vec<Output>, String> {
    let measured = measure(root)?;
    let fixture = measured
        .fixture
        .clone()
        .ok_or_else(|| format!("{CHARTS} holds no {FIXTURE_BIRTH} with every section"))?;
    Ok(vec![
        Output::new(PAGE, page(&measured)),
        Output::new(FIXTURE, fixture),
    ])
}

fn page(measured: &Measured) -> String {
    let aside = match measured.beyond.as_slice() {
        [] => String::from("No birth was set aside."),
        names => format!(
            "Of the corpus's {} births, {} {} set aside, a section reading a day the \
             built-in ephemeris does not reach: `{}`.",
            measured.births,
            names.len(),
            if names.len() == 1 { "is" } else { "are" },
            names.join("`, `"),
        ),
    };
    let mut sections = String::from(
        "| sections | charts carrying them | dossier bytes | JSON bytes | dossier / JSON | charts where the dossier is not smaller |\n|---|---|---|---|---|---|\n",
    );
    for ((names, _), total) in GROUPS.iter().zip(&measured.groups) {
        let _ = writeln!(
            sections,
            "| `{}` | {} | {} | {} | {} | {} |",
            names.join("`, `"),
            total.charts,
            count(total.dossier),
            count(total.json),
            ratio(total.dossier, total.json),
            total.larger,
        );
    }
    let whole = &measured.whole;
    let claims = [
        Claim::counted(
            "every chart's whole dossier is fewer bytes than its document's canonical JSON",
            whole.larger,
            whole.charts,
        ),
        Claim::counted(
            "every section's dossier is fewer bytes than the JSON of what it writes, chart by chart",
            measured.groups.iter().map(|total| total.larger).sum(),
            measured.groups.iter().map(|total| total.charts).sum(),
        ),
    ];
    let (low, high) = measured.extremes;
    let mut out = String::from("# The dossier, measured\n\n");
    out.push_str(
        "Status: `generated` by `cargo xtask dossier` and held by `check-dossier`; do not \
         edit. What a chart's dossier (`dossier.md`) spends against the canonical JSON of the \
         same chart, over the conformance corpus's births founded by the SDK under the \
         `conformance-baseline` profile on the built-in ephemeris, each with every section a \
         dossier writes: the state, the panchanga, the houses, the Ashtakavarga, Shadbala, \
         Bhava bala, Vimshopaka, Jaimini, the points, the aspects, the Avakahada, the D9 and \
         the D10, and the Vimshottari to the depth the request computes. The dossier is \
         date-free and on each birth's own clock. A birth whose day has no \
         sunrise (a polar summer) cannot carry the points, which count from it, and is \
         founded without them.\n\n",
    );
    out.push_str(&aside);
    let _ = write!(
        out,
        "\n\n## The claims\n\n{}\n## A chart whole\n\nOver {} births the dossiers come to {} \
         bytes and the documents' canonical JSON to {}: {} of it, a chart's own ratio between \
         {low:.2} and {high:.2}.\n\n## Section by section\n\nA section's bytes are what adding it \
         to a dossier costs — its legend lines, banner, header and rows — and its JSON is the \
         canonical JSON of the document members it is written from (`CHART` and `GRAHAS` \
         together, against the foundation and the state), summed over the births.\n\n{sections}\n\
         Bytes are not tokens: a tokenizer splits a catalogue key (`PURVA_PHALGUNI`) and a \
         number differently from a JSON key, so this page counts what is stored and sent, and \
         a token count is the model's own.\n",
        table(&claims),
        whole.charts,
        count(whole.dossier),
        count(whole.json),
        ratio(whole.dossier, whole.json),
    );
    fill(&out)
}

pub(crate) fn generate(root: &Path) -> i32 {
    match outputs(root) {
        Ok(outputs) => write(root, &outputs),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    match outputs(root) {
        Ok(outputs) => i32::from(check(root, &outputs, "cargo xtask dossier") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}
