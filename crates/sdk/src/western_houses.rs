//! A founded chart's Western houses (`03-design/western-houses.md`).

use teistro_chart::foundation::cusps_of;
use teistro_core::catalogue::HouseSystem;
use teistro_core::error::Error;
use teistro_houses::system::override_of;
use teistro_serial::Document;
use teistro_western::{
    HouseFrame, HouseOverlays, HouseRequest, LEO_HOUSE_SYSTEM, MODULE, OverlayCusps, PlanetAt,
    SynastryRequest, WesternHouses, house_overlays, place_in_houses, rising_an_hour_before,
};

use crate::area::ChartArea;
use crate::progressed::right_ascension;
use crate::western_aspects::{angle_in, planets, points};

impl ChartArea<'_> {
    /// A chart's **Western houses** (`03-design/western-houses.md`): the
    /// cusps of the request's division, else the profile's
    /// `houses.module_overrides.western`, else Placidus, the division
    /// Leo's figures are cast in (C249); each planet counted in the house
    /// whose cusp it has passed, and whether Leo reads it with the
    /// ascendant, up to the degree that rose one sidereal hour before the
    /// birth (C250). Read in the chart's own zodiac.
    ///
    /// ```no_run
    /// # use teistro::{ChartRequest, Context, Ephemeris, HouseRequest};
    /// # use teistro::quantity::{JulianDay, Utc};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::None]).build()?;
    /// # let (instant, request): (JulianDay<Utc>, ChartRequest) = todo!();
    /// let chart = sdk.chart().reading(instant, &request)?.value;
    /// let houses = sdk.chart().western_houses(&chart, &HouseRequest::default())?;
    /// for planet in &houses.planets {
    ///     let rising = if planet.with_ascendant { ", with the ascendant" } else { "" };
    ///     println!("{:?} in house {}{rising}", planet.graha, planet.house.get());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// A chart whose angles a provider supplied, or whose place the polar
    /// policy refuses the division at.
    pub fn western_houses(
        self,
        chart: &Document,
        request: &HouseRequest,
    ) -> Result<WesternHouses, Error> {
        let foundation = &chart.foundation;
        let (cusps_deg, system) = self.western_cusps(chart, *request)?;
        let angles = self.angles(chart)?;
        let zodiac = &foundation.zodiac;
        let armc_deg = right_ascension(
            zodiac.to_tropical(angles.midheaven_deg),
            angles.obliquity_deg,
        );
        let frame = HouseFrame {
            cusps_deg,
            ascendant_deg: angles.ascendant_deg,
            reach_deg: zodiac.of_tropical(rising_an_hour_before(
                armc_deg,
                foundation.place.latitude.get(),
                angles.obliquity_deg,
            )),
        };
        let bodies: Vec<PlanetAt> = planets(foundation)
            .map(|at| PlanetAt::new(at.graha, at.longitude_deg))
            .collect();
        Ok(WesternHouses {
            system,
            planets: place_in_houses(&frame, &bodies)?,
            frame,
        })
    }

    /// The **house overlays** of two charts: each point of `first` counted
    /// in the house of `second` it falls in, and each point of `second` in
    /// `first`'s (`03-design/western-synastry.md` §House overlays, C417).
    ///
    /// The points are a synastry's, each chart's planets and its lagna
    /// unless `request` leaves it out, in the request's zodiac; only its
    /// `lagna` and `zodiac` are read. Each chart's cusps are the ones
    /// [`ChartArea::western_houses`] counts its own planets on: the
    /// division `houses` names, else the profile's
    /// `houses.module_overrides.western`, else Placidus. A point on a cusp
    /// is in the house the cusp opens.
    ///
    /// ```no_run
    /// # use teistro::{ChartRequest, Context, Ephemeris, HouseRequest, SynastryRequest};
    /// # use teistro::catalogue::HouseSystem;
    /// # use teistro::quantity::{JulianDay, Utc};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::None]).build()?;
    /// # let (his, hers): ((JulianDay<Utc>, ChartRequest), (JulianDay<Utc>, ChartRequest)) = todo!();
    /// let first = sdk.chart().reading(his.0, &his.1)?.value;
    /// let second = sdk.chart().reading(hers.0, &hers.1)?.value;
    /// let whole = HouseRequest::default().with_system(HouseSystem::WholeSign);
    /// let read = sdk.chart().synastry_houses(&first, &second, &SynastryRequest::default(), &whole)?;
    /// for one in &read.first_in_second {
    ///     println!("his {:?} in her house {}", one.point, one.house.get());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Two charts founded in different zodiacs when each chart's own is
    /// read (`zodiac`), a chart whose angles a provider supplied, and a
    /// place where the polar policy refuses the division.
    pub fn synastry_houses(
        self,
        first: &Document,
        second: &Document,
        request: &SynastryRequest,
        houses: &HouseRequest,
    ) -> Result<HouseOverlays, Error> {
        let (a, b) = (&first.foundation, &second.foundation);
        crate::composites::refuse_mixed_zodiacs(a, b, request.zodiac)?;
        let cusps = |chart: &Document| -> Result<OverlayCusps, Error> {
            let (cusps_deg, system) = self.western_cusps(chart, *houses)?;
            Ok(OverlayCusps {
                system,
                cusps_deg: cusps_deg.map(|cusp| angle_in(&chart.foundation, request.zodiac, cusp)),
            })
        };
        house_overlays(
            &points(a, request),
            &cusps(first)?,
            &points(b, request),
            &cusps(second)?,
        )
    }

    /// A chart's cusps in the division `request` names, else the
    /// module's, in its own zodiac, with the division they are of.
    pub(crate) fn western_cusps(
        self,
        chart: &Document,
        request: HouseRequest,
    ) -> Result<([f64; 12], HouseSystem), Error> {
        let settings = self.context().settings();
        let system = request
            .system
            .or_else(|| override_of(settings, MODULE))
            .unwrap_or(LEO_HOUSE_SYSTEM);
        cusps_of(
            &chart.foundation,
            system,
            self.context().delta_t(),
            settings.houses.polar_policy,
        )
    }
}
