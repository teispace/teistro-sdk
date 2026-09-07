//! The falsification pass over the drishti, which the aspect module is
//! designed from (Phase 4).
//!
//! **The corpus records no aspect.** Every other Phase 4 module was
//! designed against a recorded answer: the divisional charts had 19 530
//! placements, the planetary state 837 readings, the daily panchanga
//! twenty-seven fields. `aspect` has none — not a drishti value, not an
//! aspect list, not a field in the fixture schema. So this pass measures
//! what it still can, which turns out to be four things worth having:
//!
//! 1. The drishti systems' **own invariants**, which are decidable
//!    without any recording at all — a full aspect is mutual only
//!    through the seventh; rashi drishti is mutual everywhere.
//! 2. The systems **against each other** over the corpus's real
//!    placements, which is the bhava-chalit pass's shape: they are not
//!    variants of one thing, and a value has to say which it used.
//! 3. The six avasthas `crates/state` **refused**, retried with a real
//!    drishti in hand. That is a genuine recorded answer — 651 readings
//!    the corpus does carry — and it is the reason this pass exists at
//!    all.
//! 4. What is still missing, stated as the specification the missing
//!    thing must meet.
//!
//! `cargo xtask aspect` writes the page; `check-aspect` regenerates it in
//! memory and fails on any difference.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use serde_json::Value;
use teistro_core::catalogue::{Graha, Modality, Nature, Rashi};

use crate::classical::{
    GRAHAS, PER_SIGN, edge, house_count, natural, panchadha, sign_of, temporary,
};
use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, count, fill, table, verdict_of};

const PAGE: &str = "docs/03-design/aspect-drishti-measured.md";
const CHARTS: &str = "fixtures/baseline/charts";
const VARIANTS: &str = "fixtures/baseline/variants";

/// Every word an aspect would be recorded under, looked for in every
/// fixture so that "the corpus records none" is measured and not assumed.
const ASPECT_WORDS: [&str; 6] = [
    "aspect",
    "drishti",
    "drsti",
    "sphuta_drishti",
    "graha_drishti",
    "rashi_drishti",
];

/// The quarters of a full aspect. The tradition counts a drishti in
/// quarters and a full one is four of them.
const FULL: u8 = 4;

/// Which house ahead a graha aspects, and with how many quarters, before
/// the three special grahas raise their own (BPHS as
/// `01-research/feature-universe/01-vedic-parashari-core.md` §G reads
/// it: full on the 7th, three quarters on the 4th and 8th, half on the
/// 5th and 9th, a quarter on the 3rd and 10th).
const QUARTERS: [(u8, u8); 7] = [(3, 1), (4, 3), (5, 2), (7, FULL), (8, 3), (9, 2), (10, 1)];

/// The three grahas with a full aspect beyond the seventh, and which
/// houses they raise.
const SPECIAL: [(&str, [u8; 2]); 3] = [("MARS", [4, 8]), ("JUPITER", [5, 9]), ("SATURN", [3, 10])];

/// The three readings of a node's aspect the settings offer
/// (`aspect.node_aspects`), as the houses each adds beyond the seventh.
const NODE_READINGS: [(&str, &[u8]); 3] = [
    ("NONE", &[]),
    ("FIVE_SEVEN_NINE", &[5, 9]),
    ("THREE_SEVEN_ELEVEN", &[3, 11]),
];

/// The signs a body is said to thirst in: the watery ones.
const WATERY: [&str; 3] = ["CANCER", "SCORPIO", "PISCES"];

// ── what the corpus records ────────────────────────────────────────────────

/// One body's recorded placement and the avasthas that turn on an aspect.
#[derive(Clone)]
struct Reading {
    body: &'static str,
    longitude: f64,
    house: u8,
    dignity: String,
    /// Absent for the nodes and the lagna.
    deeptadi: Option<String>,
    lajjitadi: Vec<String>,
    /// Whether the engine flagged it within a hair of a sign edge.
    near_sign: bool,
}

impl Reading {
    /// The sign it stands in.
    fn sign(&self) -> u8 {
        sign_of(self.longitude)
    }

    /// Its graha.
    fn graha(&self) -> Option<Graha> {
        Graha::from_key(self.body)
    }

    /// Whether it carries the avastha family that turns on an aspect.
    fn has_avasthas(&self) -> bool {
        self.deeptadi.is_some()
    }
}

/// One fixture: every body's reading, and the signs they stand in.
struct Chart {
    readings: Vec<Reading>,
    signs: BTreeMap<String, u8>,
    /// The sign the lagna falls in, which is the first whole-sign house.
    lagna: Option<u8>,
}

impl Chart {
    /// Every ordered pair of distinct bodies.
    fn pairs(&self) -> impl Iterator<Item = (&Reading, &Reading)> {
        self.readings.iter().flat_map(move |from| {
            self.readings
                .iter()
                .filter(move |to| to.body != from.body)
                .map(move |to| (from, to))
        })
    }
}

/// How many fixtures were read, and how many carried a field an aspect
/// would have been recorded under.
struct Corpus {
    charts: Vec<Chart>,
    files: usize,
    aspect_fields: usize,
}

fn corpus(root: &Path) -> Result<Corpus, String> {
    let mut charts = Vec::new();
    let mut files = 0;
    let mut aspect_fields = 0;
    for directory in [CHARTS, VARIANTS] {
        let dir = root.join(directory);
        let mut paths: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
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
        paths.sort();
        for path in &paths {
            let text = std::fs::read_to_string(path)
                .map_err(|err| format!("{}: {err}", path.display()))?;
            let value: Value =
                serde_json::from_str(&text).map_err(|err| format!("{}: {err}", path.display()))?;
            files += 1;
            aspect_fields += aspect_keys(&value);
            if let Some(chart) = read(&value) {
                charts.push(chart);
            }
        }
    }
    if charts.is_empty() {
        return Err(String::from("no fixture carries a body's placement"));
    }
    Ok(Corpus {
        charts,
        files,
        aspect_fields,
    })
}

/// How many keys anywhere in a fixture an aspect could have been recorded
/// under. The claim "the corpus records no aspect" is worth only as much
/// as the search that found none.
fn aspect_keys(value: &Value) -> usize {
    match value {
        Value::Object(fields) => fields
            .iter()
            .map(|(key, nested)| {
                let lower = key.to_ascii_lowercase();
                usize::from(ASPECT_WORDS.iter().any(|word| lower.contains(word)))
                    + aspect_keys(nested)
            })
            .sum(),
        Value::Array(items) => items.iter().map(aspect_keys).sum(),
        _ => 0,
    }
}

/// One fixture's readings, or `None` when it carries no positions.
fn read(fixture: &Value) -> Option<Chart> {
    let bodies = fixture["positions"]["bodies"].as_object()?;
    let mut readings = Vec::new();
    let mut signs = BTreeMap::new();
    for body in GRAHAS {
        let Some(recorded) = bodies.get(body) else {
            continue;
        };
        let longitude = recorded["sidereal_longitude_deg"].as_f64()?;
        signs.insert(body.to_string(), sign_of(longitude));
        let avasthas = &recorded["avasthas"];
        readings.push(Reading {
            body,
            longitude,
            house: u8::try_from(recorded["house"].as_u64().unwrap_or(0)).unwrap_or(0),
            dignity: recorded["dignity"].as_str().unwrap_or_default().to_string(),
            deeptadi: avasthas["deeptadi"].as_str().map(ToString::to_string),
            lajjitadi: avasthas["lajjitadi"]
                .as_array()
                .map(|list| {
                    list.iter()
                        .filter_map(|name| name.as_str().map(ToString::to_string))
                        .collect()
                })
                .unwrap_or_default(),
            near_sign: recorded["near_sign_boundary"].as_bool().unwrap_or(false),
        });
    }
    let lagna = bodies
        .get("LAGNA")
        .and_then(|recorded| recorded["sidereal_longitude_deg"].as_f64())
        .map(sign_of);
    (!readings.is_empty()).then_some(Chart {
        readings,
        signs,
        lagna,
    })
}

