//! The five limbs of a day, computed from `positions` alone.
//!
//! A panchanga is the Hindu almanac's five parts — tithi, vara,
//! nakshatra, yoga and karana — and every one of them except the weekday
//! is a function of **two longitudes**: the Sun's and the Moon's, in the
//! sidereal zodiac. So a consumer can compute a whole panchanga from
//! `positions` and `weekday_of`, without any part of the chart layer.
//!
//! The arithmetic is the SDK's own (`crates/panchanga/src/limb.rs`):
//!
//! | limb      | from       | divisions       |
//! |---|---|---|
//! | tithi     | Moon − Sun | 30 of 12°       |
//! | karana    | Moon − Sun | 60 of 6°        |
//! | nakshatra | Moon       | 27 of 360/27°   |
//! | yoga      | Moon + Sun | 27 of 360/27°   |
//! | vara      | the weekday| 7               |
//!
//! The karana is the one that is not a plain division: sixty half-tithis
//! make a lunar month and they are **not** a cycle of eleven. Kimstughna
//! opens the month, Shakuni, Chatushpada and Naga close it, and the
//! seven movable karanas repeat through everything between. `karana_of`
//! below is the SDK's rule, transcribed, and the SDK holds it to the
//! corpus's own successor relation over 109 consecutive pairs.
//!
//! What this example is honest about: a limb here is the one holding at
//! the instant asked for. A printed almanac gives the limb at sunrise
//! and the time it ends, which is a boundary search over the Moon's
//! motion — that is `almanac.rs`, and it is what `sdk.almanac()` does.
//!
//! ```sh
//! cargo run --release -p teistro --example panchanga
//! ```

#![expect(clippy::print_stdout, reason = "an example is a program that prints")]

use teistro::catalogue::{Ayanamsha, Calendar, Karana, Nakshatra, Rashi, Tithi, Vara, Yoga};
use teistro::{
    Body, CalendarDate, CivilDateTime, CivilTime, Context, Ephemeris, Error, Frame,
    PositionRequest, TimeScale, Zodiac, ZoneSpec,
};

const NAKSHATRA_DEG: f64 = 360.0 / 27.0;
const YOGA_DEG: f64 = 360.0 / 27.0;
const TITHI_DEG: f64 = 12.0;
const KARANA_DEG: f64 = 6.0;

/// An angle brought into `0..360`.
fn turn(degrees: f64) -> f64 {
    degrees.rem_euclid(360.0)
}

/// Which division of `size` an angle falls in.
fn division(degrees: f64, size: f64) -> u16 {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "an angle under 360 over a positive division is under 360"
    )]
    let index = (turn(degrees) / size).floor() as u16;
    index
}

/// The karana that a half-tithi of the lunar month is.
///
/// Transcribed from `crates/panchanga/src/limb.rs`. Sixty of them make a
/// month: Kimstughna opens it, Shakuni, Chatushpada and Naga close it,
/// and the seven movable karanas fill everything between, Bava first
/// from the second half of the first tithi.
fn karana_of(half_tithi: u16) -> Option<Karana> {
    match half_tithi % 60 {
        0 => Some(Karana::Kimstughna),
        57 => Some(Karana::Shakuni),
        58 => Some(Karana::Chatushpada),
        59 => Some(Karana::Naga),
        half => Karana::from_id((half - 1) % 7),
    }
}

/// The five limbs at an instant, each a catalogue member rather than a
/// string — so a limb that does not exist cannot be constructed.
struct Limbs {
    sun: f64,
    moon: f64,
    elongation: f64,
    tithi: Tithi,
    vara: Vara,
    nakshatra: Nakshatra,
    yoga: Yoga,
    karana: Karana,
}

impl Limbs {
    /// Shukla while the Moon is gaining on the Sun, krishna after.
    const fn paksha(&self) -> &'static str {
        if self.elongation < 180.0 {
            "shukla"
        } else {
            "krishna"
        }
    }

    /// How much of the tithi has run at this instant, 0 to 1.
    fn tithi_elapsed(&self) -> f64 {
        (self.elongation % TITHI_DEG) / TITHI_DEG
    }
}

