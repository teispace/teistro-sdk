//! The Ashta Koota, measured (`matching.md`): every pair of the 108 padas,
//! a bride's Moon in one and a groom's in another, read under the verse's
//! rules and under each knob's alternative.
//!
//! A pada fixes everything a koota reads: the nakshatra, the sign and the
//! navamsha. So 108 × 108 pairs are every pair of Moons the kootas can
//! tell apart, and nothing here is sampled.

#![expect(
    clippy::float_cmp,
    reason = "every point is a whole number or a half, exact in binary"
)]

use std::fmt::Write as _;
use std::path::Path;

use teistro::catalogue::Koota;
use teistro::matching::{
    ASHTA_KOOTA, ASHTA_KOOTA_POINTS, AshtaKoota, BhakootDosha, BhakootLift, DevaBride, EqualVarna,
    KootaReading, KootaRules, NadiDosha, Native, ashta_koota,
};

use crate::generated::{Output, check, write};
use crate::measure::{Claim, count, fill, share, spaced, table};

const PAGE: &str = "docs/03-design/matching-measured.md";

/// The padas of the circle.
const PADAS: u16 = 108;

/// One pair read.
struct Pair {
    bride: Native,
    groom: Native,
    read: AshtaKoota,
}

/// Every pada's Moon, at the middle of its arc.
fn natives() -> Result<Vec<Native>, String> {
    (0..PADAS)
        .map(|pada| {
            Native::of_moon((f64::from(pada) + 0.5) * 360.0 / f64::from(PADAS))
                .map_err(|why| format!("pada {pada}: {why}"))
        })
        .collect()
}

fn pairs(natives: &[Native], rules: KootaRules) -> Vec<Pair> {
    natives
        .iter()
        .flat_map(|&bride| {
            natives.iter().map(move |&groom| Pair {
                bride,
                groom,
                read: ashta_koota(bride, groom, rules),
            })
        })
        .collect()
}

fn points(read: &AshtaKoota, koota: Koota) -> f64 {
    read.row(koota).map_or(f64::NAN, |row| row.points)
}

fn claims(natives: &[Native], read: &[Pair]) -> Vec<Claim> {
    let summed = read
        .iter()
        .filter(|one| {
            let sum: f64 = one.read.kootas.iter().map(|row| row.points).sum();
            (sum - one.read.total).abs() > 1e-12 || !(0.0..=ASHTA_KOOTA_POINTS).contains(&sum)
        })
        .count();
    let ordered = read
        .iter()
        .filter(|one| {
            !one.read
                .kootas
                .iter()
                .map(|row| row.reading.koota())
                .eq(ASHTA_KOOTA)
                || one
                    .read
                    .kootas
                    .iter()
                    .zip(1..)
                    .any(|(row, most)| row.max_points != f64::from(most))
        })
        .count();
    let symmetric = [
        Koota::Tara,
        Koota::Yoni,
        Koota::GrahaMaitri,
        Koota::Bhakoot,
        Koota::Nadi,
    ];
    let rules = KootaRules::default();
    let swapped = read
        .iter()
        .filter(|one| {
            let back = ashta_koota(one.groom, one.bride, rules);
            symmetric
                .iter()
                .any(|&koota| points(&one.read, koota) != points(&back, koota))
        })
        .count();
    let itself = natives
        .iter()
        .filter(|&&moon| ashta_koota(moon, moon, rules).total != 28.0)
        .count();
    // The points follow the signs alone (*Daivajna-manohara*): a lifted
    // dosha keeps its 0 and says so as a clause.
    let bhakoot = read
        .iter()
        .filter(
            |one| match one.read.row(Koota::Bhakoot).map(|row| row.reading) {
                Some(KootaReading::Bhakoot { dosha, .. }) => {
                    points(&one.read, Koota::Bhakoot) != if dosha.is_none() { 7.0 } else { 0.0 }
                }
                _ => true,
            },
        )
        .count();
    vec![
        Claim::counted(
            "every total is its kootas' points summed, within 0 to 36",
            summed,
            read.len(),
        ),
        Claim::counted(
            "the eight come in the verse's order, worth 1 to 8",
            ordered,
            read.len(),
        ),
        Claim::counted(
            "Tara, Yoni, Graha Maitri, Bhakoot and Nadi give the same points when the bride and the groom swap",
            swapped,
            read.len(),
        ),
        Claim::counted(
            "a Moon matched with itself totals 28",
            itself,
            natives.len(),
        ),
        Claim::counted(
            "Bhakoot gives its 7 exactly when the signs stand without a dosha, a lifted dosha keeping its 0",
            bhakoot,
            read.len(),
        ),
    ]
}

