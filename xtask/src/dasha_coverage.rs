//! What the catalogue names, what this build computes, and why the rest is
//! not computed yet.
//!
//! `DashaSystem` catalogues forty systems and `teistro::dasha::systems()`
//! computes eighteen of them. The gap is not a defect — a catalogue is a
//! key space and a build is a set of kernels — but an **undeclared** gap
//! is, because a consumer reading the catalogue has no way to tell which
//! member is a feature and which is a name waiting for one.
//!
//! [`dasha-kernels.md`] examined the systems one by one and marked each
//! `V`, `T` or `S`, so the reasons below are read off that page's own
//! analysis rather than invented here. What the page cannot do is stay
//! true: it says "56 systems examined" beside a catalogue of forty and a
//! build of eighteen, which is three numbers in three places and the shape
//! this repository keeps finding in its own prose.
//!
//! So this pass counts both sets from the types and holds the difference
//! against an exhaustive reasoned list, which fails **three** ways:
//!
//! 1. a catalogued system this build does not compute and the list does
//!    not excuse — the gap grew and nobody said why;
//! 2. a system the list excuses that this build **does** compute — a
//!    reason outliving its blocker, which is how a page comes to describe
//!    work that is finished;
//! 3. an excuse naming something the catalogue does not name at all, so a
//!    rename cannot leave a reason pointing at nothing.
//!
//! Beside the list it measures the **refusal**, because "declared" is a
//! claim about behaviour: every unbuilt system is asked of a real founded
//! chart, and each must come back refused by name, with the built systems
//! named in the hint. A system that answered an empty reading instead
//! would be the dead end the catalogue is suspected of, and this is what
//! says it is not.
//!
//! [`dasha-kernels.md`]: ../../docs/03-design/dasha-kernels.md

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use teistro_core::catalogue::DashaSystem;

use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, count, fill, listed, plural, table};

const PAGE: &str = "docs/03-design/dasha-coverage-measured.md";

/// What stands between a catalogued system and a computed one, hardest
/// first.
///
/// A queue is grouped by its **hardest** blocker and not the first one
/// found: Patyayini needs a kernel of its own *and* the annual chart, and
/// filing it under the kernel would put it in a queue that finishing the
/// kernel does not empty.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Blocker {
    /// The text is not settled: a verse to cite, a reference index to
    /// confirm, or a shape nothing attests. Nobody here can close these by
    /// building, and a row written without the text in front of you is a
    /// table invented rather than cited.
    Text,
    /// The kernel cannot express it. A field the row schema has not got, or
    /// a kernel that does not exist.
    Kernel,
    /// It waits on a module this build has not got, so the dasha is the
    /// last step of that module rather than a task of its own.
    Module,
}

impl Blocker {
    const ALL: [Blocker; 3] = [Blocker::Text, Blocker::Kernel, Blocker::Module];

    const fn title(self) -> &'static str {
        match self {
            Blocker::Text => "the text is not settled",
            Blocker::Kernel => "the kernel cannot express it",
            Blocker::Module => "it waits on a module",
        }
    }

    const fn what_closes_it(self) -> &'static str {
        match self {
            Blocker::Text => {
                "a cited source read by someone who reads it. Building is not what \
                 is missing: every row here is one line of table data once the verse \
                 settles it, which is why none of them is scheduled and all of them \
                 are listed"
            }
            Blocker::Kernel => {
                "a change to the kernel, which is the decision ADR-0017 set a kill \
                 criterion for: a third chart-query field serves three of these at \
                 once, or the kernel is redesigned as an interpreter and serves them \
                 all. They are taken together for that reason, never one at a time"
            }
            Blocker::Module => {
                "the module itself. Each is the **last** step of one, so scheduling \
                 the dasha separately would schedule the module twice"
            }
        }
    }
}

