//! The tables a reader vets, written from the kernels that ship them
//! (`pakshi.md` §6, `lalkitab.md` §7 step 4). Pancha Pakshi and Lal Kitab
//! rest on texts the maintainers cannot read in the original, so their
//! content goes to a reader who can: this pass lays out every cell each
//! kernel answers, beside the page it cites, numbered so a reader can
//! name a cell. The sheets are made by calling the kernels' own
//! functions, never copied from their sources, so what a reader marks is
//! what ships, and the gate fails when a kernel changes and its sheet
//! does not.

use std::fmt::Write as _;
use std::path::Path;

use teistro::catalogue::{Graha, Nakshatra, Paksha, Vara};
use teistro::lalkitab::aspects::{Relation, looks_at, yog_drishti};
use teistro::lalkitab::tables::{
    MASNUI, PLANETS, RINAS, cycle_years, debilitated, exalted, joint_owners, mahadasha_years,
    matures_at, own, pakka_ghar, regard, root, thirds, waker,
};
use teistro::pakshi::tables::{
    Activity, Bird, BirthBird, Half, Relations, SubLengths, activity, birth_bird, death_bird,
    first_eater, relation, sequence, sub_share,
};

use crate::generated::{Output, check, write};

const PAKSHI: &str = "docs/03-design/pakshi-tables.md";
const LALKITAB: &str = "docs/03-design/lalkitab-tables.md";

