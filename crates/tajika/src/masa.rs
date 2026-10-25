//! The Maasa and Dina Pravesha: the year's chart divided by the Sun
//! (`03-design/annual-chart.md`, "The month's and the day's chart").
//!
//! *Tajika Nilakanthi*, Saṃjñā Tantra, the monthly and daily entry, v. 1:
//! the month's chart is cast when the Sun stands at the annual chart's
//! degree, minute and second **one sign further** each month, and the
//! day's when it stands **one degree further** each solar day. The first
//! month's and the first day's chart are the annual chart itself, as the
//! commentary says; its worked example has the annual Sun at 0s 12°57′50″,
//! the second month's at 1s 12°57′50″ and the fourth month's ninth day's
//! at 3s 20°57′50″.
//!
//! So both are one search: the instants the Sun reaches the annual Sun
//! advanced by a whole number of signs or degrees, read in the zodiac the
//! year's return was read in.

use serde::{Deserialize, Serialize};
use teistro_astro::events::{Lattice, Longitudes, Quantity, Search};
use teistro_astro::sidereal::{Sidereal, Zodiac};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Ut1, Utc};
use teistro_port_ephemeris::Body;

use crate::varsha::{Natal, Pravesha, Reading, SIDEREAL_YEAR_DAYS};

/// The months a year divides into, the Sun a sign further each.
///
/// ```
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_tajika::Natal;
/// #
/// # let natal = Natal {
/// #     instant: JulianDay::<Utc>::literal(2_447_995.489_583_333_5),
/// #     sidereal_sun_deg: 0.054_692,
/// #     tropical_sun_deg: 23.779_197,
/// # };
/// # use teistro_astro::ayanamsha::Basis;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_astro::sidereal::Zodiac;
/// # use teistro_astro::Completion;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::settings::OverridePolicy;
/// # use teistro_port_ephemeris::Frame;
/// #
/// # let provider = teistro_ephemeris_builtin::provider::Builtin::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let tropical = completion.longitudes(Frame::CANONICAL);
/// # let zodiac = Zodiac {
/// #     ayanamsha: Some(Ayanamsha::Lahiri.into()),
/// #     basis: Basis::True,
/// #     precession: PrecessionModel::default(),
/// #     delta_t: DeltaTModel::TableThenModel,
/// # };
/// # use teistro_tajika::{Reading, masa_praveshas, praveshas};
/// #
/// # let first = praveshas(&tropical, zodiac, &natal, Reading::Sidereal, 1)?[0];
/// # let months = masa_praveshas(&tropical, zodiac, &natal, &first)?;
/// use teistro_tajika::MONTHS;
///
/// assert_eq!(months.len(), usize::from(MONTHS));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
pub const MONTHS: u8 = 12;

/// The days a month divides into, the Sun a degree further each.
///
/// ```
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_tajika::Natal;
/// #
/// # let natal = Natal {
/// #     instant: JulianDay::<Utc>::literal(2_447_995.489_583_333_5),
/// #     sidereal_sun_deg: 0.054_692,
/// #     tropical_sun_deg: 23.779_197,
/// # };
/// # use teistro_astro::ayanamsha::Basis;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_astro::sidereal::Zodiac;
/// # use teistro_astro::Completion;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::settings::OverridePolicy;
/// # use teistro_port_ephemeris::Frame;
/// #
/// # let provider = teistro_ephemeris_builtin::provider::Builtin::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let tropical = completion.longitudes(Frame::CANONICAL);
/// # let zodiac = Zodiac {
/// #     ayanamsha: Some(Ayanamsha::Lahiri.into()),
/// #     basis: Basis::True,
/// #     precession: PrecessionModel::default(),
/// #     delta_t: DeltaTModel::TableThenModel,
/// # };
/// # use teistro_tajika::{Reading, masa_praveshas, praveshas};
/// #
/// # let first = praveshas(&tropical, zodiac, &natal, Reading::Sidereal, 1)?[0];
/// # let months = masa_praveshas(&tropical, zodiac, &natal, &first)?;
/// use teistro_tajika::{DAYS, dina_praveshas};
///
/// let days = dina_praveshas(&tropical, zodiac, Reading::Sidereal, &months[0])?;
/// assert_eq!(days.len(), usize::from(DAYS));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
pub const DAYS: u8 = 30;

