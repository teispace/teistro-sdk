//! A birth chart: from a Nepali birth record to the nine grahas placed.
//!
//! This is the scenario the SDK exists for, and it is not one call. What
//! it takes, in order:
//!
//! 1. A birth record as people actually write one — a Bikram Sambat
//!    date, a local clock time, and a place.
//! 2. That civil time resolved to an **instant**, which needs the zone's
//!    history: Nepal was +05:30 until 1986 and +05:45 after, and the
//!    resolution says which rule it used and from which tzdb.
//! 3. The chart founded at that instant. `chart().found` does the step
//!    everyone gets wrong: the SDK's canonical frame is *tropical*,
//!    because that is what an ephemeris computes, and a Vedic chart
//!    wants the sidereal zodiac — so the founder applies the profile's
//!    ayanamsha and **stamps every step it applied**, which this example
//!    prints.
//! 4. Each longitude read as a rashi, a nakshatra and a pada, using the
//!    catalogue's own members and the locale's own names.
//!
//! What it does not need: an ephemeris of your own, a data file, a
//! network, or a second crate. `Ephemeris::Builtin` selects the one the
//! SDK carries, so every position below is a real sky and this file runs
//! anywhere the crate builds.
//!
//! ```sh
//! cargo run --release -p teistro --example birth_chart
//! ```

#![expect(clippy::print_stdout, reason = "an example is a program that prints")]

use teistro::catalogue::{Calendar, ChartKind, Nakshatra, Rashi};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{
    CalendarDate, CivilDateTime, CivilTime, Context, Ephemeris, Error, GrahaPosition, ZoneSpec,
};

/// A nakshatra is a twenty-seventh of the circle; a pada a quarter of
/// one.
const NAKSHATRA_DEG: f64 = 360.0 / 27.0;
const PADA_DEG: f64 = NAKSHATRA_DEG / 4.0;

/// The sign a longitude stands in, and how far into it.
fn rashi_of(longitude: f64) -> Option<(Rashi, f64)> {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a normalised longitude over thirty is 0 to 11"
    )]
    let index = (longitude / 30.0).floor() as u16;
    Rashi::from_id(index).map(|rashi| (rashi, longitude % 30.0))
}

/// The lunar mansion a longitude stands in, and which quarter of it,
/// 1 to 4.
fn nakshatra_of(longitude: f64) -> Option<(Nakshatra, u8)> {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a normalised longitude over 360/27 is 0 to 26"
    )]
    let index = (longitude / NAKSHATRA_DEG).floor() as u16;
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a remainder under 360/27 over a quarter of it is 0 to 3"
    )]
    let pada = ((longitude % NAKSHATRA_DEG) / PADA_DEG).floor() as u8 + 1;
    Nakshatra::from_id(index).map(|nakshatra| (nakshatra, pada))
}

/// One graha's row, in the reader's own language.
fn row(sdk: &Context, placed: &GrahaPosition) -> Result<String, Error> {
    let graha = sdk.intl().entity(placed.graha.full_key())?;
    let (rashi, degrees) = rashi_of(placed.longitude_deg)
        .ok_or_else(|| Error::internal("a normalised longitude has a sign"))?;
    let (nakshatra, pada) = nakshatra_of(placed.longitude_deg)
        .ok_or_else(|| Error::internal("a normalised longitude has a nakshatra"))?;
    Ok(format!(
        "{:<10} {:<2} {} {:<12} {:>8.4}°  {:<14} {pada}   {:>2}",
        graha.name(),
        graha.glyph.as_deref().unwrap_or(" "),
        // There is no retrograde flag at the boundary or here: a graha
        // is retrograde when its longitude is decreasing, which is what
        // the speed says. `is_retrograde` is that comparison, named.
        if placed.is_retrograde() { "℞" } else { " " },
        sdk.intl().entity(rashi.full_key())?.name(),
        degrees,
        sdk.intl().entity(nakshatra.full_key())?.name(),
        // Which bhava it is in, under the chart's placement system --
        // the question most of the tradition answers with "in the
        // seventh". The chalit is a different question, and `placement`
        // is its answer.
        placed.house.bhava,
    ))
}

/// The record, resolved: what a stored chart keeps about *when*.
fn when(sdk: &Context, born: &CalendarDate) -> Result<teistro::Resolved, Error> {
    let civil = CivilDateTime::at(born.clone(), CivilTime::new(0, 20, 0)?);
    sdk.time().resolve(
        &civil,
        &ZoneSpec::Iana {
            zone: String::from("Asia/Kathmandu"),
        },
    )
}

