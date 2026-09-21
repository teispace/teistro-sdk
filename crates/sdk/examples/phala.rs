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
//! ```sh
//! cargo run --release -p teistro --example phala
//! ```

#![expect(clippy::print_stdout, reason = "an example is a program that prints")]

use std::path::Path;

use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
use teistro::{ChartRequest, Context, Ephemeris, Error, Tree, UtcOffset, pack};

/// Where the SDK keeps the two corpora's sources. A consumer ships the
/// built `.tpack` files instead and never sees these directories.
const CORPORA: [&str; 2] = ["packs/readings", "packs/states"];

fn main() -> Result<(), Error> {
    let sdk = Context::builder()
        .profile("nepali-default")
        .ephemeris([Ephemeris::Builtin])
        .build()?;

    // ── The packs ──────────────────────────────────────────────────────
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for corpus in CORPORA {
        let tree = Tree::load(&repository.join(corpus))
            .map_err(|why| Error::invalid_arg(why.to_string()))?;
        for locale in tree.locales.values() {
            let bytes = pack::build(locale, "sdk.entity")
                .map_err(|why| Error::invalid_arg(why.to_string()))?;
            let loaded = sdk.intl().load_pack(&bytes)?;
            println!(
                "{corpus:<15} {:<12} {:>5} records, {:>4} merged, {:>7} bytes",
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
    // `with_state` is what makes it interpretable: a composer that reads
    // what a graha *is* needs the states, and a document without them is
    // refused rather than read as neutral.
    let request = ChartRequest::at(place, UtcOffset::try_from_seconds(20_700)?).with_state();
    let document = sdk
        .chart()
        .reading(JulianDay::literal(2_447_995.489_583_333_5), &request)?
        .value;

    // ── The plan, said twice ───────────────────────────────────────────
    // In Rust a composer is a method; across the boundary it is a member
    // of `PlanRequest` — `{"phala": true}` — off unless asked for, so a
    // chart says nothing new until a consumer asks for it.
    let plan = sdk.interpret().phala(&document)?;
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
