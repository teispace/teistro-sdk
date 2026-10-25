//! The **Dinesha**, the lord of a day of the month
//! (`03-design/varshesha.md`, "The day's lord").
//!
//! *Tajika Nilakanthi*, Varsha Tantra, the chapter on the month's and the
//! day's results, v. 17: at the day's entry seven planets hold office, and
//! **the strongest of them that aspects the day's lagna** is the day's
//! lord. The verse names them in this order, the lords of:
//!
//! 1. the Muntha's sign;
//! 2. the birth lagna;
//! 3. the day lagna's Tri-Rashi;
//! 4. the Sun's sign by day, or the Moon's by night, in the day chart;
//! 5. the day chart's lagna;
//! 6. the month chart's lagna;
//! 7. the year chart's lagna.
//!
//! The commentary counts them as the month's with one added, "in the year
//! five, in the month six, in the day seven", so the chain is the year
//! lord's ([`crate::varshesha`]), read on the day chart over the seven, as
//! the month's lord is ([`crate::masesha`]). The gloss on v. 5 gives
//! "some" the day lagna's lord outright, which is
//! [`MaseshaReading::LagnaLord`] here as for the month.
//!
//! The Muntha moves 5′ a day ([`DAILY_DEG`](crate::DAILY_DEG)), the
//! source's figure for a year of twelve thirty-day months, so day `d` of
//! month `m` has it `(m − 1) · 30 + (d − 1)` days on (crux C459).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;

use crate::bala::{AnnualSky, Panchavargiya};
use crate::masa::{DAYS, MONTHS};
use crate::masesha::{MaseshaReading, MaseshaRules};
use crate::muntha::MunthaDegree;
use crate::office::{sign_at, tri_rashi_lord};
use crate::varshesha::{Bearers, Varshesha, varshesha};

/// The seven portfolios of a day, in the verse's order.
///
/// ```
/// # use teistro_tajika::AnnualSky;
/// #
/// # // Charak's Example Chart, the forty-first year's: signs counted from Aries as 0.
/// # let at = |sign: f64, deg: f64, min: f64| sign * 30.0 + deg + min / 60.0;
/// # let sky = AnnualSky {
/// #     sun_deg: at(4.0, 3.0, 50.0),
/// #     moon_deg: at(1.0, 9.0, 40.0),
/// #     mars_deg: at(7.0, 7.0, 42.0),
/// #     mercury_deg: at(4.0, 18.0, 20.0),
/// #     jupiter_deg: at(8.0, 9.0, 38.0),
/// #     venus_deg: at(4.0, 21.0, 45.0),
/// #     saturn_deg: at(6.0, 17.0, 13.0),
/// # };
/// # use teistro_tajika::{DayCharts, MunthaDegree, day_bearers, panchavargiya};
/// #
/// # let scorpio = at(7.0, 9.0, 26.0);
/// # let charts = DayCharts {
/// #     natal_lagna_deg: at(4.0, 14.0, 36.0),
/// #     completed_years: 40,
/// #     month: 1,
/// #     day: 1,
/// #     annual_lagna_deg: scorpio,
/// #     month_lagna_deg: scorpio,
/// #     day_lagna_deg: scorpio,
/// #     day_sun_deg: sky.sun_deg,
/// #     day_moon_deg: sky.moon_deg,
/// #     by_day: true,
/// # };
/// # let bearers = day_bearers(&charts, MunthaDegree::default())?;
/// # let strengths = panchavargiya(&sky)?;
/// use teistro_core::catalogue::Graha;
/// use teistro_tajika::DayOffice;
///
/// // Mars rules the day's, the month's and the year's lagna, and the Tri-Rashi.
/// assert_eq!(
///     bearers.portfolios(Graha::Mars),
///     [DayOffice::TriRashi, DayOffice::DinaLagna, DayOffice::MasaLagna, DayOffice::VarshaLagna]
/// );
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DayOffice {
    /// The lord of the Muntha's sign, the Muntha moved on 5′ a day.
    Muntha,
    /// The lord of the birth chart's lagna.
    JanmaLagna,
    /// The day lagna's triplicity lord, by day or by night (crux C458).
    TriRashi,
    /// The lord of the Sun's sign when the day opens by day, of the
    /// Moon's by night.
    DinaRatri,
    /// The lord of the day chart's lagna.
    DinaLagna,
    /// The lord of the month chart's lagna.
    MasaLagna,
    /// The lord of the year chart's lagna.
    VarshaLagna,
}

