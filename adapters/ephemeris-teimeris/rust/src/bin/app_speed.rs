//! **What an app pays**, request by request: the calls an almanac or a
//! chart screen makes, timed through the façade a Rust app links, under
//! each setup a consumer ships.
//!
//! The crates' own benches time pieces (a dasha kernel, a render, a
//! series). This times what the performance page's budgets are written
//! for: a context opened, one almanac day, a month grid, a year, a year of
//! festivals, a 90-day muhurta search, a birth chart with its Vimshottari,
//! the full parity chart with the shipped rules, a match and a full day's
//! rectification, each over
//!
//! - `drik-teimeris`: `parashari-classical` over the Teimeris engine, the
//!   modern positions;
//! - `committee-teimeris`: `nepali-committee` over the Surya Siddhanta
//!   with Teimeris second, which reckons the limbs by the text and takes
//!   sunrise and the outer planets from the engine (the official Nepali
//!   calendar's setup);
//! - `drik-builtin`: `parashari-classical` over the built-in ephemeris,
//!   which needs no engine at all.
//!
//! **Cold memo, every run.** Each run builds a fresh context outside the
//! timed region, so the call memo starts empty and a repeat is not timed
//! as a cache hit; only `open` times the build itself. The median of the
//! runs is what the page reports, with the fastest and slowest beside it.
//!
//! **Contexts on threads.** Each setup's month grid is then asked on 1, 2,
//! 4, … threads up to the machine's parallelism, each thread on a context
//! of its own opened before the clock starts. What a thread count reports
//! is its throughput beside the one thread's (the efficiency: 1 is
//! linear) and whether every thread's month hashed as the one thread's
//! did, since contexts that share nothing answer alike whatever runs
//! beside them.
//!
//! stdout is the record; `cargo xtask app-speed` renders the page from it
//! and `check-app-speed` gates each median against its budget and each
//! thread count against its floor.
//!
//! ```text
//! cargo run --release --features app-speed --bin teistro-ephemeris-teimeris-app-speed \
//!   > ../../../docs/05-testing/app-speed/desktop-$(uname -s)-$(uname -m).json
//! ```

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "a tooling binary writes its record to stdout, stops on a refused request, \
              and reads a sorted list it has just filled"
)]

use std::process::Command;
use std::sync::Barrier;
use std::time::{Duration, Instant};

use serde::Serialize;
use teistro::catalogue::{Calendar, DashaSystem};
use teistro::muhurta::ActivityRules;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::rectification::{Rules, Window};
use teistro::{
    CalendarDate, ChangesRequest, ChartRequest, Context, DayClock, Ephemeris, FestivalPack, FestivalRequest, Hash,
    KootaRules, MuhurtaRequest, RuleRequest, RuleSet, ShippedRules, UtcOffset, content_hash,
};
use teistro_ephemeris_teimeris::{DATA_DIR_ENV, TeimerisProvider, data_dir_from_env};

/// One setup a consumer ships: a profile and the chain it opens.
#[derive(Clone, Copy)]
struct Setup {
    key: &'static str,
    profile: &'static str,
    chain: fn() -> Vec<Ephemeris>,
}

/// Where the engine's data is read from: the directory `DATA_DIR_ENV`
/// names, or else the data built into the engine (`TM_EMBED_TIER`), which
/// is how a phone has it, or else the repository's own directory.
fn data() -> &'static str {
    if std::env::var_os(DATA_DIR_ENV).is_none() && TeimerisProvider::embedded().is_ok() {
        "embedded"
    } else {
        "files"
    }
}

fn teimeris() -> Ephemeris {
    Ephemeris::Provider(Box::new(if data() == "embedded" {
        TeimerisProvider::embedded().expect("the embedded data")
    } else {
        TeimerisProvider::open(&data_dir_from_env()).expect("the Teimeris data directory")
    }))
}

const SETUPS: [Setup; 3] = [
    Setup {
        key: "drik-teimeris",
        profile: "parashari-classical",
        chain: || vec![teimeris()],
    },
    Setup {
        key: "committee-teimeris",
        profile: "nepali-committee",
        chain: || vec![Ephemeris::SuryaSiddhanta, teimeris()],
    },
    Setup {
        key: "drik-builtin",
        profile: "parashari-classical",
        chain: || vec![Ephemeris::Builtin],
    },
];

