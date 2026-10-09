//! The measurement pass over **research** (`03-design/research.md` step 1,
//! §3 tests 1 to 5): whether the module's numbers mean what they say.
//!
//! Half of it needs no sky. A seeded synthetic batch carries a family of
//! predicates of spread prevalence, two pairs of them nested in one
//! another as yogas sharing a condition are, and null labellings measure
//! each method's error rate (test 1) and the exact p against the
//! permutation p's interval (test 3); a planted risk ratio measures what
//! each method finds (test 2). The other half founds charts: a sample
//! with no effect whose births peak in the early morning, as births did
//! before induction, read against a uniform expectation and against its
//! own recombined population (test 4); and event studies with no effect
//! under each shuffle (test 5).
//!
//! `cargo xtask research` writes the page; `check-research` regenerates it
//! in memory and fails on any difference.

use std::fmt::Write as _;
use std::path::Path;

use teistro::catalogue::DashaSystem;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::research::{
    AfterBirth, Birth, Cell, Contrast, Design, EventShuffle, EventStudy, EventTest, GroupTest,
    Matrix, Recombine, ReplicateTest, SplitMix64, Study, Subject, Tested, compare,
};
use teistro::rules::Rule;
use teistro::{ChartRequest, Context, Ephemeris, RuleRequest, RuleSet, ShippedRules, UtcOffset};

use crate::generated::{Output, check, write};
use crate::measure::{Claim, table, verdict_of};

const PAGE: &str = "docs/03-design/research-measured.md";

/// The level every rate below is judged at.
const ALPHA: f64 = 0.05;

/// The synthetic batch's size, its independent predicates' prevalences,
/// and the share of cases a null labelling draws.
const CHARTS: usize = 200;
const PREVALENCE: [f64; 4] = [0.05, 0.1, 0.2, 0.3];
const NAMES: [&str; 8] = ["P05", "P10", "P20", "P30", "A30", "A40", "B50", "B60"];
const CASE_SHARE: f64 = 0.3;
/// Null labellings, permutations per test, and planted studies.
const NULLS: u64 = 1_000;
const PERMUTATIONS: u32 = 499;
const PLANTED: u64 = 200;
/// The planted predicate, and the case shares where it holds and where
/// not: a risk ratio of 1.8.
const TARGET: usize = 3;
const PLANTED_SHARES: (f64, f64) = (0.45, 0.25);
/// The predicates that share nothing with the planted one.
const UNRELATED: [usize; 3] = [0, 1, 2];

/// The artefact's sample: births at Paris over 1950 to 1990, half of them
/// between 02:00 and 07:00 local time and the rest at any hour.
const ARTEFACT_BIRTHS: usize = 300;
const REPLICATES: u32 = 49;
const EARLY_SHARE: f64 = 0.5;

/// The event studies: subjects per study, studies per shuffle,
/// permutations, and the ages at the event, a band narrow beside the
/// births' forty years, so that a date shuffle moves the ages far.
const SUBJECTS: usize = 60;
const EVENT_STUDIES: u64 = 20;
const EVENT_PERMUTATIONS: u32 = 199;
const EVENT_AGES: (u64, u64) = (25, 32);

/// 1950 January 1, 0h UTC, and the forty years after it in days.
const FROM_JD: f64 = 2_433_282.5;
const SPAN_DAYS: u64 = 14_610;

pub(crate) fn generate(root: &Path) -> i32 {
    match page() {
        Ok(text) => write(root, &[Output::new(PAGE, text)]),
        Err(why) => {
            eprintln!("research: {why}");
            1
        }
    }
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    match page() {
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask research") != 0),
        Err(why) => {
            eprintln!("check-research: {why}");
            1
        }
    }
}

/// A uniform draw in `[0, 1)` from the top 53 bits.
#[allow(
    clippy::cast_precision_loss,
    reason = "53 bits fit an f64's mantissa exactly"
)]
fn unit(rng: &mut SplitMix64) -> f64 {
    (rng.next_u64() >> 11) as f64 / (1_u64 << 53) as f64
}

/// The upper end of a 99% normal interval on a rate `alpha` measured over
/// `n` trials: a rate above it rejects more often than its level allows.
#[allow(
    clippy::cast_precision_loss,
    reason = "trial counts are far inside f64's exact integers"
)]
fn ceiling(alpha: f64, n: usize) -> f64 {
    alpha + 2.576 * (alpha * (1.0 - alpha) / n.max(1) as f64).sqrt()
}

