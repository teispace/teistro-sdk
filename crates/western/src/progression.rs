//! Progressions: an instant of the sky read as an instant of the life.
//!
//! Leo's "one day measures a year" (*The Progressed Horoscope*, Appendix
//! IV, p. 303), his lunar rates (p. 311) and the year measures C236 weighs.

use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};

/// The mean tropical year, in days: the year a day measures by default,
/// and the one the mean Sun's right ascension takes to come round.
pub const TROPICAL_YEAR_DAYS: f64 = 365.242_189;

/// The Julian year, in days.
const JULIAN_YEAR_DAYS: f64 = 365.25;

/// The mean synodic month, in days: Leo's "synodic month, 29·53059
/// days" (p. 295), the month his month-for-a-year horoscope counts.
pub const SYNODIC_MONTH_DAYS: f64 = 29.530_588_9;

/// The mean sidereal month, in days: the month later authors count for
/// the tertiary progression (C238).
pub const SIDEREAL_MONTH_DAYS: f64 = 27.321_661_5;

/// One span of time a rate is stated in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Span {
    /// A day.
    Day,
    /// A mean synodic month, new Moon to new Moon.
    SynodicMonth,
    /// A mean sidereal month, the Moon back at the same star.
    SiderealMonth,
    /// A year, as long as the progression's [`YearMeasure`] makes it.
    Year,
    /// Any number of days, finite and above zero.
    Days(f64),
}

impl Span {
    /// The span in days, a year taken at `year_days`.
    fn days(self, year_days: f64) -> f64 {
        match self {
            Span::Day => 1.0,
            Span::SynodicMonth => SYNODIC_MONTH_DAYS,
            Span::SiderealMonth => SIDEREAL_MONTH_DAYS,
            Span::Year => year_days,
            Span::Days(days) => days,
        }
    }
}

/// A rate: one `sky` span of the ephemeris measures one `life` span of the
/// native's life.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rate {
    /// The span of the sky's time.
    pub sky: Span,
    /// The span of the life it measures.
    pub life: Span,
}

impl Rate {
    /// A day for a year: Leo's progressed horoscope (p. 295), the
    /// secondary progression.
    pub const SECONDARY: Rate = Rate {
        sky: Span::Day,
        life: Span::Year,
    };

    /// A day for a month: Leo's second "Progressed Lunar" horoscope (p.
    /// 311), the tertiary progression. His table names no month, so the
    /// synodic one his first lunar horoscope counts stands in (C238);
    /// `Rate { sky: Span::Day, life: Span::SiderealMonth }` is the other.
    pub const TERTIARY: Rate = Rate {
        sky: Span::Day,
        life: Span::SynodicMonth,
    };

    /// A month for a year: Leo's "Progressed Lunar" horoscope, a map "for
    /// as many months after birth as subject is years old" (p. 295), the
    /// minor progression.
    pub const MINOR: Rate = Rate {
        sky: Span::SynodicMonth,
        life: Span::Year,
    };
}

/// How long a year of life is, against the calendar (C236).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum YearMeasure {
    /// The mean tropical year, [`TROPICAL_YEAR_DAYS`]: what the mean Sun
    /// takes to come back, and every modern implementation's year.
    Tropical,
    /// The Julian year of 365.25 days.
    Julian,
    /// Leo's rule by sidereal time at noon (Appendix V, pp. 304–305), for
    /// a day for a year only. Each Greenwich noon of the ephemeris
    /// measures to one day of the calendar, and the hours after it are
    /// counted in sidereal time, so a mean solar day of the sky measures a
    /// year and a day of life. The last 3m 56s of each ephemeris day before
    /// noon measure to the same dates as the first of the next, so the rule
    /// is many-to-one there, and [`Progression::sky_at`] answers the later
    /// instant. The day of birth is the Greenwich civil day, and
    /// longitude does not enter.
    NoonSiderealTime,
}

/// A progression: the rate, and the year it counts.
///
/// ```
/// use teistro_western::{Progression, Rate, YearMeasure};
///
/// let minor = Progression::SECONDARY.with_rate(Rate::MINOR);
/// assert_eq!(minor.year, YearMeasure::Tropical);
/// // A synodic month of sky measures a year of life.
/// assert!((minor.life_days_per_sky_day().unwrap() - 365.242_189 / 29.530_588_9).abs() < 1e-12);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Progression {
    /// The rate.
    pub rate: Rate,
    /// The year a [`Span::Year`] in the rate is.
    pub year: YearMeasure,
}

