//! The A/B's agreement and accuracy (`03-design/ab-baseline.md` §10,
//! §11): the record's comparison lines, the page's accuracy section, and
//! the gate's floors and causes.
//!
//! An agreement line compares the two sides' facts of one family on one
//! request under one of the SDK's conventions; a reference line compares
//! one side's facts with a rung's. The floors are the shares the record
//! measured, so a change that agrees less, or strays further from a
//! reference, fails `check-ab` as a slower request does.

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use super::{Excused, REQUESTS, SUNLESS};

/// The SDK's two conventions (§10), in the order the page prints them.
const CONVENTIONS: [&str; 2] = ["timed", "same"];

/// What a refusal under the baseline's polar day says when a rule needs a
/// sunrise nearer than the synthesised one (§10).
const SINCE_SUNRISE: &str = "hours since sunrise";

/// The rungs (§11): each one's name in the record, its title and its
/// evidence rank.
const RUNGS: [(&str, &str, u8); 6] = [
    ("swiss-ephemeris", "Swiss Ephemeris", 2),
    ("pyjhora", "PyJHora", 3),
    ("committee", "the committee's print", 1),
    ("raman", "Raman's Example 56", 2),
    ("charak", "Charak's worked year", 2),
    ("kalaprakasika", "*Kalaprakasika* XIII", 1),
];

/// The rungs that judge each request; none, and the request is agreement
/// only (§11).
const JUDGED: [(&str, &[&str]); 14] = [
    ("chart", &["swiss-ephemeris", "pyjhora"]),
    ("day", &["committee"]),
    ("full", &["swiss-ephemeris", "pyjhora"]),
    ("vargas", &[]),
    ("yogas", &[]),
    ("strength", &["raman"]),
    ("dashas", &["pyjhora"]),
    ("month", &[]),
    ("milan", &["kalaprakasika"]),
    ("varsha", &["charak"]),
    ("kp", &["swiss-ephemeris"]),
    ("gochar", &["swiss-ephemeris"]),
    ("sadeSati", &[]),
    ("batch", &["swiss-ephemeris"]),
];

/// What a cause claims, which the gate checks on the record (§11).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cause {
    /// A convention the two chose apart: `same` agrees more than `timed`.
    Convention,
    /// A method: `same` still differs.
    Method,
    /// The baseline is wrong: the SDK is the closer to a rung.
    Baseline,
}

impl Cause {
    const fn label(self) -> &'static str {
        match self {
            Self::Convention => "convention",
            Self::Method => "method",
            Self::Baseline => "the baseline",
        }
    }
}

