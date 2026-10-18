//! The **Masesha**, the lord of a month of the year
//! (`03-design/varshesha.md`, "The month's lord").
//!
//! *Tajika Nilakanthi*, Varsha Tantra, the chapter on the month's and the
//! day's results, vv. 3–5: at the month's entry six planets hold office,
//! and **the strongest of them that aspects the month's lagna** is the
//! month's lord. The six are the lords of:
//!
//! 1. the month chart's lagna;
//! 2. the Muntha's sign;
//! 3. the birth lagna;
//! 4. the month lagna's Tri-Rashi;
//! 5. the Sun's sign by day, or the Moon's by night, in the month chart;
//! 6. the year chart's lagna.
//!
//! The commentary says that the rest is "as for the year", where five
//! hold office and here six. So the chain is the year lord's
//! ([`varshesha`]), read on the month chart over the six. Verse 5 adds
//! that "others" make the month lagna's lord the month's lord outright:
//! [`MaseshaReading::LagnaLord`], a named reading rather than a silence.
//!
//! The Muntha moves 2°30′ a month, so its lord can change inside a year.
//! The commentary's own example has it at 2s 28°20′40″ moving into Cancer
//! by the next month. That only happens where the Muntha carries the
//! natal lagna's degree ([`MunthaDegree::NatalDegree`]); one entering
//! each sign at its start never leaves it inside a year.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;

use crate::bala::{AnnualSky, Panchavargiya};
use crate::masa::MONTHS;
use crate::muntha::MunthaDegree;
use crate::office::{sign_at, tri_rashi_lord};
use crate::varshesha::{Bearers, Varshesha, VarsheshaRules, varshesha};

/// The six portfolios of a month, in the verse's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MonthOffice {
    /// The lord of the month chart's lagna.
    MasaLagna,
    /// The lord of the Muntha's sign, the Muntha moved on 2°30′ a month.
    Muntha,
    /// The lord of the birth chart's lagna.
    JanmaLagna,
    /// The month lagna's triplicity lord, by day or by night.
    TriRashi,
    /// The lord of the Sun's sign when the month opens by day, of the
    /// Moon's by night.
    DinaRatri,
    /// The lord of the year chart's lagna.
    VarshaLagna,
}

impl MonthOffice {
    /// All six, in the verse's order.
    pub const ALL: [MonthOffice; 6] = [
        MonthOffice::MasaLagna,
        MonthOffice::Muntha,
        MonthOffice::JanmaLagna,
        MonthOffice::TriRashi,
        MonthOffice::DinaRatri,
        MonthOffice::VarshaLagna,
    ];
}

/// What the six are read from: three charts, reduced to the numbers the
/// rules touch.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MonthCharts {
    /// The birth chart's lagna, sidereal degrees.
    pub natal_lagna_deg: f64,
    /// The years complete at the year's return: a
    /// [`Pravesha::year`](crate::Pravesha::year).
    pub completed_years: u16,
    /// The month of that year, `1` (the year's own chart) to `12`.
    pub month: u8,
    /// The year chart's lagna, sidereal degrees.
    pub annual_lagna_deg: f64,
    /// The month chart's lagna, sidereal degrees.
    pub month_lagna_deg: f64,
    /// The Sun in the month chart, sidereal degrees.
    pub month_sun_deg: f64,
    /// The Moon in the month chart, sidereal degrees.
    pub month_moon_deg: f64,
    /// Whether the month opens between sunrise and sunset where its chart
    /// is cast.
    pub by_day: bool,
}

/// The six office-bearers of one month.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct MonthBearers {
    /// The lord of the month lagna.
    pub masa_lagna: Graha,
    /// The lord of the Muntha's sign this month.
    pub muntha: Graha,
    /// The lord of the birth lagna.
    pub janma_lagna: Graha,
    /// The month lagna's triplicity lord for the part of the day.
    pub tri_rashi: Graha,
    /// The lord of the Sun's sign by day or the Moon's by night.
    pub dina_ratri: Graha,
    /// The lord of the year lagna.
    pub varsha_lagna: Graha,
    /// Whether the month opened by day, which chose two of them.
    pub by_day: bool,
}

impl MonthBearers {
    /// Who holds one portfolio.
    #[must_use]
    pub const fn holder(&self, office: MonthOffice) -> Graha {
        match office {
            MonthOffice::MasaLagna => self.masa_lagna,
            MonthOffice::Muntha => self.muntha,
            MonthOffice::JanmaLagna => self.janma_lagna,
            MonthOffice::TriRashi => self.tri_rashi,
            MonthOffice::DinaRatri => self.dina_ratri,
            MonthOffice::VarshaLagna => self.varsha_lagna,
        }
    }

