//! The festival saits against the committee's printed times
//! (`03-design/festival-sait.md` §4).
//!
//! Each printed sait is the rite's day asked of the `NEPAL_PANCHANGA`
//! pack at Kathmandu on Nepal's clock, over the committee's own sky
//! (`nepali-committee`), which the pass holds to the print, and over the
//! modern sky (`nepali-drik`), which it counts beside it. The readings
//! the shipped rules take where no text decides (C745 to C747) are each
//! measured against the reading they were chosen over, so a rule the
//! print does not bear out reads `falsified` here.

use std::fmt::Write as _;

use teistro::festival::{SaitChoice, SaitRule};
use teistro::{Context, FestivalPack, FestivalRequest};

use crate::measure::{Claim, table};

use super::nepal::{Day, Sky, gregorian, kathmandu};

/// The committee's printed saits: the rule, the day, and the printed
/// minute after Nepal's midnight.
const PRINTED: [(&str, Day, u32); 6] = [
    ("NAVARATRA_ARAMBHA", (2023, 10, 15), 11 * 60 + 29),
    ("NAVARATRA_ARAMBHA", (2024, 10, 3), 9 * 60 + 16),
    ("NAVARATRA_ARAMBHA", (2025, 9, 22), 9 * 60 + 13),
    ("NAVARATRA_ARAMBHA", (2026, 10, 11), 11 * 60 + 47),
    ("PHULPATI", (2024, 10, 10), 7 * 60 + 11),
    ("YAMA_DWITIYA", (2025, 10, 23), 11 * 60 + 39),
];

/// The committee prints whole minutes, so a printed time a minute past a
/// navamsa's edge still falls in it.
const PRINTED_SLACK_MINUTES: f64 = 1.0;

/// One printed sait as a sky elects it.
pub(super) struct Elected {
    rule: &'static str,
    day: Day,
    printed: u32,
    /// The chosen candidate's span, in minutes after Nepal's midnight,
    /// with its lagna and navamsa; none when nothing was chosen.
    chosen: Option<(f64, f64, String)>,
    /// The chosen middle less the printed minute.
    offset: Option<f64>,
}

impl Elected {
    fn holds_print(&self) -> bool {
        self.chosen.as_ref().is_some_and(|(from, to, _)| {
            (from - PRINTED_SLACK_MINUTES..=to + PRINTED_SLACK_MINUTES)
                .contains(&f64::from(self.printed))
        })
    }
}

/// A reading the shipped rules take, with the rules it was chosen over.
struct Reading {
    claim: &'static str,
    /// The shipped rules changed to the alternative.
    instead: fn(SaitRule) -> SaitRule,
}

const READINGS: [Reading; 3] = [
    Reading {
        claim: "C747: without the Sun's lagna barred, the print is still elected",
        instead: |mut rule| {
            if let Some(rising) = rule.rising.as_mut() {
                rising.free_of.clear();
            }
            rule
        },
    },
    Reading {
        claim: "C747: with every malefic's lagna barred, the print is still elected",
        instead: |mut rule| {
            if let Some(rising) = rule.rising.as_mut() {
                rising.free_of = vec![
                    teistro::catalogue::Graha::Sun,
                    teistro::catalogue::Graha::Mars,
                    teistro::catalogue::Graha::Saturn,
                    teistro::catalogue::Graha::Rahu,
                    teistro::catalogue::Graha::Ketu,
                ];
            }
            rule
        },
    },
    Reading {
        claim: "C746: with each rule's choice turned round, the print is still elected",
        instead: |mut rule| {
            rule.choose = match rule.choose {
                SaitChoice::Earliest => SaitChoice::Latest,
                _ => SaitChoice::Earliest,
            };
            rule
        },
    },
];

