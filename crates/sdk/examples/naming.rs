//! Naming a child (nāmakaraṇa): the syllable a birth pada names a child
//! by, names checked against it, a Latin spelling read every way it can
//! be, a family's own book of names searched, and the clauses a name in
//! daily use is held to, said in English and in Nepali.
//! `cargo xtask check-rust` runs this file, and every binding's `naming`
//! prints these lines (`03-design/namakarana.md`).
//!
//! ```sh
//! cargo run --release -p teistro --example naming
//! ```
//!
//! What it is really teaching:
//!
//! 1. **Facts, not a score.** A check says whether a name begins in the
//!    birth star's row, in the pada's own cell, or by the patro's
//!    substitute letter, and what each clause of the gṛhyasūtras makes of
//!    it. Nothing adds them up: which of them matters is the family's.
//! 2. **A Latin spelling is read every way it can be.** `CASUAL` answers
//!    each cell a spelling reaches and says it is ambiguous, rather than
//!    choosing one.
//! 3. **The SDK ships no names.** A book is the consumer's, loaded once
//!    and named by its id; a name the cakra cannot read is listed, not
//!    dropped.
//! 4. **A clause is a key; the locale says it.** `sdk.naming`'s messages
//!    turn each key into words, so one answer reads in every locale.
//!
//! It reads no sky, so it names no ephemeris.

#![expect(clippy::print_stdout, reason = "an example is a program that prints")]

use teistro::catalogue::{Nakshatra, Sex};
use teistro::matching::{
    Birth, Book, BookName, LatinName, NameCheck, NameRules, NamingRules, SuggestFilter, Suggestions,
};
use teistro::{Context, Error, NamingArea, messages};

/// A key as the wire spells it: `SONANT_INITIAL`, `SUBSTITUTE`.
fn key<T: serde::Serialize>(member: &T) -> String {
    serde_json::to_value(member)
        .ok()
        .and_then(|value| value.as_str().map(String::from))
        .unwrap_or_default()
}

/// A flag as every binding prints it.
fn yes(flag: bool) -> &'static str {
    if flag { "yes" } else { "no" }
}

fn main() -> Result<(), Error> {
    let sdk = Context::builder().locale("en-Latn").build()?;
    let naming = sdk.naming();
    let rules = NamingRules::default();
    // The Moon in Ardra's third pada.
    let birth = Birth {
        nakshatra: Nakshatra::Ardra,
        pada: 3,
    };
    let syllable = the_syllable(naming, birth, rules)?;
    checked(naming, birth, rules)?;
    spelled(naming, rules)?;
    let found = the_book(naming, birth, rules)?;

    // ── Said in each locale ────────────────────────────────────────────
    // The answer holds keys; `sdk.naming`'s messages say them.
    let ganesh = naming
        .check(&["गणेश"], birth, rules)?
        .into_iter()
        .next()
        .ok_or_else(|| Error::internal("a batch of one has one answer"))??;
    for locale in ["en-Latn", "ne-Deva-NP"] {
        sdk.intl().set_locale(locale)?;
        println!("{locale}");
        said(&sdk, birth, &syllable, &ganesh, &found);
    }
    Ok(())
}

/// The birth's syllable. Ardra's third pada is ङ, which no name begins
/// with, so the Nepali patro glosses it with ग: a substitute, listed after
/// the cell's own letter.
fn the_syllable(naming: NamingArea<'_>, birth: Birth, rules: NamingRules) -> Result<String, Error> {
    let names = naming.of(birth, rules)?;
    let syllable = &names.syllable;
    println!(
        "{} {}: {} {}, deity {}",
        birth.nakshatra.full_key(),
        birth.pada,
        syllable.devanagari,
        syllable.iast,
        names.deity.full_key()
    );
    let initials: Vec<String> = syllable
        .initials
        .iter()
        .map(|initial| format!("{} {}", initial.devanagari, key(&initial.by)))
        .collect();
    println!("initials {}", initials.join(", "));
    Ok(syllable.devanagari.clone())
}