impl Progression {
    /// A day for a tropical year: the secondary progression as it is
    /// computed today.
    pub const SECONDARY: Progression = Progression {
        rate: Rate::SECONDARY,
        year: YearMeasure::Tropical,
    };

    /// A day for a year by Leo's sidereal time at noon: the day his
    /// Appendix V prints.
    pub const LEO: Progression = Progression {
        rate: Rate::SECONDARY,
        year: YearMeasure::NoonSiderealTime,
    };

    /// The same progression at another rate.
    #[must_use]
    pub const fn with_rate(self, rate: Rate) -> Progression {
        Progression { rate, ..self }
    }

    /// The same progression under another year.
    #[must_use]
    pub const fn with_year(self, year: YearMeasure) -> Progression {
        Progression { year, ..self }
    }

    /// Days of life a day of sky measures, when that is one number:
    /// `None` under [`YearMeasure::NoonSiderealTime`], which is not
    /// proportional.
    #[must_use]
    pub fn life_days_per_sky_day(&self) -> Option<f64> {
        let year_days = match self.year {
            YearMeasure::Tropical => TROPICAL_YEAR_DAYS,
            YearMeasure::Julian => JULIAN_YEAR_DAYS,
            YearMeasure::NoonSiderealTime => return None,
        };
        Some(self.rate.life.days(year_days) / self.rate.sky.days(year_days))
    }

    /// Refuses a rate whose spans are not finite and above zero, and Leo's
    /// noon rule at any rate but a day for a year.
    ///
    /// # Errors
    ///
    /// Naming `rate.sky`, `rate.life` or `year`.
    pub fn check(&self) -> Result<(), Error> {
        for (span, field) in [(self.rate.sky, "rate.sky"), (self.rate.life, "rate.life")] {
            if let Span::Days(days) = span {
                if !(days.is_finite() && days > 0.0) {
                    return Err(Error::invalid_arg(format!("a span of {days} days"))
                        .with_field(field)
                        .with_hint("a span is a finite number of days above zero"));
                }
            }
        }
        if self.year == YearMeasure::NoonSiderealTime && self.rate != Rate::SECONDARY {
            return Err(Error::invalid_arg(
                "Leo's rule by sidereal time at noon dates a day for a year only",
            )
            .with_field("year")
            .with_hint("use the tropical or the Julian year for another rate"));
        }
        Ok(())
    }

    /// The instant of the sky that measures `life`, an instant of the life
    /// of someone born at `birth`. Before birth is allowed: that is the
    /// converse progression.
    ///
    /// # Errors
    ///
    /// What [`Progression::check`] refuses, or an instant that is not
    /// finite.
    pub fn sky_at(
        &self,
        birth: JulianDay<Utc>,
        life: JulianDay<Utc>,
    ) -> Result<JulianDay<Utc>, Error> {
        self.check()?;
        let sky = match self.life_days_per_sky_day() {
            Some(ratio) => birth.get() + (life.get() - birth.get()) / ratio,
            None => NoonRule::of(birth).sky(life.get()),
        };
        finite(sky, "life")
    }

    /// The instant of the life that `sky`, an instant of the ephemeris,
    /// measures for someone born at `birth`: the inverse of
    /// [`Progression::sky_at`].
    ///
    /// # Errors
    ///
    /// What [`Progression::check`] refuses, or an instant that is not
    /// finite.
    pub fn life_at(
        &self,
        birth: JulianDay<Utc>,
        sky: JulianDay<Utc>,
    ) -> Result<JulianDay<Utc>, Error> {
        self.check()?;
        let life = match self.life_days_per_sky_day() {
            Some(ratio) => birth.get() + (sky.get() - birth.get()) * ratio,
            None => NoonRule::of(birth).life(sky.get()),
        };
        finite(life, "sky")
    }
}

impl Default for Progression {
    fn default() -> Progression {
        Progression::SECONDARY
    }
}

/// An instant the arithmetic made, refused when it is not finite.
fn finite(jd: f64, field: &str) -> Result<JulianDay<Utc>, Error> {
    JulianDay::try_new(jd).map_err(|_| {
        Error::invalid_arg("the instant is too far from birth to measure").with_field(field)
    })
}