/// Every catalogued system this build does not compute, with the reason,
/// read from [`dasha-kernels.md`]'s own row analysis.
///
/// It is a list and not a count, because a count says nothing about which
/// and the which is the work queue. Twenty-two rows is short enough to be
/// exhaustive, and the pass refuses it in three directions, so it cannot
/// quietly stop being exhaustive.
///
/// [`dasha-kernels.md`]: ../../docs/03-design/dasha-kernels.md
const NOT_BUILT: [(&str, Blocker, &str); 22] = [
    (
        "SHODASHOTTARI",
        Blocker::Text,
        "the row is stated — from Pushya, eight lords, 116 years — and its verse \
         numbers in BPHS ch. 46 are to be confirmed before it ships",
    ),
    (
        "SHATTRIMSHA_SAMA",
        Blocker::Text,
        "the same verse numbers to confirm. Its table is Yogini's, lord for lord \
         and year for year, differing only in the reference nakshatra and the \
         offset — so shipping it on a guess would ship Yogini twice",
    ),
    (
        "SHASHTIHAYANI",
        Blocker::Text,
        "the received text gives Jupiter 13, Sun 13, Mars 13 and then six each, \
         which sums to 69 and not to the 60 the name states. A row cannot be \
         written from a text that disagrees with itself",
    ),
    (
        "TITHI_ASHTOTTARI",
        Blocker::Text,
        "Ashtottari's table seeded by the **tithi**. The reference index does not \
         carry over from the nakshatra rows because the cycles differ, 30 against \
         27, so the seat is unknown",
    ),
    (
        "TITHI_YOGINI",
        Blocker::Text,
        "Yogini's table on the same thirty-fold cycle, blocked by the same unknown \
         seat",
    ),
    (
        "YOGA_VIMSHOTTARI",
        Blocker::Text,
        "Vimshottari's table seeded by the **yoga**, on a twenty-seven-fold cycle \
         whose reference is not the nakshatra's",
    ),
    (
        "KARANA_CHATURASHITI",
        Blocker::Text,
        "Chaturashiti-sama's table seeded by the **karana**, on a sixty-fold cycle",
    ),
    (
        "NAISARGIKA",
        Blocker::Text,
        "the natural order and the lifespan periods it divides are both unsettled \
         here, which is the whole of the row",
    ),
    (
        "PANCHASWARA",
        Blocker::Text,
        "no attested shape at all. It is catalogued because a system by that name \
         is named, which is what a key space is for",
    ),
    (
        "STHIRA",
        Blocker::Text,
        "the row is stated — from the lagna, consecutive, seven, eight or nine \
         years by modality — and nothing here verifies it. It is one boolean from \
         Mandooka's row, which is exactly why a guess would go unnoticed",
    ),
    (
        "VARNADA",
        Blocker::Text,
        "the Varnada lagna it starts from **is** built (`teistro_points`), so what \
         is missing is not the point but which of five school variants of it the \
         dasha counts from",
    ),
    (
        "TARA",
        Blocker::Kernel,
        "its periods come from the chart's tara counts, and the row schema states \
         its periods. A row that asks the chart is a field K-udu has not got",
    ),
    (
        "KARAKA",
        Blocker::Kernel,
        "its lord **order** comes from chara-karaka strength, which is a second \
         thing a row would have to ask the chart for",
    ),
    (
        "ASHTAKAVARGA",
        Blocker::Kernel,
        "its periods come from the bindu counts. The three above are one \
         mechanism asked for three ways, and the kernel takes them together or \
         not at all",
    ),
    (
        "YOGARDHA",
        Blocker::Kernel,
        "the mean of two systems, if that is what it is. A composition over \
         kernels is not an algorithm inside one",
    ),
    (
        "SUDARSHANA_CHAKRA",
        Blocker::Kernel,
        "three rashi progressions running at once, from the lagna, the Sun and the \
         Moon. The same composition, and the reason it is a combinator",
    ),
    (
        "AAYU",
        Blocker::Module,
        "the longevity module decides the span this divides, and that span is \
         three methods with a reconciliation between them (`crates/rules` \
         `longevity`). The dasha is what the module ends with",
    ),
    (
        "SUDASA",
        Blocker::Module,
        "it starts from the karakamsha, which is the navamsha of the Atmakaraka \
         and is not built — the Sree lagna beside it in the sources is. Karakamsha \
         belongs to the Jaimini module and this follows it",
    ),
    (
        "MUDDA",
        Blocker::Module,
        "Vimshottari scaled to the year, which needs the **annual chart** — the \
         Varsha Pravesha solar return. The scale decorator it would use is built \
         and Tribhagi uses it",
    ),
    (
        "VARSHA_NARAYANA",
        Blocker::Module,
        "Narayana over the annual chart, blocked on the same solar return",
    ),
    (
        "VARSHA_YOGINI",
        Blocker::Module,
        "Yogini over the annual chart, blocked on the same solar return",
    ),
    (
        "PATYAYINI",
        Blocker::Module,
        "periods from the grahas' strengths **in the annual chart**, so it needs a \
         kernel of its own as well. Filed under the module because finishing the \
         kernel would not unblock it",
    ),
];

