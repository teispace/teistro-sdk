//! A consumer's name book: loaded once, indexed by the cakra's cell, and
//! asked for the names a birth pada's syllable reaches
//! (`03-design/namakarana.md` §3). The SDK ships no names; the book is
//! the consumer's.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Sex;
use teistro_core::envelope::{Hash, content_hash};
use teistro_core::error::Error;

use super::cakra::{cells, is_devanagari};
use super::check::Birth;
use super::clauses::{Clause, name_clauses, reported};
use super::pada::{InitialBy, PadaSyllable, pada_of, reached_by, substitute_letter};
use super::read::{NameReading, NamingRules, Reading, read_name};

/// A name book as the consumer writes it: `{"names": [...]}`.
///
/// ```
/// use teistro_matching::Book;
///
/// let book: Book = serde_json::from_str(r#"{"names": [{"devanagari": "चूडामणि", "sex": "MALE"}]}"#)?;
/// assert_eq!(book.names[0].devanagari, "चूडामणि");
/// assert!(serde_json::from_str::<Book>(r#"{"names": [], "meaning": "x"}"#).is_err());
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Book {
    /// The names, in the book's order, which is the order a suggestion
    /// keeps within a cell.
    pub names: Vec<BookName>,
}

/// One name of a book.
///
/// ```
/// use teistro_matching::BookName;
///
/// let name: BookName = serde_json::from_str(r#"{"devanagari": "राम", "latin": "Ram", "tags": {"meaning.en": "pleasing"}}"#)?;
/// assert_eq!((name.latin.as_deref(), name.sex), (Some("Ram"), None));
/// assert!(serde_json::from_str::<BookName>(r#"{"devanagari": "राम", "sex": "BOY"}"#).is_err());
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BookName {
    /// The name in Devanagari, which the cakra reads.
    pub devanagari: String,
    /// A display spelling, carried and never read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latin: Option<String>,
    /// Whose name it is, when the book says.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sex: Option<Sex>,
    /// The book's own fields (a meaning, a source), carried and never
    /// read.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub tags: BTreeMap<String, String>,
}

/// A loaded book's identity: the SHA-256 of its canonical JSON, so the
/// same book loaded twice, however its JSON was spaced or its keys
/// ordered, is one book.
///
/// ```
/// use teistro_matching::{Book, NameBook};
///
/// let spaced: Book = serde_json::from_str(r#"{ "names": [ {"sex": "FEMALE", "devanagari": "सीता"} ] }"#)?;
/// let tight: Book = serde_json::from_str(r#"{"names":[{"devanagari":"सीता","sex":"FEMALE"}]}"#)?;
/// assert_eq!(NameBook::load(spaced)?.id(), NameBook::load(tight)?.id());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct BookId(pub Hash);

impl std::fmt::Display for BookId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A name the book holds that the cakra does not read, with why.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct RefusedName {
    /// Its place in the book.
    pub index: u32,
    /// The reader's refusal, named `book.names[index].devanagari`.
    pub error: Error,
}

/// A name the book suggests for a birth.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Suggested {
    /// Its place in the book.
    pub index: u32,
    /// The name read, with its clauses under the request's rules.
    pub reading: Reading,
    /// How it reaches the birth pada: by the cell's own letter, a paired
    /// letter, or the pada's substitute (C294, C770).
    pub by: InitialBy,
    /// Whether it begins in the birth pada's own cell; never for a name
    /// reached by substitute, which reads in its own.
    pub same_pada: bool,
}

/// The names a book suggests for a birth.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Suggestions {
    /// The book asked.
    pub book: BookId,
    /// The birth pada's syllable, with every initial that reaches it.
    pub prescribed: PadaSyllable,
    /// The names that reach it and pass the filter, in cakra order, then
    /// the book's.
    pub names: Vec<Suggested>,
    /// Every name the book holds that the cakra refused, in the book's
    /// order: listed, never dropped.
    pub refused: Vec<RefusedName>,
}

