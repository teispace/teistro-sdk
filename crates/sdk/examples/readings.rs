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
//!    application bundle. This example builds them from the SDK's own
//!    source root so it has no file to fetch; a shipped consumer reads a
//!    `.tpack` and passes the bytes to the same call.
//! 2. **Loading changes what a composer says**, not how it is called.
//!    `readings` asks the base locale whether it carries a reading of each
//!    rule: with the pack, the item is the locale's own reading; without
//!    it, the verse's cited words. The same code composes both.
//! 3. **A plan item is a sentence.** The record also holds the full passage
//!    and its named facets; those are read from the entity directly, which
//!    is one call on an engine you already have.
//!
//! ```sh
//! cargo run --release -p teistro --example readings
//! ```

#![expect(clippy::print_stdout, reason = "an example is a program that prints")]

use std::path::Path;

use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
use teistro::{
    ChartRequest, Context, Ephemeris, Error, RuleRequest, ShippedRules, Tree, UtcOffset, pack,
};

/// Where the SDK keeps the readings' sources. A consumer ships the built
/// `.tpack` instead and never sees this directory.
const READINGS: &str = "packs/readings";

fn main() -> Result<(), Error> {
    let sdk = Context::builder()
        .profile("nepali-default")
        .ephemeris([Ephemeris::Builtin])
        .build()?;

    // ── The pack ───────────────────────────────────────────────────────
    // One pack a locale, because a Nepali application wants Nepali and its
    // fallback and not four languages' worth of prose.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(READINGS);
    let tree = Tree::load(&root).map_err(|why| Error::invalid_arg(why.to_string()))?;
    for locale in tree.locales.values() {
        let bytes =
            pack::build(locale, "sdk.entity").map_err(|why| Error::invalid_arg(why.to_string()))?;
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
    let request =
        ChartRequest::at(place, UtcOffset::try_from_seconds(20_700)?).with_rule_inputs(set.rules());
    let reading = sdk.chart().readings_with_rules(
        &[JulianDay::literal(2_447_995.489_583_333_5)],
        &request,
        &set,
    )?;
    let Some((_, held)) = reading.value.first() else {
        println!("\nno chart was read");
        return Ok(());
    };

    // ── The plan, said twice ───────────────────────────────────────────
    let plan = sdk.interpret().readings(held);
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
