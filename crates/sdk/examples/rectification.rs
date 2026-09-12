//! Rectification: a birth time known only to the hour, narrowed by
//! lagna.
//!
//! A birth record that says "some time before dawn" is the commonest
//! hard case in the field. What narrows it is the **lagna** — it moves
//! through all twelve signs in a day, so it changes sign every couple of
//! hours, and a family that remembers the ascendant remembers something
//! the clock does not.
//!
//! The point of this example is the shape of the call. A rectification
//! pass wants many charts at one place, and `found_many` founds them in
//! **one crossing**: the settings are resolved once, the solar model is
//! built once, and the day each instant belongs to is reckoned against
//! the same sunrise. Founding them one at a time would give the same
//! numbers and pay the setup for every one of them.
//!
//! `nepali-default` is what a Nepali birth record is cast under, and its
//! frame is **topocentric**: the chart is seen from the hill the record
//! was written on rather than from the centre of the Earth. The
//! completion does that step itself over any provider
//! (`03-design/topocentric-measured.md`), so the analytic ephemeris
//! below is enough, and `steps applied` names it.
//!
//! ```sh
//! cargo run --release -p teistro --example rectification
//! ```

#![expect(clippy::print_stdout, reason = "an example is a program that prints")]

use teistro::catalogue::{Calendar, ChartKind, Graha, Rashi};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{CalendarDate, CivilDateTime, CivilTime, Context, Ephemeris, Error, ZoneSpec};

/// The window the record leaves open: midnight to three, every ten
/// minutes.
const FROM_HOUR: u32 = 0;
const TO_HOUR: u32 = 3;
const EVERY_MINUTES: u32 = 10;

/// The sign a longitude stands in.
fn sign_of(longitude: f64) -> Option<Rashi> {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a normalised longitude over thirty is 0 to 11"
    )]
    let index = (longitude.rem_euclid(360.0) / 30.0).floor() as u16;
    Rashi::from_id(index)
}

/// The local clock reading of the `index`th candidate.
fn clock(index: usize) -> String {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "an index under a hundred fits a u32"
    )]
    let minutes = FROM_HOUR * 60 + index as u32 * EVERY_MINUTES;
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}

/// The candidate instants: one resolution fixes the zone and the offset,
/// and the rest is arithmetic on the instant — which is what a Julian
/// day is for.
fn candidates(start: JulianDay<Utc>) -> Vec<JulianDay<Utc>> {
    let step = f64::from(EVERY_MINUTES) / (24.0 * 60.0);
    let count = ((TO_HOUR - FROM_HOUR) * 60 / EVERY_MINUTES) as usize;
    (0..count)
        .map(|i| {
            #[expect(
                clippy::cast_precision_loss,
                reason = "a candidate index under a hundred is exact in an f64"
            )]
            let at = i as f64;
            JulianDay::<Utc>::literal(start.get() + at * step)
        })
        .collect()
}

/// The table: every candidate's lagna, and where it changes sign.
fn the_table(sdk: &Context, charts: &[teistro::ChartFoundation]) -> Result<(), Error> {
    println!("local   lagna        sign            moon         bhava");
    println!("{}", "-".repeat(58));
    let mut previous: Option<Rashi> = None;
    for (index, chart) in charts.iter().enumerate() {
        let sign = sign_of(chart.lagna_deg)
            .ok_or_else(|| Error::internal("a normalised lagna has a sign"))?;
        // `graha` is a lookup by member, not by position: the
        // catalogue's order is the catalogue's business.
        let moon = chart.graha(Graha::Moon);
        println!(
            "{}   {:>9.4}°  {:<14} {:>9.4}°  {:>2}{}",
            clock(index),
            chart.lagna_deg,
            sdk.intl().entity(sign.full_key())?.name(),
            moon.map_or(0.0, |placed| placed.longitude_deg),
            moon.map_or(0, |placed| placed.house.bhava),
            if previous.is_some_and(|before| before != sign) {
                "   <- lagna changes sign"
            } else {
                ""
            },
        );
        previous = Some(sign);
    }
    Ok(())
}

fn main() -> Result<(), Error> {
    let sdk = Context::builder()
        .profile("nepali-default")
        .locale("ne-Deva-NP")
        .ephemeris([Ephemeris::Builtin])
        .build()?;

    // The record: a Bikram Sambat date, a place, and an hour nobody is
    // sure of. Everything below narrows the last of those.
    let born = CalendarDate::defined(Calendar::BikramSambat, 2045, 9, 17);
    let place = Place::new(
        Latitude::try_new(27.7172)?,
        Longitude::try_new(85.324)?,
        Altitude::try_new(1400.0)?,
    );
    #[expect(
        clippy::cast_possible_truncation,
        reason = "an hour of the day fits a u8"
    )]
    let start = sdk.time().resolve(
        &CivilDateTime::at(born.clone(), CivilTime::new(FROM_HOUR as u8, 0, 0)?),
        &ZoneSpec::Iana {
            zone: String::from("Asia/Kathmandu"),
        },
    )?;
    let instants = candidates(start.instant);

    // ── One crossing for every candidate ───────────────────────────────
    let founded = sdk
        .chart()
        .found_many(&instants, &place, start.zone.offset, ChartKind::Natal)?;
    let charts = &founded.value;

    println!(
        "{} candidate charts, {EVERY_MINUTES} minutes apart, in one crossing",
        charts.len(),
    );
    println!(
        "place  {:.4}°N {:.4}°E   {:?}",
        place.latitude.get(),
        place.longitude.get(),
        ChartKind::Natal,
    );
    println!();
    the_table(&sdk, charts)?;

    // ── What the batch shares, and what it does not ────────────────────
    // The place, the settings, the solar model and the completion steps
    // are one to a batch: they are what "the same chart at a different
    // minute" holds constant. The instant, the lagna, the day and the
    // timing are per chart. The provenance envelope stamps the batch as
    // a whole, so a rectification run reproduces as one thing.
    println!();
    println!(
        "steps applied  {}",
        charts
            .first()
            .map(|chart| chart.steps.join(", "))
            .unwrap_or_default(),
    );
    println!(
        "provenance     profile {}, provider {}, settings {}…",
        founded.provenance.profile,
        founded.provenance.provider.name,
        &founded.provenance.settings_hash.to_string()[..16],
    );

    // A batch of one is the ordinary case, and `found` is the same
    // crossing with the batch unwrapped: the answer is a chart, not a
    // list of one.
    let single = sdk
        .chart()
        .found(start.instant, &place, start.zone.offset, ChartKind::Natal)?;
    println!();
    println!(
        "found(one)     lagna {:.4}°  vara {}  ishtakaal {}:{}:{}",
        single.value.lagna_deg,
        single.value.day.day.vara.key(),
        single.value.timing.ishtakaal.ghati,
        single.value.timing.ishtakaal.pala,
        single.value.timing.ishtakaal.vipala,
    );
    println!(
        "               and it agrees with the batch of one bit for bit: {}",
        charts
            .first()
            .is_some_and(|first| first.lagna_deg.to_bits() == single.value.lagna_deg.to_bits()),
    );
    Ok(())
}