/// The unbuilt systems a **consumer** can supply today, which is what turns
/// "not built" into "not built *here*".
///
/// `DashaSystems::register` takes a `UduDefinition`: lords, years and a
/// **nakshatra** reference. So a system is registrable exactly when its row
/// is that shape and the only thing missing is the numbers in it — which is
/// three of the twenty-two. The rest are not withheld from a consumer on
/// purpose; there is no definition that expresses them, and that asymmetry
/// is measured below rather than described.
const REGISTRABLE: [&str; 3] = ["SHODASHOTTARI", "SHATTRIMSHA_SAMA", "SHASHTIHAYANI"];

/// Whether the registry path actually works for one of them, proved by
/// walking it rather than by citing the API.
///
/// The row is the one [`dasha-kernels.md`] states for Shodashottari, whose
/// verse numbers are exactly what is unconfirmed — so it is registered
/// under a **demonstration key** the catalogue does not have. Nothing here
/// ships as the system, and the numbers are the design page's, not this
/// pass's invention.
///
/// [`dasha-kernels.md`]: ../../docs/03-design/dasha-kernels.md
fn a_consumer_supplies_one() -> Result<String, String> {
    use teistro::dasha::{DashaSystems, Lord, UduDefinition};
    use teistro_core::catalogue::{Graha, Nakshatra};

    let lords = [
        (Graha::Sun, 11),
        (Graha::Mars, 12),
        (Graha::Jupiter, 13),
        (Graha::Saturn, 14),
        (Graha::Ketu, 15),
        (Graha::Moon, 16),
        (Graha::Mercury, 17),
        (Graha::Venus, 18),
    ];
    let definition = UduDefinition {
        sources: vec![String::from("03-design/dasha-kernels.md, K-udu rows")],
        lords: lords
            .into_iter()
            .map(|(graha, years)| Lord { graha, years })
            .collect(),
        ..UduDefinition::of("DEMO_SHODASHOTTARI", Nakshatra::Pushya)
    };
    let total: u16 = lords.iter().map(|(_, years)| u16::from(*years)).sum();
    let mut systems = DashaSystems::default();
    systems
        .register(definition)
        .map_err(|why| format!("registering the stated row: {why}"))?;
    Ok(format!(
        "`DEMO_SHODASHOTTARI`, {} lords and {total} years, registered and accepted, with the total the design page states for it falling out of the lords rather than copied beside them",
        lords.len()
    ))
}

/// What one refusal looked like, so that "declared" is measured rather
/// than asserted.
struct Refused {
    system: &'static str,
    /// Whether the refusal names the system that was asked for.
    names_it: bool,
    /// Whether the hint names the systems this build does compute.
    names_the_built: bool,
    message: String,
}

