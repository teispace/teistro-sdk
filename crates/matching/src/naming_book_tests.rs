//! A consumer's name book held to `03-design/namakarana.md` §3 and §5: a
//! name in every cell suggested in its own cell's request and nowhere
//! else, the substitutes, the filter, the refused names listed, and one
//! book one `BookId`.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index their own tables"
)]

use std::collections::BTreeMap;

use teistro_core::catalogue::{Nakshatra, Rashi, Sex};
use teistro_core::envelope::content_hash;

use crate::{
    Birth, Book, BookId, BookName, Clause, FinalA, GrhyaSutra, InitialBy, NameBook, NamingRules,
    NasalInitial, PadaSyllable, SuggestFilter, Suggestions, SyllablesOf, syllables,
};

fn named(devanagari: &str) -> BookName {
    BookName {
        devanagari: devanagari.to_owned(),
        ..BookName::default()
    }
}

fn book_of(names: &[&str]) -> Book {
    Book {
        names: names.iter().map(|name| named(name)).collect(),
    }
}

fn load(names: &[&str]) -> NameBook {
    NameBook::load(book_of(names)).unwrap()
}

fn suggest(
    book: &NameBook,
    nakshatra: Nakshatra,
    pada: u8,
    filter: &SuggestFilter,
    rules: NamingRules,
) -> Suggestions {
    book.suggest(Birth { nakshatra, pada }, filter, rules)
        .unwrap()
}

fn indices(found: &Suggestions) -> Vec<u32> {
    found.names.iter().map(|name| name.index).collect()
}

/// Every cell of the cakra, in order.
fn every_cell() -> Vec<PadaSyllable> {
    let mut all: Vec<PadaSyllable> = Rashi::ALL
        .into_iter()
        .flat_map(|rashi| syllables(SyllablesOf::Rashi { rashi }, NamingRules::default()).unwrap())
        .collect();
    all.sort_by_key(|syllable| syllable.cell);
    all
}

fn bare() -> NamingRules {
    NamingRules {
        nasal: NasalInitial::None,
        ..NamingRules::default()
    }
}

fn births() -> impl Iterator<Item = (Nakshatra, u8)> {
    Nakshatra::ALL
        .into_iter()
        .flat_map(|nakshatra| (1..=4).map(move |pada| (nakshatra, pada)))
}

#[test]
fn a_name_in_every_cell_is_suggested_in_its_own_cell_and_nowhere_else() {
    let cells = every_cell();
    // Each cell's printed syllable, then each paired initial (बा for वा).
    let mut names: Vec<(String, u8, InitialBy)> = cells
        .iter()
        .map(|cell| (cell.devanagari.clone(), cell.cell, InitialBy::Cell))
        .collect();
    for cell in &cells {
        for initial in cell.initials.iter().filter(|i| i.by == InitialBy::Paired) {
            names.push((initial.devanagari.clone(), cell.cell, InitialBy::Paired));
        }
    }
    let spelled: Vec<&str> = names.iter().map(|(name, _, _)| name.as_str()).collect();
    let book = load(&spelled);
    assert_eq!(book.refused().len(), 0);
    let mut seen: BTreeMap<u32, usize> = BTreeMap::new();
    for (nakshatra, pada) in births() {
        let found = suggest(&book, nakshatra, pada, &SuggestFilter::default(), bare());
        let own: Vec<u32> = (0_u32..)
            .zip(&names)
            .filter(|(_, (_, cell, _))| *cell == found.prescribed.cell)
            .map(|(index, _)| index)
            .collect();
        assert_eq!(indices(&found), own, "{nakshatra:?} {pada}");
        for name in &found.names {
            let (_, cell, by) = &names[name.index as usize];
            assert_eq!(
                (name.by, name.same_pada, name.reading.syllable.cell),
                (*by, true, *cell)
            );
            *seen.entry(name.index).or_default() += 1;
        }
    }
    // The 108 padas' cells and their paired letters, each once; Abhijit's
    // four are no birth's.
    assert_eq!(seen.len(), names.len() - 4);
    assert!(seen.values().all(|&count| count == 1));
    assert!(cells[76..80].iter().all(|cell| cell.pada.is_none()));
}

