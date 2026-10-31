//! Nepal's official days, and the modern reckoning beside them.
//!
//! Nepal's patro is computed by the Nepali committee from the Surya
//! Siddhanta: the tithi, nakshatra and yoga by the text with the
//! committee's bija, and the sunrise from a modern ephemeris. That is
//! what [`ContextBuilder::nepal_official`] opens. The modern reckoning,
//! `nepali-drik`, reads the same day from the modern sky, and the two
//! name a different sunrise tithi on a few days a month.
//!
//! What the shape teaches, and what a reader should copy:
//!
//! * The official day is a **preset**, not a recipe:
//!   `Context::builder().nepal_official([engine])`.
//! * The Surya Siddhanta takes the engine **after** it in the chain for
//!   its sunrise; an app passes its own, `Ephemeris::Provider(teimeris)`.
//!   This file passes none, so the sunrise is the built-in ephemeris's
//!   and it runs anywhere.
//! * Where the two disagree, the reason is a tithi's end against the
//!   sunrise: the tithi the modern sky still has at sunrise had already
//!   ended by the text's, or the other way round. Each row prints both
//!   ends.
//!
//! [`ContextBuilder::nepal_official`]: teistro::ContextBuilder::nepal_official
//!
//! ```sh
//! cargo run --release -p teistro --example nepal_official
//! ```

#![expect(clippy::print_stdout, reason = "an example is a program that prints")]

use teistro::catalogue::Calendar;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{CalendarDate, Context, Ephemeris, Error, UtcOffset};

/// The offset the request is made under: Nepal's +05:45.
const OFFSET_SECONDS: i32 = 20700;

/// A Julian day as the local clock reads it.
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

fn main() -> Result<(), Error> {
    let official = Context::builder()
        .nepal_official([])
        .locale("en-Latn")
        .build()?;
    let modern = Context::builder()
        .profile("nepali-drik")
        .locale("en-Latn")
        .ephemeris([Ephemeris::Builtin])
        .build()?;
    let name = |key: &str| {
        official
            .intl()
            .entity(key)
            .map(|entity| entity.name().to_owned())
    };
    let place = Place::new(
        Latitude::try_new(27.7172)?,
        Longitude::try_new(85.324)?,
        Altitude::try_new(1400.0)?,
    );
    let offset = UtcOffset::try_from_seconds(OFFSET_SECONDS)?;

    // The same month under each, one crossing each.
    let from = CalendarDate::defined(Calendar::Gregorian, 2025, 11, 1);
    let to = CalendarDate::defined(Calendar::Gregorian, 2025, 11, 30);
    let days = official.almanac().of(&from, &to, &place, offset)?.value;
    let others = modern.almanac().of(&from, &to, &place, offset)?.value;

    println!(
        "official {}, modern {}",
        official.profile(),
        modern.profile()
    );
    let mut differ = 0;
    for (day, other) in days.iter().zip(&others) {
        // The first span of a day's limb is the one standing at its
        // sunrise.
        let (Some(ours), Some(theirs)) = (day.limbs.tithi.first(), other.limbs.tithi.first())
        else {
            continue;
        };
        if ours.member == theirs.member {
            continue;
        }
        differ += 1;
        let date = &day.day.date;
        println!(
            "{}-{:02}-{:02}  sunrise {}  official {} until {}  modern {} until {}",
            date.year,
            date.month,
            date.day,
            clock(day.day.sunrise),
            name(ours.member.full_key())?,
            clock(ours.whole.to),
            name(theirs.member.full_key())?,
            clock(theirs.whole.to),
        );
    }
    println!("{differ} of {} days name another sunrise tithi", days.len());
    Ok(())
}
