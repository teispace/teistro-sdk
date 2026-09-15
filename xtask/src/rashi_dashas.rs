//! The falsification pass over the sign-based (Jaimini) dashas: Chara,
//! Narayana, Padanadhamsa, Trikona, Drig, Shoola, Niryana Shoola and
//! Mandooka (Phase 5).
//!
//! The corpus's `baseline/rashi-dashas` records each for every fixture with
//! a recorded chart, computed from that chart's lagna, graha signs and
//! dignities, and arudha and navamsha lagnas, which each file repeats. So a
//! rule here is right or wrong against the recording engine, and the page
//! settles what `dasha-kernels.md`'s K-rashi schema left as fields: where
//! each system starts, which way it runs, how long a sign's period is, which
//! of a dual-lorded sign's lords counts, and how the antardashas run.
//!
//! These are the systems where schools disagree most, so every rule is
//! measured beside the readings the schools give instead: the direction by
//! the ninth house's footedness, antardashas beginning from the next sign,
//! the dual lord that is not in the sign, plain parity for the count. What
//! the corpus refuses is the recording engine's choice, and a refusal is not
//! a verdict on a school; the page registers each as a crux.
//!
//! `cargo xtask rashi-dashas` writes the page; `check-rashi-dashas`
//! regenerates it in memory and fails on any difference.

use std::fmt::Write as _;
use std::path::Path;

use serde_json::Value;

use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, count, fill, table, worst};

const PAGE: &str = "docs/03-design/rashi-dashas-measured.md";
const ROOT: &str = "fixtures/baseline";

/// The only year length these records use.
const YEAR: f64 = 365.25;

/// Differences inside this many days are the last bits of a double.
const SAME_DAY: f64 = 1e-8;

const GRAHAS: [&str; 9] = [
    "SUN", "MOON", "MARS", "MERCURY", "JUPITER", "VENUS", "SATURN", "RAHU", "KETU",
];

/// Each sign's first Jaimini lord, as a graha index; Scorpio's and
/// Aquarius's are Ketu and Rahu, whose co-lords are Mars and Saturn.
const LORDS: [usize; 12] = [2, 5, 3, 1, 0, 3, 5, 8, 4, 6, 7, 4];

/// The second lord of a dual-lorded sign.
const fn co_lord(sign: usize) -> Option<usize> {
    match sign {
        7 => Some(2),
        10 => Some(6),
        _ => None,
    }
}

// ── Reading the corpus ─────────────────────────────────────────────────────

/// One recorded chart.
struct Chart {
    lagna: usize,
    arudha: usize,
    navamsha: usize,
    signs: [usize; 9],
    dignities: [String; 9],
}

/// One period: its path, sign, lord and bounds.
#[derive(Clone, Debug, PartialEq)]
struct Row {
    path: String,
    sign: usize,
    lord: String,
    start: f64,
    end: f64,
}

/// One system's answer for one chart.
struct Answer {
    birth: f64,
    chart: usize,
    periods: Vec<Row>,
    active: Vec<(f64, Vec<Row>)>,
}

/// Every chart, and every system's answers over them.
struct Corpus {
    charts: Vec<Chart>,
    systems: Vec<(String, Vec<Answer>)>,
}

fn number(value: &Value, what: &str) -> Result<f64, String> {
    value
        .as_f64()
        .ok_or_else(|| format!("{what} is not a number"))
}

fn index(value: &Value, what: &str) -> Result<usize, String> {
    value
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| format!("{what} is not a whole number"))
}

fn text(value: &Value, what: &str) -> Result<String, String> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("{what} is not a string"))
}

fn row(cells: &Value, path: String, at: usize, what: &str) -> Result<Row, String> {
    Ok(Row {
        path,
        sign: index(&cells[at], what)?,
        lord: text(&cells[at + 1], what)?,
        start: number(&cells[at + 2], what)?,
        end: number(&cells[at + 3], what)?,
    })
}