// ── the proposed rules ─────────────────────────────────────────────────────

/// How many quarters of a full aspect a graha casts on the sign that is
/// `count` houses from its own, counting inclusively from one.
fn quarters(body: &str, count: u8) -> u8 {
    if SPECIAL
        .iter()
        .any(|(who, houses)| *who == body && houses.contains(&count))
    {
        return FULL;
    }
    QUARTERS
        .iter()
        .find(|(house, _)| *house == count)
        .map_or(0, |(_, given)| *given)
}

/// The quarters one body casts on another, by the signs they stand in.
fn graha_drishti(from: &str, from_sign: u8, to_sign: u8) -> u8 {
    quarters(from, house_count(from_sign, to_sign))
}

/// Whether a sign aspects another under the Jaimini rashi drishti: a
/// movable sign aspects the fixed signs but the one next to it, a fixed
/// sign the movable but the one before it, and a dual sign the other
/// dual signs.
fn rashi_drishti(from: u8, to: u8) -> bool {
    let (Some(here), Some(there)) = (modality(from), modality(to)) else {
        return false;
    };
    let ahead = house_count(from, to);
    match (here, there) {
        (Modality::Chara, Modality::Sthira) => ahead != 2,
        (Modality::Sthira, Modality::Chara) => ahead != 12,
        (Modality::Dwiswabhava, Modality::Dwiswabhava) => from != to,
        _ => false,
    }
}

/// A sign's modality.
fn modality(sign: u8) -> Option<Modality> {
    Rashi::from_id(u16::from(sign)).map(|rashi| rashi.attributes().modality)
}

/// A sign's key.
fn sign_key(sign: u8) -> &'static str {
    Rashi::from_id(u16::from(sign)).map_or("", Rashi::key)
}

/// A graha's nature, which the catalogue states.
fn nature(body: &str) -> Option<Nature> {
    Graha::from_key(body)
        .and_then(|graha| graha.attributes().descriptors.as_ref())
        .map(|descriptors| descriptors.nature)
}

/// Whether a body is a natural malefic, the nodes included.
fn is_malefic(body: &str) -> bool {
    nature(body) == Some(Nature::Malefic)
}

/// Whether a body is a natural benefic.
fn is_benefic(body: &str) -> bool {
    nature(body) == Some(Nature::Benefic)
}

/// The compound friendship a body has with the lord of the sign it
/// stands in, which is what "an enemy's sign" and "a friend's sign" mean.
fn compound(graha: Graha, sign: u8, signs: &BTreeMap<String, u8>) -> &'static str {
    let natural = natural(graha, sign);
    temporary(graha, sign, signs).map_or("NEUTRAL", |temporary| panchadha(natural, temporary))
}

/// Whether one body is the other's natural enemy.
fn is_enemy_of(graha: Graha, other: Graha) -> bool {
    graha.attributes().enemies.contains(&other)
}

/// Whether one body is the other's natural friend.
fn is_friend_of(graha: Graha, other: Graha) -> bool {
    graha.attributes().friends.contains(&other)
}

// ── the page ───────────────────────────────────────────────────────────────

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
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask aspect") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

fn page(root: &Path) -> Result<String, String> {
    let corpus = corpus(root)?;
    let sections = [
        header(&corpus),
        whole_sign(&corpus.charts),
        rashi(),
        systems(&corpus.charts),
        edges(&corpus.charts),
        nodes(&corpus.charts),
        avasthas(&corpus.charts),
        missing(),
        decides(&corpus.charts),
    ];
    Ok(fill(&sections.concat()))
}

// ── 1. what the corpus records ─────────────────────────────────────────────

fn header(corpus: &Corpus) -> String {
    let charts = corpus.charts.len();
    let placements: usize = corpus.charts.iter().map(|chart| chart.readings.len()).sum();
    let with_avasthas = corpus
        .charts
        .iter()
        .flat_map(|chart| chart.readings.iter())
        .filter(|reading| reading.has_avasthas())
        .count();
    format!(
        "# The drishti, measured\n\n\
         Status: `generated` by `cargo xtask aspect` over the conformance\n\
         corpus, 2026-09-07. Do not edit: `check-aspect` regenerates this\n\
         page and fails on any difference. The design written from it is\n\
         [`aspect-and-drishti.md`](aspect-and-drishti.md).\n\n\
         ## 1. What the corpus records, which is nothing\n\n\
         **The corpus records no aspect.** Every key of all {} fixture files\n\
         was searched for a name an aspect could have been recorded under —\n\
         {} — and {} were found. The fixture schema carries none either.\n\
         This is the first Phase 4 module the corpus cannot check directly,\n\
         and saying so plainly is the first result of the pass.\n\n\
         What it can still measure is four things:\n\n\
         1. Each system's **own invariants**, which need no recording: a\n\
            table either is mutual or is not.\n\
         2. The systems **against each other** over the corpus's {} real\n\
            placements on {charts} charts, which is the bhava-chalit pass's\n\
            shape — how far apart two things a reader might both call \"an\n\
            aspect\" actually are.\n\
         3. The avasthas `crates/state` **refused**, retried with a drishti\n\
            in hand. The corpus records {with_avasthas} of those readings,\n\
            so this part has a real answer.\n\
         4. What is still missing, written as the specification the missing\n\
            thing has to meet.\n\n",
        corpus.files,
        ASPECT_WORDS
            .iter()
            .map(|word| format!("`{word}`"))
            .collect::<Vec<_>>()
            .join(", "),
        if corpus.aspect_fields == 0 {
            String::from("none")
        } else {
            count(corpus.aspect_fields)
        },
        count(placements),
    )
}

// ── 2. the graha drishti ───────────────────────────────────────────────────

/// The house the target counts the source as, given the house the source
/// counts the target as: a relation read from the other end.
const fn looking_back(house: u8) -> u8 {
    (13 - house) % 12 + 1
}

/// What §2 finds about pairs of grahas that aspect each other fully.
struct Mutual {
    total: usize,
    beyond_seventh: usize,
    /// The graha pairs that manage it outside the seventh, with how many
    /// sign pairs each does it on.
    pairs: BTreeMap<(&'static str, &'static str), usize>,
}

/// Every ordered pair of signs on which two grahas aspect each other
/// fully, and which pairs of grahas can do it outside the seventh.
fn mutual_full() -> Mutual {
    let mut found = Mutual {
        total: 0,
        beyond_seventh: 0,
        pairs: BTreeMap::new(),
    };
    for body in GRAHAS {
        for other in GRAHAS {
            for from in 0..12_u8 {
                for to in 0..12_u8 {
                    if graha_drishti(body, from, to) != FULL
                        || graha_drishti(other, to, from) != FULL
                    {
                        continue;
                    }
                    found.total += 1;
                    if house_count(from, to) != 7 {
                        found.beyond_seventh += 1;
                        *found.pairs.entry((body, other)).or_default() += 1;
                    }
                }
            }
        }
    }
    found
}

