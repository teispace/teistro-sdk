//! The shipped rules against Nepal's national panchanga
//! (`03-design/festival-rules.md` §9.3 and §9.4).
//!
//! The Nepal Panchanga Decision Committee's national panchanga for VS 2082
//! and 2083 (npns.gov.np, the only two it hosts) prints each observance in
//! its "vrataparva vivarana" column beside the day's row; the days below
//! were read off its page images. Each rule of the `NEPAL` pack, the
//! days it counts from another's among them, is found at Kathmandu on Nepal's
//! clock twice: over the committee's own sky (`nepali-committee`, the Surya
//! Siddhanta with its bija, C187), which the pass holds to the print, and
//! over the modern sky (`nepali-default`), which it counts beside it. Every
//! parting under the committee's sky must carry its cause, and the cause is
//! checked against the observance found.

use std::fmt::Write as _;

use teistro::festival::{Case, Decided, FestivalRule, FollowingRule, Observance, Observances};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, FestivalPack, FestivalRequest, UtcOffset};
use teistro_core::catalogue::Calendar;

use crate::measure::{Claim, table};

use super::{decided_by, ordinal, spell};

/// A Gregorian day, as (year, month, day).
pub(super) type Day = (i32, u8, u8);

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
const PRINTED: [(i32, &str, Day, &str); 29] = [
    (
        2082,
        "RAKSHABANDHAN",
        (2025, 8, 9),
        "p. 9: रक्षाबन्धन, जनैपूर्णिमा",
    ),
    (2082, "UPAKARMA_MADHYANDINA", (2025, 8, 9), "p. 9: जनैपूर्णिमा"),
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
        "UPAKARMA_SAMAVEDI",
        (2025, 8, 26),
        "p. 11: सामवेदीहरूको उपाकर्म",
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
        "BALI_PRATIPADA",
        (2025, 10, 22),
        "p. 15: गोवर्धनपूजा, म्हपूजा, बलिपूजा",
    ),
    (
        2082,
        "YAMA_DWITIYA",
        (2025, 10, 23),
        "p. 15: यमद्वितीया, भाइटीका",
    ),
    (2082, "SHIVARATRI", (2026, 2, 15), "p. 22: महाशिवरात्रिव्रत"),
    (2082, "HOLIKA", (2026, 3, 2), "p. 23: राति भद्रान्तमा चिरदाह"),
    (
        2082,
        "HOLI_HILLS",
        (2026, 3, 2),
        "p. 23: पहाडी जिल्लामा होली",
    ),
    (2082, "HOLI_TERAI", (2026, 3, 3), "p. 23: तराईमा होली"),
    (
        2082,
        "RAMA_NAVAMI",
        (2026, 3, 27),
        "p. 25: रामनवमीव्रत, श्रीरामजयन्ती",
    ),
    (
        2083,
        "RAKSHABANDHAN",
        (2026, 8, 28),
        "p. 11: रक्षाबन्धन, जनैपूर्णिमा",
    ),
    (
        2083,
        "UPAKARMA_MADHYANDINA",
        (2026, 8, 28),
        "p. 11: जनैपूर्णिमा",
    ),
    (
        2083,
        "JANMASHTAMI",
        (2026, 9, 4),
        "p. 12: श्रीकृष्णजन्माष्टमीव्रत",
    ),
    (
        2083,
        "UPAKARMA_SAMAVEDI",
        (2026, 9, 13),
        "p. 13: सामवेदीहरूको उपाकर्म",
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
        "BALI_PRATIPADA",
        (2026, 11, 10),
        "p. 17: गोवर्धनपूजा, म्हपूजा, बलिपूजा",
    ),
    (
        2083,
        "YAMA_DWITIYA",
        (2026, 11, 11),
        "p. 17: यमद्वितीया (किजापूजा)",
    ),
    (2083, "SHIVARATRI", (2027, 3, 6), "p. 24: महाशिवरात्रिव्रत"),
    (2083, "HOLIKA", (2027, 3, 21), "p. 25: राति भद्रान्तमा चिरदाह"),
    (
        2083,
        "HOLI_HILLS",
        (2027, 3, 21),
        "p. 25: पहाडी जिल्लामा होली",
    ),
    (2083, "HOLI_TERAI", (2027, 3, 22), "p. 25: तराईमा होली"),
];

