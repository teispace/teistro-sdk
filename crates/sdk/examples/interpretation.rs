//! An interpretation: the same birth record, said in two languages.
//!
//! `chart_reading.rs` read the chart in full. This is what comes after: a
//! **composer** turns what was read into a narrative plan — an ordered list
//! of message keys and their slots — and the locale engine renders it. The
//! plan holds no words at all, which is why one plan says the same chart in
//! English and in Nepali without the composer knowing either language
//! (`docs/03-design/interpret-composers.md`).
//!
//! What it teaches:
//!
//! 1. **A plan is data.** Items are `(key, params)`; the words belong to the
//!    locale, and a consumer may reorder or drop items before rendering.
//! 2. **One plan, every locale.** The same plan is rendered twice here, and
//!    `Rendered` says which locale answered — so a consumer can prove the
//!    locale had the message rather than falling back to English.
//! 3. **A composer says what it can say.** The lagna stands in the chart and
//!    is in no item: these messages read a graha, and the lagna is a point.
//!    A composer that guessed would be worse than one that says nothing.
//! 4. **It is the reading that is interpreted**, not the ephemeris: the plan
//!    is composed from the document the SDK already answered with, so
//!    nothing is computed twice.
//!
//! ```sh
//! cargo run --release -p teistro --example interpretation
//! ```

#![expect(clippy::print_stdout, reason = "an example is a program that prints")]

use teistro::catalogue::Calendar;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{
    CalendarDate, ChartRequest, CivilDateTime, CivilTime, Context, Ephemeris, Error, Plan, ZoneSpec,
};

fn main() -> Result<(), Error> {
    let sdk = Context::builder()
        .profile("nepali-default")
        .ephemeris([Ephemeris::Builtin])
        .build()?;

    let born = CalendarDate::defined(Calendar::BikramSambat, 2042, 9, 17);
    let resolved = sdk.time().resolve(
        &CivilDateTime::at(born, CivilTime::new(0, 20, 0)?),
        &ZoneSpec::Iana {
            zone: String::from("Asia/Kathmandu"),
        },
    )?;
    let place = Place::new(
        Latitude::try_new(27.7172)?,
        Longitude::try_new(85.324)?,
        Altitude::try_new(1400.0)?,
    );

    // ── The reading a composer interprets ──────────────────────────────
    // `with_state` is what makes it interpretable: a rule and a composer
    // read what a graha *is*, and a document without it is refused rather
    // than read as neutral.
    let request = ChartRequest::at(place, resolved.zone.offset)
        .with_state()
        .with_shadbala()
        .with_houses()
        .with_aspects();
    let document = sdk.chart().reading(resolved.instant, &request)?.value;

    // ── The plan ───────────────────────────────────────────────────────
    // Seven composers, one plan: a report concatenates what it wants to say
    // in the order it wants to say it, which is what a flat plan is for.
    // Where a graha *stands* and what it *rules* are different facts, and
    // it takes two composers to say both.
    let mut plan: Plan = sdk.interpret().placements(&document)?;
    plan.items.extend(sdk.interpret().strength(&document)?);
    plan.items.extend(sdk.interpret().houses(&document)?);
    // `positions` says what `placements` rounds away, which is why it is a
    // composer of its own: a page picks the precision it wants.
    plan.items.extend(sdk.interpret().positions(&document)?);
    // The first composer whose messages were written for it: no locale
    // carried a word for a drishti until `sdk.aspect` was added.
    plan.items.extend(sdk.interpret().aspects(&document)?);
    // What a graha *is* where it stands, and which chara karaka it holds:
    // the six facts of a placement the two composers above round away.
    // Both read the same states `placements` does, so they cost no knob.
    plan.items.extend(sdk.interpret().conditions(&document)?);
    plan.items.extend(sdk.interpret().karakas(&document)?);
    println!("plan  {} items, {} keys", plan.len(), plan.keys().len());
    for key in plan.keys() {
        println!("  {key}");
    }

    // The same plan, said twice.
    for locale in ["en-Latn", "ne-Deva-NP"] {
        sdk.intl().set_locale(locale)?;
        println!("\n{locale}");
        for item in &plan {
            let said = sdk.intl().render(&item.key, &item.params);
            // A fallback would mean this locale had no message of its own,
            // and an example that hid that would teach the wrong thing.
            let mark = if said.is_fallback { " (fallback)" } else { "" };
            println!("  {}{mark}", said.text);
        }
    }

    // ── What it does not say ───────────────────────────────────────────
    let lagna = plan
        .items
        .iter()
        .any(|item| format!("{:?}", item.params).contains("LAGNA"));
    println!("\nthe lagna is in the plan: {lagna}");
    Ok(())
}
