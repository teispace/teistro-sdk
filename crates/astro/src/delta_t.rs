//! Delta T, the difference between Terrestrial Time and Universal Time:
//! the IERS table where it was measured, a cited model either side, and
//! an uncertainty on every value (`docs/03-design/time-and-timezone.md`,
//! §3.1; spike 3's finding that a polynomial fit alone is five seconds
//! stale by 2025).

use core::fmt;

use teistro_core::error::{Detail, Error};
use teistro_core::quantity::{JulianDay, Ut1};

use teistro_port_ephemeris::{DeltaTBand, DeltaTEstimate};

use crate::generated::{DELTA_T_ROWS, DELTA_T_SOURCE, HISTORICAL_ROWS, HISTORICAL_SOURCE};

/// J2000.0 as a Julian day.
const J2000: f64 = 2_451_545.0;
/// Days in a Julian year.
const DAYS_PER_YEAR: f64 = 365.25;
/// The Julian day of the Modified Julian Date's zero.
const MJD_EPOCH: f64 = 2_400_000.5;
/// How far beyond the table's last row the end slope is trusted before
/// the model takes over, in years.
const LINEAR_TAIL_YEARS: f64 = 10.0;
/// How far before the table's first row the seam offset is tapered away,
/// in years.
const SEAM_TAPER_YEARS: f64 = 10.0;
/// Huber's (2000) diffusion constant of the length of day, ms² a year.
const HUBER_Q_MS2_PER_YEAR: f64 = 0.058;
/// Huber's (2000) horizon of the drift's growth, years.
const HUBER_M_YEARS: f64 = 2500.0;
/// The last year Espenak and Meeus's (2006) polynomials were fitted to:
/// where their extrapolation, and Huber's band for it, begins.
const ESPENAK_MEEUS_LAST_YEAR: f64 = 2005.0;

pub use teistro_port_ephemeris::DeltaTModel;

/// Where a value came from.
///
/// ```
/// use teistro_astro::delta_t::{DeltaTModel, DeltaTSource, delta_t};
/// use teistro_core::quantity::{JulianDay, Ut1};
///
/// // Measured in 2000, so the table answers; in 1000 CE a model fitted to
/// // the eclipse record; in 1000 BC only a model carried beyond its data.
/// let measured = delta_t(JulianDay::<Ut1>::literal(2_451_544.5), DeltaTModel::TableThenModel)?;
/// assert_eq!(measured.source, DeltaTSource::Table);
/// let medieval = delta_t(JulianDay::<Ut1>::literal(2_086_308.0), DeltaTModel::TableThenModel)?;
/// assert_eq!(medieval.source, DeltaTSource::Model);
/// let ancient = delta_t(JulianDay::<Ut1>::literal(1_355_818.5), DeltaTModel::TableThenModel)?;
/// assert_eq!(ancient.source, DeltaTSource::Extrapolated);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeltaTSource {
    /// Interpolated in the IERS table.
    Table,
    /// From a model, outside the table, over the span the model's data
    /// covers (Morrison and Stephenson's, from −500 to the IERS era).
    Model,
    /// Through the leap-second table: TT less UTC, exact, with the UT1
    /// difference (under 0.9 s) not applied.
    LeapSeconds,
    /// Supplied by the consumer.
    Custom,
    /// The ephemeris engine's, under the model asked, answered through the
    /// port.
    Provider,
    /// From a model **beyond** the data it was fitted to: after the IERS
    /// table's last row (or 2005, Espenak and Meeus's last), or before
    /// −500, Morrison and Stephenson's first. Its uncertainty is Huber's
    /// (2000) random walk from the last value known ([`huber_2000_sigma`]).
    Extrapolated,
}

/// Delta T at an instant, with what produced it.
///
/// ```
/// use teistro_astro::delta_t::{DeltaTModel, delta_t};
/// use teistro_core::quantity::{JulianDay, Ut1};
///
/// let ut1 = JulianDay::<Ut1>::literal(2_451_544.5);
/// let applied = delta_t(ut1, DeltaTModel::TableThenModel)?;
/// // About 63.8 s at the start of 2000.
/// assert!(applied.seconds > 63.0 && applied.seconds < 64.5);
/// assert!(applied.to_string().starts_with("Delta T 63."));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DeltaT {
    /// TT less UT1, seconds.
    pub seconds: f64,
    /// The model asked for.
    pub model: DeltaTModel,
    /// What answered.
    pub source: DeltaTSource,
    /// The one-sigma uncertainty in seconds, when the source has one.
    pub uncertainty_seconds: Option<f64>,
}