/// Every unbuilt system asked of a real chart, and what came back.
///
/// The chart is founded once per system because the refusal happens
/// **while** a chart is founded: a dasha the build has no row for is
/// refused where the reading would have been computed, which is the only
/// place a consumer can meet it.
fn refusals(sdk: &teistro::Context, ask: &[&'static str]) -> Result<Vec<Refused>, String> {
    let place = teistro::quantity::Place::new(
        teistro::quantity::Latitude::try_new(27.7172).map_err(|why| why.to_string())?,
        teistro::quantity::Longitude::try_new(85.324).map_err(|why| why.to_string())?,
        teistro::quantity::Altitude::try_new(1400.0).map_err(|why| why.to_string())?,
    );
    let offset = teistro::UtcOffset::try_from_seconds(20_700).map_err(|why| why.to_string())?;
    let instant =
        teistro::quantity::JulianDay::try_new(2_448_000.5).map_err(|why| why.to_string())?;
    let built: Vec<&str> = teistro::dasha::systems().map(DashaSystem::key).collect();
    let mut out = Vec::new();
    for key in ask.iter().copied() {
        let system = DashaSystem::ALL
            .into_iter()
            .find(|system| system.key() == key)
            .ok_or_else(|| format!("`{key}` is not a catalogued dasha system"))?;
        let request = teistro::ChartRequest::at(place, offset).with_dashas([system]);
        let Err(error) = sdk.chart().readings(&[instant], &request) else {
            return Err(format!(
                "`{key}` is not built and a chart asking for it was answered rather than refused"
            ));
        };
        let message = error.to_string();
        out.push(Refused {
            system: key,
            names_it: message.contains(key),
            names_the_built: built.iter().all(|one| message.contains(one)),
            message,
        });
    }
    Ok(out)
}

/// The catalogue against the build, and the list against both.
fn page(_root: &Path) -> Result<String, String> {
    let computed: BTreeSet<&str> = teistro::dasha::systems().map(DashaSystem::key).collect();
    let catalogued: BTreeSet<&str> = DashaSystem::ALL.iter().map(|system| system.key()).collect();
    let excused: BTreeMap<&str, (Blocker, &str)> = NOT_BUILT
        .into_iter()
        .map(|(key, blocker, why)| (key, (blocker, why)))
        .collect();

    // The three directions, each counted rather than asserted.
    let unexcused: Vec<&str> = catalogued
        .iter()
        .filter(|key| !computed.contains(*key) && !excused.contains_key(*key))
        .copied()
        .collect();
    let outlived: Vec<&str> = excused
        .keys()
        .filter(|key| computed.contains(*key))
        .copied()
        .collect();
    let unknown: Vec<&str> = excused
        .keys()
        .filter(|key| !catalogued.contains(*key))
        .copied()
        .collect();

    let sdk = teistro::Context::builder()
        .ephemeris([teistro::Ephemeris::Builtin])
        .build()
        .map_err(|why| format!("a context over the built-in ephemeris: {why}"))?;
    // Only what is both catalogued and uncomputed is asked for, because a
    // system that failed one of the three checks above would fail this one
    // for that reason and hide the count that says so. The checks are
    // measured on the page; they are not a guard in front of it.
    let askable: Vec<&'static str> = NOT_BUILT
        .into_iter()
        .map(|(key, _, _)| key)
        .filter(|key| catalogued.contains(key) && !computed.contains(key))
        .collect();
    let refused = refusals(&sdk, &askable)?;

    let mut out = String::new();
    out.push_str("# What the catalogue names and what this build computes\n\n");
    let _ = write!(
        out,
        "Status: `generated` by `cargo xtask dasha-coverage` over \
         `DashaSystem::ALL` and `teistro::dasha::systems()`. Do not edit: \
         `check-dasha-coverage` regenerates this page and fails on any \
         difference. The design it measures is \
         [`dasha-kernels.md`](dasha-kernels.md).\n\n"
    );

    let _ = write!(
        out,
        "The catalogue names {} and this build computes {}. The {} left are \
         **not** a backlog of unwritten code: every one of them is blocked on \
         something that is not typing, and this page is the list of what, \
         grouped by the blocker that would have to go first.\n\n\
         A catalogue is a **key space** and a build is a set of kernels, so \
         the two were never going to be the same size ([Q38](../QUESTIONS.md)). \
         What would be a defect is the gap going **undeclared**, because then \
         a consumer reading the catalogue cannot tell a feature from a name \
         waiting for one. It is declared twice over: named here, and refused \
         at the call.\n\n",
        plural(catalogued.len(), "dasha system"),
        count(computed.len()),
        count(catalogued.len() - computed.len()),
    );

    the_queue(&mut out);

    asking_for_one(&mut out, &refused);

    let supplied = a_consumer_supplies_one()?;
    who_can_supply_one(&mut out, &supplied);

    what_the_types_decide(
        &mut out,
        &Sets {
            catalogued: catalogued.len(),
            computed: computed.len(),
        },
        &Falsified {
            unexcused: &unexcused,
            outlived: &outlived,
            unknown: &unknown,
        },
        &refused,
    );

    Ok(fill(&out))
}

/// The two sizes the page is about, counted from the types.
struct Sets {
    catalogued: usize,
    computed: usize,
}

/// What each of the three directions found, so that a falsification names
/// what falsified it rather than only counting it.
struct Falsified<'a> {
    unexcused: &'a [&'a str],
    outlived: &'a [&'a str],
    unknown: &'a [&'a str],
}