    /// The portfolios one planet holds, in the verse's order.
    #[must_use]
    pub fn portfolios(&self, graha: Graha) -> Vec<MonthOffice> {
        MonthOffice::ALL
            .into_iter()
            .filter(|office| self.holder(*office) == graha)
            .collect()
    }
}

impl Bearers for MonthBearers {
    fn claimants(&self) -> Vec<Graha> {
        let mut out: Vec<Graha> = Vec::with_capacity(MonthOffice::ALL.len());
        for office in MonthOffice::ALL {
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
        self.masa_lagna
    }

    fn dina_ratri_lord(&self) -> Graha {
        self.dina_ratri
    }
}

/// The six office-bearers of one month, the Muntha read in its sign by
/// `muntha`.
///
/// # Errors
///
/// A longitude that is not a number, named by its field; a
/// `completed_years` past the cap, named `completed_years`; a `month`
/// outside one to twelve, named `month`.
pub fn month_bearers(charts: &MonthCharts, muntha: MunthaDegree) -> Result<MonthBearers, Error> {
    if !(1..=MONTHS).contains(&charts.month) {
        return Err(Error::invalid_arg(format!(
            "a year divides into months 1 to {MONTHS}, not {}",
            charts.month
        ))
        .with_field("month"));
    }
    let natal = sign_at("natal_lagna_deg", charts.natal_lagna_deg)?;
    let annual = sign_at("annual_lagna_deg", charts.annual_lagna_deg)?;
    let lagna = sign_at("month_lagna_deg", charts.month_lagna_deg)?;
    let sun = sign_at("month_sun_deg", charts.month_sun_deg)?;
    let moon = sign_at("month_moon_deg", charts.month_moon_deg)?;
    let year = crate::muntha::muntha(charts.natal_lagna_deg, charts.completed_years, muntha)?;
    let moved = year.during(f64::from(charts.month - 1) / f64::from(MONTHS));
    Ok(MonthBearers {
        masa_lagna: lagna.attributes().lord,
        muntha: sign_at("muntha", moved)?.attributes().lord,
        janma_lagna: natal.attributes().lord,
        tri_rashi: tri_rashi_lord(lagna, charts.by_day),
        dina_ratri: if charts.by_day { sun } else { moon }.attributes().lord,
        varsha_lagna: annual.attributes().lord,
        by_day: charts.by_day,
    })
}

/// Which reading makes a planet the month's lord.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MaseshaReading {
    /// The year lord's chain over the six: the strongest that aspects the
    /// month lagna, with the year's fallbacks. The verse's own, and the
    /// default.
    #[default]
    Chain,
    /// The month lagna's lord outright, which v. 5 gives to "others".
    LagnaLord,
}

impl MaseshaReading {
    /// Both readings, the verse's first.
    pub const ALL: [MaseshaReading; 2] = [MaseshaReading::Chain, MaseshaReading::LagnaLord];

    /// The member's key, as serde writes it: `CHAIN`.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            MaseshaReading::Chain => "CHAIN",
            MaseshaReading::LagnaLord => "LAGNA_LORD",
        }
    }
}

/// The readings the month's lord parts on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
pub struct MaseshaRules {
    /// Which reading makes the lord.
    pub reading: MaseshaReading,
    /// Where the Muntha stands inside its sign, which decides whether its
    /// lord can change inside the year; the year's own default.
    pub muntha: MunthaDegree,
    /// The year lord's readings, which the chain takes "as for the year".
    pub chain: VarsheshaRules,
}

/// The lord of one month, and the chain it was read beside.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Masesha {
    /// The lord of the month under [`Masesha::reading`].
    pub graha: Graha,
    /// The reading that made it lord.
    pub reading: MaseshaReading,
    /// The six it was chosen among.
    pub bearers: MonthBearers,
    /// The chain over the six: its own lord, which step chose it, and
    /// every claimant. Carried under either reading, so a reader can see
    /// where the two part.
    pub chain: Varshesha,
}

