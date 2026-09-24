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
//! 3. **A composer says what it can say, and a subject needs a message that
//!    reads it.** The lagna stands in every chart and was in no item for six
//!    composers, because the placement messages read a graha and the lagna
//!    is `point.LAGNA`. It is said now, through a message that reads a
//!    **point** — not by widening the one that reads a graha, and not by
//!    guessing.
//! 4. **It is the reading that is interpreted**, not the ephemeris: the plan
//!    is composed from the document the SDK already answered with, so
//!    nothing is computed twice. `sdk.chart().interpreted` founds the
//!    chart, reads the sections the composers need and composes the plans
//!    asked for in one call, which is the call every binding's `interpret`
//!    option makes.
//!
//! ```sh
//! cargo run --release -p teistro --example interpretation
//! ```

#![expect(clippy::print_stdout, reason = "an example is a program that prints")]

use teistro::catalogue::Calendar;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{
    CalendarDate, ChartRequest, CivilDateTime, CivilTime, Context, Ephemeris, Error, Item, Plan,
    PlanRequest, Plans, RuleRequest, ShippedRules, Value, ZoneSpec,
};

/// Whether any item's slots, written as JSON, mention `needle` -- the
/// question a reader asks of a plan before rendering it.
fn mentions(items: &[Item], needle: &str) -> bool {
    items
        .iter()
        .any(|item| serde_json::to_string(&item.params).is_ok_and(|json| json.contains(needle)))
}

/// The eight plans this example asks for, each as its items. Every plan
/// asked for is present, and one that had nothing to say is present and
/// empty, which is an answer.
struct Said {
    placements: Vec<Item>,
    readings: Vec<Item>,
    strength: Vec<Item>,
    houses: Vec<Item>,
    positions: Vec<Item>,
    aspects: Vec<Item>,
    conditions: Vec<Item>,
    karakas: Vec<Item>,
}

impl Said {
    fn of(plans: Plans) -> Said {
        let items = |plan: Option<Plan>| plan.map(|plan| plan.items).unwrap_or_default();
        Said {
            placements: items(plans.placements),
            readings: items(plans.readings),
            strength: items(plans.strength),
            houses: items(plans.houses),
            positions: items(plans.positions),
            aspects: items(plans.aspects),
            conditions: items(plans.conditions),
            karakas: items(plans.karakas),
        }
    }

    /// Every plan but the readings, in the order they are said.
    fn plain(&self) -> [&Vec<Item>; 7] {
        [
            &self.placements,
            &self.strength,
            &self.houses,
            &self.positions,
            &self.aspects,
            &self.conditions,
            &self.karakas,
        ]
    }
}

/// How long each plan is, and every key the plans say.
fn the_plan(said: &Said) {
    println!(
        "plan     {} placement items, {} reading items, {} strengths, {} lordships, \
         {} positions, {} drishtis, {} conditions, {} karakas",
        said.placements.len(),
        said.readings.len(),
        said.strength.len(),
        said.houses.len(),
        said.positions.len(),
        said.aspects.len(),
        said.conditions.len(),
        said.karakas.len(),
    );
    let [placements, rest @ ..] = said.plain();
    let mut keys: Vec<&str> = Vec::new();
    for item in placements
        .iter()
        .chain(&said.readings)
        .chain(rest.into_iter().flatten())
    {
        if !keys.contains(&item.key.as_str()) {
            keys.push(&item.key);
        }
    }
    println!("keys     {}", keys.join(", "));
}

/// The same plans, said in two locales.
fn say(sdk: &Context, said: &Said) -> Result<(), Error> {
    for locale in ["en-Latn", "ne-Deva-NP"] {
        sdk.intl().set_locale(locale)?;
        println!("\n{locale}");
        for item in said.plain().into_iter().flatten() {
            let rendered = sdk.intl().render(&item.key, &item.params);
            // A fallback would mean this locale had no message of its own,
            // and an example that hid that would teach the wrong thing.
            let mark = if rendered.is_fallback {
                "  (fallback)"
            } else {
                ""
            };
            println!("  {}{mark}", rendered.text);
        }
        // A reading names its rule in a slot the message does not print,
        // so a consumer can group a plan by rule. Here it prefixes the line.
        for item in &said.readings {
            let rendered = sdk.intl().render(&item.key, &item.params);
            let rule = match item.params.get("rule") {
                Some(Value::Str(rule)) => rule.as_str(),
                _ => "",
            };
            let mark = if rendered.is_fallback {
                "  (fallback)"
            } else {
                ""
            };
            println!("  {rule}: {}{mark}", rendered.text);
        }
    }
    Ok(())
}

/// What the plans claim, and what they do not.
fn the_claims(said: &Said) {
    println!(
        "\nthe lagna is in the placements: {}",
        mentions(&said.placements, "LAGNA")
    );
    println!(
        "a strength item claims \"strong\": {}",
        mentions(&said.strength, "strong")
    );
    println!(
        "a houses item claims a sign: {}",
        mentions(&said.houses, "rashi")
    );
    println!(
        "a position item carries a rendered angle: {}",
        mentions(&said.positions, "\u{b0}")
    );
}

fn main() -> Result<(), Error> {
    let sdk = Context::builder()
        .profile("nepali-default")
        .locale("en-Latn")
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
    let request = ChartRequest::at(place, resolved.zone.offset);

    // ── One call: the chart, what its rules answer, and what it says ──
    // The rules whose answers the readings composer will say. The sections
    // the composers read are computed whether or not the request named
    // them.
    let rules =
        RuleRequest::shipped([ShippedRules::Nabhasas, ShippedRules::Arishtas]).rule_set()?;
    let asked = PlanRequest::default()
        .with_placements()
        .with_readings()
        .with_strength()
        .with_houses()
        .with_positions()
        .with_aspects()
        .with_conditions()
        .with_karakas();
    let charts = sdk
        .chart()
        .interpreted(&[resolved.instant], &request, Some(&rules), asked)?;
    let chart = charts
        .value
        .into_iter()
        .next()
        .ok_or_else(|| Error::internal("one instant founded one chart"))?;
    let said = Said::of(chart.plans);

    println!("BS 2042-09-17  00:20  Kathmandu");
    the_plan(&said);
    say(&sdk, &said)?;
    the_claims(&said);

    // `readings` says what rules answered, so asking for it without rules
    // is refused rather than answered with an empty plan.
    let without = sdk.chart().interpreted(
        &[resolved.instant],
        &request,
        None,
        PlanRequest::default().with_readings(),
    );
    if let Err(refusal) = without {
        println!(
            "refused  {}: {}",
            refusal.field().unwrap_or_default(),
            refusal.message
        );
        println!("hint     {}", refusal.hint().unwrap_or_default());
    }
    Ok(())
}
