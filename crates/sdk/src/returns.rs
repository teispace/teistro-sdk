//! A body's returns to its own natal place, and to a place an offset past
//! it: the half- and quarter-returns (`03-design/western-returns.md`).

use serde::{Deserialize, Serialize};
use teistro_astro::events::Lattice;
use teistro_chart::foundation::TransitEventKind;
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::time::UtcOffset;
use teistro_gochar::hits::{HitEvent, Motion, NatalPoint};
use teistro_serial::Document;

use crate::Envelope;
use crate::area::ChartArea;
use crate::hit_request::{HitRequest, motion, repeated};

/// One return: a graha back on the longitude it held at birth.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct BodyReturn {
    /// Whose return.
    pub graha: Graha,
    /// When, a UTC Julian day: the chart of the return is founded here.
    pub at: JulianDay<Utc>,
    /// Which way the graha was moving. A graha that turns retrograde over
    /// its natal place returns up to three times, each a row.
    pub motion: Motion,
}

/// One return at an offset: a graha come to the longitude `offset_deg`
/// past the one it held at birth, forward in the zodiac (C434). The
/// demi-solar return is the Sun's at 180°, the quarti-solar returns its
/// at 90°, 180° and 270° (C433); offset 0 is the return itself.
///
/// ```
/// use teistro::OffsetReturn;
///
/// assert_eq!(OffsetReturn::HALF, [180]);
/// assert_eq!(OffsetReturn::QUARTERS, [90, 180, 270]);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct OffsetReturn {
    /// Whose return.
    pub graha: Graha,
    /// How far past the natal longitude, whole degrees from 0 to 359,
    /// forward in the zodiac: 90 is the graha a quarter of the circle
    /// ahead of its natal place, 270 a quarter behind it.
    pub offset_deg: u16,
    /// When, a UTC Julian day: the chart of the return is founded here.
    pub at: JulianDay<Utc>,
    /// Which way the graha was moving. A graha that turns retrograde over
    /// the line reaches it up to three times, each a row.
    pub motion: Motion,
}

impl OffsetReturn {
    /// The half-return's offset: the demi-solar return when asked of the
    /// Sun, the demi-lunar of the Moon (C433, C435).
    pub const HALF: [u16; 1] = [180];

    /// The three quarter-returns' offsets, in the order the circle passes
    /// them: the quarti-solar returns when asked of the Sun (C433). With
    /// offset 0 they are the year in four.
    pub const QUARTERS: [u16; 3] = [90, 180, 270];
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
    /// # let sdk = Context::builder().ephemeris([Ephemeris::None]).build()?;
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

