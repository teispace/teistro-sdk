//! `sdk.numerology`: a name and a civil date, which read no sky
//! (`03-design/numerology.md`, C320 to C328).

use teistro_core::error::Error;
use teistro_numerology::{BirthDate, NumerologyRules, Profile, profile};

use crate::context::Context;

/// `sdk.numerology`: a name and a birth date under Balliett's letter
/// cycle and Cheiro's Chaldean table.
///
/// The operation does not read the context, and the area still takes
/// one, as `sdk.matching()` does: what a consumer holds is
/// `sdk.numerology()`, the same value in every binding.
#[derive(Clone, Copy, Debug)]
pub struct NumerologyArea<'a> {
    context: &'a Context,
}

impl<'a> NumerologyArea<'a> {
    pub(crate) fn of(context: &'a Context) -> NumerologyArea<'a> {
        NumerologyArea { context }
    }

    /// The context this area was read off.
    #[must_use]
    pub fn context(self) -> &'a Context {
        self.context
    }

    /// Everything numerology says of a name and a birth date: the name
    /// under both systems, Balliett's birth number and Cheiro's day and
    /// year, and, under [`NumerologyRules::baseline`] only, the baseline
    /// engine's own numbers.
    ///
    /// ```
    /// use teistro::numerology::{BirthDate, NumerologyRules};
    ///
    /// let sdk = teistro::Context::builder().build()?;
    /// let date = BirthDate::new(1872, 1, 17)?;
    /// let read = sdk.numerology().profile("Henry Elder", date, NumerologyRules::default())?;
    /// assert_eq!(read.pythagorean_name.reduction.number, 6);
    /// assert_eq!(read.chaldean_birth.birth.number, 8);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for a character outside A to Z while
    /// `rules.non_latin` refuses, or a name with no letter that counts,
    /// named under `name`.
    pub fn profile(
        self,
        name: &str,
        date: BirthDate,
        rules: NumerologyRules,
    ) -> Result<Profile, Error> {
        profile(name, date, &rules)
    }
}
