//! Nepal's monthly full-moon fast against the committee's print
//! (`03-design/festival-rules.md` §9.6, C200).
//!
//! The Nepal Panchanga Decision Committee prints पूर्णिमाव्रत every month
//! in its national panchanga for VS 2082 and 2083; the 24 days below were
//! read off the page images, each from its row's own Gregorian column. The
//! `NEPAL` pack's `PURNIMA_VRATA` is found over both years at Kathmandu on
//! Nepal's clock, over the committee's own sky, which the pass holds to
//! every printed day, and over the modern sky, which it counts beside it.
//! Five rival readings are counted against the same rows: the text's
//! later day, the earlier evening first, any part of pradosha, p. 20's
//! eighteen nadis, and moonrise.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use teistro::festival::{Case, Choice, FestivalRule, Guard, Observance, Predicate, Window};
use teistro::{FestivalRequest, Panchanga};
use teistro_core::catalogue::Masa;

use crate::measure::{Claim, table};

use super::nepal::{Day, Sky, gregorian, kathmandu};
use super::spell;

/// What the committee printed: the Vikram year, the page, and the day
/// (the row's own Gregorian column). VS 2083's adhika Jyeshtha (अनला) is
/// p. 5's row.
const PRINTED: [(i32, u8, Day); 24] = [
    (2082, 3, (2025, 5, 12)),
    (2082, 5, (2025, 6, 10)),
    (2082, 7, (2025, 7, 10)),
    (2082, 9, (2025, 8, 8)),
    (2082, 11, (2025, 9, 7)),
    (2082, 13, (2025, 10, 6)),
    (2082, 15, (2025, 11, 5)),
    (2082, 17, (2025, 12, 4)),
    (2082, 19, (2026, 1, 3)),
    (2082, 21, (2026, 2, 1)),
    (2082, 23, (2026, 3, 2)),
    (2082, 25, (2026, 4, 1)),
    (2083, 3, (2026, 5, 1)),
    (2083, 5, (2026, 5, 30)),
    (2083, 7, (2026, 6, 29)),
    (2083, 9, (2026, 7, 29)),
    (2083, 11, (2026, 8, 27)),
    (2083, 13, (2026, 9, 26)),
    (2083, 15, (2026, 10, 25)),
    (2083, 17, (2026, 11, 24)),
    (2083, 19, (2026, 12, 23)),
    (2083, 21, (2027, 1, 22)),
    (2083, 23, (2027, 2, 20)),
    (2083, 25, (2027, 3, 21)),
];

/// The days founded either side of a printed day: enough for its full
/// moon's two days, whichever side of the print they fall.
const AROUND: i32 = 3;

/// The workers the rows are shared between, each with its own context.
const WORKERS: usize = 4;

/// The shipped rule's key.
const SHIPPED: &str = "PURNIMA_VRATA";

/// A reading of the fast's day, each a column of the page.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Reading {
    /// The `NEPAL` pack over the committee's sky.
    Shipped,
    /// The same over the modern sky.
    Modern,
    /// *Dharmasindhu* p. 20's "the later": the day whose sunrise the full
    /// moon holds, or the day a full moon holding none falls in.
    SunriseDay,
    /// The evening the full moon holds, the earlier first.
    EveningFirst,
    /// Any part of pradosha, the later first.
    Pradosha,
    /// p. 20's allowance for family rites: the earlier when the 14th lasts
    /// under eighteen nadis past its sunrise.
    EighteenNadis,
    /// The moonrise the full moon holds, the later first.
    Moonrise,
}

impl Reading {
    const ALL: [Reading; 7] = [
        Reading::Shipped,
        Reading::Modern,
        Reading::SunriseDay,
        Reading::EveningFirst,
        Reading::Pradosha,
        Reading::EighteenNadis,
        Reading::Moonrise,
    ];

    fn index(self) -> usize {
        match self {
            Reading::Shipped => 0,
            Reading::Modern => 1,
            Reading::SunriseDay => 2,
            Reading::EveningFirst => 3,
            Reading::Pradosha => 4,
            Reading::EighteenNadis => 5,
            Reading::Moonrise => 6,
        }
    }