impl DayOffice {
    /// All seven, in the verse's order.
    ///
    /// ```
    /// use teistro_tajika::DayOffice;
    ///
    /// assert_eq!(DayOffice::ALL.len(), 7);
    /// assert_eq!(DayOffice::ALL[0], DayOffice::Muntha);
    /// ```
    pub const ALL: [DayOffice; 7] = [
        DayOffice::Muntha,
        DayOffice::JanmaLagna,
        DayOffice::TriRashi,
        DayOffice::DinaRatri,
        DayOffice::DinaLagna,
        DayOffice::MasaLagna,
        DayOffice::VarshaLagna,
    ];
}

/// What the seven are read from: four charts, reduced to the numbers the
/// rules touch.
///
/// ```
/// # use teistro_tajika::AnnualSky;
/// #
/// # // Charak's Example Chart, the forty-first year's: signs counted from Aries as 0.
/// # let at = |sign: f64, deg: f64, min: f64| sign * 30.0 + deg + min / 60.0;
/// # let sky = AnnualSky {
/// #     sun_deg: at(4.0, 3.0, 50.0),
/// #     moon_deg: at(1.0, 9.0, 40.0),
/// #     mars_deg: at(7.0, 7.0, 42.0),
/// #     mercury_deg: at(4.0, 18.0, 20.0),
/// #     jupiter_deg: at(8.0, 9.0, 38.0),
/// #     venus_deg: at(4.0, 21.0, 45.0),
/// #     saturn_deg: at(6.0, 17.0, 13.0),
/// # };
/// # use teistro_tajika::{DayCharts, MunthaDegree, day_bearers, panchavargiya};
/// #
/// # let scorpio = at(7.0, 9.0, 26.0);
/// # let charts = DayCharts {
/// #     natal_lagna_deg: at(4.0, 14.0, 36.0),
/// #     completed_years: 40,
/// #     month: 1,
/// #     day: 1,
/// #     annual_lagna_deg: scorpio,
/// #     month_lagna_deg: scorpio,
/// #     day_lagna_deg: scorpio,
/// #     day_sun_deg: sky.sun_deg,
/// #     day_moon_deg: sky.moon_deg,
/// #     by_day: true,
/// # };
/// # let bearers = day_bearers(&charts, MunthaDegree::default())?;
/// # let strengths = panchavargiya(&sky)?;
/// use teistro_core::catalogue::Graha;
///
/// // The Muntha moves 5′ a day: carrying the natal degree, it leaves
/// // Sagittarius between the fifth and the sixth day of the seventh month.
/// let day = |day| DayCharts { month: 7, day, ..charts };
/// assert_eq!(day_bearers(&day(5), MunthaDegree::NatalDegree)?.muntha, Graha::Jupiter);
/// assert_eq!(day_bearers(&day(6), MunthaDegree::NatalDegree)?.muntha, Graha::Saturn);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DayCharts {
    /// The birth chart's lagna, sidereal degrees.
    pub natal_lagna_deg: f64,
    /// The years complete at the year's return: a
    /// [`Pravesha::year`](crate::Pravesha::year).
    pub completed_years: u16,
    /// The month of that year, `1` to `12`.
    pub month: u8,
    /// The day of that month, `1` (the month's own chart) to `30`.
    pub day: u8,
    /// The year chart's lagna, sidereal degrees.
    pub annual_lagna_deg: f64,
    /// The month chart's lagna, sidereal degrees.
    pub month_lagna_deg: f64,
    /// The day chart's lagna, sidereal degrees.
    pub day_lagna_deg: f64,
    /// The Sun in the day chart, sidereal degrees.
    pub day_sun_deg: f64,
    /// The Moon in the day chart, sidereal degrees.
    pub day_moon_deg: f64,
    /// Whether the day opens between sunrise and sunset where its chart is
    /// cast.
    pub by_day: bool,
}