/// `hits` of `of` as a share.
#[allow(
    clippy::cast_precision_loss,
    reason = "counts are far inside f64's exact integers"
)]
fn share(hits: usize, of: usize) -> f64 {
    if of == 0 {
        0.0
    } else {
        hits as f64 / of as f64
    }
}

/// The synthetic batch: its matrix, and each chart's cells as booleans.
struct Batch {
    matrix: Matrix,
    holds: Vec<[bool; 8]>,
}

/// Four independent predicates of spread prevalence, then two nested
/// pairs (`A30` inside `A40`, `B50` inside `B60`), each pair read off one
/// draw.
fn synthetic(seed: u64) -> Result<Batch, String> {
    let mut rng = SplitMix64::new(seed);
    let holds: Vec<[bool; 8]> = (0..CHARTS)
        .map(|_| {
            let [p0, p1, p2, p3] = PREVALENCE.map(|p| unit(&mut rng) < p);
            let (a, b) = (unit(&mut rng), unit(&mut rng));
            [p0, p1, p2, p3, a < 0.3, a < 0.4, b < 0.5, b < 0.6]
        })
        .collect();
    let mut matrix = Matrix::new(CHARTS);
    for (j, name) in NAMES.iter().enumerate() {
        let column: Vec<Cell> = holds
            .iter()
            .map(|chart| {
                if chart.get(j).copied().unwrap_or(false) {
                    Cell::Present
                } else {
                    Cell::Absent
                }
            })
            .collect();
        matrix.push(*name, &column).map_err(|why| why.to_string())?;
    }
    Ok(Batch { matrix, holds })
}

/// A labelling: each chart a case with `share_of(chart)`, and at least
/// one chart in each group.
fn labels(seed: u64, share_of: impl Fn(usize) -> f64) -> Design {
    let mut rng = SplitMix64::new(seed);
    let mut groups: Vec<u16> = (0..CHARTS)
        .map(|chart| u16::from(unit(&mut rng) < share_of(chart)))
        .collect();
    let cases = groups.iter().filter(|&&g| g == 1).count();
    if cases == 0 || cases == CHARTS {
        if let Some(first) = groups.first_mut() {
            *first = 1 - *first;
        }
    }
    Design::new(groups)
}

fn case_test(seed: u64) -> GroupTest {
    let mut test = GroupTest::new(seed, PERMUTATIONS, Contrast::CaseVsRest { case: 1 });
    test.alpha = Some(ALPHA);
    test
}

const METHODS: [&str; 5] = ["max-T", "Holm", "Bonferroni", "BH", "BY"];

/// What a set of studies rejected, method by method.
#[derive(Default)]
struct Rejections {
    studies: usize,
    /// Per predicate, how often its raw p was under alpha.
    raw: Vec<usize>,
    /// Per predicate, by method, how often its adjusted p was.
    each: Vec<[usize; 5]>,
    /// Studies in which any predicate was under alpha, by method.
    any: [usize; 5],
    /// Rows whose exact p lay outside the permutation p's interval, of
    /// the rows that carry one.
    outside: usize,
    exact_rows: usize,
    /// Rows where Holm's adjustment exceeded Bonferroni's, of all rows.
    holm_above: usize,
    rows: usize,
}

impl Rejections {
    fn add(&mut self, tested: &Tested) {
        self.studies += 1;
        self.raw.resize(tested.rows.len(), 0);
        self.each.resize(tested.rows.len(), [0; 5]);
        let mut any = [false; 5];
        for ((row, raw), each) in tested.rows.iter().zip(&mut self.raw).zip(&mut self.each) {
            let a = &row.adjusted;
            let under = [a.max_t, a.holm, a.bonferroni, a.bh, a.by].map(|p| p <= ALPHA);
            *raw += usize::from(row.p.value <= ALPHA);
            for ((slot, hit), seen) in each.iter_mut().zip(under).zip(&mut any) {
                *slot += usize::from(hit);
                *seen |= hit;
            }
            if let Some(exact) = row.exact {
                self.exact_rows += 1;
                self.outside += usize::from(exact < row.p.low || exact > row.p.high);
            }
            self.holm_above += usize::from(a.holm > a.bonferroni);
            self.rows += 1;
        }
        for (slot, seen) in self.any.iter_mut().zip(any) {
            *slot += usize::from(seen);
        }
    }