#[test]
fn a_nasals_pada_reaches_its_substitutes_rows_under_the_default_alone() {
    let cells = every_cell();
    let spelled: Vec<&str> = cells.iter().map(|cell| cell.devanagari.as_str()).collect();
    let book = load(&spelled);
    for (nakshatra, pada, letter) in [
        (Nakshatra::Ardra, 3, 'ग'),
        (Nakshatra::Hasta, 3, 'ड'),
        (Nakshatra::UttaraBhadrapada, 4, 'ज'),
    ] {
        let found = suggest(
            &book,
            nakshatra,
            pada,
            &SuggestFilter::default(),
            NamingRules::default(),
        );
        let (own, by_substitute): (Vec<_>, Vec<_>) =
            found.names.iter().partition(|name| name.same_pada);
        assert_eq!(own.len(), 1);
        assert_eq!(own[0].by, InitialBy::Cell);
        assert_ne!(by_substitute.len(), 0);
        for name in &by_substitute {
            assert_eq!(name.by, InitialBy::Substitute);
            assert!(
                name.reading.spelled.starts_with(letter),
                "{}",
                name.reading.spelled
            );
        }
        // Every cell whose syllable begins with the letter, and no other.
        let expected = cells
            .iter()
            .filter(|cell| cell.devanagari.starts_with(letter))
            .count();
        assert_eq!(by_substitute.len(), expected, "{letter}");
        // Cakra order: ascending cells.
        let order: Vec<u8> = found
            .names
            .iter()
            .map(|name| name.reading.syllable.cell)
            .collect();
        assert!(order.windows(2).all(|pair| pair[0] < pair[1]));
        let none = suggest(&book, nakshatra, pada, &SuggestFilter::default(), bare());
        assert_eq!(indices(&none), [u32::from(found.prescribed.cell)]);
    }
    // ज's row runs into Abhijit's: its जू, जे and जो are reached too.
    let uttara = suggest(
        &book,
        Nakshatra::UttaraBhadrapada,
        4,
        &SuggestFilter::default(),
        NamingRules::default(),
    );
    assert!(
        uttara
            .names
            .iter()
            .any(|name| name.reading.syllable.nakshatra.is_none())
    );
}

#[test]
fn a_cells_names_keep_the_books_order_and_say_how_they_reach_it() {
    // वा is Rohini's second pada; बा reaches it paired; अ is a vowel's cell.
    let book = load(&["बालक", "वासु", "राम", "वाणी", "अनिल"]);
    let found = suggest(
        &book,
        Nakshatra::Rohini,
        2,
        &SuggestFilter::default(),
        NamingRules::default(),
    );
    let by: Vec<(u32, InitialBy)> = found
        .names
        .iter()
        .map(|name| (name.index, name.by))
        .collect();
    assert_eq!(
        by,
        [
            (0, InitialBy::Paired),
            (1, InitialBy::Cell),
            (3, InitialBy::Cell)
        ]
    );
    assert!(found.names.iter().all(|name| name.same_pada));
    let krittika = suggest(
        &book,
        Nakshatra::Krittika,
        1,
        &SuggestFilter::default(),
        NamingRules::default(),
    );
    assert_eq!(
        krittika
            .names
            .iter()
            .map(|name| (name.index, name.by))
            .collect::<Vec<_>>(),
        [(4, InitialBy::Cell)]
    );
    // A pada no name reaches suggests nothing, and says what it asked.
    let empty = suggest(
        &book,
        Nakshatra::Revati,
        4,
        &SuggestFilter::default(),
        NamingRules::default(),
    );
    assert_eq!(empty.names.len(), 0);
    assert_eq!(empty.prescribed.nakshatra, Some(Nakshatra::Revati));
}

