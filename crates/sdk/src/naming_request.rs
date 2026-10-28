//! The naming area as a binding asks it: each operation one JSON record,
//! read strictly and refused by its field under `naming`
//! (`03-design/namakarana.md` §4.2, §4.3).

use serde::{Deserialize, Serialize};
use teistro_core::envelope::Hash;
use teistro_core::error::Error;
use teistro_matching::{
    Birth, Book, BookId, NameBook, NameCheck, NameReading, Names, NamingRules, PadaSyllable,
    RefusedName, SuggestFilter, Suggestions, SyllablesOf,
};

use crate::Context;

/// The root every naming request is read and refused under.
const NAMING: &str = "naming";

/// A refusal of one name in a batch, named by its place in the request
/// (`naming.names[2]`) in place of the reader's `name`.
fn of_name(error: Error, index: usize) -> Error {
    error.with_field(format!("{NAMING}.names[{index}]"))
}

/// A refusal of the birth, named under `naming.birth`.
fn of_birth(error: Error) -> Error {
    error.under(&format!("{NAMING}.birth"))
}

/// The syllables a pada, a star or a sign names a child by.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NamingSyllablesRequest {
    /// What is asked: one pada, a star's four, or a sign's nine (and
    /// Abhijit's four for Makara).
    pub of: SyllablesOf,
    /// The rules; the sources' own when left out.
    #[serde(default)]
    pub rules: NamingRules,
}

/// Names to read into the cakra, each on its own.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NamingReadRequest {
    /// The names, in Devanagari or in the Latin scheme
    /// `rules.name.latin` declares.
    pub names: Vec<String>,
    /// The rules; the sources' own when left out.
    #[serde(default)]
    pub rules: NamingRules,
}

/// Names to check against a birth's pada, each on its own.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NamingCheckRequest {
    /// The names, in Devanagari or in the Latin scheme
    /// `rules.name.latin` declares.
    pub names: Vec<String>,
    /// The birth: the Moon's nakshatra and pada.
    pub birth: Birth,
    /// The rules; the sources' own when left out.
    #[serde(default)]
    pub rules: NamingRules,
}

/// A birth, for the names it gives.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NamingOfRequest {
    /// The birth: the Moon's nakshatra and pada.
    pub birth: Birth,
    /// The rules; the sources' own when left out.
    #[serde(default)]
    pub rules: NamingRules,
}

/// A birth and the book to suggest its names from: a book loaded into
/// the context, named by `bookId`, or one carried here as `book` for one
/// use.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NamingSuggestRequest {
    /// A book loaded into this context, by the id its load answered:
    /// sixty-four hex digits, held as text so that an id this context
    /// never answered is refused like one it did not load; exactly one of
    /// `bookId` and `book` is given.
    #[serde(default)]
    pub book_id: Option<String>,
    /// A book carried in the request, loaded for this one suggestion;
    /// exactly one of `bookId` and `book` is given.
    #[serde(default)]
    pub book: Option<Book>,
    /// The birth: the Moon's nakshatra and pada.
    pub birth: Birth,
    /// What a suggestion keeps; every name the pada reaches when left
    /// out.
    #[serde(default)]
    pub filter: SuggestFilter,
    /// The rules; the sources' own when left out.
    #[serde(default)]
    pub rules: NamingRules,
}

/// One name of a batch read: its reading, or why it was refused.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct NameRead {
    /// The name read; null when it was refused.
    pub reading: Option<NameReading>,
    /// The refusal, named `naming.names[i]`; null when it was read.
    pub refused: Option<Error>,
}

/// One name of a batch checked: its check, or why it was refused.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct NameChecked {
    /// The name checked; null when it was refused.
    pub check: Option<NameCheck>,
    /// The refusal, named `naming.names[i]`; null when it was checked.
    pub refused: Option<Error>,
}

/// A book loaded into a context.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct LoadedBook {
    /// Its id, which a suggestion names it by: the SHA-256 of its
    /// canonical JSON, so the same book loaded twice is one book.
    pub book: BookId,
    /// How many names it holds, refused ones among them.
    pub names: u32,
    /// Every name it holds that the cakra refused, in the book's order.
    pub refused: Vec<RefusedName>,
}

impl LoadedBook {
    pub(crate) fn of(book: &NameBook) -> LoadedBook {
        LoadedBook {
            book: book.id(),
            names: u32::try_from(book.names().len()).unwrap_or(u32::MAX),
            refused: book.refused().to_vec(),
        }
    }
}

impl NamingSyllablesRequest {
    /// The record a binding sends, as JSON: `of`, the syllables asked
    /// (`{"of": "PADA", "nakshatra", "pada"}`, `{"of": "NAKSHATRA",
    /// "nakshatra"}` or `{"of": "RASHI", "rashi"}`), and `rules`.
    ///
    /// ```
    /// use teistro::NamingSyllablesRequest;
    ///
    /// let sdk = teistro::Context::builder().build()?;
    /// let asked = NamingSyllablesRequest::from_json(r#"{"of": {"of": "NAKSHATRA", "nakshatra": "ASHWINI"}}"#)?;
    /// assert_eq!(asked.answer(&sdk)?.len(), 4);
    /// let typo = NamingSyllablesRequest::from_json(r#"{"of": {"of": "NAKSHATRA", "nakshatra": "ASHWINI"}, "rule": {}}"#)
    ///     .unwrap_err();
    /// assert_eq!(typo.field(), Some("naming.rule"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record or a key it does not
    /// read, named under `naming`.
    pub fn from_json(text: &str) -> Result<NamingSyllablesRequest, Error> {
        teistro_core::strict::read(text, NAMING)
    }