/// Leo's rule by sidereal time at noon, in closed form.
///
/// Sidereal time at Greenwich noon advances by `1/Y` of a day each day, so
/// the day whose noon has a given sidereal time comes round once in `Y`
/// days, and an hour of sidereal time is `Y/24` days of the calendar. Leo
/// finds the day birth's noon measures to by the sidereal time from birth
/// to that noon, then adds the hours past an ephemeris noon in sidereal
/// time (pp. 304–305). With `N` the noon of the day of birth, `δ` the birth's
/// offset from it, and `k` whole days and `f` a fraction past it:
///
/// `life = N + k·Y + r·Y·(f − δ)`, where `r = 1 + 1/Y` turns solar time
/// into sidereal time.
struct NoonRule {
    /// Greenwich noon of the civil day of birth.
    noon: f64,
    /// The birth's offset from that noon, in days, in `[-0.5, 0.5)`.
    offset: f64,
}

impl NoonRule {
    /// Days of calendar a day of sidereal time measures: a year and a day.
    const SIDEREAL_DAY: f64 = TROPICAL_YEAR_DAYS + 1.0;

    fn of(birth: JulianDay<Utc>) -> NoonRule {
        let noon = (birth.get() + 0.5).floor();
        NoonRule {
            noon,
            offset: birth.get() - noon,
        }
    }

    /// The life an instant of sky measures.
    fn life(&self, sky: f64) -> f64 {
        let past = sky - self.noon;
        let days = past.floor();
        self.noon + days * TROPICAL_YEAR_DAYS + (past - days - self.offset) * Self::SIDEREAL_DAY
    }

