//! What the chart *is*, read aloud: the state readings, loaded beside the
//! rule readings.
//!
//! `readings.rs` loaded a pack of readings for the yogas and doshas a chart
//! **holds**. This one loads the other half of that corpus: a reading for
//! what the chart *is* without holding anything — Jupiter in the first
//! house, the lagna's sign, the tithi, the nakshatra the Moon stands in
//! (`docs/03-design/state-readings.md`).
//!
//! What it teaches:
//!
//! 1. **Two packs, one engine.** Each root builds one pack a locale, and a
//!    consumer loads the ones it wants. They are loaded in either order.
//! 2. **A record gains forms; it does not lose them.** Both corpora
//!    describe some of the same subjects, and so may yours: a pack carrying
//!    one form adds that form and leaves the rest of the record standing.
//!    `loaded.merged` counts the records that kept something.
//! 3. **A composer says nothing it has no words for.** `phala` asks the
//!    base locale for each subject and is silent where the answer is no, so
//!    a chart composes exactly as it did before until a pack is loaded.
//!
//! The packs are the files `teistro-intl build` writes, one a locale, which
//! `cargo xtask check-parity` builds before it runs any example:
//!
//! ```sh
//! cargo run -p teistro-intl -- --root packs/readings build --out target/packs/readings
//! cargo run -p teistro-intl -- --root packs/states build --out target/packs/states
//! cargo run --release -p teistro --example phala
//! ```

#![expect(clippy::print_stdout, reason = "an example is a program that prints")]

use std::path::{Path, PathBuf};

use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
use teistro::{ChartRequest, Context, Ephemeris, Error, PlanRequest, UtcOffset};

/// The two corpora, each built from its source under `packs/` into packs of
/// its own (`packs/README.md`).
const CORPORA: [&str; 2] = ["readings", "states"];

/// Where the built packs are: `TEISTRO_PACKS`, or the repository's
/// `target/packs`, which `cargo xtask check-parity` builds before it runs
/// any example.
fn packs() -> PathBuf {
    std::env::var_os("TEISTRO_PACKS").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/packs"),
        PathBuf::from,
    )
}

/// Every pack a corpus built, one a locale, in name order.
fn packs_of(corpus: &str) -> Result<Vec<PathBuf>, Error> {
    let directory = packs().join(corpus);
    let unbuilt = |why: std::io::Error| {
        Error::invalid_arg(format!("no packs in {}: {why}", directory.display())).with_hint(
            format!(
                "build them: cargo run -p teistro-intl -- --root packs/{corpus} build --out target/packs/{corpus}"
            ),
        )
    };
    let mut found: Vec<PathBuf> = std::fs::read_dir(&directory)
        .map_err(unbuilt)?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|kind| kind == "tpack"))
        .collect();
    found.sort();
    Ok(found)
}

fn main() -> Result<(), Error> {
    let sdk = Context::builder()
        .profile("nepali-default")
        .ephemeris([Ephemeris::Builtin])
        .build()?;

    // ── The packs ──────────────────────────────────────────────────────
    for corpus in CORPORA {
        for file in packs_of(corpus)? {
            let bytes = std::fs::read(&file)
                .map_err(|why| Error::invalid_arg(format!("{}: {why}", file.display())))?;
            let loaded = sdk.intl().load_pack(&bytes)?;
            println!(
                "{:<15} {:<12} {:>5} records, {:>4} merged, {:>7} bytes",
                format!("packs/{corpus}"),
                loaded.locale,
                loaded.entries,
                loaded.merged,
                bytes.len()
            );
        }
    }

    // ── A chart ────────────────────────────────────────────────────────
    let place = Place::new(
        Latitude::try_new(27.7172)?,
        Longitude::try_new(85.3240)?,
        Altitude::try_new(1400.0)?,
    );
    let request = ChartRequest::at(place, UtcOffset::try_from_seconds(20_700)?);

    // ── The plan, said twice ───────────────────────────────────────────
    // A composer is a member of `PlanRequest` — `{"phala": true}` in every
    // binding — off unless asked for, so a chart says nothing new until a
    // consumer asks for it. `interpreted` founds the chart with what the
    // composer reads, its states and the panchanga's limbs among them, and
    // composes in the same call.
    let read = sdk.chart().interpreted(
        &[JulianDay::literal(2_447_995.489_583_333_5)],
        &request,
        None,
        PlanRequest::default().with_phala(),
    )?;
    let plan = read
        .value
        .into_iter()
        .next()
        .and_then(|chart| chart.plans.phala)
        .ok_or_else(|| Error::internal("one chart asked for its phala"))?;
    println!("\n{} items", plan.len());
    for locale in ["en-Latn", "ne-Deva-NP"] {
        sdk.intl().set_locale(locale)?;
        println!("\n{locale}");
        for item in plan.iter().take(4) {
            let said = sdk.intl().render(&item.key, &item.params);
            let mark = if said.is_fallback { "  (fallback)" } else { "" };
            println!("  {}{mark}", shortened(&said.text));
        }
    }

    // ── The record that two corpora describe ───────────────────────────
    // `nakshatra-phala` says what the nakshatra portends and
    // `namakarana-nakshatra` what to name a child born under it. Both are
    // forms on the record the SDK already names, beside its own `name` and
    // `iast` — which is what the merge on load is for.
    sdk.intl().set_locale("en-Latn")?;
    let record = sdk.intl().entity("nakshatra.ASHWINI")?;
    println!("\nnakshatra.ASHWINI");
    for form in ["name", "iast", "phala", "namakarana"] {
        if let Some(text) = record.form(form) {
            println!("  {form:<12} {}", shortened(text));
        }
    }

    // ── A reading no composer says ─────────────────────────────────────
    // Half the corpus is glossary rather than narrative: what it means for
    // a graha to be exalted is true of every exalted graha, so no composer
    // says it per chart. It is a record like any other, and any catalogue
    // key you can name you can ask for — which is how a consumer builds a
    // legend beside the plan.
    println!("\ndignity.EXALTED");
    let record = sdk.intl().entity("dignity.EXALTED")?;
    for form in ["name", "phala"] {
        if let Some(text) = record.form(form) {
            println!("  {form:<12} {}", shortened(text));
        }
    }
    Ok(())
}

/// Enough of a passage to show it is there, without printing an essay.
fn shortened(text: &str) -> String {
    let mut out: String = text.chars().take(88).collect();
    if text.chars().count() > 88 {
        out.push('…');
    }
    out
}
