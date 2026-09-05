//! The calendar tasks: `calendars bs-fit` measures the Bikram Sambat
//! engine against the official table under every frame and publishes the
//! numbers; `gen calendars` writes the shipped table from the official
//! rows and the engine; `check-calendars` regenerates in memory and fails
//! on any difference, so the checked-in table can never drift from its
//! sources or its engine.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use serde::Deserialize;
use teistro_calendar::bikram_sambat::{Divergence, Engine, FitReport, KATHMANDU, YearRow, fit};
use teistro_calendar::gregorian::fixed_from_gregorian;
use teistro_calendar::solar::MonthStartRule;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::time::{LocalClock, LocalMeanTime};
use teistro_siddhanta::{Parameters, SuryaSiddhanta, Trig};
use teistro_time::zones;

/// The span the shipped table covers, Bikram Sambat years.
const SHIPPED_SPAN: (i32, i32) = (1700, 2500);

/// The rule the shipped table is computed under: the measurement's best,
/// as `docs/calendars/bikram-sambat.md` publishes it.
const SHIPPED_RULE: MonthStartRule = MonthStartRule::Punyakala;

/// The last official year the fit runs to (the table's own end).
const OFFICIAL_DATA: &str = "crates/calendar/data/bikram-sambat.json";
const TABLE_FILE: &str = "crates/calendar/src/bikram_sambat/generated.rs";
const FIT_FILE: &str = "crates/calendar/data/bikram-sambat-fit.json";

#[derive(Deserialize)]
struct OfficialFile {
    authority: String,
    edition: String,
    anchor: Anchor,
    years: BTreeMap<String, Vec<u8>>,
}

#[derive(Deserialize)]
struct Anchor {
    year: i32,
    gregorian: String,
}

/// The official table as loaded.
struct Official {
    authority: String,
    edition: String,
    anchor_year: i32,
    anchor_gregorian: (i32, u8, u8),
    rows: Vec<(i32, [u8; 12])>,
}

impl Official {
    fn load(root: &Path) -> Official {
        let text = crate::read(&root.join(OFFICIAL_DATA));
        let file: OfficialFile = serde_json::from_str(&text).expect("a well-formed official table");
        let mut rows = Vec::new();
        for (year, months) in &file.years {
            let year: i32 = year.parse().expect("a year key");
            let months: [u8; 12] = months
                .as_slice()
                .try_into()
                .unwrap_or_else(|_| panic!("{year} has {} months", months.len()));
            rows.push((year, months));
        }
        let mut parts = file
            .anchor
            .gregorian
            .split('-')
            .map(|p| p.parse::<i32>().expect("a date part"));
        let (y, m, d) = (
            parts.next().expect("year"),
            parts.next().expect("month"),
            parts.next().expect("day"),
        );
        Official {
            authority: file.authority,
            edition: file.edition,
            anchor_year: file.anchor.year,
            anchor_gregorian: (
                y,
                u8::try_from(m).expect("month"),
                u8::try_from(d).expect("day"),
            ),
            rows,
        }
    }

    fn first_year(&self) -> i32 {
        self.rows.first().map_or(0, |r| r.0)
    }

    fn last_year(&self) -> i32 {
        self.rows.last().map_or(0, |r| r.0)
    }

    /// The fixed day of 1 Baisakh of the first official year, from the
    /// anchor and the rows between.
    fn first_start(&self) -> teistro_calendar::FixedDay {
        let (y, m, d) = self.anchor_gregorian;
        let mut start = fixed_from_gregorian(y, m, d);
        for (year, months) in &self.rows {
            if *year < self.anchor_year {
                start = start.plus_days(-months.iter().map(|x| i64::from(*x)).sum::<i64>());
            }
        }
        // The anchor is inside the rows; walk back from it to the first.
        let (y, m, d) = self.anchor_gregorian;
        let mut start_of_anchor = fixed_from_gregorian(y, m, d);
        for (year, months) in self.rows.iter().rev() {
            if *year < self.anchor_year {
                start_of_anchor =
                    start_of_anchor.plus_days(-months.iter().map(|x| i64::from(*x)).sum::<i64>());
            }
        }
        debug_assert_eq!(start, start_of_anchor);
        start_of_anchor
    }
}

/// A model variant of the measurement.
struct ModelVariant {
    label: &'static str,
    model: SuryaSiddhanta,
}