/// A catalogue key as a reader writes it: `PURVA_PHALGUNI` as "Purva
/// Phalguni".
fn spoken(key: &str) -> String {
    key.split('_')
        .map(|word| {
            let mut letters = word.chars();
            letters.next().map_or_else(String::new, |first| {
                first.to_string() + &letters.as_str().to_lowercase()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// A serde-named value (a bird, an activity) as a reader writes it.
fn said(value: impl serde::Serialize) -> String {
    match serde_json::to_value(value) {
        Ok(serde_json::Value::String(key)) => spoken(&key),
        other => format!("{other:?}"),
    }
}

/// A Markdown table: the header, then each row, the first column the
/// cell id a reader quotes.
fn grid(header: &[&str], rows: &[Vec<String>]) -> String {
    let mut out = format!(
        "| {} |\n|{}\n",
        header.join(" | "),
        "---|".repeat(header.len())
    );
    for row in rows {
        let _ = writeln!(out, "| {} |", row.join(" | "));
    }
    out
}

/// A table's heading: its id, what it is, and the page it cites.
fn heading(out: &mut String, id: &str, title: &str, cites: &str) {
    let _ = write!(out, "\n## {id}. {title}\n\n*Cites:* {cites}.\n\n");
}

const HALVES: [(Paksha, Half, &str); 4] = [
    (Paksha::Shukla, Half::Day, "bright day"),
    (Paksha::Shukla, Half::Night, "bright night"),
    (Paksha::Krishna, Half::Day, "dark day"),
    (Paksha::Krishna, Half::Night, "dark night"),
];

const ACTIVITIES: [Activity; 5] = [
    Activity::Eating,
    Activity::Walking,
    Activity::Ruling,
    Activity::Sleeping,
    Activity::Dying,
];

fn pakshi_page() -> String {
    let mut out = String::from(
        "# Pancha Pakshi: the tables, for a reader\n\n\
         Status: `generated` by `cargo xtask review-sheets` from `crates/pakshi`,\n\
         2026-10-12. Do not edit: `check-review-sheets` regenerates this page\n\
         and fails on any difference.\n\n\
         Every cell `crates/pakshi` answers, laid out from its own functions, beside\n\
         the page it cites (short names as in [`pakshi.md`](pakshi.md) §1). A reader\n\
         who finds a cell the print does not bear out names it by its id — `P4.17.3`\n\
         is table P4, row 17, the third yama — and says what the print has. The\n\
         questions the kernel could not settle are [`pakshi.md`](pakshi.md) §6.\n",
    );

    out += &table_p1();
    out += &table_p2();
    out += &table_p3();
    out += &table_p4();
    out += &table_p5();
    out += &table_p6();
    out += &table_p7();
    out += &table_p8();
    out
}

fn table_p1() -> String {
    let mut out = String::new();
    heading(
        &mut out,
        "P1",
        "The native's bird from the birth star",
        "AY pp. 104–105; AG07 p. 60 vv. 22, 24 (`SINGLE`); PUL p. vi (`BY_PAKSHA`, the default)",
    );
    let rows: Vec<Vec<String>> = Nakshatra::ALL
        .iter()
        .enumerate()
        .map(|(at, star)| {
            vec![
                format!("P1.{}", at + 1),
                spoken(star.key()),
                said(birth_bird(*star, Paksha::Shukla, BirthBird::ByPaksha)),
                said(birth_bird(*star, Paksha::Krishna, BirthBird::ByPaksha)),
                said(birth_bird(*star, Paksha::Krishna, BirthBird::Single)),
            ]
        })
        .collect();
    out += &grid(
        &[
            "id",
            "star",
            "bright half",
            "dark half, `BY_PAKSHA`",
            "either half, `SINGLE`",
        ],
        &rows,
    );

    out
}

fn table_p2() -> String {
    let mut out = String::new();
    heading(
        &mut out,
        "P2",
        "The order a half's activities run in",
        "AG p5 v. 6, p15 v. 3, p25 v. 3; AY pp. 106–109",
    );
    let rows: Vec<Vec<String>> = HALVES
        .iter()
        .enumerate()
        .map(|(at, (paksha, half, name))| {
            let mut row = vec![format!("P2.{}", at + 1), (*name).to_owned()];
            row.extend(sequence(*paksha, *half).into_iter().map(said));
            row
        })
        .collect();
    out += &grid(&["id", "half", "1", "2", "3", "4", "5"], &rows);

    out
}

fn table_p3() -> String {
    let mut out = String::new();
    heading(
        &mut out,
        "P3",
        "The bird that eats in a half's first yama",
        "AG p15 v. 1, p20 v. 1, p25 v. 1; AY pp. 106–109",
    );
    let rows: Vec<Vec<String>> = Vara::ALL
        .iter()
        .enumerate()
        .map(|(at, vara)| {
            let mut row = vec![format!("P3.{}", at + 1), spoken(vara.key())];
            row.extend(
                HALVES
                    .iter()
                    .map(|(paksha, half, _)| said(first_eater(*paksha, *half, *vara))),
            );
            row
        })
        .collect();
    let mut header = vec!["id", "weekday"];
    header.extend(HALVES.iter().map(|(_, _, name)| *name));
    out += &grid(&header, &rows);

    out
}

fn table_p4() -> String {
    let mut out = String::new();
    heading(
        &mut out,
        "P4",
        "Each bird's activity in each yama",
        "Ayyar's four tables, AY pp. 106–109, the rule reproducing every cell but his printing slip (crux P7); AG pp. 7–9",
    );
    let mut rows = Vec::new();
    for (paksha, half, name) in HALVES {
        for vara in Vara::ALL {
            for bird in Bird::ALL {
                let mut row = vec![
                    format!("P4.{}", rows.len() + 1),
                    name.to_owned(),
                    spoken(vara.key()),
                    said(bird),
                ];
                row.extend((0..5).map(|yama| said(activity(bird, paksha, half, vara, yama))));
                rows.push(row);
            }
        }
    }
    out += &grid(
        &["id", "half", "weekday", "bird", "1", "2", "3", "4", "5"],
        &rows,
    );

    out
}

fn table_p5() -> String {
    let mut out = String::new();
    heading(
        &mut out,
        "P5",
        "The bird dead the whole day",
        "AG p15 v. 2, p25 v. 2; PUL p. 57",
    );
    let rows: Vec<Vec<String>> = Vara::ALL
        .iter()
        .enumerate()
        .map(|(at, vara)| {
            vec![
                format!("P5.{}", at + 1),
                spoken(vara.key()),
                said(death_bird(Paksha::Shukla, *vara)),
                said(death_bird(Paksha::Krishna, *vara)),
            ]
        })
        .collect();
    out += &grid(&["id", "weekday", "bright half", "dark half"], &rows);

    out
}

fn table_p6() -> String {
    let mut out = String::new();
    heading(
        &mut out,
        "P6",
        "A sub-period's share of its yama, in minutes of 144",
        "AG p5 vv. 7–9, p17 (`AGASTYA`, one length for both halves); PUL p. 46 (`PULIPPANI`)",
    );
    let rows: Vec<Vec<String>> = ACTIVITIES
        .iter()
        .enumerate()
        .map(|(at, done)| {
            let mut row = vec![format!("P6.{}", at + 1), said(*done)];
            row.push(sub_share(*done, Paksha::Shukla, Half::Day, SubLengths::Agastya).to_string());
            row.extend(HALVES.iter().map(|(paksha, half, _)| {
                sub_share(*done, *paksha, *half, SubLengths::Pulippani).to_string()
            }));
            row
        })
        .collect();
    let mut header = vec!["id", "activity", "Agastya"];
    let named: Vec<String> = HALVES
        .iter()
        .map(|(_, _, name)| format!("Pulippani, {name}"))
        .collect();
    header.extend(named.iter().map(String::as_str));
    out += &grid(&header, &rows);

    out
}

fn table_p7() -> String {
    let mut out = String::new();
    heading(
        &mut out,
        "P7",
        "How one bird regards another",
        "AG p51, p40 v. 13 (`AGASTYA`, both halves); PUL p. 48 (bright) and p. 121 (dark) (`PULIPPANI`)",
    );
    let schemes = [
        (Relations::Agastya, Paksha::Shukla, "Agastya"),
        (Relations::Pulippani, Paksha::Shukla, "Pulippani, bright"),
        (Relations::Pulippani, Paksha::Krishna, "Pulippani, dark"),
    ];
    let mut rows = Vec::new();
    for (scheme, paksha, name) in schemes {
        for of in Bird::ALL {
            let mut row = vec![format!("P7.{}", rows.len() + 1), name.to_owned(), said(of)];
            row.extend(Bird::ALL.map(|to| said(relation(of, to, paksha, scheme))));
            rows.push(row);
        }
    }
    let birds: Vec<String> = Bird::ALL.iter().map(|bird| said(*bird)).collect();
    let mut header = vec!["id", "scheme", "how"];
    header.extend(birds.iter().map(String::as_str));
    out += &grid(&header, &rows);

    out
}

fn table_p8() -> String {
    let mut out = String::new();
    heading(
        &mut out,
        "P8",
        "A bird's natural strength, in eighths",
        "AG p40, the *mūkku pakṣi* verse",
    );
    let rows: Vec<Vec<String>> = Bird::ALL
        .iter()
        .enumerate()
        .map(|(at, bird)| {
            vec![
                format!("P8.{}", at + 1),
                said(*bird),
                bird.strength_eighths().to_string(),
            ]
        })
        .collect();
    out += &grid(&["id", "bird", "eighths"], &rows);
    out
}

fn houses(list: &[u8]) -> String {
    if list.is_empty() {
        return "—".to_owned();
    }
    list.iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn planets(list: &[Graha]) -> String {
    list.iter()
        .map(|graha| spoken(graha.key()))
        .collect::<Vec<_>>()
        .join(", ")
}

fn lalkitab_page() -> String {
    let mut out = String::from(
        "# Lal Kitab: the tables, for a reader\n\n\
         Status: `generated` by `cargo xtask review-sheets` from `crates/lalkitab`,\n\
         2026-10-12. Do not edit: `check-review-sheets` regenerates this page\n\
         and fails on any difference.\n\n\
         Every table `crates/lalkitab` holds, laid out from its own functions, beside\n\
         the page of the 1952 edition it cites. A reader with the Urdu who finds a\n\
         cell the print does not bear out names it by its id — `L5.3` is table L5,\n\
         row 3 — and says what the page has. The open readings are\n\
         [`lalkitab.md`](lalkitab.md) §6 and §7 step 4.\n",
    );

    out += &table_l1();
    out += &table_l2();
    out += &table_l3();
    out += &table_l4();
    out += &table_l5();
    out += &table_l6();
    out += &table_l7();
    out
}

fn table_l1() -> String {
    let mut out = String::new();
    heading(
        &mut out,
        "L1",
        "Each planet's houses",
        "pakka ghar p. 29; owned pp. ~6–7, the nodes' joint houses p. ~46; exalted and debilitated pp. ~44–46; root p. 128",
    );
    let rows: Vec<Vec<String>> = PLANETS
        .iter()
        .enumerate()
        .map(|(at, graha)| {
            vec![
                format!("L1.{}", at + 1),
                spoken(graha.key()),
                houses(pakka_ghar(*graha)),
                houses(&own(*graha)),
                houses(&exalted(*graha)),
                houses(&debilitated(*graha)),
                houses(root(*graha)),
            ]
        })
        .collect();
    out += &grid(
        &[
            "id",
            "planet",
            "pakka ghar",
            "owns",
            "exalted",
            "debilitated",
            "root",
        ],
        &rows,
    );
    let shared: Vec<String> = (1..=12_u8)
        .filter_map(|house| joint_owners(house).map(|pair| format!("{house}: {}", planets(&pair))))
        .collect();
    let _ = writeln!(out, "\nHeld jointly when empty: {}.", shared.join("; "));

    out
}

fn table_l2() -> String {
    let mut out = String::new();
    heading(&mut out, "L2", "How one planet regards another", "p. 31");
    let mut rows = Vec::new();
    for of in PLANETS {
        let mut row = vec![format!("L2.{}", rows.len() + 1), spoken(of.key())];
        row.extend(PLANETS.map(|to| regard(of, to).map_or_else(|| "—".to_owned(), said)));
        rows.push(row);
    }
    let names: Vec<String> = PLANETS.iter().map(|graha| spoken(graha.key())).collect();
    let mut header = vec!["id", "how"];
    header.extend(names.iter().map(String::as_str));
    out += &grid(&header, &rows);

    out
}

fn table_l3() -> String {
    let mut out = String::new();
    heading(
        &mut out,
        "L3",
        "Years and ages",
        "the 35-year cycle and the mahadasha p. 33; maturity pp. 33, 99; the thirds of a year p. 34",
    );
    let rows: Vec<Vec<String>> = PLANETS
        .iter()
        .enumerate()
        .map(|(at, graha)| {
            vec![
                format!("L3.{}", at + 1),
                spoken(graha.key()),
                cycle_years(*graha).to_string(),
                mahadasha_years(*graha).to_string(),
                matures_at(*graha).map_or_else(|| "—".to_owned(), |age| age.to_string()),
                thirds(*graha).map_or_else(|| "—".to_owned(), |three| planets(&three)),
            ]
        })
        .collect();
    out += &grid(
        &[
            "id",
            "planet",
            "cycle years",
            "mahadasha years",
            "matures at",
            "months 1–4, 5–8, 9–12",
        ],
        &rows,
    );

    out
}

fn table_l4() -> String {
    let mut out = String::new();
    heading(
        &mut out,
        "L4",
        "The planet that wakes a sleeping house",
        "p. 98",
    );
    let rows: Vec<Vec<String>> = (1..=12_u8)
        .map(|house| {
            vec![
                format!("L4.{house}"),
                house.to_string(),
                spoken(waker(house).key()),
            ]
        })
        .collect();
    out += &grid(&["id", "house", "waker"], &rows);

    out
}

fn table_l5() -> String {
    let mut out = String::new();
    heading(&mut out, "L5", "Artificial planets", "p. 27");
    let rows: Vec<Vec<String>> = MASNUI
        .iter()
        .enumerate()
        .map(|(at, (one, two, made))| {
            vec![
                format!("L5.{}", at + 1),
                planets(&[*one, *two]),
                said(*made),
            ]
        })
        .collect();
    out += &grid(&["id", "pair in one house", "counts as"], &rows);

    out
}

fn table_l6() -> String {
    let mut out = String::new();
    heading(&mut out, "L6", "The nine debts", "p. 125");
    let rows: Vec<Vec<String>> = RINAS
        .iter()
        .enumerate()
        .map(|(at, (rin, graha, seats, enemies))| {
            vec![
                format!("L6.{}", at + 1),
                said(*rin),
                spoken(graha.key()),
                houses(seats),
                planets(enemies),
            ]
        })
        .collect();
    out += &grid(&["id", "debt", "planet", "its houses", "raised by"], &rows);

    out
}

fn table_l7() -> String {
    let mut out = String::new();
    heading(
        &mut out,
        "L7",
        "Which houses look at which",
        "the arrow table pp. 102–104 (crux LK-C7); yog drishti p. 112",
    );
    let rows: Vec<Vec<String>> = (1..=12_u8)
        .map(|house| {
            let looks: Vec<String> = looks_at(house)
                .iter()
                .map(|(to, strength)| format!("{to} ({}%)", strength.percent()))
                .collect();
            let yog = yog_drishti(house);
            let pair = |relation: Relation| format!("{} / {}", relation.sees, relation.seen_from);
            vec![
                format!("L7.{house}"),
                house.to_string(),
                if looks.is_empty() {
                    "—".to_owned()
                } else {
                    looks.join(", ")
                },
                pair(yog.help),
                pair(yog.general),
                pair(yog.collision),
                pair(yog.foundation),
                pair(yog.deceit),
                pair(yog.wall),
            ]
        })
        .collect();
    out += &grid(
        &[
            "id",
            "house",
            "looks at",
            "help",
            "general",
            "collision",
            "foundation",
            "deceit",
            "wall",
        ],
        &rows,
    );
    out += "\nEach yog drishti cell is the house seen, then the house it is seen from.\n";
    out
}

fn outputs() -> [Output; 2] {
    [
        Output::new(PAKSHI, pakshi_page()),
        Output::new(LALKITAB, lalkitab_page()),
    ]
}

pub(crate) fn generate(root: &Path) -> i32 {
    write(root, &outputs())
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    i32::from(check(root, &outputs(), "cargo xtask review-sheets") != 0)
}
