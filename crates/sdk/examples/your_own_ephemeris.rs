//! An ephemeris of your own, and what happens when it goes wrong.
//!
//! The SDK computes no positions itself: it asks a **provider**, and a
//! provider is a trait you implement. That is the point of the port — an
//! application that already has an ephemeris, a cache, or a table of
//! precomputed positions can put it behind the SDK and get the whole
//! chart layer for free.
//!
//! The contract is small and worth reading carefully:
//!
//! - **One call for the whole grid, never a loop.** The SDK hands over
//!   every instant and every body at once and expects every cell back.
//! - **Refuse the frame you cannot produce.** Answering at all asserts
//!   the answer is in the frame that was asked for, so a provider that
//!   computes in one frame compares `request.frame` with its own and
//!   answers `ProviderError::unsupported`; the SDK then asks again in
//!   the provider's native frame and completes the rest itself, stamping
//!   each step. This is why an engine that knows nothing about the
//!   ayanamsha can serve a Vedic chart.
//! - **Say what you cover.** `bodies` and `jd_range` are checked
//!   *before* your `positions` runs — `validate` is the SDK's own check,
//!   re-exported so you use the same one — so a request you cannot serve
//!   is refused by name rather than by a wrong answer.
//! - **A refusal is a value, not a panic.** `ProviderError` is what
//!   crosses back, and the sentence in it reaches the caller. A Rust
//!   consumer gets the whole error; only a code crosses the C ABI.
//!
//! One dependency is enough to write all of this: everything below comes
//! from `teistro`.
//!
//! ```sh
//! cargo run --release -p teistro --example your_own_ephemeris
//! ```

#![expect(clippy::print_stdout, reason = "an example is a program that prints")]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use teistro::catalogue::Ayanamsha;
use teistro::{
    Body, Capabilities, Cell, CellStatus, Context, Ephemeris, EphemerisKind, EphemerisProvider,
    Error, Frame, Identity, PositionColumns, PositionRequest, ProviderError, Source, TimeScale,
    Zodiac, validate,
};

/// J2000.0, which this toy measures from.
const J2000: f64 = 2_451_545.0;

/// An ephemeris backed by whatever you already have.
///
/// This one is a two-body toy — a circular Sun and Moon — standing in
/// for the real thing: a `.se1` reader, a JPL kernel, a database of
/// precomputed rows, or a cache in front of any of them. What matters is
/// the shape, not the arithmetic.
///
/// The counters are atomics because a provider is `Send + Sync`: the
/// context shares it between threads, so a provider that wants to count
/// its own calls counts them the way any shared value would.
#[derive(Debug, Default)]
struct TableEphemeris {
    /// The frame it insists on, when it insists on one.
    only_frame: Option<Frame>,
    /// Whether its data file is missing, so the failure path can be run.
    broken: bool,
    calls: AtomicUsize,
    cells: AtomicUsize,
    refusals: AtomicUsize,
}

impl TableEphemeris {
    /// Coverage: J2000 to about 2050. A request outside this is refused
    /// before `positions` is ever called.
    const JD_RANGE: (f64, f64) = (J2000, 2_469_807.0);
    /// The bodies it has rows for.
    const BODIES: [Body; 2] = [Body::Sun, Body::Moon];

    fn new() -> TableEphemeris {
        TableEphemeris::default()
    }

    /// The same toy, insisting on one frame.
    fn only(frame: Frame) -> TableEphemeris {
        TableEphemeris {
            only_frame: Some(frame),
            ..TableEphemeris::default()
        }
    }

    /// The same toy, with its data file missing.
    fn broken() -> TableEphemeris {
        TableEphemeris {
            broken: true,
            ..TableEphemeris::default()
        }
    }

    /// How many times the SDK has asked, how many cells it asked for,
    /// and how many frames were refused -- the three numbers a provider
    /// author wants when they are checking that the grid really is one
    /// call.
    fn tally(&self) -> (usize, usize, usize) {
        (
            self.calls.load(Ordering::Relaxed),
            self.cells.load(Ordering::Relaxed),
            self.refusals.load(Ordering::Relaxed),
        )
    }
}