fn models() -> Vec<ModelVariant> {
    let mut count_plus_one = Parameters::TEXT;
    count_plus_one.epoch_jd_ut -= 1.0;
    vec![
        ModelVariant {
            label: "text, table",
            model: SuryaSiddhanta::text(),
        },
        ModelVariant {
            label: "text, exact trig",
            model: SuryaSiddhanta::new(Parameters::TEXT, Trig::Exact),
        },
        ModelVariant {
            label: "tradition's count (+1 day), table",
            model: SuryaSiddhanta::new(count_plus_one, Trig::Table),
        },
    ]
}

/// A clock variant of the measurement.
struct ClockVariant {
    label: &'static str,
    clock: Box<dyn LocalClock>,
}

fn clocks() -> Vec<ClockVariant> {
    vec![
        ClockVariant {
            label: "Nepal's clock history",
            clock: Box::new(zones::nepal().clone()),
        },
        ClockVariant {
            label: "Kathmandu local mean time",
            clock: Box::new(LocalMeanTime::new(KATHMANDU.longitude)),
        },
    ]
}

/// The sankrantis of every official year under a model, found once.
fn sankrantis(engine: &Engine<'_>, official: &Official) -> Vec<(i32, [JulianDay<Utc>; 13])> {
    (official.first_year()..=official.last_year())
        .map(|year| {
            (
                year,
                engine
                    .sankrantis(year)
                    .unwrap_or_else(|e| panic!("{year}: {e}")),
            )
        })
        .collect()
}

fn rows_under(
    engine: &Engine<'_>,
    instants: &[(i32, [JulianDay<Utc>; 13])],
) -> Result<Vec<YearRow>, teistro_core::Error> {
    instants
        .iter()
        .map(|(year, found)| engine.year_from(*year, found))
        .collect()
}

/// Runs the measurement and prints the table; writes nothing. With
/// `--detail`, prints where the shipped frame's divergences fall: the
/// local time of each divergent boundary, their spread over the day and
/// over the decades, which is what tells a uniform offset from a drift.
pub(crate) fn bs_fit(root: &Path, detail: bool) -> i32 {
    let official = Official::load(root);
    if detail {
        return bs_fit_detail(&official);
    }
    let first_start = official.first_start();
    println!(
        "official span BS {} to {} ({} years), 1 Baisakh {} = {:?}",
        official.first_year(),
        official.last_year(),
        official.rows.len(),
        official.anchor_year,
        official.anchor_gregorian
    );
    println!();
    println!("{}", FitReport::MARKDOWN_HEADER);
    let mut reports: Vec<FitReport> = Vec::new();
    for model in models() {
        for clock in clocks() {
            let probe = Engine::new(
                &model.model,
                clock.clock.as_ref(),
                KATHMANDU,
                MonthStartRule::SankrantiDay,
            );
            let instants = sankrantis(&probe, &official);
            let mut rules: Vec<MonthStartRule> = MonthStartRule::NAMED.to_vec();
            // The shift family, scanned in steps of 0.005 of a day over a
            // whole day: the best shift, and the plateau of shifts that tie
            // with it, since a uniform shift of every boundary changes no
            // length and only the boundaries near it decide.
            let mut curve: Vec<(f64, u32)> = Vec::new();
            for step in -100i32..100 {
                let days = f64::from(step) * 0.005;
                let rule = MonthStartRule::Shifted { days };
                let engine = Engine::new(&model.model, clock.clock.as_ref(), KATHMANDU, rule);
                if let Ok(rows) = rows_under(&engine, &instants) {
                    let report = fit("", &rows, &official.rows, first_start);
                    curve.push((days, report.months_matched));
                }
            }
            if let Some(&(_, top)) = curve.iter().max_by_key(|(_, matched)| *matched) {
                let plateau: Vec<f64> = curve
                    .iter()
                    .filter(|(_, m)| *m == top)
                    .map(|(f, _)| *f)
                    .collect();
                let worst = curve.iter().map(|(_, m)| *m).min().unwrap_or(0);
                println!(
                    "| shift scan: {}; {} | best {top} at {:+.3} to {:+.3} days ({} of 200 shifts), worst {worst} | | | | |",
                    model.label,
                    clock.label,
                    plateau.first().copied().unwrap_or(0.0),
                    plateau.last().copied().unwrap_or(0.0),
                    plateau.len()
                );
                if let Some(days) = plateau
                    .iter()
                    .copied()
                    .min_by(|a, b| a.abs().total_cmp(&b.abs()))
                {
                    rules.push(MonthStartRule::Shifted { days });
                }
            }
            for rule in rules {
                let engine = Engine::new(&model.model, clock.clock.as_ref(), KATHMANDU, rule);
                let label = format!("{}; {}; {}", model.label, clock.label, rule);
                match rows_under(&engine, &instants) {
                    Ok(rows) => {
                        let report = fit(&label, &rows, &official.rows, first_start);
                        println!("{}", report.markdown_row());
                        reports.push(report);
                    }
                    Err(error) => println!("| {label} | refused: {error} | | | | |"),
                }
            }
        }
    }
    reports.sort_by_key(|report| std::cmp::Reverse(report.months_matched));
    if let Some(best) = reports.first() {
        println!();
        println!("best: {}", best.markdown_row());
        println!(
            "divergent months under the best frame: {}",
            best.divergences.len()
        );
    }
    0
}