/// The work queue: every unbuilt system under the blocker that would have
/// to go first, and what closes that blocker.
fn the_queue(out: &mut String) {
    for blocker in Blocker::ALL {
        let rows: Vec<(&str, &str)> = NOT_BUILT
            .into_iter()
            .filter(|(_, one, _)| *one == blocker)
            .map(|(key, _, why)| (key, why))
            .collect();
        let _ = write!(
            out,
            "## {}: {}\n\n",
            plural(rows.len(), "system"),
            blocker.title()
        );
        let _ = write!(out, "What closes them is {}.\n\n", blocker.what_closes_it());
        out.push_str("| system | why it is not computed |\n|---|---|\n");
        for (key, why) in rows {
            let _ = writeln!(out, "| `{key}` | {why} |");
        }
        out.push('\n');
    }
}

/// The refusal, measured rather than claimed.
fn asking_for_one(out: &mut String, refused: &[Refused]) {
    out.push_str("## Asking for one\n\n");
    let silent: Vec<String> = refused
        .iter()
        .filter(|one| !one.names_it || !one.names_the_built)
        .map(|one| format!("`{}`", one.system))
        .collect();
    let _ = write!(
        out,
        "Every one of the {} was asked of a real founded chart. None of them \
         answered: each came back refused, naming the system asked for and \
         hinting with every system this build does compute. That is what \
         makes the gap a declared one rather than a dead end — a member that \
         answered an empty reading would be indistinguishable from a bug, and \
         nothing but a call can tell the two apart.\n\n",
        plural(refused.len(), "unbuilt system"),
    );
    if let Some(sample) = refused.first() {
        let _ = write!(out, "```text\n{}\n```\n\n", sample.message);
    }
    if !silent.is_empty() {
        let _ = write!(
            out,
            "Refused without naming what it should: {}.\n\n",
            listed(&silent)
        );
    }
}