fn corpus(root: &Path) -> Result<Corpus, String> {
    let mut corpus = Corpus {
        charts: Vec::new(),
        systems: Vec::new(),
    };
    for dir in ["charts", "variants"] {
        let directory = root.join(ROOT).join("rashi-dashas").join(dir);
        let mut paths: Vec<_> = std::fs::read_dir(&directory)
            .map_err(|err| format!("{}: {err}", directory.display()))?
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
            .collect();
        paths.sort();
        for path in paths {
            let at = |err: String| format!("{}: {err}", path.display());
            let file: Value = serde_json::from_str(
                &std::fs::read_to_string(&path).map_err(|err| at(err.to_string()))?,
            )
            .map_err(|err| at(err.to_string()))?;
            let inputs = &file["inputs"];
            let mut signs = [0; 9];
            let mut dignities: [String; 9] = Default::default();
            for (graha, name) in GRAHAS.iter().enumerate() {
                signs[graha] = index(&inputs["graha_sign_index"][name], name).map_err(&at)?;
                dignities[graha] = text(&inputs["graha_dignity"][name], name).map_err(&at)?;
            }
            corpus.charts.push(Chart {
                lagna: index(&inputs["lagna_sign_index"], "lagna").map_err(&at)?,
                arudha: index(&inputs["arudha_lagna_sign_index"], "arudha").map_err(&at)?,
                navamsha: index(&inputs["navamsha_lagna_sign_index"], "navamsha").map_err(&at)?,
                signs,
                dignities,
            });
            let chart = corpus.charts.len() - 1;
            let birth = number(&inputs["jd_ut"], "jd_ut").map_err(&at)?;
            for (key, system) in file["systems"]
                .as_object()
                .ok_or_else(|| at("no systems".into()))?
            {
                let periods = system["periods"]
                    .as_array()
                    .ok_or_else(|| at("no periods".into()))?
                    .iter()
                    .map(|cells| row(cells, text(&cells[0], "path")?, 1, "period"))
                    .collect::<Result<Vec<_>, String>>()
                    .map_err(&at)?;
                let active = system["active_at"]
                    .as_array()
                    .ok_or_else(|| at("no active_at".into()))?
                    .iter()
                    .map(|instant| {
                        let chain = instant["chain"]
                            .as_array()
                            .ok_or("no chain")?
                            .iter()
                            .map(|cells| {
                                row(cells, index(&cells[1], "index")?.to_string(), 2, "link")
                            })
                            .collect::<Result<Vec<_>, String>>()?;
                        Ok((number(&instant["jd"], "jd")?, chain))
                    })
                    .collect::<Result<Vec<_>, String>>()
                    .map_err(&at)?;
                let answer = Answer {
                    birth,
                    chart,
                    periods,
                    active,
                };
                match corpus.systems.iter_mut().find(|(name, _)| name == key) {
                    Some((_, answers)) => answers.push(answer),
                    None => corpus.systems.push((key.clone(), vec![answer])),
                }
            }
        }
    }
    Ok(corpus)
}

// ── The rules proposed ─────────────────────────────────────────────────────

/// Odd by number: Aries, Gemini, Leo… (index 0, 2, 4…).
const fn odd(sign: usize) -> bool {
    sign % 2 == 0
}

/// Odd-footed, by threes from Aries: Aries to Gemini and Libra to
/// Sagittarius.
const fn odd_footed(sign: usize) -> bool {
    (sign / 3) % 2 == 0
}

/// A sign `step` signs on, backwards when negative.
fn on(sign: usize, step: i32) -> usize {
    let at = i32::try_from(sign % 12).unwrap_or(0) + step;
    usize::try_from(at.rem_euclid(12)).unwrap_or(0)
}

/// How a dual-lorded sign's lord is chosen.
#[derive(Clone, Copy)]
enum DualLord {
    /// The one in a kendra from the sign when only one is; else the first.
    Kendra,
    /// Always the first (Ketu, Rahu).
    First,
    /// The other one when one lord occupies the sign; else by kendra.
    OtherWhenOneIsIn,
}