/// Where the shipped frame's divergences fall.
#[allow(
    clippy::too_many_lines,
    reason = "one report, printed section by section"
)]
fn bs_fit_detail(official: &Official) -> i32 {
    let (model, clock) = shipped_frame();
    let engine = Engine::new(&model, clock, KATHMANDU, SHIPPED_RULE);
    let first_start = official.first_start();
    let instants = sankrantis(&engine, official);
    let rows = rows_under(&engine, &instants).expect("the shipped frame computes");
    let report = fit(&engine.describe(), &rows, &official.rows, first_start);
    println!("{}", report.markdown_row());
    println!();
    println!(
        "| year | month | official | computed | boundary sankranti, local | hours from midnight |"
    );
    println!("|---|---|---:|---:|---|---:|");
    let mut hours_all: Vec<f64> = Vec::new();
    let mut hours_divergent: Vec<f64> = Vec::new();
    let mut per_decade: BTreeMap<i32, u32> = BTreeMap::new();
    for (year, found) in &instants {
        for (index, instant) in found.iter().enumerate().skip(1) {
            let local = clock.local_jd(*instant) + 0.5;
            let fraction = local - local.floor();
            let hours = fraction * 24.0;
            let signed = if hours > 12.0 { hours - 24.0 } else { hours };
            hours_all.push(signed);
            let month = u8::try_from(index).expect("small");
            if let Some(d) = report
                .divergences
                .iter()
                .find(|d| d.year == *year && d.month == month)
            {
                hours_divergent.push(signed);
                *per_decade.entry(year / 10 * 10).or_default() += 1;
                println!(
                    "| {} | {} | {} | {} | {:02}:{:02} | {:+.2} |",
                    d.year,
                    d.month,
                    d.tabular,
                    d.computed,
                    hours.floor(),
                    ((hours - hours.floor()) * 60.0).floor(),
                    signed
                );
            }
        }
    }
    println!();
    let histogram = |label: &str, values: &[f64]| {
        let mut bins = [0u32; 12];
        for v in values {
            let bin = ((v + 12.0) / 2.0).floor().clamp(0.0, 11.0);
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let bin = bin as usize;
            bins[bin] += 1;
        }
        println!("{label}: two-hour bins from -12 h to +12 h around local midnight: {bins:?}");
    };
    histogram("all boundaries", &hours_all);
    histogram("divergent boundaries", &hours_divergent);
    println!("divergences per decade: {per_decade:?}");
    println!();
    // The official start of every month, from the anchor and the rows.
    let mut official_starts: BTreeMap<(i32, u8), teistro_calendar::FixedDay> = BTreeMap::new();
    let mut start = first_start;
    for (year, months) in &official.rows {
        for (index, length) in months.iter().enumerate() {
            official_starts.insert((*year, u8::try_from(index + 1).expect("small")), start);
            start = start.plus_days(i64::from(*length));
        }
        official_starts.insert((*year, 13), start);
    }
    // Per sign: how many of the 126 boundaries each rule reproduces, and
    // the plateau of uniform shifts (days added before taking the civil
    // day) that reproduces the most.
    let place = KATHMANDU;
    let boundary_matches =
        |sign: u8,
         place_day: &dyn Fn(JulianDay<Utc>) -> Option<teistro_calendar::FixedDay>|
         -> u32 {
            let mut matched = 0;
            for (year, found) in &instants {
                let index = usize::from(sign);
                let (instant, official_start) = if sign == 0 {
                    (found[0], official_starts.get(&(*year, 1)))
                } else {
                    (found[index], official_starts.get(&(*year, sign + 1)))
                };
                if let (Some(day), Some(official_start)) = (place_day(instant), official_start)
                    && day == *official_start
                {
                    matched += 1;
                }
            }
            matched
        };
    println!(
        "| sign | civil day | following day | sunrise to sunrise | before sunset | before aparahna | best shift plateau (days) | matches |"
    );
    println!("|---|---:|---:|---:|---:|---:|---|---:|");
    for sign in 0..12u8 {
        let under = |rule: MonthStartRule| {
            boundary_matches(sign, &|instant| {
                rule.month_start(sign, instant, clock, &model, &place).ok()
            })
        };
        let civil = under(MonthStartRule::SankrantiDay);
        let following = under(MonthStartRule::FollowingDay);
        let sunrise = under(MonthStartRule::SunriseToSunrise);
        let sunset = under(MonthStartRule::BeforeSunset);
        let aparahna = under(MonthStartRule::BeforeAparahna);
        let mut curve: Vec<(f64, u32)> = Vec::new();
        for step in -100i32..=100 {
            let shift = f64::from(step) * 0.005;
            let matched = boundary_matches(sign, &|instant| {
                MonthStartRule::Shifted { days: shift }
                    .month_start(sign, instant, clock, &model, &place)
                    .ok()
            });
            curve.push((shift, matched));
        }
        let top = curve.iter().map(|(_, m)| *m).max().unwrap_or(0);
        let plateau: Vec<f64> = curve
            .iter()
            .filter(|(_, m)| *m == top)
            .map(|(s, _)| *s)
            .collect();
        println!(
            "| {sign} | {civil} | {following} | {sunrise} | {sunset} | {aparahna} | {:+.3} to {:+.3} | {top} |",
            plateau.first().copied().unwrap_or(0.0),
            plateau.last().copied().unwrap_or(0.0)
        );
    }
    println!();
    // The punya-kala rule (the two ayana sankrantis by their punya-kala,
    // the rest by the civil day), and where it still differs.
    let rows = rows_under(&engine, &instants).expect("computes");
    let report = fit(
        &MonthStartRule::Punyakala.key(),
        &rows,
        &official.rows,
        first_start,
    );
    println!("{}", report.markdown_row());
    println!(
        "remaining divergences ({}): boundary local time, sunrise and sunset that day",
        report.divergences.len()
    );
    for d in &report.divergences {
        let Some((_, found)) = instants.iter().find(|(y, _)| *y == d.year) else {
            continue;
        };
        let instant = found[usize::from(d.month)];
        let local = clock.local_jd(instant) + 0.5;
        let hours = (local - local.floor()) * 24.0;
        let (day, _) = teistro_calendar::FixedDay::from_local_jd(clock.local_jd(instant));
        let arc = teistro_calendar::solar::SolarModel::day_arc(&model, day, &place)
            .ok()
            .flatten();
        let fmt = |jd: f64| {
            let l = clock.local_jd(JulianDay::try_new(jd).expect("finite")) + 0.5;
            let h = (l - l.floor()) * 24.0;
            format!("{:02}:{:02}", h.floor(), ((h - h.floor()) * 60.0).floor())
        };
        println!(
            "  {} month {} official {} computed {}: sankranti {:02}:{:02}, sunrise {}, sunset {}",
            d.year,
            d.month,
            d.tabular,
            d.computed,
            hours.floor(),
            ((hours - hours.floor()) * 60.0).floor(),
            arc.map_or(String::from("-"), |a| fmt(a.sunrise.get())),
            arc.map_or(String::from("-"), |a| fmt(a.sunset.get()))
        );
    }
    0
}

