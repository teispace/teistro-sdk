//! The Muntha: the birth lagna progressed one sign for each completed
//! year (`03-design/muntha.md`, measured in
//! `03-design/muntha-measured.md`).
//!
//! It is the first of the annual chart's five office-bearers, and the one
//! that takes the year's lordship when none of the others qualifies, so
//! an error here is an error in the year lord as well.
//!
//! **It is progressed by years completed, never by the year of life it
//! opens.** The source states the rule one way and numbers its own worked
//! chart the other — the chart it calls the forty-first year's is the one
//! with forty years complete — and the two are one apart. Over the twelve
//! lagnas and a hundred and twenty years the two readings never once
//! agree on the sign, and they agree on the sign's *lord* in 120 of 1440
//! cases, only where Capricorn meets Aquarius because Saturn rules both.
//! So the argument is named [`completed_years`](muntha) rather than
//! `year`, and a caller holding a [`Pravesha`](crate::Pravesha) passes its
//! `year` straight in, that field counting returns for this reason.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::error::Error;

use crate::varsha::MOST_YEARS;

/// The signs a Muntha steps through.
const SIGNS: u16 = 12;

/// How far the Muntha travels in a month of the year, degrees.
///
/// A sign a year over twelve months. The source gives this figure and the
/// daily one, and gives them only for timing inside a year.
pub const MONTHLY_DEG: f64 = 30.0 / 12.0;

/// How far the Muntha travels in a day, degrees: five arcminutes.
pub const DAILY_DEG: f64 = MONTHLY_DEG / 30.0;

/// Where the Muntha stands inside the sign it has reached (crux C107).
///
/// Both readings put it in the same sign at the year's opening, and they
/// part over the course of the year, so they differ for a Tajika aspect
/// taken to the Muntha and for nothing else. The sign — which is all the
/// house-by-house readings want — is the same either way.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MunthaDegree {
    /// It enters each year at its sign's first degree and crosses the
    /// whole sign during the year.
    ///
    /// The source's own reading: it progresses the Muntha 2°30′ a month
    /// and 5′ a day, which fills a sign in exactly a year only if the
    /// year begins at the sign's start.
    #[default]
    SignStart,
    /// It carries the natal lagna's degree into each new sign.
    ///
    /// Then a year's progression carries it past that sign's end, which
    /// is why the two readings are a knob and not a detail.
    NatalDegree,
}

/// The Muntha at one annual chart.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Muntha {
    /// The years of life complete at the return this stands at, which is
    /// how far the Muntha has been progressed. Zero is the birth itself,
    /// where the Muntha sits on the lagna.
    pub completed_years: u16,
    /// The sign it has reached. Both readings of [`MunthaDegree`] give
    /// this one.
    pub sign: Rashi,
    /// The sign's lord: the **Munthesha**, first of the five
    /// office-bearers.
    pub lord: Graha,
    /// Its longitude at the return, degrees, under [`Self::degree`].
    pub longitude_deg: f64,
    /// Which reading of the degree produced [`Self::longitude_deg`],
    /// carried so a stored answer says what it is.
    pub degree: MunthaDegree,
}

impl Muntha {
    /// Its longitude a fraction of the way through the year, degrees.
    ///
    /// `0.0` is the return itself and `1.0` the next one. The source
    /// gives the rate as 2°30′ a month and 5′ a day, so a caller that
    /// knows the days elapsed divides by the year's own length rather
    /// than assuming one.
    ///
    /// Outside `0.0..=1.0` it keeps progressing, because a caller
    /// comparing a monthly chart against its year wants the continuation
    /// rather than a refusal.
    #[must_use]
    pub fn during(&self, fraction_of_year: f64) -> f64 {
        (self.longitude_deg + 30.0 * fraction_of_year).rem_euclid(360.0)
    }
}