/// The lord of one month.
///
/// `sky` and `strengths` describe the **month** chart, the one whose lagna
/// `charts` names.
///
/// # Errors
///
/// As [`month_bearers`] and [`varshesha`].
pub fn masesha(
    charts: &MonthCharts,
    sky: &AnnualSky,
    strengths: &[Panchavargiya],
    rules: MaseshaRules,
) -> Result<Masesha, Error> {
    let bearers = month_bearers(charts, rules.muntha)?;
    let chain = varshesha(
        &bearers,
        sky,
        strengths,
        charts.month_lagna_deg,
        rules.chain,
    )?;
    let graha = match rules.reading {
        MaseshaReading::Chain => chain.graha,
        MaseshaReading::LagnaLord => bearers.masa_lagna,
    };
    Ok(Masesha {
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
    use crate::office::{YearCharts, office_bearers};

    fn at(sign: f64, deg: f64, min: f64) -> f64 {
        sign * 30.0 + deg + min / 60.0
    }

    /// Charak's worked year, used as a month chart: its sky and lagna.
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

    fn charts(month: u8) -> MonthCharts {
        let sky = sky();
        MonthCharts {
            natal_lagna_deg: at(4.0, 14.0, 36.0),
            completed_years: 40,
            month,
            annual_lagna_deg: at(7.0, 9.0, 26.0),
            month_lagna_deg: at(7.0, 9.0, 26.0),
            month_sun_deg: sky.sun_deg,
            month_moon_deg: sky.moon_deg,
            by_day: true,
        }
    }

    /// The first month is the year's own chart, so its six offices are
    /// the year's five and the year lagna's lord again, and its lord is
    /// the year's.
    #[test]
    fn the_first_month_is_the_year() {
        let month = month_bearers(&charts(1), MunthaDegree::SignStart).unwrap();
        let sky = sky();
        let year = office_bearers(&YearCharts {
            natal_lagna_deg: at(4.0, 14.0, 36.0),
            completed_years: 40,
            annual_lagna_deg: at(7.0, 9.0, 26.0),
            annual_sun_deg: sky.sun_deg,
            annual_moon_deg: sky.moon_deg,
            by_day: true,
        })
        .unwrap();
        assert_eq!(month.masa_lagna, year.varsha_lagna);
        assert_eq!(month.varsha_lagna, year.varsha_lagna);
        assert_eq!(month.muntha, year.muntha);
        assert_eq!(month.janma_lagna, year.janma_lagna);
        assert_eq!(month.tri_rashi, year.tri_rashi);
        assert_eq!(month.dina_ratri, year.dina_ratri);
        let sorted = |mut grahas: Vec<Graha>| {
            grahas.sort_by_key(|graha| graha.id());
            grahas
        };
        assert_eq!(sorted(Bearers::claimants(&month)), sorted(year.claimants()));
        let strengths = panchavargiya(&sky).unwrap();
        let lagna = at(7.0, 9.0, 26.0);
        let lord = masesha(&charts(1), &sky, &strengths, MaseshaRules::default()).unwrap();
        assert_eq!(lord.bearers, month);
        let year_lord =
            varshesha(&year, &sky, &strengths, lagna, VarsheshaRules::default()).unwrap();
        assert_eq!(lord.graha, year_lord.graha);
        assert_eq!(lord.graha, Graha::Sun, "Charak's year lord");
        // Scorpio's lord holds the month lagna, its Tri-Rashi by day and
        // the year lagna: three portfolios.
        assert_eq!(month.portfolios(Graha::Mars).len(), 3);
    }

    /// The commentary's example: a Muntha at 2s 28°20′40″ is in Cancer a
    /// month on, so its lord is the Moon, where it had been Mercury.
    /// Carried at the natal degree, the Muntha changes sign in the year;
    /// entered at the sign's start, it cannot.
    #[test]
    fn a_muntha_carrying_its_degree_changes_lord_in_the_year() {
        // Forty signs on from Aquarius 28°20′40″ is Gemini 28°20′40″.
        let natal = at(10.0, 28.0, 20.0 + 40.0 / 60.0);
        let year = |month, degree| {
            month_bearers(
                &MonthCharts {
                    natal_lagna_deg: natal,
                    ..charts(month)
                },
                degree,
            )
            .unwrap()
            .muntha
        };
        assert_eq!(year(1, MunthaDegree::NatalDegree), Graha::Mercury);
        assert_eq!(year(2, MunthaDegree::NatalDegree), Graha::Moon);
        assert_eq!(year(12, MunthaDegree::SignStart), Graha::Mercury);
    }

    #[test]
    fn the_others_reading_takes_the_month_lagnas_lord() {
        let sky = sky();
        let strengths = panchavargiya(&sky).unwrap();
        let rules = MaseshaRules {
            reading: MaseshaReading::LagnaLord,
            ..MaseshaRules::default()
        };
        let lord = masesha(&charts(1), &sky, &strengths, rules).unwrap();
        assert_eq!(lord.graha, Graha::Mars, "Scorpio's lord");
        assert_eq!(lord.chain.graha, Graha::Sun, "the chain still says the Sun");
    }

    #[test]
    fn a_month_outside_the_year_is_refused_by_name() {
        for month in [0, 13] {
            let refused = month_bearers(&charts(month), MunthaDegree::SignStart).unwrap_err();
            assert_eq!(refused.field(), Some("month"));
        }
    }
}
