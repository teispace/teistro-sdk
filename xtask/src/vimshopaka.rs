//! The falsification pass over the Vimshopaka (Phase 5).
//!
//! The corpus's `baseline/vimshopaka` records the recording engine's
//! Vimshopaka for every fixture with its divisional charts, computed from the
//! seven grahas' recorded signs in the sixteen vargas, with the dignity it
//! scored each varga by and the four schemes' weights.
//!
//! BPHS ch. 7 (English translation, verses 17 to 27) gives the weights, and
//! the corpus's agree; it scores a varga 20 in the graha's own, 18 in a great
//! friend's, 15 in a friend's, 10 in a neutral's, 7 in an enemy's and 5 in a
//! great enemy's, the full 20 in exaltation as in its own (verses 9 to 12),
//! and a varga's share is those points times its weight over 20. The engine
//! scores by a different scale — the Saptavargaja virupas over 45, which puts
//! a friend's varga at a third of the text's — and by natural friendship
//! alone, which never reaches a great friend or great enemy. Both are
//! measured here, the text's with the compound relationship taken in the
//! rasi chart.
//!
//! `cargo xtask vimshopaka` writes the page; `check-vimshopaka` regenerates it
//! in memory and fails on any difference.

use std::fmt::Write as _;
use std::path::Path;

use serde_json::Value;
use teistro_core::catalogue::{Graha, Rashi};

use crate::generated::{Output, check, write};
use crate::measure::{Claim, count, fill, table, worst};

const PAGE: &str = "docs/03-design/vimshopaka-measured.md";
const ROOT: &str = "fixtures/baseline/vimshopaka";
const GRAHAS: [Graha; 7] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];
const SCHEMES: [&str; 4] = ["shadvarga", "saptavarga", "dashavarga", "shodashavarga"];

/// The weights BPHS ch. 7 vv. 17 to 25 gives, in halves of a point, by
/// scheme and varga.
const WEIGHTS: [&[(&str, u8)]; 4] = [
    &[
        ("D1", 12),
        ("D2", 4),
        ("D3", 8),
        ("D9", 10),
        ("D12", 4),
        ("D30", 2),
    ],
    &[
        ("D1", 10),
        ("D2", 4),
        ("D3", 6),
        ("D7", 5),
        ("D9", 9),
        ("D12", 4),
        ("D30", 2),
    ],
    &[
        ("D1", 6),
        ("D2", 3),
        ("D3", 3),
        ("D7", 3),
        ("D9", 3),
        ("D10", 3),
        ("D12", 3),
        ("D16", 3),
        ("D30", 3),
        ("D60", 10),
    ],
    &[
        ("D1", 7),
        ("D2", 2),
        ("D3", 2),
        ("D4", 1),
        ("D7", 1),
        ("D9", 6),
        ("D10", 1),
        ("D12", 1),
        ("D16", 4),
        ("D20", 1),
        ("D24", 1),
        ("D27", 1),
        ("D30", 2),
        ("D40", 1),
        ("D45", 1),
        ("D60", 8),
    ],
];

/// One recorded Vimshopaka.
struct Record {
    /// Each varga's key and the seven grahas' signs in it.
    signs: Vec<(String, [Rashi; 7])>,
    /// Each varga's key and the dignity the engine scored each graha by.
    dignity: Vec<(String, [String; 7])>,
    /// Each graha's four scores.
    scores: [[f64; 4]; 7],
}

fn records(root: &Path) -> Result<(Vec<Record>, Value), String> {
    let read = |path: &Path| -> Result<Value, String> {
        serde_json::from_str(
            &std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?,
        )
        .map_err(|e| format!("{}: {e}", path.display()))
    };
    let manifest = read(&root.join(ROOT).join("manifest.json"))?;
    let mut out = Vec::new();
    for dir in ["charts", "variants"] {
        let mut paths: Vec<_> = std::fs::read_dir(root.join(ROOT).join(dir))
            .map_err(|e| format!("{dir}: {e}"))?
            .flatten()
            .map(|entry| entry.path())
            .collect();
        paths.sort();
        for path in paths {
            let file = read(&path)?;
            let mut signs = Vec::new();
            let mut dignity = Vec::new();
            for (varga, by_graha) in file["inputs"]["sign_index"]
                .as_object()
                .ok_or("sign_index")?
            {
                let mut row = [Rashi::Aries; 7];
                let mut names: [String; 7] = Default::default();
                for (i, graha) in GRAHAS.iter().enumerate() {
                    row[i] = by_graha[graha.key()]
                        .as_u64()
                        .and_then(|n| u16::try_from(n).ok())
                        .and_then(Rashi::from_id)
                        .ok_or("sign")?;
                    file["dignity"][varga][graha.key()]
                        .as_str()
                        .ok_or("dignity")?
                        .clone_into(&mut names[i]);
                }
                signs.push((varga.clone(), row));
                dignity.push((varga.clone(), names));
            }
            let mut scores = [[0.0; 4]; 7];
            for (i, graha) in GRAHAS.iter().enumerate() {
                for (j, scheme) in SCHEMES.iter().enumerate() {
                    scores[i][j] = file["scores"][graha.key()][scheme]
                        .as_f64()
                        .ok_or("score")?;
                }
            }
            out.push(Record {
                signs,
                dignity,
                scores,
            });
        }
    }
    Ok((out, manifest))
}