/// One month's chart's instant.
///
/// ```
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_tajika::Natal;
/// #
/// # let natal = Natal {
/// #     instant: JulianDay::<Utc>::literal(2_447_995.489_583_333_5),
/// #     sidereal_sun_deg: 0.054_692,
/// #     tropical_sun_deg: 23.779_197,
/// # };
/// # use teistro_astro::ayanamsha::Basis;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_astro::sidereal::Zodiac;
/// # use teistro_astro::Completion;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::settings::OverridePolicy;
/// # use teistro_port_ephemeris::Frame;
/// #
/// # let provider = teistro_ephemeris_builtin::provider::Builtin::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let tropical = completion.longitudes(Frame::CANONICAL);
/// # let zodiac = Zodiac {
/// #     ayanamsha: Some(Ayanamsha::Lahiri.into()),
/// #     basis: Basis::True,
/// #     precession: PrecessionModel::default(),
/// #     delta_t: DeltaTModel::TableThenModel,
/// # };
/// # use teistro_tajika::{Reading, masa_praveshas, praveshas};
/// #
/// # let first = praveshas(&tropical, zodiac, &natal, Reading::Sidereal, 1)?[0];
/// # let months = masa_praveshas(&tropical, zodiac, &natal, &first)?;
/// // The second month opens with the Sun a sign on from the annual Sun.
/// let second = months[1];
/// assert_eq!((second.year, second.month), (1, 2));
/// assert!((second.sun_deg - (months[0].sun_deg + 30.0)).abs() < 1e-9);
/// assert!((28.0..32.0).contains(&(second.at.get() - months[0].at.get())));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct MasaPravesha {
    /// The year it divides, counted as [`Pravesha::year`] counts it.
    pub year: u16,
    /// The month, `1` the annual chart itself, to `12`.
    pub month: u8,
    /// The instant, UTC.
    pub at: JulianDay<Utc>,
    /// Where the Sun stands then, degrees in the year's zodiac: the annual
    /// Sun a sign further each month.
    pub sun_deg: f64,
}

/// One day's chart's instant.
///
/// ```
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_tajika::Natal;
/// #
/// # let natal = Natal {
/// #     instant: JulianDay::<Utc>::literal(2_447_995.489_583_333_5),
/// #     sidereal_sun_deg: 0.054_692,
/// #     tropical_sun_deg: 23.779_197,
/// # };
/// # use teistro_astro::ayanamsha::Basis;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_astro::sidereal::Zodiac;
/// # use teistro_astro::Completion;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::settings::OverridePolicy;
/// # use teistro_port_ephemeris::Frame;
/// #
/// # let provider = teistro_ephemeris_builtin::provider::Builtin::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let tropical = completion.longitudes(Frame::CANONICAL);
/// # let zodiac = Zodiac {
/// #     ayanamsha: Some(Ayanamsha::Lahiri.into()),
/// #     basis: Basis::True,
/// #     precession: PrecessionModel::default(),
/// #     delta_t: DeltaTModel::TableThenModel,
/// # };
/// # use teistro_tajika::{Reading, masa_praveshas, praveshas};
/// #
/// # let first = praveshas(&tropical, zodiac, &natal, Reading::Sidereal, 1)?[0];
/// # let months = masa_praveshas(&tropical, zodiac, &natal, &first)?;
/// use teistro_tajika::{DinaPravesha, dina_praveshas};
///
/// let days = dina_praveshas(&tropical, zodiac, Reading::Sidereal, &months[3])?;
/// // The month's own chart is its first day's, and each day the Sun is a degree on.
/// assert_eq!(days[0], DinaPravesha::from(months[3]));
/// assert_eq!((days[8].month, days[8].day), (4, 9));
/// assert!((days[8].sun_deg - (months[3].sun_deg + 8.0)).abs() < 1e-9);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct DinaPravesha {
    /// The year, counted as [`Pravesha::year`] counts it.
    pub year: u16,
    /// The month it falls in, `1` to `12`.
    pub month: u8,
    /// The day, `1` the month's chart itself, to `30`.
    pub day: u8,
    /// The instant, UTC.
    pub at: JulianDay<Utc>,
    /// Where the Sun stands then, degrees in the year's zodiac: the
    /// month's Sun a degree further each day.
    pub sun_deg: f64,
}

