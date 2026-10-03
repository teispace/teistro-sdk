//! A body's returns to its own natal place (`03-design/western-returns.md`).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_gochar::hits::{HitEvent, Motion, NatalPoint};
use teistro_serial::Document;

use crate::Envelope;
use crate::area::ChartArea;
use crate::hit_request::HitRequest;

/// One return: a graha back on the longitude it held at birth.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BodyReturn {
    /// Whose return.
    pub graha: Graha,
    /// When, a UTC Julian day: the chart of the return is founded here.
    pub at: JulianDay<Utc>,
    /// Which way the graha was moving. A graha that turns retrograde over
    /// its natal place returns up to three times, each a row.
    pub motion: Motion,
}

impl ChartArea<'_> {
    /// Every **return** of each graha asked between two instants: the
    /// instants it comes back to the longitude it held in `birth` (Morin,
    /// *Astrologia Gallica* XXIII, p. 633), the longitude alone (C255), in
    /// the chart's own zodiac (C256) and on the centre the settings name
    /// (C257). The Moon's is the lunar return, the Sun's the solar.
    ///
    /// One search for every graha: it is the hit list asked for each
    /// graha's conjunction with its own natal place
    /// ([`HitRequest::returns`]), a graha crossing another's natal place
    /// dropped. The figure is the chart founded at a return's instant,
    /// wherever the native is (C258).
    ///
    /// ```no_run
    /// # use teistro::{ChartRequest, Context, Document, Ephemeris, UtcOffset};
    /// # use teistro::catalogue::Graha;
    /// # use teistro::quantity::{JulianDay, Place, Utc};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let (birth, from, to, residence): (Document, JulianDay<Utc>, JulianDay<Utc>, Place) = todo!();
    /// let returns = sdk.chart().returns(&birth, from, to, [Graha::Moon])?.value;
    /// for one in &returns {
    ///     // Erected where the native is: the birth place, or anywhere else.
    ///     let at_home = sdk.chart().reading(one.at, &ChartRequest::at(birth.foundation.place, UtcOffset::UTC))?;
    ///     let away = sdk.chart().reading(one.at, &ChartRequest::at(residence, UtcOffset::UTC))?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// What [`HitRequest::check`] refuses (a window that does not run
    /// forward, no graha, a graha named twice); whatever the search
    /// refuses.
    pub fn returns(
        self,
        birth: &Document,
        from: JulianDay<Utc>,
        to: JulianDay<Utc>,
        grahas: impl IntoIterator<Item = Graha>,
    ) -> Result<Envelope<Vec<BodyReturn>>, Error> {
        let found = self.hits(birth, &HitRequest::returns(from, to, grahas))?;
        let provenance = found.provenance.clone();
        let returns = found
            .value
            .into_iter()
            .filter_map(|hit| match hit.event {
                HitEvent::Aspect {
                    to: NatalPoint::Graha { graha },
                    motion,
                    ..
                } if graha == hit.graha => Some(BodyReturn {
                    graha,
                    at: hit.instant,
                    motion,
                }),
                _ => None,
            })
            .collect();
        Ok(Envelope::sealing(returns, provenance))
    }
}
