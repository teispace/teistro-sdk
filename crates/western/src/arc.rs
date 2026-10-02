//! Arc measures: degrees of a direction turned into years of life, and back
//! (Leo, *The Progressed Horoscope*, pp. 260–262).

use teistro_core::error::Error;

use crate::progression::TROPICAL_YEAR_DAYS;

/// How many degrees of arc measure a year of life.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ArcMeasure {
    /// Ptolemy's: a degree for a year (p. 260).
    Ptolemy,
    /// Naibod's: "the mean daily motion of the Sun to represent one year of
    /// life" (p. 261), `360 / TROPICAL_YEAR_DAYS` degrees, 0° 59′ 8.33″.
    Naibod,
    /// Any number of degrees a year, finite and above zero.
    PerYear(f64),
}

impl ArcMeasure {
    /// Degrees of arc a year measures.
    ///
    /// # Errors
    ///
    /// A [`ArcMeasure::PerYear`] that is not finite and above zero, under
    /// `arc`.
    pub fn degrees_per_year(self) -> Result<f64, Error> {
        match self {
            ArcMeasure::Ptolemy => Ok(1.0),
            ArcMeasure::Naibod => Ok(360.0 / TROPICAL_YEAR_DAYS),
            ArcMeasure::PerYear(degrees) if degrees.is_finite() && degrees > 0.0 => Ok(degrees),
            ArcMeasure::PerYear(degrees) => {
                Err(Error::invalid_arg(format!("{degrees} degrees a year"))
                    .with_field("arc")
                    .with_hint("an arc measure is a finite number of degrees above zero"))
            }
        }
    }

    /// The years of life an arc of `degrees` measures.
    ///
    /// # Errors
    ///
    /// As [`ArcMeasure::degrees_per_year`], or an arc that is not finite.
    pub fn years(self, degrees: f64) -> Result<f64, Error> {
        if !degrees.is_finite() {
            return Err(Error::invalid_arg(format!("an arc of {degrees}°")).with_field("degrees"));
        }
        Ok(degrees / self.degrees_per_year()?)
    }

    /// The arc, in degrees, that `years` of life measure.
    ///
    /// # Errors
    ///
    /// As [`ArcMeasure::degrees_per_year`], or a span that is not finite.
    pub fn degrees(self, years: f64) -> Result<f64, Error> {
        if !years.is_finite() {
            return Err(Error::invalid_arg(format!("{years} years")).with_field("years"));
        }
        Ok(years * self.degrees_per_year()?)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    /// Years, and days of a tropical year, as Leo's table prints them.
    fn years_and_days(measure: ArcMeasure, degrees: f64) -> (f64, f64) {
        let years = measure.years(degrees).unwrap();
        (years.floor(), years.fract() * TROPICAL_YEAR_DAYS)
    }

    #[test]
    fn leos_naibod_example() {
        // "Whole arc of 20° 15′ gives 20 [years] 198 [days] 16 [hours]"
        // (p. 261), the sum of his table's 20° row, "20 106 0", and its
        // 15′ row, "92 16". The rate gives 106.37 days and 92.64 days, so
        // the printed sum is two rows rounded: the whole arc is 199 days.
        let (years, days) = years_and_days(ArcMeasure::Naibod, 20.0);
        assert_eq!(years, 20.0);
        assert!((days - 106.37).abs() < 0.01, "{days}");
        let (_, days) = years_and_days(ArcMeasure::Naibod, 0.25);
        assert!((days - 92.64).abs() < 0.01, "{days}");
        let (years, days) = years_and_days(ArcMeasure::Naibod, 20.25);
        assert_eq!(years, 20.0);
        assert!((days - (198.0 + 16.0 / 24.0)).abs() < 0.5, "{days}");
    }

    #[test]
    fn leos_naibod_table_is_the_rate_to_the_day_and_the_hour() {
        // Rows read off the page image (p. 262): degrees in years and days,
        // minutes in days and hours. Each row is within a day, or an hour
        // and a little (the 40′ row is 1.03 hours long), of the rate; the
        // table rounds some rows down and others up.
        let degrees = [
            (1.0, 1.0, 5.0),
            (2.0, 2.0, 10.0),
            (10.0, 10.0, 53.0),
            (20.0, 20.0, 106.0),
            (30.0, 30.0, 160.0),
            (40.0, 40.0, 213.0),
            (50.0, 50.0, 266.0),
            (60.0, 60.0, 320.0),
        ];
        for (arc, years, days) in degrees {
            let (y, d) = years_and_days(ArcMeasure::Naibod, arc);
            assert_eq!(y, years, "{arc}°");
            assert!((d - days).abs() < 1.0, "{arc}°: {d} against {days}");
        }
        let minutes = [
            (1.0, 6.0, 4.0),
            (2.0, 12.0, 8.0),
            (10.0, 61.0, 18.0),
            (15.0, 92.0, 16.0),
            (20.0, 123.0, 13.0),
            (30.0, 185.0, 7.0),
            (40.0, 247.0, 2.0),
            (60.0, 370.0, 14.0),
        ];
        for (arc, days, hours) in minutes {
            let years = ArcMeasure::Naibod.years(arc / 60.0).unwrap();
            let printed = days + hours / 24.0;
            let d = years * TROPICAL_YEAR_DAYS;
            assert!(
                (d - printed).abs() < 1.1 / 24.0,
                "{arc}′: {d} against {printed}"
            );
        }
    }

    #[test]
    fn ptolemy_is_a_degree_a_year_and_both_ways_agree() {
        assert_eq!(ArcMeasure::Ptolemy.years(33.5).unwrap(), 33.5);
        for measure in [
            ArcMeasure::Ptolemy,
            ArcMeasure::Naibod,
            ArcMeasure::PerYear(1.25),
        ] {
            let years = measure.years(47.3).unwrap();
            assert!((measure.degrees(years).unwrap() - 47.3).abs() < 1e-12);
        }
    }

    #[test]
    fn a_measure_that_is_not_a_rate_is_refused() {
        for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            let error = ArcMeasure::PerYear(bad).years(10.0).unwrap_err();
            assert_eq!(error.field(), Some("arc"));
        }
        assert_eq!(
            ArcMeasure::Naibod.years(f64::NAN).unwrap_err().field(),
            Some("degrees")
        );
    }
}