    fn raw_of(&self, j: usize) -> usize {
        self.raw.get(j).copied().unwrap_or(0)
    }

    fn method_of(&self, j: usize, method: usize) -> usize {
        self.each
            .get(j)
            .and_then(|each| each.get(method))
            .copied()
            .unwrap_or(0)
    }
}

fn nulls(batch: &Batch) -> Result<Rejections, String> {
    let mut seen = Rejections::default();
    for k in 0..NULLS {
        let design = labels(1_000 + k, |_| CASE_SHARE);
        let tested =
            compare(&batch.matrix, &design, &case_test(k)).map_err(|why| why.to_string())?;
        seen.add(&tested);
    }
    Ok(seen)
}

/// Studies whose case share is raised on the charts the target holds on.
fn planted(batch: &Batch) -> Result<Rejections, String> {
    let mut seen = Rejections::default();
    for k in 0..PLANTED {
        let design = labels(5_000 + k, |chart| {
            let holds = batch
                .holds
                .get(chart)
                .and_then(|row| row.get(TARGET))
                .copied()
                .unwrap_or(false);
            if holds {
                PLANTED_SHARES.0
            } else {
                PLANTED_SHARES.1
            }
        });
        let tested =
            compare(&batch.matrix, &design, &case_test(k)).map_err(|why| why.to_string())?;
        seen.add(&tested);
    }
    Ok(seen)
}

fn place(latitude: f64, longitude: f64) -> Result<Place, String> {
    Ok(Place::new(
        Latitude::try_new(latitude).map_err(|why| why.to_string())?,
        Longitude::try_new(longitude).map_err(|why| why.to_string())?,
        Altitude::try_new(0.0).map_err(|why| why.to_string())?,
    ))
}

fn offset(seconds: i32) -> Result<UtcOffset, String> {
    UtcOffset::try_from_seconds(seconds).map_err(|why| why.to_string())
}

/// A birth on day `day` of the span at `clock` local time (a fraction of
/// a day) and `seconds` east of Greenwich.
#[allow(
    clippy::cast_precision_loss,
    reason = "a day count and an offset are far inside f64's exact integers"
)]
fn born(at: Place, seconds: i32, day: u64, clock: f64) -> Result<Birth, String> {
    let instant = FROM_JD + day as f64 + clock - f64::from(seconds) / 86_400.0;
    Ok(Birth::new(
        JulianDay::<Utc>::literal(instant),
        at,
        offset(seconds)?,
    ))
}

/// Births at Paris with no effect in them, but born as births were before
/// induction: half between 02:00 and 07:00.
fn early_births() -> Result<Vec<Birth>, String> {
    let paris = place(48.8566, 2.3522)?;
    let mut rng = SplitMix64::new(1_955);
    (0..ARTEFACT_BIRTHS)
        .map(|_| {
            let day = rng.below(SPAN_DAYS);
            let clock = if unit(&mut rng) < EARLY_SHARE {
                (2.0 + 5.0 * unit(&mut rng)) / 24.0
            } else {
                unit(&mut rng)
            };
            born(paris, 3_600, day, clock)
        })
        .collect()
}

/// A rule that holds when `planet` stands in one of `houses`.
fn house_rule(key: &str, planet: &str, houses: [u8; 2]) -> Result<Rule, String> {
    let json = serde_json::json!({
        "key": key,
        "category": "research",
        "source": {"text": "research-measured.md: a sector of the diurnal circle"},
        "conditions": [{"type": "planet-in-house", "planet": planet, "houses": houses}],
    });
    serde_json::from_value(json).map_err(|why| format!("{key}: {why}"))
}

/// The Sun and Mars rising (the 12th and 1st) and culminating (the 9th
/// and 10th), whole-sign houses standing in for Gauquelin's sectors.
fn sector_rules() -> Result<RuleSet, String> {
    let rules = [
        house_rule("SUN_RISING", "SUN", [12, 1])?,
        house_rule("SUN_CULMINATING", "SUN", [9, 10])?,
        house_rule("MARS_RISING", "MARS", [12, 1])?,
        house_rule("MARS_CULMINATING", "MARS", [9, 10])?,
    ];
    RuleRequest::default()
        .with_rules(rules)
        .rule_set()
        .map_err(|why| why.to_string())
}

/// One sector's reading of the artefact sample.
struct Sector {
    name: String,
    observed: f64,
    /// The binomial z of the observed share against two houses in twelve.
    uniform_z: f64,
    recombined: f64,
    ratio: f64,
    max_t: f64,
}