    /// The column's heading.
    fn heading(self) -> &'static str {
        match self {
            Reading::Shipped => "`NEPAL`",
            Reading::Modern => "modern sky",
            Reading::SunriseDay => "sunrise day",
            Reading::EveningFirst => "evening, earlier first",
            Reading::Pradosha => "pradosha",
            Reading::EighteenNadis => "18 nadis",
            Reading::Moonrise => "moonrise",
        }
    }

    /// The claim the page counts.
    fn claim(self) -> &'static str {
        match self {
            Reading::Shipped => {
                "the `NEPAL` pack's PURNIMA_VRATA over the committee's sky falls on the printed day"
            }
            Reading::Modern => "the same over the modern sky falls on the printed day",
            Reading::SunriseDay => {
                "the full moon's sunrise day (Dharmasindhu p. 20's \"the later\") is the printed day"
            }
            Reading::EveningFirst => {
                "the evening the full moon holds, the earlier when both do, is the printed day"
            }
            Reading::Pradosha => {
                "the pradosha the full moon touches, the later when both do, is the printed day"
            }
            Reading::EighteenNadis => {
                "the earlier day when the 14th lasts under eighteen nadis past its sunrise (p. 20's allowance for family rites) is the printed day"
            }
            Reading::Moonrise => {
                "the moonrise the full moon holds, the later when both do, is the printed day"
            }
        }
    }

    /// The rival as a rule the evaluator reads, where it is one: the
    /// shipped rule with another key, window and table.
    fn rule(self, shipped: &FestivalRule) -> Option<FestivalRule> {
        let (key, at, decide) = match self {
            Reading::SunriseDay => (
                "PURNIMA_SUNRISE_DAY",
                Window::Sunrise,
                vec![Guard::new([case(Case::Neither)], Choice::Earlier)],
            ),
            Reading::EveningFirst => (
                "PURNIMA_EVENING_FIRST",
                Window::Sunset,
                vec![
                    Guard::new([case(Case::EarlierOnly)], Choice::Earlier),
                    Guard::new([case(Case::Both)], Choice::Earlier),
                ],
            ),
            Reading::Pradosha => (
                "PURNIMA_PRADOSHA",
                Window::Pradosha,
                vec![Guard::new([case(Case::EarlierOnly)], Choice::Earlier)],
            ),
            Reading::Shipped | Reading::Modern | Reading::EighteenNadis | Reading::Moonrise => {
                return None;
            }
        };
        Some(FestivalRule {
            key: key.to_owned(),
            source: "a rival C200 weighs".to_owned(),
            at,
            decide,
            otherwise: Choice::Later,
            ..shipped.clone()
        })
    }
}

const fn case(case: Case) -> Predicate {
    Predicate::Case { case }
}

/// The shipped rule.
fn shipped() -> Result<FestivalRule, String> {
    FestivalRule::nepal()
        .into_iter()
        .find(|rule| rule.key == SHIPPED)
        .ok_or_else(|| format!("the NEPAL pack has no {SHIPPED}"))
}

/// What a sky gives over both years: its observances and its days.
struct Found {
    observances: Vec<Observance>,
    days: BTreeMap<Day, Panchanga>,
}

fn day_of(date: &teistro::CalendarDate) -> Day {
    (date.year, date.month, date.day)
}

/// A day `by` days from another, across a year's end.
fn shifted((year, month, day): Day, by: i32) -> Day {
    let length = |year| super::month_lengths(year).iter().sum::<i32>();
    let (mut year, mut ordinal) = (year, super::ordinal(year, (month, day)) + by);
    if ordinal < 1 {
        year -= 1;
        ordinal += length(year);
    } else if ordinal > length(year) {
        ordinal -= length(year);
        year += 1;
    }
    let (month, day) = super::date_of_year(year, ordinal);
    (year, month, day)
}

