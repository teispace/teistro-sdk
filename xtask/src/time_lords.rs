//! Valens's time lords, measured (`hellenistic-time-lords.md` step 3,
//! C223–C224): where each starts held on every birth, a stored document's
//! reading rebuilt from its start sign, how often Daimon shares Fortune's
//! sign, and whether a life reaches a loosing of the bond at the second
//! level from every start.
//!
//! Valens's worked nativities are the acceptance tests in
//! `crates/dasha/src/releasing.rs`; this page counts how often the choices
//! they leave open fall on real skies.

use std::fmt::Write as _;
use std::path::Path;

use teistro::catalogue::{DashaSystem, Rashi};
use teistro::dasha::{DashaReading, ReleasingDasha, TIME_LORDS, Timeline};
use teistro::quantity::{JulianDay, Utc};
use teistro::settings::YearLength;
use teistro::{Context, Document, Lot};

use crate::births::{Birth, CHARTS, births};
use crate::fortitudes::tropical;
use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, count, fill, median, table};

const PAGE: &str = "docs/03-design/time-lords-measured.md";

/// The age a loosing is looked for by, in calendar years.
const BY_AGE_YEARS: f64 = 80.0;

/// A calendar year, in days: the Julian year Valens's own calendar keeps.
const JULIAN_YEAR_DAYS: f64 = 365.25;

/// The signs a level runs through before it is loosed.
const SIGNS: usize = 12;

/// One birth founded with its three time lords, and the points they start
/// from as the façade reads them.
struct Read {
    fortune: Rashi,
    daimon: Rashi,
    ascendant: Rashi,
    document: Document,
}

impl Read {
    fn of(sdk: &Context, birth: &Birth) -> Result<Read, String> {
        let why = |what: &str, err: teistro::Error| format!("{}: {what}: {err}", birth.name);
        let request = birth.request().with_dashas(TIME_LORDS);
        let document = sdk
            .chart()
            .reading(JulianDay::<Utc>::literal(birth.at()), &request)
            .map_err(|err| why("founding it with the time lords", err))?
            .value;
        let lots = sdk
            .chart()
            .lots(&document, &[Lot::Fortune, Lot::Daimon])
            .map_err(|err| why("reading its lots", err))?;
        let sign = |lot: Lot| {
            lots.lots
                .iter()
                .find(|placed| placed.lot == lot)
                .map(|placed| placed.place.sign)
                .ok_or_else(|| format!("{}: no {lot:?}", birth.name))
        };
        let ascendant = sdk
            .chart()
            .angles(&document)
            .map_err(|err| why("reading its angles", err))?
            .ascendant_deg;
        Ok(Read {
            fortune: sign(Lot::Fortune)?,
            daimon: sign(Lot::Daimon)?,
            ascendant: Rashi::of_longitude(ascendant),
            document,
        })
    }

    fn reading(&self, system: DashaSystem) -> Option<&DashaReading> {
        self.document
            .dashas
            .iter()
            .find(|reading| reading.system == system)
    }

    fn shared(&self) -> bool {
        self.fortune == self.daimon
    }

    /// The sign Valens starts `system` from.
    fn start(&self, system: DashaSystem) -> Rashi {
        match system {
            DashaSystem::ReleasingFortune => self.fortune,
            DashaSystem::ReleasingDaimon if self.shared() => {
                Rashi::of_longitude(self.daimon.start_deg() + 30.0)
            }
            DashaSystem::ReleasingDaimon => self.daimon,
            _ => self.ascendant,
        }
    }
}

/// How many days after birth releasing first reaches a second-level
/// period past the twelfth, one loosed to the opposite sign.
fn first_loosing(dasha: &ReleasingDasha, birth: f64) -> Option<f64> {
    (0..SIGNS)
        .filter_map(|index| dasha.mahadasha(0, index))
        .find_map(|period| {
            (SIGNS..dasha.breadth())
                .find_map(|index| dasha.child(&period, index))
                .map(|child| child.interval.from.get() - birth)
        })
}

/// Whether the façade's cursor, rebuilt from `stored`'s start sign, gives
/// the stored reading back whole.
fn rebuilds(sdk: &Context, read: &Read, system: DashaSystem, stored: &DashaReading) -> bool {
    let Ok(cursor) = sdk.chart().dasha(&read.document, system) else {
        return false;
    };
    let again = match (cursor.releasing(), cursor.profection()) {
        (Some(kernel), _) => DashaReading::of_time_lord(system, kernel, stored.rules, stored.depth),
        (_, Some(kernel)) => DashaReading::of_time_lord(system, kernel, stored.rules, stored.depth),
        _ => return false,
    };
    again == *stored
}

/// Each time lord starts where Valens counts it, and a stored reading
/// rebuilt from its start sign gives back every row it carries.
fn structural_claims(sdk: &Context, reads: &[Read]) -> Vec<Claim> {
    let (mut start_wrong, mut rebuild_wrong) = ([0_usize; 3], 0);
    for read in reads {
        for (slot, system) in TIME_LORDS.into_iter().enumerate() {
            let stored = read.reading(system);
            let starts = stored.and_then(DashaReading::start_sign) == Some(read.start(system));
            if let Some(wrong) = start_wrong.get_mut(slot) {
                *wrong += usize::from(!starts);
            }
            let rebuilt = stored.is_some_and(|stored| rebuilds(sdk, read, system, stored));
            rebuild_wrong += usize::from(!rebuilt);
        }
    }
    let of = reads.len();
    let [fortune, daimon, profection] = start_wrong;
    vec![
        Claim::counted(
            "releasing from Fortune starts at the sign of the Lot of Fortune (IV.4)",
            fortune,
            of,
        ),
        Claim::counted(
            "releasing from Daimon starts at its sign, or at the next when it shares Fortune's (IV.4, C223)",
            daimon,
            of,
        ),
        Claim::counted(
            "the profected year starts at the Ascendant's sign (IV.11)",
            profection,
            of,
        ),
        Claim::counted(
            "a stored reading rebuilt from its start sign gives back every row it carries",
            rebuild_wrong,
            of * TIME_LORDS.len(),
        ),
    ]
}