/// The shipped engine: the text's model over Nepal's clock at Kathmandu
/// under the shipped rule.
fn shipped_frame() -> (SuryaSiddhanta, &'static teistro_time::ZoneClock) {
    (SuryaSiddhanta::text(), zones::nepal())
}

/// Renders the generated table and the fit report for the shipped frame.
#[allow(
    clippy::too_many_lines,
    reason = "one generated file, written top to bottom"
)]
fn render(root: &Path) -> (String, String) {
    let official = Official::load(root);
    let (model, clock) = shipped_frame();
    let engine = Engine::new(&model, clock, KATHMANDU, SHIPPED_RULE);
    let frame = engine.describe();
    let (first, last) = SHIPPED_SPAN;
    assert!(
        first < official.first_year() && last > official.last_year(),
        "the span must enclose the official rows"
    );
    let computed = engine
        .span(first, last)
        .unwrap_or_else(|e| panic!("computing {first} to {last}: {e}"));
    let report = fit(&frame, &computed, &official.rows, official.first_start());
    assert!(
        report.drift_within_a_day(),
        "the shipped frame drifts: {report}"
    );

    let mut rows: Vec<[u8; 12]> = Vec::new();
    let mut model_rows: Vec<(i32, i8, [u8; 12])> = Vec::new();
    let official_by_year: BTreeMap<i32, [u8; 12]> = official.rows.iter().copied().collect();
    let mut official_start = official.first_start();
    for row in &computed {
        if let Some(months) = official_by_year.get(&row.year) {
            rows.push(*months);
            if report.divergences.iter().any(|d| d.year == row.year) {
                let offset = i8::try_from(official_start.days_until(row.start))
                    .expect("an offset within a day or two");
                model_rows.push((row.year, offset, row.months));
            }
            official_start = official_start.plus_days(months.iter().map(|m| i64::from(*m)).sum());
        } else {
            rows.push(row.months);
        }
    }

    let mut out = String::new();
    let _ = writeln!(
        out,
        "//! GENERATED by `cargo xtask gen calendars` from `{OFFICIAL_DATA}` (the"
    );
    let _ = writeln!(
        out,
        "//! official rows, verbatim) and the Bikram Sambat engine (every other row);"
    );
    let _ = writeln!(
        out,
        "//! never hand-edited: `cargo xtask check-calendars` regenerates and compares."
    );
    let _ = writeln!(out, "//!");
    let _ = writeln!(out, "//! Frame: {frame}.");
    let _ = writeln!(
        out,
        "//! Fit over the official span: {}/{} months ({:.1} %), {}/{} years exact,",
        report.months_matched,
        report.months,
        report.month_agreement(),
        report.years_exact,
        report.years
    );
    let _ = writeln!(
        out,
        "//! drift {} (max {}), 1 Baisakh offset at most {} day(s), {} divergent month(s).",
        report.drift_end,
        report.drift_max,
        report.start_offset_max,
        report.divergences.len()
    );
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "#![allow(clippy::unreadable_literal, clippy::too_many_lines, reason = \"generated tables\")]"
    );
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "use crate::bikram_sambat::{{Divergence, ModelRow, Table}};"
    );
    let _ = writeln!(out, "use crate::solar::MonthStartRule;");
    let _ = writeln!(out);
    let _ = writeln!(out, "/// The first year of the table.");
    let _ = writeln!(out, "pub const FIRST_YEAR: i32 = {first};");
    let _ = writeln!(out, "/// The last year of the table.");
    let _ = writeln!(out, "pub const LAST_YEAR: i32 = {last};");
    let _ = writeln!(out, "/// The first year of the official span.");
    let _ = writeln!(
        out,
        "pub const OFFICIAL_FIRST_YEAR: i32 = {};",
        official.first_year()
    );
    let _ = writeln!(out, "/// The last year of the official span.");
    let _ = writeln!(
        out,
        "pub const OFFICIAL_LAST_YEAR: i32 = {};",
        official.last_year()
    );
    let _ = writeln!(out, "/// The rule the computed rows follow.");
    let _ = writeln!(
        out,
        "pub(super) const RULE: MonthStartRule = {};",
        rust_rule(SHIPPED_RULE)
    );
    let _ = writeln!(out, "/// The table.");
    let _ = writeln!(out, "pub(super) static TABLE: Table = Table {{");
    let _ = writeln!(out, "    first_year: FIRST_YEAR,");
    let _ = writeln!(out, "    month_lengths: &MONTH_LENGTHS,");
    let (ay, (gy, gm, gd)) = (official.anchor_year, official.anchor_gregorian);
    let _ = writeln!(out, "    anchor: ({ay}, ({gy}, {gm}, {gd})),");
    let _ = writeln!(
        out,
        "    official: (OFFICIAL_FIRST_YEAR, OFFICIAL_LAST_YEAR),"
    );
    let _ = writeln!(out, "    authority: {:?},", official.authority);
    let _ = writeln!(out, "    edition: {:?},", official.edition);
    let _ = writeln!(out, "    frame: {frame:?},");
    let _ = writeln!(out, "    divergences: &DIVERGENCES,");
    let _ = writeln!(out, "    model_rows: &MODEL_ROWS,");
    let _ = writeln!(out, "}};");
    let _ = writeln!(
        out,
        "/// Month lengths per year, `[FIRST_YEAR ..= LAST_YEAR]`."
    );
    let _ = writeln!(out, "static MONTH_LENGTHS: [[u8; 12]; {}] = [", rows.len());
    for (i, months) in rows.iter().enumerate() {
        let year = first + i32::try_from(i).expect("a small index");
        let tag = if official_by_year.contains_key(&year) {
            "official"
        } else {
            "computed"
        };
        let _ = writeln!(out, "    {}, // {year} {tag}", months_literal(months));
    }
    let _ = writeln!(out, "];");
    let _ = writeln!(
        out,
        "/// The months inside the official span where the engine differs."
    );
    let _ = writeln!(
        out,
        "static DIVERGENCES: [Divergence; {}] = [",
        report.divergences.len()
    );
    for d in &report.divergences {
        let _ = writeln!(
            out,
            "    Divergence {{ year: {}, month: {}, tabular: {}, computed: {} }},",
            d.year, d.month, d.tabular, d.computed
        );
    }
    let _ = writeln!(out, "];");
    let _ = writeln!(
        out,
        "/// The engine's rows for the years with a divergence."
    );
    let _ = writeln!(
        out,
        "static MODEL_ROWS: [ModelRow; {}] = [",
        model_rows.len()
    );
    for (year, offset, months) in &model_rows {
        let _ = writeln!(
            out,
            "    ModelRow {{ year: {year}, start_offset: {offset}, months: {} }},",
            months_literal(months)
        );
    }
    let _ = writeln!(out, "];");

    let fit_json = fit_report_json(&report, &official);
    (out, fit_json)
}