/// How the totals spread, in bands of six points.
fn spread(read: &[Pair]) -> String {
    let mut out = String::from("| total | pairs | share |\n|---|---|---|\n");
    for low in (0..36).step_by(6) {
        let high = low + 6;
        let within = read
            .iter()
            .filter(|one| {
                let total = one.read.total;
                total >= f64::from(low)
                    && (total < f64::from(high) || (high == 36 && total <= 36.0))
            })
            .count();
        let band = if high == 36 {
            format!("{low} to 36")
        } else {
            format!("{low} to under {high}")
        };
        let _ = writeln!(
            out,
            "| {band} | {} | {} |",
            count(within),
            share(within, read.len())
        );
    }
    out
}

/// Each koota's mean points, and how often it gives all or none.
fn by_koota(read: &[Pair]) -> String {
    let mut out =
        String::from("| koota | most | mean | all its points | none |\n|---|---|---|---|---|\n");
    for (koota, most) in ASHTA_KOOTA.into_iter().zip(1_u8..) {
        let given: Vec<f64> = read.iter().map(|one| points(&one.read, koota)).collect();
        let mean =
            given.iter().sum::<f64>() / f64::from(u32::try_from(given.len()).unwrap_or(u32::MAX));
        let all = given.iter().filter(|&&p| p == f64::from(most)).count();
        let none = given.iter().filter(|&&p| p == 0.0).count();
        let _ = writeln!(
            out,
            "| {} | {most} | {} | {} | {} |",
            koota.key(),
            spaced(mean, 2),
            share(all, read.len()),
            share(none, read.len()),
        );
    }
    out
}

/// The bad Bhakoots, and how often each exception holds of them.
fn bhakoot(read: &[Pair], garga: &[Pair]) -> String {
    let doshas: Vec<_> = read
        .iter()
        .zip(garga)
        .filter_map(|(one, theirs)| {
            match (
                one.read.row(Koota::Bhakoot).map(|row| row.reading),
                theirs.read.row(Koota::Bhakoot).map(|row| row.reading),
            ) {
                (
                    Some(KootaReading::Bhakoot {
                        dosha: Some(dosha),
                        exceptions,
                        lifted,
                        ..
                    }),
                    Some(KootaReading::Bhakoot {
                        lifted: by_garga, ..
                    }),
                ) => Some((dosha, exceptions, lifted, by_garga)),
                _ => None,
            }
        })
        .collect();
    let mut out = String::from(
        "| dosha | pairs | one lord | lords friends | navamsha lords friends | tara pure | vashya | lifted by any one | lifted by Garga's count |\n\
         |---|---|---|---|---|---|---|---|---|\n",
    );
    for (name, kind) in [
        ("6/8", BhakootDosha::SixEight),
        ("5/9", BhakootDosha::FiveNine),
        ("2/12", BhakootDosha::TwoTwelve),
    ] {
        let these: Vec<_> = doshas.iter().filter(|one| one.0 == kind).collect();
        let held = |clause: fn(&teistro::matching::BhakootExceptions) -> bool| {
            share(
                these.iter().filter(|one| clause(&one.1)).count(),
                these.len(),
            )
        };
        let _ = writeln!(
            out,
            "| {name} | {} | {} | {} | {} | {} | {} | {} | {} |",
            count(these.len()),
            held(|e| e.one_lord),
            held(|e| e.lords_friends),
            held(|e| e.navamsha_lords_friends),
            held(|e| e.tara_pure),
            held(|e| e.vashya),
            share(these.iter().filter(|one| one.2).count(), these.len()),
            share(these.iter().filter(|one| one.3).count(), these.len()),
        );
    }
    out
}

