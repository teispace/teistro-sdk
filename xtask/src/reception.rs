//! Reception, measured (crux C210): Lilly's three worked examples (p. 112)
//! held as printed, and how often each kind of reception holds over the
//! corpus's recorded births, with the structural facts the shipped
//! `Reception` and `PlanetDignity::reception` rest on counted rather than
//! believed.
//!
//! Whose dignity a planet stands in is the other planet's essential
//! dignity at its place, so reception needs no table; what is measured is
//! how the doctrine's choices (which pairs are reported, which are scored)
//! fall on real skies.

use std::fmt::Write as _;
use std::path::Path;

use teistro::catalogue::Graha;
use teistro::hellenistic::{DignityKind, essential_dignity};
use teistro::{Context, Dignities, DignityRequest, DignityRules, Ephemeris, Sect, Triplicities};

use crate::births::{CHARTS, births};
use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, count, fill, table};

const PAGE: &str = "docs/03-design/reception-measured.md";

/// One of Lilly's worked receptions: two planets at their longitudes, the
/// chart's sect, and the kind he says they receive each other by.
struct Example {
    name: &'static str,
    first: (Graha, f64),
    second: (Graha, f64),
    sect: Sect,
    kind: DignityKind,
}

/// Lilly's worked examples (p. 112, read off the page image): two planets
/// at the longitudes his text names, the sect, and the kind he says they
/// receive each other by. The first two name signs only, so a planet is
/// set at the sign's fifth degree; Lilly's own table (p. 104) decides
/// every lesser dignity there.
const EXAMPLES: [Example; 3] = [
    Example {
        name: "the Sun in Aries and Mars in Leo, by house",
        first: (Graha::Sun, 5.0),
        second: (Graha::Mars, 125.0),
        sect: Sect::Day,
        kind: DignityKind::House,
    },
    Example {
        name: "Venus in Aries and the Sun in Taurus, by triplicity, by day",
        first: (Graha::Venus, 5.0),
        second: (Graha::Sun, 35.0),
        sect: Sect::Day,
        kind: DignityKind::Triplicity,
    },
    Example {
        name: "Venus at 24° Aries and Mars at 16° Gemini, by term",
        first: (Graha::Venus, 23.5),
        second: (Graha::Mars, 75.5),
        sect: Sect::Day,
        kind: DignityKind::Term,
    },
];

fn conformance() -> Result<Context, String> {
    Context::builder()
        .profile("conformance-baseline")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .map_err(|why| format!("the conformance profile: {why}"))
}

/// Whether two planets receive each other by `kind`: each standing in the
/// other's dignity of that kind.
fn mutual_by(
    (first, first_deg): (Graha, f64),
    (second, second_deg): (Graha, f64),
    sect: Sect,
    rules: &DignityRules,
    kind: DignityKind,
) -> Result<bool, String> {
    let at = |planet, deg| {
        essential_dignity(planet, deg, sect, rules).map_err(|why| format!("{planet:?}: {why}"))
    };
    Ok(at(second, first_deg)?.holds(kind) && at(first, second_deg)?.holds(kind))
}

/// Lilly's examples as printed, and the second's converse by night, where
/// fire is Jupiter's and earth the Moon's.
fn example_claims() -> Result<Vec<Claim>, String> {
    let rules = DignityRules::LILLY;
    let mut failing = Vec::new();
    for one in &EXAMPLES {
        if !mutual_by(one.first, one.second, one.sect, &rules, one.kind)? {
            failing.push(one.name);
        }
    }
    let [_, triplicity, _] = &EXAMPLES;
    let by_night = mutual_by(
        triplicity.first,
        triplicity.second,
        Sect::Night,
        &rules,
        triplicity.kind,
    )?;
    Ok(vec![
        Claim::counted(
            "Lilly's three receptions (p. 112) hold as he prints them, under his own table (p. 104)",
            failing.len(),
            EXAMPLES.len(),
        )
        .with_note(if failing.is_empty() {
            String::from("by house, by triplicity and by term")
        } else {
            failing.join("; ")
        }),
        Claim::stated(
            "the triplicity example is a day chart's alone (\"if the Question or Nativity be by day\")",
            if by_night { Verdict::Falsified } else { Verdict::Holds },
            if by_night {
                "it holds by night too"
            } else {
                "by night it does not hold"
            },
        ),
    ])
}

/// One birth's dignities under each triplicity scheme.
struct Read {
    lilly: Dignities,
    ptolemy: Dignities,
}

/// How many times a planet is received by `kind` in one chart, by how many
/// partners: a sign has one lord, one exalted planet, one term lord and one
/// face lord, so only the triplicity, ruled jointly in Ptolemy's water,
/// can give two.
fn partners(read: &Dignities, planet: Graha, kind: DignityKind) -> usize {
    read.receptions
        .iter()
        .filter(|one| one.planets.contains(&planet) && one.mutual_by(kind))
        .count()
}

/// Each planet's reception score recomputed from its pairs: how many
/// disagree with the shipped one, how many score at all, and how many of
/// those are peregrine.
struct Scoring {
    wrong: usize,
    scored: usize,
    peregrine: usize,
}

impl Scoring {
    fn of(reads: &[Read]) -> Scoring {
        let mut scoring = Scoring {
            wrong: 0,
            scored: 0,
            peregrine: 0,
        };
        for read in reads.iter().map(|read| &read.lilly) {
            for at in &read.planets {
                let earned: i16 = [
                    (DignityKind::House, read.scores.house),
                    (DignityKind::Exaltation, read.scores.exaltation),
                ]
                .into_iter()
                .filter(|&(kind, _)| partners(read, at.planet, kind) > 0)
                .map(|(_, worth)| i16::from(worth))
                .sum();
                scoring.wrong += usize::from(earned != at.reception);
                if at.reception != 0 {
                    scoring.scored += 1;
                    scoring.peregrine += usize::from(at.dignity.peregrine());
                }
            }
        }
        scoring
    }
}

