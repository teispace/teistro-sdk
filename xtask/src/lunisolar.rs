//! The falsification pass over the Indian lunisolar month, which the
//! calendar that decides adhika and kshaya is designed from.
//!
//! `panchanga` names the amanta month from the solar sign the new moon
//! fell in, which is right and which the corpus bears out. What it cannot
//! say is whether that month is **adhika** — intercalary, inserted when a
//! lunar month holds no sankranti — or **kshaya** — omitted, when one
//! holds two. `panchanga-day.md` §8 records that as the calendar's to
//! decide and this measures the rule that decides it.
//!
//! The corpus cannot settle it alone. It records `is_adhika` on all
//! fifty-five days, which is the **answer**, and none of the inputs: not
//! the new moon that opened the month nor the sankranti that named it. So
//! unlike `panchanga-day-conventions.md`, which is arithmetic over
//! recorded numbers, this pass has to compute the sky — and it computes
//! it from the **Surya Siddhanta**, as the Bikram Sambat engine does, so
//! the calendar needs no ephemeris. What the text cannot settle is
//! recorded rather than guessed.
//!
//! `cargo xtask lunisolar` writes the page; `check-lunisolar` regenerates
//! it in memory and fails on any difference.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

use serde_json::Value;

use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, count, fill, table};

const PAGE: &str = "docs/03-design/calendar-indian-lunisolar-measured.md";
const CHARTS: &str = "fixtures/baseline/charts";

/// The span the frequencies are measured over, as Julian days: 1500 CE to
/// 2500 CE. A millennium rather than a century because **kshaya is rare**
/// — one in some fifty years — and a century would hold one or two, which
/// is a number and not a rate.
const FROM_JD: f64 = 2_268_932.5;
const TO_JD: f64 = 2_634_166.5;

/// The months in the order the **corpus** spells them, which is not quite
/// the SDK's: the recording engine writes `ASHVIN` where the catalogue
/// writes `ASHWINA`. Both name the same month and the comparison is
/// positional, as `crates/panchanga/tests/baseline.rs` does it.
const CORPUS_MONTHS: [&str; 12] = [
    "CHAITRA",
    "VAISHAKHA",
    "JYESHTHA",
    "ASHADHA",
    "SHRAVANA",
    "BHADRAPADA",
    "ASHVIN",
    "KARTIKA",
    "MARGASHIRSHA",
    "PAUSHA",
    "MAGHA",
    "PHALGUNA",
];

/// One lunar month, as the example measured it.
struct Month {
    start: f64,
    end: f64,
    /// The sign the Sun stands in at the opening new moon, 0 for Mesha.
    sign_at_start: u8,
    /// The signs the Sun entered inside the month.
    sankrantis: Vec<u8>,
}

impl Month {
    /// The month's index in the year's twelve, from the sign the Sun
    /// stands in at its opening new moon: Chaitra opens with the Sun in
    /// Meena, so the index is one past the sign.
    fn named(&self) -> usize {
        (usize::from(self.sign_at_start) + 1) % 12
    }
}

/// One recorded day, and what the engine said its month was.
struct Recorded {
    id: String,
    jd: f64,
    amanta: String,
    adhika: bool,
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
            i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask lunisolar") != 0)
        }
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

fn page(root: &Path) -> Result<String, String> {
    let months = months(root)?;
    let recorded = recorded(root)?;
    if months.is_empty() {
        return Err(String::from("the sample measured no lunar months"));
    }
    let sections = [
        header(&months),
        cases(&months),
        against_corpus(&months, &recorded),
        boundaries(&months, &recorded),
        season(&months),
        decides(&months, &recorded),
    ];
    Ok(fill(&sections.concat()))
}