impl DeltaT {
    /// The value in days, for Julian-day arithmetic.
    ///
    /// ```
    /// use teistro_astro::delta_t::{DeltaTModel, delta_t};
    /// use teistro_core::quantity::{JulianDay, Ut1};
    ///
    /// let ut1 = JulianDay::<Ut1>::literal(2_451_544.5);
    /// let applied = delta_t(ut1, DeltaTModel::Custom { seconds: 86_400.0 })?;
    /// assert_eq!(applied.days(), 1.0);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn days(&self) -> f64 {
        self.seconds / 86_400.0
    }

    /// Where the uncertainty comes from: the IERS's measurement inside its
    /// table, Morrison and Stephenson's (2004) standard errors for the
    /// SDK's models outside it, and none for a custom value.
    ///
    /// ```
    /// use teistro_astro::delta_t::{DeltaTModel, delta_t};
    /// use teistro_core::quantity::{JulianDay, Ut1};
    /// use teistro_port_ephemeris::DeltaTBand;
    ///
    /// let measured = delta_t(JulianDay::<Ut1>::literal(2_451_544.5), DeltaTModel::TableThenModel)?;
    /// assert_eq!(measured.band(), DeltaTBand::Measured);
    /// let custom = delta_t(JulianDay::<Ut1>::literal(2_451_544.5), DeltaTModel::Custom { seconds: 64.0 })?;
    /// assert_eq!(custom.band(), DeltaTBand::Unstated);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub const fn band(&self) -> DeltaTBand {
        match self.source {
            DeltaTSource::Table | DeltaTSource::LeapSeconds => DeltaTBand::Measured,
            DeltaTSource::Model => DeltaTBand::MorrisonStephenson2004,
            DeltaTSource::Extrapolated => DeltaTBand::Huber2000,
            DeltaTSource::Custom | DeltaTSource::Provider => DeltaTBand::Unstated,
        }
    }

    /// An engine's answer under a model, as the SDK carries it.
    ///
    /// ```
    /// use teistro_astro::delta_t::{DeltaT, DeltaTSource};
    /// use teistro_port_ephemeris::{DeltaTBand, DeltaTEstimate, DeltaTModel};
    ///
    /// let answered = DeltaTEstimate { seconds: 69.1, sigma_seconds: Some(0.9), band: DeltaTBand::StephensonMorrisonHohenkerk2016 };
    /// let carried = DeltaT::from_provider(DeltaTModel::StephensonMorrisonHohenkerk2016, answered);
    /// assert_eq!((carried.source, carried.uncertainty_seconds), (DeltaTSource::Provider, Some(0.9)));
    /// ```
    #[must_use]
    pub const fn from_provider(model: DeltaTModel, estimate: DeltaTEstimate) -> DeltaT {
        DeltaT {
            seconds: estimate.seconds,
            model,
            source: DeltaTSource::Provider,
            uncertainty_seconds: estimate.sigma_seconds,
        }
    }

    /// The value as the port carries it.
    ///
    /// ```
    /// use teistro_astro::{DeltaTModel, delta_t};
    /// use teistro_core::quantity::JulianDay;
    ///
    /// let applied = delta_t(JulianDay::literal(2_451_545.0), DeltaTModel::TableThenModel)?;
    /// let carried = applied.estimate();
    /// assert_eq!((carried.seconds, carried.band), (applied.seconds, applied.band()));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub const fn estimate(&self) -> DeltaTEstimate {
        DeltaTEstimate {
            seconds: self.seconds,
            sigma_seconds: self.uncertainty_seconds,
            band: self.band(),
        }
    }
}

impl fmt::Display for DeltaT {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Delta T {:.3} s ({})", self.seconds, self.model.key())?;
        if let Some(sigma) = self.uncertainty_seconds {
            write!(f, " ± {sigma:.2} s")?;
        }
        Ok(())
    }
}