/// Why a rule over the committee's sky parts from the print, per (year,
/// rule), with the case and the deciding guard the cause asserts. The pass
/// fails on a parting with no entry, on an entry whose row agrees, and on
/// an entry the observance found does not bear out.
const PARTS: [(i32, &str, Case, Decided, &str); 3] = [
    (
        2083,
        "VIJAYA_DASHAMI",
        Case::EarlierOnly,
        Decided::Guard { index: 5 },
        "the committee keeps the day whose sunrise the 10th holds, until 10:51 by its print, with Shravana joining the 10th on the earlier day only; p. 71 gives the earlier day, which alone holds aparahna, and moves to the later only with Shravana joined there alone (C197)",
    ),
    (
        2083,
        "BALI_PRATIPADA",
        Case::LaterOnly,
        Decided::Otherwise,
        "the committee keeps the day whose sunrise the 1st holds, though by its print the 1st lasts only 15 ghatis 49 palas past it, until 12:41; p. 78 keeps that day only when the 1st lasts nine muhurtas (18 ghatis) past sunrise, and otherwise the earlier day the new moon pierces (C197)",
    ),
    (
        2083,
        "UPAKARMA_MADHYANDINA",
        Case::LaterOnly,
        Decided::Otherwise,
        "the committee prints Janai purnima on the day whose sunrise the full moon holds, until 9:16 by its print after a 5:41 sunrise, about nine ghatis; p. 47 gives the Madhyandina the later day only past six muhurtas (12 ghatis), and the earlier when less (C197)",
    ),
];

pub(super) fn kathmandu() -> (Place, UtcOffset) {
    (
        Place::new(
            Latitude::literal(27.7172),
            Longitude::literal(85.324),
            Altitude::literal(1400.0),
        ),
        UtcOffset::literal(5, 45, 0),
    )
}

pub(super) fn gregorian((year, month, day): Day) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, year, month, day)
}

/// A sky the rules are found over.
#[derive(Clone, Copy)]
pub(super) enum Sky {
    /// `nepali-committee`, the committee's own.
    Committee,
    /// `nepali-default`, the modern sky under Lahiri.
    Modern,
}

