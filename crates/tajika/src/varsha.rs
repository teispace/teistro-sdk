//! The Varsha Pravesha: the Sun's return to where it stood at birth
//! (`03-design/annual-chart.md`, measured in
//! `03-design/annual-chart-measured.md`).
//!
//! Three words in "the Sun returns to its natal longitude" are decisions,
//! and each was measured against its rival before it was built:
//!
//! - **sidereal, not tropical** — the tradition's return, and forty years
//!   on the two are 14.2 hours apart, which puts the lagna 177° away;
//! - **true, not mean** — 14.5 minutes and 5.6° of lagna at forty;
//! - **read as the chart reads it** — a `Frame`'s own `Zodiac::Sidereal`
//!   applies the *mean* ayanamsha and a founded chart applies the
//!   *nutated* one, 18.46 arcseconds apart, which for the Sun is about
//!   seven minutes of time and about two degrees of lagna. This module
//!   takes a [`Zodiac`] and never builds a sidereal frame of its own.
//!
//! None of that is this module's opinion: the first is the tradition's,
//! and the third was a defect until the pass caught it.

use serde::{Deserialize, Serialize};
use teistro_astro::events::{Lattice, Longitudes, Quantity, Search};
use teistro_astro::sidereal::{Sidereal, Zodiac};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Ut1, Utc};
use teistro_port_ephemeris::Body;

/// The sidereal year, days (IERS 2010): what the [`Reading::Mean`] rival
/// steps by, and what a true return averages.
pub const SIDEREAL_YEAR_DAYS: f64 = 365.256_363_004;

/// The step the return's search samples at, days.
///
/// The Sun never turns and moves about a degree a day, so ten days cannot
/// pass the one target twice between two samples. The default step is
/// capped at one day, which samples a working lifetime of every chart for
/// no gain: the measured page is byte-identical at one day and at ten.
pub const STEP_DAYS: f64 = 10.0;

/// The most returns one call will answer.
///
/// A cap rather than a silent truncation: a caller asking for a thousand
/// years is refused by name rather than waiting for a search nobody wants.
///
/// Public because the Muntha is progressed over the same span and refuses
/// by the same bound: one cap, one place.
pub const MOST_YEARS: u16 = 200;

/// Which longitude a return returns to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Reading {
    /// The natal **sidereal** longitude, read on the chart's own ayanamsha
    /// basis: the tradition's, and the default.
    #[default]
    Sidereal,
    /// The natal **tropical** longitude: the Western solar return, which a
    /// consumer building a Western tool asks for by name.
    ///
    /// Forty years on it is about fourteen hours from the sidereal return
    /// and puts the lagna most of a circle away, so it is never a fallback
    /// for the sidereal one — only ever a choice.
    Tropical,
    /// A whole sidereal year for each year of life, from birth.
    ///
    /// The older arithmetic, still taught, and the only reading that needs
    /// no ephemeris. About fifteen minutes from the true return at forty,
    /// which is a house boundary when the birth is near one.
    Mean,
}

impl Reading {
    /// Every reading, the tradition's first.
    pub const ALL: [Reading; 3] = [Reading::Sidereal, Reading::Tropical, Reading::Mean];

    /// The member's key, as serde writes it and every binding reads it
    /// back: `SIDEREAL`.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Reading::Sidereal => "SIDEREAL",
            Reading::Tropical => "TROPICAL",
            Reading::Mean => "MEAN",
        }
    }
}

/// What a return is computed from: the birth, and the Sun where it stood.
///
/// Both longitudes are taken rather than one and an ayanamsha, because
/// the chart that founded them already holds both and recomputing either
/// here would be a second answer to a question already answered.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Natal {
    /// The birth, UTC.
    pub instant: JulianDay<Utc>,
    /// The Sun's sidereal longitude at birth, degrees.
    pub sidereal_sun_deg: f64,
    /// Its tropical longitude at birth, degrees.
    pub tropical_sun_deg: f64,
}

impl Natal {
    /// The longitude a reading returns to.
    #[must_use]
    pub const fn target_deg(&self, reading: Reading) -> f64 {
        match reading {
            Reading::Tropical => self.tropical_sun_deg,
            Reading::Sidereal | Reading::Mean => self.sidereal_sun_deg,
        }
    }
}