    /// The syllables, in cakra order.
    ///
    /// # Errors
    ///
    /// A `pada` outside 1 to 4, named `naming.of.pada`.
    pub fn answer(&self, context: &Context) -> Result<Vec<PadaSyllable>, Error> {
        context
            .naming()
            .syllables(self.of, self.rules)
            .map_err(|error| error.under(&format!("{NAMING}.of")))
    }
}

impl NamingReadRequest {
    /// The record a binding sends, as JSON: `names` and `rules`.
    ///
    /// ```
    /// use teistro::NamingReadRequest;
    ///
    /// let sdk = teistro::Context::builder().build()?;
    /// let asked = NamingReadRequest::from_json(r#"{"names": ["राम", "Rama"]}"#)?;
    /// let read = asked.answer(&sdk);
    /// // A Latin name while `latin` refuses is that name's refusal alone.
    /// assert!(read[0].reading.is_some());
    /// assert_eq!(read[1].refused.as_ref().and_then(|error| error.field()), Some("naming.names[1]"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record or a key it does not
    /// read, named under `naming`.
    pub fn from_json(text: &str) -> Result<NamingReadRequest, Error> {
        teistro_core::strict::read(text, NAMING)
    }

    /// Each name read or refused, in the request's order; the batch never
    /// fails as a whole.
    #[must_use]
    pub fn answer(&self, context: &Context) -> Vec<NameRead> {
        let names: Vec<&str> = self.names.iter().map(String::as_str).collect();
        (0..)
            .zip(context.naming().read(&names, self.rules))
            .map(|(index, one)| match one {
                Ok(reading) => NameRead {
                    reading: Some(reading),
                    refused: None,
                },
                Err(error) => NameRead {
                    reading: None,
                    refused: Some(of_name(error, index)),
                },
            })
            .collect()
    }
}

impl NamingCheckRequest {
    /// The record a binding sends, as JSON: `names`, `birth` and `rules`.
    ///
    /// ```
    /// use teistro::NamingCheckRequest;
    ///
    /// let sdk = teistro::Context::builder().build()?;
    /// let asked = NamingCheckRequest::from_json(
    ///     r#"{"names": ["चूडामणि"], "birth": {"nakshatra": "ASHWINI", "pada": 1}}"#,
    /// )?;
    /// assert!(asked.answer(&sdk)?[0].check.as_ref().is_some_and(|check| check.same_pada));
    /// let wrong = NamingCheckRequest::from_json(r#"{"names": [], "birth": {"nakshatra": "ASHWINI", "pada": 5}}"#)?;
    /// assert_eq!(wrong.answer(&sdk).unwrap_err().field(), Some("naming.birth.pada"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record or a key it does not
    /// read, named under `naming`.
    pub fn from_json(text: &str) -> Result<NamingCheckRequest, Error> {
        teistro_core::strict::read(text, NAMING)
    }

    /// Each name checked or refused, in the request's order.
    ///
    /// # Errors
    ///
    /// A `pada` outside 1 to 4, named `naming.birth.pada`, which refuses
    /// the batch: every name is read against it.
    pub fn answer(&self, context: &Context) -> Result<Vec<NameChecked>, Error> {
        let names: Vec<&str> = self.names.iter().map(String::as_str).collect();
        let checked = context
            .naming()
            .check(&names, self.birth, self.rules)
            .map_err(of_birth)?;
        Ok((0..)
            .zip(checked)
            .map(|(index, one)| match one {
                Ok(check) => NameChecked {
                    check: Some(check),
                    refused: None,
                },
                Err(error) => NameChecked {
                    check: None,
                    refused: Some(of_name(error, index)),
                },
            })
            .collect())
    }
}

impl NamingOfRequest {
    /// The record a binding sends, as JSON: `birth` and `rules`.
    ///
    /// ```
    /// use teistro::NamingOfRequest;
    /// use teistro::catalogue::Deity;
    ///
    /// let sdk = teistro::Context::builder().build()?;
    /// let asked = NamingOfRequest::from_json(r#"{"birth": {"nakshatra": "KRITTIKA", "pada": 1}}"#)?;
    /// assert_eq!(asked.answer(&sdk)?.deity, Deity::Agni);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record or a key it does not
    /// read, named under `naming`.
    pub fn from_json(text: &str) -> Result<NamingOfRequest, Error> {
        teistro_core::strict::read(text, NAMING)
    }