/// What the page reports.
pub(super) struct Saits {
    committee: Vec<Elected>,
    modern: Vec<Elected>,
    /// Each reading's claim, with how many printed saits it misses on
    /// the committee's sky.
    readings: Vec<(&'static str, usize)>,
}

/// Every printed sait elected over both skies, and each reading's
/// alternative over the committee's. A problem for every printed time
/// the committee's sky does not elect.
pub(super) fn compare(problems: &mut Vec<String>) -> Result<Saits, String> {
    let shipped = FestivalRequest::from(FestivalPack::NepalPanchanga);
    let committee = Sky::Committee.context()?;
    let modern = Sky::Modern.context()?;
    let on_committee = elect(&committee, &shipped)?;
    for elected in on_committee.iter().filter(|e| !e.holds_print()) {
        problems.push(format!(
            "{} {:?}: the committee's sky elects no candidate holding the printed {}",
            elected.rule,
            elected.day,
            clock(f64::from(elected.printed))
        ));
    }
    let mut readings = Vec::new();
    for reading in &READINGS {
        let request = shipped
            .saits()
            .iter()
            .cloned()
            .fold(shipped.clone(), |request, rule| {
                request.with_sait((reading.instead)(rule))
            });
        let missed = elect(&committee, &request)?
            .iter()
            .filter(|e| !e.holds_print())
            .count();
        readings.push((reading.claim, missed));
    }
    Ok(Saits {
        committee: on_committee,
        modern: elect(&modern, &shipped)?,
        readings,
    })
}

/// Each printed row's sait, as `request` elects it over `sky`.
fn elect(sky: &Context, request: &FestivalRequest) -> Result<Vec<Elected>, String> {
    let (place, offset) = kathmandu();
    PRINTED
        .iter()
        .map(|&(rule, day, printed)| {
            let on = gregorian(day);
            let found = sky
                .almanac()
                .festivals(&on, &on, &place, offset, request)
                .map_err(|e| format!("{rule} {day:?}: {e}"))?
                .value;
            let sait = found
                .observances
                .iter()
                .find(|o| o.rule == rule)
                .and_then(|o| o.sait.as_ref())
                .ok_or_else(|| format!("{rule} {day:?}: no observance with a sait"))?;
            let chosen = sait.chosen.and_then(|i| sait.candidates.get(i));
            Ok(Elected {
                rule,
                day,
                printed,
                chosen: chosen.map(|c| {
                    let signs = match (c.lagna, c.navamsa) {
                        (Some(lagna), Some(navamsa)) => format!("{lagna:?} / {navamsa:?}"),
                        _ => String::from("-"),
                    };
                    (minutes(c.span.from.get()), minutes(c.span.to.get()), signs)
                }),
                offset: sait.at.map(|at| minutes(at.get()) - f64::from(printed)),
            })
        })
        .collect()
}

/// Minutes after Nepal's midnight of a UTC Julian day.
fn minutes(jd: f64) -> f64 {
    (jd + 0.5 + 5.75 / 24.0).rem_euclid(1.0) * 1440.0
}

fn clock(minutes: f64) -> String {
    let whole = minutes.round();
    format!(
        "{:02}:{:02}",
        (whole / 60.0).floor(),
        whole.rem_euclid(60.0)
    )
}

/// §8 of the page.
pub(super) fn render(out: &mut String, saits: &Saits) {
    out.push_str(
        "\n## 8. Saits against the print\n\n\
         Each printed sait (`festival-sait.md` §1) is the rite's day asked of\n\
         the `NEPAL_PANCHANGA` pack, whose saits elect a navamsa of a dual sign\n\
         in a lagna the Sun does not stand in, inside the text's window less\n\
         what it avoids. The pass holds every printed minute to the candidate\n\
         the committee's sky elects, within a printed minute; the elected\n\
         middle's distance from the print is measured, not held.\n\n",
    );
    let missed = |of: &[Elected]| of.iter().filter(|e| !e.holds_print()).count();
    let mut claims = vec![
        Claim::counted(
            "the committee's sky elects every printed sait",
            missed(&saits.committee),
            saits.committee.len(),
        ),
        Claim::counted(
            "the modern sky elects every printed sait",
            missed(&saits.modern),
            saits.modern.len(),
        ),
    ];
    claims.extend(
        saits
            .readings
            .iter()
            .map(|&(claim, wrong)| Claim::counted(claim, wrong, saits.committee.len())),
    );
    out.push_str(&table(&claims));
    out.push_str(
        "\n| rite | day | printed | the committee's sky | lagna / navamsa | middle | the modern sky's middle |\n\
         |---|---|---|---|---|---|---|\n",
    );
    for (committee, modern) in saits.committee.iter().zip(&saits.modern) {
        let (span, signs) = committee.chosen.as_ref().map_or_else(
            || (String::from("none"), String::from("-")),
            |(from, to, signs)| (format!("{}–{}", clock(*from), clock(*to)), signs.clone()),
        );
        let offset = |e: &Elected| {
            e.offset
                .map_or_else(|| String::from("-"), |o| format!("{o:+.1} min"))
        };
        let (year, month, day) = committee.day;
        let _ = writeln!(
            out,
            "| {} | {year}-{month:02}-{day:02} | {} | {span} | {signs} | {} | {} |",
            committee.rule,
            clock(f64::from(committee.printed)),
            offset(committee),
            offset(modern),
        );
    }
}
