//! The month's dasha (`03-design/varshesha.md`, "The month's dasha").
//!
//! *Tajika Nilakanthi*, Varsha Tantra, the chapter on the month's and the
//! day's results, v. 14: of the Muntha's lord and the month lagna's lord,
//! in whichever's sign is the stronger, the Hudda lords of that sign are
//! the month's dasha lords, each running "by days equal to its own
//! degrees", which the verse gives as "some say from the tradition". The
//! five Huddas of a sign fill its thirty degrees, so the five run the
//! month.
//!
//! v. 15 records that others reject it, since the Sun and the Moon, which
//! hold no Hudda, get no share, and read a month's results in the
//! Patyayini's antardasha instead. That reading needs nothing new: the
//! annual dasha already carries the Patyayini's antardashas (crux C477).
//!
//! The verse's "sign of the two lords" is read as where each stands on the
//! month chart ([`MasaDashaSign::Occupied`]); the sign each rules, the
//! Muntha's and the month lagna's, is the other reading
//! ([`MasaDashaSign::Owned`]). Either way the stronger lord is the one with
//! the greater Vishwa bala, which every strength in the chapter reads; a
//! tie, or one graha holding both offices, takes the Muntha's lord, whom
//! the verse names first.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};

use crate::bala::{HuddaTable, Panchavargiya, hudda_terms};
use crate::masa::DAYS;
use crate::masesha::{MonthCharts, MonthOffice, month_bearers, muntha_sign};
use crate::muntha::MunthaDegree;
use crate::office::sign_at;

/// Which sign of the two lords the month's dasha is read in (crux C477).
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
/// # use teistro_tajika::{MonthCharts, MunthaDegree, month_bearers, panchavargiya};
/// #
/// # let charts = MonthCharts {
/// #     natal_lagna_deg: at(4.0, 14.0, 36.0),
/// #     completed_years: 40,
/// #     month: 1,
/// #     annual_lagna_deg: at(7.0, 9.0, 26.0),
/// #     month_lagna_deg: at(7.0, 9.0, 26.0),
/// #     month_sun_deg: sky.sun_deg,
/// #     month_moon_deg: sky.moon_deg,
/// #     by_day: true,
/// # };
/// # let bearers = month_bearers(&charts, MunthaDegree::default())?;
/// # let strengths = panchavargiya(&sky)?;
/// use teistro_core::catalogue::Graha;
/// use teistro_core::quantity::JulianDay;
/// use teistro_tajika::{MasaDashaRules, MonthClock, masa_dasha};
///
/// use teistro_core::catalogue::Rashi;
/// use teistro_tajika::MasaDashaSign;
///
/// // Jupiter, the Muntha's lord, is the stronger and stands in Sagittarius,
/// // which is also the Muntha's sign: both readings read Sagittarius here.
/// let opens = [JulianDay::literal(2_461_000.0)];
/// let rules = MasaDashaRules { clock: MonthClock::Days, ..MasaDashaRules::default() };
/// for sign in [MasaDashaSign::Occupied, MasaDashaSign::Owned] {
///     let dasha = masa_dasha(&charts, &strengths, &opens, MunthaDegree::default(), MasaDashaRules { sign, ..rules })?;
///     assert_eq!((dasha.graha, dasha.sign), (Graha::Jupiter, Rashi::Sagittarius));
/// }
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MasaDashaSign {
    /// The sign the stronger lord stands in on the month chart. The
    /// default.
    #[default]
    Occupied,
    /// The sign it rules here: the Muntha's own sign for the Muntha's
    /// lord, the month lagna for the lagna's.
    Owned,
}