impl Sky {
    pub(super) fn context(self) -> Result<Context, String> {
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
    /// The text's rules over the committee's sky.
    text: Option<Observance>,
    /// The text's rules over the modern sky.
    modern: Option<Observance>,
    /// Every rule read at sunrise, over the committee's sky.
    udaya: Option<Observance>,
    /// The `NEPAL` pack over the committee's sky.
    nepal: Option<Observance>,
}

/// The text's rules, with the days Nepal counts from them.
fn text() -> FestivalRequest {
    FollowingRule::nepal().into_iter().fold(
        FestivalRequest::from(FestivalPack::Dharmasindhu),
        FestivalRequest::with_following,
    )
}

/// Whether `rule`'s rite lies in the daylight: a counted day's is the rite
/// it counts from.
fn in_daylight(rule: &str) -> Option<bool> {
    let leader = FollowingRule::nepal()
        .into_iter()
        .find(|counted| counted.key == rule)
        .map_or_else(|| rule.to_owned(), |counted| counted.after);
    FestivalRule::dharmasindhu()
        .into_iter()
        .find(|shipped| shipped.key == leader)
        .map(|shipped| shipped.at.in_daylight())
}

/// The rival C197 weighed: every rule, of the night as of the day, read as
/// the day whose sunrise holds its tithi.
fn udaya() -> FestivalRequest {
    let rules = FestivalRule::dharmasindhu()
        .into_iter()
        .map(FestivalRule::udaya)
        .collect();
    FollowingRule::nepal()
        .into_iter()
        .fold(FestivalRequest::new(rules), FestivalRequest::with_following)
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

/// Each printed row beside the day each reading gives, with a problem for
/// every parting of the text's rules under the committee's sky without its
/// cause, every cause without its parting or not borne out, and every
/// parting of the `NEPAL` pack, which C197 holds to the print whole.
pub(super) fn compare(problems: &mut Vec<String>) -> Result<Vec<Row>, String> {
    let text_request = text();
    let text = found(Sky::Committee, &text_request)?;
    let modern = found(Sky::Modern, &text_request)?;
    let rival = found(Sky::Committee, &udaya())?;
    let nepal = found(Sky::Committee, &FestivalRequest::from(FestivalPack::Nepal))?;
    let rows: Vec<Row> = PRINTED
        .iter()
        .map(|&(year, rule, printed, page)| Row {
            year,
            rule,
            printed,
            page,
            text: near(&text, rule, printed),
            modern: near(&modern, rule, printed),
            udaya: near(&rival, rule, printed),
            nepal: near(&nepal, rule, printed),
        })
        .collect();
    for row in &rows {
        let excuse = PARTS.iter().find(|p| (p.0, p.1) == (row.year, row.rule));
        let agrees = on(row.text.as_ref(), row.printed);
        match (agrees, excuse) {
            (false, None) => problems.push(format!(
                "VS {} {} printed {:?}, found {:?} by the text over the committee's sky: a parting needs its cause in nepal::PARTS",
                row.year,
                row.rule,
                row.printed,
                row.text
                    .as_ref()
                    .map(|o| (o.day.month, o.day.day, o.case, decided_by(&o.decided_by)))
            )),
            (true, Some(_)) => problems.push(format!(
                "VS {} {} agrees and is excused in nepal::PARTS: remove the entry",
                row.year, row.rule
            )),
            (false, Some((_, _, case, decided, _))) => {
                let found = row.text.as_ref().map(|o| (o.case, &o.decided_by));
                if found != Some((*case, decided)) {
                    problems.push(format!(
                        "VS {} {} is excused as {case:?} decided by {decided:?}, and the observance found is {found:?}",
                        row.year, row.rule
                    ));
                }
            }
            (true, None) => {}
        }
        if !on(row.udaya.as_ref(), row.printed) && in_daylight(row.rule) != Some(false) {
            problems.push(format!(
                "VS {} {}: read at sunrise it parts from the print, and its rite is not of the night; the page says every such parting is",
                row.year, row.rule
            ));
        }
        if !on(row.nepal.as_ref(), row.printed) {
            problems.push(format!(
                "VS {} {} printed {:?}, and the NEPAL pack gives {:?}: C197's reading (the text for a rite of the night, the tithi at sunrise for one of the daylight) no longer holds every printed day, so reopen it in festival-rules.md §9.5",
                row.year,
                row.rule,
                row.printed,
                row.nepal.as_ref().map(|o| (o.day.month, o.day.day))
            ));
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
         panchanga for VS 2082 and 2083, read off the page images, at Kathmandu on\n\
         Nepal's clock, beside four readings of each observance (Holi in the hills\n\
         and the Terai counted from the Holika day, §9.4): the text's rules over\n\
         the committee's own sky (`nepali-committee`, the Surya Siddhanta with its\n\
         bija, C187), each parting named with its cause; the same over the modern\n\
         sky (`nepali-default`); every rule read as the day whose sunrise holds\n\
         its tithi, C197's rival; and the `NEPAL` pack, the text for a rite of the\n\
         night and the tithi at sunrise for one of the daylight (§9.5), which the\n\
         pass holds to every printed day.\n\n",
    );
    out.push_str(
        "| VS | rule | printed | where | the text | case | decided by | modern sky | udaya | `NEPAL` |\n\
         |---:|---|---|---|---|---|---|---|---|---|\n",
    );
    for row in rows {
        let (case, by) = row.text.as_ref().map_or_else(
            || (String::new(), String::new()),
            |o| (format!("{:?}", o.case), decided_by(&o.decided_by)),
        );
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            row.year,
            row.rule,
            spell((row.printed.1, row.printed.2)),
            row.page,
            day_of(row.text.as_ref(), row.printed),
            case,
            by,
            day_of(row.modern.as_ref(), row.printed),
            day_of(row.udaya.as_ref(), row.printed),
            day_of(row.nepal.as_ref(), row.printed),
        );
    }
    let parted = |pick: fn(&Row) -> Option<&Observance>| {
        rows.iter()
            .filter(|row| !on(pick(row), row.printed))
            .count()
    };
    out.push('\n');
    // The pass has refused a parting of the text without its cause and any
    // parting of the NEPAL pack, so every one counted is named below.
    out.push_str(&table(&[
        Claim::counted(
            "the text's rules over the committee's sky fall on the printed day",
            parted(|row| row.text.as_ref()),
            rows.len(),
        )
        .with_note("each parting is named below with its cause"),
        Claim::counted(
            "the text's rules over the modern sky fall on the printed day",
            parted(|row| row.modern.as_ref()),
            rows.len(),
        )
        .with_note("the sky decides these: a tithi's end moves the day where the modern sky's and the text's part"),
        Claim::counted(
            "every rule read at sunrise, over the committee's sky, falls on the printed day",
            parted(|row| row.udaya.as_ref()),
            rows.len(),
        )
        .with_note("each parting is a rite of the night or the evening, where the committee keeps the text's window"),
        Claim::counted(
            "the `NEPAL` pack over the committee's sky falls on the printed day",
            parted(|row| row.nepal.as_ref()),
            rows.len(),
        )
        .with_note("C197: the text for a rite of the night, the tithi at sunrise for one of the daylight"),
    ]));
    out.push('\n');
    out.push_str(
        "Where the text's rules over the committee's sky part from the print, and why:\n\n",
    );
    for (year, rule, _, _, why) in PARTS {
        let _ = writeln!(out, "- VS {year} {rule}: {why}");
    }
}