/// How many charts hold two grahas that aspect each other fully outside
/// the seventh, and how many such pairs there are in all.
fn mutual_in_charts(charts: &[Chart]) -> (usize, usize) {
    let mut found = 0;
    let mut in_charts = 0;
    for chart in charts {
        let here = chart
            .pairs()
            .filter(|(from, to)| {
                graha_drishti(from.body, from.sign(), to.sign()) == FULL
                    && graha_drishti(to.body, to.sign(), from.sign()) == FULL
                    && house_count(from.sign(), to.sign()) != 7
            })
            .count();
        found += here;
        in_charts += usize::from(here > 0);
    }
    (found, in_charts)
}

/// The claims §2 makes about the table's own shape.
fn whole_sign_claims(mutual: &Mutual) -> Vec<Claim> {
    let mut aspected_wrong = 0;
    let mut full_wrong = 0;
    let mut self_wrong = 0;
    for body in GRAHAS {
        let seen = (1..=12).filter(|c| quarters(body, *c) > 0).count();
        let full = (1..=12).filter(|c| quarters(body, *c) == FULL).count();
        let expected_full = 1 + SPECIAL
            .iter()
            .find(|(who, _)| *who == body)
            .map_or(0, |(_, houses)| houses.len());
        aspected_wrong += usize::from(seen != 7);
        full_wrong += usize::from(full != expected_full);
        self_wrong += usize::from(quarters(body, 1) != 0);
    }
    // The two configurations that can do it, which the measurement below
    // either confirms are the only ones or contradicts.
    let expected: [(&str, &str); 3] = [
        ("JUPITER", "JUPITER"),
        ("MARS", "SATURN"),
        ("SATURN", "MARS"),
    ];
    let unexpected: usize = mutual
        .pairs
        .iter()
        .filter(|((body, other), _)| !expected.contains(&(body, other)))
        .map(|(_, seen)| *seen)
        .sum();
    vec![
        Claim::counted(
            "every graha aspects seven of the twelve houses",
            aspected_wrong,
            GRAHAS.len(),
        ),
        Claim::counted(
            "a graha's full aspects are the seventh and its own specials",
            full_wrong,
            GRAHAS.len(),
        ),
        Claim::counted(
            "no graha aspects the sign it stands in",
            self_wrong,
            GRAHAS.len(),
        ),
        Claim::counted(
            "two full aspects that meet do so through the seventh",
            mutual.beyond_seventh,
            mutual.total,
        ),
        Claim::counted(
            "the ones that do not are Jupiter with itself, or Mars with Saturn",
            unexpected,
            mutual.beyond_seventh,
        ),
    ]
}

fn whole_sign(charts: &[Chart]) -> String {
    let mutual = mutual_full();
    let (in_corpus, charts_with) = mutual_in_charts(charts);
    let mut out = format!(
        "## 2. The graha drishti, and what holds it together\n\n\
         A graha aspects the sign seven from its own fully, the fourth and\n\
         eighth at three quarters, the fifth and ninth at a half, and the\n\
         third and tenth at a quarter; Mars raises the fourth and eighth to\n\
         full, Jupiter the fifth and ninth, Saturn the third and tenth. The\n\
         corpus records none of this, so what can be measured is the table's\n\
         own consistency — and the fourth claim below is one this pass\n\
         proposed and the measurement **refused**.\n\n{}\n\
         A full aspect is very nearly mutual only through the seventh, and\n\
         the exceptions are exactly two configurations out of the {} ordered\n\
         pairs of signs where two grahas reach each other fully:\n\n",
        table(&whole_sign_claims(&mutual)),
        count(mutual.total),
    );
    for ((body, other), seen) in &mutual.pairs {
        let houses: Vec<String> = (1..=12_u8)
            .filter(|house| {
                *house != 7
                    && quarters(body, *house) == FULL
                    && quarters(other, looking_back(*house)) == FULL
            })
            .map(|house| format!("its {house}th"))
            .collect();
        let _ = writeln!(
            out,
            "- **{body} and {other}** on {seen} sign pairs, the first \
             reaching the second across {}.",
            if houses.is_empty() {
                String::from("no house at all")
            } else {
                houses.join(" and ")
            }
        );
    }
    let _ = write!(
        out,
        "\nMars three signs before Saturn is the one that matters: Saturn\n\
         stands in Mars's fourth, which Mars aspects fully, and Mars stands\n\
         in Saturn's tenth, which Saturn aspects fully. The Jupiter pair is\n\
         arithmetic rather than astrology — a graha is in one sign, so\n\
         Jupiter never faces itself — which is why a module that reasons\n\
         about mutual aspects has to take the pair of **bodies** and not the\n\
         pair of positions. Over the corpus's own charts the Mars and Saturn\n\
         configuration occurs {in_corpus} times on {charts_with} of the {}\n\
         charts, so it is not a curiosity.\n\n\
         | count | Sun | Moon | Mars | Mercury | Jupiter | Venus | Saturn | Rahu | Ketu |\n\
         |---|---|---|---|---|---|---|---|---|---|\n",
        charts.len(),
    );
    for house in 1..=12_u8 {
        let cells: Vec<String> = GRAHAS
            .iter()
            .map(|body| match quarters(body, house) {
                0 => String::from("—"),
                FULL => String::from("**full**"),
                given => format!("{given}/4"),
            })
            .collect();
        let _ = writeln!(out, "| {house} | {} |", cells.join(" | "));
    }
    out.push_str(
        "\nThe nodes take the same table as everything else here. What a\n\
         node aspects beyond the seventh is a settings question and is\n\
         measured in §6.\n\n",
    );
    out
}

// ── 3. the rashi drishti ───────────────────────────────────────────────────