/// What a degree of a Hudda is in time (crux C477).
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
/// # use teistro_tajika::{MonthCharts, MunthaDegree, month_bearers, panchavargiya};
/// #
/// # let charts = MonthCharts {
/// #     natal_lagna_deg: at(4.0, 14.0, 36.0),
/// #     completed_years: 40,
/// #     month: 1,
/// #     annual_lagna_deg: at(7.0, 9.0, 26.0),
/// #     month_lagna_deg: at(7.0, 9.0, 26.0),
/// #     month_sun_deg: sky.sun_deg,
/// #     month_moon_deg: sky.moon_deg,
/// #     by_day: true,
/// # };
/// # let bearers = month_bearers(&charts, MunthaDegree::default())?;
/// # let strengths = panchavargiya(&sky)?;
/// use teistro_core::quantity::JulianDay;
/// use teistro_tajika::{MasaDashaRules, MonthClock, masa_dasha};
///
///
/// // Thirty-one solar-day knots, a little longer than a civil day each.
/// let knots: Vec<JulianDay<_>> = (0..=30).map(|degree| JulianDay::literal(2_461_000.0 + f64::from(degree) * 1.016)).collect();
/// let under = |clock| masa_dasha(&charts, &strengths, &knots, MunthaDegree::default(), MasaDashaRules { clock, ..MasaDashaRules::default() });
/// // The first Hudda is Jupiter's twelve degrees: twelve knots, or twelve civil days.
/// let by_sun = under(MonthClock::SunDegrees)?.periods[0];
/// let by_days = under(MonthClock::Days)?.periods[0];
/// assert_eq!(by_sun.ends, knots[12]);
/// assert_eq!(by_days.ends.get() - by_days.starts.get(), 12.0);
/// assert!((under(MonthClock::Even)?.periods[0].ends.get() - knots[12].get()).abs() < 1e-6);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MonthClock {
    /// The Sun's motion through a degree from where it stood when the
    /// month opened: the "solar day" the year's dasha counts in (C122),
    /// and the day the Dina Pravesha opens. The month ends as the next
    /// opens. The default.
    #[default]
    SunDegrees,
    /// An equal share of the time from the month's opening to the next.
    Even,
    /// A civil day from the month's opening: thirty days, which can end a
    /// little before or after the next month opens.
    Days,
}

/// The readings the month's dasha is computed under.
///
/// ```
/// use teistro_tajika::{HuddaTable, MasaDashaRules, MasaDashaSign, MonthClock};
///
/// let rules = MasaDashaRules::default();
/// assert_eq!((rules.sign, rules.clock, rules.table), (MasaDashaSign::Occupied, MonthClock::SunDegrees, HuddaTable::Nilakanthi));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
pub struct MasaDashaRules {
    /// Which sign of the stronger lord its Huddas are read in.
    pub sign: MasaDashaSign,
    /// What a degree is in time.
    pub clock: MonthClock,
    /// Which printing of the Hudda table the sign's terms are read from.
    pub table: HuddaTable,
}

/// One lord's run of the month.
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
/// # use teistro_tajika::{MonthCharts, MunthaDegree, month_bearers, panchavargiya};
/// #
/// # let charts = MonthCharts {
/// #     natal_lagna_deg: at(4.0, 14.0, 36.0),
/// #     completed_years: 40,
/// #     month: 1,
/// #     annual_lagna_deg: at(7.0, 9.0, 26.0),
/// #     month_lagna_deg: at(7.0, 9.0, 26.0),
/// #     month_sun_deg: sky.sun_deg,
/// #     month_moon_deg: sky.moon_deg,
/// #     by_day: true,
/// # };
/// # let bearers = month_bearers(&charts, MunthaDegree::default())?;
/// # let strengths = panchavargiya(&sky)?;
/// use teistro_core::quantity::JulianDay;
/// use teistro_tajika::{MasaDashaRules, MonthClock, masa_dasha};
///
///
/// let opens = [JulianDay::literal(2_461_000.0)];
/// let rules = MasaDashaRules { clock: MonthClock::Days, ..MasaDashaRules::default() };
/// let dasha = masa_dasha(&charts, &strengths, &opens, MunthaDegree::default(), rules)?;
/// // The five fill the month's thirty degrees, each beginning where the last ends.
/// assert_eq!(dasha.periods.iter().map(|period| u32::from(period.degrees)).sum::<u32>(), 30);
/// assert!(dasha.periods.windows(2).all(|pair| pair[0].ends == pair[1].starts));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct MasaDashaPeriod {
    /// The Hudda's lord.
    pub lord: Graha,
    /// The Hudda's width, degrees: the days it runs.
    pub degrees: u8,
    /// When it begins, UTC.
    pub starts: JulianDay<Utc>,
    /// When it ends, UTC: where the next begins.
    pub ends: JulianDay<Utc>,
}