/// The rules over a sky around each printed day, the rows shared between
/// workers, each with its own context.
fn found(sky: Sky, rules: &[FestivalRule]) -> Result<Found, String> {
    let (place, clock) = kathmandu();
    let request = FestivalRequest::new(rules.to_vec());
    let rows = PRINTED.len().div_ceil(WORKERS);
    std::thread::scope(|scope| {
        let workers: Vec<_> = PRINTED
            .chunks(rows)
            .map(|chunk| {
                let request = &request;
                scope.spawn(move || {
                    let almanac = sky.context()?;
                    chunk
                        .iter()
                        .map(|&(_, _, printed)| {
                            let (from, to) = (shifted(printed, -AROUND), shifted(printed, AROUND));
                            almanac
                                .almanac()
                                .festivals_with_days(
                                    &gregorian(from),
                                    &gregorian(to),
                                    &place,
                                    clock,
                                    request,
                                )
                                .map_err(|e| format!("{printed:?}: {e}"))
                        })
                        .collect::<Result<Vec<_>, String>>()
                })
            })
            .collect();
        let mut all = Found {
            observances: Vec::new(),
            days: BTreeMap::new(),
        };
        for worker in workers {
            let parts = worker
                .join()
                .map_err(|_| String::from("a worker finding the full moons panicked"))??;
            for part in parts {
                if !part.answer.value.unjudged.is_empty() {
                    return Err(format!("unjudged {:?}", part.answer.value.unjudged));
                }
                for observance in part.answer.value.observances {
                    if !all.observances.contains(&observance) {
                        all.observances.push(observance);
                    }
                }
                for day in part.days.value {
                    all.days.insert(day_of(&day.day.date), day);
                }
            }
        }
        Ok(all)
    })
}

/// The observance of `rule` within two days of the printed day.
fn near<'f>(found: &'f Found, rule: &str, printed: Day) -> Option<&'f Observance> {
    let serial = |(y, m, d): Day| super::ordinal(y, (m, d)) + y * 366;
    found
        .observances
        .iter()
        .filter(|o| o.rule == rule && (serial(day_of(&o.day)) - serial(printed)).abs() <= 2)
        .min_by_key(|o| (serial(day_of(&o.day)) - serial(printed)).abs())
}

/// A ghati, as a fraction of a day.
const GHATI: f64 = 1.0 / 60.0;

/// The readings computed from the shipped observance's two days rather
/// than evaluated as rules.
fn computed(reading: Reading, observance: &Observance, days: &BTreeMap<Day, Panchanga>) -> Option<Day> {
    let earlier = days.get(&day_of(&observance.extents[0].day))?;
    let later = days.get(&day_of(&observance.extents[1].day))?;
    let pick = |earlier_wins: bool| {
        if earlier_wins {
            day_of(&earlier.day.date)
        } else {
            day_of(&later.day.date)
        }
    };
    match reading {
        Reading::EighteenNadis => Some(pick(
            observance.tithi.from.get() < earlier.day.sunrise.get() + 18.0 * GHATI,
        )),
        Reading::Moonrise => {
            let holds = |day: &Panchanga| {
                day.moon
                    .rises
                    .first()
                    .is_some_and(|rise| observance.tithi.contains(*rise))
            };
            Some(pick(!holds(later) && holds(earlier)))
        }
        _ => None,
    }
}

/// One printed row and the day each reading gives.
pub(super) struct Row {
    year: i32,
    page: u8,
    printed: Day,
    month: Option<(Masa, bool)>,
    case: Option<Case>,
    readings: [Option<Day>; 7],
}