/// The IERS series the table comes from.
///
/// ```
/// use teistro_astro::delta_t::table_source;
///
/// assert!(!table_source().is_empty());
/// ```
#[must_use]
pub const fn table_source() -> &'static str {
    DELTA_T_SOURCE
}

/// The historical table the uncertainties before the IERS era come from.
///
/// ```
/// use teistro_astro::delta_t::historical_source;
///
/// assert!(!historical_source().is_empty());
/// ```
#[must_use]
pub const fn historical_source() -> &'static str {
    HISTORICAL_SOURCE
}

/// The span the table covers, as Julian days.
///
/// ```
/// use teistro_astro::delta_t::{decimal_year, table_span};
///
/// // The IERS table begins in the 1950s or 1970s and runs to the present.
/// let (first, last) = table_span().ok_or("a table")?;
/// assert!(decimal_year(first) < 1975.0 && decimal_year(last) > 2020.0);
/// # Ok::<(), &str>(())
/// ```
#[must_use]
pub fn table_span() -> Option<(f64, f64)> {
    let first = DELTA_T_ROWS.first()?;
    let last = DELTA_T_ROWS.last()?;
    Some((
        f64::from(first.0) + MJD_EPOCH,
        f64::from(last.0) + MJD_EPOCH,
    ))
}

/// The decimal year of a Julian day, in Julian years from J2000.0.
///
/// ```
/// use teistro_astro::delta_t::decimal_year;
///
/// assert_eq!(decimal_year(2_451_545.0), 2000.0);
/// assert_eq!(decimal_year(2_451_545.0 + 365.25), 2001.0);
/// ```
#[must_use]
pub fn decimal_year(jd: f64) -> f64 {
    2000.0 + (jd - J2000) / DAYS_PER_YEAR
}

/// The polynomial expressions of Espenak and Meeus (2006, "Five
/// Millennium Canon of Solar Eclipses", NASA/TP-2006-214141, section
/// 2.7), fitted to Morrison and Stephenson's data, over the decimal
/// year: the long-term parabola before −500 and after 2150, and twelve
/// piecewise polynomials between, with the 2050 to 2150 expression that
/// joins the parabola. Seconds.
///
/// ```
/// use teistro_astro::delta_t::espenak_meeus_2006;
///
/// // The 500 to 1600 polynomial at its centre, 1000.
/// assert_eq!(espenak_meeus_2006(1000.0), 1574.2);
/// // And 2000, 63.86 s by the 1986 to 2005 fit.
/// assert!((espenak_meeus_2006(2000.0) - 63.86).abs() < 0.01);
/// ```
#[must_use]
pub fn espenak_meeus_2006(year: f64) -> f64 {
    let parabola = |y: f64| {
        let u = (y - 1820.0) / 100.0;
        -20.0 + 32.0 * u * u
    };
    if year < -500.0 {
        parabola(year)
    } else if year < 500.0 {
        let u = year / 100.0;
        10_583.6
            + u * (-1014.41
                + u * (33.783_11
                    + u * (-5.952_053
                        + u * (-0.179_845_2 + u * (0.022_174_192 + u * 0.009_031_652_1)))))
    } else if year < 1600.0 {
        let u = (year - 1000.0) / 100.0;
        1574.2
            + u * (-556.01
                + u * (71.234_72
                    + u * (0.319_781
                        + u * (-0.850_346_3 + u * (-0.005_050_998 + u * 0.008_357_207_3)))))
    } else if year < 1700.0 {
        let t = year - 1600.0;
        120.0 + t * (-0.9808 + t * (-0.01532 + t / 7129.0))
    } else if year < 1800.0 {
        let t = year - 1700.0;
        8.83 + t * (0.1603 + t * (-0.005_928_5 + t * (0.000_133_36 - t / 1_174_000.0)))
    } else if year < 1860.0 {
        let t = year - 1800.0;
        13.72
            + t * (-0.332_447
                + t * (0.006_861_2
                    + t * (0.004_111_6
                        + t * (-0.000_374_36
                            + t * (0.000_012_127_2
                                + t * (-0.000_000_169_9 + t * 0.000_000_000_875))))))
    } else if year < 1900.0 {
        let t = year - 1860.0;
        7.62 + t
            * (0.5737
                + t * (-0.251_754 + t * (0.016_806_68 + t * (-0.000_447_362_4 + t / 233_174.0))))
    } else if year < 1920.0 {
        let t = year - 1900.0;
        -2.79 + t * (1.494_119 + t * (-0.059_893_9 + t * (0.006_196_6 - t * 0.000_197)))
    } else if year < 1941.0 {
        let t = year - 1920.0;
        21.20 + t * (0.844_93 + t * (-0.076_100 + t * 0.002_093_6))
    } else if year < 1961.0 {
        let t = year - 1950.0;
        29.07 + t * (0.407 + t * (-1.0 / 233.0 + t / 2547.0))
    } else if year < 1986.0 {
        let t = year - 1975.0;
        45.45 + t * (1.067 + t * (-1.0 / 260.0 - t / 718.0))
    } else if year < 2005.0 {
        let t = year - 2000.0;
        63.86
            + t * (0.3345
                + t * (-0.060_374 + t * (0.001_727_5 + t * (0.000_651_814 + t * 0.000_023_735_99))))
    } else if year < 2050.0 {
        let t = year - 2000.0;
        62.92 + t * (0.322_17 + t * 0.005_589)
    } else if year < 2150.0 {
        parabola(year) - 0.5628 * (2150.0 - year)
    } else {
        parabola(year)
    }
}

