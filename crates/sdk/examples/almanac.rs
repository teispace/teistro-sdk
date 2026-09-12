//! A week's panchangam: the five limbs of each day, and its periods.
//!
//! A chart is consulted once; a panchanga every morning. This is the
//! page a Nepali or Indian almanac prints, and the SDK computes it in
//! **one crossing** for the whole week — consecutive days share a
//! boundary, so day *n*'s next sunrise is day *n+1*'s sunrise, and
//! asking for seven days costs much less than seven days asked for
//! separately.
//!
//! What the shape teaches, and what a reader should copy:
//!
//! * A limb is a **span**, not a name. "Today's tithi" is a question
//!   with two answers on most days, and the SDK gives both with the
//!   instant each gives way — which is what an almanac row prints.
//! * A span carries its **own** bounds as well as the clipped ones, so
//!   "the tithi began yesterday at 21:05" is a fact you can print.
//! * A value a day may not have is `Option`, never a sentinel: no
//!   sankranti is `None`, not Julian day zero. In Rust the compiler
//!   makes that unignorable, which is the one place this surface is
//!   stricter than the other three rather than merely different.
//!
//! ```sh
//! cargo run --release -p teistro --example almanac
//! ```

#![expect(clippy::print_stdout, reason = "an example is a program that prints")]

use teistro::catalogue::Calendar;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{CalendarDate, Context, Ephemeris, Error, Interval, Panchanga, UtcOffset};

/// The offset the request is made under: Nepal's +05:45.
const OFFSET_SECONDS: i32 = 20700;

/// A Julian day as the local clock reads it, which is what an almanac
/// prints.
fn clock(at: JulianDay<Utc>) -> String {
    let local = (at.get() + f64::from(OFFSET_SECONDS) / 86400.0 + 0.5).rem_euclid(1.0);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a fraction of a day in minutes is under 1440"
    )]
    let minutes = (local * 1440.0).round() as u32 % 1440;
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}

/// An entity's name in the reader's language, or the key made readable.
///
/// A locale pack names most of the catalogue and not all of it: `masa`
/// and `direction` have no entries in any of the five the SDK ships, so
/// an almanac falls back to the key rather than refusing to print. A
/// program that must have the name in the reader's language should ask
/// `has` first and say so, rather than showing a bare key.
fn name(sdk: &Context, key: &str) -> String {
    sdk.intl().entity(key).map_or_else(
        |_| {
            key.split_once('.')
                .map_or(key, |(_, tail)| tail)
                .to_lowercase()
                .replace('_', " ")
        },
        |entity| entity.name().to_owned(),
    )
}