/// One month's dasha.
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
/// # use teistro_tajika::{MonthCharts, MunthaDegree, month_bearers, panchavargiya};
/// #
/// # let charts = MonthCharts {
/// #     natal_lagna_deg: at(4.0, 14.0, 36.0),
/// #     completed_years: 40,
/// #     month: 1,
/// #     annual_lagna_deg: at(7.0, 9.0, 26.0),
/// #     month_lagna_deg: at(7.0, 9.0, 26.0),
/// #     month_sun_deg: sky.sun_deg,
/// #     month_moon_deg: sky.moon_deg,
/// #     by_day: true,
/// # };
/// # let bearers = month_bearers(&charts, MunthaDegree::default())?;
/// # let strengths = panchavargiya(&sky)?;
/// use teistro_core::catalogue::Graha;
/// use teistro_core::quantity::JulianDay;
/// use teistro_tajika::{MasaDashaRules, MonthClock, masa_dasha};
///
/// use teistro_tajika::{HuddaTable, MonthOffice, hudda_terms};
///
/// let opens = [JulianDay::literal(2_461_000.0)];
/// let rules = MasaDashaRules { clock: MonthClock::Days, ..MasaDashaRules::default() };
/// let dasha = masa_dasha(&charts, &strengths, &opens, MunthaDegree::default(), rules)?;
/// assert_eq!((dasha.office, dasha.graha), (MonthOffice::Muntha, Graha::Jupiter));
/// let lords: Vec<Graha> = dasha.periods.iter().map(|period| period.lord).collect();
/// assert_eq!(lords, hudda_terms(HuddaTable::Nilakanthi, dasha.sign).map(|(_, lord)| lord));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct MasaDasha {
    /// Whose sign it is read in: [`MonthOffice::Muntha`] or
    /// [`MonthOffice::MasaLagna`].
    pub office: MonthOffice,
    /// The stronger of the two lords.
    pub graha: Graha,
    /// The sign whose five Huddas run the month.
    pub sign: Rashi,
    /// The readings it was computed under.
    pub rules: MasaDashaRules,
    /// The five lords in the sign's order, from the month's opening.
    pub periods: Vec<MasaDashaPeriod>,
}

/// The month's dasha of one month.
///
/// `strengths` are the **month** chart's, the one `charts` names the lagna
/// of. `knots` are the month's [`month_knots`](crate::month_knots): the
/// thirty-one instants the Sun stands none to thirty degrees on from the
/// month's opening, which [`MonthClock::SunDegrees`] and
/// [`MonthClock::Even`] read; [`MonthClock::Days`] reads the first only.
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
/// # use teistro_tajika::{MonthCharts, MunthaDegree, month_bearers, panchavargiya};
/// #
/// # let charts = MonthCharts {
/// #     natal_lagna_deg: at(4.0, 14.0, 36.0),
/// #     completed_years: 40,
/// #     month: 1,
/// #     annual_lagna_deg: at(7.0, 9.0, 26.0),
/// #     month_lagna_deg: at(7.0, 9.0, 26.0),
/// #     month_sun_deg: sky.sun_deg,
/// #     month_moon_deg: sky.moon_deg,
/// #     by_day: true,
/// # };
/// # let bearers = month_bearers(&charts, MunthaDegree::default())?;
/// # let strengths = panchavargiya(&sky)?;
/// use teistro_core::quantity::JulianDay;
/// use teistro_tajika::{MasaDashaRules, MonthClock, masa_dasha};
///
///
/// let opens = [JulianDay::literal(2_461_000.0)];
/// let rules = MasaDashaRules { clock: MonthClock::Days, ..MasaDashaRules::default() };
/// assert_eq!(masa_dasha(&charts, &strengths, &opens, MunthaDegree::default(), rules)?.periods.len(), 5);
/// // The solar-day clock reads thirty-one knots; a strength table must carry both lords.
/// let short = masa_dasha(&charts, &strengths, &opens, MunthaDegree::default(), MasaDashaRules::default());
/// assert_eq!(short.unwrap_err().field(), Some("knots"));
/// assert_eq!(masa_dasha(&charts, &[], &opens, MunthaDegree::default(), rules).unwrap_err().field(), Some("strengths"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`month_bearers`]; a lord whose strength `strengths` lacks, named
/// `strengths`; fewer knots than the clock reads, named `knots`.
pub fn masa_dasha(
    charts: &MonthCharts,
    strengths: &[Panchavargiya],
    knots: &[JulianDay<Utc>],
    muntha: MunthaDegree,
    rules: MasaDashaRules,
) -> Result<MasaDasha, Error> {
    let bearers = month_bearers(charts, muntha)?;
    let strength = |graha: Graha| {
        strengths
            .iter()
            .find(|one| one.graha == graha)
            .ok_or_else(|| {
                Error::invalid_arg(format!("no strength was given for {graha:?}"))
                    .with_field("strengths")
            })
    };
    let (by_muntha, by_lagna) = (strength(bearers.muntha)?, strength(bearers.masa_lagna)?);
    let (office, lord) = if by_lagna.vishwa > by_muntha.vishwa {
        (MonthOffice::MasaLagna, by_lagna)
    } else {
        (MonthOffice::Muntha, by_muntha)
    };
    let sign = match (rules.sign, office) {
        (MasaDashaSign::Occupied, _) => lord.sign,
        (MasaDashaSign::Owned, MonthOffice::MasaLagna) => {
            sign_at("month_lagna_deg", charts.month_lagna_deg)?
        }
        (MasaDashaSign::Owned, _) => muntha_sign(charts, muntha)?,
    };
    let at = clock(rules.clock, knots)?;
    let mut opened = 0;
    let periods = hudda_terms(rules.table, sign)
        .into_iter()
        .map(|(degrees, lord)| {
            let ends = opened + degrees;
            let period = MasaDashaPeriod {
                lord,
                degrees,
                starts: at(opened),
                ends: at(ends),
            };
            opened = ends;
            period
        })
        .collect();
    Ok(MasaDasha {
        office,
        graha: lord.graha,
        sign,
        rules,
        periods,
    })
}