/// One annual chart's instant.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Pravesha {
    /// How many years the native has completed at this instant: `1` is
    /// the first return, a year after birth.
    ///
    /// Counted in **returns** and not in years of life, because the two
    /// namings differ by one and both are in use — the instant that
    /// completes a native's first year is the one that opens their
    /// second. Everything Tajika progresses by this number, so a reader
    /// that takes it for an age off by one is off by a whole sign in
    /// every judgement made from the chart.
    pub year: u16,
    /// The instant, UTC.
    pub at: JulianDay<Utc>,
    /// Which reading produced it, carried so a stored answer says what it
    /// is rather than depending on the settings that read it back.
    pub reading: Reading,
}

/// The returns of a birth, in order, up to and including `through`.
///
/// # Errors
///
/// A `through` of zero or past [`MOST_YEARS`], named `through`; a natal
/// longitude that is not a number, named `natal`; whatever the source
/// refuses while searching.
///
/// # Notes
///
/// The window **starts a step in**. The search brackets a crossing
/// between two samples, so it reads up to a step below where it is told
/// to start, and a birth at the very edge of an ephemeris then fails on
/// an instant the caller never asked about. A step costs nothing: the
/// first return is a year away.
///
/// Fewer returns than asked for is the answer, not an error: an ephemeris
/// that ends before a birth's fortieth year has said so, and a caller
/// comparing lengths learns it from the answer.
pub fn praveshas<S: Longitudes + ?Sized>(
    tropical: &S,
    zodiac: Zodiac,
    natal: &Natal,
    reading: Reading,
    through: u16,
) -> Result<Vec<Pravesha>, Error> {
    check(natal, through)?;
    if reading == Reading::Mean {
        return mean_praveshas(natal, through);
    }
    let sidereal = Sidereal {
        tropical,
        ayanamsha: zodiac.ayanamsha,
        basis: zodiac.basis,
        precession: zodiac.precession,
        delta_t: zodiac.delta_t,
    };
    let lattice = Lattice {
        origin_deg: natal.target_deg(reading).rem_euclid(360.0),
        step_deg: 0.0,
    };
    let from = JulianDay::<Ut1>::literal(natal.instant.get() + STEP_DAYS + 1.0);
    let to = JulianDay::<Ut1>::literal(
        natal.instant.get() + f64::from(through) * (SIDEREAL_YEAR_DAYS + 1.0),
    );
    let events = match reading {
        Reading::Tropical => Search::new(tropical, Quantity::Longitude(Body::Sun), lattice)
            .with_step_days(STEP_DAYS)
            .between(from, to),
        // `Mean` returned above; the sidereal reading is the one that
        // needs the chart's own zodiac rather than a frame's.
        Reading::Sidereal | Reading::Mean => {
            Search::new(&sidereal, Quantity::Longitude(Body::Sun), lattice)
                .with_step_days(STEP_DAYS)
                .between(from, to)
        }
    }?;
    Ok(events
        .into_iter()
        .zip(1..)
        .take(usize::from(through))
        .map(|(event, year)| Pravesha {
            year,
            at: JulianDay::<Utc>::literal(event.instant.get()),
            reading,
        })
        .collect())
}

/// The returns under [`Reading::Mean`]: a whole sidereal year each time.
///
/// Its own entry point because it needs no ephemeris at all, and a caller
/// that wants the old arithmetic should not have to hold a provider to
/// get it.
///
/// # Errors
///
/// As [`praveshas`], but nothing can be refused by a source.
pub fn mean_praveshas(natal: &Natal, through: u16) -> Result<Vec<Pravesha>, Error> {
    check(natal, through)?;
    Ok((1..=through)
        .map(|year| Pravesha {
            year,
            at: JulianDay::<Utc>::literal(
                natal.instant.get() + f64::from(year) * SIDEREAL_YEAR_DAYS,
            ),
            reading: Reading::Mean,
        })
        .collect())
}

/// The years a caller may ask for, refused by name when they are not.
///
/// Public because a caller that **caps** `through` — to a provider's
/// coverage, say — must refuse a nonsense year before capping it, or a
/// request for none comes back as an empty answer instead of a refusal.
/// One rule, one place, whoever asks.
///
/// # Errors
///
/// A `through` of zero or past two hundred, named `through`.
pub fn years(through: u16) -> Result<u16, Error> {
    if through == 0 || through > MOST_YEARS {
        return Err(Error::invalid_arg(format!(
            "a return is asked for by the years it completes, 1 to {MOST_YEARS}, not {through}"
        ))
        .with_field("through"));
    }
    Ok(through)
}