/// What a suggestion keeps. A filter never ranks; an empty one keeps
/// every name that reaches the pada.
///
/// ```
/// use teistro_core::catalogue::Sex;
/// use teistro_matching::{Clause, SuggestFilter};
///
/// let filter: SuggestFilter = serde_json::from_str(r#"{"sex": "FEMALE", "syllables": [3], "clauses": ["ODD_FOR_A_GIRL"]}"#)?;
/// assert_eq!(filter, SuggestFilter { sex: Some(Sex::Female), syllables: vec![3], clauses: vec![Clause::OddForAGirl] });
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct SuggestFilter {
    /// Keeps names of this sex and names the book gives no sex: a name
    /// the book leaves open is anyone's.
    pub sex: Option<Sex>,
    /// Keeps names of one of these syllable counts, counted as the
    /// rules' `finalA` says; empty keeps every count.
    pub syllables: Vec<u8>,
    /// Keeps names meeting every one of these clauses, each one the
    /// rules' sutra reports.
    pub clauses: Vec<Clause>,
}

/// A name the book holds that the cakra read.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Indexed {
    index: u32,
    sex: Option<Sex>,
    reading: Reading,
}

/// A loaded name book: every name read once into the cakra and indexed by
/// its cell, so a suggestion reads only the cells it asks for.
///
/// ```
/// use teistro_core::catalogue::Nakshatra;
/// use teistro_matching::{Birth, Book, BookName, NameBook, NamingRules, SuggestFilter};
///
/// let book = Book {
///     names: ["चूडामणि", "चेदीश", "चूडा", "ऐश्वर्य"]
///         .map(|name| BookName { devanagari: name.to_owned(), ..BookName::default() })
///         .to_vec(),
/// };
/// let book = NameBook::load(book)?;
/// // ऐ is no cell of the cakra: the name is listed, not dropped.
/// assert_eq!(book.refused()[0].index, 3);
/// let birth = Birth { nakshatra: Nakshatra::Ashwini, pada: 1 };
/// let found = book.suggest(birth, &SuggestFilter::default(), NamingRules::default())?;
/// let indices: Vec<u32> = found.names.iter().map(|name| name.index).collect();
/// assert_eq!(indices, [0, 2]);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NameBook {
    id: BookId,
    names: Vec<BookName>,
    /// The names the cakra read, by cell, each cell in the book's order.
    cells: BTreeMap<usize, Vec<Indexed>>,
    refused: Vec<RefusedName>,
}

impl NameBook {
    /// Loads a book: checks that every name holds Devanagari, reads each
    /// into the cakra once, and indexes it by its cell. A name the cakra
    /// does not read (an ai or au initial) is kept as refused, not
    /// failed; its Devanagari is read as Devanagari whatever the rules'
    /// `latin`, so the index is the same under every rule.
    ///
    /// ```
    /// use teistro_matching::{Book, BookName, NameBook};
    ///
    /// let latin = Book { names: vec![BookName { devanagari: "Ram".to_owned(), ..BookName::default() }] };
    /// assert_eq!(NameBook::load(latin).unwrap_err().field(), Some("book.names[0].devanagari"));
    /// ```
    ///
    /// # Errors
    ///
    /// A name with no Devanagari letter, named `book.names[i].devanagari`.
    pub fn load(book: Book) -> Result<NameBook, Error> {
        let id = BookId(content_hash(&book));
        let mut cells: BTreeMap<usize, Vec<Indexed>> = BTreeMap::new();
        let mut refused = Vec::new();
        for (index, name) in (0_u32..).zip(&book.names) {
            let field = format!("book.names[{index}].devanagari");
            if !name.devanagari.chars().any(is_devanagari) {
                return Err(Error::invalid_arg("a book's name has no Devanagari")
                    .with_field(field)
                    .with_hint(
                        "the cakra reads Devanagari; `latin` is a display spelling, never read",
                    ));
            }
            match read_name(&name.devanagari, NamingRules::default()).and_then(NameReading::one) {
                Ok(reading) => cells
                    .entry(usize::from(reading.syllable.cell))
                    .or_default()
                    .push(Indexed {
                        index,
                        sex: name.sex,
                        reading,
                    }),
                Err(error) => refused.push(RefusedName {
                    index,
                    error: error.with_field(field),
                }),
            }
        }
        Ok(NameBook {
            id,
            names: book.names,
            cells,
            refused,
        })
    }

    /// The book's identity.
    ///
    /// ```
    /// use teistro_matching::{Book, NameBook};
    ///
    /// assert_eq!(
    ///     NameBook::load(Book::default())?.id().to_string(),
    ///     teistro_core::envelope::content_hash(&Book::default()).to_string(),
    /// );
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn id(&self) -> BookId {
        self.id
    }