/// The requests, in the order the page prints them.
const REQUESTS: [&str; 12] = [
    "open",
    "day",
    "month",
    "year",
    "festivals",
    "changes",
    "muhurta",
    "chart",
    "parity-chart",
    "widest-chart",
    "matching",
    "rectify",
];

/// Runs a request gets: at least [`MIN_RUNS`], and more while the total
/// stays under [`SPEND`], up to [`MAX_RUNS`].
const MIN_RUNS: usize = 3;
const MAX_RUNS: usize = 21;
const SPEND: Duration = Duration::from_secs(4);

fn place() -> Place {
    Place::new(
        Latitude::try_new(27.7172).expect("a latitude"),
        Longitude::try_new(85.324).expect("a longitude"),
        Altitude::try_new(1400.0).expect("an altitude"),
    )
}

fn offset() -> UtcOffset {
    UtcOffset::literal(5, 45, 0)
}

fn date(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, year, month, day)
}

fn open(setup: Setup) -> Context {
    Context::builder()
        .profile(setup.profile)
        .locale("ne-Deva-NP")
        .ephemeris((setup.chain)())
        .build()
        .expect("the setup opens")
}

/// A birth chart with its Vimshottari, to the settings' depth (three).
fn chart_request() -> ChartRequest {
    ChartRequest::at(place(), offset()).with_dashas([DashaSystem::Vimshottari])
}

/// The baseline engine's parity-tier dashas a birth chart gives, as
/// `01-research/feature-universe/03-dashas.md` marks them `P0`: the Sthira
/// dasa is left out as a request for everything leaves it out (a tie in
/// sign strength refuses it), and Patyayini and Mudda divide a year, which
/// the annual chart reads.
const PARITY_DASHAS: [DashaSystem; 16] = [
    DashaSystem::Vimshottari,
    DashaSystem::Ashtottari,
    DashaSystem::Yogini,
    DashaSystem::Kalachakra,
    DashaSystem::Shodashottari,
    DashaSystem::Dwadashottari,
    DashaSystem::Panchottari,
    DashaSystem::Shatabdika,
    DashaSystem::ChaturashitiSama,
    DashaSystem::DwisaptatiSama,
    DashaSystem::Shashtihayani,
    DashaSystem::ShattrimshaSama,
    DashaSystem::Chara,
    DashaSystem::Narayana,
    DashaSystem::Shoola,
    DashaSystem::SudarshanaChakra,
];

/// The instant the chart requests are founded at, and the day the
/// rectification window opens on: 1 January 2000, 06:00 UT.
const BIRTH: f64 = 2_451_545.25;

/// Every shipped rule, as one set.
fn shipped_rules() -> RuleSet {
    RuleRequest::shipped(ShippedRules::ALL)
        .rule_set()
        .expect("the shipped rules")
}

/// A chart at the birth read with a rule set, the reading adding what the
/// rules read itself.
fn read_with_rules(sdk: &Context, request: &ChartRequest, set: &RuleSet) {
    drop(
        sdk.chart()
            .readings_with_rules(&[JulianDay::<Utc>::literal(BIRTH)], request, set)
            .expect("a chart read with its rules"),
    );
}

