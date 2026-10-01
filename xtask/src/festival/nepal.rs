//! The shipped rules against Nepal's national panchanga
//! (`03-design/festival-rules.md` §9.3).
//!
//! The Nepal Panchanga Decision Committee's national panchanga for VS 2082
//! and 2083 (npns.gov.np, the only two it hosts) prints each observance in
//! its "vrataparva vivarana" column beside the day's row; the days below
//! were read off its page images. Each rule is found at Kathmandu on Nepal's
//! clock twice: over the committee's own sky (`nepali-committee`, the Surya
//! Siddhanta with its bija, C187), which the pass holds to the print, and
//! over the modern sky (`nepali-default`), which it counts beside it. Every
//! parting under the committee's sky must carry its cause, and the cause is
//! checked against the observance found.

use std::fmt::Write as _;

use teistro::festival::{Case, Decided, Observance, Observances};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, FestivalRequest, UtcOffset};
use teistro_core::catalogue::Calendar;

use crate::measure::{Claim, table};

use super::{ordinal, spell};

/// A Gregorian day, as (year, month, day).
type Day = (i32, u8, u8);

/// The days founded together, `(from, to)`: the seasons the rules of the
/// two printed years fall in.
const SPANS: [(Day, Day); 4] = [
    ((2025, 8, 1), (2025, 11, 20)),
    ((2026, 2, 1), (2026, 3, 31)),
    ((2026, 8, 1), (2026, 11, 20)),
    ((2027, 2, 1), (2027, 3, 31)),
];

/// What the committee printed: the Vikram year, the rule, the Gregorian
/// day (the row's own column), and the page and the words it prints. VS
/// 2083 ends at Chaitra's bright 7th, before Rama Navami.
const PRINTED: [(i32, &str, Day, &str); 15] = [
    (
        2082,
        "JANMASHTAMI",
        (2025, 8, 16),
        "p. 10: श्रीकृष्णजन्माष्टमीव्रत",
    ),
    (
        2082,
        "HARITALIKA",
        (2025, 8, 26),
        "p. 11: हरितालिकाव्रत (तीज)",
    ),
    (
        2082,
        "NAVARATRA_ARAMBHA",
        (2025, 9, 22),
        "p. 13: नवरात्रारम्भ",
    ),
    (
        2082,
        "VIJAYA_DASHAMI",
        (2025, 10, 2),
        "p. 13: विजयादशमी, टीका",
    ),
    (
        2082,
        "LAKSHMI_PUJA",
        (2025, 10, 20),
        "p. 14: लक्ष्मीपूजा, दीपमालिका",
    ),
    (
        2082,
        "YAMA_DWITIYA",
        (2025, 10, 23),
        "p. 15: यमद्वितीया, भाइटीका",
    ),
    (2082, "SHIVARATRI", (2026, 2, 15), "p. 22: महाशिवरात्रिव्रत"),
    (
        2082,
        "RAMA_NAVAMI",
        (2026, 3, 27),
        "p. 25: रामनवमीव्रत, श्रीरामजयन्ती",
    ),
    (
        2083,
        "JANMASHTAMI",
        (2026, 9, 4),
        "p. 12: श्रीकृष्णजन्माष्टमीव्रत",
    ),
    (
        2083,
        "HARITALIKA",
        (2026, 9, 14),
        "p. 13: हरितालिकाव्रत (तीज)",
    ),
    (
        2083,
        "NAVARATRA_ARAMBHA",
        (2026, 10, 11),
        "p. 15: घटस्थापना, नवरात्रारम्भ",
    ),
    (
        2083,
        "VIJAYA_DASHAMI",
        (2026, 10, 21),
        "p. 15: विजयादशमी, दशैंको टीका",
    ),
    (
        2083,
        "LAKSHMI_PUJA",
        (2026, 11, 8),
        "p. 16: लक्ष्मीपूजा, दीपमालिका",
    ),
    (
        2083,
        "YAMA_DWITIYA",
        (2026, 11, 11),
        "p. 17: यमद्वितीया (किजापूजा)",
    ),
    (2083, "SHIVARATRI", (2027, 3, 6), "p. 24: महाशिवरात्रिव्रत"),
];