/// The seven office-bearers of one day.
///
/// ```
/// # use teistro_tajika::AnnualSky;
/// #
/// # // Charak's Example Chart, the forty-first year's: signs counted from Aries as 0.
/// # let at = |sign: f64, deg: f64, min: f64| sign * 30.0 + deg + min / 60.0;
/// # let sky = AnnualSky {
/// #     sun_deg: at(4.0, 3.0, 50.0),
/// #     moon_deg: at(1.0, 9.0, 40.0),
/// #     mars_deg: at(7.0, 7.0, 42.0),
/// #     mercury_deg: at(4.0, 18.0, 20.0),
/// #     jupiter_deg: at(8.0, 9.0, 38.0),
/// #     venus_deg: at(4.0, 21.0, 45.0),
/// #     saturn_deg: at(6.0, 17.0, 13.0),
/// # };
/// # use teistro_tajika::{DayCharts, MunthaDegree, day_bearers, panchavargiya};
/// #
/// # let scorpio = at(7.0, 9.0, 26.0);
/// # let charts = DayCharts {
/// #     natal_lagna_deg: at(4.0, 14.0, 36.0),
/// #     completed_years: 40,
/// #     month: 1,
/// #     day: 1,
/// #     annual_lagna_deg: scorpio,
/// #     month_lagna_deg: scorpio,
/// #     day_lagna_deg: scorpio,
/// #     day_sun_deg: sky.sun_deg,
/// #     day_moon_deg: sky.moon_deg,
/// #     by_day: true,
/// # };
/// # let bearers = day_bearers(&charts, MunthaDegree::default())?;
/// # let strengths = panchavargiya(&sky)?;
/// use teistro_core::catalogue::Graha;
///
/// assert_eq!((bearers.muntha, bearers.janma_lagna, bearers.dina_ratri), (Graha::Jupiter, Graha::Sun, Graha::Sun));
/// assert_eq!((bearers.dina_lagna, bearers.masa_lagna, bearers.varsha_lagna), (Graha::Mars, Graha::Mars, Graha::Mars));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct DayBearers {
    /// The lord of the Muntha's sign this day.
    pub muntha: Graha,
    /// The lord of the birth lagna.
    pub janma_lagna: Graha,
    /// The day lagna's triplicity lord for the part of the day.
    pub tri_rashi: Graha,
    /// The lord of the Sun's sign by day or the Moon's by night.
    pub dina_ratri: Graha,
    /// The lord of the day lagna.
    pub dina_lagna: Graha,
    /// The lord of the month lagna.
    pub masa_lagna: Graha,
    /// The lord of the year lagna.
    pub varsha_lagna: Graha,
    /// Whether the day opened by day, which chose two of them.
    pub by_day: bool,
}

