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
//!
//! And it measures the dossier's companion over the same documents, the
//! gochar sidecar (`03-design/gochar-sidecar.md`): that each birth's reads
//! back as itself, that the transits read from it are the document's, and
//! its bytes against the document's, on a page of its own.

use std::fmt::Write as _;
use std::path::Path;

use serde_json::Value;
use teistro::catalogue::{DashaSystem, Varga};
use teistro::dossier::{DossierOptions, dossier};
use teistro::{
    Document, Envelope, GocharRequest, GocharSidecar, SarvatobhadraRequest, Status,
    quantity::{JulianDay, Utc},
};
use teistro_serial::canonical::to_hash_form;

use crate::births::{CHARTS, births, conformance};
use crate::generated::{Output, check, write};
use crate::measure::{Claim, count, fill, table};

const PAGE: &str = "docs/03-design/dossier-measured.md";
/// The gochar sidecar's page.
const SIDECAR_PAGE: &str = "docs/03-design/gochar-sidecar-measured.md";
/// The instants the sidecar's transits are compared at: four a quarter
/// apart from 1 January 2025, inside the built-in ephemeris for every birth.
const TRANSIT_DAYS: [f64; 4] = [2_460_676.5, 2_460_767.5, 2_460_858.5, 2_460_949.5];
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
fn whole(
    sdk: &teistro::Context,
    birth: &crate::births::Birth,
) -> Result<Envelope<Document>, teistro::Error> {
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
}

/// One birth's sidecar against its document.
#[derive(Default)]
struct Sidecars {
    births: usize,
    bytes: usize,
    document: usize,
    least: Option<usize>,
    most: usize,
    larger: usize,
    unread: usize,
    compared: usize,
    differ: usize,
}

impl Sidecars {
    /// Cuts `read`'s sidecar, stores and reads it back, opens it, and
    /// compares its transits with the document's.
    fn add(&mut self, sdk: &teistro::Context, read: &Envelope<Document>) -> Result<(), String> {
        let text = GocharSidecar::cut(&read.value, &read.provenance)
            .map_err(|why| why.to_string())?
            .to_text();
        let document = to_hash_form(&read.value).len();
        self.births += 1;
        self.bytes += text.len();
        self.document += document;
        self.least = Some(self.least.map_or(text.len(), |least| least.min(text.len())));
        self.most = self.most.max(text.len());
        self.larger += usize::from(text.len() >= document);
        let sidecar = match GocharSidecar::read(&text) {
            Ok(sidecar) if sidecar.to_text() == text => sidecar,
            _ => {
                self.unread += 1;
                return Ok(());
            }
        };
        sidecar
            .check_chart(read.provenance.content_hash)
            .map_err(|why| why.to_string())?;
        let natal = sdk.chart().natal(&sidecar).map_err(|why| why.to_string())?;
        let days: Vec<JulianDay<Utc>> = TRANSIT_DAYS.map(JulianDay::literal).to_vec();
        let gochar = GocharRequest::over(days.clone()).with_ashtakavarga();
        let chakra = SarvatobhadraRequest::over(days);
        let answer = |result: Result<String, teistro::Error>| result.map_err(|why| why.to_string());
        let pairs = [
            (
                answer(
                    sdk.chart()
                        .gochar(&natal, &gochar)
                        .map(|got| to_hash_form(&got.value)),
                ),
                answer(
                    sdk.chart()
                        .gochar(&read.value, &gochar)
                        .map(|got| to_hash_form(&got.value)),
                ),
            ),
            (
                answer(
                    sdk.chart()
                        .sarvatobhadra(&natal, &chakra)
                        .map(|got| to_hash_form(&got.value)),
                ),
                answer(
                    sdk.chart()
                        .sarvatobhadra(&read.value, &chakra)
                        .map(|got| to_hash_form(&got.value)),
                ),
            ),
        ];
        for (opened, whole) in pairs {
            self.compared += 1;
            self.differ += usize::from(opened != whole);
        }
        Ok(())
    }
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
    sidecars: Sidecars,
}

fn measure(root: &Path) -> Result<Measured, String> {
    let sdk = conformance()?;
    let corpus = births(root, &sdk)?;
    let mut groups: Vec<Total> = GROUPS.iter().map(|_| Total::default()).collect();
    let mut whole_total = Total::default();
    let mut extremes: Option<(f64, f64)> = None;
    let mut beyond = Vec::new();
    let mut fixture = None;
    let mut sidecars = Sidecars::default();
    for birth in &corpus {
        let read = match whole(&sdk, birth) {
            Ok(read) => read,
            // A section reads a day the ephemeris does not reach.
            Err(why) if why.status == Status::OutOfRange => {
                beyond.push(birth.name.clone());
                continue;
            }
            Err(why) => return Err(format!("{}: {why}", birth.name)),
        };
        sidecars
            .add(&sdk, &read)
            .map_err(|why| format!("{}: {why}", birth.name))?;
        let document = read.value;
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
        sidecars,
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
        Output::new(SIDECAR_PAGE, sidecar_page(&measured)),
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

fn sidecar_page(measured: &Measured) -> String {
    let sidecars = &measured.sidecars;
    let claims = [
        Claim::counted(
            "every birth's sidecar reads back as itself, to the byte",
            sidecars.unread,
            sidecars.births,
        ),
        Claim::counted(
            "the gochar (with the Ashtakavarga's judgement) and the Sarvatobhadra read from an \
             opened sidecar are the whole document's, in bytes, at four instants",
            sidecars.differ,
            sidecars.compared,
        ),
        Claim::counted(
            "every birth's sidecar is fewer bytes than its document",
            sidecars.larger,
            sidecars.births,
        ),
    ];
    let mut out = String::from("# The gochar sidecar, measured\n\n");
    out.push_str(
        "Status: `generated` by `cargo xtask dossier` and held by `check-dossier`; do not \
         edit. The gochar sidecar (`gochar-sidecar.md`) of each birth the dossier's page \
         measures (`dossier-measured.md`), cut from the same document with every section: \
         stored as its canonical text, read back, checked against the document's content \
         hash, opened under the same context, and its transits read at 1 January, 2 April, \
         2 July and 1 October 2025 (UTC midnight) beside the whole document's. A transit \
         read refused on both sides is the same answer.\n\n",
    );
    let _ = write!(
        out,
        "## The claims\n\n{}\n## Its size\n\nOver {} births the sidecars come to {} \
         bytes and the documents they were cut from to {}: {} of it, one sidecar between {} \
         and {} bytes. The sidecar is the document's foundation and panchanga whole; the \
         rest of the document is what no transit reads.\n",
        table(&claims),
        sidecars.births,
        count(sidecars.bytes),
        count(sidecars.document),
        ratio(sidecars.bytes, sidecars.document),
        count(sidecars.least.unwrap_or_default()),
        count(sidecars.most),
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