/// Times one run of `request` on a fresh context; `open` times the build.
fn run(setup: Setup, request: &str) -> Duration {
    if request == "open" {
        let started = Instant::now();
        let context = open(setup);
        let spent = started.elapsed();
        drop(context);
        return spent;
    }
    let sdk = open(setup);
    // The partners are cast before the clock starts: what is timed is the
    // match, which is the call a matching screen makes on two saved charts.
    let partners = (request == "matching").then(|| {
        let cast = |jd: f64| {
            sdk.chart()
                .reading(JulianDay::<Utc>::literal(jd), &chart_request())
                .expect("a chart")
                .value
        };
        (cast(2_447_892.6), cast(2_448_380.2))
    });
    // The shipped rules are read once, as an app reads them when it opens:
    // what is timed is a chart read with them.
    let rules = matches!(request, "parity-chart" | "widest-chart").then(shipped_rules);
    let started = Instant::now();
    match request {
        "day" => drop(
            sdk.almanac()
                .of(&date(2026, 10, 20), &date(2026, 10, 20), &place(), offset())
                .expect("a day"),
        ),
        "month" => drop(
            sdk.almanac()
                .of(&date(2026, 9, 27), &date(2026, 11, 7), &place(), offset())
                .expect("a month grid"),
        ),
        "year" => drop(
            sdk.almanac()
                .of(&date(2026, 1, 1), &date(2026, 12, 31), &place(), offset())
                .expect("a year"),
        ),
        "festivals" => drop(
            sdk.almanac()
                .festivals(
                    &date(2026, 1, 1),
                    &date(2026, 12, 31),
                    &place(),
                    offset(),
                    &FestivalRequest::from(FestivalPack::Nepal),
                )
                .expect("a year of festivals"),
        ),
        "changes" => drop(
            sdk.almanac()
                .changes(&ChangesRequest {
                    first: date(2026, 1, 1),
                    last: date(2026, 12, 31),
                    place: place(),
                    clock: DayClock::Offset(offset()),
                    kinds: Vec::new(),
                })
                .expect("a year of change points"),
        ),
        "muhurta" => drop(
            sdk.almanac()
                .muhurta(
                    &date(2026, 11, 1),
                    &date(2027, 1, 29),
                    &place(),
                    offset(),
                    &MuhurtaRequest::new(ActivityRules::raman_marriage()),
                )
                .expect("a muhurta search"),
        ),
        "chart" => drop(
            sdk.chart()
                .reading(JulianDay::<Utc>::literal(BIRTH), &chart_request())
                .expect("a chart"),
        ),
        // The performance page's full parity chart: every section and
        // divisional chart, the baseline engine's parity-tier dashas to
        // the settings' depth (three), and every shipped rule, which is
        // more than the page's 624 (the reading adds what they read).
        "parity-chart" => read_with_rules(
            &sdk,
            &ChartRequest::at(place(), offset())
                .with_everything()
                .with_dashas(PARITY_DASHAS),
            rules.as_ref().expect("read above"),
        ),
        // The widest chart there is: every dasha a chart always gives as
        // well, which a parity runner asks of the SDK.
        "widest-chart" => read_with_rules(
            &sdk,
            &ChartRequest::at(place(), offset()).with_everything(),
            rules.as_ref().expect("read above"),
        ),
        // A birth known to the day: a window a day long, judged at the
        // default seed step.
        "rectify" => {
            let window = Window::between(
                JulianDay::<Utc>::literal(BIRTH - 0.25),
                JulianDay::<Utc>::literal(BIRTH + 0.75),
            )
            .expect("a day's window");
            drop(
                sdk.chart()
                    .rectify(window, &place(), offset(), &Rules::default())
                    .expect("a rectification"),
            );
        }
        "matching" => {
            let (bride, groom) = partners.expect("cast above");
            drop(
                sdk.chart()
                    .matching(&bride, &groom, KootaRules::default())
                    .expect("a match"),
            );
        }
        other => unreachable!("no request `{other}`"),
    }
    started.elapsed()
}

#[derive(Serialize)]
struct Timing {
    setup: &'static str,
    request: &'static str,
    runs: usize,
    median_ms: f64,
    min_ms: f64,
    max_ms: f64,
}

#[derive(Serialize)]
struct Record {
    machine: String,
    os: String,
    arch: &'static str,
    commit: String,
    teimeris: String,
    data: &'static str,
    memo: &'static str,
    timings: Vec<Timing>,
    /// The threads the machine offers, as the standard library reads them.
    parallelism: usize,
    scaling: Vec<Scaling>,
}

/// One setup's month grid on a number of threads.
#[derive(Serialize)]
struct Scaling {
    setup: &'static str,
    threads: usize,
    /// Months each thread asks in a trial.
    months: u32,
    trials: usize,
    /// The median trial's wall time, from the threads' start to the last
    /// one's end.
    median_ms: f64,
    /// Months answered a second over the median trial, all threads
    /// together.
    per_second: f64,
    /// `per_second` over the one thread's times this many threads.
    efficiency: f64,
    /// Whether every thread's month hashed as the one thread's.
    identical: bool,
}