impl DayBearers {
    /// Who holds one portfolio.
    ///
    /// ```
    /// # use teistro_tajika::AnnualSky;
    /// #
    /// # // Charak's Example Chart, the forty-first year's: signs counted from Aries as 0.
    /// # let at = |sign: f64, deg: f64, min: f64| sign * 30.0 + deg + min / 60.0;
    /// # let sky = AnnualSky {
    /// #     sun_deg: at(4.0, 3.0, 50.0),
    /// #     moon_deg: at(1.0, 9.0, 40.0),
    /// #     mars_deg: at(7.0, 7.0, 42.0),
    /// #     mercury_deg: at(4.0, 18.0, 20.0),
    /// #     jupiter_deg: at(8.0, 9.0, 38.0),
    /// #     venus_deg: at(4.0, 21.0, 45.0),
    /// #     saturn_deg: at(6.0, 17.0, 13.0),
    /// # };
    /// # use teistro_tajika::{DayCharts, MunthaDegree, day_bearers, panchavargiya};
    /// #
    /// # let scorpio = at(7.0, 9.0, 26.0);
    /// # let charts = DayCharts {
    /// #     natal_lagna_deg: at(4.0, 14.0, 36.0),
    /// #     completed_years: 40,
    /// #     month: 1,
    /// #     day: 1,
    /// #     annual_lagna_deg: scorpio,
    /// #     month_lagna_deg: scorpio,
    /// #     day_lagna_deg: scorpio,
    /// #     day_sun_deg: sky.sun_deg,
    /// #     day_moon_deg: sky.moon_deg,
    /// #     by_day: true,
    /// # };
    /// # let bearers = day_bearers(&charts, MunthaDegree::default())?;
    /// # let strengths = panchavargiya(&sky)?;
    /// use teistro_core::catalogue::Graha;
    /// use teistro_tajika::DayOffice;
    ///
    /// assert_eq!(bearers.holder(DayOffice::Muntha), Graha::Jupiter);
    /// assert_eq!(bearers.holder(DayOffice::TriRashi), Graha::Mars);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub const fn holder(&self, office: DayOffice) -> Graha {
        match office {
            DayOffice::Muntha => self.muntha,
            DayOffice::JanmaLagna => self.janma_lagna,
            DayOffice::TriRashi => self.tri_rashi,
            DayOffice::DinaRatri => self.dina_ratri,
            DayOffice::DinaLagna => self.dina_lagna,
            DayOffice::MasaLagna => self.masa_lagna,
            DayOffice::VarshaLagna => self.varsha_lagna,
        }
    }

    /// The portfolios one planet holds, in the verse's order.
    ///
    /// ```
    /// # use teistro_tajika::AnnualSky;
    /// #
    /// # // Charak's Example Chart, the forty-first year's: signs counted from Aries as 0.
    /// # let at = |sign: f64, deg: f64, min: f64| sign * 30.0 + deg + min / 60.0;
    /// # let sky = AnnualSky {
    /// #     sun_deg: at(4.0, 3.0, 50.0),
    /// #     moon_deg: at(1.0, 9.0, 40.0),
    /// #     mars_deg: at(7.0, 7.0, 42.0),
    /// #     mercury_deg: at(4.0, 18.0, 20.0),
    /// #     jupiter_deg: at(8.0, 9.0, 38.0),
    /// #     venus_deg: at(4.0, 21.0, 45.0),
    /// #     saturn_deg: at(6.0, 17.0, 13.0),
    /// # };
    /// # use teistro_tajika::{DayCharts, MunthaDegree, day_bearers, panchavargiya};
    /// #
    /// # let scorpio = at(7.0, 9.0, 26.0);
    /// # let charts = DayCharts {
    /// #     natal_lagna_deg: at(4.0, 14.0, 36.0),
    /// #     completed_years: 40,
    /// #     month: 1,
    /// #     day: 1,
    /// #     annual_lagna_deg: scorpio,
    /// #     month_lagna_deg: scorpio,
    /// #     day_lagna_deg: scorpio,
    /// #     day_sun_deg: sky.sun_deg,
    /// #     day_moon_deg: sky.moon_deg,
    /// #     by_day: true,
    /// # };
    /// # let bearers = day_bearers(&charts, MunthaDegree::default())?;
    /// # let strengths = panchavargiya(&sky)?;
    /// use teistro_core::catalogue::Graha;
    /// use teistro_tajika::DayOffice;
    ///
    /// assert_eq!(bearers.portfolios(Graha::Sun), [DayOffice::JanmaLagna, DayOffice::DinaRatri]);
    /// assert!(bearers.portfolios(Graha::Moon).is_empty());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn portfolios(&self, graha: Graha) -> Vec<DayOffice> {
        DayOffice::ALL
            .into_iter()
            .filter(|office| self.holder(*office) == graha)
            .collect()
    }
}