/// The limbs at an instant.
///
/// `weekday` is the ISO weekday of the **civil day** the instant belongs
/// to, which the caller has because it asked the calendar for it: a vara
/// is a property of the day, not of the moment.
fn limbs_at(sdk: &Context, instant: f64, weekday: u8) -> Result<Limbs, Error> {
    // The canonical frame with the zodiac changed. Everything else --
    // the centre, the corrections, the equinox -- is left as the SDK
    // computes it, so this asks for "what you would give me, but
    // sidereal".
    let frame = Frame::CANONICAL.with_zodiac(Zodiac::sidereal(Ayanamsha::Lahiri));
    let jds = [instant];
    let bodies = [Body::Sun, Body::Moon];
    let sky = sdk.positions(&PositionRequest::new(&jds, TimeScale::Ut1, &bodies, frame))?;
    let missing = || Error::internal("a grid of one instant by two bodies has both cells");
    let sun = sky.columns.at(0, 0).ok_or_else(missing)?.lon;
    let moon = sky.columns.at(0, 1).ok_or_else(missing)?.lon;
    let elongation = turn(moon - sun);
    let unknown = |what: &'static str| move || Error::internal(what);
    Ok(Limbs {
        sun,
        moon,
        elongation,
        tithi: Tithi::from_id(division(elongation, TITHI_DEG))
            .ok_or_else(unknown("an elongation under 360 is one of thirty tithis"))?,
        // `weekday_of` is ISO (Monday 1 … Sunday 7) and a vara counts
        // from Sunday, so the one becomes the other by `% 7`.
        vara: Vara::from_id(u16::from(weekday % 7))
            .ok_or_else(unknown("a weekday modulo seven is one of seven varas"))?,
        nakshatra: Nakshatra::from_id(division(moon, NAKSHATRA_DEG))
            .ok_or_else(unknown("a longitude under 360 is one of 27 nakshatras"))?,
        yoga: Yoga::from_id(division(moon + sun, YOGA_DEG))
            .ok_or_else(unknown("a sum under 360 is one of 27 yogas"))?,
        karana: karana_of(division(elongation, KARANA_DEG))
            .ok_or_else(unknown("a half-tithi is one of eleven karanas"))?,
    })
}

fn main() -> Result<(), Error> {
    let sdk = Context::builder()
        .profile("nepali-default")
        .locale("ne-Deva-NP")
        .ephemeris([Ephemeris::Builtin])
        .build()?;

    // Nepali New Year: the first day of Baisakh, BS 2082.
    let day = CalendarDate::defined(Calendar::BikramSambat, 2082, 1, 1);
    let gregorian = sdk.calendar().convert(&day, Calendar::Gregorian)?;
    // Six in the morning stands in for sunrise, which the almanac uses
    // and which needs the rise-and-set solver.
    let when = sdk.time().resolve(
        &CivilDateTime::at(day.clone(), CivilTime::new(6, 0, 0)?),
        &ZoneSpec::Iana {
            zone: String::from("Asia/Kathmandu"),
        },
    )?;
    let weekday = sdk.calendar().weekday_of(&day)? as u8;
    let found = limbs_at(&sdk, when.instant.get(), weekday)?;

    println!(
        "BS {}-{:02}-{:02}  ({}-{:02}-{:02})  06:00 Kathmandu",
        day.year, day.month, day.day, gregorian.year, gregorian.month, gregorian.day,
    );
    println!(
        "  sun {:>8.4}°   moon {:>8.4}°   elongation {:>8.4}°",
        found.sun, found.moon, found.elongation,
    );
    println!();

    for (label, member) in [
        ("tithi", found.tithi.full_key()),
        ("vara", found.vara.full_key()),
        ("nakshatra", found.nakshatra.full_key()),
        ("yoga", found.yoga.full_key()),
        ("karana", found.karana.full_key()),
    ] {
        let entity = sdk.intl().entity(member)?;
        // A member's full key is `tithi.PURNIMA`; the bare key is the
        // tail, which is what the other bindings call `key`.
        let key = member.rsplit('.').next().unwrap_or(member);
        println!(
            "  {label:<10} {:<14} {:<18} ({key})",
            entity.name(),
            entity.form("iast").unwrap_or_default(),
        );
    }
    println!();
    println!("  paksha     {}", found.paksha());
    println!(
        "  tithi is   {:.1}% elapsed at this instant",
        found.tithi_elapsed() * 100.0,
    );

    // The Sun on this day is the reason the year turns: BS begins at the
    // **Mesha Sankranti**, the instant the Sun enters Aries, and the
    // year's first day is the civil day that instant is reckoned into.
    // So the interesting number is how far past the crossing this
    // moment is -- which is why the almanac's year-start is an instant
    // and not a date.
    let sign = Rashi::from_id(division(found.sun, 30.0))
        .ok_or_else(|| Error::internal("a longitude under 360 is one of twelve signs"))?;
    let into = found.sun % 30.0;
    println!(
        "  the Sun stands {:.4}° into {}, so the Mesha Sankranti is {:.1} hours past --",
        into,
        sdk.intl().entity(sign.full_key())?.name(),
        // The Sun moves about 0.9856° a day, so its distance into the
        // sign is a good estimate of how long ago it crossed. The
        // almanac reports the crossing itself (`sun.sankranti`) and does
        // not estimate.
        into / 0.9856 * 24.0,
    );
    println!("  which is what the new year is reckoned from, and why BS 2082 opens today");
    Ok(())
}