impl From<MasaPravesha> for DinaPravesha {
    /// The month's chart as its own first day's, which the commentary
    /// says it is.
    fn from(month: MasaPravesha) -> DinaPravesha {
        DinaPravesha {
            year: month.year,
            month: month.month,
            day: 1,
            at: month.at,
            sun_deg: month.sun_deg,
        }
    }
}

/// The search's step for the months, in days: a sign is about thirty days,
/// so ten cannot pass two.
const MONTH_STEP_DAYS: f64 = 10.0;

/// The search's step for the days: the Sun moves 0.95 to 1.02 degrees a
/// day, so half a day cannot pass two degrees.
const DAY_STEP_DAYS: f64 = 0.5;

/// The twelve months of `year`, the first its own instant.
///
/// ```
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_tajika::Natal;
/// #
/// # let natal = Natal {
/// #     instant: JulianDay::<Utc>::literal(2_447_995.489_583_333_5),
/// #     sidereal_sun_deg: 0.054_692,
/// #     tropical_sun_deg: 23.779_197,
/// # };
/// # use teistro_astro::ayanamsha::Basis;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_astro::sidereal::Zodiac;
/// # use teistro_astro::Completion;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::settings::OverridePolicy;
/// # use teistro_port_ephemeris::Frame;
/// #
/// # let provider = teistro_ephemeris_builtin::provider::Builtin::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let tropical = completion.longitudes(Frame::CANONICAL);
/// # let zodiac = Zodiac {
/// #     ayanamsha: Some(Ayanamsha::Lahiri.into()),
/// #     basis: Basis::True,
/// #     precession: PrecessionModel::default(),
/// #     delta_t: DeltaTModel::TableThenModel,
/// # };
/// # use teistro_tajika::{Reading, masa_praveshas, praveshas};
/// #
/// # let first = praveshas(&tropical, zodiac, &natal, Reading::Sidereal, 1)?[0];
/// # let months = masa_praveshas(&tropical, zodiac, &natal, &first)?;
/// use teistro_tajika::mean_praveshas;
///
/// // The first month is the annual chart, and the months run in order.
/// assert_eq!(months[0].at, first.at);
/// assert!(months.windows(2).all(|pair| pair[0].at.get() < pair[1].at.get()));
/// // The mean year places no Sun to divide.
/// let mean = mean_praveshas(&natal, 1)?[0];
/// assert_eq!(masa_praveshas(&tropical, zodiac, &natal, &mean).unwrap_err().field(), Some("reading"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// [`Reading::Mean`], which divides no month (`reading`); whatever the
/// source refuses while searching.
pub fn masa_praveshas<S: Longitudes + ?Sized>(
    tropical: &S,
    zodiac: Zodiac,
    natal: &Natal,
    year: &Pravesha,
) -> Result<Vec<MasaPravesha>, Error> {
    let origin = natal.target_deg(divided(year.reading)?).rem_euclid(360.0);
    let first = MasaPravesha {
        year: year.year,
        month: 1,
        at: year.at,
        sun_deg: origin,
    };
    // Up to the twelfth month's start, a sign short of the next return.
    let to = year.at.get() + SIDEREAL_YEAR_DAYS - 20.0;
    let found = crossings(
        tropical,
        zodiac,
        year.reading,
        Lattice {
            origin_deg: origin,
            step_deg: 30.0,
        },
        (year.at.get() + 1.0, to, MONTH_STEP_DAYS),
    )?;
    let mut months = vec![first];
    months.extend(found.into_iter().map(|(at, sun_deg)| MasaPravesha {
        year: year.year,
        month: ordinal(sun_deg, origin, 30.0),
        at,
        sun_deg,
    }));
    Ok(months)
}