impl Bearers for DayBearers {
    fn claimants(&self) -> Vec<Graha> {
        let mut out: Vec<Graha> = Vec::with_capacity(DayOffice::ALL.len());
        for office in DayOffice::ALL {
            let graha = self.holder(office);
            if !out.contains(&graha) {
                out.push(graha);
            }
        }
        out
    }

    fn portfolio_count(&self, graha: Graha) -> u8 {
        u8::try_from(self.portfolios(graha).len()).unwrap_or(u8::MAX)
    }

    fn muntha_lord(&self) -> Graha {
        self.muntha
    }

    fn lagna_lord(&self) -> Graha {
        self.dina_lagna
    }

    fn dina_ratri_lord(&self) -> Graha {
        self.dina_ratri
    }
}

/// The seven office-bearers of one day, the Muntha read in its sign by
/// `muntha`.
///
/// ```
/// # use teistro_tajika::AnnualSky;
/// #
/// # // Charak's Example Chart, the forty-first year's: signs counted from Aries as 0.
/// # let at = |sign: f64, deg: f64, min: f64| sign * 30.0 + deg + min / 60.0;
/// # let sky = AnnualSky {
/// #     sun_deg: at(4.0, 3.0, 50.0),
/// #     moon_deg: at(1.0, 9.0, 40.0),
/// #     mars_deg: at(7.0, 7.0, 42.0),
/// #     mercury_deg: at(4.0, 18.0, 20.0),
/// #     jupiter_deg: at(8.0, 9.0, 38.0),
/// #     venus_deg: at(4.0, 21.0, 45.0),
/// #     saturn_deg: at(6.0, 17.0, 13.0),
/// # };
/// # use teistro_tajika::{DayCharts, MunthaDegree, day_bearers, panchavargiya};
/// #
/// # let scorpio = at(7.0, 9.0, 26.0);
/// # let charts = DayCharts {
/// #     natal_lagna_deg: at(4.0, 14.0, 36.0),
/// #     completed_years: 40,
/// #     month: 1,
/// #     day: 1,
/// #     annual_lagna_deg: scorpio,
/// #     month_lagna_deg: scorpio,
/// #     day_lagna_deg: scorpio,
/// #     day_sun_deg: sky.sun_deg,
/// #     day_moon_deg: sky.moon_deg,
/// #     by_day: true,
/// # };
/// # let bearers = day_bearers(&charts, MunthaDegree::default())?;
/// # let strengths = panchavargiya(&sky)?;
///
/// assert!(bearers.by_day);
/// let past = DayCharts { day: 31, ..charts };
/// assert_eq!(day_bearers(&past, MunthaDegree::default()).unwrap_err().field(), Some("day"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// A longitude that is not a number, named by its field; a
/// `completed_years` past the cap, named `completed_years`; a `month`
/// outside one to twelve, named `month`; a `day` outside one to thirty,
/// named `day`.
pub fn day_bearers(charts: &DayCharts, muntha: MunthaDegree) -> Result<DayBearers, Error> {
    if !(1..=MONTHS).contains(&charts.month) {
        return Err(Error::invalid_arg(format!(
            "a year divides into months 1 to {MONTHS}, not {}",
            charts.month
        ))
        .with_field("month"));
    }
    if !(1..=DAYS).contains(&charts.day) {
        return Err(Error::invalid_arg(format!(
            "a month divides into days 1 to {DAYS}, not {}",
            charts.day
        ))
        .with_field("day"));
    }
    let natal = sign_at("natal_lagna_deg", charts.natal_lagna_deg)?;
    let annual = sign_at("annual_lagna_deg", charts.annual_lagna_deg)?;
    let month = sign_at("month_lagna_deg", charts.month_lagna_deg)?;
    let lagna = sign_at("day_lagna_deg", charts.day_lagna_deg)?;
    let sun = sign_at("day_sun_deg", charts.day_sun_deg)?;
    let moon = sign_at("day_moon_deg", charts.day_moon_deg)?;
    let year = crate::muntha::muntha(charts.natal_lagna_deg, charts.completed_years, muntha)?;
    let days_on = u16::from(charts.month - 1) * u16::from(DAYS) + u16::from(charts.day - 1);
    let moved = year.during(f64::from(days_on) / (f64::from(MONTHS) * f64::from(DAYS)));
    Ok(DayBearers {
        muntha: sign_at("muntha", moved)?.attributes().lord,
        janma_lagna: natal.attributes().lord,
        tri_rashi: tri_rashi_lord(lagna, charts.by_day),
        dina_ratri: if charts.by_day { sun } else { moon }.attributes().lord,
        dina_lagna: lagna.attributes().lord,
        masa_lagna: month.attributes().lord,
        varsha_lagna: annual.attributes().lord,
        by_day: charts.by_day,
    })
}