/// The instant `knots` put a degree of the month at, under `reading`.
///
/// # Errors
///
/// Fewer knots than `reading` reads, named `knots`.
fn clock(
    reading: MonthClock,
    knots: &[JulianDay<Utc>],
) -> Result<impl Fn(u8) -> JulianDay<Utc> + '_, Error> {
    let needed = match reading {
        MonthClock::SunDegrees | MonthClock::Even => usize::from(DAYS) + 1,
        MonthClock::Days => 1,
    };
    let opens = match knots.first() {
        Some(opens) if knots.len() >= needed => opens.get(),
        _ => {
            return Err(Error::invalid_arg(format!(
                "the month's clock reads {needed} knots of the Sun, and {} were given",
                knots.len()
            ))
            .with_field("knots"));
        }
    };
    let span = knots
        .get(usize::from(DAYS))
        .map_or(0.0, |closes| closes.get() - opens);
    Ok(move |degree: u8| match reading {
        MonthClock::SunDegrees => knots
            .get(usize::from(degree))
            .copied()
            .unwrap_or(JulianDay::literal(opens)),
        MonthClock::Even => JulianDay::literal(opens + span * f64::from(degree) / f64::from(DAYS)),
        MonthClock::Days => JulianDay::literal(opens + f64::from(degree)),
    })
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::float_cmp,
        clippy::indexing_slicing,
        reason = "tests fail by panicking, index their own fixtures and compare exact sums"
    )]

    use super::*;
    use crate::bala::{AnnualSky, panchavargiya};

    fn at(sign: f64, deg: f64, min: f64) -> f64 {
        sign * 30.0 + deg + min / 60.0
    }

    /// Charak's worked year, as a month chart.
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

    fn charts(sky: &AnnualSky) -> MonthCharts {
        MonthCharts {
            natal_lagna_deg: at(4.0, 14.0, 36.0),
            completed_years: 40,
            month: 1,
            annual_lagna_deg: at(7.0, 9.0, 26.0),
            month_lagna_deg: at(7.0, 9.0, 26.0),
            month_sun_deg: sky.sun_deg,
            month_moon_deg: sky.moon_deg,
            by_day: true,
        }
    }

    /// Thirty-one knots, the Sun's degrees a little over a day apart.
    fn knots() -> Vec<JulianDay<Utc>> {
        (0..=30)
            .map(|degree| JulianDay::literal(2_445_932.5 + f64::from(degree) * 1.02))
            .collect()
    }

    fn dasha(sky: &AnnualSky, rules: MasaDashaRules) -> MasaDasha {
        let strengths = panchavargiya(sky).unwrap();
        masa_dasha(
            &charts(sky),
            &strengths,
            &knots(),
            MunthaDegree::SignStart,
            rules,
        )
        .unwrap()
    }

    /// Charak's year has the Muntha in Sagittarius, Jupiter's, and the
    /// lagna in Scorpio, Mars's: the stronger's five Huddas run the month
    /// in the sign's order, end to end over its thirty degrees.
    #[test]
    fn the_stronger_lords_huddas_run_the_month() {
        let sky = sky();
        let strengths = panchavargiya(&sky).unwrap();
        let vishwa = |graha| {
            strengths
                .iter()
                .find(|one| one.graha == graha)
                .unwrap()
                .vishwa
        };
        let month = dasha(&sky, MasaDashaRules::default());
        let (office, graha, sign) = if vishwa(Graha::Mars) > vishwa(Graha::Jupiter) {
            (MonthOffice::MasaLagna, Graha::Mars, Rashi::Scorpio)
        } else {
            (MonthOffice::Muntha, Graha::Jupiter, Rashi::Sagittarius)
        };
        assert_eq!(
            (month.office, month.graha, month.sign),
            (office, graha, sign)
        );
        let terms = hudda_terms(HuddaTable::Nilakanthi, sign);
        let lords: Vec<(Graha, u8)> = month.periods.iter().map(|p| (p.lord, p.degrees)).collect();
        assert_eq!(lords, terms.map(|(width, lord)| (lord, width)).to_vec());
        let knots = knots();
        assert_eq!(month.periods.first().unwrap().starts, knots[0]);
        assert_eq!(month.periods.last().unwrap().ends, knots[30]);
        for pair in month.periods.windows(2) {
            assert_eq!(pair[0].ends, pair[1].starts, "end to end");
        }
        let first = terms[0].0;
        assert_eq!(
            month.periods[0].ends,
            knots[usize::from(first)],
            "a degree is a knot"
        );
    }

    /// Mars moved into Aries, which it also rules: read where it stands, the
    /// month runs Aries's Huddas; read by the sign it rules here, the
    /// lagna's, Scorpio's. With Jupiter stronger, both read Sagittarius.
    #[test]
    fn the_sign_is_where_the_lord_stands_or_what_it_rules() {
        let sky = AnnualSky {
            mars_deg: at(0.0, 9.0, 0.0),
            ..sky()
        };
        let occupied = dasha(&sky, MasaDashaRules::default());
        let owned = dasha(
            &sky,
            MasaDashaRules {
                sign: MasaDashaSign::Owned,
                ..MasaDashaRules::default()
            },
        );
        assert_eq!(occupied.graha, owned.graha);
        if occupied.graha == Graha::Mars {
            assert_eq!((occupied.sign, owned.sign), (Rashi::Aries, Rashi::Scorpio));
        } else {
            assert_eq!(
                (occupied.sign, owned.sign),
                (Rashi::Sagittarius, Rashi::Sagittarius)
            );
        }
    }

    /// One graha holding both offices is the Muntha's lord, the verse naming
    /// it first, and with the Muntha and the lagna in its two signs the
    /// Muntha's is the one read.
    #[test]
    fn one_lord_of_both_is_the_munthas() {
        let sky = sky();
        let charts = MonthCharts {
            // Forty signs on from Sagittarius is Aries: Mars's, as the
            // lagna's.
            natal_lagna_deg: at(8.0, 1.0, 0.0),
            ..charts(&sky)
        };
        let strengths = panchavargiya(&sky).unwrap();
        let rules = MasaDashaRules {
            sign: MasaDashaSign::Owned,
            ..MasaDashaRules::default()
        };
        let month = masa_dasha(
            &charts,
            &strengths,
            &knots(),
            MunthaDegree::SignStart,
            rules,
        )
        .unwrap();
        assert_eq!(
            (month.office, month.graha),
            (MonthOffice::Muntha, Graha::Mars)
        );
        assert_eq!(month.sign, Rashi::Aries);
    }

    #[test]
    fn each_clock_reads_a_degree_its_own_way() {
        let sky = sky();
        let with = |clock| {
            dasha(
                &sky,
                MasaDashaRules {
                    clock,
                    ..MasaDashaRules::default()
                },
            )
        };
        let knots = knots();
        let span = knots[30].get() - knots[0].get();
        let even = with(MonthClock::Even);
        let days = with(MonthClock::Days);
        let mut degree = 0.0;
        for (e, d) in even.periods.iter().zip(&days.periods) {
            degree += f64::from(e.degrees);
            assert!((e.ends.get() - (knots[0].get() + span * degree / 30.0)).abs() < 1e-9);
            assert_eq!(d.ends.get(), knots[0].get() + degree);
        }
        assert_eq!(degree, 30.0);
        // Days reads only the opening.
        let strengths = panchavargiya(&sky).unwrap();
        let rules = MasaDashaRules {
            clock: MonthClock::Days,
            ..MasaDashaRules::default()
        };
        let one = masa_dasha(
            &charts(&sky),
            &strengths,
            &knots[..1],
            MunthaDegree::SignStart,
            rules,
        );
        assert_eq!(one.unwrap().periods, days.periods);
    }

    #[test]
    fn a_clock_short_of_knots_and_a_lord_without_strength_are_refused_by_name() {
        let sky = sky();
        let strengths = panchavargiya(&sky).unwrap();
        let short = masa_dasha(
            &charts(&sky),
            &strengths,
            &knots()[..30],
            MunthaDegree::SignStart,
            MasaDashaRules::default(),
        )
        .unwrap_err();
        assert_eq!(short.field(), Some("knots"));
        let none = masa_dasha(
            &charts(&sky),
            &strengths[..1],
            &knots(),
            MunthaDegree::SignStart,
            MasaDashaRules::default(),
        )
        .unwrap_err();
        assert_eq!(none.field(), Some("strengths"));
    }
}
