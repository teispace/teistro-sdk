//! The dossier's tokens, measured (`03-design/dossier.md`, "Names", A2-7):
//! how many tokens a chart's whole dossier spends written in catalogue
//! keys and in each shipped locale's names, beside its document's
//! canonical JSON, over the conformance corpus's births, counted with the
//! dependency-free approximation of `tokens.rs` (C1165).
//!
//! `cargo xtask dossier-tokens` writes the page; `check-dossier-tokens`
//! regenerates it in memory and fails on any difference.

use std::fmt::Write as _;
use std::path::Path;

use teistro::Status;
use teistro::dossier::{DossierOptions, dossier};
use teistro_serial::canonical::to_hash_form;

use crate::births::{CHARTS, births, conformance};
use crate::dossier_measured::whole;
use crate::generated::{Output, check, write};
use crate::measure::{Claim, count, fill, table};
use crate::tokens::approximate;

const PAGE: &str = "docs/03-design/dossier-tokens-measured.md";

/// The locales the SDK ships, each a dossier is written in.
const LOCALES: [&str; 5] = ["en-Latn", "hi-Deva-IN", "ne-Deva-NP", "sa-Deva", "sa-Latn"];

/// One way of writing the corpus's dossiers, summed.
#[derive(Default)]
struct Mode {
    bytes: usize,
    scalars: usize,
    tokens: usize,
}

impl Mode {
    fn add(&mut self, text: &str) {
        self.bytes += text.len();
        self.scalars += text.chars().count();
        self.tokens += approximate(text);
    }
}

struct Measured {
    births: usize,
    beyond: Vec<String>,
    keys: Mode,
    json: Mode,
    locales: Vec<Mode>,
    /// Named dossiers whose lines, banners or fields differ in shape from
    /// the keyed one's, and how many were compared.
    misshapen: usize,
    compared: usize,
}

/// A dossier's shape: its banners and header lines whole, and each row's
/// field count.
fn shape(text: &str) -> Vec<String> {
    text.lines()
        .map(|line| {
            if line.starts_with("== ") || line.starts_with("# ") && !line.contains("Members are") {
                line.to_owned()
            } else {
                line.split('|').count().to_string()
            }
        })
        .collect()
}

fn measure(root: &Path) -> Result<Measured, String> {
    let sdk = conformance()?;
    let corpus = births(root, &sdk)?;
    let mut measured = Measured {
        births: corpus.len(),
        beyond: Vec::new(),
        keys: Mode::default(),
        json: Mode::default(),
        locales: LOCALES.iter().map(|_| Mode::default()).collect(),
        misshapen: 0,
        compared: 0,
    };
    for birth in &corpus {
        let document = match whole(&sdk, birth) {
            Ok(read) => read.value,
            Err(why) if why.status == Status::OutOfRange => {
                measured.beyond.push(birth.name.clone());
                continue;
            }
            Err(why) => return Err(format!("{}: {why}", birth.name)),
        };
        let options = DossierOptions {
            utc_offset_seconds: Some(birth.offset.seconds()),
            ..DossierOptions::default()
        };
        let keyed =
            dossier(&document, &options, &[]).map_err(|why| format!("{}: {why}", birth.name))?;
        measured.keys.add(&keyed);
        measured.json.add(&to_hash_form(&document));
        let keyed_shape = shape(&keyed);
        for (locale, mode) in LOCALES.iter().zip(&mut measured.locales) {
            let named = sdk
                .intl()
                .named_dossier(&keyed, Some(locale))
                .map_err(|why| format!("{} in {locale}: {why}", birth.name))?;
            mode.add(&named);
            measured.compared += 1;
            measured.misshapen += usize::from(shape(&named) != keyed_shape);
        }
    }
    Ok(measured)
}

/// `part / whole` to two decimals.
fn ratio(part: usize, whole: usize) -> String {
    #[allow(
        clippy::cast_precision_loss,
        reason = "counts of a corpus, far under 2^52"
    )]
    let value = part as f64 / whole.max(1) as f64;
    format!("{value:.2}")
}

fn page(measured: &Measured) -> String {
    let charts = measured.births - measured.beyond.len();
    let aside = match measured.beyond.as_slice() {
        [] => String::from("No birth was set aside."),
        names => format!(
            "{} of the corpus's {} births {} set aside, a section reading a day the built-in \
             ephemeris does not reach: `{}`.",
            names.len(),
            measured.births,
            if names.len() == 1 { "is" } else { "are" },
            names.join("`, `"),
        ),
    };
    let mut rows = String::from(
        "| written in | bytes | scalar values | tokens | tokens a chart | tokens against keys |\n|---|---|---|---|---|---|\n",
    );
    let mut row = |name: &str, mode: &Mode| {
        let _ = writeln!(
            rows,
            "| {name} | {} | {} | {} | {} | {} |",
            count(mode.bytes),
            count(mode.scalars),
            count(mode.tokens),
            count(mode.tokens / charts.max(1)),
            ratio(mode.tokens, measured.keys.tokens),
        );
    };
    row("keys (language-neutral)", &measured.keys);
    for (locale, mode) in LOCALES.iter().zip(&measured.locales) {
        row(&format!("`{locale}` names"), mode);
    }
    row("the document's canonical JSON", &measured.json);
    let claims = [
        Claim::counted(
            "every dossier written in a locale's names keeps the keyed one's lines, banners, \
             column headers and field counts",
            measured.misshapen,
            measured.compared,
        ),
        Claim::counted(
            "the keyed dossier is fewer tokens than the document's canonical JSON, over the corpus",
            usize::from(measured.keys.tokens >= measured.json.tokens),
            1,
        ),
    ];
    let mut out = String::from("# The dossier's tokens, measured\n\n");
    out.push_str(
        "Status: `generated` by `cargo xtask dossier-tokens` and held by \
         `check-dossier-tokens`; do not edit. Each birth of the conformance corpus founded \
         with every section, as the dossier's own page founds it (`dossier-measured.md`), \
         written as a whole date-free dossier on its own clock: once in catalogue keys, the \
         language-neutral mode, and once in each shipped locale's names (`dossier.md`, \
         \"Names\"), beside the canonical JSON of the same document.\n\n",
    );
    out.push_str(&aside);
    let _ = write!(
        out,
        "\n\n## The tokeniser\n\nTokens are counted by an approximation with no dependency \
         (`xtask/src/tokens.rs`, C1165): a run of ASCII letters is a token for each 4 letters \
         or part of 4, a run of digits a token for each 3, a newline one, other whitespace \
         nothing, any other ASCII character one, and any other scalar value (Devanagari, \
         IAST's marked letters) one. No model's tokenizer ships with the SDK and each is its \
         own trained vocabulary, so a count by one is that model's alone; this one prices \
         the pieces a byte-pair tokenizer cannot merge across, the same ruler for every mode, \
         and its prices are held by the module's own tests. Read the columns against each \
         other, not as a model's bill.\n\n## The claims\n\n{}\n## Over {charts} charts\n\n{rows}",
        table(&claims),
    );
    fill(&out)
}

fn outputs(root: &Path) -> Result<Vec<Output>, String> {
    let measured = measure(root)?;
    if measured.births == measured.beyond.len() {
        return Err(format!("{CHARTS} holds no birth the dossier reaches"));
    }
    Ok(vec![Output::new(PAGE, page(&measured))])
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
        Ok(outputs) => i32::from(check(root, &outputs, "cargo xtask dossier-tokens") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}