#[test]
fn the_filter_keeps_by_sex_count_and_clause_and_never_ranks() {
    let mut book = book_of(&["देवः", "देवी", "देविका", "देवदत्त"]);
    book.names[0].sex = Some(Sex::Male);
    book.names[1].sex = Some(Sex::Female);
    let book = NameBook::load(book).unwrap();
    // The birth whose pada is दे's cell.
    let cell = crate::read_name("देवः", NamingRules::default())
        .unwrap()
        .readings[0]
        .syllable
        .cell;
    let birth = births()
        .find(|&(nakshatra, pada)| crate::birth_syllable(nakshatra, pada).unwrap().cell == cell)
        .unwrap();
    let ask = |filter: SuggestFilter, rules: NamingRules| {
        indices(&suggest(&book, birth.0, birth.1, &filter, rules))
    };
    assert_eq!(
        ask(SuggestFilter::default(), NamingRules::default()),
        [0, 1, 2, 3]
    );
    // A name the book gives no sex is anyone's.
    let girls = SuggestFilter {
        sex: Some(Sex::Female),
        ..SuggestFilter::default()
    };
    assert_eq!(ask(girls, NamingRules::default()), [1, 2, 3]);
    let boys = SuggestFilter {
        sex: Some(Sex::Male),
        ..SuggestFilter::default()
    };
    assert_eq!(ask(boys, NamingRules::default()), [0, 2, 3]);
    // de-vaḥ 2, de-vī 2, de-vi-kā 3, de-va-da-tta 4; spoken, the last 3.
    let three = SuggestFilter {
        syllables: vec![3],
        ..SuggestFilter::default()
    };
    assert_eq!(ask(three.clone(), NamingRules::default()), [2]);
    let spoken = NamingRules {
        final_a: FinalA::Silent,
        ..NamingRules::default()
    };
    assert_eq!(ask(three, spoken), [2, 3]);
    let two_or_four = SuggestFilter {
        syllables: vec![2, 4],
        ..SuggestFilter::default()
    };
    assert_eq!(ask(two_or_four, NamingRules::default()), [0, 1, 3]);
    // Every clause asked must hold.
    let visarga = SuggestFilter {
        clauses: vec![Clause::VisargaFinal],
        ..SuggestFilter::default()
    };
    assert_eq!(ask(visarga, NamingRules::default()), [0]);
    let both = SuggestFilter {
        clauses: vec![Clause::SonantInitial, Clause::LongAForAGirl],
        ..SuggestFilter::default()
    };
    assert_eq!(ask(both, NamingRules::default()), [2]);
    let even = SuggestFilter {
        clauses: vec![Clause::EvenForABoy],
        ..SuggestFilter::default()
    };
    let ashvalayana = NamingRules {
        sutra: GrhyaSutra::Ashvalayana,
        ..NamingRules::default()
    };
    assert_eq!(ask(even, ashvalayana), [0, 1, 3]);
    // Each suggestion carries its clauses under the request's rules.
    let found = suggest(&book, birth.0, birth.1, &SuggestFilter::default(), spoken);
    assert!(
        found
            .names
            .iter()
            .all(|name| name.reading.clauses.counted == FinalA::Silent)
    );
}

#[test]
fn a_clause_the_sutra_does_not_report_is_refused_by_name() {
    let book = load(&["राम"]);
    let birth = Birth {
        nakshatra: Nakshatra::Chitra,
        pada: 3,
    };
    for (clause, sutra) in [
        (Clause::EvenForABoy, GrhyaSutra::Paraskara),
        (Clause::LongAForAGirl, GrhyaSutra::Ashvalayana),
    ] {
        let filter = SuggestFilter {
            clauses: vec![clause],
            ..SuggestFilter::default()
        };
        let rules = NamingRules {
            sutra,
            ..NamingRules::default()
        };
        let error = book.suggest(birth, &filter, rules).unwrap_err();
        assert_eq!(error.field(), Some("filter.clauses"));
        assert!(error.hint().unwrap().contains("sutra"));
    }
    // The clause of neither sutra is reported under both.
    let neither = SuggestFilter {
        clauses: vec![Clause::NoFinalLOrR],
        ..SuggestFilter::default()
    };
    assert_eq!(
        indices(
            &book
                .suggest(birth, &neither, NamingRules::default())
                .unwrap()
        ),
        [0]
    );
    let pada = book.suggest(
        Birth {
            nakshatra: Nakshatra::Chitra,
            pada: 0,
        },
        &SuggestFilter::default(),
        NamingRules::default(),
    );
    assert_eq!(pada.unwrap_err().field(), Some("pada"));
}

#[test]
fn a_refused_name_is_listed_and_a_name_without_devanagari_refuses_the_book() {
    let book = load(&["ऐश्वर्या", "राम", "Ram राम", "औषधि"]);
    let refused: Vec<(u32, Option<&str>)> = book
        .refused()
        .iter()
        .map(|refused| (refused.index, refused.error.field()))
        .collect();
    assert_eq!(
        refused,
        [
            (0, Some("book.names[0].devanagari")),
            (2, Some("book.names[2].devanagari")),
            (3, Some("book.names[3].devanagari")),
        ]
    );
    let found = suggest(
        &book,
        Nakshatra::Chitra,
        3,
        &SuggestFilter::default(),
        NamingRules::default(),
    );
    assert_eq!((indices(&found), found.refused.len()), (vec![1], 3));
    assert_eq!(book.names().len(), 4);
    for blank in ["", "  ", "Sita"] {
        let error = NameBook::load(book_of(&["सीता", blank])).unwrap_err();
        assert_eq!(error.field(), Some("book.names[1].devanagari"), "{blank:?}");
        assert!(error.hint().unwrap().contains("latin"));
    }
}