fn rashi() -> String {
    let mut wrong_count = 0;
    let mut not_mutual = 0;
    let mut aspects_itself = 0;
    let mut pairs = 0;
    for from in 0..12_u8 {
        let seen = (0..12_u8).filter(|to| rashi_drishti(from, *to)).count();
        wrong_count += usize::from(seen != 3);
        aspects_itself += usize::from(rashi_drishti(from, from));
        for to in 0..12_u8 {
            if rashi_drishti(from, to) {
                pairs += 1;
                not_mutual += usize::from(!rashi_drishti(to, from));
            }
        }
    }
    // How far the two systems are from being the same relation: over the
    // ordered pairs of signs, where does one see an aspect and not the
    // other? The graha side is read for a body with no special aspect,
    // which is the fairest comparison there is.
    let mut both = 0;
    let mut graha_only = 0;
    let mut rashi_only = 0;
    for from in 0..12_u8 {
        for to in 0..12_u8 {
            if from == to {
                continue;
            }
            let graha = graha_drishti("SUN", from, to) > 0;
            let sign = rashi_drishti(from, to);
            match (graha, sign) {
                (true, true) => both += 1,
                (true, false) => graha_only += 1,
                (false, true) => rashi_only += 1,
                (false, false) => {}
            }
        }
    }
    let claims = [
        Claim::counted("every sign aspects exactly three signs", wrong_count, 12),
        Claim::counted("no sign aspects itself", aspects_itself, 12),
        Claim::counted("a rashi drishti is always mutual", not_mutual, pairs),
        Claim::stated(
            "the two systems are the same relation",
            Verdict::Falsified,
            format!("{both} pairs shared, {graha_only} graha only, {rashi_only} rashi only"),
        ),
    ];
    let mut out = format!(
        "## 3. The rashi drishti is a different relation, not a variant\n\n\
         The Jaimini reading is a relation between **signs** and not between\n\
         bodies: a movable sign aspects the three fixed signs but the one\n\
         next to it, a fixed sign the three movable but the one before it,\n\
         and a dual sign the other three dual signs.\n\n{}\n\
         Mutuality is the difference that matters. A rashi drishti always\n\
         looks back — all {pairs} directed pairs — and a graha drishti\n\
         almost never does, which is why a module cannot quietly offer one\n\
         where a caller asked for the other. Of the 132 ordered pairs of\n\
         distinct signs the two agree on {both}, and each sees {graha_only}\n\
         and {rashi_only} the other does not.\n\n\
         | sign | aspects |\n|---|---|\n",
        table(&claims),
    );
    for from in 0..12_u8 {
        let seen: Vec<&str> = (0..12_u8)
            .filter(|to| rashi_drishti(from, *to))
            .map(sign_key)
            .collect();
        let _ = writeln!(out, "| {} | {} |", sign_key(from), seen.join(", "));
    }
    out.push('\n');
    out
}

// ── 4. the systems over the corpus's own placements ────────────────────────

/// What §4 counts over every ordered pair of bodies in every chart.
struct Systems {
    pairs: usize,
    graha: usize,
    full: usize,
    rashi: usize,
    both: usize,
    neither: usize,
    /// Pairs whose house count by sign and by the recorded bhava
    /// disagree.
    bhava_differs: usize,
    /// Readings whose recorded house is not the whole-sign house.
    not_whole_sign: usize,
    /// Readings a chart's lagna lets that be checked on.
    placed: usize,
}

fn measure_systems(charts: &[Chart]) -> Systems {
    let mut found = Systems {
        pairs: 0,
        graha: 0,
        full: 0,
        rashi: 0,
        both: 0,
        neither: 0,
        bhava_differs: 0,
        not_whole_sign: 0,
        placed: 0,
    };
    for chart in charts {
        if let Some(lagna) = chart.lagna {
            for reading in &chart.readings {
                if reading.house == 0 {
                    continue;
                }
                found.placed += 1;
                found.not_whole_sign +=
                    usize::from(reading.house != house_count(lagna, reading.sign()));
            }
        }
        for (from, to) in chart.pairs() {
            found.pairs += 1;
            let quarters = graha_drishti(from.body, from.sign(), to.sign());
            let sign = rashi_drishti(from.sign(), to.sign());
            found.graha += usize::from(quarters > 0);
            found.full += usize::from(quarters == FULL);
            found.rashi += usize::from(sign);
            found.both += usize::from(quarters > 0 && sign);
            found.neither += usize::from(quarters == 0 && !sign);
            if from.house > 0 && to.house > 0 {
                let by_sign = house_count(from.sign(), to.sign());
                let by_bhava = (to.house + 12 - from.house) % 12 + 1;
                found.bhava_differs += usize::from(by_sign != by_bhava);
            }
        }
    }
    found
}

fn systems(charts: &[Chart]) -> String {
    let found = measure_systems(charts);
    let share = |value: usize| {
        if found.pairs == 0 {
            String::from("—")
        } else {
            #[expect(
                clippy::cast_precision_loss,
                reason = "a share of a count under a million, printed to a tenth"
            )]
            let percent = value as f64 * 100.0 / found.pairs as f64;
            format!("{percent:.1}%")
        }
    };
    let claims = [
        Claim::stated(
            "the two systems pick out the same pairs of bodies",
            Verdict::Falsified,
            format!(
                "{} of {} pairs agree",
                count(found.both + found.neither),
                count(found.pairs)
            ),
        ),
        Claim::counted(
            "the recorded house is the whole-sign house from the lagna",
            found.not_whole_sign,
            found.placed,
        ),
        Claim::counted(
            "so a house counted by sign is the house the corpus records",
            found.bhava_differs,
            found.pairs,
        ),
    ];
    format!(
        "## 4. The two systems over the corpus's own placements\n\n\
         {} ordered pairs of bodies over {} charts — every graha against\n\
         every other, in the positions the corpus actually records.\n\n{}\n\
         | relation | pairs | share |\n|---|---|---|\n\
         | a graha drishti of any strength | {} | {} |\n\
         | of them, full | {} | {} |\n\
         | a rashi drishti | {} | {} |\n\
         | both | {} | {} |\n\
         | neither | {} | {} |\n\n\
         The two systems agree on {} of the {} pairs and each sees\n\
         relations the other does not, which is the bhava-chalit finding in\n\
         another place: they are not variants of one thing, so a value has\n\
         to say which produced it.\n\n\
         The middle claim is worth stating because it decides how this\n\
         corpus may be compared against at all. All {} of the recorded\n\
         placements put a body in the **whole-sign** house counted from the\n\
         lagna's own sign, whatever bhava chalit the fixture's settings\n\
         name. So over this corpus a drishti counted from the sign and one\n\
         counted from the recorded house are the same relation, and a\n\
         harness cannot tell them apart here. They part on any chart whose\n\
         houses come from cusps, and the module therefore counts from the\n\
         **sign** — which is what the tradition's \"the seventh from it\"\n\
         means — and stamps the reading beside the answer.\n\n",
        count(found.pairs),
        charts.len(),
        table(&claims),
        count(found.graha),
        share(found.graha),
        count(found.full),
        share(found.full),
        count(found.rashi),
        share(found.rashi),
        count(found.both),
        share(found.both),
        count(found.neither),
        share(found.neither),
        count(found.both + found.neither),
        count(found.pairs),
        count(found.placed),
    )
}

// ── 5. how near an aspect is decided to a sign edge ────────────────────────

fn edges(charts: &[Chart]) -> String {
    let mut readings = 0;
    let mut flagged = 0;
    let mut nearest = f64::INFINITY;
    let mut flips = 0;
    let mut pairs = 0;
    for chart in charts {
        for reading in &chart.readings {
            readings += 1;
            flagged += usize::from(reading.near_sign);
            nearest = nearest.min(edge(reading.longitude, PER_SIGN));
        }
        // How many aspect relations a body's own flagged nearness puts in
        // question: move it to the neighbouring sign and count what
        // changes.
        for (from, to) in chart.pairs() {
            pairs += 1;
            if !from.near_sign && !to.near_sign {
                continue;
            }
            let now = graha_drishti(from.body, from.sign(), to.sign());
            let moved_from = if from.near_sign {
                neighbour(from.longitude)
            } else {
                from.sign()
            };
            let moved_to = if to.near_sign {
                neighbour(to.longitude)
            } else {
                to.sign()
            };
            flips += usize::from(graha_drishti(from.body, moved_from, moved_to) != now);
        }
    }
    let claims = [Claim::stated(
        "a whole-sign drishti is safe wherever a body stands",
        verdict_of(flips == 0),
        format!(
            "{} of {} pairs change if a flagged body crosses its edge",
            count(flips),
            count(pairs)
        ),
    )];
    format!(
        "## 5. A drishti counted by sign is a step function\n\n\
         A whole-sign relation changes all at once when a body crosses a\n\
         sign boundary, so a body near one has an aspect that is decided by\n\
         the last arcsecond of the ayanamsha. The corpus flags such bodies\n\
         itself: {} of {} placements carry `near_sign_boundary`, and the\n\
         closest any body stands to an edge is {:.4}″.\n\n{}\n\
         Moving every flagged body across its own edge changes {} of the {}\n\
         ordered pairs. That is the same argument `state::boundary` makes\n\
         and the module answers it the same way: it reports the **distance**\n\
         to the edge beside the aspect, and the caller decides whether its\n\
         provider is accurate enough for the answer to stand. A drishti is a\n\
         step function, and a value that hides which step it is on is\n\
         hiding the only thing that could be wrong about it.\n\n",
        count(flagged),
        count(readings),
        if nearest.is_finite() {
            nearest * 3600.0
        } else {
            0.0
        },
        table(&claims),
        count(flips),
        count(pairs),
    )
}

