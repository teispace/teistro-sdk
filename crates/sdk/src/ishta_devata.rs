//! The ishṭa-devatā through the façade (`03-design/remedies.md` step 4):
//! the 12th from the kārakāṁśa the chart's Jaimini reading names, and the
//! same from the amātyakāraka, read in the rasi chart and in the navāṁśa
//! both (C130).

use teistro_core::catalogue::{CharaKaraka, Rashi};
use teistro_core::error::Error;
use teistro_core::house::House;
use teistro_remedies::{AmatyaDevatas, DevataRules, IshtaDevatas, amatya_devata, ishta_devata};
use teistro_serial::Document;

use crate::ChartArea;
use crate::rules_bridge::navamsha_of;

impl ChartArea<'_> {
    /// The deity a chart's 12th from the kārakāṁśa names, BPHS (1923) ch.
    /// 9 vv. 70–76: each graha there with its verse's deity and whether
    /// Ketu shares the sign, in both charts the schools count in; and the
    /// same from the amātyakāraka's navāṁśa, with the grahas joined to it
    /// in its house from the lagna (vv. 76–79, C357).
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
    /// // The amātya is never the ātmakāraka.
    /// assert_ne!(read.amatya.graha, read.atmakaraka);
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
        let in_rasi = signs(reading.in_rasi);
        let in_navamsha = signs(reading.in_navamsha);
        let foundation = &document.foundation;
        let amatya = self.karaka_of(foundation, CharaKaraka::Amatyakaraka)?;
        let amsha = in_navamsha
            .get(amatya as usize)
            .copied()
            .ok_or_else(|| Error::internal("the amātya is one of the nine"))?;
        let lagna = Rashi::from_id(u16::from(foundation.lagna_sign_index()))
            .ok_or_else(|| Error::internal("a lagna in no sign"))?;
        let lagna_navamsha = navamsha_of(foundation.lagna_deg)?;
        Ok(IshtaDevatas {
            atmakaraka: reading.atmakaraka,
            karakamsha: from,
            in_rasi: ishta_devata(from, &in_rasi, rules),
            in_navamsha: ishta_devata(from, &in_navamsha, rules),
            amatya: AmatyaDevatas {
                graha: amatya,
                amsha,
                in_rasi: amatya_devata(amsha, amatya, lagna, &in_rasi, rules),
                in_navamsha: amatya_devata(amsha, amatya, lagna_navamsha, &in_navamsha, rules),
            },
        })
    }
}