/// Each request's dominant cause of disagreement, and what the record
/// shows of it.
const CAUSES: [(&str, Cause, &str); 14] = [
    (
        "chart",
        Cause::Convention,
        "the centre: `timed` casts the Moon from the Earth's centre and the baseline from the \
         birthplace, up to an hour of arc apart, on a mean ayanamsha where the baseline's is \
         nutated (12″), which moves the Vimshottari's starts; under `same` only the lagnas of \
         2350 and 2399 differ, the SDK's IAU 2006 sidereal time against Swiss Ephemeris's that \
         far from J2000",
    ),
    (
        "day",
        Cause::Method,
        "a day with no sunrise: both now take the refracted upper limb, so only the polar births \
         differ, which the SDK reckons by its polar policy (civil midnight under `timed`, the \
         conformance profile's nearest sunrise and sunset under `same`) and the baseline by its \
         civil-day bounds. The committee's tithi ends are Surya Siddhanta's, hours from both",
    ),
    (
        "full",
        Cause::Convention,
        "the centre, and the classical Shadbala readings; under `same` the yogas' readings (as \
         `yogas`), and c022's kaala bala, a birth the baseline places in the wrong Hindu day \
         (crux C35)",
    ),
    (
        "vargas",
        Cause::Convention,
        "the centre: a Moon a few arcminutes from a division's edge",
    ),
    (
        "yogas",
        Cause::Method,
        "the rules' readings: the SDK applies Kemadruma's cancellations and reads the Nabhasa \
         shapes, Gaja Kesari and Chamara by its texts' clauses, where the baseline holds them \
         present",
    ),
    (
        "strength",
        Cause::Convention,
        "the readings: `timed` is the root's, Sripati's, which reproduces Raman's totals, and \
         the baseline its own; under `same` the largest gaps are c022's, a birth the baseline \
         places in the wrong Hindu day (crux C35)",
    ),
    (
        "dashas",
        Cause::Convention,
        "the centre: the Moon's place sets every system's start; PyJHora casts from the Earth's \
         centre as `timed` does",
    ),
    (
        "month",
        Cause::Convention,
        "the sunrise, as `day`; under `same` every limb agrees",
    ),
    (
        "milan",
        Cause::Method,
        "the koota tables: Vashya, Gana, Yoni and Bhakoot score apart under `same`, and the \
         baseline's Stree-deergha is Brihaspati's count beyond 9 where *Kalaprakasika* prints \
         beyond 13",
    ),
    (
        "varsha",
        Cause::Baseline,
        "the year lord: on Charak's worked year the SDK names all seven offices and the baseline \
         three, its year lord the Moon for the Sun; the sahams are other formulas, and the Mudda \
         dasha follows the year lord",
    ),
    (
        "kp",
        Cause::Convention,
        "the frame: `kp-default` casts from the Earth's centre on its own node, the baseline from \
         the birthplace on the mean node; under `same` c043's cusps, where the baseline's \
         Placidus falls back at 64.8° N and Swiss Ephemeris and the SDK do not, and the lagnas \
         of 2350 and 2399, as `chart`; the day lord, which the SDK takes from sunrise to sunrise \
         as KP Reader VI does and the baseline from midnight, so a birth before sunrise rules by \
         the day before; and an intercepted sign's lord, which the baseline counts a significator \
         of its house and the SDK reports and does not count (crux C154)",
    ),
    (
        "gochar",
        Cause::Convention,
        "the ayanamsha's basis: `timed` takes the mean and the baseline the nutated, about 12″ \
         apart; under `same`, cast from the Earth's centre as the baseline casts a transit, the \
         transits agree to a hundredth of an arcsecond and only c051's transit bindus differ",
    ),
    (
        "sadeSati",
        Cause::Convention,
        "the ayanamsha's basis, as `gochar`: 12″ of Saturn moves an edge by hours; under `same` \
         every edge agrees within two seconds",
    ),
    ("batch", Cause::Convention, "as `full`, at ten instants"),
];

/// The least share of compared facts each request agrees on, under
/// `timed` and under `same`: the record's of 26 October 2026, to the
/// tenth of a percent below; `kp`'s of 28 October, when its significators
/// and ruling planets joined the comparison.
const AGREEMENT_FLOORS: [(&str, f64, f64); 14] = [
    ("chart", 0.545, 0.999),
    ("day", 0.991, 0.965),
    ("full", 0.643, 0.999),
    ("vargas", 0.97, 1.0),
    ("yogas", 0.968, 0.97),
    ("strength", 0.691, 0.986),
    ("dashas", 0.619, 1.0),
    ("month", 0.991, 1.0),
    ("milan", 0.734, 0.763),
    ("varsha", 0.26, 0.243),
    ("kp", 0.808, 0.968),
    ("gochar", 0.659, 0.997),
    ("sadeSati", 0.076, 1.0),
    ("batch", 0.653, 0.999),
];

