//! A founded chart's Western houses (`03-design/western-houses.md`).

use teistro_chart::foundation::cusps_of;
use teistro_core::catalogue::HouseSystem;
use teistro_core::error::Error;
use teistro_houses::system::override_of;
use teistro_serial::Document;
use teistro_western::{
    HouseFrame, HouseRequest, LEO_HOUSE_SYSTEM, MODULE, PlanetAt, WesternHouses, place_in_houses,
    rising_an_hour_before,
};

use crate::area::ChartArea;
use crate::progressed::right_ascension;
use crate::western_aspects::planets;

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
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
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