/// The thirty days of `month`, the first its own instant.
///
/// ```
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_tajika::Natal;
/// #
/// # let natal = Natal {
/// #     instant: JulianDay::<Utc>::literal(2_447_995.489_583_333_5),
/// #     sidereal_sun_deg: 0.054_692,
/// #     tropical_sun_deg: 23.779_197,
/// # };
/// # use teistro_astro::ayanamsha::Basis;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_astro::sidereal::Zodiac;
/// # use teistro_astro::Completion;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::settings::OverridePolicy;
/// # use teistro_port_ephemeris::Frame;
/// #
/// # let provider = teistro_ephemeris_builtin::provider::Builtin::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let tropical = completion.longitudes(Frame::CANONICAL);
/// # let zodiac = Zodiac {
/// #     ayanamsha: Some(Ayanamsha::Lahiri.into()),
/// #     basis: Basis::True,
/// #     precession: PrecessionModel::default(),
/// #     delta_t: DeltaTModel::TableThenModel,
/// # };
/// # use teistro_tajika::{Reading, masa_praveshas, praveshas};
/// #
/// # let first = praveshas(&tropical, zodiac, &natal, Reading::Sidereal, 1)?[0];
/// # let months = masa_praveshas(&tropical, zodiac, &natal, &first)?;
/// use teistro_tajika::dina_praveshas;
///
/// let days = dina_praveshas(&tropical, zodiac, Reading::Sidereal, &months[0])?;
/// // About a day apart, all inside the month.
/// assert!(days.windows(2).all(|pair| (0.9..1.1).contains(&(pair[1].at.get() - pair[0].at.get()))));
/// assert!(days[29].at.get() < months[1].at.get());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`masa_praveshas`].
pub fn dina_praveshas<S: Longitudes + ?Sized>(
    tropical: &S,
    zodiac: Zodiac,
    reading: Reading,
    month: &MasaPravesha,
) -> Result<Vec<DinaPravesha>, Error> {
    // The thirtieth day's degree at the Sun's slowest is under 30.5 days
    // on, and the next month's start, a sign on, past 31.
    degrees_on(tropical, zodiac, reading, month, DAYS, 30.6)
}

/// The month's thirty-one knots: the instants the Sun stands a whole
/// number of degrees, none to thirty, on from where `month` opened. The
/// first is the month's own instant and the last the next month's, or the
/// next year's return after the twelfth; a degree between is a "solar
/// day", which is what the month's dasha counts its lords' degrees in.
///
/// ```
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_tajika::Natal;
/// #
/// # let natal = Natal {
/// #     instant: JulianDay::<Utc>::literal(2_447_995.489_583_333_5),
/// #     sidereal_sun_deg: 0.054_692,
/// #     tropical_sun_deg: 23.779_197,
/// # };
/// # use teistro_astro::ayanamsha::Basis;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_astro::sidereal::Zodiac;
/// # use teistro_astro::Completion;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::settings::OverridePolicy;
/// # use teistro_port_ephemeris::Frame;
/// #
/// # let provider = teistro_ephemeris_builtin::provider::Builtin::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let tropical = completion.longitudes(Frame::CANONICAL);
/// # let zodiac = Zodiac {
/// #     ayanamsha: Some(Ayanamsha::Lahiri.into()),
/// #     basis: Basis::True,
/// #     precession: PrecessionModel::default(),
/// #     delta_t: DeltaTModel::TableThenModel,
/// # };
/// # use teistro_tajika::{Reading, masa_praveshas, praveshas};
/// #
/// # let first = praveshas(&tropical, zodiac, &natal, Reading::Sidereal, 1)?[0];
/// # let months = masa_praveshas(&tropical, zodiac, &natal, &first)?;
/// use teistro_tajika::month_knots;
///
/// // Thirty-one knots: the month's own instant to the next month's.
/// let knots = month_knots(&tropical, zodiac, Reading::Sidereal, &months[0])?;
/// assert_eq!(knots.len(), 31);
/// assert_eq!(knots[0], months[0].at);
/// assert!((knots[30].get() - months[1].at.get()).abs() < 1e-5);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`masa_praveshas`]; fewer than thirty-one when the source's coverage
/// ends inside the month.
pub fn month_knots<S: Longitudes + ?Sized>(
    tropical: &S,
    zodiac: Zodiac,
    reading: Reading,
    month: &MasaPravesha,
) -> Result<Vec<JulianDay<Utc>>, Error> {
    // A sign on at the Sun's slowest, under a degree a day, is under 31.6
    // days on.
    Ok(
        degrees_on(tropical, zodiac, reading, month, DAYS + 1, 31.8)?
            .into_iter()
            .map(|day| day.at)
            .collect(),
    )
}