/// The value and uncertainty interpolated in the IERS table, `None`
/// outside it.
fn table(mjd: f64) -> Option<(f64, f64)> {
    let first = DELTA_T_ROWS.first()?;
    let last = DELTA_T_ROWS.last()?;
    if mjd < f64::from(first.0) || mjd > f64::from(last.0) {
        return None;
    }
    let after = DELTA_T_ROWS.partition_point(|row| f64::from(row.0) <= mjd);
    let upper = DELTA_T_ROWS.get(after.min(DELTA_T_ROWS.len() - 1))?;
    let lower = DELTA_T_ROWS.get(after.saturating_sub(1))?;
    let (m0, v0, s0) = (f64::from(lower.0), f64::from(lower.1), f64::from(lower.2));
    let (m1, v1, s1) = (f64::from(upper.0), f64::from(upper.1), f64::from(upper.2));
    let value = if m1 > m0 {
        v0 + (v1 - v0) * (mjd - m0) / (m1 - m0)
    } else {
        v0
    };
    Some((value, s0.max(s1)))
}

/// The one-sigma uncertainty of Delta T `years` from the last value
/// known, seconds: Huber's (2000) model of the length of day as a random
/// walk with drift, `σ = 365.25 N √(N Q / 3 · (1 + N / M)) / 1000` with
/// `N` the years, `Q = 0.058 ms²/yr` and `M = 2500` years, as Espenak and
/// Meeus apply it to the centuries before their data and after it
/// ("Uncertainty in Delta T", eclipse.gsfc.nasa.gov/SEhelp/uncertainty2004.html,
/// Table 3, calibrated at −500 and at 2005).
///
/// ```
/// use teistro_astro::delta_t::huber_2000_sigma;
///
/// // Table 3: +2500 is 495 years from 2005, ±612 s; −1000 is 500 years
/// // before −500, ±622 s; −4000, ±16291 s.
/// assert_eq!(huber_2000_sigma(495.0).round(), 612.0);
/// assert_eq!(huber_2000_sigma(500.0).round(), 622.0);
/// assert_eq!(huber_2000_sigma(3500.0).round(), 16_291.0);
/// assert_eq!(huber_2000_sigma(0.0), 0.0);
/// ```
#[must_use]
pub fn huber_2000_sigma(years: f64) -> f64 {
    let n = years.abs();
    DAYS_PER_YEAR * n * (n * HUBER_Q_MS2_PER_YEAR / 3.0 * (1.0 + n / HUBER_M_YEARS)).sqrt() / 1000.0
}

