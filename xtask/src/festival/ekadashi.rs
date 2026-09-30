//! The Ekadashi observers against a published almanac
//! (`03-design/festival-rules.md` §8.4).
//!
//! Drik Panchang's New Delhi lists: its Ekadashi list gives the Smarta day
//! and, where the renunciant's differs, a *Gauna* day; its ISKCON list
//! gives the Vaishnava day. Each is read against the shipped rule that
//! keeps it, and every parting must carry its cause, which the pass checks
//! both ways.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use teistro::catalogue::Tithi;
use teistro::festival::{EkadashiFast, EkadashiRule, Observances};
use teistro::{Context, UtcOffset};

use crate::measure::{Claim, table};

use super::{MonthDay, ordinal, spell};

/// One year of the record, as (month, day).
struct Lists {
    year: i32,
    /// The Ekadashi list's days, the Smarta householder's.
    smarta: &'static [MonthDay],
    /// Its Gauna days, each the day after a Smarta day, which the
    /// renunciant keeps in its place.
    gauna: &'static [MonthDay],
    /// The ISKCON list's days.
    vaishnava: &'static [MonthDay],
}

/// Read off drikpanchang.com's `vrats/ekadashidates.html` and
/// `iskcon/iskcon-ekadashi-list.html` for New Delhi (geoname 1261481),
/// 2026-09-30.
const RECORD: [Lists; 4] = [
    Lists {
        year: 2023,
        smarta: &[
            (1, 2),
            (1, 18),
            (2, 1),
            (2, 16),
            (3, 3),
            (3, 18),
            (4, 1),
            (4, 16),
            (5, 1),
            (5, 15),
            (5, 31),
            (6, 14),
            (6, 29),
            (7, 13),
            (7, 29),
            (8, 12),
            (8, 27),
            (9, 10),
            (9, 25),
            (10, 10),
            (10, 25),
            (11, 9),
            (11, 23),
            (12, 8),
            (12, 22),
        ],
        gauna: &[(9, 26), (12, 23)],
        vaishnava: &[
            (1, 2),
            (1, 18),
            (2, 1),
            (2, 17),
            (3, 3),
            (3, 18),
            (4, 2),
            (4, 16),
            (5, 1),
            (5, 15),
            (5, 31),
            (6, 14),
            (6, 29),
            (7, 13),
            (7, 29),
            (8, 12),
            (8, 27),
            (9, 11),
            (9, 26),
            (10, 10),
            (10, 25),
            (11, 9),
            (11, 23),
            (12, 9),
            (12, 23),
        ],
    },
    Lists {
        year: 2024,
        smarta: &[
            (1, 7),
            (1, 21),
            (2, 6),
            (2, 20),
            (3, 6),
            (3, 20),
            (4, 5),
            (4, 19),
            (5, 4),
            (5, 19),
            (6, 2),
            (6, 18),
            (7, 2),
            (7, 17),
            (7, 31),
            (8, 16),
            (8, 29),
            (9, 14),
            (9, 28),
            (10, 13),
            (10, 28),
            (11, 12),
            (11, 26),
            (12, 11),
            (12, 26),
        ],
        gauna: &[(10, 14)],
        vaishnava: &[
            (1, 7),
            (1, 21),
            (2, 6),
            (2, 20),
            (3, 7),
            (3, 20),
            (4, 5),
            (4, 19),
            (5, 4),
            (5, 19),
            (6, 3),
            (6, 18),
            (7, 2),
            (7, 17),
            (7, 31),
            (8, 16),
            (8, 30),
            (9, 14),
            (9, 28),
            (10, 14),
            (10, 28),
            (11, 12),
            (11, 26),
            (12, 11),
            (12, 26),
        ],
    },
    Lists {
        year: 2025,
        smarta: &[
            (1, 10),
            (1, 25),
            (2, 8),
            (2, 24),
            (3, 10),
            (3, 25),
            (4, 8),
            (4, 24),
            (5, 8),
            (5, 23),
            (6, 6),
            (6, 21),
            (7, 6),
            (7, 21),
            (8, 5),
            (8, 19),
            (9, 3),
            (9, 17),
            (10, 3),
            (10, 17),
            (11, 1),
            (11, 15),
            (12, 1),
            (12, 15),
            (12, 30),
        ],
        gauna: &[(6, 22), (11, 2), (12, 31)],
        vaishnava: &[
            (1, 10),
            (1, 25),
            (2, 8),
            (2, 24),
            (3, 10),
            (3, 26),
            (4, 8),
            (4, 24),
            (5, 8),
            (5, 23),
            (6, 7),
            (6, 22),
            (7, 6),
            (7, 21),
            (8, 5),
            (8, 19),
            (9, 3),
            (9, 17),
            (10, 3),
            (10, 17),
            (11, 2),
            (11, 15),
            (12, 1),
            (12, 16),
            (12, 31),
        ],
    },
    Lists {
        year: 2026,
        smarta: &[
            (1, 14),
            (1, 29),
            (2, 13),
            (2, 27),
            (3, 15),
            (3, 29),
            (4, 13),
            (4, 27),
            (5, 13),
            (5, 27),
            (6, 11),
            (6, 25),
            (7, 10),
            (7, 25),
            (8, 9),
            (8, 23),
            (9, 7),
            (9, 22),
            (10, 6),
            (10, 22),
            (11, 5),
            (11, 20),
            (12, 4),
            (12, 20),
        ],
        gauna: &[(7, 11), (11, 21)],
        vaishnava: &[
            (1, 14),
            (1, 29),
            (2, 13),
            (2, 27),
            (3, 15),
            (3, 29),
            (4, 13),
            (4, 27),
            (5, 13),
            (5, 27),
            (6, 11),
            (6, 25),
            (7, 11),
            (7, 25),
            (8, 9),
            (8, 24),
            (9, 7),
            (9, 22),
            (10, 6),
            (10, 22),
            (11, 5),
            (11, 21),
            (12, 4),
            (12, 20),
        ],
    },
];