fn corpus_claims(reads: &[Read]) -> Vec<Claim> {
    let charts = reads.len();
    let all = || reads.iter().map(|read| &read.lilly);
    let pairs: usize = all().map(|read| read.receptions.len()).sum();
    let by_kind: Vec<String> = DignityKind::ALL
        .into_iter()
        .map(|kind| {
            let held = all()
                .flat_map(|read| &read.receptions)
                .filter(|one| one.mutual_by(kind))
                .count();
            let charts_with = all()
                .filter(|read| read.receptions.iter().any(|one| one.mutual_by(kind)))
                .count();
            format!("{kind:?} {held} (in {charts_with} charts)").to_lowercase()
        })
        .collect();
    let mixed = all()
        .flat_map(|read| &read.receptions)
        .filter(|one| one.mutual().next().is_none())
        .count();
    let one_sided = all()
        .flat_map(|read| &read.receptions)
        .filter(|one| one.first_in.peregrine() || one.second_in.peregrine())
        .count();
    let doubled: usize = reads
        .iter()
        .flat_map(|read| {
            read.lilly.planets.iter().flat_map(move |at| {
                DignityKind::ALL
                    .into_iter()
                    .filter(move |&kind| partners(&read.lilly, at.planet, kind) > 1)
            })
        })
        .count();
    let water_twice: usize = reads
        .iter()
        .map(|read| {
            read.ptolemy
                .planets
                .iter()
                .filter(|at| partners(&read.ptolemy, at.planet, DignityKind::Triplicity) > 1)
                .count()
        })
        .sum();
    let scoring = Scoring::of(reads);
    vec![
        Claim::stated(
            "how many pairs receive each other, and by what",
            Verdict::Holds,
            format!(
                "{} pairs over {} charts; mutual by {}; {} mixed (by no kind both ways)",
                count(pairs),
                count(charts),
                by_kind.join(", "),
                count(mixed),
            ),
        ),
        Claim::counted(
            "every reported pair has each planet standing in at least one of the other's five dignities",
            one_sided,
            pairs,
        ),
        Claim::counted(
            "under Lilly's triplicities a planet is received by any one kind by one partner at most, so a kind's score is earned once",
            doubled,
            charts * 7 * DignityKind::ALL.len(),
        ),
        Claim::stated(
            "under Ptolemy's, a planet in water can be received by triplicity by two partners, Venus or the Moon and Mars, and both are reported",
            Verdict::Holds,
            format!("{} planets so received over the corpus", count(water_twice)),
        ),
        Claim::counted(
            "each planet's reception score is Lilly's house worth when received by house and his exaltation worth when by exaltation, and nothing else",
            scoring.wrong,
            charts * 7,
        )
        .with_note(format!(
            "{} planets score by reception; {} of them peregrine, which reception does not lift (C210)",
            count(scoring.scored),
            count(scoring.peregrine)
        )),
    ]
}

fn page(root: &Path) -> Result<String, String> {
    let sdk = conformance()?;
    let births = births(root, &sdk)?;
    if births.is_empty() {
        return Err(format!("{CHARTS} records no chart"));
    }
    let ptolemy = DignityRequest::default()
        .with_rules(DignityRules::LILLY.with_triplicities(Triplicities::Ptolemy));
    let reads = births
        .iter()
        .map(|birth| {
            let read = |request: &DignityRequest| {
                sdk.chart()
                    .dignities(&birth.document, request)
                    .map_err(|why| format!("{}: {why}", birth.name))
            };
            Ok(Read {
                lilly: read(&DignityRequest::default())?,
                ptolemy: read(&ptolemy)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let claims: Vec<Claim> = example_claims()?
        .into_iter()
        .chain(corpus_claims(&reads))
        .collect();
    let mut out = String::from(
        "# Reception, measured\n\n\
         Status: `generated` by `cargo xtask reception` from Lilly's \
         *Christian Astrology* (1647) and the corpus's recorded births, \
         2026-10-02. Do not edit: `check-reception` regenerates this page \
         and fails on any difference.\n\n\
         Crux C210 asks which receptions count and what they score. Lilly \
         calls it reception when two planets stand \"in each others \
         dignity\", by any essential dignity, and his table (p. 115) scores \
         only mutual reception by house and by exaltation \
         (`essential-dignities.md` §Reception). The SDK reports every pair \
         whole and scores those two alone, apart from a planet's own score. ",
    );
    let _ = write!(
        out,
        "This page holds Lilly's worked examples to the shipped tables and \
         measures, at each of the corpus's {} births under the default \
         request (Valens's horizon, Lilly's tables and scores), how often \
         each kind holds; the triplicity claim reads the same births under \
         Ptolemy's triplicities.\n\n",
        count(births.len())
    );
    out.push_str(&table(&claims));
    out.push_str(
        "\n## What it means\n\n\
         Lilly scores two kinds and the SDK scores those two. Every other \
         reception, mixed or by a lesser dignity, is reported with both \
         sides whole, so a reader who scores more adds the kinds each pair \
         carries rather than the SDK giving a worth Lilly never printed. \
         The last row counts how many planets scored by reception are \
         peregrine all the same: that is how many totals the choice not \
         to lift peregrine moves.\n",
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
            i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask reception") != 0)
        }
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}