impl EphemerisProvider for TableEphemeris {
    /// What identifies the **data**, not the code. A result's provenance
    /// carries it, so two runs against different rows are
    /// distinguishable even when the code is identical.
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            identity: Identity {
                name: String::from("table-ephemeris"),
                version: String::from("1.0.0"),
                data_version: String::from("demo-rows-2025a"),
                tier: None,
                data_hashes: Vec::new(),
            },
            jd_range: TableEphemeris::JD_RANGE,
            bodies: TableEphemeris::BODIES.to_vec(),
            // Declare only what is true. Everything not named here is
            // `Capabilities::default()`, which declares nothing -- and
            // declaring nothing is what makes the SDK do the work.
            ..Capabilities::default()
        }
    }

    fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
        if self.broken {
            return Err(ProviderError::DataMissing {
                detail: String::from("de431.eph is not where the index says"),
            });
        }
        // The SDK's own check, so a provider refuses a body or an
        // instant it does not cover in exactly the words every other
        // provider uses.
        let capabilities = self.capabilities();
        validate(&capabilities, request)?;
        // **Check the frame.** Answering at all asserts that the answer
        // is in the frame that was asked for; a provider that computes
        // only its own must say so.
        if self.only_frame.is_some_and(|only| only != request.frame) {
            self.refusals.fetch_add(1, Ordering::Relaxed);
            return Err(ProviderError::unsupported(
                "this table holds one frame only",
            ));
        }
        self.calls.fetch_add(1, Ordering::Relaxed);
        self.cells
            .fetch_add(request.cell_count(), Ordering::Relaxed);

        // Cells run instants outermost: cell `i * bodies + j` is instant
        // `i`, body `j`. `set_at` takes the pair, so the order is the
        // SDK's business and not yours.
        let mut columns = PositionColumns::new(
            request.jds.len(),
            request.bodies.len(),
            capabilities.native_frame,
        );
        for (at, jd) in request.jds.iter().enumerate() {
            let days = jd - J2000;
            for (which, body) in request.bodies.iter().enumerate() {
                // **Coverage is per cell**, which is the port's own
                // decision: a year-long grid whose last day runs past
                // the table keeps the 364 days it can compute. So an
                // instant outside the span is a cell with a status, not
                // a refusal of the batch -- `capabilities.covers` is the
                // question, and `Cell::failed` the answer.
                if !capabilities.covers(*jd) {
                    columns.set_at(at, which, Cell::failed(CellStatus::OutOfRange));
                    continue;
                }
                let (start, rate) = if *body == Body::Sun {
                    (280.46, 0.9856)
                } else {
                    (218.32, 13.1764)
                };
                columns.set_at(
                    at,
                    which,
                    Cell {
                        lon: (start + rate * days).rem_euclid(360.0),
                        lat: 0.0,
                        dist: 1.0,
                        // Speeds are optional -- `speeds: false` in the
                        // capabilities would tell the SDK not to expect
                        // them -- but a provider that has them should
                        // give them, because a station is a speed.
                        lon_speed: if request.speeds { rate } else { 0.0 },
                        lat_speed: 0.0,
                        dist_speed: 0.0,
                        status: CellStatus::Ok,
                        source: Source {
                            kind: EphemerisKind::Files,
                            tier: None,
                        },
                    },
                );
            }
        }
        Ok(columns)
    }
}

/// A provider you keep a handle on.
///
/// `Ephemeris::Provider` takes ownership of the box, so a provider with
/// state you want to read back — a cache's hit rate, a file handle, the
/// counters above — is shared rather than given away. One newtype over
/// an `Arc` does it, delegating every method, and the SDK cannot tell
/// the difference.
#[derive(Clone, Debug)]
struct Shared(Arc<TableEphemeris>);