/// The years the record covers.
pub(super) const YEARS: std::ops::RangeInclusive<i32> = 2023..=2026;

/// Whether a year is one the record covers.
pub(super) fn covers(year: i32) -> bool {
    YEARS.contains(&year)
}

/// The observer each shipped rule is read against, and the record's day
/// for it.
const OBSERVERS: [(&str, &str); 3] = [
    ("EKADASHI_SMARTA", "Smarta"),
    ("EKADASHI_SMARTA_RENUNCIANT", "Gauna"),
    ("EKADASHI_VAISHNAVA", "ISKCON"),
];

/// Why a rule parts from the record.
#[derive(Clone, Copy)]
enum Cause {
    /// ISKCON's *Paksha Vardhini Mahadvadashi*: the fortnight's new or
    /// full moon holds two sunrises, and the fast moves to the 12th's
    /// day. *Dharmasindhu* states no such rule. The pass founds the
    /// fortnight and checks that it is so.
    PakshaVardhini,
}

impl Cause {
    const fn said(self) -> &'static str {
        match self {
            Cause::PakshaVardhini => {
                "the ISKCON list's Paksha Vardhini Mahadvadashi: the fortnight's new or full moon holds two sunrises, which the pass finds so, and the fast moves to the 12th's day, a rule *Dharmasindhu* does not state"
            }
        }
    }
}

/// A published day and the day found for it, either alone when nothing
/// lies a day away.
#[derive(Clone, Copy, PartialEq)]
struct Parting {
    year: i32,
    published: Option<MonthDay>,
    found: Option<MonthDay>,
}

/// Why a rule parts from the record on one day.
struct Excuse {
    year: i32,
    rule: &'static str,
    published: MonthDay,
    found: MonthDay,
    cause: Cause,
}

impl Excuse {
    const fn parting(&self) -> Parting {
        Parting {
            year: self.year,
            published: Some(self.published),
            found: Some(self.found),
        }
    }
}

/// Why a rule parts from the record. The pass fails on a parting with no
/// entry, on an entry whose parting is gone, and on a cause that does not
/// hold.
const PARTS: [Excuse; 3] = [
    Excuse {
        year: 2023,
        rule: "EKADASHI_VAISHNAVA",
        published: (9, 11),
        found: (9, 10),
        cause: Cause::PakshaVardhini,
    },
    Excuse {
        year: 2024,
        rule: "EKADASHI_VAISHNAVA",
        published: (8, 30),
        found: (8, 29),
        cause: Cause::PakshaVardhini,
    },
    Excuse {
        year: 2025,
        rule: "EKADASHI_VAISHNAVA",
        published: (12, 16),
        found: (12, 15),
        cause: Cause::PakshaVardhini,
    },
];

/// The record's days for one observer in one year.
fn published(year: &Lists, rule: &str) -> BTreeSet<MonthDay> {
    match rule {
        "EKADASHI_SMARTA" => year.smarta.iter().copied().collect(),
        "EKADASHI_SMARTA_RENUNCIANT" => {
            // A Gauna day replaces the Smarta day before it.
            let replaced = |smarta: MonthDay| {
                year.gauna
                    .iter()
                    .any(|&gauna| ordinal(year.year, gauna) == ordinal(year.year, smarta) + 1)
            };
            year.smarta
                .iter()
                .copied()
                .filter(|&smarta| !replaced(smarta))
                .chain(year.gauna.iter().copied())
                .collect()
        }
        _ => year.vaishnava.iter().copied().collect(),
    }
}

/// One observer's measure over the record.
pub(super) struct Measured {
    rule: &'static str,
    list: &'static str,
    days: usize,
    partings: Vec<Parting>,
}