#[allow(
    clippy::cast_precision_loss,
    reason = "the sample's size is far inside f64's exact integers"
)]
fn artefact(sdk: &Context) -> Result<Vec<Sector>, String> {
    let births = early_births()?;
    let set = sector_rules()?;
    let request = ChartRequest::at(place(48.8566, 2.3522)?, offset(3_600)?);
    let control = Recombine {
        seed: 1_983,
        replicates: REPLICATES,
        strata: None,
    };
    let test = ReplicateTest::default();
    let tested = sdk
        .research()
        .expected(Study::new(&births, &request, &set), &control, &test)
        .map_err(|why| why.to_string())?
        .value;
    let uniform = 2.0 / 12.0;
    let spread = (uniform * (1.0 - uniform) / births.len() as f64).sqrt();
    tested
        .rows
        .iter()
        .map(|row| {
            let expectation = row.expected.ok_or_else(|| {
                format!(
                    "{}: a recombined row carries its expectation",
                    row.predicate
                )
            })?;
            Ok(Sector {
                name: row.predicate.clone(),
                observed: expectation.observed,
                uniform_z: (expectation.observed - uniform) / spread,
                recombined: expectation.expected,
                ratio: expectation.ratio.unwrap_or(f64::NAN),
                max_t: row.adjusted.max_t,
            })
        })
        .collect()
}

/// Subjects born over the forty years at Kathmandu at any hour, each with
/// an event in the narrow band of ages.
fn subjects(seed: u64) -> Result<Vec<Subject>, String> {
    let kathmandu = place(27.7172, 85.324)?;
    let mut rng = SplitMix64::new(seed);
    let (from, to) = EVENT_AGES;
    (0..SUBJECTS)
        .map(|_| {
            let day = rng.below(SPAN_DAYS);
            let birth = born(kathmandu, 20_700, day, unit(&mut rng))?;
            let age = from * 365 + rng.below((to - from) * 365);
            #[allow(
                clippy::cast_precision_loss,
                reason = "an age in days is far inside f64's exact integers"
            )]
            let event = JulianDay::<Utc>::literal(birth.instant.get() + age as f64);
            Ok(Subject { birth, event })
        })
        .collect()
}

/// How often a null event study's raw p fell under alpha, over the rows a
/// study could read (a rule delivered on some pair and not on all).
struct EventNull {
    under: usize,
    rows: usize,
}

fn event_nulls(sdk: &Context, shuffle: EventShuffle) -> Result<EventNull, String> {
    let set = RuleRequest::shipped([ShippedRules::Yogas])
        .rule_set()
        .map_err(|why| why.to_string())?;
    let request = ChartRequest::at(place(27.7172, 85.324)?, offset(20_700)?);
    let mut seen = EventNull { under: 0, rows: 0 };
    for k in 0..EVENT_STUDIES {
        let lives = subjects(9_000 + k)?;
        let study = EventStudy::new(&lives, &request, &set, DashaSystem::Vimshottari, shuffle);
        let test = EventTest {
            after_birth: AfterBirth::RestrictPairings,
            ..EventTest::new(k, EVENT_PERMUTATIONS)
        };
        let tested = sdk
            .research()
            .timed(study, None, &test)
            .map_err(|why| why.to_string())?
            .value;
        for row in &tested.rows {
            let informative = row
                .expected
                .is_some_and(|e| e.expected > 0.0 && e.expected < 1.0);
            if informative {
                seen.rows += 1;
                seen.under += usize::from(row.p.value <= ALPHA);
            }
        }
    }
    Ok(seen)
}

fn page() -> Result<String, String> {
    let batch = synthetic(2_026)?;
    let null = nulls(&batch)?;
    let found = planted(&batch)?;
    let sdk = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .map_err(|why| why.to_string())?;
    let sectors = artefact(&sdk)?;
    let ages = event_nulls(&sdk, EventShuffle::AgesAtEvent)?;
    let dates = event_nulls(&sdk, EventShuffle::EventDates)?;
    let mut out = String::new();
    header(&mut out);
    calibration(&mut out, &null);
    power(&mut out, &found);
    gauquelin(&mut out, &sectors);
    shuffles(&mut out, &ages, &dates);
    Ok(out)
}

