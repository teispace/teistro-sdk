//! The smallest thing that works, kept honest: `cargo xtask check-rust`
//! runs this file, so the README cannot drift from what the crate does.
//!
//! ```sh
//! cargo run --release -p teistro --example quickstart
//! ```
//!
//! **A Rust consumer composes the crates; it does not cross a C ABI.**
//! There is no handle to open, no blob to decode and no `dispose`: a
//! `Context` is a value, an area is a borrowing view of it, and `Drop`
//! frees what it held (`03-design/rust-consumer-surface.md` §3).

#![expect(clippy::print_stdout, reason = "an example is a program that prints")]

use teistro::catalogue::{Calendar, Graha};
use teistro::{
    Body, CalendarDate, CivilDateTime, CivilTime, Context, Ephemeris, Error, Frame,
    PositionRequest, TimeScale, ZoneSpec, messages,
};

fn main() -> Result<(), Error> {
    // One value, built once. The chain is ordered: the first entry that
    // opens is the one that answers, so a real engine first and the
    // built-in behind it is the shape a consumer wants.
    let sdk = Context::builder()
        .profile("nepali-default")
        .locale("ne-Deva-NP")
        .ephemeris([Ephemeris::Builtin])
        .build()?;

    // The version is **Cargo's**, not a call: a Rust consumer's
    // dependency graph already fixed it, so there is no `sdkVersion()`
    // here and no `abiVersion()` at all (§6). What a stored result keeps
    // is on the provenance envelope, which `chart` and `almanac` carry.
    println!("Teistro {}", env!("CARGO_PKG_VERSION"));

    // 14 April 2015 is 1 Baisakh 2072 BS.
    let day = CalendarDate::defined(Calendar::Gregorian, 2015, 4, 14);
    let bs = sdk.calendar().convert(&day, Calendar::BikramSambat)?;
    println!(
        "{}-{}-{} {}",
        bs.year,
        bs.month,
        bs.day,
        bs.era
            .map_or_else(String::new, |era| era.era.full_key().to_owned())
    );

    // A Kathmandu birth time, with the metadata a stored chart keeps.
    let civil = CivilDateTime::at(
        CalendarDate::defined(Calendar::Gregorian, 1986, 1, 1),
        CivilTime::new(0, 20, 0)?,
    );
    let zone = ZoneSpec::Iana {
        zone: String::from("Asia/Kathmandu"),
    };
    let when = sdk.time().resolve(&civil, &zone)?;
    println!(
        "JD {:.6} UTC, {} s, tzdb {}",
        when.instant.get(),
        when.zone.offset.seconds(),
        when.zone.tzdb_version,
    );

    // The Sun and the Moon at J2000, in the SDK's canonical frame. One
    // call for the whole grid, and the answer is the astronomy crate's
    // own `Completed` -- typed columns, not a blob to decode.
    let jds = [2_451_545.0];
    let bodies = [Body::Sun, Body::Moon];
    let sky = sdk.positions(&PositionRequest::new(
        &jds,
        TimeScale::Ut1,
        &bodies,
        Frame::CANONICAL,
    ))?;
    let sun = sky
        .columns
        .at(0, 0)
        .ok_or_else(|| Error::internal("a grid of one has a first cell"))?;
    println!("the Sun at {:.4} degrees", sun.lon);

    // A message in the context's locale, by its typed accessor: the key
    // is spelled once, in the generator, and the parameters are typed.
    // Where Node writes `ctx.intl.messages.sdk.reason.grahaInBhava({…})`,
    // Rust reads a module tree, because that is what a namespace is here.
    let said = sdk
        .intl()
        .render_typed(&messages::sdk::reason::GrahaInBhava {
            graha: Graha::Jupiter,
            bhava: 7,
        });
    println!("{}", said.text);
    let sun_entity = sdk.intl().entity("graha.SUN")?;
    println!(
        "{} {}",
        sun_entity.name(),
        sun_entity.glyph.as_deref().unwrap_or_default()
    );

    // No `dispose`: the context is dropped here, and with it the locale
    // engine, the ephemeris and everything either of them held.
    Ok(())
}