/// The sign a body would stand in if it crossed the edge it is nearest.
fn neighbour(longitude: f64) -> u8 {
    let sign = sign_of(longitude);
    let inside = longitude.rem_euclid(PER_SIGN);
    if inside < PER_SIGN - inside {
        (sign + 11) % 12
    } else {
        (sign + 1) % 12
    }
}

// ── 6. the nodes ───────────────────────────────────────────────────────────

fn nodes(charts: &[Chart]) -> String {
    let mut rows = Vec::new();
    for (name, extra) in NODE_READINGS {
        let mut cast = 0;
        for chart in charts {
            for (from, to) in chart.pairs() {
                if !matches!(from.body, "RAHU" | "KETU") {
                    continue;
                }
                let house = house_count(from.sign(), to.sign());
                if house == 7 || extra.contains(&house) {
                    cast += 1;
                }
            }
        }
        rows.push((name, extra, cast));
    }
    let base = rows.first().map_or(0, |row| row.2);
    let mut out = String::from(
        "## 6. What a node aspects is a choice, and the corpus does not make it\n\n\
         `aspect.node_aspects` offers three readings and the root takes the\n\
         first. Nothing in the corpus prefers any of them — there is no\n\
         recorded aspect to compare — so this is a measurement of what the\n\
         choice **costs**, not of which is right.\n\n\
         | reading | beyond the seventh | aspects cast by a node |\n|---|---|---|\n",
    );
    for (name, extra, cast) in &rows {
        let beyond = if extra.is_empty() {
            String::from("nothing")
        } else {
            extra
                .iter()
                .map(|house| format!("the {}", ordinal(*house)))
                .collect::<Vec<_>>()
                .join(" and ")
        };
        let change = if *cast == base {
            String::new()
        } else {
            #[expect(
                clippy::cast_precision_loss,
                reason = "a ratio of counts under a million"
            )]
            let times = *cast as f64 / f64::from(u32::try_from(base.max(1)).unwrap_or(1));
            format!(" ({times:.1}× the first)")
        };
        let _ = writeln!(out, "| `{name}` | {beyond} | {}{change} |", count(*cast));
    }
    let widest = rows.iter().map(|row| row.2).max().unwrap_or(base);
    #[expect(
        clippy::cast_precision_loss,
        reason = "a ratio of counts under a million"
    )]
    let times = widest as f64 / f64::from(u32::try_from(base.max(1)).unwrap_or(1));
    let _ = write!(
        out,
        "\nThe two readings that give a node an aspect beyond the seventh\n\
         each add two houses, so they reach the same number of bodies and\n\
         differ in **which**: {times:.1} times what `NONE` reaches, and not\n\
         the same set. That is a large enough difference that a value has\n\
         to carry which reading made it, which is what the provenance stamp\n\
         is for; it is not a reason for the SDK to pick one on the corpus's\n\
         behalf, because the corpus is silent. The default stays the root's\n\
         `NONE` — the reading that claims least.\n\n",
    );
    out
}