/// Names checked against the birth. Each is answered on its own: ऐ is no
/// cell of the cakra, and its refusal refuses no other name.
fn checked(naming: NamingArea<'_>, birth: Birth, rules: NamingRules) -> Result<(), Error> {
    let asked = ["गणेश", "चूडामणि", "ऐश्वर्य"];
    for (name, checked) in asked.into_iter().zip(naming.check(&asked, birth, rules)?) {
        match checked {
            Ok(check) => println!(
                "{name}: same star {}, same pada {}, by substitute {}, {} syllables",
                yes(check.same_star),
                yes(check.same_pada),
                yes(check.by_substitute),
                check.clauses.syllables
            ),
            Err(error) => println!("{name}: refused, {}", error.message),
        }
    }
    Ok(())
}

/// A Latin spelling, read every way `CASUAL` can read it.
fn spelled(naming: NamingArea<'_>, rules: NamingRules) -> Result<(), Error> {
    let casual = NamingRules {
        name: NameRules {
            latin: LatinName::Casual,
            ..NameRules::default()
        },
        ..rules
    };
    let spellings = ["Krishna", "Shiva"];
    for (name, read) in spellings.into_iter().zip(naming.read(&spellings, casual)) {
        let reading = read?;
        let cells: Vec<String> = reading
            .readings
            .iter()
            .map(|one| {
                let star = one
                    .syllable
                    .nakshatra
                    .map_or("Abhijit", |star| star.full_key());
                format!("{} {star} {}", one.spelled, one.syllable.quarter)
            })
            .collect();
        println!(
            "{name}: ambiguous {}, {}",
            yes(reading.ambiguous),
            cells.join(" | ")
        );
    }
    Ok(())
}

/// A family's own book. The SDK ships no names: a book is loaded once into
/// the context and named after by its id; औषधि begins with औ, no cell, and
/// is listed.
fn the_book(
    naming: NamingArea<'_>,
    birth: Birth,
    rules: NamingRules,
) -> Result<Suggestions, Error> {
    let name = |devanagari: &str, sex: Option<Sex>| BookName {
        devanagari: devanagari.to_owned(),
        sex,
        ..BookName::default()
    };
    let loaded = naming.load(Book {
        names: vec![
            name("गणेश", Some(Sex::Male)),
            name("गीता", Some(Sex::Female)),
            name("गोपाल", None),
            name("औषधि", None),
        ],
    })?;
    let refused: Vec<String> = loaded
        .refused
        .iter()
        .map(|one| format!("{} {}", one.index, one.error.field().unwrap_or_default()))
        .collect();
    println!(
        "a book of {} names, refused: {}",
        loaded.names,
        refused.join(", ")
    );
    // A boy's names: the book's boys and the names it gives no sex.
    let boys = SuggestFilter {
        sex: Some(Sex::Male),
        ..SuggestFilter::default()
    };
    let found = naming.suggest(loaded.book, birth, &boys, rules)?;
    let suggested: Vec<String> = found
        .names
        .iter()
        .map(|one| format!("{} {}", one.reading.spelled, key(&one.by)))
        .collect();
    println!("suggested for a boy: {}", suggested.join(", "));
    Ok(found)
}

/// One check said in the context's locale, by `sdk.naming`'s typed
/// messages.
fn said(sdk: &Context, birth: Birth, syllable: &str, ganesh: &NameCheck, found: &Suggestions) {
    let intl = sdk.intl();
    let pada = intl.render_typed(&messages::sdk::naming::Pada {
        nakshatra: birth.nakshatra,
        pada: i64::from(birth.pada),
        syllable: syllable.to_owned(),
    });
    println!("  {}", pada.text);
    let count = intl.render_typed(&messages::sdk::naming::Syllables {
        count: i64::from(ganesh.clauses.syllables),
    });
    let counted = intl.render_typed(&messages::sdk::naming::Counted {
        counted: key(&ganesh.clauses.counted),
    });
    println!("  गणेश: {}, {}", count.text, counted.text);
    for row in &ganesh.clauses.rows {
        let clause = intl.render_typed(&messages::sdk::naming::Clause {
            clause: key(&row.clause),
        });
        let holds = intl.render_typed(&messages::sdk::naming::Holds {
            holds: yes(row.holds).to_owned(),
        });
        let source = intl.render_typed(&messages::sdk::naming::Source {
            source: key(&row.source),
        });
        println!("    {}: {} ({})", clause.text, holds.text, source.text);
    }
    for one in &found.names {
        let by = intl.render_typed(&messages::sdk::naming::By { by: key(&one.by) });
        println!("  {}: {}", one.reading.spelled, by.text);
    }
}