/// How often the choices the worked examples leave open fall: Daimon in
/// Fortune's sign on the corpus, and a second-level loosing within a life
/// from every start.
fn open_claims(reads: &[Read]) -> Vec<Claim> {
    let shared = reads.iter().filter(|read| read.shared()).count();
    let of = count(reads.len());
    let mut claims = vec![Claim::stated(
        "C223: births whose Daimon falls in Fortune's sign, so that activity is released from the next",
        Verdict::Holds,
        format!("{} of {of}", count(shared)),
    )];
    // Whether a life meets the loosing is the start sign's arithmetic, not
    // the sky's: every one of the twelve is tried, born at day 0.
    let mut latest: Option<(f64, Rashi)> = None;
    let mut never = 0;
    for start in Rashi::ALL {
        let reached = ReleasingDasha::new(start, JulianDay::literal(0.0), YearLength::Savana360)
            .ok()
            .and_then(|dasha| first_loosing(&dasha, 0.0))
            .map(|days| days / JULIAN_YEAR_DAYS);
        match reached {
            Some(age) if age < BY_AGE_YEARS => {
                if latest.is_none_or(|(worst, _)| age > worst) {
                    latest = Some((age, start));
                }
            }
            _ => never += 1,
        }
    }
    let note = latest.map_or_else(String::new, |(age, start)| {
        format!("the latest is released from {start:?}, at {age:.1} calendar years")
    });
    claims.push(
        Claim::counted(
            "C224: releasing from any of the twelve signs reaches a second-level loosing before the age of 80, so its reading decides a period in every life",
            never,
            SIGNS,
        )
        .with_note(note),
    );
    claims
}

/// How many rows a document carries for each time lord, so that what a
/// request for one costs is seen.
fn row_counts(reads: &[Read]) -> String {
    let mut out =
        String::from("| system | depth | rows, median | rows, most |\n|---|---|---|---|\n");
    for system in TIME_LORDS {
        let readings: Vec<&DashaReading> = reads
            .iter()
            .filter_map(|read| read.reading(system))
            .collect();
        let rows = || readings.iter().map(|reading| reading.periods.len());
        #[allow(clippy::cast_precision_loss, reason = "row counts far below 2^53")]
        let middle = median(rows().map(|rows| rows as f64));
        let depth = readings.first().map_or(0, |reading| reading.depth.get());
        let _ = writeln!(
            out,
            "| `{}` | {depth} | {middle} | {} |",
            system.key(),
            count(rows().max().unwrap_or_default())
        );
    }
    out
}

fn page(root: &Path) -> Result<String, String> {
    let sdk = tropical()?;
    let births = births(root, &sdk)?;
    if births.is_empty() {
        return Err(format!("{CHARTS} records no chart"));
    }
    let reads = births
        .iter()
        .map(|birth| Read::of(&sdk, birth))
        .collect::<Result<Vec<_>, String>>()?;
    let claims: Vec<Claim> = structural_claims(&sdk, &reads)
        .into_iter()
        .chain(open_claims(&reads))
        .collect();
    let mut out = String::from(
        "# Valens's time lords, measured\n\n\
         Status: `generated` by `cargo xtask time-lords` from the corpus's \
         recorded births, 2026-10-02. Do not edit: `check-time-lords` \
         regenerates this page and fails on any difference.\n\n\
         Releasing and the profected year (`hellenistic-time-lords.md`) \
         are held to Valens's worked nativities by the unit tests of \
         `crates/dasha/src/releasing.rs`. Those give signs and years from \
         a stated start, so they cannot say how often a real sky puts \
         Daimon in Fortune's sign, or whether a life reaches the \
         loosing of the bond; this page answers both. ",
    );
    let _ = write!(
        out,
        "It founds each of the corpus's {} births in the tropical zodiac \
         with `RELEASING_FORTUNE`, `RELEASING_DAIMON` and `PROFECTION` \
         asked for, and reads their lots through `ChartArea::lots` under \
         Valens's rules.\n\n",
        count(reads.len())
    );
    out.push_str(&table(&claims));
    out.push_str("\n## What a document carries\n\n");
    out.push_str(&row_counts(&reads));
    out.push_str(
        "\n## What it means\n\n\
         The first four rows hold what the façade must do on every birth: \
         start each time lord where Valens counts it, and record enough \
         that a stored chart rebuilds the same periods. The next counts \
         the births C223 decides, where the activity count moves to the \
         sign after Fortune's. The last asks the twelve start signs rather \
         than the births, because whether a life meets the loosing at the \
         second level is arithmetic: it falls only in a sign whose years \
         outlast 17 years 7 months (Gemini, Cancer, Leo, Virgo, Capricorn \
         and Aquarius), one of them begins within any 80 years, and C224's \
         reading of the opposite sign then decides every period after it. \
         A document carries the whole 211-year cycle to its depth, which \
         the row counts above price.\n",
    );
    Ok(fill(&out))
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
        Ok(text) => {
            i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask time-lords") != 0)
        }
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}