/// How many pairs each knob's alternative moves, and by how much at most.
fn knobs(natives: &[Native], read: &[Pair]) -> String {
    let alternatives: [(&str, KootaRules); 4] = [
        (
            "an equal varna gives half (C259)",
            KootaRules {
                equal_varna: EqualVarna::Half,
                ..KootaRules::default()
            },
        ),
        (
            "a Deva bride with a Manushya groom gives 3 (C262)",
            KootaRules {
                deva_bride: DevaBride::Three,
                ..KootaRules::default()
            },
        ),
        (
            "a bad Bhakoot is lifted by Garga's count (C263)",
            KootaRules {
                bhakoot_lift: BhakootLift::Garga,
                ..KootaRules::default()
            },
        ),
        (
            "only the middle nadi is a dosha (C264)",
            KootaRules {
                nadi_dosha: NadiDosha::MiddleOnly,
                ..KootaRules::default()
            },
        ),
    ];
    let mut out = String::from(
        "| reading | totals it moves | share | most it moves | readings it changes | share |\n|---|---|---|---|---|---|\n",
    );
    for (label, rules) in alternatives {
        let theirs = pairs(natives, rules);
        let moved: Vec<f64> = theirs
            .iter()
            .zip(read)
            .map(|(theirs, ours)| (theirs.read.total - ours.read.total).abs())
            .filter(|&moved| moved > 0.0)
            .collect();
        let most = moved.iter().copied().fold(0.0, f64::max);
        let changed = theirs
            .iter()
            .zip(read)
            .filter(|(theirs, ours)| theirs.read != ours.read)
            .count();
        let _ = writeln!(
            out,
            "| {label} | {} | {} | {} | {} | {} |",
            count(moved.len()),
            share(moved.len(), read.len()),
            spaced(most, 1),
            count(changed),
            share(changed, read.len()),
        );
    }
    out
}

fn page() -> Result<String, String> {
    let natives = natives()?;
    let read = pairs(&natives, KootaRules::default());
    let garga = pairs(
        &natives,
        KootaRules {
            bhakoot_lift: BhakootLift::Garga,
            ..KootaRules::default()
        },
    );
    let mut out = String::from(
        "# The Ashta Koota, measured\n\n\
         Status: `generated` by `cargo xtask matching` from the kernel \
         alone, 2026-10-04. Do not edit: `check-matching` regenerates this \
         page and fails on any difference.\n\n",
    );
    let _ = write!(
        out,
        "A pada fixes all a koota reads: the nakshatra, the sign and the \
         navamsha. So the {} padas give {} pairs, a bride's Moon in one and \
         a groom's in another, which is every pair the Ashta Koota can tell \
         apart (`matching.md`). Each is read under the verse's rules, the \
         default, and again under each knob's alternative.\n\n",
        count(natives.len()),
        count(read.len()),
    );
    out.push_str(&table(&claims(&natives, &read)));
    out.push_str("\n## How the totals spread\n\n");
    out.push_str(&spread(&read));
    out.push_str(
        "\n## Each koota\n\n\
         The mean of each koota's points over every pair, and how often it \
         gives all its points or none.\n\n",
    );
    out.push_str(&by_koota(&read));
    out.push_str(
        "\n## The bad Bhakoots and their exceptions\n\n\
         For each dosha, how often each of the five exceptions of VI.32–33 \
         holds, and how often the dosha is lifted: by any one exception with \
         the nadi pure, the verse's reading, or by Garga's count, three for \
         the 6/8 and two for the others (C263).\n\n",
    );
    out.push_str(&bhakoot(&read, &garga));
    out.push_str(
        "\n## What each knob moves\n\n\
         How many pairs change when one reading is swapped for its \
         alternative, the others left at the verse's: in their totals, and \
         in anything the answer says. The Bhakoot's lift and the nadi's \
         dosha are clauses beside points the signs and the nadis fix, so \
         those two knobs can change what is said without moving a total.\n\n",
    );
    out.push_str(&knobs(&natives, &read));
    Ok(fill(&out))
}

pub(crate) fn generate(root: &Path) -> i32 {
    match page() {
        Ok(text) => write(root, &[Output::new(PAGE, text)]),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    match page() {
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask matching") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}
