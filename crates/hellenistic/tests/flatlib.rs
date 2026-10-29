//! The traditional doctrine against the corpus's `flatlib/traditional`:
//! flatlib's three term tables, its essential dignities and scores, its 21
//! Arabic parts and its sect. Each chart is recorded beside the tool's own
//! places, so the SDK's kernels are fed those places and the comparison
//! measures the doctrine and never the ephemeris
//! (`03-design/western-hellenistic-oracle.md`, C46).
//!
//! flatlib is rank 3 (MIT), a witness and never a correction: where it reads
//! a rule otherwise than the texts the SDK cites, the miss is counted against
//! the crux that names the difference, and any other miss fails.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "tests fail by panicking, index JSON by key and print what they skip"
)]

use std::path::{Path, PathBuf};

#[path = "../../core/tests/support/conformance.rs"]
mod conformance;

use conformance::{Tally, carried};

use serde_json::Value;
use teistro_core::catalogue::{Graha, Rashi};
use teistro_hellenistic::{
    ChartSky, DignityRules, Lot, LotArc, LotFormula, LotPoint, LotRequest, LotSky, Scores, Sect,
    SectRule, Terms, Triplicities, essential_dignity, face_lord, lot_place, lots,
};

const SECTION: &str = "flatlib/traditional";

/// Agreement on a longitude the SDK computes by the same arithmetic from the
/// same recorded places: rounding only.
const SAME_DEG: f64 = 1e-9;

/// The tool's three term tables and the SDK's term system each is.
const TABLES: [(&str, Terms); 3] = [
    ("EGYPTIAN", Terms::Egyptian),
    ("TETRABIBLOS", Terms::PtolemaicAshmand),
    ("LILLY", Terms::PtolemaicLilly),
];

/// The tool's parts that are Valens's lots under the same arcs by day
/// (`hellenistic-lots.md`).
const NAMED: [(&str, Lot); 5] = [
    ("Pars Fortuna", Lot::Fortune),
    ("Pars Spirit", Lot::Daimon),
    ("Pars Diseases", Lot::Crisis),
    ("Pars Father", Lot::Father),
    ("Pars Brothers", Lot::Brothers),
];

/// The cruxes a miss is counted against.
const PRINTING: &str =
    "Ptolemy's terms as another witness prints them (western-hellenistic-oracle.md, C208)";
const PARTICIPATING: &str =
    "the participating triplicity lord (western-hellenistic-oracle.md, C790)";
const NIGHT_ARC: &str =
    "a lot's night arc other than Valens's (western-hellenistic-oracle.md, C791)";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/flatlib")
}

fn read(path: &Path) -> Value {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("{}: {err}; the corpus lists {SECTION}", path.display()));
    serde_json::from_str(&text).unwrap()
}

/// Every file the section's manifest lists, each read; a listed file that
/// is missing fails rather than reading as a skip.
fn files() -> (Value, Vec<(String, Value)>) {
    let manifest = read(&root().join("traditional/manifest.json"));
    let files = manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            let file = entry["file"].as_str().unwrap();
            (file.to_owned(), read(&root().join(file)))
        })
        .collect();
    (manifest, files)
}

fn graha(key: &str) -> Graha {
    Graha::from_key(key).unwrap_or_else(|| panic!("{key} is not a graha"))
}

fn number(value: &Value) -> f64 {
    value.as_f64().unwrap()
}

/// The arc between two longitudes, either way round.
fn apart(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d)
}

/// The comparisons of the section: each agrees, is explained by a crux, or
/// is kept, so the test fails naming every unexplained miss at once.
struct Score {
    tally: Tally,
    unexplained: Vec<String>,
}

impl Score {
    /// One comparison. `explained` names the crux a miss is counted
    /// against, when the miss is the one that crux describes.
    fn check(&mut self, agrees: bool, explained: Option<&'static str>, what: impl Fn() -> String) {
        match (agrees, explained) {
            (true, _) => self.tally.agreed(1),
            (false, Some(crux)) => self.tally.explained(crux, 1),
            (false, None) => self.unexplained.push(what()),
        }
    }
}

/// The sky the SDK's lots read, from the tool's recorded places and the
/// exporter's altitudes of the tool's Sun and Moon.
fn sky(file: &Value) -> LotSky {
    let sky = &file["sky"];
    let at = |key: &str| number(&sky["planets"][key]["longitude_deg"]);
    let sun_altitude_deg = number(&file["exporter"]["sun_altitude_deg"]);
    LotSky {
        chart: ChartSky {
            saturn_deg: at("SATURN"),
            jupiter_deg: at("JUPITER"),
            mars_deg: at("MARS"),
            sun_deg: at("SUN"),
            venus_deg: at("VENUS"),
            mercury_deg: at("MERCURY"),
            moon_deg: at("MOON"),
            sun_altitude_deg,
            moon_altitude_deg: number(&file["exporter"]["moon_altitude_deg"]),
            daylight: sun_altitude_deg > 0.0,
        },
        ascendant_deg: number(&sky["ascendant_deg"]),
        midheaven_deg: number(&sky["midheaven_deg"]),
    }
}