// ── The rules proposed ─────────────────────────────────────────────────────

/// The dignity the engine scores a graha by in a varga: exaltation,
/// debilitation, moolatrikona and own sign by sign alone, then natural
/// friendship with the sign's lord.
fn engine_dignity(graha: Graha, sign: Rashi) -> &'static str {
    let attributes = graha.attributes();
    if attributes.exaltation.is_some_and(|at| at.sign == sign) {
        "EXALTED"
    } else if attributes.debilitation.is_some_and(|at| at.sign == sign) {
        "DEBILITATED"
    } else if attributes
        .moolatrikona
        .is_some_and(|span| span.sign == sign)
    {
        "MOOLTRIKONA"
    } else if attributes.own.contains(&sign) {
        "OWN_SIGN"
    } else {
        let lord = sign.attributes().lord;
        if attributes.friends.contains(&lord) {
            "FRIEND"
        } else if attributes.enemies.contains(&lord) {
            "ENEMY"
        } else {
            "NEUTRAL"
        }
    }
}

/// The engine's virupas for a dignity.
fn virupas(dignity: &str) -> f64 {
    match dignity {
        "EXALTED" | "DEEP_EXALTED" => 45.0,
        "MOOLTRIKONA" | "OWN_SIGN" => 30.0,
        "GREAT_FRIEND" => 22.5,
        "FRIEND" => 15.0,
        "NEUTRAL" => 7.5,
        "ENEMY" => 3.75,
        "GREAT_ENEMY" => 1.875,
        _ => 0.0,
    }
}

/// The text's points for a graha in a varga sign: 20 in exaltation or its own
/// sign, else by its compound relationship with the sign's lord, the
/// temporary part taken from where both stand in the rasi chart.
fn text_points(graha: Graha, sign: Rashi, rasi: &[Rashi; 7]) -> f64 {
    let attributes = graha.attributes();
    if attributes.exaltation.is_some_and(|at| at.sign == sign) || attributes.own.contains(&sign) {
        return 20.0;
    }
    let lord = sign.attributes().lord;
    let natural = if attributes.friends.contains(&lord) {
        1
    } else if attributes.enemies.contains(&lord) {
        -1
    } else {
        0
    };
    let at = |g: Graha| GRAHAS.iter().position(|x| *x == g).map(|i| rasi[i]);
    let temporary = match (at(graha), at(lord)) {
        (Some(from), Some(to)) => {
            let distance = (to as usize + 12 - from as usize) % 12 + 1;
            if [2, 3, 4, 10, 11, 12].contains(&distance) {
                1
            } else {
                -1
            }
        }
        _ => 0,
    };
    match natural + temporary {
        2 => 18.0,
        1 => 15.0,
        0 => 10.0,
        -1 => 7.0,
        _ => 5.0,
    }
}

/// A scheme's score per graha: each varga's fraction of full strength times
/// its weight, accumulated in the scheme's order.
fn score(scheme: usize, points: impl Fn(&str, usize) -> f64) -> [f64; 7] {
    let mut out = [0.0; 7];
    for (graha, slot) in out.iter_mut().enumerate() {
        for (varga, halves) in WEIGHTS[scheme] {
            *slot += points(varga, graha) * (f64::from(*halves) / 2.0);
        }
    }
    out
}

fn rasi_of(record: &Record) -> [Rashi; 7] {
    record
        .signs
        .iter()
        .find(|(v, _)| v == "D1")
        .map_or([Rashi::Aries; 7], |(_, s)| *s)
}

fn sign_in(record: &Record, varga: &str, graha: usize) -> Rashi {
    record
        .signs
        .iter()
        .find(|(v, _)| v == varga)
        .map_or(Rashi::Aries, |(_, s)| s[graha])
}

// ── The page ───────────────────────────────────────────────────────────────