fn ms(duration: Duration) -> f64 {
    (duration.as_secs_f64() * 1e6).round() / 1e3
}

fn measure(setup: Setup, request: &'static str) -> Timing {
    // A warm-up, so the first run's page faults and lazily built tables
    // are not one of the timed ones.
    run(setup, request);
    let mut times = Vec::new();
    let began = Instant::now();
    while times.len() < MIN_RUNS || (times.len() < MAX_RUNS && began.elapsed() < SPEND) {
        times.push(run(setup, request));
    }
    times.sort();
    Timing {
        setup: setup.key,
        request,
        runs: times.len(),
        median_ms: ms(times[times.len() / 2]),
        min_ms: ms(times[0]),
        max_ms: ms(times[times.len() - 1]),
    }
}

/// Months each thread asks in a trial, and the trials a thread count gets.
const MONTHS: u32 = 3;
const TRIALS: usize = 5;

/// The month grid every thread asks: the `month` request's.
fn month(sdk: &Context) -> Hash {
    let grid = sdk
        .almanac()
        .of(&date(2026, 9, 27), &date(2026, 11, 7), &place(), offset())
        .expect("a month grid");
    content_hash(&grid.value)
}

/// 1, 2, 4, … up to `most`, and `most` itself.
fn counts(most: usize) -> Vec<usize> {
    let mut counts: Vec<usize> = std::iter::successors(Some(1usize), |n| n.checked_mul(2))
        .take_while(|n| *n <= most)
        .collect();
    if counts.last() != Some(&most) {
        counts.push(most);
    }
    counts
}

/// One trial on `threads` threads: its wall time and every month's hash.
/// Each thread opens its context and asks once before the clock starts,
/// so what is timed is asking, on contexts already warm.
fn trial(setup: Setup, threads: usize) -> (Duration, Vec<Hash>) {
    let ready = Barrier::new(threads + 1);
    let done = Barrier::new(threads + 1);
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|_| {
                scope.spawn(|| {
                    let sdk = open(setup);
                    month(&sdk);
                    ready.wait();
                    let hashes: Vec<Hash> = (0..MONTHS).map(|_| month(&sdk)).collect();
                    done.wait();
                    hashes
                })
            })
            .collect();
        ready.wait();
        let started = Instant::now();
        done.wait();
        let spent = started.elapsed();
        let hashes = workers
            .into_iter()
            .flat_map(|worker| worker.join().expect("a thread answers"))
            .collect();
        (spent, hashes)
    })
}

fn scaling(setup: Setup, parallelism: usize) -> Vec<Scaling> {
    let mut rows: Vec<Scaling> = Vec::new();
    let mut expected: Option<Hash> = None;
    for threads in counts(parallelism) {
        let mut spent = Vec::with_capacity(TRIALS);
        let mut identical = true;
        for _ in 0..TRIALS {
            let (wall, hashes) = trial(setup, threads);
            let first = *expected.get_or_insert(hashes[0]);
            identical &= hashes.iter().all(|hash| *hash == first);
            spent.push(wall);
        }
        spent.sort();
        let median = spent[TRIALS / 2];
        let many = f64::from(u32::try_from(threads).expect("a count of threads"));
        let per_second = many * f64::from(MONTHS) / median.as_secs_f64();
        let single = rows.first().map_or(per_second, |one| one.per_second);
        rows.push(Scaling {
            setup: setup.key,
            threads,
            months: MONTHS,
            trials: TRIALS,
            median_ms: ms(median),
            per_second: (per_second * 100.0).round() / 100.0,
            efficiency: (per_second / (single * many) * 1000.0).round() / 1000.0,
            identical,
        });
    }
    rows
}

fn output(program: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(program).args(args).output().ok()?;
    let text = String::from_utf8(out.stdout).ok()?;
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_owned())
}