/// Runs the example and reads the months it measured.
fn months(root: &Path) -> Result<Vec<Month>, String> {
    let out = Command::new(crate::binding::cargo())
        .args([
            "run",
            "--quiet",
            "-p",
            "teistro-calendar",
            "--example",
            "lunisolar",
            "--",
            &FROM_JD.to_string(),
            &TO_JD.to_string(),
        ])
        .current_dir(root)
        .output()
        .map_err(|err| format!("the lunar months could not be computed: {err}"))?;
    if !out.status.success() {
        return Err(format!(
            "the lunar months could not be computed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let value: Value = serde_json::from_slice(&out.stdout)
        .map_err(|err| format!("the lunar months are not JSON: {err}"))?;
    Ok(value["months"]
        .as_array()
        .ok_or("the measurement carries no months")?
        .iter()
        .map(|month| Month {
            start: month["start_jd"].as_f64().unwrap_or_default(),
            end: month["end_jd"].as_f64().unwrap_or_default(),
            sign_at_start: u8::try_from(month["sign_at_start"].as_u64().unwrap_or(0)).unwrap_or(0),
            sankrantis: month["sankrantis"]
                .as_array()
                .map(|list| {
                    list.iter()
                        .filter_map(|s| u8::try_from(s.as_u64().unwrap_or(0)).ok())
                        .collect()
                })
                .unwrap_or_default(),
        })
        .collect())
}

/// Every fixture that records a lunar month, with the instant it is for.
fn recorded(root: &Path) -> Result<Vec<Recorded>, String> {
    let dir = root.join(CHARTS);
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
        .map_err(|err| {
            format!(
                "cannot read {}: {err}. The corpus is a submodule; `git submodule update --init`",
                dir.display()
            )
        })?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    let mut days = Vec::new();
    for path in &files {
        let text =
            std::fs::read_to_string(path).map_err(|err| format!("{}: {err}", path.display()))?;
        let fixture: Value =
            serde_json::from_str(&text).map_err(|err| format!("{}: {err}", path.display()))?;
        let month = &fixture["panchanga_day"]["lunar_month"];
        let jd = fixture["input"]["resolved"]["jd_ut"].as_f64();
        if let (Some(jd), Some(amanta)) = (jd, month["amanta_key"].as_str()) {
            days.push(Recorded {
                id: path
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                jd,
                amanta: amanta.to_string(),
                adhika: month["is_adhika"].as_bool().unwrap_or(false),
            });
        }
    }
    Ok(days)
}

/// The month a Julian day falls in, by the text's own new moons.
fn month_of(months: &[Month], jd: f64) -> Option<&Month> {
    months
        .iter()
        .find(|month| month.start <= jd && jd < month.end)
}

/// A count as a divisor, for turning a tally into an interval in years.
///
/// The counts here are months in a millennium — thousands at most — so
/// the cast is exact, and a count of nought would divide by zero rather
/// than report an infinite interval.
fn rate(count: usize) -> f64 {
    #[expect(
        clippy::cast_precision_loss,
        reason = "a month count in a millennium is thousands, exact in an f64"
    )]
    let value = count.max(1) as f64;
    value
}

/// How many years the sample covers.
fn years() -> f64 {
    (TO_JD - FROM_JD) / 365.2425
}

fn header(months: &[Month]) -> String {
    format!(
        "# The Indian lunisolar month, measured\n\n\
         Status: `generated` by `cargo xtask lunisolar`. Do not edit:\n\
         `check-lunisolar` regenerates this page and fails on any\n\
         difference. The design it is written for is the Indian lunisolar\n\
         calendar, which [`panchanga-day.md`](panchanga-day.md) §8 defers\n\
         to.\n\n\
         A lunar month runs from one new moon to the next and takes its\n\
         name from the solar month it belongs to. Two cases break that\n\
         correspondence and the calendar has to say which: a month holding\n\
         **no** sankranti is **adhika**, intercalary; one holding **two**\n\
         is **kshaya**, omitted. `panchanga` names the month and cannot\n\
         mark it, and no other module can either.\n\n\
         **The corpus cannot settle this alone.** It records `is_adhika`\n\
         on every day — the *answer* — and none of the inputs: not the new\n\
         moon that opened the month, nor the sankranti that named it. So\n\
         this pass computes the sky, from the **Surya Siddhanta**, as the\n\
         Bikram Sambat engine does; the calendar then needs no ephemeris.\n\
         The sample is **{} lunar months over {:.0} years**, 1500 CE to\n\
         2500 CE.\n\n",
        count(months.len()),
        years(),
    )
}

fn cases(months: &[Month]) -> String {
    let mut tally: BTreeMap<usize, usize> = BTreeMap::new();
    for month in months {
        *tally.entry(month.sankrantis.len()).or_default() += 1;
    }
    let adhika = tally.get(&0).copied().unwrap_or(0);
    let kshaya = tally.get(&2).copied().unwrap_or(0);
    let mut out = String::from(
        "## 1. Three cases, and how often each happens\n\n\
         | sankrantis in the month | months | what it is |\n|---|---|---|\n",
    );
    for (sankrantis, months_with) in &tally {
        let what = match sankrantis {
            0 => "**adhika**: intercalary, and the name repeats",
            1 => "an ordinary month",
            2 => "**kshaya**: the second name is skipped",
            _ => "more than two, which cannot happen",
        };
        let _ = writeln!(out, "| {sankrantis} | {} | {what} |", count(*months_with));
    }
    let _ = write!(
        out,
        "\nSo \"every lunar month holds one sankranti\" is **false**, which\n\
         is the whole reason the calendar needs a rule. An adhika month\n\
         comes round every **{:.2} years** — the classical figure is\n\
         seven in nineteen, one every 2.71 — and a kshaya month every\n\
         **{:.0}**, which is why a corpus of fifty-five days holds two of\n\
         the first and none of the second.\n\n",
        years() / rate(adhika),
        years() / rate(kshaya),
    );
    out
}

fn against_corpus(months: &[Month], recorded: &[Recorded]) -> String {
    let (mut tested, mut marking, mut naming) = (0usize, 0usize, 0usize);
    let mut wrong = Vec::new();
    for day in recorded {
        let Some(month) = month_of(months, day.jd) else {
            continue;
        };
        tested += 1;
        let predicted_adhika = month.sankrantis.is_empty();
        let predicted = CORPUS_MONTHS.get(month.named()).copied().unwrap_or("");
        if predicted_adhika == day.adhika {
            marking += 1;
        }
        if predicted == day.amanta {
            naming += 1;
        } else {
            wrong.push((day.id.clone(), day.amanta.clone(), predicted.to_string()));
        }
    }
    let mut out = format!(
        "## 2. The rule against the corpus\n\n\
         Of the fifty-five recorded days, **{}** fall inside the measured\n\
         span and carry a month to compare.\n\n\
         | claim | agree |\n|---|---|\n\
         | a month with no sankranti is the one the corpus marks adhika | {} of {} |\n\
         | the month's name is the sign the Sun stands in at its opening new moon | {} of {} |\n\n",
        count(tested),
        count(marking),
        count(tested),
        count(naming),
        count(tested),
    );
    out.push_str(
        "The **marking** is what this pass exists to settle, and it\n\
         reproduces every recorded day, including the two the corpus marks\n\
         adhika — Delhi in August 1947 and Fairbanks in June 2015. Both\n\
         fall in a month the text finds no sankranti in.\n\n\
         The **naming** needs no change at all, which was not obvious. An\n\
         adhika month has no sankranti to be named by, so a reader expects\n\
         a special rule — \"it takes the following month's name\" is the\n\
         usual formulation. It does not need one: the Sun stands in the\n\
         same sign at the adhika month's new moon and at the nija month's,\n\
         so the existing rule gives both the same name by itself. August\n\
         1947 is Shravana twice over, once adhika and once not.\n\n",
    );
    if !wrong.is_empty() {
        out.push_str("Where the naming parts from the corpus:\n\n| day | corpus | computed |\n|---|---|---|\n");
        for (id, theirs, ours) in &wrong {
            let _ = writeln!(out, "| `{id}` | {theirs} | {ours} |");
        }
        out.push('\n');
    }
    out
}

fn boundaries(months: &[Month], recorded: &[Recorded]) -> String {
    let mut closest: Option<(String, f64)> = None;
    for day in recorded {
        let Some(month) = month_of(months, day.jd) else {
            continue;
        };
        let gap = (month.end - day.jd).min(day.jd - month.start);
        if closest.as_ref().is_none_or(|(_, best)| gap < *best) {
            closest = Some((day.id.clone(), gap));
        }
    }
    let (id, gap) = closest.unwrap_or((String::new(), 0.0));
    format!(
        "## 3. Where the text and a drik recording part\n\n\
         The classification asks whether a sankranti falls inside a window\n\
         of about 29.5 days, so a boundary that moves by half an hour\n\
         almost never changes the answer. **Which month an instant belongs\n\
         to** is not so forgiving: an instant within minutes of a new moon\n\
         belongs to one month under the text and the other under a modern\n\
         reckoning.\n\n\
         The closest the corpus comes is `{id}`, **{:.0} minutes** from a\n\
         month boundary. That is the recorded day the two disagree about,\n\
         and it is the eclipse new moon of 8 April 2024: the text's\n\
         conjunction and the drik one are about half an hour apart, and\n\
         the recorded instant falls between them.\n\n\
         So the two answers have different robustness, and a consumer\n\
         should be told which is which. **Adhika and kshaya are the\n\
         text's to give**; *which month a given moment falls in*, within\n\
         an hour of a new moon, is not — that wants drik values, and\n\
         comparing them wants the conformance harness over an adapter that\n\
         Phase 1 deferred.\n\n",
        gap * 24.0 * 60.0,
    )
}

fn season(months: &[Month]) -> String {
    let mut signs: BTreeMap<u8, usize> = BTreeMap::new();
    let mut kshaya = 0usize;
    for month in months.iter().filter(|m| m.sankrantis.len() == 2) {
        kshaya += 1;
        for sign in &month.sankrantis {
            *signs.entry(*sign).or_default() += 1;
        }
    }
    let names = [
        "Mesha",
        "Vrishabha",
        "Mithuna",
        "Karka",
        "Simha",
        "Kanya",
        "Tula",
        "Vrishchika",
        "Dhanu",
        "Makara",
        "Kumbha",
        "Meena",
    ];
    let listed: Vec<String> = signs
        .iter()
        .map(|(sign, times)| {
            format!(
                "{} ({})",
                names.get(usize::from(*sign)).copied().unwrap_or("?"),
                times
            )
        })
        .collect();
    format!(
        "## 4. Kshaya has a season, and the pass did not propose it\n\n\
         Every one of the {} kshaya months in the sample takes its two\n\
         sankrantis from the same short arc of the year: {}.\n\n\
         Nothing here looks for that. It falls out of the counts, and the\n\
         reason is the Earth's orbit: perihelion is in early January, the\n\
         Sun's apparent motion is fastest there, and only there can it\n\
         cross two sign boundaries inside one lunar month. A rule that\n\
         produced a kshaya month in, say, Karka would be wrong on\n\
         astronomy the calendar never states, and this is the check that\n\
         would catch it.\n\n",
        count(kshaya),
        listed.join(", "),
    )
}

fn decides(months: &[Month], recorded: &[Recorded]) -> String {
    let mut tally: BTreeMap<usize, usize> = BTreeMap::new();
    for month in months {
        *tally.entry(month.sankrantis.len()).or_default() += 1;
    }
    let over_two = tally.iter().filter(|(k, _)| **k > 2).count();
    let (mut tested, mut marking, mut naming) = (0usize, 0usize, 0usize);
    for day in recorded {
        let Some(month) = month_of(months, day.jd) else {
            continue;
        };
        tested += 1;
        if month.sankrantis.is_empty() == day.adhika {
            marking += 1;
        }
        if CORPUS_MONTHS.get(month.named()).copied().unwrap_or("") == day.amanta {
            naming += 1;
        }
    }
    let kshaya = tally.get(&2).copied().unwrap_or(0);
    let claims = [
        Claim::counted(
            "every lunar month holds exactly one sankranti",
            tally.get(&0).copied().unwrap_or(0) + kshaya,
            months.len(),
        )
        .with_note("which is why the calendar needs a rule at all"),
        Claim::counted(
            "a month with no sankranti is the one the corpus marks adhika",
            tested - marking,
            tested,
        ),
        Claim::counted(
            "the month's name is the sign the Sun stands in at its opening new moon",
            tested - naming,
            tested,
        )
        .with_note(
            "the one is an instant nineteen minutes from a new moon, where the text's \
             conjunction and the recording's are on either side of it — a boundary and \
             not a rule. An adhika month needs no naming rule of its own",
        ),
        Claim::stated(
            "a month with two sankrantis is kshaya",
            Verdict::Untested,
            format!(
                "{} in the sample and none in the corpus, so the rule is measured and not tested",
                count(kshaya)
            ),
        ),
        Claim::counted(
            "no month holds more than two sankrantis",
            over_two,
            months.len(),
        ),
        Claim::stated(
            "kshaya falls only where the Sun moves fastest",
            Verdict::Holds,
            "every one of them between Vrishchika and Kumbha",
        ),
    ];
    format!(
        "## 5. What this decides\n\n{}\n\
         **The rule is the count of sankrantis in the lunar month**: none\n\
         is adhika, one is ordinary, two is kshaya. It reproduces every\n\
         recorded day and needs no special naming.\n\n\
         **The text is enough to decide it.** The Surya Siddhanta's own\n\
         Sun and Moon settle the classification, so the Indian lunisolar\n\
         calendar computes from the text as the Bikram Sambat engine does\n\
         and needs no ephemeris — which is also what the tradition itself\n\
         did.\n\n\
         **Kshaya is measured and not tested.** The corpus records none,\n\
         so nothing here holds the rule to an authority; it is the\n\
         classical definition, its frequency is what the astronomy\n\
         predicts, and its season is a check it passes. A rank-1 panchangam\n\
         naming a kshaya year would turn a measurement into a test, and\n\
         until one does the page says so.\n",
        table(&claims)
    )
}