#[test]
fn one_book_is_one_book_id() {
    let book = book_of(&["राम", "सीता"]);
    let id = NameBook::load(book.clone()).unwrap().id();
    assert_eq!(NameBook::load(book.clone()).unwrap().id(), id);
    assert_eq!(id, BookId(content_hash(&book)));
    assert_eq!(id.to_string(), content_hash(&book).to_string());
    // The JSON's spacing and key order do not move it; a name, an order,
    // a sex or a tag does.
    let read: Book =
        serde_json::from_str(r#"{ "names": [ {"devanagari": "राम"}, {"devanagari": "सीता"} ] }"#)
            .unwrap();
    assert_eq!(NameBook::load(read).unwrap().id(), id);
    let mut tagged = book.clone();
    tagged.names[0]
        .tags
        .insert("meaning.en".to_owned(), "pleasing".to_owned());
    let mut sexed = book.clone();
    sexed.names[1].sex = Some(Sex::Female);
    let mut latin = book.clone();
    latin.names[0].latin = Some("Rama".to_owned());
    for other in [
        book_of(&["सीता", "राम"]),
        book_of(&["राम"]),
        tagged,
        sexed,
        latin,
    ] {
        assert_ne!(NameBook::load(other).unwrap().id(), id);
    }
    // Answered with the book it came from, as the hex the hash prints.
    let found = suggest(
        &NameBook::load(book).unwrap(),
        Nakshatra::Chitra,
        3,
        &SuggestFilter::default(),
        NamingRules::default(),
    );
    assert_eq!(serde_json::to_value(found.book).unwrap(), id.to_string());
}

#[test]
fn a_book_writes_and_reads_back_its_own_shape() {
    let mut book = book_of(&["सीता"]);
    book.names[0].latin = Some("Sita".to_owned());
    book.names[0].sex = Some(Sex::Female);
    book.names[0]
        .tags
        .insert("meaning.en".to_owned(), "furrow".to_owned());
    let text = serde_json::to_string(&book).unwrap();
    assert_eq!(
        text,
        r#"{"names":[{"devanagari":"सीता","latin":"Sita","sex":"FEMALE","tags":{"meaning.en":"furrow"}}]}"#
    );
    assert_eq!(serde_json::from_str::<Book>(&text).unwrap(), book);
    // Only the Devanagari is required, and an unknown field is refused.
    assert_eq!(
        serde_json::to_string(&book_of(&["राम"])).unwrap(),
        r#"{"names":[{"devanagari":"राम"}]}"#
    );
    assert!(serde_json::from_str::<Book>(r#"{"names":[{"latin":"Ram"}]}"#).is_err());
    assert!(
        serde_json::from_str::<Book>(r#"{"names":[{"devanagari":"राम","meaning":"x"}]}"#).is_err()
    );
    assert!(serde_json::from_str::<SuggestFilter>(r#"{"gender":"MALE"}"#).is_err());
}

#[test]
fn a_book_of_25000_names_loads_and_suggests_every_birth() {
    // Each cell's syllable with a two-letter tail, 25,000 in all.
    const TAILS: [&str; 16] = [
        "क", "ग", "त", "द", "न", "प", "म", "य", "र", "ल", "व", "स", "ह", "ज", "ब", "श",
    ];
    let cells = every_cell();
    let names: Vec<String> = (0..25_000_usize)
        .map(|at| {
            let cell = &cells[at % cells.len()];
            let tail = at / cells.len();
            format!(
                "{}{}{}",
                cell.devanagari,
                TAILS[tail % 16],
                TAILS[(tail / 16) % 16]
            )
        })
        .collect();
    let spelled: Vec<&str> = names.iter().map(String::as_str).collect();
    let book = load(&spelled);
    assert_eq!(book.refused().len(), 0);
    let abhijit = (0..25_000_usize)
        .filter(|at| (76..80).contains(&(at % cells.len())))
        .count();
    let suggested: usize = births()
        .map(|(nakshatra, pada)| {
            suggest(&book, nakshatra, pada, &SuggestFilter::default(), bare())
                .names
                .len()
        })
        .sum();
    assert_eq!(suggested, 25_000 - abhijit);
}