fn lord_of(chart: &Chart, sign: usize, rule: DualLord) -> usize {
    let first = LORDS[sign];
    let Some(second) = co_lord(sign) else {
        return first;
    };
    let from = |graha: usize| (chart.signs[graha] + 12 - sign) % 12;
    let kendra = |graha: usize| from(graha) % 3 == 0;
    let by_kendra = match (kendra(first), kendra(second)) {
        (false, true) => second,
        _ => first,
    };
    match rule {
        DualLord::First => first,
        DualLord::Kendra => by_kendra,
        DualLord::OtherWhenOneIsIn => match (from(first) == 0, from(second) == 0) {
            (true, false) => second,
            (false, true) => first,
            _ => by_kendra,
        },
    }
}

/// How the count from a sign to its lord runs.
#[derive(Clone, Copy)]
enum Footing {
    /// Forward from an odd-footed sign, back from an even-footed one.
    Footed,
    /// Forward from an odd sign, back from an even one.
    Parity,
}

/// The years a sign's period runs by the count to its lord: the distance,
/// twelve when the lord is in the sign.
fn counted_years(chart: &Chart, sign: usize, lord: usize, footing: Footing) -> f64 {
    let forward = match footing {
        Footing::Footed => odd_footed(sign),
        Footing::Parity => odd(sign),
    };
    let at = chart.signs[lord];
    let distance = if forward {
        (at + 12 - sign) % 12
    } else {
        (sign + 12 - at) % 12
    };
    if distance == 0 {
        12.0
    } else {
        f64::from(u8::try_from(distance).unwrap_or(12))
    }
}

/// Every rule a system can take, to propose one reading or its rival.
#[derive(Clone, Copy)]
struct Rules {
    dual: DualLord,
    footing: Footing,
    /// Whether the mahadasha sequence runs by the ninth house's footedness
    /// rather than the start sign's parity.
    ninth: bool,
    /// Whether antardashas begin from the sign after the mahadasha's.
    next: bool,
    /// Whether Narayana's dignity adjustment applies.
    dignity: bool,
}

const RECORDED: Rules = Rules {
    dual: DualLord::Kendra,
    footing: Footing::Footed,
    ninth: false,
    next: false,
    dignity: true,
};

/// The consecutive sequence from `start`, forward or back.
fn consecutive(start: usize, forward: bool) -> Vec<usize> {
    (0..12)
        .map(|step| on(start, if forward { step } else { -step }))
        .collect()
}

/// The mahadasha signs of a system for a chart.
fn sequence(system: &str, chart: &Chart, rules: Rules) -> Vec<usize> {
    let direction = |start: usize| {
        if rules.ninth {
            odd_footed(on(chart.lagna, 8))
        } else {
            odd(start)
        }
    };
    match system {
        "padanadhamsa" => consecutive(chart.arudha, direction(chart.arudha)),
        "niryana_shoola" => consecutive(chart.navamsha, direction(chart.navamsha)),
        "trikona" => {
            let group = chart.lagna % 4;
            (0..4)
                .flat_map(|g| {
                    let base = (group + g) % 4;
                    let trine = [base, base + 4, base + 8];
                    if direction(chart.lagna) {
                        trine
                    } else {
                        [trine[2], trine[1], trine[0]]
                    }
                })
                .collect()
        }
        "drig" => {
            let mut seen = Vec::new();
            for house in [8, 9, 10] {
                let anchor = on(chart.lagna, house);
                for sign in std::iter::once(anchor).chain(aspected(anchor)) {
                    if !seen.contains(&sign) {
                        seen.push(sign);
                    }
                }
            }
            // Signs the chain leaves out follow in the zodiac's order.
            (0..12).for_each(|sign| {
                if !seen.contains(&sign) {
                    seen.push(sign);
                }
            });
            seen
        }
        "mandooka" => (0..12)
            .map(|k: i32| {
                let seed = if k < 6 {
                    chart.lagna
                } else {
                    on(chart.lagna, -1)
                };
                on(seed, -2 * (k % 6))
            })
            .collect(),
        _ => consecutive(chart.lagna, direction(chart.lagna)),
    }
}

