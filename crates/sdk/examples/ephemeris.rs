//! A year of the sky in one call, and what to do with the columns.
//!
//! `positions` takes a **grid** — instants by bodies — and answers with
//! columns. That shape is the whole reason a year of positions costs one
//! crossing rather than 366, and it is what a service computing tables,
//! transits or ingresses should be using.
//!
//! Three things this example is really about:
//!
//! 1. **One call, not a loop.** 366 instants by 3 bodies is 1098 cells
//!    in a single request. Asking day by day would rebuild the
//!    provider's own setup 366 times.
//! 2. **The columns are the crates' own.** `sky.columns.lon` is a
//!    `Vec<f64>`, not a blob section to decode and not a vector of
//!    structs: Rust composes the crates rather than crossing the C ABI,
//!    so there is nothing between the astronomy and the caller
//!    (`03-design/rust-consumer-surface.md` §3).
//! 3. **What the answer says about itself.** `steps` names every
//!    correction applied, and the provider's own `capabilities` say what
//!    it is and what it covers.
//!
//! Where the other three bindings print a `buildInfo`, Rust has none and
//! should not: Cargo resolved the versions and there is no ABI between
//! this program and the SDK (§6). What is worth logging at start-up is
//! the **provider**, which is what the last section prints.
//!
//! `Ephemeris::Builtin` computes with the analytic ephemeris the SDK
//! carries, so this file runs anywhere with nothing installed. It is the
//! fallback rather than the intended path — most consumers belong on a
//! real engine — but it is astronomy: the scans below find sign
//! ingresses and retrograde stations because the sky has them.
//!
//! ```sh
//! cargo run --release -p teistro --example ephemeris
//! ```

#![expect(clippy::print_stdout, reason = "an example is a program that prints")]

use teistro::catalogue::{Ayanamsha, Graha, Rashi};
use teistro::{Body, Context, Ephemeris, Error, Frame, PositionRequest, TimeScale, Zodiac};

/// A year from the start of 2025, one sample a day at noon UTC.
const START_JD: f64 = 2_460_676.5;
const DAYS: usize = 366;

/// A body is what an ephemeris answers; a graha is what a chart names.
/// They are different catalogues and the lunar node is where they part —
/// `MeanNode` is the body, `Rahu` the graha — so the two are paired
/// explicitly rather than derived from each other's spelling.
const BODIES: [(Body, Graha); 3] = [
    (Body::Sun, Graha::Sun),
    (Body::Mars, Graha::Mars),
    (Body::MeanNode, Graha::Rahu),
];

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

/// Every day on which a body changed sign.
///
/// Reads one body's column out of the grid. Cells run instants
/// outermost, so body `column` at day `i` is cell `i * stride + column`.
fn ingresses(longitudes: &[f64], stride: usize, column: usize) -> Vec<(usize, Rashi)> {
    let mut found = Vec::new();
    let mut previous = None;
    for day in 0..DAYS {
        let Some(sign) = longitudes
            .get(day * stride + column)
            .copied()
            .and_then(sign_of)
        else {
            continue;
        };
        if previous.is_some_and(|before| before != sign) {
            found.push((day, sign));
        }
        previous = Some(sign);
    }
    found
}

/// Every day on which a body turned, direct to retrograde or back.
///
/// The same shape as [`ingresses`] over a different column: a station is
/// a sign change in the speed rather than in the longitude. One grid
/// answers both, which is the reason to ask for a grid.
fn stations(speeds: &[f64], stride: usize, column: usize) -> Vec<(usize, &'static str)> {
    let mut found = Vec::new();
    for day in 1..DAYS {
        let Some((&before, &after)) = speeds
            .get((day - 1) * stride + column)
            .zip(speeds.get(day * stride + column))
        else {
            continue;
        };
        if (before < 0.0) != (after < 0.0) {
            found.push((day, if after < 0.0 { "retrograde" } else { "direct" }));
        }
    }
    found
}