fn months_literal(months: &[u8; 12]) -> String {
    let parts: Vec<String> = months.iter().map(u8::to_string).collect();
    format!("[{}]", parts.join(", "))
}

fn rust_rule(rule: MonthStartRule) -> String {
    match rule {
        MonthStartRule::SankrantiDay => String::from("MonthStartRule::SankrantiDay"),
        MonthStartRule::FollowingDay => String::from("MonthStartRule::FollowingDay"),
        MonthStartRule::Shifted { days } => format!("MonthStartRule::Shifted {{ days: {days:?} }}"),
        MonthStartRule::SunriseToSunrise => String::from("MonthStartRule::SunriseToSunrise"),
        MonthStartRule::BeforeSunset => String::from("MonthStartRule::BeforeSunset"),
        MonthStartRule::BeforeAparahna => String::from("MonthStartRule::BeforeAparahna"),
        MonthStartRule::Punyakala => String::from("MonthStartRule::Punyakala"),
    }
}

/// The published report of the shipped frame.
fn fit_report_json(report: &FitReport, official: &Official) -> String {
    let divergences: Vec<serde_json::Value> = report
        .divergences
        .iter()
        .map(|d: &Divergence| {
            serde_json::json!({
                "year": d.year,
                "month": d.month,
                "tabular": d.tabular,
                "computed": d.computed,
            })
        })
        .collect();
    let value = serde_json::json!({
        "calendar": "BIKRAM_SAMBAT",
        "generator": "cargo xtask gen calendars",
        "official": {
            "authority": official.authority,
            "edition": official.edition,
            "span": [official.first_year(), official.last_year()],
        },
        "shipped_span": [SHIPPED_SPAN.0, SHIPPED_SPAN.1],
        "frame": report.frame,
        "months": {"compared": report.months, "matched": report.months_matched, "agreement_percent": (report.month_agreement() * 100.0).round() / 100.0},
        "years": {"compared": report.years, "exact": report.years_exact, "totals_matched": report.totals_matched},
        "drift": {"end": report.drift_end, "max": report.drift_max, "within_a_day": report.drift_within_a_day()},
        "start_offset_max_days": report.start_offset_max,
        "divergences": divergences,
    });
    let mut text = serde_json::to_string_pretty(&value).expect("serialisable");
    text.push('\n');
    text
}

/// Writes the table and the report.
pub(crate) fn generate(root: &Path) -> i32 {
    crate::generated::write(root, &outputs(root))
}

/// Regenerates in memory and compares with the checked-in files.
pub(crate) fn check(root: &Path) -> i32 {
    crate::generated::check(root, &outputs(root), "cargo xtask gen calendars")
}

fn outputs(root: &Path) -> Vec<crate::generated::Output> {
    let (table, report) = render(root);
    vec![
        crate::generated::Output {
            path: TABLE_FILE,
            text: table,
        },
        crate::generated::Output {
            path: FIT_FILE,
            text: report,
        },
    ]
}