/// One span of one limb: the member's key, its own bounds, and the part
/// of it inside the day.
type LimbSpan = (&'static str, Interval, Interval);

/// One limb's spans as an almanac row prints them.
///
/// `whole` is the member's own span and `inside` the clipped one, so a
/// member that began yesterday says so with a `<` rather than looking as
/// though it began at sunrise.
fn spans(sdk: &Context, printed: &[LimbSpan]) -> String {
    printed
        .iter()
        .map(|(member, whole, inside)| {
            format!(
                "{}{} until {}{}",
                if whole.from.get() < inside.from.get() {
                    '<'
                } else {
                    ' '
                },
                name(sdk, member),
                clock(inside.to),
                if whole.to.get() > inside.to.get() {
                    '>'
                } else {
                    ' '
                },
            )
        })
        .collect::<Vec<_>>()
        .join("  ")
}

/// The four moving limbs of a day, flattened so they can be walked as
/// one: the member's key, its own span, and the part inside the day.
fn limbs(day: &Panchanga) -> [(&'static str, Vec<LimbSpan>); 4] {
    [
        (
            "tithi",
            day.limbs
                .tithi
                .iter()
                .map(|span| (span.member.full_key(), span.whole, span.inside))
                .collect(),
        ),
        (
            "nakshatra",
            day.limbs
                .nakshatra
                .iter()
                .map(|span| (span.member.full_key(), span.whole, span.inside))
                .collect(),
        ),
        (
            "yoga",
            day.limbs
                .yoga
                .iter()
                .map(|span| (span.member.full_key(), span.whole, span.inside))
                .collect(),
        ),
        (
            "karana",
            day.limbs
                .karana
                .iter()
                .map(|span| (span.member.full_key(), span.whole, span.inside))
                .collect(),
        ),
    ]
}

/// One day of the page.
fn print_day(sdk: &Context, day: &Panchanga) {
    let local = &day.day;
    println!(
        "{:<12} {}-{:02}-{:02}   sunrise {}  sunset {}   {} {}",
        name(sdk, local.vara.full_key()),
        local.date.year,
        local.date.month,
        local.date.day,
        clock(local.sunrise),
        clock(local.sunset),
        name(sdk, day.month.amanta.full_key()),
        name(sdk, day.month.paksha.full_key()),
    );
    // The five limbs. The vara is one of them and is the day's own; the
    // other four are spans, and a day usually has two of each.
    for (limb, printed) in limbs(day) {
        println!("  {limb:<10} {}", spans(sdk, &printed));
    }
    // The periods a day is planned around. Rahu kalam is the one
    // everybody checks; the choghadiya are what a shop opens on.
    println!(
        "  {:<10} {}",
        "kaala",
        day.kaalas
            .iter()
            .map(|kaala| format!(
                "{} {}-{}",
                name(sdk, kaala.kaala.full_key()),
                clock(kaala.at.from),
                clock(kaala.at.to)
            ))
            .collect::<Vec<_>>()
            .join("  "),
    );
    println!(
        "  {:<10} {} …",
        "choghadiya",
        day.choghadiya
            .iter()
            .filter(|part| part.is_daytime)
            .take(3)
            .map(|part| format!(
                "{} {}",
                name(sdk, part.choghadiya.full_key()),
                clock(part.at.from)
            ))
            .collect::<Vec<_>>()
            .join("  "),
    );
    // Absent is absent, and the type says so: `if let Some` is the only
    // way to read one of these, so a sentinel cannot be mistaken for a
    // value.
    if let Some(abhijit) = day.muhurtas.abhijit {
        println!(
            "  {:<10} {}-{}{}",
            "abhijit",
            clock(abhijit.from),
            clock(abhijit.to),
            if day.muhurtas.abhijit_effective {
                ""
            } else {
                "  (not effective on a Wednesday)"
            },
        );
    }
    if let Some(sankranti) = day.sun.sankranti {
        println!(
            "  {:<10} the Sun enters a new sign at {}",
            "sankranti",
            clock(sankranti),
        );
    }
    let moon: Vec<String> = day
        .moon
        .rises
        .iter()
        .map(|at| format!("rise {}", clock(*at)))
        .chain(day.moon.sets.iter().map(|at| format!("set {}", clock(*at))))
        .collect();
    println!(
        "  {:<10} {}",
        "moon",
        if moon.is_empty() {
            String::from("(neither rise nor set inside the window)")
        } else {
            moon.join("  ")
        },
    );
    println!();
}

fn main() -> Result<(), Error> {
    let sdk = Context::builder()
        .profile("parashari-classical")
        .locale("ne-Deva-NP")
        .ephemeris([Ephemeris::Builtin])
        .build()?;
    let place = Place::new(
        Latitude::try_new(27.7172)?,
        Longitude::try_new(85.324)?,
        Altitude::try_new(1400.0)?,
    );
    let offset = UtcOffset::try_from_seconds(OFFSET_SECONDS)?;

    // ── One crossing for the whole week ───────────────────────────────
    let from = CalendarDate::defined(Calendar::Gregorian, 2024, 6, 17);
    let to = CalendarDate::defined(Calendar::Gregorian, 2024, 6, 23);
    let week = sdk.almanac().of(&from, &to, &place, offset)?;

    println!(
        "{} days at {:.4}°N {:.4}°E, one crossing",
        week.value.len(),
        place.latitude.get(),
        place.longitude.get(),
    );
    println!(
        "calendar {}   provider {}",
        from.calendar.full_key(),
        week.provenance.provider.name,
    );
    println!();

    for day in &week.value {
        print_day(&sdk, day);
    }

    // ── What the boundary shared, which is the point ──────────────────
    // Day *n*'s next sunrise is day *n+1*'s sunrise, computed once. That
    // is why a week is one crossing and not seven.
    let shared = week.value.windows(2).all(|pair| match pair {
        [earlier, later] => {
            earlier.day.next_sunrise.get().to_bits() == later.day.sunrise.get().to_bits()
        }
        _ => false,
    });
    println!("consecutive days share a boundary, bit for bit: {shared}");
    let karanas: usize = week.value.iter().map(|day| day.limbs.karana.len()).sum();
    println!(
        "{karanas} karanas across {} days, from one crossing",
        week.value.len(),
    );
    println!(
        "content hash   {}…  -- of the week as a value, so a stored page can be checked",
        &week.provenance.content_hash.to_string()[..16],
    );

    // A day on its own is the range of one unwrapped: same crossing, and
    // the answer is a day rather than a list of one.
    let one = sdk.almanac().day(
        &CalendarDate::defined(Calendar::Gregorian, 2024, 6, 21),
        &place,
        offset,
    )?;
    println!(
        "almanac().day  {}  {} horas, {} muhurtas, {} choghadiya",
        name(&sdk, one.value.day.vara.full_key()),
        one.value.horas.len(),
        one.value.muhurtas.daylight.len() + one.value.muhurtas.night.len(),
        one.value.choghadiya.len(),
    );
    Ok(())
}