    /// The names the birth gives.
    ///
    /// # Errors
    ///
    /// A `pada` outside 1 to 4, named `naming.birth.pada`.
    pub fn answer(&self, context: &Context) -> Result<Names, Error> {
        context
            .naming()
            .of(self.birth, self.rules)
            .map_err(of_birth)
    }
}

impl NamingSuggestRequest {
    /// The record a binding sends, as JSON: `bookId` or `book`, `birth`,
    /// `filter` and `rules`.
    ///
    /// ```
    /// use teistro::NamingSuggestRequest;
    ///
    /// let refused = NamingSuggestRequest::from_json(
    ///     r#"{"book": {"names": [{"devanagari": "राम", "sex": "BOY"}]}, "birth": {"nakshatra": "CHITRA", "pada": 3}}"#,
    /// )
    /// .unwrap_err();
    /// assert_eq!(refused.field(), Some("naming.book.names[0].sex"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record or a key it does not
    /// read, named under `naming`.
    pub fn from_json(text: &str) -> Result<NamingSuggestRequest, Error> {
        teistro_core::strict::read(text, NAMING)
    }

    /// The names the book suggests for the birth.
    ///
    /// ```
    /// use teistro::NamingSuggestRequest;
    ///
    /// let sdk = teistro::Context::builder().build()?;
    /// let asked = NamingSuggestRequest::from_json(
    ///     r#"{"book": {"names": [{"devanagari": "राम"}, {"devanagari": "सीता"}]}, "birth": {"nakshatra": "CHITRA", "pada": 3}}"#,
    /// )?;
    /// assert_eq!(asked.answer(&sdk)?.names[0].index, 0);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Neither or both of `bookId` and `book`, named `naming.book` or
    /// `naming.bookId`; a book not loaded in `context`, named
    /// `naming.bookId`; a carried book's name with no Devanagari, named
    /// `naming.book.names[i].devanagari`; a `pada` outside 1 to 4, named
    /// `naming.birth.pada`; a filter clause the sutra does not report,
    /// named `naming.filter.clauses`.
    pub fn answer(&self, context: &Context) -> Result<Suggestions, Error> {
        let suggested = match (&self.book_id, &self.book) {
            (Some(_), Some(_)) => {
                return Err(Error::invalid_arg("both `bookId` and `book` are given")
                    .with_field(format!("{NAMING}.bookId"))
                    .with_hint(
                        "name a loaded book by `bookId`, or carry one as `book`, not both",
                    ));
            }
            (None, None) => {
                return Err(Error::invalid_arg("no book is given")
                    .with_field(format!("{NAMING}.book"))
                    .with_hint("name a loaded book by `bookId`, or carry one as `book`"));
            }
            (Some(id), None) => book_id(id).and_then(|id| {
                context
                    .naming()
                    .suggest(id, self.birth, &self.filter, self.rules)
            }),
            (None, Some(book)) => NameBook::load(book.clone())
                .and_then(|book| book.suggest(self.birth, &self.filter, self.rules)),
        };
        suggested.map_err(|error| match error.field() {
            Some("pada") => of_birth(error),
            _ => error.under(NAMING),
        })
    }
}

/// The id `text` spells, refused as `bookId` when it is not sixty-four
/// hex digits.
fn book_id(text: &str) -> Result<BookId, Error> {
    Hash::from_hex(text).map(BookId).ok_or_else(|| {
        Error::invalid_arg(format!("`{text}` is no book's id"))
            .with_field("bookId")
            .with_hint("a book's id is the sixty-four hex digits its load answered")
    })
}

/// A book to load into the context: `{"book": {"names": [...]}}`.
///
/// Not a record of the agent server's, whose contexts are a cache that
/// keeps no state between calls: there a suggestion carries its book.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NamingLoadRequest {
    /// The book.
    pub book: Book,
}

impl NamingLoadRequest {
    /// The record a binding sends, as JSON: `book`.
    ///
    /// ```
    /// use teistro::NamingLoadRequest;
    ///
    /// let refused = NamingLoadRequest::from_json(r#"{"book": {"names": [{"devanagari": "राम", "sexx": "MALE"}]}}"#)
    ///     .unwrap_err();
    /// assert_eq!(refused.field(), Some("naming.book.names[0].sexx"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record or a key it does not
    /// read, named under `naming`.
    pub fn from_json(text: &str) -> Result<NamingLoadRequest, Error> {
        teistro_core::strict::read(text, NAMING)
    }

    /// The book loaded into `context`, kept there under its id.
    ///
    /// ```
    /// use teistro::NamingLoadRequest;
    ///
    /// let sdk = teistro::Context::builder().build()?;
    /// let asked = NamingLoadRequest::from_json(r#"{"book": {"names": [{"devanagari": "राम"}]}}"#)?;
    /// let loaded = asked.answer(&sdk)?;
    /// // The same book again is the same book.
    /// assert_eq!(asked.answer(&sdk)?.book, loaded.book);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A name with no Devanagari letter, named
    /// `naming.book.names[i].devanagari`.
    pub fn answer(&self, context: &Context) -> Result<LoadedBook, Error> {
        context
            .naming()
            .load(self.book.clone())
            .map_err(|error| error.under(NAMING))
    }
}