/// A house number as the tradition says it.
fn ordinal(house: u8) -> String {
    let suffix = match (house % 10, house % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{house}{suffix}")
}

// ── 7. the avasthas, retried ───────────────────────────────────────────────

/// One proposed rule for an avastha, and how it did.
struct Attempt {
    rule: &'static str,
    wrong: usize,
    missed: usize,
    spurious: usize,
}

/// Whether any body satisfying a test aspects this one.
fn aspected_by(chart: &Chart, target: &Reading, test: impl Fn(&Reading) -> bool) -> bool {
    chart.readings.iter().any(|other| {
        other.body != target.body
            && test(other)
            && graha_drishti(other.body, other.sign(), target.sign()) > 0
    })
}

/// Whether any body satisfying a test shares this one's sign.
fn joined_by(chart: &Chart, target: &Reading, test: impl Fn(&Reading) -> bool) -> bool {
    chart
        .readings
        .iter()
        .any(|other| other.body != target.body && test(other) && other.sign() == target.sign())
}

/// Measures one proposed rule for one lajjitadi over every reading that
/// records the family.
fn attempt(
    charts: &[Chart],
    state: &str,
    rule: &'static str,
    holds: impl Fn(&Chart, &Reading) -> bool,
) -> Attempt {
    let mut missed = 0;
    let mut spurious = 0;
    for chart in charts {
        for reading in &chart.readings {
            if !reading.has_avasthas() {
                continue;
            }
            let recorded = reading.lajjitadi.iter().any(|name| name == state);
            match (holds(chart, reading), recorded) {
                (false, true) => missed += 1,
                (true, false) => spurious += 1,
                _ => {}
            }
        }
    }
    Attempt {
        rule,
        wrong: missed + spurious,
        missed,
        spurious,
    }
}

/// One family: the state's key, how often the corpus records it, and the
/// rules tried for it.
type Family = (&'static str, usize, Vec<Attempt>);

/// How many readings the corpus records a lajjitadi on.
fn recorded(charts: &[Chart], state: &str) -> usize {
    charts
        .iter()
        .flat_map(|chart| chart.readings.iter())
        .filter(|reading| reading.has_avasthas())
        .filter(|reading| reading.lajjitadi.iter().any(|name| name == state))
        .count()
}

/// Whether a body stands in a sign whose lord it is at odds with.
fn in_enemys_sign(chart: &Chart, reading: &Reading) -> bool {
    reading.graha().is_some_and(|graha| {
        matches!(
            compound(graha, reading.sign(), &chart.signs),
            "ENEMY" | "GREAT_ENEMY"
        )
    })
}

/// Whether a body stands in a sign whose lord it is at ease with.
fn in_friends_sign(chart: &Chart, reading: &Reading) -> bool {
    reading.graha().is_some_and(|graha| {
        matches!(
            compound(graha, reading.sign(), &chart.signs),
            "FRIEND" | "GREAT_FRIEND"
        )
    })
}

/// Whether a body stands in one of the watery signs.
fn in_watery_sign(reading: &Reading) -> bool {
    WATERY.contains(&sign_key(reading.sign()))
}

/// The rules tried for **kshudha**, the hungry: an enemy's sign, or an
/// enemy's company, or an enemy's aspect, or Saturn's company.
fn kshudha(charts: &[Chart]) -> Family {
    let enemy = |graha: Graha| {
        move |other: &Reading| other.graha().is_some_and(|who| is_enemy_of(graha, who))
    };
    (
        "KSHUDHA",
        recorded(charts, "KSHUDHA"),
        vec![
            attempt(charts, "KSHUDHA", "in an enemy's sign", in_enemys_sign),
            attempt(
                charts,
                "KSHUDHA",
                "in an enemy's sign, or joined by an enemy",
                |chart, reading| {
                    in_enemys_sign(chart, reading)
                        || reading
                            .graha()
                            .is_some_and(|graha| joined_by(chart, reading, enemy(graha)))
                },
            ),
            attempt(
                charts,
                "KSHUDHA",
                "the same, or **aspected by** an enemy",
                |chart, reading| {
                    in_enemys_sign(chart, reading)
                        || reading.graha().is_some_and(|graha| {
                            joined_by(chart, reading, enemy(graha))
                                || aspected_by(chart, reading, enemy(graha))
                        })
                },
            ),
            attempt(
                charts,
                "KSHUDHA",
                "the same, and with Saturn as well",
                |chart, reading| {
                    in_enemys_sign(chart, reading)
                        || joined_by(chart, reading, |other| other.body == "SATURN")
                        || reading.graha().is_some_and(|graha| {
                            joined_by(chart, reading, enemy(graha))
                                || aspected_by(chart, reading, enemy(graha))
                        })
                },
            ),
        ],
    )
}

/// The rules tried for **trishita**, the thirsty: a watery sign under a
/// malefic's gaze, with and without a benefic's.
fn trishita(charts: &[Chart]) -> Family {
    (
        "TRISHITA",
        recorded(charts, "TRISHITA"),
        vec![
            attempt(charts, "TRISHITA", "in a watery sign", |_, reading| {
                in_watery_sign(reading)
            }),
            attempt(
                charts,
                "TRISHITA",
                "in a watery sign, **aspected by** a malefic",
                |chart, reading| {
                    in_watery_sign(reading)
                        && aspected_by(chart, reading, |other| is_malefic(other.body))
                },
            ),
            attempt(
                charts,
                "TRISHITA",
                "the same, and no benefic aspecting it",
                |chart, reading| {
                    in_watery_sign(reading)
                        && aspected_by(chart, reading, |other| is_malefic(other.body))
                        && !aspected_by(chart, reading, |other| is_benefic(other.body))
                },
            ),
        ],
    )
}

/// The rules tried for **mudita**, the delighted: a friend's sign, a
/// friend's company, or Jupiter's aspect.
fn mudita(charts: &[Chart]) -> Family {
    let friend = |graha: Graha| {
        move |other: &Reading| other.graha().is_some_and(|who| is_friend_of(graha, who))
    };
    (
        "MUDITA",
        recorded(charts, "MUDITA"),
        vec![
            attempt(charts, "MUDITA", "in a friend's sign", in_friends_sign),
            attempt(
                charts,
                "MUDITA",
                "in a friend's sign, or joined by a friend",
                |chart, reading| {
                    in_friends_sign(chart, reading)
                        || reading
                            .graha()
                            .is_some_and(|graha| joined_by(chart, reading, friend(graha)))
                },
            ),
            attempt(
                charts,
                "MUDITA",
                "the same, or **aspected by** Jupiter",
                |chart, reading| {
                    in_friends_sign(chart, reading)
                        || aspected_by(chart, reading, |other| other.body == "JUPITER")
                        || reading
                            .graha()
                            .is_some_and(|graha| joined_by(chart, reading, friend(graha)))
                },
            ),
        ],
    )
}

/// The three families, in the order a reader would try them.
fn lajjitadi_attempts(charts: &[Chart]) -> Vec<Family> {
    vec![kshudha(charts), trishita(charts), mudita(charts)]
}

/// The dignities that decide a deeptadi outright, which the state pass
/// settled (`state-tables-measured.md` §7). Everything else splits six
/// ways on something it could not find.
const DECIDED_BY_DIGNITY: [&str; 4] = ["EXALTED", "OWN_SIGN", "MOOLTRIKONA", "GREAT_FRIEND"];

/// The state the undecided readings mostly carry, and the one a benefic
/// aspect would be expected to give.
const CALM: &str = "SHANTA";

/// Measures one proposed rule for the deeptadi over the readings the
/// dignity does not decide.
fn deeptadi_attempt(
    charts: &[Chart],
    rule: &'static str,
    holds: impl Fn(&Chart, &Reading) -> bool,
) -> Attempt {
    let mut missed = 0;
    let mut spurious = 0;
    for chart in charts {
        for reading in &chart.readings {
            let Some(recorded) = reading.deeptadi.as_deref() else {
                continue;
            };
            if DECIDED_BY_DIGNITY.contains(&reading.dignity.as_str()) {
                continue;
            }
            match (holds(chart, reading), recorded == CALM) {
                (false, true) => missed += 1,
                (true, false) => spurious += 1,
                _ => {}
            }
        }
    }
    Attempt {
        rule,
        wrong: missed + spurious,
        missed,
        spurious,
    }
}

/// The rules tried for the deeptadi, whose lower six states the state
/// pass could not separate.
fn deeptadi_attempts(charts: &[Chart]) -> (usize, usize, Vec<Attempt>) {
    let undecided = charts
        .iter()
        .flat_map(|chart| chart.readings.iter())
        .filter(|reading| {
            reading.deeptadi.is_some() && !DECIDED_BY_DIGNITY.contains(&reading.dignity.as_str())
        })
        .count();
    let calm = charts
        .iter()
        .flat_map(|chart| chart.readings.iter())
        .filter(|reading| {
            reading.deeptadi.as_deref() == Some(CALM)
                && !DECIDED_BY_DIGNITY.contains(&reading.dignity.as_str())
        })
        .count();
    let attempts = vec![
        deeptadi_attempt(charts, "aspected by a benefic", |chart, reading| {
            aspected_by(chart, reading, |other| is_benefic(other.body))
        }),
        deeptadi_attempt(
            charts,
            "aspected by a benefic and no malefic",
            |chart, reading| {
                aspected_by(chart, reading, |other| is_benefic(other.body))
                    && !aspected_by(chart, reading, |other| is_malefic(other.body))
            },
        ),
        deeptadi_attempt(
            charts,
            "joined or aspected by a benefic",
            |chart, reading| {
                aspected_by(chart, reading, |other| is_benefic(other.body))
                    || joined_by(chart, reading, |other| is_benefic(other.body))
            },
        ),
        deeptadi_attempt(charts, "not aspected by a malefic", |chart, reading| {
            !aspected_by(chart, reading, |other| is_malefic(other.body))
        }),
    ];
    (undecided, calm, attempts)
}

fn avasthas(charts: &[Chart]) -> String {
    let families = lajjitadi_attempts(charts);
    let readings = charts
        .iter()
        .flat_map(|chart| chart.readings.iter())
        .filter(|reading| reading.has_avasthas())
        .count();
    let best: Vec<(&str, usize, usize)> = families
        .iter()
        .map(|(state, recorded, attempts)| {
            let fewest = attempts.iter().map(|a| a.wrong).min().unwrap_or(readings);
            (*state, *recorded, fewest)
        })
        .collect();
    let settled = best.iter().filter(|(_, _, wrong)| *wrong == 0).count();
    let mut out = format!(
        "## 7. The avasthas the state pass refused, retried\n\n\
         `crates/state` ships three of the six lajjitadi as **undecided**\n\
         because their classical definitions read \"or aspected by\" and no\n\
         aspect model existed (`state-tables-measured.md` §7). One now does,\n\
         so the rules are proposed again here, against the {readings}\n\
         readings the corpus records the family for. Each rule is scored\n\
         both ways: a reading the rule misses and a reading it invents are\n\
         different mistakes.\n\n",
    );
    for (state, recorded, attempts) in &families {
        let _ = writeln!(
            out,
            "**{state}**, recorded on {recorded} of {readings} readings.\n"
        );
        out.push_str("| proposed rule | verdict | missed | invented |\n|---|---|---|---|\n");
        for attempt in attempts {
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} |",
                attempt.rule,
                verdict_of(attempt.wrong == 0).mark(),
                attempt.missed,
                attempt.spurious
            );
        }
        out.push('\n');
    }
    out.push_str(&lajjitadi_verdict(&families, settled));
    out.push_str("\n\n");
    out.push_str(&deeptadi_retried(charts));
    out
}