/// The Muntha of a birth, progressed `completed_years` signs.
///
/// The argument is the **years completed**, which is exactly a
/// [`Pravesha::year`](crate::Pravesha::year): pass that field in
/// unchanged. Zero is the birth, where the Muntha sits on the lagna.
///
/// # Errors
///
/// A `completed_years` past [`MOST_YEARS`], named `completed_years`; a
/// lagna that is not a number, named `lagna`.
///
/// # Examples
///
/// The source's worked chart: Leo rising, forty years complete, and the
/// Muntha in Sagittarius.
///
/// ```
/// use teistro_tajika::{MunthaDegree, muntha};
/// use teistro_core::catalogue::{Graha, Rashi};
///
/// let found = muntha(124.5, 40, MunthaDegree::default())?;
/// assert_eq!(found.sign, Rashi::Sagittarius);
/// assert_eq!(found.lord, Graha::Jupiter);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
pub fn muntha(
    natal_lagna_deg: f64,
    completed_years: u16,
    degree: MunthaDegree,
) -> Result<Muntha, Error> {
    if completed_years > MOST_YEARS {
        return Err(Error::invalid_arg(format!(
            "the Muntha is progressed by the years completed, 0 to {MOST_YEARS}, not {completed_years}"
        ))
        .with_field("completed_years"));
    }
    if !natal_lagna_deg.is_finite() {
        return Err(
            Error::invalid_arg("the lagna stood at a longitude that is not a number")
                .with_field("lagna"),
        );
    }
    let lagna = natal_lagna_deg.rem_euclid(360.0);
    let at_birth = lagna / 30.0;
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a longitude folded into 0..360 divides by thirty into 0..12"
    )]
    let birth_sign = at_birth as u16 % SIGNS;
    let reached = (birth_sign + completed_years % SIGNS) % SIGNS;
    let sign = Rashi::from_id(reached)
        .ok_or_else(|| Error::internal("a sign reduced modulo twelve is a sign"))?;
    let longitude_deg = match degree {
        MunthaDegree::SignStart => f64::from(reached) * 30.0,
        MunthaDegree::NatalDegree => f64::from(reached) * 30.0 + lagna % 30.0,
    };
    Ok(Muntha {
        completed_years,
        sign,
        lord: sign.attributes().lord,
        longitude_deg,
        degree,
    })
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::float_cmp,
        reason = "tests fail by panicking, and these longitudes are exact multiples of thirty"
    )]

    use super::{DAILY_DEG, MONTHLY_DEG, MOST_YEARS, Muntha, MunthaDegree, muntha};
    use teistro_core::catalogue::{Graha, Rashi};

    /// The source's worked chart, which is the only value in it that can
    /// check the rule end to end: Leo rising, the forty-first year of
    /// life, forty years complete, the Muntha in Sagittarius.
    #[test]
    fn the_sources_worked_chart_is_reproduced() {
        let found = muntha(124.5, 40, MunthaDegree::SignStart).unwrap();
        assert_eq!(found.sign, Rashi::Sagittarius);
        assert_eq!(found.lord, Graha::Jupiter);
        assert_eq!(found.completed_years, 40);
    }

    /// At the birth the Muntha is the lagna, which is the rule's own
    /// starting statement and the only fixed point it has.
    #[test]
    fn at_the_birth_it_sits_on_the_lagna() {
        for index in 0..12_u16 {
            let sign = Rashi::from_id(index).unwrap();
            let lagna = f64::from(index) * 30.0 + 17.25;
            let found = muntha(lagna, 0, MunthaDegree::NatalDegree).unwrap();
            assert_eq!(found.sign, sign);
            assert_eq!(found.longitude_deg, lagna);
        }
    }

    /// It advances one sign a year and comes home after twelve, for every
    /// lagna, which is the whole of the progression.
    #[test]
    fn it_steps_one_sign_a_year_and_closes_the_circle() {
        for index in 0..12_u16 {
            let lagna = f64::from(index) * 30.0;
            for year in 0..=24_u16 {
                let found = muntha(lagna, year, MunthaDegree::SignStart).unwrap();
                let want = Rashi::from_id((index + year) % 12).unwrap();
                assert_eq!(found.sign, want, "lagna {index}, year {year}");
            }
            let twelve = muntha(lagna, 12, MunthaDegree::SignStart).unwrap();
            let none = muntha(lagna, 0, MunthaDegree::SignStart).unwrap();
            assert_eq!(twelve.sign, none.sign);
        }
    }

    /// The rival reading — the year of life the chart opens, one more than
    /// the years complete — never agrees on the sign, and agrees on the
    /// lord only where Saturn rules both sides of the step.
    ///
    /// That is what makes the off-by-one survivable: a spot check has one
    /// chance in twelve of landing where it cannot be seen.
    #[test]
    fn the_rival_reading_differs_everywhere_and_is_invisible_once_in_twelve() {
        let (mut sign_agrees, mut lord_agrees, mut cases) = (0_u32, 0_u32, 0_u32);
        for index in 0..12_u16 {
            let lagna = f64::from(index) * 30.0 + 4.0;
            for year in 1..=120_u16 {
                let right = muntha(lagna, year, MunthaDegree::SignStart).unwrap();
                let rival = muntha(lagna, year + 1, MunthaDegree::SignStart).unwrap();
                cases += 1;
                sign_agrees += u32::from(right.sign == rival.sign);
                lord_agrees += u32::from(right.lord == rival.lord);
                if right.lord == rival.lord {
                    assert_eq!(
                        (right.sign, rival.sign),
                        (Rashi::Capricorn, Rashi::Aquarius),
                        "only Saturn rules two signs in a row"
                    );
                }
            }
        }
        assert_eq!((cases, sign_agrees, lord_agrees), (1440, 0, 120));
    }

    /// The two degree readings never disagree on the sign at the opening,
    /// and always disagree on the longitude unless the lagna is on a sign
    /// boundary.
    #[test]
    fn the_degree_readings_part_only_inside_the_sign() {
        for index in 0..12_u16 {
            for offset in [0.0_f64, 0.5, 29.75] {
                let lagna = f64::from(index) * 30.0 + offset;
                for year in 0..=13_u16 {
                    let start = muntha(lagna, year, MunthaDegree::SignStart).unwrap();
                    let natal = muntha(lagna, year, MunthaDegree::NatalDegree).unwrap();
                    assert_eq!(start.sign, natal.sign);
                    assert_eq!(natal.longitude_deg - start.longitude_deg, offset);
                }
            }
        }
    }

    /// A year's progression is exactly one sign, and the month and day
    /// rates are that year cut up, not figures of their own.
    #[test]
    fn a_year_of_progression_is_one_sign() {
        let found = muntha(0.0, 3, MunthaDegree::SignStart).unwrap();
        assert_eq!(found.during(0.0), found.longitude_deg);
        assert_eq!(found.during(1.0), found.longitude_deg + 30.0);
        assert_eq!(MONTHLY_DEG * 12.0, 30.0);
        assert_eq!(DAILY_DEG * 360.0, 30.0);
    }

    /// The progression wraps rather than running off the zodiac, so the
    /// last year of the cap is still a longitude.
    #[test]
    fn the_progression_stays_on_the_circle() {
        let found = muntha(345.0, MOST_YEARS, MunthaDegree::NatalDegree).unwrap();
        assert!((0.0..360.0).contains(&found.during(0.9)));
        assert!((0.0..360.0).contains(&found.during(1.0)));
    }

    #[test]
    fn a_year_past_the_cap_is_refused_by_that_field() {
        let why = muntha(0.0, MOST_YEARS + 1, MunthaDegree::SignStart).expect_err("refused");
        assert_eq!(why.field(), Some("completed_years"));
        assert!(why.to_string().contains("0 to 200"), "{why}");
        assert!(muntha(0.0, MOST_YEARS, MunthaDegree::SignStart).is_ok());
    }

    #[test]
    fn a_lagna_that_is_not_a_number_is_refused_by_that_field() {
        let why = muntha(f64::NAN, 1, MunthaDegree::SignStart).expect_err("refused");
        assert_eq!(why.field(), Some("lagna"));
    }

    /// Every answer says which reading made it, as a pravesha does, so a
    /// stored Muntha does not depend on the settings that read it back.
    #[test]
    fn every_muntha_says_which_reading_made_it() {
        for degree in [MunthaDegree::SignStart, MunthaDegree::NatalDegree] {
            let found: Muntha = muntha(100.0, 5, degree).unwrap();
            assert_eq!(found.degree, degree);
        }
        assert_eq!(MunthaDegree::default(), MunthaDegree::SignStart);
    }
}
