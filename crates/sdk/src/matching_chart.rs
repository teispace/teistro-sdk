//! Two founded charts matched through the Moon of each
//! (`03-design/matching.md`).

use teistro_core::catalogue::Graha;
use teistro_core::error::Error;
use teistro_matching::{AshtaKoota, KootaRules, Native, ashta_koota};
use teistro_serial::Document;

use crate::area::ChartArea;

impl ChartArea<'_> {
    /// The **Ashta Koota** of a bride's chart and a groom's (*Muhurta
    /// Chintamani* VI.21–34): each koota's points out of 36 and what it
    /// read, Bhakoot's dosha with its five exceptions as clauses, and
    /// Nadi's dosha. Never a verdict: the texts leave the judgement to the
    /// reader (`03-design/matching.md`).
    ///
    /// Each native is the chart's Moon, in its own sidereal zodiac. The
    /// bride and the groom are named because Varna and Gana read
    /// differently when they swap.
    ///
    /// ```no_run
    /// # use teistro::{Context, Document, Ephemeris, KootaRules};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let (bride, groom): (Document, Document) = todo!();
    /// let koota = sdk.chart().matching(&bride, &groom, KootaRules::default())?;
    /// for row in &koota.kootas {
    ///     println!("{:?}: {} of {}", row.reading, row.points, row.max_points);
    /// }
    /// println!("{} of 36", koota.total);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// A chart founded in the tropical zodiac, where a nakshatra means
    /// nothing, or one that does not place the Moon, named `bride` or
    /// `groom`.
    pub fn matching(
        self,
        bride: &Document,
        groom: &Document,
        rules: KootaRules,
    ) -> Result<AshtaKoota, Error> {
        Ok(ashta_koota(
            native(bride, "bride")?,
            native(groom, "groom")?,
            rules,
        ))
    }
}

/// A chart's Moon as matching reads it, refused by the role it was given.
fn native(chart: &Document, role: &str) -> Result<Native, Error> {
    let foundation = &chart.foundation;
    if !foundation.zodiac.is_sidereal() {
        return Err(Error::invalid_arg(format!(
            "the {role}'s chart is founded in the tropical zodiac, where a nakshatra means nothing"
        ))
        .with_field(role)
        .with_hint("found it under a sidereal profile, such as the default"));
    }
    let moon = foundation.graha(Graha::Moon).ok_or_else(|| {
        Error::invalid_arg(format!("the {role}'s chart does not place the Moon")).with_field(role)
    })?;
    Native::of_moon(moon.longitude_deg).map_err(|error| error.with_field(role))
}