/// The lord of one day, and the chain it was read beside.
///
/// ```
/// # use teistro_tajika::AnnualSky;
/// #
/// # // Charak's Example Chart, the forty-first year's: signs counted from Aries as 0.
/// # let at = |sign: f64, deg: f64, min: f64| sign * 30.0 + deg + min / 60.0;
/// # let sky = AnnualSky {
/// #     sun_deg: at(4.0, 3.0, 50.0),
/// #     moon_deg: at(1.0, 9.0, 40.0),
/// #     mars_deg: at(7.0, 7.0, 42.0),
/// #     mercury_deg: at(4.0, 18.0, 20.0),
/// #     jupiter_deg: at(8.0, 9.0, 38.0),
/// #     venus_deg: at(4.0, 21.0, 45.0),
/// #     saturn_deg: at(6.0, 17.0, 13.0),
/// # };
/// # use teistro_tajika::{DayCharts, MunthaDegree, day_bearers, panchavargiya};
/// #
/// # let scorpio = at(7.0, 9.0, 26.0);
/// # let charts = DayCharts {
/// #     natal_lagna_deg: at(4.0, 14.0, 36.0),
/// #     completed_years: 40,
/// #     month: 1,
/// #     day: 1,
/// #     annual_lagna_deg: scorpio,
/// #     month_lagna_deg: scorpio,
/// #     day_lagna_deg: scorpio,
/// #     day_sun_deg: sky.sun_deg,
/// #     day_moon_deg: sky.moon_deg,
/// #     by_day: true,
/// # };
/// # let bearers = day_bearers(&charts, MunthaDegree::default())?;
/// # let strengths = panchavargiya(&sky)?;
/// use teistro_core::catalogue::Graha;
/// use teistro_tajika::{MaseshaReading, MaseshaRules, dinesha};
///
/// // The day lagna's lord under the other reading, the chain carried beside it.
/// let rules = MaseshaRules { reading: MaseshaReading::LagnaLord, ..MaseshaRules::default() };
/// let day = dinesha(&charts, &sky, &strengths, rules)?;
/// assert_eq!((day.graha, day.chain.graha), (Graha::Mars, Graha::Sun));
/// assert_eq!(day.bearers, bearers);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Dinesha {
    /// The lord of the day under [`Dinesha::reading`].
    pub graha: Graha,
    /// The reading that made it lord: the chain, or the day lagna's lord.
    pub reading: MaseshaReading,
    /// The seven it was chosen among.
    pub bearers: DayBearers,
    /// The chain over the seven, carried under either reading.
    pub chain: Varshesha,
}