/// The commit measured, with `-dirty` when the tree has changes other than
/// the records and their page, which recording writes (the record this
/// run's output is redirected into among them): the rule
/// `xtask/src/app_speed.rs`'s `recorded_commit` applies to a device's.
fn commit() -> Option<String> {
    let head = output("git", &["rev-parse", "--short=8", "HEAD"])?;
    let changes = output(
        "git",
        &[
            "status",
            "--porcelain",
            "--untracked-files=no",
            "--",
            ":/",
            ":(top,exclude)docs/05-testing/app-speed",
            ":(top,exclude)docs/05-testing/app-speed-measured.md",
        ],
    );
    Some(if changes.is_some() { format!("{head}-dirty") } else { head })
}

/// An environment variable's value, when it is set and not empty.
fn var(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|value| !value.is_empty())
}

/// What ran it. A phone, an emulator or a simulator names itself, since
/// its processor is the host's or one no table would know: Android by its
/// model property (`sdk_gphone64_arm64` is the emulator), the simulator by
/// the device `simctl` booted.
fn machine() -> String {
    output("getprop", &["ro.product.model"])
        .map(|model| {
            // The emulator says so in its boot properties; a phone does not.
            let emulated = output("getprop", &["ro.boot.qemu"]).as_deref() == Some("1")
                || output("getprop", &["ro.kernel.qemu"]).as_deref() == Some("1");
            if emulated {
                format!("Android emulator, `{model}`")
            } else {
                format!("Android phone, `{model}`")
            }
        })
        .or_else(|| var("SIMULATOR_DEVICE_NAME").map(|name| format!("iOS simulator, {name}")))
        .or_else(|| output("sysctl", &["-n", "machdep.cpu.brand_string"]))
        .or_else(|| {
            std::fs::read_to_string("/proc/cpuinfo")
                .ok()
                .and_then(|info| {
                    info.lines()
                        .find(|line| line.starts_with("model name"))
                        .and_then(|line| line.split_once(':'))
                        .map(|(_, name)| name.trim().to_owned())
                })
        })
        .unwrap_or_else(|| String::from("unknown"))
}

fn os() -> String {
    let android = (
        output("getprop", &["ro.build.version.release"]),
        output("getprop", &["ro.build.version.sdk"]),
    );
    if let (Some(release), Some(api)) = android {
        return format!("Android {release}, API {api}");
    }
    if let Some(version) = var("SIMULATOR_RUNTIME_VERSION") {
        return format!("iOS {version}");
    }
    output("uname", &["-sr"]).unwrap_or_else(|| std::env::consts::OS.to_owned())
}

/// Whether `TEISTRO_APP_SPEED_ONLY` (`setup:request`, comma-separated)
/// lets a row run: a profiler's way to time one request alone. Unset, every
/// row runs; a partial run is a profile, not a record.
fn wanted(setup: Setup, request: &str) -> bool {
    var("TEISTRO_APP_SPEED_ONLY").is_none_or(|only| {
        only.split(',')
            .any(|row| row.trim() == format!("{}:{request}", setup.key))
    })
}

fn main() {
    let mut timings = Vec::new();
    for setup in SETUPS {
        for request in REQUESTS {
            if !wanted(setup, request) {
                continue;
            }
            let timing = measure(setup, request);
            eprintln!(
                "{:<20} {:<10} {:>10.3} ms  ({} runs)",
                timing.setup, timing.request, timing.median_ms, timing.runs
            );
            timings.push(timing);
        }
    }
    let parallelism = std::thread::available_parallelism().map_or(1, usize::from);
    let mut scaled = Vec::new();
    for setup in SETUPS {
        if !wanted(setup, "threads") {
            continue;
        }
        for row in scaling(setup, parallelism) {
            eprintln!(
                "{:<20} {:>2} threads {:>8.2} months/s  efficiency {:.3}{}",
                row.setup,
                row.threads,
                row.per_second,
                row.efficiency,
                if row.identical { "" } else { "  DIFFERS" }
            );
            scaled.push(row);
        }
    }
    let record = Record {
        machine: machine(),
        os: os(),
        arch: std::env::consts::ARCH,
        // A device has no checkout: its runner passes the commit in.
        commit: var("TEISTRO_COMMIT")
            .or_else(commit)
            .unwrap_or_else(|| String::from("unknown")),
        teimeris: teimeris::version().to_string(),
        data: data(),
        memo: "cold: a fresh context every run",
        timings,
        parallelism,
        scaling: scaled,
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&record).expect("the record serialises")
    );
}