/// Each observer against the record, with a problem for every parting
/// without its cause and every cause without its parting.
pub(super) fn compare(
    context: &Context,
    rules: &[EkadashiRule],
    years: &BTreeMap<i32, Observances>,
    problems: &mut Vec<String>,
) -> Vec<Measured> {
    let mut measured = Vec::new();
    for (rule, list) in OBSERVERS {
        if !rules.iter().any(|shipped| shipped.key == rule) {
            problems.push(format!("the shipped Ekadashi rules have no {rule}"));
            continue;
        }
        let mut partings = Vec::new();
        let mut days = 0;
        for year in &RECORD {
            let want = published(year, rule);
            days += want.len();
            let found: BTreeSet<MonthDay> = years
                .get(&year.year)
                .into_iter()
                .flat_map(|y| &y.ekadashis)
                .filter(|fast: &&EkadashiFast| fast.rule == rule && fast.day.year == year.year)
                .map(|fast| (fast.day.month, fast.day.day))
                .collect();
            let mut extra: Vec<MonthDay> = found.difference(&want).copied().collect();
            for &missing in want.difference(&found) {
                let near = extra.iter().position(|&got| {
                    (ordinal(year.year, missing) - ordinal(year.year, got)).abs() <= 1
                });
                let got = near.map(|at| extra.remove(at));
                partings.push(Parting {
                    year: year.year,
                    published: Some(missing),
                    found: got,
                });
            }
            partings.extend(extra.into_iter().map(|got| Parting {
                year: year.year,
                published: None,
                found: Some(got),
            }));
        }
        for parting in &partings {
            let excused = PARTS
                .iter()
                .any(|excuse| excuse.rule == rule && excuse.parting() == *parting);
            if !excused {
                let Parting {
                    year,
                    published: want,
                    found: got,
                } = parting;
                problems.push(format!(
                    "{year} {rule}: published {want:?}, found {got:?}: a parting needs its cause in ekadashi::PARTS"
                ));
            }
        }
        for excuse in PARTS.iter().filter(|excuse| excuse.rule == rule) {
            let Excuse {
                year,
                published: want,
                found: got,
                cause,
                ..
            } = *excuse;
            if !partings.contains(&excuse.parting()) {
                problems.push(format!(
                    "{year} {rule} {want:?} → {got:?} is excused in ekadashi::PARTS and no longer parts: remove the entry"
                ));
            }
            if let Err(why) = holds(context, cause, year, got) {
                problems.push(format!("{year} {rule} {got:?}: {why}"));
            }
        }
        measured.push(Measured {
            rule,
            list,
            days,
            partings,
        });
    }
    measured
}

/// Whether a cause holds for the fast found on `day`.
fn holds(context: &Context, cause: Cause, year: i32, day: MonthDay) -> Result<(), String> {
    match cause {
        Cause::PakshaVardhini => {
            // The fortnight's last tithi falls within five days of its 11th.
            let at = |ahead: i32| {
                let (month, day) = super::date_of_year(year, ordinal(year, day) + ahead);
                super::date(year, (month, day))
            };
            let (days, _) = context
                .almanac()
                .of_each(
                    &at(2),
                    &at(6),
                    &super::place(),
                    UtcOffset::literal(5, 30, 0),
                )
                .map_err(|e| e.to_string())?;
            let held = days
                .value
                .iter()
                .filter(|p| {
                    p.limbs
                        .tithi
                        .first()
                        .is_some_and(|t| matches!(t.member, Tithi::Amavasya | Tithi::Purnima))
                })
                .count();
            if held == 2 {
                Ok(())
            } else {
                Err(format!(
                    "is excused as Paksha Vardhini, but the fortnight's last tithi holds {held} sunrises, not two"
                ))
            }
        }
    }
}

/// §3 of the page.
pub(super) fn render(out: &mut String, measured: &[Measured]) {
    let _ = writeln!(
        out,
        "\n## 3. Ekadashi against a published almanac\n\n\
         Drik Panchang's New Delhi lists for {}–{}: its Ekadashi list's day for\n\
         the Smarta householder, its Gauna day for the renunciant where that\n\
         differs, and its ISKCON list for the Vaishnava. The rules are\n\
         `EkadashiRule::dharmasindhu()` (`festival-rules.md` §8).\n",
        YEARS.start(),
        YEARS.end()
    );
    let claims: Vec<Claim> = measured
        .iter()
        .map(|m| {
            let claim = Claim::counted(
                format!("{} falls on the {} list's day", m.rule, m.list),
                m.partings.len(),
                m.days,
            );
            if m.partings.is_empty() {
                claim
            } else {
                claim.with_note("each parting is named below with its cause")
            }
        })
        .collect();
    out.push_str(&table(&claims));
    out.push('\n');
    if PARTS.is_empty() {
        out.push_str("No rule parts from the record.\n");
    } else {
        out.push_str("Where a rule parts from the record, and why:\n\n");
        for excuse in &PARTS {
            let _ = writeln!(
                out,
                "- {} {}: published {}, found {}: {}",
                excuse.year,
                excuse.rule,
                spell(excuse.published),
                spell(excuse.found),
                excuse.cause.said()
            );
        }
    }
}