    /// The instant of sky that measures a life: on the dates two ephemeris
    /// days both reach, the later.
    fn sky(&self, life: f64) -> f64 {
        let past = life - self.noon + self.offset * Self::SIDEREAL_DAY;
        let days = (past / TROPICAL_YEAR_DAYS).floor();
        self.noon + days + (past - days * TROPICAL_YEAR_DAYS) / Self::SIDEREAL_DAY
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    /// Leo's birth: London, 7 August 1860, 5.49 a.m. (p. 305), which is
    /// Greenwich time to the half minute.
    const BIRTH: f64 = 2_400_629.742_361_111;
    /// 10.40 a.m. on 22 September 1860, when the progressed Moon reaches
    /// Capricorn 5° 11′, sesquiquadrate the radical Mercury (p. 304).
    const CONTACT: f64 = 2_400_675.944_444_444;
    /// 0h UT on 21 October 1906.
    const OCTOBER_21: f64 = 2_417_504.5;
    /// 0h UT on 22 October 1906.
    const OCTOBER_22: f64 = 2_417_505.5;

    fn jd(value: f64) -> JulianDay<Utc> {
        JulianDay::try_new(value).unwrap()
    }

    fn life(progression: Progression, sky: f64) -> f64 {
        progression.life_at(jd(BIRTH), jd(sky)).unwrap().get()
    }

    #[test]
    fn leos_contact_falls_on_the_22nd_by_his_rule_and_the_21st_by_a_year() {
        // "we find this date to be the 22nd of October, and not the 19th"
        // (p. 305).
        let leo = life(Progression::LEO, CONTACT);
        assert!((OCTOBER_22..OCTOBER_22 + 1.0).contains(&leo), "{leo}");
        // His printed sidereal times put it 55 s of sidereal time past
        // noon, 0.23 of a day; the closed form's straight line puts it at
        // 0.15, two hours of life apart.
        assert!((leo - (OCTOBER_22 + 0.5 + 0.23)).abs() < 0.1, "{leo}");

        // The tropical year: 21 October, about 4.40 UT, a day and a half
        // before his. The Julian year gives the 21st too, at about 13h.
        let tropical = life(Progression::SECONDARY, CONTACT);
        assert!((tropical - (OCTOBER_21 + 0.192)).abs() < 0.01, "{tropical}");
        let julian = life(
            Progression::SECONDARY.with_year(YearMeasure::Julian),
            CONTACT,
        );
        assert!((julian - (OCTOBER_21 + 0.553)).abs() < 0.01, "{julian}");
    }

    #[test]
    fn leos_noon_measures_to_the_tenth_of_november() {
        // Noon on the day of birth measures to the day whose noon has
        // sidereal time 15h 16m 56s; in 1906 that is "November 10th"
        // (p. 305), whose noon he prints at 15h 14m 56s, so the instant is
        // 120 s of sidereal time, 0.507 of a day, past it.
        let noon_46 = 2_400_630.0 + 46.0;
        let leo = life(Progression::LEO, noon_46);
        let november_10_noon = 2_417_525.0;
        assert!((leo - (november_10_noon + 0.507)).abs() < 0.05, "{leo}");
    }

    #[test]
    fn leos_progressed_map_measures_to_the_8th_of_august_by_the_tropical_year() {
        // "5.49 a.m. 22/9/'60 measures to the year commencing August 8th,
        // 1906" (p. 35): the birth's own hour on the forty-sixth day. That
        // is his approximate reckoning, which his Appendix V corrects by a
        // day for noon (p. 305); the tropical year agrees with it, and his
        // rule by sidereal time gives the 9th.
        let august_8 = 2_417_430.5;
        let tropical = life(Progression::SECONDARY, BIRTH + 46.0);
        assert!((august_8..august_8 + 1.0).contains(&tropical), "{tropical}");
        let leo = life(Progression::LEO, BIRTH + 46.0);
        assert!((august_8 + 1.0..august_8 + 2.0).contains(&leo), "{leo}");
    }

    #[test]
    fn birth_measures_to_itself_or_the_next_noon_by_leos_rule() {
        // Before noon, the rule's day of birth is the next day's noon; after
        // it, the day's own noon.
        let leo = life(Progression::LEO, BIRTH);
        assert!((leo - 2_400_631.0).abs() < 1e-9, "{leo}");
        let evening = 2_400_630.25;
        let after = Progression::LEO.life_at(jd(evening), jd(evening)).unwrap();
        assert!((after.get() - 2_400_630.0).abs() < 1e-9);
    }

    #[test]
    fn every_measure_answers_both_ways() {
        let rates = [
            Rate::SECONDARY,
            Rate::TERTIARY,
            Rate::MINOR,
            Rate {
                sky: Span::Days(2.0),
                life: Span::SiderealMonth,
            },
        ];
        let years = [YearMeasure::Tropical, YearMeasure::Julian];
        for rate in rates {
            for year in years {
                let progression = Progression { rate, year };
                for age in [-3.0, 0.0, 0.3, 17.0, 80.5] {
                    let at = jd(BIRTH + age * 365.25);
                    let sky = progression.sky_at(jd(BIRTH), at).unwrap();
                    let back = progression.life_at(jd(BIRTH), sky).unwrap();
                    assert!(
                        (back.get() - at.get()).abs() < 1e-6,
                        "{rate:?} {year:?} {age}"
                    );
                }
            }
        }
        // Leo's rule from life to sky and back, on every day of a year.
        for day in 0..400 {
            let at = jd(OCTOBER_21 + f64::from(day) + 0.37);
            let sky = Progression::LEO.sky_at(jd(BIRTH), at).unwrap();
            let back = Progression::LEO.life_at(jd(BIRTH), sky).unwrap();
            assert!((back.get() - at.get()).abs() < 1e-6, "{day}");
        }
    }

    #[test]
    fn leos_rule_reaches_some_dates_twice_and_answers_the_later_sky() {
        // The last 3m 56s before an ephemeris noon measure a day of life
        // the next noon measures too.
        let noon = 2_400_630.0 + 30.0;
        let late = noon - 0.001;
        let life = life(Progression::LEO, late);
        let sky = Progression::LEO.sky_at(jd(BIRTH), jd(life)).unwrap().get();
        assert!(sky > noon && sky - late < 1.0 / TROPICAL_YEAR_DAYS, "{sky}");
    }

    #[test]
    fn a_tertiary_day_is_a_synodic_month_and_a_minor_month_a_year() {
        let birth = jd(BIRTH);
        let tertiary = Progression::SECONDARY.with_rate(Rate::TERTIARY);
        let month_on = jd(BIRTH + SYNODIC_MONTH_DAYS);
        assert!((tertiary.sky_at(birth, month_on).unwrap().get() - BIRTH - 1.0).abs() < 1e-9);
        let minor = Progression::SECONDARY.with_rate(Rate::MINOR);
        let year_on = jd(BIRTH + TROPICAL_YEAR_DAYS);
        let sky = minor.sky_at(birth, year_on).unwrap().get();
        assert!((sky - BIRTH - SYNODIC_MONTH_DAYS).abs() < 1e-9);
    }

    #[test]
    fn a_rate_is_refused_by_the_field_it_breaks() {
        let zero = Progression::SECONDARY.with_rate(Rate {
            sky: Span::Days(0.0),
            life: Span::Year,
        });
        let error = zero.check().unwrap_err();
        assert_eq!(error.field(), Some("rate.sky"));
        let lunar_leo = Progression::LEO.with_rate(Rate::TERTIARY);
        let error = lunar_leo.sky_at(jd(BIRTH), jd(BIRTH)).unwrap_err();
        assert_eq!(error.field(), Some("year"));
    }
}