/// Why a rule over the committee's sky parts from the print, per (year,
/// rule), with the case and the deciding guard the cause asserts. The pass
/// fails on a parting with no entry, on an entry whose row agrees, and on
/// an entry the observance found does not bear out.
const PARTS: [(i32, &str, Case, Decided, &str); 1] = [(
    2083,
    "VIJAYA_DASHAMI",
    Case::EarlierOnly,
    Decided::Guard { index: 5 },
    "the committee keeps the day whose sunrise the 10th holds, until 10:51 by its print, with Shravana joining the 10th on the earlier day only; p. 71 gives the earlier day, which alone holds aparahna, and moves to the later only with Shravana joined there alone (C197)",
)];

fn kathmandu() -> (Place, UtcOffset) {
    (
        Place::new(
            Latitude::literal(27.7172),
            Longitude::literal(85.324),
            Altitude::literal(1400.0),
        ),
        UtcOffset::literal(5, 45, 0),
    )
}

fn gregorian((year, month, day): Day) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, year, month, day)
}

/// A sky the rules are found over.
#[derive(Clone, Copy)]
enum Sky {
    /// `nepali-committee`, the committee's own.
    Committee,
    /// `nepali-default`, the modern sky under Lahiri.
    Modern,
}

impl Sky {
    fn context(self) -> Result<Context, String> {
        let (profile, ephemeris) = match self {
            Sky::Committee => ("nepali-committee", Ephemeris::SuryaSiddhanta),
            Sky::Modern => ("nepali-default", Ephemeris::Builtin),
        };
        Context::builder()
            .profile(profile)
            .ephemeris([ephemeris])
            .build()
            .map_err(|e| e.to_string())
    }
}

/// Every span's observances over a sky, a worker per span.
fn found(sky: Sky, request: &FestivalRequest) -> Result<Observances, String> {
    let (place, clock) = kathmandu();
    std::thread::scope(|scope| {
        let workers: Vec<_> = SPANS
            .iter()
            .map(|&(from, to)| {
                scope.spawn(move || {
                    let found = sky
                        .context()?
                        .almanac()
                        .festivals(&gregorian(from), &gregorian(to), &place, clock, request)
                        .map_err(|e| format!("{from:?}: {e}"))?;
                    if found.value.unjudged.is_empty() {
                        Ok(found.value)
                    } else {
                        Err(format!("{from:?}: unjudged {:?}", found.value.unjudged))
                    }
                })
            })
            .collect();
        workers
            .into_iter()
            .try_fold(Observances::default(), |all, worker| {
                let part = worker
                    .join()
                    .map_err(|_| String::from("a worker finding Nepal's festivals panicked"))??;
                Ok(all.merged(part))
            })
    })
}

pub(super) struct Row {
    year: i32,
    rule: &'static str,
    printed: Day,
    page: &'static str,
    committee: Option<Observance>,
    modern: Option<Observance>,
}

fn on(found: Option<&Observance>, (year, month, day): Day) -> bool {
    found.is_some_and(|o| (o.day.year, o.day.month, o.day.day) == (year, month, day))
}

/// A day's place in a count that runs on across years, near enough to
/// order days a few weeks apart.
fn serial((year, month, day): Day) -> i32 {
    year * 366 + ordinal(year, (month, day))
}

/// The observance of `rule` within a fortnight of the printed day: a rule
/// falls once a year, so the one nearest is the year's.
fn near(observances: &Observances, rule: &str, printed: Day) -> Option<Observance> {
    let distance =
        |o: &Observance| (serial((o.day.year, o.day.month, o.day.day)) - serial(printed)).abs();
    observances
        .observances
        .iter()
        .filter(|o| o.rule == rule && distance(o) <= 15)
        .min_by_key(|o| distance(o))
        .cloned()
}