/// The least share each side keeps on a rung for a request, under a
/// convention, from the same record: rung, request, side, convention,
/// floor.
const REFERENCE_FLOORS: [(&str, &str, &str, &str, f64); 38] = [
    ("swiss-ephemeris", "chart", "sdk", "timed", 0.996),
    ("swiss-ephemeris", "full", "sdk", "timed", 0.998),
    ("swiss-ephemeris", "batch", "sdk", "timed", 0.998),
    ("swiss-ephemeris", "kp", "sdk", "timed", 0.978),
    ("swiss-ephemeris", "gochar", "sdk", "timed", 1.0),
    ("swiss-ephemeris", "chart", "sdk", "same", 0.996),
    ("swiss-ephemeris", "full", "sdk", "same", 0.998),
    ("swiss-ephemeris", "batch", "sdk", "same", 0.998),
    ("swiss-ephemeris", "kp", "sdk", "same", 0.978),
    ("swiss-ephemeris", "gochar", "sdk", "same", 1.0),
    ("swiss-ephemeris", "chart", "baseline", "timed", 1.0),
    ("swiss-ephemeris", "full", "baseline", "timed", 1.0),
    ("swiss-ephemeris", "batch", "baseline", "timed", 1.0),
    ("swiss-ephemeris", "kp", "baseline", "timed", 0.982),
    ("swiss-ephemeris", "gochar", "baseline", "timed", 1.0),
    ("pyjhora", "chart", "sdk", "timed", 0.728),
    ("pyjhora", "full", "sdk", "timed", 0.727),
    ("pyjhora", "dashas", "sdk", "timed", 0.728),
    ("pyjhora", "chart", "sdk", "same", 0.465),
    ("pyjhora", "full", "sdk", "same", 0.463),
    ("pyjhora", "dashas", "sdk", "same", 0.465),
    ("pyjhora", "dashas", "sdk", "source", 1.0),
    ("pyjhora", "chart", "baseline", "timed", 0.465),
    ("pyjhora", "full", "baseline", "timed", 0.463),
    ("pyjhora", "dashas", "baseline", "timed", 0.465),
    ("committee", "day", "sdk", "timed", 0.0),
    ("committee", "day", "sdk", "same", 0.346),
    ("committee", "day", "baseline", "timed", 0.346),
    ("raman", "strength", "sdk", "timed", 0.0),
    ("raman", "strength", "sdk", "same", 0.0),
    ("raman", "strength", "baseline", "timed", 0.0),
    ("raman", "strength", "sdk", "source", 1.0),
    ("charak", "varsha", "sdk", "timed", 1.0),
    ("charak", "varsha", "sdk", "same", 0.875),
    ("charak", "varsha", "baseline", "timed", 0.375),
    ("kalaprakasika", "milan", "sdk", "timed", 1.0),
    ("kalaprakasika", "milan", "sdk", "same", 1.0),
    ("kalaprakasika", "milan", "baseline", "timed", 0.921),
];

/// One family's comparison on one request: the two sides' facts, or one
/// side's against a rung's.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(super) struct Tally {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    rung: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    rank: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    side: Option<String>,
    convention: String,
    request: String,
    family: String,
    inputs: usize,
    /// Facts one side holds and the other does not, counted and not
    /// compared.
    unpaired: usize,
    compared: usize,
    /// Exact matches, or values within the tolerance.
    matches: usize,
    share: f64,
    /// The first mismatching inputs of a categorical family.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    mismatches: Vec<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    unit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tolerance: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    median: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    max: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    worst: Option<serde_json::Value>,
}

/// A request the SDK did not answer under a convention, for one birth.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(super) struct Unanswered {
    convention: String,
    request: String,
    id: String,
    reason: String,
}

/// The record's agreement and accuracy.
#[derive(Debug, Default, Deserialize, Serialize)]
pub(super) struct Accuracy {
    pub(super) agree: Vec<Tally>,
    pub(super) reference: Vec<Tally>,
    pub(super) unanswered: Vec<Unanswered>,
}

/// The share of facts matching over `lines`, or `None` for none compared.
fn pooled<'a>(lines: impl Iterator<Item = &'a Tally>) -> Option<(f64, usize)> {
    let (matches, compared) = lines.fold((0, 0), |(m, c), l| (m + l.matches, c + l.compared));
    #[expect(
        clippy::cast_precision_loss,
        reason = "a count of facts is far below 2^52"
    )]
    (compared > 0).then(|| (matches as f64 / compared as f64, compared))
}