fn header(out: &mut String) {
    let _ = writeln!(
        out,
        "# Research, measured\n\n\
         Status: `generated` by `cargo xtask research`. Do not edit: \
         `cargo xtask check-research` fails on any difference.\n\n\
         Whether the numbers `sdk.research()` answers mean what they say \
         (`research.md` §3). Every batch is seeded, so the page is the same \
         on every run; a rate is judged against the upper end of a 99% \
         interval on its level, since a seeded run is one draw of the \
         sampling noise.\n"
    );
}

fn calibration(out: &mut String, null: &Rejections) {
    let n = null.studies;
    let bound = ceiling(ALPHA, n);
    let over: Vec<&str> = NAMES
        .iter()
        .enumerate()
        .filter(|&(j, _)| share(null.raw_of(j), n) > bound)
        .map(|(_, name)| *name)
        .collect();
    let family_over = METHODS
        .iter()
        .zip(null.any)
        .filter(|&(_, hits)| share(hits, n) > bound)
        .count();
    let exact_bound = ceiling(0.05, null.exact_rows);
    let claims = [
        Claim::counted(
            "on labels that mean nothing, a raw p is under 0.05 no more often than 5%",
            over.len(),
            NAMES.len(),
        ),
        Claim::counted(
            "every correction holds the familywise rate at 0.05 under the complete null",
            family_over,
            METHODS.len(),
        ),
        Claim::stated(
            "the exact p lies inside the permutation p's 95% interval at least 95% of the time",
            verdict_of(share(null.outside, null.exact_rows) <= exact_bound),
            format!("{} of {} rows outside", null.outside, null.exact_rows),
        ),
        Claim::counted(
            "Holm never adjusts above Bonferroni",
            null.holm_above,
            null.rows,
        ),
    ];
    let _ = writeln!(
        out,
        "## 1. Calibration (tests 1 and 3)\n\n\
         A synthetic batch of {CHARTS} charts carries eight predicates: four \
         independent, of prevalence 5% to 30%, and two nested pairs read off \
         one draw each, as yogas sharing a condition are. {NULLS} labellings \
         draw each chart a case with probability {CASE_SHARE}, independently \
         of every predicate, and each is tested case against rest over \
         {PERMUTATIONS} permutations. A rate is judged against {bound:.4}, \
         the upper end of a 99% interval on 0.05 over {n} studies.\n\n{}",
        table(&claims)
    );
    let _ = writeln!(out, "| predicate | raw p under 0.05 |\n|---|--:|");
    for (j, name) in NAMES.iter().enumerate() {
        let _ = writeln!(out, "| `{name}` | {:.3} |", share(null.raw_of(j), n));
    }
    let _ = writeln!(
        out,
        "\n| method | studies with any predicate under 0.05 |\n|---|--:|"
    );
    for (method, hits) in METHODS.iter().zip(null.any) {
        let _ = writeln!(out, "| {method} | {:.3} |", share(hits, n));
    }
    let _ = writeln!(
        out,
        "\nA permutation test conditions on the margins, so a rare predicate \
         cannot reach 0.05 as often as a common one: its rate sits under the \
         level, which is the conservative side.\n"
    );
}

fn power(out: &mut String, found: &Rejections) {
    let n = found.studies;
    let bound = ceiling(ALPHA, n);
    let leaked = UNRELATED
        .iter()
        .filter(|&&j| share(found.raw_of(j), n) > bound)
        .count();
    let claims = [Claim::counted(
        "a predicate that shares nothing with the planted one stays at the null rate",
        leaked,
        UNRELATED.len(),
    )];
    let _ = writeln!(
        out,
        "## 2. Power (test 2)\n\n\
         {PLANTED} labellings over the same batch draw a chart a case with \
         probability {} where `{}` holds and {} where it does not, a risk \
         ratio of {:.1}. What each method finds of the planted predicate:\n",
        PLANTED_SHARES.0,
        NAMES.get(TARGET).copied().unwrap_or_default(),
        PLANTED_SHARES.1,
        PLANTED_SHARES.0 / PLANTED_SHARES.1,
    );
    let _ = writeln!(out, "| method | share of studies under 0.05 |\n|---|--:|");
    let _ = writeln!(out, "| raw | {:.3} |", share(found.raw_of(TARGET), n));
    for (m, method) in METHODS.iter().enumerate() {
        let _ = writeln!(
            out,
            "| {method} | {:.3} |",
            share(found.method_of(TARGET, m), n)
        );
    }
    let _ = writeln!(out, "\n{}", table(&claims));
}

