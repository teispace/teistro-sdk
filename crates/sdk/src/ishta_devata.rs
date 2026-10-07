//! The ishṭa-devatā through the façade (`03-design/remedies.md` step 4):
//! the 12th from the kārakāṁśa the chart's Jaimini reading names, read in
//! the rasi chart and in the navāṁśa both (C130).

use teistro_core::catalogue::Rashi;
use teistro_core::error::Error;
use teistro_core::house::House;
use teistro_remedies::{DevataRules, IshtaDevatas, ishta_devata};
use teistro_serial::Document;

use crate::ChartArea;

impl ChartArea<'_> {
    /// The deity a chart's 12th from the kārakāṁśa names, BPHS (1923) ch.
    /// 9 vv. 70–76: each graha there with its verse's deity and whether
    /// Ketu shares the sign, in both charts the schools count in.
    ///
    /// ```
    /// # use teistro::{ChartRequest, Context, Ephemeris, UtcOffset};
    /// # use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
    /// use teistro::remedies::DevataRules;
    ///
    /// let sdk = Context::builder().ephemeris([Ephemeris::Test]).build()?;
    /// let kathmandu = Place::new(
    ///     Latitude::literal(27.7172),
    ///     Longitude::literal(85.324),
    ///     Altitude::literal(1400.0),
    /// );
    /// let request = ChartRequest::at(kathmandu, UtcOffset::literal(5, 45, 0));
    /// let chart = sdk.chart().reading(JulianDay::<Utc>::literal(2_451_545.0), &request)?.value;
    /// let read = sdk.chart().ishta_devata(&chart, DevataRules::default())?;
    /// // Both charts count from the one kārakāṁśa.
    /// assert_eq!(read.in_rasi.sign, read.in_navamsha.sign);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`ChartArea::jaimini`].
    pub fn ishta_devata(
        self,
        document: &Document,
        rules: DevataRules,
    ) -> Result<IshtaDevatas, Error> {
        let reading = self.jaimini(document)?.karakamsha;
        let from = reading.sign;
        let signs = |houses: [u8; 9]| -> [Rashi; 9] {
            houses.map(|house| House::try_from(house).map_or(from, |house| house.sign_from(from)))
        };
        Ok(IshtaDevatas {
            atmakaraka: reading.atmakaraka,
            karakamsha: from,
            in_rasi: ishta_devata(from, &signs(reading.in_rasi), rules),
            in_navamsha: ishta_devata(from, &signs(reading.in_navamsha), rules),
        })
    }
}