impl Accuracy {
    /// A request's pooled agreement under a convention.
    fn agreement(&self, request: &str, convention: &str) -> Option<(f64, usize)> {
        pooled(
            self.agree
                .iter()
                .filter(|l| l.request == request && l.convention == convention),
        )
    }

    /// A side's pooled share on a rung for a request, under a convention.
    fn judged(
        &self,
        rung: &str,
        request: &str,
        side: &str,
        convention: &str,
    ) -> Option<(f64, usize)> {
        pooled(self.reference.iter().filter(|l| {
            l.rung.as_deref() == Some(rung)
                && l.request == request
                && l.side.as_deref() == Some(side)
                && l.convention == convention
        }))
    }

    /// The conventions a side was held to a rung under, for a request.
    fn conventions(&self, rung: &str, request: &str, side: &str) -> Vec<&str> {
        let mut out: Vec<&str> = self
            .reference
            .iter()
            .filter(|l| {
                l.rung.as_deref() == Some(rung)
                    && l.request == request
                    && l.side.as_deref() == Some(side)
            })
            .map(|l| l.convention.as_str())
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    /// The SDK's best share on a rung, with its convention, and the
    /// baseline's.
    fn closer(&self, rung: &str, request: &str) -> (Option<(f64, String)>, Option<f64>) {
        let sdk = self
            .conventions(rung, request, "sdk")
            .into_iter()
            .filter_map(|c| {
                self.judged(rung, request, "sdk", c)
                    .map(|(share, _)| (share, c.to_string()))
            })
            .max_by(|a, b| a.0.total_cmp(&b.0));
        let baseline = self
            .judged(rung, request, "baseline", "timed")
            .map(|(share, _)| share);
        (sdk, baseline)
    }
}

/// Whether an SDK answer missing under a convention is excused at its
/// birth: as a refusal is (§2), or at a polar birth for a rule that needs
/// a sunrise nearer than the baseline's polar day gives (§10).
fn excuses(excused: &[Excused], u: &Unanswered) -> bool {
    excused.iter().any(|(id, words)| {
        *id == u.id
            && (u.reason.contains(words) || (*words == SUNLESS && u.reason.contains(SINCE_SUNRISE)))
    })
}

/// The accuracy's faults (§8, §10, §11).
pub(super) fn faults(
    name: &str,
    accuracy: &Accuracy,
    excused: &[Excused],
    found: &mut Vec<String>,
) {
    if accuracy.agree.is_empty() {
        found.push(format!(
            "{name}: holds no agreement; re-record with `cargo xtask measure-ab`"
        ));
        return;
    }
    for (request, _) in REQUESTS {
        for convention in CONVENTIONS {
            if accuracy.agreement(request, convention).is_none() {
                found.push(format!(
                    "{name}: `{request}` compares no facts under `{convention}`"
                ));
            }
        }
    }
    for (request, rungs) in JUDGED {
        for rung in rungs {
            for side in ["sdk", "baseline"] {
                if accuracy.conventions(rung, request, side).is_empty() {
                    found.push(format!(
                        "{name}: `{request}` lacks the {side}'s {rung} rung (§11)"
                    ));
                }
            }
        }
    }
    for l in &accuracy.reference {
        let rung = l.rung.as_deref().unwrap_or("");
        let judged = JUDGED
            .iter()
            .any(|(request, rungs)| *request == l.request && rungs.contains(&rung));
        if !judged {
            found.push(format!(
                "{name}: the {rung} rung judges `{}`, which the page does not say it judges",
                l.request
            ));
        }
    }
    for u in accuracy.unanswered.iter().filter(|u| !excuses(excused, u)) {
        found.push(format!(
            "{name}: the SDK left `{}` unanswered under `{}` for {}, which the corpus does not \
             excuse there: {}",
            u.request, u.convention, u.id, u.reason
        ));
    }
    floor_faults(name, accuracy, found);
    cause_faults(name, accuracy, found);
}

/// A share below the least the record measured.
fn floor_faults(name: &str, accuracy: &Accuracy, found: &mut Vec<String>) {
    let mut below = |what: String, share: Option<(f64, usize)>, floor: f64| {
        let share = share.map_or(0.0, |(s, _)| s);
        if share + 1e-12 < floor {
            found.push(format!(
                "{name}: {what} is {:.1}%, below its floor of {:.1}%",
                share * 100.0,
                floor * 100.0
            ));
        }
    };
    for (request, timed, same) in AGREEMENT_FLOORS {
        below(
            format!("`{request}`'s agreement under `timed`"),
            accuracy.agreement(request, "timed"),
            timed,
        );
        below(
            format!("`{request}`'s agreement under `same`"),
            accuracy.agreement(request, "same"),
            same,
        );
    }
    for (rung, request, side, convention, floor) in REFERENCE_FLOORS {
        below(
            format!("the {side} under `{convention}` on the {rung} rung for `{request}`"),
            accuracy.judged(rung, request, side, convention),
            floor,
        );
    }
}

/// A cause whose claim the record does not bear out (§11).
fn cause_faults(name: &str, accuracy: &Accuracy, found: &mut Vec<String>) {
    for (request, cause, _) in CAUSES {
        let share = |c| accuracy.agreement(request, c).map_or(0.0, |(s, _)| s);
        let rungs = JUDGED
            .iter()
            .find(|(r, _)| *r == request)
            .map_or(&[][..], |(_, rungs)| *rungs);
        let sdk_closer = || {
            rungs.iter().any(|rung| {
                matches!(accuracy.closer(rung, request),
                    (Some((sdk, _)), Some(baseline)) if sdk > baseline)
            })
        };
        let holds = match cause {
            Cause::Convention => share("same") > share("timed"),
            Cause::Method => share("same") < 1.0,
            Cause::Baseline => sdk_closer(),
        };
        if !holds {
            found.push(format!(
                "{name}: `{request}`'s cause is {}, which the record does not bear out (§11)",
                cause.label()
            ));
        }
    }
}

/// A share as the page prints it.
fn percent(share: f64) -> String {
    format!("{:.1}%", share * 100.0)
}

/// A pooled agreement as a cell: its share and of how many facts.
fn agreement_cell(share: Option<(f64, usize)>) -> String {
    share.map_or_else(
        || String::from("—"),
        |(s, n)| format!("{} of {n}", percent(s)),
    )
}

/// A line's share, and for a continuous family its median and largest.
fn line_cell(line: Option<&Tally>) -> String {
    let Some(l) = line else {
        return String::from("—");
    };
    let mut out = format!("{} of {}", percent(l.share), l.compared);
    if let (Some(median), Some(max)) = (l.median, l.max) {
        let _ = write!(out, "; median {}, largest {}", number(median), number(max));
    }
    out
}

/// A difference as the page prints it: to three places below 10, else to
/// one.
fn number(value: f64) -> String {
    if value.abs() < 10.0 {
        format!("{value:.3}")
    } else {
        format!("{value:.1}")
    }
}

/// A value of a mismatch, as the page prints it.
fn value(v: Option<&serde_json::Value>) -> String {
    v.map_or_else(|| String::from("—"), ToString::to_string)
}

/// What a line shows of where it differs: its first mismatches, or its
/// worst difference.
fn differs(line: &Tally) -> String {
    if let Some(worst) = &line.worst {
        if line.matches == line.compared {
            return String::new();
        }
        let id = worst
            .get("id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let key = worst
            .get("key")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        return format!("worst {id} `{key}`");
    }
    line.mismatches
        .iter()
        .map(|m| {
            let id = m
                .get("id")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            let key = m
                .get("key")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            let (a, b) = if m.get("got").is_some() {
                (m.get("got"), m.get("reference"))
            } else {
                (m.get("sdk"), m.get("baseline"))
            };
            format!("{id} `{key}` {} against {}", value(a), value(b))
        })
        .collect::<Vec<_>>()
        .join("; ")
}

/// A rung's title and rank.
fn rung_title(rung: &str) -> String {
    RUNGS.iter().find(|(id, _, _)| *id == rung).map_or_else(
        || rung.to_string(),
        |(_, title, rank)| format!("{title} ({rank})"),
    )
}

/// Which side a rung finds the closer, as a cell.
fn closer_cell(accuracy: &Accuracy, rung: &str, request: &str) -> String {
    let (sdk, baseline) = accuracy.closer(rung, request);
    let verdict = match (&sdk, baseline) {
        (Some((s, _)), Some(b)) if s > &b => "the SDK",
        (Some((s, _)), Some(b)) if &b > s => "the baseline",
        (Some(_), Some(_)) => "neither",
        _ => "not judged",
    };
    let sdk = sdk.map_or_else(
        || String::from("—"),
        |(s, c)| format!("{} `{c}`", percent(s)),
    );
    let baseline = baseline.map_or_else(|| String::from("—"), percent);
    format!("{verdict}: SDK {sdk}, baseline {baseline}")
}

/// The page's accuracy section for one record (§10, §11).
pub(super) fn tables(out: &mut String, accuracy: &Accuracy) {
    let _ = writeln!(
        out,
        "\n### Agreement and accuracy\n\nThe share of facts on which the two sides agree, \
         exactly or within the field's tolerance, under the SDK's timed profiles (`timed`) and \
         under the profiles that cast as the baseline casts (`same`), so that what still differs \
         under `same` is method (§10); and each side against a reference neither side computed, \
         with its evidence rank, where one exists (§11). Closer is the larger share of the \
         reference's facts matched, the SDK's under its best convention.\n"
    );
    let _ = writeln!(
        out,
        "| request | agreement, `timed` | agreement, `same` | reference | closer to it | dominant cause |"
    );
    let _ = writeln!(out, "|---|---:|---:|---|---|---|");
    for (request, rungs) in JUDGED {
        let reference = if rungs.is_empty() {
            String::from("agreement only")
        } else {
            rungs
                .iter()
                .map(|r| rung_title(r))
                .collect::<Vec<_>>()
                .join("; ")
        };
        let closer = if rungs.is_empty() {
            String::from("—")
        } else {
            rungs
                .iter()
                .map(|r| closer_cell(accuracy, r, request))
                .collect::<Vec<_>>()
                .join("; ")
        };
        let cause = CAUSES
            .iter()
            .find(|(r, _, _)| *r == request)
            .map_or_else(String::new, |(_, cause, why)| {
                format!("{}: {why}", cause.label())
            });
        let _ = writeln!(
            out,
            "| {request} | {} | {} | {reference} | {closer} | {cause} |",
            agreement_cell(accuracy.agreement(request, "timed")),
            agreement_cell(accuracy.agreement(request, "same")),
        );
    }
    field_table(out, accuracy);
    reference_table(out, accuracy);
    if !accuracy.unanswered.is_empty() {
        let _ = writeln!(out, "\nThe SDK left unanswered under a convention:\n");
        for u in &accuracy.unanswered {
            let _ = writeln!(
                out,
                "- {}, `{}`, under `{}`: {}",
                u.id, u.request, u.convention, u.reason
            );
        }
    }
}

/// Agreement by field: one row a request and family.
fn field_table(out: &mut String, accuracy: &Accuracy) {
    let _ = writeln!(
        out,
        "\n#### Agreement by field\n\nA continuous field's median and largest difference are in \
         its unit; `same`'s first mismatches, or its worst input, follow.\n"
    );
    let _ = writeln!(
        out,
        "| request | field | unit, tolerance | `timed` | `same` | unpaired | where `same` differs |"
    );
    let _ = writeln!(out, "|---|---|---|---:|---:|---:|---|");
    for (request, _) in REQUESTS {
        let mut families: Vec<&str> = accuracy
            .agree
            .iter()
            .filter(|l| l.request == request)
            .map(|l| l.family.as_str())
            .collect();
        let mut seen = std::collections::BTreeSet::new();
        families.retain(|f| seen.insert(*f));
        for family in families {
            let line = |c: &str| {
                accuracy
                    .agree
                    .iter()
                    .find(|l| l.request == request && l.family == family && l.convention == c)
            };
            let (timed, same) = (line("timed"), line("same"));
            let unit = same
                .or(timed)
                .map_or_else(String::new, |l| match (&l.unit, l.tolerance) {
                    (Some(unit), Some(tolerance)) => format!("{unit}, {}", number(tolerance)),
                    _ => String::from("exact"),
                });
            let unpaired = same.or(timed).map_or(0, |l| l.unpaired);
            let _ = writeln!(
                out,
                "| {request} | {family} | {unit} | {} | {} | {unpaired} | {} |",
                line_cell(timed),
                line_cell(same),
                same.map_or_else(String::new, differs)
            );
        }
    }
}

/// Each side against each rung: one row a rung, request, family and who.
fn reference_table(out: &mut String, accuracy: &Accuracy) {
    let _ = writeln!(
        out,
        "\n#### Against the references\n\nEach side under each convention it was held to the \
         rung under; the SDK's `source` is `timed` given the reading the source works by.\n"
    );
    let _ = writeln!(
        out,
        "| reference | request | field | side | convention | matching | where it differs |"
    );
    let _ = writeln!(out, "|---|---|---|---|---|---:|---|");
    for l in &accuracy.reference {
        let rung = l.rung.as_deref().unwrap_or("");
        let side = match l.side.as_deref() {
            Some("sdk") => "the SDK",
            _ => "the baseline",
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | {side} | `{}` | {} | {} |",
            rung_title(rung),
            l.request,
            l.family,
            l.convention,
            line_cell(Some(l)),
            differs(l)
        );
    }
}

/// One comparison line, for the gate's tests.
#[cfg(test)]
fn line(request: &str, convention: &str, matches: usize, compared: usize) -> Tally {
    #[expect(clippy::cast_precision_loss, reason = "a test's small counts")]
    let share = matches as f64 / compared as f64;
    Tally {
        rung: None,
        rank: None,
        side: None,
        convention: convention.to_string(),
        request: request.to_string(),
        family: String::from("sign"),
        inputs: 1,
        unpaired: 0,
        compared,
        matches,
        share,
        mismatches: Vec::new(),
        unit: None,
        tolerance: None,
        median: None,
        max: None,
        worst: None,
    }
}

/// One reference line, for the gate's tests.
#[cfg(test)]
fn judged(rung: &str, request: &str, side: &str, convention: &str, matches: usize) -> Tally {
    Tally {
        rung: Some(rung.to_string()),
        rank: Some(2),
        side: Some(side.to_string()),
        ..line(request, convention, matches, 1000)
    }
}

/// A count of a thousand facts at `share`, for the gate's tests.
#[cfg(test)]
fn thousandths(share: f64) -> usize {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a share is between 0 and 1"
    )]
    let n = (share * 1000.0).round() as usize;
    n
}