/// One point of a part's formula as the tool writes it, in the SDK's terms:
/// a planet, the ascendant, Fortune or Spirit (Valens's Daimon), a house's
/// cusp, or `$R` and a house for the lord of the sign on its cusp.
fn point(name: &str, cusps: &[f64]) -> LotPoint {
    let house = |text: &str| -> f64 {
        let k: usize = text.strip_prefix("House").unwrap().parse().unwrap();
        cusps[k - 1]
    };
    match name {
        "Asc" => LotPoint::Ascendant,
        "Pars Fortuna" => LotPoint::Lot(Lot::Fortune),
        "Pars Spirit" => LotPoint::Lot(Lot::Daimon),
        _ if name.starts_with("$R") => {
            LotPoint::Planet(Rashi::of_longitude(house(&name[2..])).attributes().lord)
        }
        _ if name.starts_with("House") => LotPoint::Degrees(house(name)),
        _ => LotPoint::Planet(graha(&name.to_uppercase())),
    }
}

fn arc(points: &Value, cusps: &[f64]) -> LotArc {
    let name = |k: usize| points[k].as_str().unwrap();
    LotArc::new(
        point(name(0), cusps),
        point(name(1), cusps),
        point(name(2), cusps),
    )
}

/// The tool's three term tables against the SDK's, term by term: each
/// term's lord and where it ends. Returns the signs where the two part.
fn score_tables(manifest: &Value, score: &mut Score) -> Vec<(&'static str, Rashi)> {
    let mut parted = Vec::new();
    for (table, terms) in TABLES {
        for sign in Rashi::ALL {
            let ours = terms.table(Sect::Day).of(sign);
            let tool = manifest["term_tables"][table][sign.key()]
                .as_array()
                .unwrap();
            for (term, cell) in ours.iter().zip(tool) {
                let same = term.lord.key() == cell[0].as_str().unwrap()
                    && u64::from(term.end) == cell[1].as_u64().unwrap();
                if !same && !parted.contains(&(table, sign)) {
                    parted.push((table, sign));
                }
                // The Egyptian table is one printing; Ptolemy's own terms
                // are printed differently by each witness (C208).
                let explained = (table != "EGYPTIAN").then_some(PRINTING);
                score.check(same, explained, || {
                    format!("{table} {sign:?}: the tool's term {cell} against {term:?}")
                });
            }
        }
    }
    parted
}