fn gauquelin(out: &mut String, sectors: &[Sector]) {
    let farthest = sectors
        .iter()
        .max_by(|a, b| a.uniform_z.abs().total_cmp(&b.uniform_z.abs()));
    let claims = [
        Claim::stated(
            "a uniform expectation reads a sector of a sample with no effect as beyond chance, \
             at a two-sided p of 0.001 (z beyond ±3.29)",
            verdict_of(farthest.is_some_and(|s| s.uniform_z.abs() > 3.29)),
            farthest.map_or_else(String::new, |s| {
                format!(
                    "`{}` at z {:.1} against 2 houses in 12",
                    s.name, s.uniform_z
                )
            }),
        ),
        Claim::counted(
            "the sample's own recombined population finds no sector under 0.05 by max-T",
            sectors.iter().filter(|s| s.max_t <= ALPHA).count(),
            sectors.len(),
        ),
    ];
    let _ = writeln!(
        out,
        "## 3. The Gauquelin artefact (test 4)\n\n\
         {ARTEFACT_BIRTHS} births at Paris over 1950 to 1990, no effect planted, \
         but born as births were before induction: half between 02:00 and \
         07:00 local time, the rest at any hour. Each sector is a whole-sign \
         pair of houses standing in for Gauquelin's: rising (the 12th and 1st) \
         and culminating (the 9th and 10th). The uniform column is what a \
         study that expects two houses in twelve would report; the recombined \
         column is `expected` over {REPLICATES} replicates, each birth keeping \
         its date and place and taking another's clock time. A uniform \
         expectation is not offered (C365), and this is why.\n\n{}",
        table(&claims)
    );
    let _ = writeln!(
        out,
        "| sector | observed | z against uniform | recombined expectation | ratio | max-T |\n\
         |---|--:|--:|--:|--:|--:|"
    );
    for s in sectors {
        let _ = writeln!(
            out,
            "| `{}` | {:.3} | {:.1} | {:.3} | {:.2} | {:.3} |",
            s.name, s.observed, s.uniform_z, s.recombined, s.ratio, s.max_t
        );
    }
    let z_of = |name: &str| {
        sectors
            .iter()
            .find(|s| s.name == name)
            .map_or(0.0, |s| s.uniform_z)
    };
    let (rising, culminating) = (z_of("SUN_RISING"), z_of("SUN_CULMINATING"));
    let _ = writeln!(
        out,
        "\nThe rising Sun, the sector the early hours fill, reads z {rising:.1} \
         alone: the 12th and 1st hold the Sun for about four hours around \
         sunrise, and at Paris sunrise moves through the year by more than \
         the early peak is wide. The culminating Sun, which the same hours empty, departs {} \
         (z {culminating:.1}), and a study reading one sector would have \
         reported whichever it looked at.\n",
        if culminating.abs() > rising.abs() {
            "further"
        } else {
            "less far"
        },
    );
}

fn shuffles(out: &mut String, ages: &EventNull, dates: &EventNull) {
    let judged = |seen: &EventNull, name: &str| {
        Claim::stated(
            format!("under {name}, a null event study's raw p is under 0.05 no more often than 5%"),
            verdict_of(share(seen.under, seen.rows) <= ceiling(ALPHA, seen.rows)),
            format!(
                "{} of {} rows, {:.3}",
                seen.under,
                seen.rows,
                share(seen.under, seen.rows)
            ),
        )
    };
    let claims = [
        judged(ages, "`AGES_AT_EVENT`"),
        judged(dates, "`EVENT_DATES`"),
    ];
    let _ = writeln!(
        out,
        "## 4. The two event shuffles (test 5)\n\n\
         {EVENT_STUDIES} studies under each shuffle, each of {SUBJECTS} \
         subjects born at Kathmandu at any hour over 1950 to 1990 with one \
         event between the ages of {} and {}, placed with no regard to the \
         chart; the shipped yogas are each tested as delivered by the \
         Vimshottari to the antardasha, over {EVENT_PERMUTATIONS} \
         permutations, a pairing before a birth never drawn. A row counts \
         where the rule is delivered on some pairings and not all. The ages \
         are narrow and the births forty years wide, so a date shuffle reads \
         each person at ages far from any event's (C364).\n\n{}",
        EVENT_AGES.0,
        EVENT_AGES.1,
        table(&claims)
    );
}
