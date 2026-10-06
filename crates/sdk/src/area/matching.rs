//! `sdk.matching`: what matches without a chart — two names, star to
//! star (`03-design/matching.md`, C291 to C296).

use teistro_core::error::Error;
use teistro_matching::{NaamMilan, NaamRules, naam_milan};

use crate::context::Context;

/// `sdk.matching`: what matches without a chart.
///
/// A match of two **births** is asked of the charts, as the `matching`
/// beside a chart request, because it reads both Moons; a match of two
/// **names** reads no sky, and is asked here. The operations do not read
/// the context, and the area still takes one, as `sdk.frame()` does: what
/// a consumer holds is `sdk.matching()`, the same value in every binding.
#[derive(Clone, Copy, Debug)]
pub struct MatchingArea<'a> {
    context: &'a Context,
}

impl<'a> MatchingArea<'a> {
    pub(crate) fn of(context: &'a Context) -> MatchingArea<'a> {
        MatchingArea { context }
    }

    /// The context this area was read off.
    #[must_use]
    pub fn context(self) -> &'a Context {
        self.context
    }

    /// Two names matched star to star (naam milan): each name's first
    /// syllable in the śatapada cakra, the varga koota of *Muhurta
    /// Chintamani* VI.35, and the Ashta Koota and the ten considerations
    /// read from the two name stars, as a chart's match reads two Moons.
    ///
    /// ```
    /// use teistro::catalogue::Nakshatra;
    /// use teistro::matching::{NaamRules, VargaRelation};
    ///
    /// let sdk = teistro::Context::builder().build()?;
    /// let read = sdk.matching().naam("प्रिया", "कृष्ण", NaamRules::default())?;
    /// assert_eq!(read.bride.nakshatra, Some(Nakshatra::UttaraPhalguni));
    /// assert_eq!(read.varga.relation, VargaRelation::Enemy);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for a name the cakra does not read, a Latin name
    /// while `rules.name.latin` refuses, or an Abhijit syllable while
    /// `rules.name.abhijit` refuses, named under `bride` or `groom`.
    pub fn naam(self, bride: &str, groom: &str, rules: NaamRules) -> Result<NaamMilan, Error> {
        naam_milan(bride, groom, rules)
    }
}