/// What every entry point refuses, in one place so the two cannot drift.
fn check(natal: &Natal, through: u16) -> Result<(), Error> {
    years(through)?;
    if !natal.sidereal_sun_deg.is_finite() || !natal.tropical_sun_deg.is_finite() {
        return Err(
            Error::invalid_arg("the Sun stood at a longitude that is not a number")
                .with_field("natal"),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::float_cmp,
        reason = "tests fail by panicking, index what they asked for, and \
                  compare longitudes this module copies rather than computes"
    )]

    use super::*;

    fn natal() -> Natal {
        Natal {
            instant: JulianDay::<Utc>::literal(2_447_995.489_583_333_5),
            sidereal_sun_deg: 0.054_692_050_319_381_735,
            tropical_sun_deg: 23.779_196_990_611_27,
        }
    }

    #[test]
    fn the_mean_reading_steps_a_sidereal_year_and_needs_nothing() {
        let found = mean_praveshas(&natal(), 40).unwrap();
        assert_eq!(found.len(), 40);
        assert_eq!(found[0].year, 1);
        assert_eq!(found[39].year, 40);
        for pair in found.windows(2) {
            let gap = pair[1].at.get() - pair[0].at.get();
            assert!((gap - SIDEREAL_YEAR_DAYS).abs() < 1e-9, "{gap}");
        }
        // The first is a year after birth, not a year after nothing.
        let first = found[0].at.get() - natal().instant.get();
        assert!((first - SIDEREAL_YEAR_DAYS).abs() < 1e-9, "{first}");
    }

    /// `year` counts **returns**, so year `n` stands `n` years after the
    /// birth and the native is aged exactly `n` there.
    ///
    /// The rival reading — that year `n` opens the native's `n`-th year
    /// of life, and so falls at age `n - 1` — is the off-by-one every
    /// Tajika progression inherits, and it is a whole sign of Muntha.
    /// Nothing but this test says which of the two this module means.
    #[test]
    fn a_year_counts_returns_and_not_years_of_life() {
        let natal = natal();
        let found = mean_praveshas(&natal, 40).unwrap();
        for one in &found {
            let stood = one.at.get() - natal.instant.get();
            let completed = f64::from(one.year) * SIDEREAL_YEAR_DAYS;
            assert!(
                (stood - completed).abs() < 1e-9,
                "year {} stood {stood} days out, not {completed}",
                one.year
            );
            // And not the rival: a year short of that.
            assert!((stood - (completed - SIDEREAL_YEAR_DAYS)).abs() > 1.0);
        }
    }

    #[test]
    fn every_pravesha_says_which_reading_made_it() {
        let found = mean_praveshas(&natal(), 3).unwrap();
        assert!(found.iter().all(|one| one.reading == Reading::Mean));
    }

    #[test]
    fn a_year_outside_the_cap_is_refused_by_that_field() {
        for through in [0, MOST_YEARS + 1] {
            let why = mean_praveshas(&natal(), through).expect_err("refused");
            assert_eq!(why.field(), Some("through"));
            assert!(why.to_string().contains("1 to 200"), "{why}");
        }
        assert!(mean_praveshas(&natal(), MOST_YEARS).is_ok());
    }

    #[test]
    fn a_sun_that_is_not_a_number_is_refused_by_the_birth() {
        let broken = Natal {
            sidereal_sun_deg: f64::NAN,
            ..natal()
        };
        assert_eq!(
            mean_praveshas(&broken, 1).unwrap_err().field(),
            Some("natal")
        );
    }

    #[test]
    fn a_reading_names_the_longitude_it_returns_to() {
        let natal = natal();
        assert_eq!(natal.target_deg(Reading::Sidereal), natal.sidereal_sun_deg);
        assert_eq!(natal.target_deg(Reading::Mean), natal.sidereal_sun_deg);
        assert_eq!(natal.target_deg(Reading::Tropical), natal.tropical_sun_deg);
        // The two are the ayanamsha apart, which is the whole reason the
        // readings are different instants.
        let apart = natal.tropical_sun_deg - natal.sidereal_sun_deg;
        assert!((apart - 23.724_504_94).abs() < 1e-6, "{apart}");
    }

    #[test]
    fn the_default_reading_is_the_tradition_s() {
        assert_eq!(Reading::default(), Reading::Sidereal);
        let json = serde_json::to_string(&Reading::Sidereal).unwrap();
        assert_eq!(json, r#""SIDEREAL""#);
        assert_eq!(
            serde_json::from_str::<Reading>(r#""TROPICAL""#).unwrap(),
            Reading::Tropical
        );
    }
}