    /// Every return of each graha asked between two instants **at each
    /// offset asked**: the instants it reaches the longitude `offset` past
    /// the one it held in `birth`, forward in the zodiac (C434). The Sun's
    /// at [`OffsetReturn::HALF`] is the demi-solar return, at
    /// [`OffsetReturn::QUARTERS`] the quarti-solar returns (C433), and the
    /// Moon's the same of a month (C435). Offset 0 is
    /// [`ChartArea::returns`], instant for instant.
    ///
    /// The line is read as a return's own is: longitude alone, in the
    /// chart's own zodiac and on the centre the settings name (C255 to
    /// C257, C437). It is searched as a line of its own, so a quarter
    /// ahead of the natal place is never taken for a quarter behind it,
    /// as an aspect's angle, 0 to 180, would take it. Rows are in instant
    /// order, then graha, then offset.
    ///
    /// ```no_run
    /// # use teistro::{ChartRequest, Context, Document, Ephemeris, OffsetReturn, UtcOffset};
    /// # use teistro::catalogue::Graha;
    /// # use teistro::quantity::{JulianDay, Utc};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::None]).build()?;
    /// # let (birth, from, to): (Document, JulianDay<Utc>, JulianDay<Utc>) = todo!();
    /// // The solar year in four: the return and the three quarti-solars.
    /// let offsets = [0].into_iter().chain(OffsetReturn::QUARTERS);
    /// let quarters = sdk.chart().returns_at(&birth, from, to, [Graha::Sun], offsets)?.value;
    /// for one in &quarters {
    ///     // Erected where the native is, as a return's figure is (C258).
    ///     let figure = sdk.chart().reading(one.at, &ChartRequest::at(birth.foundation.place, UtcOffset::UTC))?;
    ///     println!("{}° at {}", one.offset_deg, one.at.get());
    /// }
    /// // A refusal names the field and says what to write instead.
    /// let full_circle = sdk.chart().returns_at(&birth, from, to, [Graha::Sun], [360]).unwrap_err();
    /// assert_eq!(full_circle.field(), Some("offsets"));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// What [`ChartArea::returns`] refuses, by the same fields; no offset,
    /// an offset named twice, or one of 360 or more, `INVALID_ARG` naming
    /// `offsets` (C436); a graha the birth chart was not founded with,
    /// naming `grahas`; whatever the search refuses.
    pub fn returns_at(
        self,
        birth: &Document,
        from: JulianDay<Utc>,
        to: JulianDay<Utc>,
        grahas: impl IntoIterator<Item = Graha>,
        offsets: impl IntoIterator<Item = u16>,
    ) -> Result<Envelope<Vec<OffsetReturn>>, Error> {
        let grahas: Vec<Graha> = grahas.into_iter().collect();
        let offsets: Vec<u16> = offsets.into_iter().collect();
        HitRequest::returns(from, to, grahas.iter().copied()).check()?;
        check_offsets(&offsets)?;
        // One lattice per graha and offset, its step the whole circle, so
        // each holds its one line: the one-sided target that an aspect's
        // lattice, symmetric about the natal place, cannot hold.
        let mut lines: Vec<(Graha, u16)> = Vec::with_capacity(grahas.len() * offsets.len());
        let mut lattices: Vec<Lattice> = Vec::with_capacity(lines.capacity());
        for graha in &grahas {
            let natal_deg = birth
                .foundation
                .graha(*graha)
                .map(|at| at.longitude_deg)
                .ok_or_else(|| {
                    Error::invalid_arg(format!(
                        "the birth chart has no {} to return to",
                        graha.key()
                    ))
                    .with_field("grahas")
                    .with_hint(
                        "an outer planet is in a chart founded `with_outer_planets`; found the birth chart with it",
                    )
                })?;
            for offset in &offsets {
                lines.push((*graha, *offset));
                lattices.push(Lattice {
                    origin_deg: (natal_deg + f64::from(*offset)).rem_euclid(360.0),
                    step_deg: 360.0,
                });
            }
        }
        // The hit list's own search for one chart: the birth place, which
        // is the observer's under a topocentric frame.
        let place = birth.foundation.place;
        let found = self.founding(UtcOffset::UTC, |founder| {
            founder.transit_events(&place, &grahas, &lattices, false, (from, to))
        })?;
        let mut returns: Vec<OffsetReturn> = found
            .value
            .iter()
            .filter_map(|event| match event.kind {
                TransitEventKind::Crossing {
                    lattice, direction, ..
                } => {
                    let (graha, offset_deg) = *lines.get(lattice)?;
                    // Each graha's scan tests every graha's lines, and
                    // another's line crossed is not a return.
                    (graha == event.graha).then_some(OffsetReturn {
                        graha,
                        offset_deg,
                        at: event.instant,
                        motion: motion(direction),
                    })
                }
                TransitEventKind::Station { .. } => None,
            })
            .collect();
        returns.sort_by(|a, b| {
            a.at.get()
                .total_cmp(&b.at.get())
                .then((a.graha as u8).cmp(&(b.graha as u8)))
                .then(a.offset_deg.cmp(&b.offset_deg))
        });
        Ok(Envelope::sealing(returns, found.provenance))
    }
}

/// The offsets a return is asked at: at least one, none twice, each a
/// whole degree short of the full circle (C436).
fn check_offsets(offsets: &[u16]) -> Result<(), Error> {
    if offsets.is_empty() {
        return Err(
            Error::invalid_arg("no offset from the natal place to return to")
                .with_field("offsets")
                .with_hint("the return itself is offset 0, the half-return 180"),
        );
    }
    repeated(offsets, "offsets")?;
    if let Some(offset) = offsets.iter().find(|offset| **offset >= 360) {
        return Err(Error::invalid_arg(format!(
            "an offset is a whole degree from 0 to 359 past the natal place, not {offset}"
        ))
        .with_field("offsets")
        .with_hint(format!(
            "the circle closes at 360, so write {}, the same line",
            offset % 360
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    #[test]
    fn an_offset_is_refused_by_its_field_and_its_own_line_named() {
        assert!(check_offsets(&[0, 90, 180, 270, 359]).is_ok());
        let past = check_offsets(&[450]).unwrap_err();
        assert_eq!(past.field(), Some("offsets"));
        assert!(past.hint().is_some_and(|hint| hint.contains("write 90")));
        assert_eq!(check_offsets(&[]).unwrap_err().field(), Some("offsets"));
        assert_eq!(
            check_offsets(&[90, 90]).unwrap_err().field(),
            Some("offsets")
        );
    }
}