/// The one-sigma uncertainty of the historical models at a year: the
/// standard errors of Morrison and Stephenson's table, interpolated in
/// the year; between 1800 and the IERS era, tapered from a second to
/// five hundredths; before the table's first year, −500, the larger of
/// its standard error there and Huber's walk from it (Espenak and
/// Meeus's Table 3, crux C2020).
fn historical_uncertainty(year: f64) -> f64 {
    let with_sigma: Vec<(f64, f64)> = HISTORICAL_ROWS
        .iter()
        .filter(|row| row.2 > 0)
        .map(|row| (f64::from(row.0), f64::from(row.2)))
        .collect();
    let Some(&(first_year, first_sigma)) = with_sigma.first() else {
        return 1.0;
    };
    let Some(&(last_year, last_sigma)) = with_sigma.last() else {
        return 1.0;
    };
    if year <= first_year {
        // Before the table: Huber's walk back from its first row, never
        // below that row's own standard error.
        return first_sigma.max(huber_2000_sigma(first_year - year));
    }
    if year >= last_year {
        let table_end = DELTA_T_ROWS
            .first()
            .map_or(1956.0, |row| decimal_year(f64::from(row.0) + MJD_EPOCH));
        let fraction = ((table_end - year) / (table_end - last_year)).clamp(0.0, 1.0);
        return 0.05 + (last_sigma - 0.05) * fraction;
    }
    let after = with_sigma.partition_point(|(y, _)| *y <= year);
    let (y0, s0) = with_sigma
        .get(after.saturating_sub(1))
        .copied()
        .unwrap_or((first_year, first_sigma));
    let (y1, s1) = with_sigma
        .get(after)
        .copied()
        .unwrap_or((last_year, last_sigma));
    if y1 > y0 {
        s0 + (s1 - s0) * (year - y0) / (y1 - y0)
    } else {
        s0
    }
}

/// What answered before or after the data a model was fitted to:
/// [`DeltaTSource::Extrapolated`] before Morrison and Stephenson's first
/// year (−500), [`DeltaTSource::Model`] after it.
fn historical_source_at(year: f64) -> DeltaTSource {
    let first = HISTORICAL_ROWS
        .first()
        .map_or(-500.0, |row| f64::from(row.0));
    if year < first {
        DeltaTSource::Extrapolated
    } else {
        DeltaTSource::Model
    }
}

/// The table where measured, the model either side, continuous at the
/// seams.
fn table_then_model(jd: f64) -> DeltaT {
    let mjd = jd - MJD_EPOCH;
    if let Some((seconds, sigma)) = table(mjd) {
        return DeltaT {
            seconds,
            model: DeltaTModel::TableThenModel,
            source: DeltaTSource::Table,
            uncertainty_seconds: Some(sigma),
        };
    }
    let year = decimal_year(jd);
    let (Some(first), Some(last)) = (DELTA_T_ROWS.first(), DELTA_T_ROWS.last()) else {
        return DeltaT {
            seconds: espenak_meeus_2006(year),
            model: DeltaTModel::TableThenModel,
            source: historical_source_at(year),
            uncertainty_seconds: Some(historical_uncertainty(year)),
        };
    };
    let first_year = decimal_year(f64::from(first.0) + MJD_EPOCH);
    let last_year = decimal_year(f64::from(last.0) + MJD_EPOCH);
    if year < first_year {
        // Before the table: the model, with the seam offset tapered away
        // over ten years so the value is continuous at the first row.
        let offset = f64::from(first.1) - espenak_meeus_2006(first_year);
        let taper = (1.0 - (first_year - year) / SEAM_TAPER_YEARS).max(0.0);
        DeltaT {
            seconds: espenak_meeus_2006(year) + offset * taper,
            model: DeltaTModel::TableThenModel,
            source: historical_source_at(year),
            uncertainty_seconds: Some(historical_uncertainty(year)),
        }
    } else {
        // After the table: the end slope for ten years, then the model
        // shifted to meet it, with Huber's (2000) uncertainty walked from
        // the table's last row, never below that row's own (crux C2020).
        let years_beyond = year - last_year;
        let slope = end_slope(last_year);
        let linear = |dy: f64| f64::from(last.1) + slope * dy;
        let seconds = if years_beyond <= LINEAR_TAIL_YEARS {
            linear(years_beyond)
        } else {
            let switch_year = last_year + LINEAR_TAIL_YEARS;
            espenak_meeus_2006(year) + (linear(LINEAR_TAIL_YEARS) - espenak_meeus_2006(switch_year))
        };
        DeltaT {
            seconds,
            model: DeltaTModel::TableThenModel,
            source: DeltaTSource::Extrapolated,
            uncertainty_seconds: Some(f64::from(last.2).max(huber_2000_sigma(years_beyond))),
        }
    }
}