/// One planet's dignities: its sign's ruler, exaltation, detriment and
/// fall, its face, its sign's triplicity lords, and under each table its
/// term's lord, its score and whether it is peregrine.
fn score_dignities(
    name: &str,
    key: &str,
    row: &Value,
    longitude: f64,
    parted: &[(&'static str, Rashi)],
    score: &mut Score,
) {
    let planet = graha(key);
    let sign = Rashi::of_longitude(longitude);
    let ours = essential_dignity(planet, longitude, Sect::Day, &DignityRules::LILLY).unwrap();
    let is = |field: &str| row[field].as_str().unwrap() == key;
    for (what, sdk, tool) in [
        ("house", ours.house, is("ruler")),
        ("exaltation", ours.exaltation, is("exaltation")),
        ("detriment", ours.detriment, is("detriment")),
        ("fall", ours.fall, is("fall")),
    ] {
        score.check(sdk == tool, None, || format!("{name} {key}: {what}"));
    }
    score.check(
        face_lord(longitude).key() == row["face"].as_str().unwrap(),
        None,
        || format!("{name} {key}: the face's lord"),
    );
    // The tool's triplicities are Dorotheus's: a day lord, a night lord and
    // a participating lord. Its day and night lords are Ptolemy's (I.XXI);
    // its participating lord is the SDK's only in water, where Ptolemy has
    // Mars rule by day and night "together with" the others.
    let rules = |lord| {
        Triplicities::Ptolemy.rules(lord, sign, Sect::Day)
            || Triplicities::Ptolemy.rules(lord, sign, Sect::Night)
    };
    for (field, sect) in [
        ("day_triplicity", Sect::Day),
        ("night_triplicity", Sect::Night),
    ] {
        let lord = graha(row[field].as_str().unwrap());
        score.check(Triplicities::Ptolemy.rules(lord, sign, sect), None, || {
            format!("{name} {key}: {field} {lord:?} of {sign:?}")
        });
    }
    let participating = graha(row["participating_triplicity"].as_str().unwrap());
    let third_lord = !rules(participating);
    score.check(!third_lord, Some(PARTICIPATING), String::new);
    for (table, terms) in TABLES {
        let tool = &row["terms"][table];
        let printing = parted.contains(&(table, sign)).then_some(PRINTING);
        let ours_term = terms.table(Sect::Day).lord_at(longitude).unwrap();
        let tool_term = tool["lord"].as_str().unwrap();
        let term_differs = ours_term.key() != tool_term;
        score.check(!term_differs, printing, || {
            format!("{name} {key}: the term's lord in {table}, {ours_term:?} against {tool_term}")
        });
        // The tool's score is Lilly's weights with no peregrine debility and
        // a triplicity counted for any of the sign's lords by day or night
        // (its README). The SDK reads the planet under both sects and takes
        // the triplicity from either.
        let mut either = ours;
        either.term = ours_term == planet;
        either.triplicity = rules(planet);
        let sdk = either.score(&Scores::new(5, 4, 3, 2, 1, -5, -4, 0));
        let recorded = i16::try_from(tool["score"].as_i64().unwrap()).unwrap();
        // What the tool counts that the SDK does not: its third lord's 3,
        // and a term a differing printing gives or takes away.
        let third = participating == planet && third_lord;
        let tool_has_term = tool_term == key;
        let term_points = if term_differs {
            2 * (i16::from(tool_has_term) - i16::from(either.term))
        } else {
            0
        };
        let crux = if term_differs {
            printing
        } else if third {
            Some(PARTICIPATING)
        } else {
            None
        };
        let explained = (sdk + 3 * i16::from(third) + term_points == recorded)
            .then_some(crux)
            .flatten();
        score.check(sdk == recorded, explained, || {
            format!("{name} {key}: the score in {table}, {sdk} against {recorded}")
        });
        let peregrine = either.peregrine();
        let tool_peregrine = tool["peregrine"].as_bool().unwrap();
        let by_the_tool = !(either.house
            || either.exaltation
            || either.triplicity
            || third
            || tool_has_term
            || either.face);
        let explained = (by_the_tool == tool_peregrine).then_some(crux).flatten();
        score.check(peregrine == tool_peregrine, explained, || {
            format!("{name} {key}: peregrine in {table}, {peregrine} against {tool_peregrine}")
        });
    }
}

/// The tool's 21 parts through the SDK's evaluator under the tool's own
/// sect and formulas, then the five that are Valens's lots, under his.
fn score_parts(name: &str, file: &Value, formulas: &Value, score: &mut Score) {
    let sky = sky(file);
    let night = !file["diurnal"].as_bool().unwrap();
    let cusps: Vec<f64> = file["sky"]["cusps_deg"]
        .as_array()
        .unwrap()
        .iter()
        .map(number)
        .collect();
    let request = LotRequest::VALENS.with_sect_rule(if night {
        SectRule::Night
    } else {
        SectRule::Day
    });
    for (part, recorded) in file["parts"].as_object().unwrap() {
        let formula = LotFormula {
            day: arc(&formulas[part][0], &cusps),
            night: arc(&formulas[part][1], &cusps),
        };
        let ours = lot_place(&sky, &formula, request).unwrap().longitude_deg;
        score.check(apart(ours, number(recorded)) <= SAME_DEG, None, || {
            format!("{name}: {part} by the tool's formula, {ours} against {recorded}")
        });
    }
    for (part, lot) in NAMED {
        let ours = lots(&sky, &[lot], request).unwrap().lots[0]
            .place
            .longitude_deg;
        let recorded = number(&file["parts"][part]);
        // Valens counts the father from Venus to the Moon by night and
        // reverses the brothers' (II.31, II.40); the tool does neither.
        let explained = (night && matches!(lot, Lot::Father | Lot::Brothers)).then_some(NIGHT_ARC);
        score.check(apart(ours, recorded) <= SAME_DEG, explained, || {
            format!("{name}: {part} as Valens's {lot:?}, {ours} against {recorded}")
        });
    }
}

#[test]
fn flatlib_terms_dignities_parts_and_sect() {
    if let Err(skip) = carried(SECTION) {
        println!("{skip}");
        return;
    }
    let (manifest, files) = files();
    assert_eq!(
        files.len(),
        usize::try_from(manifest["counts"]["files"].as_u64().unwrap()).unwrap()
    );
    let mut score = Score {
        tally: Tally::new(
            SECTION,
            "flatlib 0.2.3's recorded places fed to the SDK's dignities and lots",
        ),
        unexplained: Vec::new(),
    };
    let parted = score_tables(&manifest, &mut score);
    for (name, file) in &files {
        // The sect: Valens's horizon on the tool's Sun, against the tool's.
        let sect = Sect::from_altitude(number(&file["exporter"]["sun_altitude_deg"]));
        score.check(
            (sect == Sect::Day) == file["diurnal"].as_bool().unwrap(),
            None,
            || format!("{name}: the sect"),
        );
        for (key, row) in file["dignities"].as_object().unwrap() {
            let longitude = number(&file["sky"]["planets"][key]["longitude_deg"]);
            score_dignities(name, key, row, longitude, &parted, &mut score);
        }
        score_parts(name, file, &manifest["parts"], &mut score);
    }
    println!("parted printings: {parted:?}");
    assert!(
        score.unexplained.is_empty(),
        "{} unexplained, the first: {:#?}",
        score.unexplained.len(),
        &score.unexplained[..score.unexplained.len().min(20)]
    );
    score.tally.record();
}