fn claims(records: &[Record], manifest: &Value) -> Vec<Claim> {
    let charts = records.len();
    let mut claims = Vec::new();

    let wrong = SCHEMES
        .iter()
        .zip(WEIGHTS)
        .filter(|(key, weights)| {
            let recorded = manifest["schemes"]
                .as_array()
                .and_then(|schemes| schemes.iter().find(|s| s["key"] == **key))
                .map(|s| &s["weights"]);
            recorded.is_none_or(|w| {
                w.as_object().map_or(0, serde_json::Map::len) != weights.len()
                    || weights
                        .iter()
                        .any(|(v, halves)| w[*v].as_f64() != Some(f64::from(*halves) / 2.0))
            })
        })
        .count();
    claims.push(Claim::counted(
        "the four schemes' weights are BPHS ch. 7 vv. 17 to 25's, each summing to 20",
        wrong,
        4,
    ));

    let cells = charts * 16 * 7;
    let wrong = records
        .iter()
        .flat_map(|r| r.signs.iter().zip(&r.dignity))
        .flat_map(|((_, signs), (_, names))| (0..7).map(move |g| (signs[g], &names[g], g)))
        .filter(|(sign, name, g)| engine_dignity(GRAHAS[*g], *sign) != name.as_str())
        .count();
    claims.push(Claim::counted(
        "the engine scores a varga by exaltation, debilitation, moolatrikona and own sign by the sign alone, then by **natural** friendship with the sign's lord",
        wrong,
        cells,
    ));

    let mut far = 0.0_f64;
    let wrong = records
        .iter()
        .filter(|r| {
            (0..4).any(|scheme| {
                let ours = score(scheme, |varga, g| {
                    virupas(engine_dignity(GRAHAS[g], sign_in(r, varga, g))) / 45.0
                });
                ours.iter().zip(&r.scores).any(|(o, recorded)| {
                    // The engine's own arithmetic: normalised by the total weight
                    // and back, and rounded half up to hundredths.
                    let rounded = ((o / 20.0 * 20.0) * 100.0 + 0.5).floor() / 100.0;
                    far = far.max((rounded - recorded[scheme]).abs());
                    (rounded - recorded[scheme]).abs() > 1e-9
                })
            })
        })
        .count();
    claims.push(
        Claim::counted(
            "a scheme's score is each varga's Saptavargaja virupas over 45 times its weight, summed and rounded to hundredths",
            wrong,
            charts,
        )
        .with_note(format!("worst {far:.1e}")),
    );

    let (mut differ, mut far) = (0, 0.0_f64);
    for r in records {
        let rasi = rasi_of(r);
        for scheme in 0..4 {
            let text = score(scheme, |varga, g| {
                text_points(GRAHAS[g], sign_in(r, varga, g), &rasi) / 20.0
            });
            for (t, recorded) in text.iter().zip(&r.scores) {
                let gap = (t - recorded[scheme]).abs();
                far = far.max(gap);
                differ += usize::from(gap > 0.005);
            }
        }
    }
    claims.push(
        Claim::counted(
            "the text's: 20 in exaltation or the own sign, else 18, 15, 10, 7 or 5 by the compound relationship in the rasi chart, times the weight over 20",
            differ,
            charts * 28,
        )
        .with_note(format!("worst gap {far:.2} of 20")),
    );
    claims
}

fn page(root: &Path) -> Result<String, String> {
    let (records, manifest) = records(root)?;
    if records.is_empty() {
        return Err(String::from("the corpus records no Vimshopaka"));
    }
    let friend_share = worst(std::iter::once(15.0 / 45.0 * 20.0));
    let mut out = String::new();
    let _ = write!(
        out,
        "# The Vimshopaka, measured\n\n\
         Status: `generated` by `cargo xtask vimshopaka` over the conformance corpus's \
         `baseline/vimshopaka`, 2026-09-15. Do not edit: `check-vimshopaka` regenerates this page \
         and fails on any difference. The design it measures is \
         [`strength-schemes.md`](strength-schemes.md).\n\n\
         The corpus records the recording engine's Vimshopaka for {charts} charts, computed from the \
         seven grahas' recorded signs in the sixteen vargas, under the shadvarga, saptavarga, \
         dashavarga and shodashavarga schemes. BPHS ch. 7 was read beside it.\n\n\
         ## What the corpus decides\n\n{table}\n",
        charts = count(records.len()),
        table = table(&claims(&records, &manifest)),
    );
    let _ = write!(
        out,
        "## What it means for the module\n\n\
         **The weights are settled**: the engine's are the text's.\n\n\
         **The scoring is not the text's.** The engine reuses the Saptavargaja virupas and divides \
         by 45, so a varga in a friend's sign earns {friend_share:.2} of 20 where the text gives 15, \
         and it takes natural friendship alone, so no varga is ever a great friend's or a great \
         enemy's. A rank-1 text corrects a rank-2 value, so the module's default is the text's \
         points by the compound relationship and the engine's scale is a setting the conformance \
         profile takes. Which chart the compound relationship's temporary half is taken in, and \
         whether debilitation takes anything from a varga, the text does not settle (crux C63).\n",
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
            i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask vimshopaka") != 0)
        }
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}