/// The table's slope over its last two years, seconds per year.
fn end_slope(last_year: f64) -> f64 {
    let last = DELTA_T_ROWS.last().map_or(0.0, |row| f64::from(row.1));
    let two_years_back_mjd = (last_year - 2.0 - 2000.0) * DAYS_PER_YEAR + J2000 - MJD_EPOCH;
    match table(two_years_back_mjd) {
        Some((earlier, _)) => (last - earlier) / 2.0,
        None => 0.0,
    }
}

/// Delta T at a UT1 instant under a model.
///
/// ```
/// use teistro_astro::delta_t::{DeltaTModel, delta_t};
/// use teistro_core::quantity::{JulianDay, Ut1};
///
/// let ut1 = JulianDay::<Ut1>::literal(2_451_544.5);
/// let table = delta_t(ut1, DeltaTModel::TableThenModel)?;
/// let fit = delta_t(ut1, DeltaTModel::EspenakMeeus2006)?;
/// // The fit is within a second of the measurement here.
/// assert!((table.seconds - fit.seconds).abs() < 1.0);
/// // An unsourced model refuses.
/// assert!(delta_t(ut1, DeltaTModel::StephensonMorrisonHohenkerk2016).is_err());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// `UNSUPPORTED (unsourced)` for the Stephenson, Morrison and Hohenkerk
/// model until its coefficients are cited.
pub fn delta_t(instant: JulianDay<Ut1>, model: DeltaTModel) -> Result<DeltaT, Error> {
    let jd = instant.get();
    match model {
        DeltaTModel::TableThenModel => Ok(table_then_model(jd)),
        DeltaTModel::EspenakMeeus2006 => {
            // The fit's own half second at least; after 2005, the last
            // year it was fitted to, Huber's walk from there, as Espenak
            // and Meeus calibrate their Table 3.
            let year = decimal_year(jd);
            let (source, sigma) = if year > ESPENAK_MEEUS_LAST_YEAR {
                (
                    DeltaTSource::Extrapolated,
                    huber_2000_sigma(year - ESPENAK_MEEUS_LAST_YEAR),
                )
            } else {
                (historical_source_at(year), historical_uncertainty(year))
            };
            Ok(DeltaT {
                seconds: espenak_meeus_2006(year),
                model,
                source,
                uncertainty_seconds: Some(sigma.max(0.5)),
            })
        }
        DeltaTModel::StephensonMorrisonHohenkerk2016 => Err(Error::unsupported(
            "the Stephenson, Morrison and Hohenkerk (2016) Delta T model is registered but its coefficients are not yet cited",
        )
        .with_detail(Detail::Unsourced)
        .with_field("time.delta_t")
        .with_hint("use TABLE_THEN_MODEL or ESPENAK_MEEUS_2006")),
        DeltaTModel::Custom { seconds } => Ok(DeltaT {
            seconds,
            model,
            source: DeltaTSource::Custom,
            uncertainty_seconds: None,
        }),
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    fn at(jd: f64) -> JulianDay<Ut1> {
        JulianDay::try_new(jd).unwrap()
    }

    #[test]
    fn the_table_answers_inside_its_span_with_the_iers_values() {
        // 2000 January 1.0: the IERS series gives 63.83 s.
        let value = delta_t(at(2_451_544.5), DeltaTModel::TableThenModel).unwrap();
        assert_eq!(value.source, DeltaTSource::Table);
        assert!((value.seconds - 63.83).abs() < 0.02, "{value}");
        assert!(value.uncertainty_seconds.unwrap() < 0.01);
        // 1956 January: 31.3 s; 2020: 69.4 s.
        let early = delta_t(at(2_435_473.9), DeltaTModel::TableThenModel).unwrap();
        assert!((early.seconds - 31.33).abs() < 0.05, "{early}");
        let recent = delta_t(at(2_458_849.5), DeltaTModel::TableThenModel).unwrap();
        assert!((recent.seconds - 69.36).abs() < 0.05, "{recent}");
        assert!(table_source().contains("IERS"));
        assert!(historical_source().contains("Morrison"));
        let (first, last) = table_span().unwrap();
        assert!(first < 2_435_500.0 && last > 2_461_000.0);
        assert!(value.to_string().starts_with("Delta T 63.8"));
    }

    #[test]
    fn the_model_joins_the_table_continuously_at_both_seams() {
        let (first, last) = table_span().unwrap();
        for (seam, inward) in [(first, 0.5), (last, -0.5)] {
            let inside = delta_t(at(seam + inward), DeltaTModel::TableThenModel).unwrap();
            let outside = delta_t(at(seam - inward), DeltaTModel::TableThenModel).unwrap();
            assert!(
                (inside.seconds - outside.seconds).abs() < 0.5,
                "seam {seam}: {inside} vs {outside}"
            );
            let expected = if inward > 0.0 {
                DeltaTSource::Model
            } else {
                DeltaTSource::Extrapolated
            };
            assert_eq!(outside.source, expected);
        }
        // Beyond the table the uncertainty grows and the value stays
        // near the table's end for a decade, not the fit's runaway.
        let five_years_on = delta_t(at(last + 5.0 * 365.25), DeltaTModel::TableThenModel).unwrap();
        assert!(five_years_on.uncertainty_seconds.unwrap() > 0.5);
        let end = delta_t(at(last), DeltaTModel::TableThenModel).unwrap();
        assert!(
            (five_years_on.seconds - end.seconds).abs() < 5.0,
            "{five_years_on} vs {end}"
        );
        let thirty_years_on =
            delta_t(at(last + 30.0 * 365.25), DeltaTModel::TableThenModel).unwrap();
        assert!(thirty_years_on.seconds > end.seconds);
        assert!(
            thirty_years_on.uncertainty_seconds.unwrap()
                > five_years_on.uncertainty_seconds.unwrap()
        );
    }

    #[test]
    fn espenak_meeus_reproduces_its_published_points() {
        // The fits' own anchor values.
        assert!((espenak_meeus_2006(1600.0) - 120.0).abs() < 1e-9);
        assert!((espenak_meeus_2006(1700.0) - 8.83).abs() < 1e-9);
        assert!((espenak_meeus_2006(1900.0) + 2.79).abs() < 1e-9);
        assert!((espenak_meeus_2006(1920.0) - 21.20).abs() < 1e-9);
        assert!((espenak_meeus_2006(2000.0) - 63.86).abs() < 1e-9);
        // The parabola at −500 against Morrison and Stephenson's 17190 s.
        assert!((espenak_meeus_2006(-500.0) - 17_190.0).abs() < 200.0);
        assert!((espenak_meeus_2006(1000.0) - 1570.0).abs() < 50.0);
        // The fit is continuous within a second at its own seams.
        for seam in [
            -500.0, 500.0, 1600.0, 1700.0, 1800.0, 1860.0, 1900.0, 1920.0, 1941.0, 1961.0, 1986.0,
            2005.0, 2050.0, 2150.0,
        ] {
            let a = espenak_meeus_2006(seam - 1e-6);
            let b = espenak_meeus_2006(seam + 1e-6);
            assert!((a - b).abs() < 1.5, "seam {seam}: {a} {b}");
        }
        let modelled = delta_t(at(2_451_544.5), DeltaTModel::EspenakMeeus2006).unwrap();
        assert!((modelled.seconds - 63.86).abs() < 0.1);
        assert_eq!(modelled.source, DeltaTSource::Model);
    }

    #[test]
    fn uncertainties_follow_the_historical_table() {
        // At −500 the table's own 430 s; at 1000 about 55 s; at 1700 5 s;
        // in 1850 under a second; growing again before −500.
        assert!((historical_uncertainty(-500.0) - 430.0).abs() < 1.0);
        assert!((historical_uncertainty(1000.0) - 55.0).abs() < 1.0);
        assert!((historical_uncertainty(1700.0) - 5.0).abs() < 0.5);
        assert!(historical_uncertainty(1850.0) < 1.0 && historical_uncertainty(1850.0) > 0.05);
        assert!(historical_uncertainty(-1500.0) > 430.0);
        // Before −500, Huber's walk back from it once it passes the
        // table's 430 s: Espenak and Meeus's Table 3 gives −1000 ±622 s,
        // −1500 ±1900 s and −4000 ±16291 s.
        assert_eq!(historical_uncertainty(-600.0), 430.0);
        assert_eq!(historical_uncertainty(-1000.0).round(), 622.0);
        assert_eq!(historical_uncertainty(-1500.0).round(), 1900.0);
        assert!((historical_uncertainty(-4000.0) - 16_291.0).abs() < 1.0);
        let medieval = delta_t(at(2_086_308.0), DeltaTModel::TableThenModel).unwrap(); // about 1000 CE
        assert!(medieval.uncertainty_seconds.unwrap() > 50.0);
        assert!((decimal_year(2_086_308.0) - 1000.0).abs() < 0.5);
    }

    #[test]
    fn the_other_models_answer_or_refuse_as_documented() {
        let custom = delta_t(at(J2000), DeltaTModel::Custom { seconds: 64.0 }).unwrap();
        assert_eq!(
            (custom.seconds, custom.source),
            (64.0, DeltaTSource::Custom)
        );
        assert!((custom.days() - 64.0 / 86_400.0).abs() < 1e-15);
        let error = delta_t(at(J2000), DeltaTModel::StephensonMorrisonHohenkerk2016).unwrap_err();
        assert_eq!(error.field(), Some("time.delta_t"));
        assert_eq!(DeltaTModel::default(), DeltaTModel::TableThenModel);
        assert_eq!(
            DeltaTModel::from_knob(teistro_core::settings::DeltaT::Provider),
            None
        );
        assert_eq!(
            DeltaTModel::from_knob(teistro_core::settings::DeltaT::EspenakMeeus2006),
            Some(DeltaTModel::EspenakMeeus2006)
        );
        assert_eq!(DeltaTModel::Custom { seconds: 1.0 }.key(), "CUSTOM");
        let json = serde_json::to_string(&custom).unwrap();
        assert_eq!(serde_json::from_str::<DeltaT>(&json).unwrap(), custom);
    }

    #[test]
    fn beyond_the_data_the_band_is_hubers_walk() {
        // Espenak and Meeus's Table 3 for the future, calibrated at 2005:
        // +2500 ±612 s, +3000 ±1885 s, +5000 ±12341 s.
        for (year, sigma) in [(2500.0, 612.0), (3000.0, 1885.0), (5000.0, 12_341.0)] {
            let jd = J2000 + (year - 2000.0) * DAYS_PER_YEAR;
            let fit = delta_t(at(jd), DeltaTModel::EspenakMeeus2006).unwrap();
            assert_eq!(fit.source, DeltaTSource::Extrapolated);
            assert_eq!(fit.band(), DeltaTBand::Huber2000);
            assert!(
                (fit.uncertainty_seconds.unwrap() - sigma).abs() < 1.0,
                "{year}: {fit}"
            );
        }
        // Inside the fit's data its own half second stands.
        let fitted = delta_t(at(J2000), DeltaTModel::EspenakMeeus2006).unwrap();
        assert_eq!(fitted.uncertainty_seconds, Some(0.5));
        // The default walks from the IERS table's last row instead: in
        // 2050 and 2100 the band is Huber's at that distance.
        let (_, last) = table_span().unwrap();
        for year in [2050.0, 2100.0] {
            let jd = J2000 + (year - 2000.0) * DAYS_PER_YEAR;
            let ahead = delta_t(at(jd), DeltaTModel::TableThenModel).unwrap();
            let expected = huber_2000_sigma((jd - last) / DAYS_PER_YEAR);
            assert_eq!(ahead.source, DeltaTSource::Extrapolated);
            assert!((ahead.uncertainty_seconds.unwrap() - expected).abs() < 1e-9);
        }
        // A day past the table the band is still the measurement's scale.
        let next = delta_t(at(last + 1.0), DeltaTModel::TableThenModel).unwrap();
        assert!(next.uncertainty_seconds.unwrap() < 0.01, "{next}");
    }
}