/// The lord of one day.
///
/// `sky` and `strengths` describe the **day** chart, the one whose lagna
/// `charts` names. `rules` are the month's lord's own, read for the day as
/// the commentary reads the month "as for the year".
///
/// ```
/// # use teistro_tajika::AnnualSky;
/// #
/// # // Charak's Example Chart, the forty-first year's: signs counted from Aries as 0.
/// # let at = |sign: f64, deg: f64, min: f64| sign * 30.0 + deg + min / 60.0;
/// # let sky = AnnualSky {
/// #     sun_deg: at(4.0, 3.0, 50.0),
/// #     moon_deg: at(1.0, 9.0, 40.0),
/// #     mars_deg: at(7.0, 7.0, 42.0),
/// #     mercury_deg: at(4.0, 18.0, 20.0),
/// #     jupiter_deg: at(8.0, 9.0, 38.0),
/// #     venus_deg: at(4.0, 21.0, 45.0),
/// #     saturn_deg: at(6.0, 17.0, 13.0),
/// # };
/// # use teistro_tajika::{DayCharts, MunthaDegree, day_bearers, panchavargiya};
/// #
/// # let scorpio = at(7.0, 9.0, 26.0);
/// # let charts = DayCharts {
/// #     natal_lagna_deg: at(4.0, 14.0, 36.0),
/// #     completed_years: 40,
/// #     month: 1,
/// #     day: 1,
/// #     annual_lagna_deg: scorpio,
/// #     month_lagna_deg: scorpio,
/// #     day_lagna_deg: scorpio,
/// #     day_sun_deg: sky.sun_deg,
/// #     day_moon_deg: sky.moon_deg,
/// #     by_day: true,
/// # };
/// # let bearers = day_bearers(&charts, MunthaDegree::default())?;
/// # let strengths = panchavargiya(&sky)?;
/// use teistro_core::catalogue::Graha;
/// use teistro_tajika::{Chosen, MaseshaRules, dinesha};
///
/// // The strongest of the seven that aspects the day lagna: the Sun, as for the year.
/// let day = dinesha(&charts, &sky, &strengths, MaseshaRules::default())?;
/// assert_eq!((day.graha, day.chain.chosen), (Graha::Sun, Chosen::Strongest));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`day_bearers`] and [`varshesha`].
pub fn dinesha(
    charts: &DayCharts,
    sky: &AnnualSky,
    strengths: &[Panchavargiya],
    rules: MaseshaRules,
) -> Result<Dinesha, Error> {
    let bearers = day_bearers(charts, rules.muntha)?;
    let chain = varshesha(&bearers, sky, strengths, charts.day_lagna_deg, rules.chain)?;
    let graha = match rules.reading {
        MaseshaReading::Chain => chain.graha,
        MaseshaReading::LagnaLord => bearers.dina_lagna,
    };
    Ok(Dinesha {
        graha,
        reading: rules.reading,
        bearers,
        chain,
    })
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "tests fail by panicking"
    )]

    use super::*;
    use crate::bala::panchavargiya;
    use crate::masesha::{MonthCharts, masesha, month_bearers};

    fn at(sign: f64, deg: f64, min: f64) -> f64 {
        sign * 30.0 + deg + min / 60.0
    }

    /// Charak's worked year, used as a month chart and its first day's.
    fn sky() -> AnnualSky {
        AnnualSky {
            sun_deg: at(4.0, 3.0, 50.0),
            moon_deg: at(1.0, 9.0, 40.0),
            mars_deg: at(7.0, 7.0, 42.0),
            mercury_deg: at(4.0, 18.0, 20.0),
            jupiter_deg: at(8.0, 9.0, 38.0),
            venus_deg: at(4.0, 21.0, 45.0),
            saturn_deg: at(6.0, 17.0, 13.0),
        }
    }

    fn charts(month: u8, day: u8) -> DayCharts {
        let sky = sky();
        DayCharts {
            natal_lagna_deg: at(4.0, 14.0, 36.0),
            completed_years: 40,
            month,
            day,
            annual_lagna_deg: at(7.0, 9.0, 26.0),
            month_lagna_deg: at(7.0, 9.0, 26.0),
            day_lagna_deg: at(7.0, 9.0, 26.0),
            day_sun_deg: sky.sun_deg,
            day_moon_deg: sky.moon_deg,
            by_day: true,
        }
    }

    /// A month's first day is the month's own chart, so its seven offices
    /// are the month's six with the month lagna's lord again, and its lord
    /// is the month's.
    #[test]
    fn the_first_day_is_the_month() {
        let day = day_bearers(&charts(1, 1), MunthaDegree::SignStart).unwrap();
        let month = month_bearers(
            &MonthCharts {
                natal_lagna_deg: at(4.0, 14.0, 36.0),
                completed_years: 40,
                month: 1,
                annual_lagna_deg: at(7.0, 9.0, 26.0),
                month_lagna_deg: at(7.0, 9.0, 26.0),
                month_sun_deg: sky().sun_deg,
                month_moon_deg: sky().moon_deg,
                by_day: true,
            },
            MunthaDegree::SignStart,
        )
        .unwrap();
        assert_eq!(day.dina_lagna, month.masa_lagna);
        assert_eq!(day.masa_lagna, month.masa_lagna);
        assert_eq!(day.muntha, month.muntha);
        assert_eq!(day.tri_rashi, month.tri_rashi);
        assert_eq!(day.dina_ratri, month.dina_ratri);
        let strengths = panchavargiya(&sky()).unwrap();
        let lord = dinesha(&charts(1, 1), &sky(), &strengths, MaseshaRules::default()).unwrap();
        let month_charts = MonthCharts {
            natal_lagna_deg: at(4.0, 14.0, 36.0),
            completed_years: 40,
            month: 1,
            annual_lagna_deg: at(7.0, 9.0, 26.0),
            month_lagna_deg: at(7.0, 9.0, 26.0),
            month_sun_deg: sky().sun_deg,
            month_moon_deg: sky().moon_deg,
            by_day: true,
        };
        let month_lord =
            masesha(&month_charts, &sky(), &strengths, MaseshaRules::default()).unwrap();
        assert_eq!(lord.graha, month_lord.graha);
        assert_eq!(lord.graha, Graha::Sun, "Charak's year lord");
        // Scorpio's lord holds the day lagna, the month lagna, its
        // Tri-Rashi by day and the year lagna: four portfolios.
        assert_eq!(day.portfolios(Graha::Mars).len(), 4);
    }

    /// A Muntha at 2s 28°20′40″ carrying its degree reaches Cancer after
    /// 1°39′20″, which at 5′ a day is the 20th day, 19 days on.
    #[test]
    fn the_muntha_moves_five_minutes_a_day() {
        let natal = at(10.0, 28.0, 20.0 + 40.0 / 60.0);
        let lord = |day| {
            day_bearers(
                &DayCharts {
                    natal_lagna_deg: natal,
                    ..charts(1, day)
                },
                MunthaDegree::NatalDegree,
            )
            .unwrap()
            .muntha
        };
        assert_eq!(lord(20), Graha::Mercury);
        assert_eq!(lord(21), Graha::Moon);
    }

    #[test]
    fn the_others_reading_takes_the_day_lagnas_lord() {
        let strengths = panchavargiya(&sky()).unwrap();
        let rules = MaseshaRules {
            reading: MaseshaReading::LagnaLord,
            ..MaseshaRules::default()
        };
        let lord = dinesha(&charts(3, 9), &sky(), &strengths, rules).unwrap();
        assert_eq!(lord.graha, Graha::Mars, "Scorpio's lord");
        assert_eq!(lord.chain.graha, Graha::Sun);
    }

    #[test]
    fn a_day_outside_the_month_is_refused_by_name() {
        for day in [0, 31] {
            let refused = day_bearers(&charts(1, day), MunthaDegree::SignStart).unwrap_err();
            assert_eq!(refused.field(), Some("day"));
        }
        let refused = day_bearers(&charts(13, 1), MunthaDegree::SignStart).unwrap_err();
        assert_eq!(refused.field(), Some("month"));
    }
}
