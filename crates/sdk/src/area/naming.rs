//! `sdk.naming`: the naming of a child (nāmakaraṇa), which reads no sky
//! (`03-design/namakarana.md`, C291 to C299, C334, C770 to C775).

use teistro_core::error::Error;
use teistro_matching::{
    Birth, Book, BookId, NameBook, NameCheck, NameReading, Names, NamingRules, PadaSyllable,
    SuggestFilter, Suggestions, SyllablesOf, name_check, names_of, read_name, syllables,
};

use crate::context::Context;
use crate::naming_request::LoadedBook;

/// `sdk.naming`: the syllables a pada, a star or a sign names a child
/// by, names read into the śatapada cakra and checked against a birth,
/// the names a birth gives, and the names a consumer's own book suggests.
///
/// Every operation but [`NamingArea::load`] and a suggestion from a
/// loaded book is a pure function of its arguments. A book is loaded
/// once into the context, indexed by the cakra's cell, and named after by
/// its [`BookId`]; the SDK ships no names (§3).
#[derive(Clone, Copy, Debug)]
pub struct NamingArea<'a> {
    context: &'a Context,
}

impl<'a> NamingArea<'a> {
    pub(crate) fn new(context: &'a Context) -> NamingArea<'a> {
        NamingArea { context }
    }

    /// The context this area was read off.
    #[must_use]
    pub fn context(self) -> &'a Context {
        self.context
    }

    /// The syllables a pada, a star or a sign names a child by, in cakra
    /// order, each with every initial that reaches it (C297, C770, C771).
    ///
    /// ```
    /// use teistro::catalogue::Rashi;
    /// use teistro::matching::{NamingRules, SyllablesOf};
    ///
    /// let sdk = teistro::Context::builder().build()?;
    /// // Makara's nine padas, and Abhijit's four placed in it.
    /// let makara = sdk.naming().syllables(SyllablesOf::Rashi { rashi: Rashi::Capricorn }, NamingRules::default())?;
    /// assert_eq!(makara.len(), 13);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A `pada` outside 1 to 4, named `pada`.
    pub fn syllables(
        self,
        of: SyllablesOf,
        rules: NamingRules,
    ) -> Result<Vec<PadaSyllable>, Error> {
        syllables(of, rules)
    }

    /// Names read into the cakra, each on its own: one name refused never
    /// refuses the others (C291 to C294, C772, C774).
    ///
    /// ```
    /// use teistro::matching::NamingRules;
    ///
    /// let sdk = teistro::Context::builder().build()?;
    /// let read = sdk.naming().read(&["राम", "ऐश्वर्य"], NamingRules::default());
    /// assert_eq!(read[0].as_ref().map(|name| name.readings.len()), Ok(1));
    /// // ऐ is no cell of the cakra.
    /// assert_eq!(read[1].as_ref().unwrap_err().field(), Some("name"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    #[must_use]
    pub fn read(self, names: &[&str], rules: NamingRules) -> Vec<Result<NameReading, Error>> {
        names.iter().map(|name| read_name(name, rules)).collect()
    }

    /// Names read against a birth's pada, each on its own: where each
    /// begins, the pada's own syllable, whether they share a star or a
    /// pada or the name took the pada's substitute, and the clauses
    /// (C334, C770, C774). Facts, not a score.
    ///
    /// ```
    /// use teistro::catalogue::Nakshatra;
    /// use teistro::matching::{Birth, NamingRules};
    ///
    /// let sdk = teistro::Context::builder().build()?;
    /// let birth = Birth { nakshatra: Nakshatra::Ashwini, pada: 1 };
    /// let checked = sdk.naming().check(&["चूडामणि", "चेदीश"], birth, NamingRules::default())?;
    /// let same: Vec<bool> = checked.iter().map(|one| one.as_ref().is_ok_and(|check| check.same_pada)).collect();
    /// assert_eq!(same, [true, false]);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A `pada` outside 1 to 4, named `pada`, which refuses the whole
    /// batch since every name is read against it; a name the cakra does
    /// not read is that name's refusal alone, named `name`.
    pub fn check(
        self,
        names: &[&str],
        birth: Birth,
        rules: NamingRules,
    ) -> Result<Vec<Result<NameCheck, Error>>, Error> {
        syllables(
            SyllablesOf::Pada {
                nakshatra: birth.nakshatra,
                pada: birth.pada,
            },
            rules,
        )?;
        Ok(names
            .iter()
            .map(|name| name_check(name, birth, rules))
            .collect())
    }