/// Each printed row beside the day each sky gives, with a problem for every
/// parting under the committee's sky without its cause, and every cause
/// without its parting or not borne out.
pub(super) fn compare(
    request: &FestivalRequest,
    problems: &mut Vec<String>,
) -> Result<Vec<Row>, String> {
    let committee = found(Sky::Committee, request)?;
    let modern = found(Sky::Modern, request)?;
    let rows: Vec<Row> = PRINTED
        .iter()
        .map(|&(year, rule, printed, page)| Row {
            year,
            rule,
            printed,
            page,
            committee: near(&committee, rule, printed),
            modern: near(&modern, rule, printed),
        })
        .collect();
    for row in &rows {
        let excuse = PARTS.iter().find(|p| (p.0, p.1) == (row.year, row.rule));
        let agrees = on(row.committee.as_ref(), row.printed);
        match (agrees, excuse) {
            (false, None) => problems.push(format!(
                "VS {} {} printed {:?}, found {:?} over the committee's sky: a parting needs its cause in nepal::PARTS",
                row.year,
                row.rule,
                row.printed,
                row.committee.as_ref().map(|o| (o.day.month, o.day.day, o.case, o.decided_by))
            )),
            (true, Some(_)) => problems.push(format!(
                "VS {} {} agrees and is excused in nepal::PARTS: remove the entry",
                row.year, row.rule
            )),
            (false, Some(&(_, _, case, decided, _))) => {
                let found = row.committee.as_ref().map(|o| (o.case, o.decided_by));
                if found != Some((case, decided)) {
                    problems.push(format!(
                        "VS {} {} is excused as {case:?} decided by {decided:?}, and the observance found is {found:?}",
                        row.year, row.rule
                    ));
                }
            }
            (true, None) => {}
        }
    }
    Ok(rows)
}

fn day_of(found: Option<&Observance>, printed: Day) -> String {
    found.map_or_else(
        || "none ✗".to_owned(),
        |o| {
            format!(
                "{}{}",
                spell((o.day.month, o.day.day)),
                if on(Some(o), printed) { "" } else { " ✗" }
            )
        },
    )
}

/// §4 of the page.
pub(super) fn render(out: &mut String, rows: &[Row]) {
    out.push_str(
        "\n## 4. Against Nepal's national panchanga\n\n\
         The days the Nepal Panchanga Decision Committee printed in its national\n\
         panchanga for VS 2082 and 2083, read off the page images, beside each\n\
         shipped rule's day at Kathmandu on Nepal's clock: over the committee's\n\
         own sky (`nepali-committee`, the Surya Siddhanta with its bija, C187),\n\
         which the pass holds to the print, and over the modern sky\n\
         (`nepali-default`), counted beside it.\n\n",
    );
    out.push_str(
        "| VS | rule | printed | where | committee's sky | case | decided by | modern sky |\n\
         |---:|---|---|---|---|---|---|---|\n",
    );
    for row in rows {
        let (case, by) = row.committee.as_ref().map_or_else(
            || (String::new(), String::new()),
            |o| {
                (
                    format!("{:?}", o.case),
                    match o.decided_by {
                        Decided::Guard { index } => format!("guard {index}"),
                        Decided::Otherwise => "otherwise".to_owned(),
                    },
                )
            },
        );
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            row.year,
            row.rule,
            spell((row.printed.1, row.printed.2)),
            row.page,
            day_of(row.committee.as_ref(), row.printed),
            case,
            by,
            day_of(row.modern.as_ref(), row.printed),
        );
    }
    let parted = |pick: fn(&Row) -> Option<&Observance>| {
        rows.iter()
            .filter(|row| !on(pick(row), row.printed))
            .count()
    };
    out.push('\n');
    // The pass has refused a parting under the committee's sky without its
    // cause, so every one counted is named below.
    out.push_str(&table(&[
        Claim::counted(
            "each shipped rule over the committee's sky falls on the printed day",
            parted(|row| row.committee.as_ref()),
            rows.len(),
        )
        .with_note("each parting is named below with its cause"),
        Claim::counted(
            "each shipped rule over the modern sky falls on the printed day",
            parted(|row| row.modern.as_ref()),
            rows.len(),
        )
        .with_note("the sky decides these: a tithi's end moves the day where the modern sky's and the text's part"),
    ]));
    out.push('\n');
    out.push_str("Where a rule over the committee's sky parts from the print, and why:\n\n");
    for (year, rule, _, _, why) in PARTS {
        let _ = writeln!(out, "- VS {year} {rule}: {why}");
    }
}