/// A record's accuracy the gate passes: every share at its floor.
#[cfg(test)]
pub(super) fn whole() -> Accuracy {
    let mut agree = Vec::new();
    for (request, timed, same) in AGREEMENT_FLOORS {
        agree.push(line(request, "timed", thousandths(timed), 1000));
        agree.push(line(request, "same", thousandths(same), 1000));
    }
    let reference = REFERENCE_FLOORS
        .iter()
        .map(|(rung, request, side, convention, floor)| {
            judged(rung, request, side, convention, thousandths(*floor))
        })
        .collect();
    Accuracy {
        agree,
        reference,
        unanswered: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn faults_of(accuracy: &Accuracy) -> Vec<String> {
        let mut found = Vec::new();
        faults(
            "r",
            accuracy,
            &[(String::from("c028"), SUNLESS)],
            &mut found,
        );
        found
    }

    #[test]
    fn a_cause_is_held_to_what_the_record_shows() {
        let mut accuracy = whole();
        assert!(
            faults_of(&accuracy).is_empty(),
            "{:?}",
            faults_of(&accuracy)
        );
        // varsha names the baseline; a rung finding the two alike fails it.
        for l in accuracy
            .reference
            .iter_mut()
            .filter(|l| l.request == "varsha")
        {
            l.matches = 1000;
        }
        // A convention cause where `same` agrees no more than `timed`.
        for l in accuracy.agree.iter_mut().filter(|l| l.request == "chart") {
            l.matches = 1000;
        }
        let found = faults_of(&accuracy);
        assert_eq!(found.len(), 2, "{found:?}");
        assert!(
            found[0].contains("`chart`'s cause is convention"),
            "{found:?}"
        );
        assert!(
            found[1].contains("`varsha`'s cause is the baseline"),
            "{found:?}"
        );
        // A share below its floor.
        for l in accuracy.agree.iter_mut().filter(|l| l.request == "dashas") {
            l.matches = 10;
        }
        let found = faults_of(&accuracy);
        assert!(
            found.iter().any(|f| f
                .contains("`dashas`'s agreement under `same` is 1.0%, below its floor of 100.0%")),
            "{found:?}"
        );
    }

    #[test]
    fn a_missing_rung_or_convention_and_an_unexcused_gap_fail() {
        let mut accuracy = whole();
        accuracy
            .agree
            .retain(|l| !(l.request == "month" && l.convention == "same"));
        accuracy.reference.retain(|l| l.request != "milan");
        accuracy
            .reference
            .push(judged("raman", "yogas", "sdk", "timed", 900));
        accuracy.unanswered.push(Unanswered {
            convention: String::from("same"),
            request: String::from("chart"),
            id: String::from("c028"),
            reason: String::from(
                "BHAVA_LAGNA needs an elapsed time of 0 to 36 hours since sunrise",
            ),
        });
        accuracy.unanswered.push(Unanswered {
            convention: String::from("same"),
            request: String::from("kp"),
            id: String::from("c001"),
            reason: String::from("no"),
        });
        let found = faults_of(&accuracy);
        assert!(
            found
                .iter()
                .any(|f| f.contains("`month` compares no facts under `same`")),
            "{found:?}"
        );
        assert!(
            found
                .iter()
                .any(|f| f.contains("lacks the sdk's kalaprakasika rung")),
            "{found:?}"
        );
        assert!(
            found.iter().any(|f| f.contains("judges `yogas`")),
            "{found:?}"
        );
        assert!(
            found
                .iter()
                .any(|f| f.contains("`kp` unanswered under `same` for c001")),
            "{found:?}"
        );
        assert!(!found.iter().any(|f| f.contains("c028")), "{found:?}");
    }

    #[test]
    fn a_record_without_agreement_is_no_claim() {
        let found = faults_of(&Accuracy::default());
        assert_eq!(found.len(), 1);
        assert!(found[0].contains("holds no agreement"));
    }

    #[test]
    fn the_page_marks_a_request_without_a_rung_agreement_only() {
        let mut out = String::new();
        tables(&mut out, &whole());
        assert!(
            out.contains("| vargas | 97.0% of 1000 | 100.0% of 1000 | agreement only | — |"),
            "{out}"
        );
        assert!(
            out.contains(
                "| strength | 69.1% of 1000 | 98.6% of 1000 | Raman's Example 56 (2) | the SDK: SDK \
                 100.0% `source`, baseline 0.0% |"
            ),
            "{out}"
        );
    }

    #[test]
    fn every_request_has_its_rungs_floors_and_cause() {
        let requests: Vec<&str> = REQUESTS.iter().map(|(r, _)| *r).collect();
        assert_eq!(JUDGED.map(|(r, _)| r).to_vec(), requests);
        assert_eq!(CAUSES.map(|(r, _, _)| r).to_vec(), requests);
        assert_eq!(AGREEMENT_FLOORS.map(|(r, _, _)| r).to_vec(), requests);
        for (rung, request, _, _, _) in REFERENCE_FLOORS {
            assert!(
                JUDGED
                    .iter()
                    .any(|(r, rungs)| *r == request && rungs.contains(&rung)),
                "{rung} {request}"
            );
        }
    }
}