/// What a consumer can supply that this build does not, and what nobody
/// can supply because no definition expresses it.
fn who_can_supply_one(out: &mut String, supplied: &str) {
    let registrable: Vec<String> = REGISTRABLE.iter().map(|key| format!("`{key}`")).collect();
    out.push_str("## Who can supply one\n\n");
    let _ = write!(
        out,
        "\"Not built\" is not \"not available\". `DashaSystems::register` takes a \
         `UduDefinition` — lords, years and a **nakshatra** reference, checked by \
         the same `UduRow::validate` a shipped row passes — so a consumer holding \
         the text can register the system on their context and ask for it by key, \
         today, with no change here. That covers {} of the {} systems left: {}, \
         each of which is a stated row waiting only on its citation.\n\n\
         The path is walked rather than cited: {}.\n\n\
         **The other {} cannot be supplied by anyone, and that is the finding.** \
         The registry takes nakshatra-seeded rows and nothing else, so a sign-based \
         system a consumer has the text for — `STHIRA`, `VARNADA` — has no \
         definition to arrive as, and neither has a tithi, yoga or karana seed. \
         Under the no-dead-ends mandate that is a gap in the SDK and not in the \
         sources: the text being unsettled blocks *this* build, while a missing \
         definition blocks *everyone*. A `RashiDefinition` beside `UduDefinition` \
         is what would close it.\n\n",
        count(REGISTRABLE.len()),
        count(NOT_BUILT.len()),
        listed(&registrable),
        supplied,
        count(NOT_BUILT.len() - REGISTRABLE.len()),
    );
}

/// The three directions the list is refused in, and the two counts the
/// page rests on.
fn what_the_types_decide(
    out: &mut String,
    sets: &Sets,
    falsified: &Falsified<'_>,
    refused: &[Refused],
) {
    out.push_str("## What the types decide\n\n");
    let claims = [
        Claim::counted(
            "every catalogued system this build does not compute is listed here with a reason",
            falsified.unexcused.len(),
            sets.catalogued - sets.computed,
        ),
        Claim::counted(
            "no reason here outlives its blocker: nothing listed is already computed",
            falsified.outlived.len(),
            NOT_BUILT.len(),
        ),
        Claim::counted(
            "every reason names a system the catalogue names",
            falsified.unknown.len(),
            NOT_BUILT.len(),
        ),
        Claim::counted(
            "every system said to be registrable is one this build does not compute",
            REGISTRABLE
                .iter()
                .filter(|key| !NOT_BUILT.iter().any(|(one, _, _)| one == *key))
                .count(),
            REGISTRABLE.len(),
        ),
        Claim::counted(
            "asking for an unbuilt system is refused and never answered",
            0,
            refused.len(),
        ),
        Claim::counted(
            "the refusal names the system asked for",
            refused.iter().filter(|one| !one.names_it).count(),
            refused.len(),
        ),
        Claim::counted(
            "the refusal names every system this build does compute",
            refused.iter().filter(|one| !one.names_the_built).count(),
            refused.len(),
        ),
        Claim::stated(
            "the counts on this page are read from the types and not written down",
            Verdict::Holds,
            format!(
                "`DashaSystem::ALL` {}, `teistro::dasha::systems()` {}",
                sets.catalogued, sets.computed
            ),
        ),
    ];
    out.push_str(&table(&claims));
    out.push('\n');

    // Each falsification names what falsified it. A count alone would say
    // the list has gone wrong and leave the reader to find out where.
    for (which, phrase) in [
        (falsified.unexcused, "Not computed and not listed"),
        (falsified.outlived, "Listed and already computed"),
        (falsified.unknown, "Listed and not catalogued"),
    ] {
        if which.is_empty() {
            continue;
        }
        let spelled: Vec<String> = which.iter().map(|key| format!("`{key}`")).collect();
        let _ = write!(out, "{phrase}: {}.\n\n", listed(&spelled));
    }
}

pub(crate) fn generate(root: &Path) -> i32 {
    match page(root) {
        Ok(text) => write(root, &[Output::new(PAGE, text)]),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    match page(root) {
        Ok(text) => i32::from(
            check(
                root,
                &[Output::new(PAGE, text)],
                "cargo xtask dasha-coverage",
            ) != 0,
        ),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}