/// What §7 concludes about the three lajjitadi.
fn lajjitadi_verdict(families: &[Family], settled: usize) -> String {
    // The finding that matters is not which rule is exact — none is —
    // but that the first rule of each family misses nothing at all.
    let all_necessary = families
        .iter()
        .all(|(_, _, attempts)| attempts.first().is_some_and(|first| first.missed == 0));
    let claims: Vec<Claim> = families
        .iter()
        .map(|(state, recorded, attempts)| {
            Claim::counted(
                format!("every recorded {state} satisfies the tradition's plainest condition"),
                attempts.first().map_or(*recorded, |first| first.missed),
                *recorded,
            )
        })
        .collect();
    if settled == families.len() {
        format!(
            "All {} are settled: a drishti closes what the state pass left\n\
             open, and `crates/state` can decide the whole lajjitadi family\n\
             once it can ask this module.",
            families.len()
        )
    } else if all_necessary {
        format!(
            "**No rule is exact, and every rule's misses are zero.** That \
             pattern is the result. Each family's plainest condition — an \
             enemy's sign, a watery sign, a friend's sign — holds on every \
             one of the readings the engine records the state for, and on \
             a good many it does not; and **adding the aspect clause only \
             widens the gap**, because an aspect reaches more bodies than a \
             sign does. So the tradition's condition is *necessary* and the \
             engine applies something narrower, and whatever that is, it is \
             not a drishti: if it were, the aspect rules would have moved \
             the count towards the engine's rather than away from it.\n\n{}\n\
             Two things follow, and they are worth more than a fitted rule \
             would have been. **`crates/state` can say `no` where it now \
             says nothing**: a body outside the necessary condition \
             certainly does not hold the state, and that costs no aspect \
             model at all — a sign and a dignity decide it, which the \
             module already has. And what stays undecided stays undecided \
             for a measured reason: {} readings satisfy the condition and \
             only {} of them are recorded, so the engine's own extra \
             restriction is not in anything it records beside them.",
            table(&claims),
            count(
                families
                    .iter()
                    .map(|(_, recorded, attempts)| recorded
                        + attempts.iter().next().map_or(0, |first| first.spurious))
                    .sum::<usize>()
            ),
            count(
                families
                    .iter()
                    .map(|(_, recorded, _)| *recorded)
                    .sum::<usize>()
            ),
        )
    } else {
        format!(
            "{settled} of the {} are settled and the rest are not. A drishti \
             was necessary and is not sufficient: the states it closes are \
             closed, and the others still turn on something neither the \
             corpus nor this module records.",
            families.len()
        )
    }
}

/// The other half of §7: the deeptadi's lower states, tried the same way.
fn deeptadi_retried(charts: &[Chart]) -> String {
    let mut out = String::new();
    let (undecided, calm, attempts) = deeptadi_attempts(charts);
    let _ = write!(
        out,
        "The deeptadi's lower states are the other half of what the state \n\
         pass refused, and they are tried the same way. {undecided} readings \n\
         have a dignity that does not decide one, {calm} of them are {CALM}, \n\
         and the question is whether a benefic's aspect is what makes the \n\
         difference.\n\n\
         | proposed rule | verdict | missed | invented |\n|---|---|---|---|\n",
    );
    for attempt in &attempts {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} |",
            attempt.rule,
            verdict_of(attempt.wrong == 0).mark(),
            attempt.missed,
            attempt.spurious
        );
    }
    let fewest = attempts.iter().map(|a| a.wrong).min().unwrap_or(undecided);
    let _ = write!(
        out,
        "\nThe best of them is wrong on {fewest} of the {undecided}. A \n\
         drishti does not separate the deeptadi either, and the same \n\
         conclusion follows: what the engine records under these names \n\
         does not follow from what it records beside them, and the SDK \n\
         goes on reporting nothing where it cannot decide.\n\n",
    );
    out
}

// ── 8. what is missing ─────────────────────────────────────────────────────

fn missing() -> String {
    let mut out = String::from(
        "## 8. The sphuta drishti, and why it does not ship\n\n\
         The degree-based drishti — a value in virupas rather than a\n\
         relation between signs — is what Drik Bala weighs (Phase 5,\n\
         `strength-schemes.md`) and what `aspect.drishti_table` is named\n\
         for. **No source in this project gives its construction.** The\n\
         research page records that one exists (\"the BPHS formula with the\n\
         Mars, Jupiter and Saturn special cases\",\n\
         `01-research/feature-universe/01-vedic-parashari-core.md` §G) and\n\
         nowhere writes it down; the corpus records no drishti value to fit\n\
         one to; and a piecewise formula recalled rather than read is\n\
         exactly the kind of thing this pass exists to keep out of the\n\
         code.\n\n\
         So the module ships no sphuta drishti and refuses the table by\n\
         name, and the question is registered as a crux. What the pass can\n\
         do is publish the **specification** any construction has to meet,\n\
         so that reading a source later is a matter of checking it against\n\
         this table rather than trusting it:\n\n\
         | at the house | the value must be | because |\n|---|---|---|\n",
    );
    for (house, given) in QUARTERS {
        let virupas = u16::from(given) * 15;
        let _ = writeln!(
            out,
            "| {house} | {virupas} virupas | {} of a full aspect |",
            match given {
                FULL => String::from("all"),
                given => format!("{given}/4"),
            }
        );
    }
    let _ = write!(
        out,
        "\nand, for each of the three special grahas, {} virupas at its own\n\
         two houses instead of the value above: Mars at the fourth and\n\
         eighth, Jupiter at the fifth and ninth, Saturn at the third and\n\
         tenth. A construction that does not reproduce all {} of those\n\
         values is not the one the whole-sign table is a summary of, and\n\
         that is a test a future source has to pass before it ships.\n\n",
        u16::from(FULL) * 15,
        QUARTERS.len() + SPECIAL.len() * 2,
    );
    out
}

// ── 9. what the pass decides ───────────────────────────────────────────────