/// What the SDK does when the birth time is not recorded, which is the
/// commonest hard case in the field — and it does **not** pick one for
/// you.
///
/// What happens is the profile's `time.unknown_time` policy, and by
/// default there is none, so the call is refused with a hint naming the
/// choices.
fn with_no_time(born: &CalendarDate) -> Result<(), Error> {
    let zone = ZoneSpec::Iana {
        zone: String::from("Asia/Kathmandu"),
    };
    let no_time = CivilDateTime::date_only(born.clone());
    for policy in [None, Some("NOON"), Some("MIDNIGHT")] {
        let mut builder = Context::builder()
            .profile("nepali-default")
            .locale("ne-Deva-NP")
            .ephemeris([Ephemeris::Builtin]);
        if let Some(chosen) = policy {
            builder = builder.settings_json(format!(r#"{{"time":{{"unknown_time":"{chosen}"}}}}"#));
        }
        let scoped = builder.build()?;
        let label = policy.unwrap_or("refuse");
        match scoped.time().resolve(&no_time, &zone) {
            Ok(resolved) => println!(
                "{label:<9} JD {:.6}  time known {}  {}",
                resolved.instant.get(),
                resolved.zone.time_known,
                if resolved.zone.warnings.is_empty() {
                    String::from("(no warning)")
                } else {
                    resolved
                        .zone
                        .warnings
                        .iter()
                        .map(|warning| warning.key())
                        .collect::<Vec<_>>()
                        .join(", ")
                },
            ),
            Err(refusal) => {
                println!("{label:<9} {}", refusal.message);
                println!("          hint: {}", refusal.hint().unwrap_or_default());
            }
        }
    }
    // MIDNIGHT is refused for a different reason, and it is this
    // record's own: the clocks jumped at midnight on this very date, so
    // 00:00 never happened in Kathmandu. A chart cast on a guessed
    // midnight would have been cast on a time that does not exist.
    // NOON answers, and says so twice -- `time_known` is false and the
    // resolution carries a `time-unknown-fallback` warning -- so a
    // stored chart can never quietly claim a birth time it never had.
    Ok(())
}

fn main() -> Result<(), Error> {
    let sdk = Context::builder()
        .profile("nepali-default")
        .locale("ne-Deva-NP")
        .ephemeris([Ephemeris::Builtin])
        .build()?;

    // ── 1. The record, as it would be written on a form ────────────────
    let born = CalendarDate::defined(Calendar::BikramSambat, 2042, 9, 17);
    let gregorian = sdk.calendar().convert(&born, Calendar::Gregorian)?;
    println!(
        "born  BS {}-{:02}-{:02}  ({}-{:02}-{:02})  00:20  Kathmandu",
        born.year, born.month, born.day, gregorian.year, gregorian.month, gregorian.day,
    );

    // ── 2. The instant, with the zone's own history ────────────────────
    let resolved = when(&sdk, &born)?;
    let seconds = resolved.zone.offset.seconds();
    println!(
        "      JD {:.6} UTC   offset {}{:02}:{:02}   {:?} (tzdb {})",
        resolved.instant.get(),
        if seconds < 0 { '-' } else { '+' },
        seconds.abs() / 3600,
        (seconds.abs() % 3600) / 60,
        resolved.zone.source,
        resolved.zone.tzdb_version,
    );
    // This record sits on the day Nepal moved from +05:30 to +05:45,
    // which is why the zone's history matters and a fixed offset would
    // be wrong: the source above says the answer came from the embedded
    // database rather than from a guess.

    // ── 3. The chart ───────────────────────────────────────────────────
    let place = Place::new(
        Latitude::try_new(27.7172)?,
        Longitude::try_new(85.324)?,
        Altitude::try_new(1400.0)?,
    );
    let founded = sdk.chart().found(
        resolved.instant,
        &place,
        resolved.zone.offset,
        ChartKind::Natal,
    )?;
    let chart = &founded.value;

    println!();
    println!("graha         glyph  sign             deg  nakshatra      pada bhava");
    println!("{}", "-".repeat(70));
    for placed in &chart.grahas {
        println!("{}", row(&sdk, placed)?);
    }

    // ── 4. What the chart is measured in, and against ──────────────────
    println!();
    let (lagna, into) = rashi_of(chart.lagna_deg)
        .ok_or_else(|| Error::internal("a normalised lagna has a sign"))?;
    println!(
        "lagna          {:.4}° -- {} at {:.4}°, vara {}",
        chart.lagna_deg,
        sdk.intl().entity(lagna.full_key())?.name(),
        into,
        chart.day.day.vara.key(),
    );
    // `Option`, and it means what it says: a tropical chart has no
    // ayanamsha, not an ayanamsha of nought.
    println!(
        "ayanamsha      {:.6}° applied ({})",
        chart.zodiac.offset_deg,
        chart.zodiac.ayanamsha.as_ref().map_or_else(
            || String::from("tropical, none applied"),
            |choice| format!("{choice:?}")
        ),
    );
    println!("steps applied  {}", chart.steps.join(", "));
    // The provenance envelope is the chart's own, not the context's: it
    // stamps the settings, the provider, the time layer and a content
    // hash of the value. This is what a stored chart keeps in order to
    // say what computed it.
    println!(
        "settings hash  {}…  content {}…",
        &founded.provenance.settings_hash.to_string()[..16],
        &founded.provenance.content_hash.to_string()[..16],
    );

    // ── A birth with no recorded time ──────────────────────────────────
    println!();
    with_no_time(&born)
}