impl EphemerisProvider for Shared {
    fn capabilities(&self) -> Capabilities {
        self.0.capabilities()
    }

    fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
        self.0.positions(request)
    }
}

/// A context over a provider of your own. `Ephemeris::Provider` takes
/// the built thing; `Ephemeris::opening` takes a recipe, for an entry a
/// later one can be the fallback for.
fn context_over(provider: &Arc<TableEphemeris>) -> Result<Context, Error> {
    Context::builder()
        .profile("parashari-classical")
        .ephemeris([Ephemeris::Provider(Box::new(Shared(Arc::clone(provider))))])
        .build()
}

/// The happy path: a week of the Sun and the Moon, in one call.
fn the_happy_path() -> Result<(), Error> {
    let table = Arc::new(TableEphemeris::new());
    let sdk = context_over(&table)?;
    let jds: Vec<f64> = (0..7).map(|day| J2000 + f64::from(day)).collect();
    let bodies = [Body::Sun, Body::Moon];
    let sky = sdk.positions(&PositionRequest::new(
        &jds,
        TimeScale::Ut1,
        &bodies,
        Frame::CANONICAL,
    ))?;
    // **One call, not fourteen.** This is the number to watch when you
    // write a provider: a week of two bodies asked the table once.
    let (asked, cell_count, _) = table.tally();
    println!("asked    {asked} time(s) for {cell_count} cells");
    println!(
        "answered {} cells over {} days",
        sky.columns.len(),
        sky.columns.jd_count
    );
    let cell = |instant, body| {
        sky.columns
            .at(instant, body)
            .map_or(f64::NAN, |cell| cell.lon)
    };
    println!(
        "  sun  {:>8.4}° -> {:>8.4}° in a week",
        cell(0, 0),
        cell(6, 0)
    );
    println!(
        "  moon {:>8.4}° -> {:>8.4}° in a week",
        cell(0, 1),
        cell(6, 1)
    );
    Ok(())
}

/// What the SDK refuses before the provider is asked, what it marks
/// per cell, and what the provider refuses itself.
fn the_refusals() -> Result<(), Error> {
    println!();
    // A body the provider never declared refuses the **whole request**,
    // and names what it does answer -- a refusal that is an instruction.
    let sdk = context_over(&Arc::new(TableEphemeris::new()))?;
    let jds = [J2000];
    let bodies = [Body::Saturn];
    match sdk.positions(&PositionRequest::new(
        &jds,
        TimeScale::Ut1,
        &bodies,
        Frame::CANONICAL,
    )) {
        Ok(_) => println!("refused  a body it never declared: it was not, which is a defect"),
        Err(refusal) => println!("refused  a body it never declared: {}", refusal.message),
    }

    // **Coverage is a per-cell outcome, not a refusal**, and that is the
    // port's own decision (`provider::validate`): a year-long grid whose
    // last day runs past the ephemeris keeps the 364 days it can compute
    // rather than losing all of them. So an instant outside the declared
    // span comes back as a cell with a status, and a caller reads the
    // status rather than catching an error.
    //
    // This is the one place a Rust provider differs from the same
    // provider written in JavaScript, Dart or Python: those reach the
    // port through an adapter shim that checks the coverage span up
    // front and refuses the batch, because a provider in those languages
    // answers columns and not statuses. In Rust you are the port, so you
    // say it per cell -- or refuse the batch yourself, which a provider
    // that would rather is free to do.
    let sdk = context_over(&Arc::new(TableEphemeris::new()))?;
    let jds = [J2000, 2_200_000.0];
    let bodies = [Body::Sun];
    let sky = sdk.positions(&PositionRequest::new(
        &jds,
        TimeScale::Ut1,
        &bodies,
        Frame::CANONICAL,
    ))?;
    println!(
        "per cell an instant inside coverage is {:?} and one outside it is {:?};",
        sky.columns.at(0, 0).map(|cell| cell.status),
        sky.columns.at(1, 0).map(|cell| cell.status),
    );
    println!("         the grid keeps what it could compute");

    // When the provider itself fails, its own sentence reaches the
    // caller -- not a code, and not "the provider failed".
    let sdk = context_over(&Arc::new(TableEphemeris::broken()))?;
    let jds = [J2000];
    let bodies = [Body::Sun];
    if let Err(refusal) = sdk.positions(&PositionRequest::new(
        &jds,
        TimeScale::Ut1,
        &bodies,
        Frame::CANONICAL,
    )) {
        println!();
        println!("thrown   {:?}: {}", refusal.status, refusal.message);
        println!("         the provider's own sentence crossed back, not just a code");
    }
    Ok(())
}