fn decides(charts: &[Chart]) -> String {
    let found = measure_systems(charts);
    format!(
        "## 9. What this pass decides\n\n\
         - **The graha drishti and the rashi drishti ship**, as tables whose\n\
           invariants hold and whose sources are the project's own research\n\
           page. They are different relations and the module keeps them\n\
           apart: over the corpus's {} pairs they agree on {} and disagree\n\
           on the rest.\n\
         - **A drishti is counted from the sign.** Over this corpus that is\n\
           indistinguishable from counting by house, because all {} recorded\n\
           placements are whole-sign; on a chart whose houses come from\n\
           cusps it is not, so the reading is part of the answer and is\n\
           stamped with it.\n\
         - **A drishti carries the distance to the sign edge that decides\n\
           it**, for the reason `state::boundary` carries one: the module\n\
           cannot know how accurate the caller's provider is.\n\
         - **The node's aspect stays the root's `NONE`.** Nothing in the\n\
           corpus prefers a reading, and the knob already exists to say so.\n\
         - **The sphuta drishti does not ship.** Its construction has no\n\
           source here; §8 is the specification it will have to meet.\n\
         - **The three lajjitadi gain a necessary condition and stay\n\
           undecided beyond it.** §7 tried every rule the tradition states,\n\
           with a real drishti, and none is exact — but none misses a\n\
           single recorded reading either, so `crates/state` can answer\n\
           `no` with certainty where the condition fails and needs no\n\
           aspect model to do it. Adding the aspect clause widens the gap\n\
           rather than closing it, which is evidence the recording engine\n\
           does not compute these from a drishti at all.\n",
        count(found.pairs),
        count(found.both + found.neither),
        count(found.placed),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        FULL, SPECIAL, aspect_keys, graha_drishti, is_benefic, is_malefic, looking_back, modality,
        neighbour, quarters, rashi_drishti, sign_key,
    };
    use serde_json::json;
    use teistro_core::catalogue::Modality;

    #[test]
    fn a_graha_aspects_the_seventh_fully_and_the_first_not_at_all() {
        assert_eq!(quarters("SUN", 7), FULL);
        assert_eq!(quarters("SUN", 1), 0, "not the sign it stands in");
        assert_eq!(quarters("SUN", 4), 3);
        assert_eq!(quarters("SUN", 5), 2);
        assert_eq!(quarters("SUN", 3), 1);
        assert_eq!(quarters("SUN", 2), 0);
        assert_eq!(quarters("SUN", 6), 0);
    }

    #[test]
    fn the_three_special_grahas_raise_their_own_houses_to_full() {
        assert_eq!(quarters("MARS", 4), FULL, "and not three quarters");
        assert_eq!(quarters("MARS", 8), FULL);
        assert_eq!(quarters("MARS", 5), 2, "the rest of the table is unchanged");
        assert_eq!(quarters("JUPITER", 5), FULL);
        assert_eq!(quarters("JUPITER", 9), FULL);
        assert_eq!(quarters("SATURN", 3), FULL);
        assert_eq!(quarters("SATURN", 10), FULL);
        for (body, houses) in SPECIAL {
            for house in houses {
                assert_eq!(quarters(body, house), FULL, "{body} {house}");
            }
        }
    }

    #[test]
    fn a_relation_read_from_the_other_end_is_its_complement() {
        assert_eq!(looking_back(7), 7, "the seventh looks back at itself");
        assert_eq!(looking_back(4), 10, "and the fourth at the tenth");
        assert_eq!(looking_back(10), 4);
        assert_eq!(looking_back(5), 9);
        assert_eq!(looking_back(9), 5);
        assert_eq!(looking_back(1), 1, "a sign is its own first either way");
        for house in 1..=12_u8 {
            assert_eq!(looking_back(looking_back(house)), house, "{house}");
        }
    }

    #[test]
    fn a_drishti_is_counted_from_the_sign_the_body_stands_in() {
        // Aries to Libra is the seventh either way.
        assert_eq!(graha_drishti("SUN", 0, 6), FULL);
        assert_eq!(graha_drishti("SUN", 6, 0), FULL);
        // Mars in Aries reaches Cancer, its fourth, fully.
        assert_eq!(graha_drishti("MARS", 0, 3), FULL);
        // And the Sun in Aries only three quarters of the way.
        assert_eq!(graha_drishti("SUN", 0, 3), 3);
        // Over the end of the zodiac.
        assert_eq!(graha_drishti("SUN", 11, 5), FULL, "Pisces to Virgo");
    }

    #[test]
    fn the_rashi_drishti_is_mutual_and_three_wide() {
        for from in 0..12_u8 {
            let seen: Vec<u8> = (0..12_u8).filter(|to| rashi_drishti(from, *to)).collect();
            assert_eq!(seen.len(), 3, "{} aspects three", sign_key(from));
            for to in seen {
                assert!(rashi_drishti(to, from), "{from} and {to} look both ways");
            }
            assert!(!rashi_drishti(from, from));
        }
    }

    #[test]
    fn the_rashi_drishti_is_the_classical_one() {
        // Aries is movable and aspects the fixed signs but Taurus.
        assert!(rashi_drishti(0, 4), "Aries to Leo");
        assert!(rashi_drishti(0, 7), "Aries to Scorpio");
        assert!(rashi_drishti(0, 10), "Aries to Aquarius");
        assert!(!rashi_drishti(0, 1), "but not the sign next to it");
        // Taurus is fixed and aspects the movable but Aries.
        assert!(rashi_drishti(1, 3) && rashi_drishti(1, 6) && rashi_drishti(1, 9));
        assert!(!rashi_drishti(1, 0));
        // Gemini is dual and aspects the other dual signs.
        assert!(rashi_drishti(2, 5) && rashi_drishti(2, 8) && rashi_drishti(2, 11));
        assert!(!rashi_drishti(2, 4), "and no fixed sign");
        assert_eq!(modality(0), Some(Modality::Chara));
        assert_eq!(modality(1), Some(Modality::Sthira));
        assert_eq!(modality(2), Some(Modality::Dwiswabhava));
    }

    #[test]
    fn the_natures_are_the_catalogue_s() {
        assert!(is_benefic("JUPITER") && is_benefic("VENUS") && is_benefic("MOON"));
        assert!(is_malefic("SUN") && is_malefic("SATURN") && is_malefic("MARS"));
        assert!(is_malefic("RAHU") && is_malefic("KETU"));
        assert!(
            !is_benefic("MERCURY") && !is_malefic("MERCURY"),
            "Mercury is neutral in the catalogue"
        );
    }

    #[test]
    fn the_neighbour_is_the_sign_on_the_near_side() {
        assert_eq!(neighbour(0.5), 11, "just inside Aries looks back");
        assert_eq!(neighbour(29.5), 1, "just before Taurus looks forward");
        assert_eq!(neighbour(30.5), 0);
        assert_eq!(neighbour(15.0), 1, "the far middle rounds forward");
    }

    #[test]
    fn an_aspect_field_would_be_found_if_one_existed() {
        assert_eq!(aspect_keys(&json!({"positions": {"bodies": {}}})), 0);
        assert_eq!(aspect_keys(&json!({"aspects": []})), 1);
        assert_eq!(
            aspect_keys(&json!({"a": {"graha_drishti": 1, "b": [{"drishti": 2}]}})),
            2,
            "nested and inside an array"
        );
        assert_eq!(aspect_keys(&json!({"ASPECT": 1})), 1, "whatever the case");
    }
}