    /// The names a birth gives, of *Dharmasindhu*'s four that the SDK can
    /// compute (C773): the pada's syllable and the star's deity.
    ///
    /// ```
    /// use teistro::catalogue::{Deity, Nakshatra};
    /// use teistro::matching::{Birth, NamingRules};
    ///
    /// let sdk = teistro::Context::builder().build()?;
    /// let names = sdk.naming().of(Birth { nakshatra: Nakshatra::Krittika, pada: 1 }, NamingRules::default())?;
    /// assert_eq!(names.deity, Deity::Agni);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A `pada` outside 1 to 4, named `pada`.
    pub fn of(self, birth: Birth, rules: NamingRules) -> Result<Names, Error> {
        names_of(birth, rules)
    }

    /// Loads a consumer's name book into the context: every name read
    /// once into its cell, kept under the book's [`BookId`] for
    /// [`NamingArea::suggest`]. The same book loaded again is the book
    /// already kept.
    ///
    /// ```
    /// use teistro::matching::{Book, BookName};
    ///
    /// let sdk = teistro::Context::builder().build()?;
    /// let names = ["चूडामणि", "औषधि"].map(|name| BookName { devanagari: name.to_owned(), ..BookName::default() });
    /// let loaded = sdk.naming().load(Book { names: names.to_vec() })?;
    /// // औ is no cell of the cakra: listed, not dropped.
    /// assert_eq!((loaded.names, loaded.refused[0].index), (2, 1));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A name with no Devanagari letter, named
    /// `book.names[i].devanagari`.
    pub fn load(self, book: Book) -> Result<LoadedBook, Error> {
        let kept = self.context.keep_book(NameBook::load(book)?);
        Ok(LoadedBook::of(&kept))
    }

    /// The names a loaded book suggests for a birth: those its pada's
    /// syllable reaches, by its own letter, a paired one or its
    /// substitute, in cakra order and then the book's, each with its
    /// clauses; and every name the book holds that the cakra refused.
    ///
    /// ```
    /// use teistro::catalogue::Nakshatra;
    /// use teistro::matching::{Birth, Book, BookName, NamingRules, SuggestFilter};
    ///
    /// let sdk = teistro::Context::builder().build()?;
    /// let names = ["चूडामणि", "राम"].map(|name| BookName { devanagari: name.to_owned(), ..BookName::default() });
    /// let book = sdk.naming().load(Book { names: names.to_vec() })?.book;
    /// let birth = Birth { nakshatra: Nakshatra::Ashwini, pada: 1 };
    /// let found = sdk.naming().suggest(book, birth, &SuggestFilter::default(), NamingRules::default())?;
    /// assert_eq!(found.names[0].index, 0);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A book this context has not loaded, named `bookId`; a `pada`
    /// outside 1 to 4, named `pada`; a clause the rules' sutra does not
    /// report, named `filter.clauses`.
    pub fn suggest(
        self,
        book: BookId,
        birth: Birth,
        filter: &SuggestFilter,
        rules: NamingRules,
    ) -> Result<Suggestions, Error> {
        let loaded = self.context.book(book).ok_or_else(|| {
            Error::invalid_arg(format!("no book `{book}` is loaded in this context"))
                .with_field("bookId")
                .with_hint(
                    "load the book into this context first, or carry it in the request as `book`",
                )
        })?;
        loaded.suggest(birth, filter, rules)
    }
}