/// Every graha's column, read for the two questions a grid answers at
/// once: where it went, and where it turned.
fn the_scans(sdk: &Context, sky: &teistro::Completed) -> Result<(), Error> {
    let stride = sky.columns.body_count;
    for (column, &(body, graha)) in BODIES.iter().enumerate() {
        let name = sdk.intl().entity(graha.full_key())?.name().to_owned();
        let crossings = ingresses(&sky.columns.lon, stride, column);
        let turns = stations(&sky.columns.lon_speed, stride, column);
        let speed = sky.columns.lon_speed.get(column).copied().unwrap_or(0.0);
        println!(
            "  {:<10} {name:<8} {:<10} at {}{:>7.4}°/day, {} sign change(s), {} station(s)",
            format!("{body:?}"),
            if speed < 0.0 { "retrograde" } else { "direct" },
            if speed < 0.0 { "" } else { "+" },
            speed,
            crossings.len(),
            turns.len(),
        );
        for &(day, sign) in crossings.iter().take(3) {
            println!(
                "      day {day:>3}  enters {:<12} {}",
                sign.key(),
                sdk.intl().entity(sign.full_key())?.name(),
            );
        }
        if crossings.len() > 3 {
            println!("      … and {} more", crossings.len() - 3);
        }
        for (day, into) in turns {
            println!("      day {day:>3}  turns  {into}");
        }
    }
    Ok(())
}

/// What the answer says about itself: the steps applied, and the hash
/// that makes two runs comparable.
fn what_it_says(sdk: &Context, sky: &teistro::Completed) {
    println!("steps    {}", sky.step_keys().join(", "));
    println!("profile  {}", sdk.profile());
    println!("hash     {}", sdk.settings_hash());
    println!(
        "         two contexts with the same settings hash compute the same numbers,\n\
         \x20        so it is the cache key"
    );
    println!(
        "settings {} bytes of canonical JSON -- byte-identical in every binding,\n\
         \x20        which is what makes it safe to hash and store",
        sdk.settings_json().len(),
    );
}

/// Which ephemeris am I talking to?
///
/// A service logs this once at start-up. There is no `buildInfo` here:
/// Cargo fixed the versions and no ABI is crossed, so what there is to
/// know is what the provider declares about itself.
fn which_ephemeris(sdk: &Context) -> Result<(), Error> {
    let provider = sdk
        .ephemeris()
        .ok_or_else(|| Error::internal("this context was built with an ephemeris"))?;
    let what = provider.capabilities();
    let or_dash = |value: &str| {
        if value.is_empty() {
            String::from("-")
        } else {
            value.to_owned()
        }
    };
    println!(
        "provider {} {} (data {})",
        what.identity.name,
        or_dash(&what.identity.version),
        or_dash(&what.identity.data_version),
    );
    println!(
        "covers   JD {:.1} to {:.1}, {} bodies, native frame {:?}/{:?}",
        what.jd_range.0,
        what.jd_range.1,
        what.bodies.len(),
        what.native_frame.centre,
        what.native_frame.zodiac,
    );
    println!(
        "natively {}",
        if what.overrides.names().is_empty() {
            String::from("nothing -- every step above is the SDK's own")
        } else {
            what.overrides.names().join(", ")
        },
    );
    Ok(())
}

fn main() -> Result<(), Error> {
    let sdk = Context::builder()
        .profile("nepali-default")
        .locale("ne-Deva-NP")
        .ephemeris([Ephemeris::Builtin])
        .build()?;

    // ── One call for the whole year ────────────────────────────────────
    let jds: Vec<f64> = (0..DAYS)
        .map(|day| {
            #[expect(
                clippy::cast_precision_loss,
                reason = "a day count under a thousand is exact in an f64"
            )]
            let offset = day as f64;
            START_JD + offset
        })
        .collect();
    let bodies: Vec<Body> = BODIES.iter().map(|&(body, _)| body).collect();
    let frame = Frame::CANONICAL.with_zodiac(Zodiac::sidereal(Ayanamsha::Lahiri));
    let sky = sdk.positions(&PositionRequest::new(&jds, TimeScale::Ut1, &bodies, frame))?;
    println!(
        "grid     {} instants x {} bodies = {} cells in one call",
        sky.columns.jd_count,
        sky.columns.body_count,
        sky.columns.len(),
    );
    println!(
        "columns  lon is a Vec<f64> of {} values, {} bytes -- the astronomy crate's own,\n\
         \x20        with no blob between it and this program",
        sky.columns.lon.len(),
        sky.columns.lon.len() * size_of::<f64>(),
    );

    // ── What the columns are for ───────────────────────────────────────
    println!();
    the_scans(&sdk, &sky)?;

    // ── What the answer says about itself ──────────────────────────────
    println!();
    what_it_says(&sdk, &sky);

    // ── Which ephemeris am I talking to? ───────────────────────────────
    println!();
    which_ephemeris(&sdk)
}