/// Whether the ninth, tenth and eleventh houses and the signs they aspect
/// reach all twelve signs by themselves.
fn drishti_chain_covers(chart: &Chart) -> bool {
    let mut seen = [false; 12];
    for house in [8, 9, 10] {
        let anchor = on(chart.lagna, house);
        seen[anchor] = true;
        for sign in aspected(anchor) {
            seen[sign] = true;
        }
    }
    seen.iter().all(|&s| s)
}

/// The signs a sign aspects by rashi drishti, ascending: a movable sign the
/// fixed signs but the next, a fixed sign the movable signs but the one
/// before, a dual sign the other duals.
fn aspected(sign: usize) -> Vec<usize> {
    let target = match sign % 3 {
        0 => 1,
        1 => 0,
        _ => 2,
    };
    (0..12)
        .filter(|&other| {
            other != sign && other % 3 == target && other != on(sign, 1) && other != on(sign, -1)
        })
        .collect()
}

/// A sign's period in years under a system's length rule.
fn years(system: &str, chart: &Chart, sign: usize, rules: Rules) -> f64 {
    match system {
        "shoola" | "niryana_shoola" => 9.0,
        "mandooka" => [7.0, 8.0, 9.0][sign % 3],
        _ => {
            let lord = lord_of(chart, sign, rules.dual);
            let counted = counted_years(chart, sign, lord, rules.footing);
            if system == "narayana" && rules.dignity {
                let adjust = match chart.dignities[lord].as_str() {
                    "EXALTED" | "DEEP_EXALTED" => 1.0,
                    "DEBILITATED" | "DEEP_DEBILITATED" => -1.0,
                    _ => 0.0,
                };
                (counted + adjust).clamp(1.0, 12.0)
            } else {
                counted
            }
        }
    }
}

/// The lord a mahadasha names: the chosen dual lord, except in the systems
/// whose length is not counted to one, which name the first.
fn named_lord(system: &str, chart: &Chart, sign: usize, rules: Rules) -> &'static str {
    let lord = match system {
        "shoola" | "niryana_shoola" => LORDS[sign],
        _ => lord_of(chart, sign, rules.dual),
    };
    GRAHAS[lord]
}

/// The tree to antardashas.
fn tree(system: &str, answer: &Answer, chart: &Chart, rules: Rules) -> Vec<Row> {
    let mut rows = Vec::new();
    let mut at = answer.birth;
    for (i, sign) in sequence(system, chart, rules).into_iter().enumerate() {
        let length = years(system, chart, sign, rules) * YEAR;
        rows.push(Row {
            path: i.to_string(),
            sign,
            lord: named_lord(system, chart, sign, rules).to_owned(),
            start: at,
            end: at + length,
        });
        rows.extend(antardashas(sign, at, length, rules, &i.to_string()));
        at += length;
    }
    rows
}

/// Twelve equal antardashas, forward from an odd sign and back from an
/// even one, each naming its sign's first lord.
fn antardashas(sign: usize, start: f64, length: f64, rules: Rules, path: &str) -> Vec<Row> {
    let step: i32 = if odd(sign) { 1 } else { -1 };
    let first = if rules.next { on(sign, step) } else { sign };
    let share = length / 12.0;
    (0..12)
        .map(|k| {
            let child = on(first, step * k);
            Row {
                path: format!("{path}/{k}"),
                sign: child,
                lord: GRAHAS[LORDS[child]].to_owned(),
                start: start + share * f64::from(k),
                end: start + share * f64::from(k + 1),
            }
        })
        .collect()
}

