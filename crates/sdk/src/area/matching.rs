//! `sdk.matching`: what matches without a chart — two names, star to
//! star (`03-design/matching.md`, C291 to C296) — and a match's total
//! graded into bands (SR-39).

use teistro_core::error::Error;
use teistro_matching::{AshtaKoota, Graded, GunaBands, NaamMilan, NaamRules, naam_milan};

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

    /// A match's total graded by a table of bands (`03-design/matching.md`
    /// §8, SR-39): [`GunaBands::default`] is the rank 1 text's floor,
    /// *Muhurta Martanda*'s "above 18", and [`GunaBands::nepali_almanac`]
    /// the bands Nepal's almanacs print; a consumer's own table is a
    /// [`GunaBands`] read with [`GunaBands::from_json`]. The words for the
    /// band are the `sdk.matching.band` message's, `$band` the band's
    /// spelling, and `sdk.matching.graded` says the total with it.
    ///
    /// ```
    /// use teistro::{Value, params};
    /// use teistro::matching::{GunaBand, GunaBands, KootaRules, Native, ashta_koota};
    ///
    /// let sdk = teistro::Context::builder().build()?;
    /// // Two Moons in Rohini: 28 of 36.
    /// let moon = Native::of_moon(45.0)?;
    /// let koota = ashta_koota(moon, moon, KootaRules::default());
    /// let graded = sdk.matching().band(&koota, &GunaBands::nepali_almanac());
    /// assert_eq!(graded.band, GunaBand::Good);
    /// let said = sdk.intl().render(
    ///     "sdk.matching.graded",
    ///     &params([("total", Value::Num(graded.total)), ("band", Value::from("GOOD"))]),
    /// );
    /// assert_eq!(said.text, "28 of 36 gunas: a good match");
    /// # Ok::<(), teistro::Error>(())
    /// ```
    #[must_use]
    pub fn band(self, koota: &AshtaKoota, bands: &GunaBands) -> Graded {
        bands.grade(koota)
    }
}