/// The instants the Sun stands `1` (the month's own) to `last` degrees'
/// ordinals on from where `month` opened, searched up to `reach` days on.
fn degrees_on<S: Longitudes + ?Sized>(
    tropical: &S,
    zodiac: Zodiac,
    reading: Reading,
    month: &MasaPravesha,
    last: u8,
    reach: f64,
) -> Result<Vec<DinaPravesha>, Error> {
    divided(reading)?;
    let origin = month.sun_deg;
    let first = DinaPravesha {
        year: month.year,
        month: month.month,
        day: 1,
        at: month.at,
        sun_deg: origin,
    };
    let found = crossings(
        tropical,
        zodiac,
        reading,
        Lattice {
            origin_deg: origin,
            step_deg: 1.0,
        },
        (month.at.get() + 0.25, month.at.get() + reach, DAY_STEP_DAYS),
    )?;
    let mut days = vec![first];
    days.extend(
        found
            .into_iter()
            .map(|(at, sun_deg)| (ordinal(sun_deg, origin, 1.0), at, sun_deg))
            .filter(|(day, _, _)| *day > 1 && *day <= last)
            .map(|(day, at, sun_deg)| DinaPravesha {
                year: month.year,
                month: month.month,
                day,
                at,
                sun_deg,
            }),
    );
    Ok(days)
}

/// The reading a division is read in: the year's own, refused for the
/// mean year, which places no Sun.
///
/// ```
/// use teistro_tajika::{Reading, divided};
///
/// assert_eq!(divided(Reading::Tropical)?, Reading::Tropical);
/// assert_eq!(divided(Reading::Mean).unwrap_err().field(), Some("reading"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// [`Reading::Mean`], named `reading`.
pub fn divided(reading: Reading) -> Result<Reading, Error> {
    if reading == Reading::Mean {
        return Err(Error::invalid_arg(
            "the mean year places no Sun, so it divides into no month or day; \
             ask for the SIDEREAL or the TROPICAL return",
        )
        .with_field("reading"));
    }
    Ok(reading)
}

/// Which division a crossing at `sun_deg` opens, counted from `origin`
/// in steps of `step_deg`, `1` the origin itself.
fn ordinal(sun_deg: f64, origin: f64, step_deg: f64) -> u8 {
    let steps = ((sun_deg - origin).rem_euclid(360.0) / step_deg).round();
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a whole number of steps in a circle, 0 to 360"
    )]
    let steps = steps as u16;
    u8::try_from(steps % 360 + 1).unwrap_or(u8::MAX)
}

/// The Sun's crossings of `lattice` between two instants, at a sampling
/// step, as instants and the longitude reached.
fn crossings<S: Longitudes + ?Sized>(
    tropical: &S,
    zodiac: Zodiac,
    reading: Reading,
    lattice: Lattice,
    (from, to, step): (f64, f64, f64),
) -> Result<Vec<(JulianDay<Utc>, f64)>, Error> {
    let (from, to) = (
        JulianDay::<Ut1>::literal(from),
        JulianDay::<Ut1>::literal(to),
    );
    let sun = Quantity::Longitude(Body::Sun);
    let events = match reading {
        Reading::Tropical => Search::new(tropical, sun, lattice)
            .with_step_days(step)
            .between(from, to),
        Reading::Sidereal | Reading::Mean => {
            Search::new(&Sidereal::over(tropical, zodiac), sun, lattice)
                .with_step_days(step)
                .between(from, to)
        }
    }?;
    Ok(events
        .into_iter()
        .map(|event| {
            (
                JulianDay::<Utc>::literal(event.instant.get()),
                event.boundary_deg,
            )
        })
        .collect())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    #[test]
    fn a_division_is_numbered_from_the_degree_it_reaches() {
        // The commentary's example: the annual Sun at 12°57′50″.
        let origin = 12.0 + 57.0 / 60.0 + 50.0 / 3600.0;
        assert_eq!(ordinal(origin, origin, 30.0), 1);
        assert_eq!(ordinal(origin + 30.0, origin, 30.0), 2);
        assert_eq!(ordinal((origin + 330.0) % 360.0, origin, 30.0), 12);
        // The fourth month's ninth day: 3s 20°57′50″.
        let month = origin + 90.0;
        assert_eq!(ordinal(month + 8.0, month, 1.0), 9);
        // A wrap past Aries counts on.
        assert_eq!(ordinal(2.0, 350.0, 1.0), 13);
    }

    #[test]
    fn the_mean_year_divides_into_nothing() {
        assert_eq!(divided(Reading::Mean).unwrap_err().field(), Some("reading"));
        assert_eq!(divided(Reading::Sidereal).unwrap(), Reading::Sidereal);
    }
}
