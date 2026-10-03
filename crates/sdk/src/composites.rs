//! Composite and Davison charts: one chart of two
//! (`03-design/western-composites.md`, C247, C248).

use teistro_chart::foundation::ChartFoundation;
use teistro_core::angle::near_midpoint_deg;
use teistro_core::error::{Error, Status};
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_core::time::UtcOffset;
use teistro_serial::Document;
use teistro_western::{ChartPoints, Composite, HouseRequest, Placed, SynastryZodiac, composite};

use crate::area::ChartArea;
use crate::western_aspects::{Partner, angle_in, planets_in};

impl ChartArea<'_> {
    /// The **composite** of two charts (Townley; Astrolog, C247): each
    /// planet at the near midpoint of its two places and the mean of its
    /// two speeds, the midheaven at the near midpoint of the two, and the
    /// lagna at the near midpoint of the two, turned by 180° when that
    /// stands before the midheaven. Its cusps are the near midpoints of
    /// the two charts' in the `western` module's division, each turned the
    /// same way (`western-houses.md`), and none where the profile's polar
    /// policy refuses that division at either birthplace. Read in the
    /// tropical zodiac, or each chart's own (C241).
    ///
    /// ```no_run
    /// # use teistro::{ChartRequest, Context, Ephemeris, SynastryZodiac};
    /// # use teistro::quantity::{JulianDay, Utc};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let (his, hers): ((JulianDay<Utc>, ChartRequest), (JulianDay<Utc>, ChartRequest)) = todo!();
    /// let first = sdk.chart().reading(his.0, &his.1)?.value;
    /// let second = sdk.chart().reading(hers.0, &hers.1)?.value;
    /// let both = sdk.chart().composite(&first, &second, SynastryZodiac::Tropical)?;
    /// println!("composite lagna {:.2}°, midheaven {:.2}°", both.lagna_deg, both.midheaven_deg);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Two charts that place different planets; in each chart's own
    /// zodiac, two charts founded in different ones (`zodiac`); and what
    /// [`ChartArea::angles`] refuses of either chart.
    pub fn composite(
        self,
        first: &Document,
        second: &Document,
        zodiac: SynastryZodiac,
    ) -> Result<Composite, Error> {
        refuse_mixed_zodiacs(&first.foundation, &second.foundation, zodiac)?;
        let points = |chart: &Document| -> Result<ChartPoints, Error> {
            let angles = self.angles(chart)?;
            let foundation = &chart.foundation;
            Ok(ChartPoints {
                planets: planets_in(foundation, zodiac)
                    .map(|(at, longitude)| Placed::new(at.graha, longitude, at.speed_deg_per_day))
                    .collect(),
                lagna_deg: angle_in(foundation, zodiac, angles.ascendant_deg),
                midheaven_deg: angle_in(foundation, zodiac, angles.midheaven_deg),
                cusps_deg: match self.western_cusps(chart, HouseRequest::default()) {
                    Ok((cusps, _)) => Some(cusps.map(|cusp| angle_in(foundation, zodiac, cusp))),
                    Err(why) if why.status == Status::Unsupported => None,
                    Err(why) => return Err(why),
                },
            })
        };
        composite(
            &points(first).map_err(|why| why.under("first"))?,
            &points(second).map_err(|why| why.under("second"))?,
        )
    }
}

impl Partner {
    /// The **Davison** birth between this one and `other` (C248): the
    /// mean of the two instants on the UTC scale; the mean of the two
    /// latitudes and of the two altitudes; the mean of the two longitudes
    /// the shorter way round; and the mean of the two clocks, to the
    /// second, which names only the civil day. Any chart request founds it,
    /// so a Davison chart takes every reading a natal chart does.
    ///
    /// ```
    /// use teistro::{Partner, UtcOffset};
    /// use teistro::quantity::{JulianDay, Place, Utc};
    ///
    /// let his = Partner {
    ///     instant: JulianDay::<Utc>::literal(2_402_390.554_166_667),
    ///     place: Place::try_from_degrees(51.5045, -0.1366, 0.0)?,
    ///     utc_offset: UtcOffset::UTC,
    /// };
    /// let hers = Partner {
    ///     instant: JulianDay::<Utc>::literal(2_403_113.499_305_556),
    ///     place: Place::try_from_degrees(51.5058, -0.1878, 0.0)?,
    ///     utc_offset: UtcOffset::UTC,
    /// };
    /// let between = his.davison(&hers)?;
    /// assert!((between.instant.get() - 2_402_752.026_736_111).abs() < 1e-9);
    /// assert!((between.place.longitude.get() + 0.1622).abs() < 1e-9);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// None for two valid births; the result is checked as a birth is.
    pub fn davison(&self, other: &Partner) -> Result<Partner, Error> {
        let (a, b) = (&self.place, &other.place);
        let offset =
            (i64::from(self.utc_offset.seconds()) + i64::from(other.utc_offset.seconds())) / 2;
        let longitude = near_midpoint_deg(a.longitude.get(), b.longitude.get());
        Ok(Partner {
            instant: JulianDay::<Utc>::try_new(f64::midpoint(
                self.instant.get(),
                other.instant.get(),
            ))?,
            place: Place::try_from_degrees(
                f64::midpoint(a.latitude.get(), b.latitude.get()),
                if longitude > 180.0 {
                    longitude - 360.0
                } else {
                    longitude
                },
                f64::midpoint(a.altitude.get(), b.altitude.get()),
            )?,
            utc_offset: UtcOffset::try_from_seconds(
                i32::try_from(offset)
                    .map_err(|_| Error::internal("a mean of two clocks overflowed"))?,
            )?,
        })
    }
}

/// Refuses two charts founded in different zodiacs when they are to be
/// compared in their own (C241).
pub(crate) fn refuse_mixed_zodiacs(
    a: &ChartFoundation,
    b: &ChartFoundation,
    zodiac: SynastryZodiac,
) -> Result<(), Error> {
    if zodiac == SynastryZodiac::Charts && a.zodiac.ayanamsha != b.zodiac.ayanamsha {
        return Err(Error::invalid_arg(
            "the two charts were founded in different zodiacs, so their longitudes do not compare",
        )
        .with_field("zodiac")
        .with_hint("found both under one zodiac, or compare them in the tropical one"));
    }
    Ok(())
}
