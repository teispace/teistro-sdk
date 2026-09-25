//! A rule's own reading, in the reader's language — loaded, not embedded.
//!
//! `interpretation.rs` composed a plan and said it in two languages. One
//! thing it could not say in Nepali was **what a rule's verse states**:
//! that crosses as the words the rule cites, in the language the rule was
//! written in, and a Nepali reading shows the seam.
//!
//! This is how the seam closes. The SDK carries a corpus of readings — one
//! for each of 649 yogas and doshas, in Sanskrit, Nepali, English and Hindi
//! — and it is **not compiled into the library**: it is several times the
//! size of every message pack together, and a consumer computing a Julian
//! day should not carry every Nepali yoga reading to do it. It is a pack
//! that is loaded (`docs/03-design/interpretation-records.md`).
//!
//! What it teaches:
//!
//! 1. **A pack is bytes.** `intl.load_pack` takes them from wherever you
//!    got them — a file beside your binary, a download, an asset in your
//!    application bundle. This example reads the files `teistro-intl build`
//!    wrote from the SDK's own source root, as every binding's does.
//! 2. **Loading changes what a composer says**, not how it is called.
//!    `readings` asks the base locale whether it carries a reading of each
//!    rule: with the pack, the item is the locale's own reading; without
//!    it, the verse's cited words. The same code composes both.
//! 3. **A plan item is a sentence.** The record also holds the full passage
//!    and its named facets; those are read from the entity directly, which
//!    is one call on an engine you already have.
//!
//! ```sh
//! cargo run -p teistro-intl -- --root packs/readings build --out target/packs/readings
//! cargo run --release -p teistro --example readings
//! ```

#![expect(clippy::print_stdout, reason = "an example is a program that prints")]

use std::path::{Path, PathBuf};

use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
use teistro::{
    ChartRequest, Context, Ephemeris, Error, PlanRequest, RuleRequest, ShippedRules, UtcOffset,
};

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

    // ── The pack ───────────────────────────────────────────────────────
    // One pack a locale, because a Nepali application wants Nepali and its
    // fallback and not four languages' worth of prose.
    for file in packs_of("readings")? {
        let bytes = std::fs::read(&file)
            .map_err(|why| Error::invalid_arg(format!("{}: {why}", file.display())))?;
        // This is the call a consumer makes, whatever the bytes came from.
        let loaded = sdk.intl().load_pack(&bytes)?;
        println!(
            "loaded {:<12} {:>6} readings, {:>7} bytes",
            loaded.locale,
            loaded.entries,
            bytes.len()
        );
    }

    // ── A chart, and the rules it holds ────────────────────────────────
    let place = Place::new(
        Latitude::try_new(27.7172)?,
        Longitude::try_new(85.3240)?,
        Altitude::try_new(1400.0)?,
    );
    // The computed yogas are the set whose rules the corpus wrote readings
    // for; the nabhasas are the kernel's own and have none, which is the
    // gap `interpret-measured.md` counts.
    let set = RuleRequest::shipped([ShippedRules::Yogas, ShippedRules::Doshas]).rule_set()?;
    let read = sdk.chart().interpreted(
        &[JulianDay::literal(2_447_995.489_583_333_5)],
        &ChartRequest::at(place, UtcOffset::try_from_seconds(20_700)?),
        Some(&set),
        PlanRequest::default().with_readings(),
    )?;
    let plan = read
        .value
        .into_iter()
        .next()
        .and_then(|chart| chart.plans.readings)
        .ok_or_else(|| Error::internal("one chart asked for its readings"))?;

    // ── The plan, said twice ───────────────────────────────────────────
    println!("\n{} items", plan.len());
    for locale in ["en-Latn", "ne-Deva-NP"] {
        sdk.intl().set_locale(locale)?;
        println!("\n{locale}");
        for item in &plan {
            let said = sdk.intl().render(&item.key, &item.params);
            // A fallback would mean this locale had no reading of its own,
            // which is exactly what an example must not hide.
            let mark = if said.is_fallback { "  (fallback)" } else { "" };
            println!("  {}{mark}", said.text);
        }
    }

    // ── The passage, which the plan does not carry ─────────────────────
    // A plan item is a sentence. The record holds the essay and its named
    // facets beside it, for a page that wants them.
    if let Some(item) = plan
        .items
        .iter()
        .find(|item| item.key == "sdk.reading.says")
        && let Some(teistro::Value::Entity(key)) = item.params.get("reading")
    {
        let record = sdk.intl().entity(key)?;
        println!("\n{key}");
        for form in ["prose", "career", "mind", "spirituality"] {
            if let Some(text) = record.form(form) {
                println!("  {form:<14} {}", first_sentence(text));
            }
        }
    }
    Ok(())
}

/// Enough of a passage to show it is there, without printing an essay.
fn first_sentence(text: &str) -> String {
    let mut out: String = text.chars().take(96).collect();
    if text.chars().count() > 96 {
        out.push('…');
    }
    out
}