/// How many rows disagree in path, sign or lord, and the worst bound.
fn rows_agree(got: &[Row], want: &[Row]) -> (usize, f64) {
    let mut wrong = got.len().abs_diff(want.len());
    let mut far = 0.0_f64;
    for (a, b) in got.iter().zip(want) {
        if a.path != b.path || a.sign != b.sign || a.lord != b.lord {
            wrong += 1;
        }
        far = far
            .max((a.start - b.start).abs())
            .max((a.end - b.end).abs());
    }
    (wrong, far)
}

// ── The page ───────────────────────────────────────────────────────────────

fn name(key: &str) -> String {
    key.split('_')
        .map(|word| {
            let mut letters = word.chars();
            letters
                .next()
                .map(|first| first.to_uppercase().chain(letters).collect::<String>())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The answers a rules reading gets wrong, by the mahadashas and by the
/// whole tree.
fn wrong(corpus: &Corpus, keys: &[&str], rules: Rules) -> (usize, usize, usize) {
    let (mut mahadashas, mut trees, mut of) = (0, 0, 0);
    for (key, answers) in corpus
        .systems
        .iter()
        .filter(|(key, _)| keys.contains(&key.as_str()))
    {
        for answer in answers {
            of += 1;
            let chart = &corpus.charts[answer.chart];
            let proposed = tree(key, answer, chart, rules);
            let top = |rows: &[Row]| {
                rows.iter()
                    .filter(|r| !r.path.contains('/'))
                    .cloned()
                    .collect::<Vec<_>>()
            };
            let (w, far) = rows_agree(&top(&proposed), &top(&answer.periods));
            mahadashas += usize::from(w > 0 || far > SAME_DAY);
            let (w, far) = rows_agree(&proposed, &answer.periods);
            trees += usize::from(w > 0 || far > SAME_DAY);
        }
    }
    (mahadashas, trees, of)
}

const COUNTED: [&str; 5] = ["chara", "narayana", "padanadhamsa", "trikona", "drig"];
const CONSECUTIVE: [&str; 5] = [
    "chara",
    "narayana",
    "padanadhamsa",
    "shoola",
    "niryana_shoola",
];
const DUAL: [&str; 6] = [
    "chara",
    "narayana",
    "padanadhamsa",
    "trikona",
    "drig",
    "mandooka",
];

/// The periods running at the recorded instants.
fn chain_claim(corpus: &Corpus) -> Claim {
    let (mut chains, mut chains_wrong, mut past) = (0, 0, 0);
    let mut far = 0.0_f64;
    for (key, answers) in &corpus.systems {
        for answer in answers {
            let chart = &corpus.charts[answer.chart];
            let proposed = tree(key, answer, chart, RECORDED);
            for (jd, recorded) in &answer.active {
                chains += 1;
                past += usize::from(recorded.is_empty());
                let top: Vec<&Row> = proposed.iter().filter(|r| !r.path.contains('/')).collect();
                let found = top.iter().position(|r| r.start <= *jd && *jd < r.end);
                let agrees = match (found, recorded.first(), recorded.get(1)) {
                    (None, None, _) => true,
                    (Some(i), Some(maha), sub) => {
                        let child = proposed.iter().find(|r| {
                            r.path.starts_with(&format!("{i}/")) && r.start <= *jd && *jd < r.end
                        });
                        far = far.max((top[i].start - maha.start).abs());
                        top[i].sign == maha.sign
                            && i.to_string() == maha.path
                            && match (child, sub) {
                                (Some(child), Some(sub)) => {
                                    child.sign == sub.sign
                                        && child.path.ends_with(&format!("/{}", sub.path))
                                }
                                (None, None) => true,
                                _ => false,
                            }
                    }
                    _ => false,
                };
                chains_wrong += usize::from(!agrees);
            }
        }
    }
    Claim::counted(
            "the periods running at an instant are the same tree's, and none runs past the twelfth mahadasha",
            chains_wrong,
            chains,
        )
        .with_note(format!(
            "{} of the instants fall past the cycle, where no second cycle begins; worst start {far:.1e} days",
            count(past)
        ))
}

/// What a rival reading is judged by: the mahadashas alone, or the whole
/// tree when the reading moves only the antardashas.
#[derive(Clone, Copy)]
enum Judged {
    Mahadashas,
    Trees,
}

/// Every rival reading the page measures: what it says, the systems it
/// applies to (none for all), its rules, and what it is judged by.
const RIVALS: [(&str, &[&str], Rules, Judged); 6] = [
    (
        "the mahadashas run forward when the ninth house is odd-footed and back when it is even-footed, rather than by the start sign's parity",
        &CONSECUTIVE,
        Rules {
            ninth: true,
            ..RECORDED
        },
        Judged::Mahadashas,
    ),
    (
        "each mahadasha's antardashas begin from the sign after its own, its own sign last",
        &[],
        Rules {
            next: true,
            ..RECORDED
        },
        Judged::Trees,
    ),
    (
        "a sign's years count to its lord forward from an **odd** sign and back from an even one, rather than by footedness",
        &COUNTED,
        Rules {
            footing: Footing::Parity,
            ..RECORDED
        },
        Judged::Mahadashas,
    ),
    (
        "Scorpio's and Aquarius's periods count to Ketu and Rahu whatever the chart",
        &DUAL,
        Rules {
            dual: DualLord::First,
            ..RECORDED
        },
        Judged::Mahadashas,
    ),
    (
        "when one of a dual-lorded sign's lords occupies it, the period counts to the other",
        &DUAL,
        Rules {
            dual: DualLord::OtherWhenOneIsIn,
            ..RECORDED
        },
        Judged::Mahadashas,
    ),
    (
        "Narayana's years take no adjustment for an exalted or debilitated lord",
        &["narayana"],
        Rules {
            dignity: false,
            ..RECORDED
        },
        Judged::Mahadashas,
    ),
];

fn claims(corpus: &Corpus) -> Vec<Claim> {
    let all: Vec<&str> = corpus.systems.iter().map(|(key, _)| key.as_str()).collect();
    let (_, trees, of) = wrong(corpus, &all, RECORDED);
    let mut claims = vec![Claim::counted(
        "every system's tree is its stated start, order and years, each mahadasha divided into twelve equal antardashas from its own sign, forward for an odd sign and back for an even one",
        trees,
        of,
    )];
    for (rule, systems, rules, judged) in RIVALS {
        let (mahadashas, trees, of) = wrong(
            corpus,
            if systems.is_empty() { &all } else { systems },
            rules,
        );
        claims.push(Claim::counted(
            rule,
            match judged {
                Judged::Mahadashas => mahadashas,
                Judged::Trees => trees,
            },
            of,
        ));
    }
    let short = corpus
        .charts
        .iter()
        .filter(|chart| !drishti_chain_covers(chart))
        .count();
    claims.push(Claim::stated(
        "Drig's ninth, tenth and eleventh houses and the signs each aspects reach all twelve signs",
        if short == 0 {
            Verdict::Holds
        } else {
            Verdict::Falsified
        },
        format!(
            "{} of {} charts leave signs out, which follow in the zodiac's order",
            count(short),
            count(corpus.charts.len())
        ),
    ));
    claims.push(chain_claim(corpus));
    claims
}

fn summary(corpus: &Corpus) -> String {
    let mut out = String::from(
        "| system | answers | tree rows | wrong | worst boundary, days | cycle, years |\n|---|---|---|---|---|---|\n",
    );
    for (key, answers) in &corpus.systems {
        let (mut rows, mut bad, mut far) = (0, 0, 0.0_f64);
        let mut cycles = Vec::new();
        for answer in answers {
            let chart = &corpus.charts[answer.chart];
            let proposed = tree(key, answer, chart, RECORDED);
            let (w, f) = rows_agree(&proposed, &answer.periods);
            rows += answer.periods.len();
            bad += w;
            far = far.max(f);
            cycles.push(
                proposed
                    .iter()
                    .filter(|r| !r.path.contains('/'))
                    .map(|r| (r.end - r.start) / YEAR)
                    .sum::<f64>(),
            );
        }
        let low = cycles.iter().copied().fold(f64::INFINITY, f64::min);
        let high = worst(cycles.iter().copied());
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {far:.1e} | {} |",
            name(key),
            count(answers.len()),
            count(rows),
            count(bad),
            if (high - low).abs() < 1e-9 {
                format!("{high:.0}")
            } else {
                format!("{low:.0} to {high:.0}")
            }
        );
    }
    out
}

fn page(root: &Path) -> Result<String, String> {
    let corpus = corpus(root)?;
    if corpus.systems.is_empty() {
        return Err(String::from("the corpus records no rashi dashas"));
    }
    let mut out = String::new();
    let _ = write!(
        out,
        "# The sign-based dashas, measured\n\n\
         Status: `generated` by `cargo xtask rashi-dashas` over the conformance corpus's \
         `baseline/rashi-dashas`, 2026-09-15. Do not edit: `check-rashi-dashas` regenerates \
         this page and fails on any difference. The design it measures is \
         [`dasha-kernels.md`](dasha-kernels.md)'s K-rashi schema.\n\n\
         The corpus records {systems} sign-based systems for {charts} charts, each computed \
         from the chart's recorded lagna, graha signs and dignities and arudha and navamsha \
         lagnas: {answers} answers, each a tree to antardashas and the periods running at two \
         instants. None has a balance at birth: the first mahadasha runs its whole years from \
         the moment of birth.\n\n\
         ## Each system\n\n{summary}\n\
         ## What the corpus decides\n\n{table}\n",
        systems = count(corpus.systems.len()),
        charts = count(corpus.charts.len()),
        answers = count(corpus.systems.iter().map(|(_, a)| a.len()).sum()),
        summary = summary(&corpus),
        table = table(&claims(&corpus)),
    );
    out.push_str(
        "## What it means for the module\n\n\
         **One kernel, eight rows.** Every system is a start sign, an order, a length rule \
         and the same antardashas, so the K-rashi schema holds as designed: three direction \
         rules on two kinds of odd, a dual-lord rule, and a sub-progression.\n\n\
         **The recording engine takes one school's reading where the schools differ, and the \
         corpus can only confirm which.** Each rival reading below is taught by a school, \
         none was checked against a rank-1 text, and each is registered as a crux: the \
         mahadashas' direction by the start sign's parity or by the ninth house's footedness \
         (C49); antardashas from the mahadasha's own sign or from the next (C50); a dual-lorded \
         sign's lord by kendra, or by the ladder whose first step counts to the lord not in \
         the sign (C51); Drig's order when the ninth, tenth and eleventh houses' aspects repeat \
         a sign, where the engine appends the signs left out (C52); and the start sign and \
         second cycle the systems take (C53). So each is a field of the row with the engine's \
         reading as its value and its rival expressible, and nothing here says which school \
         is right.\n\n\
         **Footedness is not parity.** Counting a sign's years by plain parity instead of by \
         threes from Aries is refused by most charts, which is the design page's direction \
         error measured: the two kinds of odd are distinct types.\n",
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
            i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask rashi-dashas") != 0)
        }
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn footedness_and_parity_are_different_odds() {
        let footed: Vec<usize> = (0..12).filter(|&s| odd_footed(s)).collect();
        assert_eq!(footed, [0, 1, 2, 6, 7, 8]);
        let odd_signs: Vec<usize> = (0..12).filter(|&s| odd(s)).collect();
        assert_eq!(odd_signs, [0, 2, 4, 6, 8, 10]);
    }

    #[test]
    fn a_sign_aspects_what_rashi_drishti_says() {
        assert_eq!(aspected(0), [4, 7, 10], "Aries: the fixed signs but Taurus");
        assert_eq!(
            aspected(1),
            [3, 6, 9],
            "Taurus: the movable signs but Aries"
        );
        assert_eq!(aspected(2), [5, 8, 11], "Gemini: the other duals");
    }
}