/// A frame it does not compute, which the SDK completes.
fn the_frame_it_refuses() -> Result<(), Error> {
    // This table computes tropical positions and insists on that frame.
    // Ask for a sidereal zodiac and the SDK does the rest, naming every
    // step it applied.
    let table = Arc::new(TableEphemeris::only(Frame::CANONICAL));
    let sdk = context_over(&table)?;
    let jds = [J2000];
    let bodies = [Body::Sun];
    let tropical = sdk.positions(&PositionRequest::new(
        &jds,
        TimeScale::Ut1,
        &bodies,
        Frame::CANONICAL,
    ))?;
    let sidereal = sdk.positions(&PositionRequest::new(
        &jds,
        TimeScale::Ut1,
        &bodies,
        Frame::CANONICAL.with_zodiac(Zodiac::sidereal(Ayanamsha::Lahiri)),
    ))?;
    let lon = |sky: &teistro::Completed| sky.columns.at(0, 0).map_or(f64::NAN, |cell| cell.lon);
    println!();
    println!(
        "frames   the table answered {:.4}° tropical; a sidereal request is {:.4}°",
        lon(&tropical),
        lon(&sidereal),
    );
    let (_, _, refusals) = table.tally();
    println!("         it refused the frame {refusals} time(s), and the SDK completed it:");
    println!("         {}", sidereal.step_keys().join(", "));
    Ok(())
}

/// A refusal a user should see.
///
/// Every refusal from the library carries a status a program can match
/// on and, where the SDK knows one, the detail and a hint to act on.
/// That is what to put in front of a person.
fn a_refusal_a_user_should_see() -> Result<(), Error> {
    let sdk = context_over(&Arc::new(TableEphemeris::new()))?;
    let Err(refusal) = sdk.keys().id("graha.SUNN") else {
        return Err(Error::internal("`graha.SUNN` is not a key"));
    };
    println!();
    println!("status   {:?}", refusal.status);
    println!("message  {}", refusal.message);
    println!(
        "detail   {}",
        refusal
            .detail
            .map_or_else(String::new, |detail| format!("{detail:?}"))
    );
    println!("hint     {}", refusal.hint().unwrap_or_default());
    println!("         a program matches on `status`; a person reads the message and the hint");
    Ok(())
}

/// And the chain, which is what a real consumer writes.
///
/// **An entry the chain can fall back *from* is a recipe, not a built
/// provider**: a `Box<dyn EphemerisProvider>` cannot fail to open, so a
/// chain of them always stops at the first. `Ephemeris::opening` takes
/// the closure, under the name a refusal will use.
fn the_chain() -> Result<(), Error> {
    let sdk = Context::builder()
        .profile("parashari-classical")
        .ephemeris([
            Ephemeris::opening("table-ephemeris", || {
                Err(Error::internal("the rows are not on this machine"))
            }),
            Ephemeris::Builtin,
        ])
        .build()?;
    println!();
    println!(
        "chain    the first entry could not open, so the second answered: {}",
        sdk.ephemeris()
            .map(|provider| provider.capabilities().identity.name)
            .unwrap_or_default(),
    );
    Ok(())
}

fn main() -> Result<(), Error> {
    the_happy_path()?;
    the_refusals()?;
    the_frame_it_refuses()?;
    a_refusal_a_user_should_see()?;
    the_chain()
}