/// Each printed row beside each reading, with a problem for every row the
/// shipped rule over the committee's sky does not give.
pub(super) fn compare(problems: &mut Vec<String>) -> Result<Vec<Row>, String> {
    let shipped = shipped()?;
    let mut rules = vec![shipped.clone()];
    rules.extend(Reading::ALL.iter().filter_map(|reading| reading.rule(&shipped)));
    let committee = found(Sky::Committee, &rules)?;
    let modern = found(Sky::Modern, std::slice::from_ref(&shipped))?;
    let rows: Vec<Row> = PRINTED
        .iter()
        .map(|&(year, page, printed)| {
            let found = near(&committee, SHIPPED, printed);
            let mut readings = [None; 7];
            for reading in Reading::ALL {
                readings[reading.index()] = match reading {
                    Reading::Shipped => found.map(|o| day_of(&o.day)),
                    Reading::Modern => near(&modern, SHIPPED, printed).map(|o| day_of(&o.day)),
                    Reading::EighteenNadis | Reading::Moonrise => {
                        found.and_then(|o| computed(reading, o, &committee.days))
                    }
                    _ => reading
                        .rule(&shipped)
                        .and_then(|rule| near(&committee, &rule.key, printed))
                        .map(|o| day_of(&o.day)),
                };
            }
            Row {
                year,
                page,
                printed,
                month: found.map(|o| (o.month, o.adhika)),
                case: found.map(|o| o.case),
                readings,
            }
        })
        .collect();
    for row in &rows {
        if row.readings[Reading::Shipped.index()] != Some(row.printed) {
            problems.push(format!(
                "VS {} p. {}: the committee printed पूर्णिमाव्रत on {:?}, and the NEPAL pack's PURNIMA_VRATA gives {:?}: C200's reading (the evening the full moon holds, the later first) no longer holds every printed day, so reopen festival-rules.md §9.6",
                row.year,
                row.page,
                row.printed,
                row.readings[Reading::Shipped.index()]
            ));
        }
    }
    Ok(rows)
}

fn shown(found: Option<Day>, printed: Day) -> String {
    found.map_or_else(
        || "none ✗".to_owned(),
        |day| {
            format!(
                "{}{}",
                spell((day.1, day.2)),
                if day == printed { "" } else { " ✗" }
            )
        },
    )
}

/// §5 of the page.
pub(super) fn render(out: &mut String, rows: &[Row]) {
    out.push_str(
        "\n## 5. Nepal's monthly full-moon fast\n\n\
         The committee prints पूर्णिमाव्रत every month of VS 2082 and 2083, the\n\
         adhika Jyeshtha too; the 24 days were read off the page images, each\n\
         from its row's own Gregorian column. No text in hand states the rule:\n\
         *Dharmasindhu* p. 20 gives the full moon the later day. The `NEPAL`\n\
         pack's `PURNIMA_VRATA` takes the day whose sunset the full moon holds,\n\
         the later when both or neither do (§9.6, C200). It is found at\n\
         Kathmandu on Nepal's clock over the committee's own sky, which the pass\n\
         holds to every printed day, and beside it the modern sky and five rival\n\
         readings. The case is the shipped rule's, at the two sunsets.\n\n",
    );
    let mut heading = String::from("| VS | page | printed | month | case |");
    let mut rule = String::from("|---:|---:|---|---|---|");
    for reading in Reading::ALL {
        let _ = write!(heading, " {} |", reading.heading());
        rule.push_str("---|");
    }
    let _ = writeln!(out, "{heading}\n{rule}");
    for row in rows {
        let month = row.month.map_or_else(String::new, |(masa, adhika)| {
            format!("{}{}", masa.key(), if adhika { " (adhika)" } else { "" })
        });
        let case = row.case.map_or_else(String::new, |case| format!("{case:?}"));
        let _ = write!(
            out,
            "| {} | {} | {} | {month} | {case} |",
            row.year,
            row.page,
            spell((row.printed.1, row.printed.2)),
        );
        for reading in Reading::ALL {
            let _ = write!(out, " {} |", shown(row.readings[reading.index()], row.printed));
        }
        out.push('\n');
    }
    out.push('\n');
    let claims: Vec<Claim> = Reading::ALL
        .iter()
        .map(|reading| {
            let parted = rows
                .iter()
                .filter(|row| row.readings[reading.index()] != Some(row.printed))
                .count();
            Claim::counted(reading.claim(), parted, rows.len())
        })
        .collect();
    out.push_str(&table(&claims));
}