    /// The book's names, in its order, as loaded.
    ///
    /// ```
    /// use teistro_matching::{Book, BookName, NameBook};
    ///
    /// let one = Book { names: vec![BookName { devanagari: "सीता".to_owned(), ..BookName::default() }] };
    /// assert_eq!(NameBook::load(one)?.names()[0].devanagari, "सीता");
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn names(&self) -> &[BookName] {
        &self.names
    }

    /// The names the cakra refused, in the book's order.
    ///
    /// ```
    /// use teistro_matching::{Book, BookName, NameBook};
    ///
    /// let one = Book { names: vec![BookName { devanagari: "औषधि".to_owned(), ..BookName::default() }] };
    /// assert_eq!(NameBook::load(one)?.refused().len(), 1);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn refused(&self) -> &[RefusedName] {
        &self.refused
    }

    /// The book's names a birth pada's syllable reaches (C297, C294,
    /// C770): those in the pada's own cell, by its letter or a paired one,
    /// and those beginning with the substitute it lists for a letter no
    /// name begins with. Each comes with its clauses under `rules`, and
    /// the filter keeps some; the order is the cakra's, then the book's.
    ///
    /// ```
    /// use teistro_core::catalogue::Nakshatra;
    /// use teistro_matching::{Birth, Book, BookName, InitialBy, NameBook, NamingRules, SuggestFilter};
    ///
    /// let book = NameBook::load(Book {
    ///     names: ["गणेश", "गोपाल", "राम"]
    ///         .map(|name| BookName { devanagari: name.to_owned(), ..BookName::default() })
    ///         .to_vec(),
    /// })?;
    /// // Ardra's third pada is ङ, which the patro reads as ग.
    /// let ardra = Birth { nakshatra: Nakshatra::Ardra, pada: 3 };
    /// let found = book.suggest(ardra, &SuggestFilter::default(), NamingRules::default())?;
    /// let by: Vec<(u32, InitialBy)> = found.names.iter().map(|name| (name.index, name.by)).collect();
    /// assert_eq!(by, [(0, InitialBy::Substitute), (1, InitialBy::Substitute)]);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A `pada` outside 1 to 4, named `pada`; a clause in the filter that
    /// the rules' sutra does not report, named `filter.clauses`.
    pub fn suggest(
        &self,
        birth: Birth,
        filter: &SuggestFilter,
        rules: NamingRules,
    ) -> Result<Suggestions, Error> {
        let prescribed = pada_of(birth.nakshatra, birth.pada, rules)?;
        if filter
            .clauses
            .iter()
            .any(|&clause| reported(rules.sutra).all(|(of, _)| of != clause))
        {
            return Err(Error::invalid_arg(
                "the filter asks for a clause the rules' sutra does not report",
            )
            .with_field("filter.clauses")
            .with_hint(
                "EVEN_FOR_A_BOY is Ashvalayana's and LONG_A_FOR_A_GIRL Paraskara's: \
                 set sutra to the one that reports it",
            ));
        }
        let substitutes: Vec<char> = prescribed
            .initials
            .iter()
            .filter_map(substitute_letter)
            .collect();
        // The pada's own cell, and the cells a substitute's letter heads:
        // a name beginning with it reads in one of those, whatever its
        // vowel.
        let reached = cells().enumerate().filter(|(at, cell)| {
            *at == usize::from(prescribed.cell)
                || cell
                    .consonant()
                    .is_some_and(|letter| substitutes.contains(&letter))
        });
        let mut names = Vec::new();
        for (at, _) in reached {
            for name in self.cells.get(&at).into_iter().flatten() {
                let clauses = name_clauses(&name.reading.spelled, rules);
                let keep = filter
                    .sex
                    .is_none_or(|sex| name.sex.is_none_or(|own| own == sex))
                    && (filter.syllables.is_empty()
                        || filter.syllables.contains(&clauses.syllables))
                    && filter.clauses.iter().all(|&clause| {
                        clauses
                            .rows
                            .iter()
                            .any(|row| row.clause == clause && row.holds)
                    });
                if keep {
                    names.push(Suggested {
                        index: name.index,
                        by: reached_by(&prescribed, &name.reading.spelled),
                        same_pada: at == usize::from(prescribed.cell),
                        reading: Reading {
                            clauses,
                            ..name.reading.clone()
                        },
                    });
                }
            }
        }
        Ok(Suggestions {
            book: self.id,
            prescribed,
            names,
            refused: self.refused.clone(),
        })
    }
}
