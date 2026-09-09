//! The Indian lunisolar month: which month a moment falls in, and
//! whether that month is intercalary or omitted.
//!
//! A lunar month runs from one new moon to the next and takes its name
//! from the solar month it belongs to. A lunar month is about 29.53 days
//! and a solar month about 30.44, so it usually holds exactly one
//! sankranti — the Sun's entry into a sign — and takes that month's name.
//!
//! Usually. **The rule is the count**: a month holding none is
//! [`MonthKind::Adhika`], intercalary, and the name repeats; one holding
//! two is [`MonthKind::Kshaya`], omitted, and the next name is skipped.
//! Measured over 12 368 months of a millennium — 388 adhika, 11 961
//! ordinary, 19 kshaya, none with three — and it reproduces the corpus's
//! marking on all fifty-five recorded days
//! (`03-design/calendar-indian-lunisolar-measured.md`).
//!
//! This is **the mark and not dates**, and the design page's §6 is why: a
//! lunisolar date's day is the tithi at sunrise, which repeats one day in
//! forty-four and is skipped one in twenty-six, so `(year, month, day)`
//! is not a key and a date needs two flags `CalendarDate` does not carry.
//! `panchanga` needs the mark; nothing yet needs the date.
//!
//! Everything here runs on a model rather than an ephemeris. The Surya
//! Siddhanta's own Sun and Moon give both the new moons and the
//! sankrantis, so a calendar can be asked for a month without a provider
//! — which is a decision and not only a convenience: a calendar is a rule
//! *about* the sky rather than a measurement *of* it.

use serde::{Deserialize, Serialize};
use teistro_astro::solve::{Caps, SolveError, next_crossing};
use teistro_core::error::{Error, Status};
use teistro_core::quantity::{JulianDay, Utc};

use crate::solar::{SolarModel, find_sankranti};

/// The Moon a lunisolar calendar needs, beside the Sun a [`SolarModel`]
/// gives.
///
/// A trait of its own rather than a method on [`SolarModel`]: a solar
/// calendar needs no Moon, and five implementations of that trait — three
/// of them test doubles — would have to grow one for nothing.
///
/// An implementation must answer in the **same sky** as the [`SolarModel`]
/// it is used with. Nothing in the types can hold a caller to that, so
/// [`month_at`] takes both together and says so; passing the Surya
/// Siddhanta for one and a modern ephemeris for the other gives a month
/// bounded by one sky and named by another.
pub trait LunarModel: Send + Sync {
    /// The Moon's elongation from the Sun, degrees in `[0, 360)`, which
    /// is nought at a new moon.
    ///
    /// # Errors
    ///
    /// An instant the model cannot answer for; the classical model never
    /// fails.
    fn elongation_deg(&self, jd_ut: f64) -> Result<f64, Error>;
}

impl<T: LunarModel + ?Sized> LunarModel for &T {
    fn elongation_deg(&self, jd_ut: f64) -> Result<f64, Error> {
        (**self).elongation_deg(jd_ut)
    }
}

/// What a lunar month is, beyond its name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MonthKind {
    /// One sankranti: the ordinary month.
    Nija,
    /// None: intercalary, and the name repeats — the next month takes it
    /// too, as the *nija* one.
    Adhika,
    /// Two: the second name is skipped this year.
    Kshaya,
}

impl MonthKind {
    /// The kind a count of sankrantis names.
    ///
    /// More than two cannot happen — a solar month is longer than a lunar
    /// one, so two boundaries is the most that fit — and a count that
    /// large is a model answering nonsense rather than a fourth case, so
    /// it is refused rather than named.
    #[must_use]
    pub const fn of(sankrantis: usize) -> Option<MonthKind> {
        match sankrantis {
            0 => Some(MonthKind::Adhika),
            1 => Some(MonthKind::Nija),
            2 => Some(MonthKind::Kshaya),
            _ => None,
        }
    }

    /// The key the catalogue would spell it with.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            MonthKind::Nija => "NIJA",
            MonthKind::Adhika => "ADHIKA",
            MonthKind::Kshaya => "KSHAYA",
        }
    }
}

/// The lunar month a moment falls in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LunarMonthSpan {
    /// The new moon that opened it.
    pub from: JulianDay<Utc>,
    /// The new moon that closes it.
    pub to: JulianDay<Utc>,
    /// The sign the Sun stood in at [`LunarMonthSpan::from`], which names
    /// the month: Chaitra opens with the Sun in Meena, so the masa is one
    /// past the sign.
    ///
    /// An adhika month needs no rule of its own here. The Sun stands in
    /// the same sign at its new moon as at the following nija month's —
    /// that is what having no sankranti between them means — so both take
    /// the same name from this one field.
    pub sign: u8,
    /// Whether it is ordinary, intercalary or omitted.
    pub kind: MonthKind,
}

/// A synodic month, days: the mean interval between new moons.
pub const SYNODIC_DAYS: f64 = 29.530_588_9;

/// The Moon's mean gain on the Sun, degrees per day: a full circle in a
/// synodic month, and the rate the crossing search is guided by.
pub const MEAN_ELONGATION_RATE_DEG_PER_DAY: f64 = 360.0 / SYNODIC_DAYS;

/// The tolerance a new moon is found to, days: under a tenth of a second,
/// which is far inside the half-hour the text and a modern reckoning
/// differ by anyway.
pub const TOLERANCE_DAYS: f64 = 1e-6;

/// The first new moon at or after an instant.
///
/// # Errors
///
/// The model's own refusal, or a search that does not converge, which
/// cannot happen for a real sky and is `NOT_CONVERGED` naming the
/// instant.
pub fn next_new_moon(moon: &dyn LunarModel, from: JulianDay<Utc>) -> Result<JulianDay<Utc>, Error> {
    let crossing = next_crossing(
        |jd| moon.elongation_deg(jd),
        0.0,
        from.get(),
        MEAN_ELONGATION_RATE_DEG_PER_DAY,
        TOLERANCE_DAYS,
        Caps::DEFAULT,
    )
    .map_err(|error| match error {
        SolveError::Evaluation(inner) => inner,
        other => Error::new(
            Status::NotConverged,
            format!("the new moon after {from} was not found: {other}"),
        ),
    })?;
    JulianDay::try_new(crossing.instant).map_err(Error::from)
}

/// The sign the Sun stands in at an instant, 0 for Mesha.
fn sign_at(sun: &dyn SolarModel, jd: f64) -> Result<u8, Error> {
    let longitude = sun.sidereal_sun_deg(jd)?;
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a normalised longitude over thirty is 0 to 11"
    )]
    let index = (longitude.rem_euclid(360.0) / 30.0) as u8;
    Ok(index.min(11))
}

/// Every sankranti inside a half-open span, as the signs entered.
///
/// The Sun runs forwards, so the next sankranti after an instant is its
/// entry into the sign after the one it stands in; asking for that in
/// turn walks the span without a scan.
fn sankrantis_in(sun: &dyn SolarModel, from: f64, to: f64) -> Result<Vec<u8>, Error> {
    let mut found = Vec::new();
    let mut at = from;
    // A solar month is 30.44 days and a lunar one 29.53, so two entries
    // is the most that fit; a third would be the model answering
    // nonsense, and the loop stops rather than running away on it.
    while found.len() <= 2 {
        let next = (sign_at(sun, at)? + 1) % 12;
        let sankranti = find_sankranti(sun, next, JulianDay::try_new(at).map_err(Error::from)?)?;
        let instant = sankranti.instant.get();
        if instant >= to {
            break;
        }
        found.push(sankranti.sign);
        // Past the entry, so the next search cannot find it again.
        at = instant + TOLERANCE_DAYS;
    }
    Ok(found)
}

/// What kind of month a span is, from the sankrantis inside it.
///
/// Split out of [`month_at`] because a caller that already knows the
/// month's bounds — `panchanga` finds them for the limbs anyway — should
/// not find them twice, and because the rule is then one function whether
/// the sky is the text's or a provider's.
///
/// # Errors
///
/// The model's own refusal, or — impossibly for a real sky — a span
/// holding more than two sankrantis, which is `INTERNAL` rather than a
/// fourth kind.
pub fn kind_of(
    sun: &dyn SolarModel,
    from: JulianDay<Utc>,
    to: JulianDay<Utc>,
) -> Result<MonthKind, Error> {
    let sankrantis = sankrantis_in(sun, from.get(), to.get())?;
    MonthKind::of(sankrantis.len()).ok_or_else(|| {
        Error::new(
            Status::Internal,
            format!(
                "the lunar month from {from} to {to} holds {} sankrantis, and a month cannot hold more than two",
                sankrantis.len()
            ),
        )
    })
}

/// The lunar month a moment falls in, and what kind of month it is.
///
/// The two models must answer in the same sky: the month is bounded by
/// the Moon's new moons and named by the Sun's signs, and mixing a
/// classical Moon with a modern Sun gives a month bounded by one and
/// named by the other. `SuryaSiddhanta` implements both, so the ordinary
/// call passes it twice.
///
/// # Errors
///
/// Either model's own refusal, a search that does not converge, or —
/// impossibly for a real sky — a month holding more than two sankrantis,
/// which is `INTERNAL` rather than a fourth kind.
pub fn month_at(
    sun: &dyn SolarModel,
    moon: &dyn LunarModel,
    at: JulianDay<Utc>,
) -> Result<LunarMonthSpan, Error> {
    // The month is bounded by the new moon at or before `at` and the one
    // after it. Searching backwards for the *first* new moon in a window
    // finds the wrong one whenever two fall in it, so the search runs
    // forwards from `at` and steps back one month from what it finds.
    let after = next_new_moon(moon, at)?;
    let (from, to) = if after.get() - at.get() < TOLERANCE_DAYS {
        // `at` is itself a new moon, so it opens the month rather than
        // closing the one before.
        (after, next_new_moon(moon, forward(after, 1.0)?)?)
    } else {
        (
            next_new_moon(moon, back(after, SYNODIC_DAYS + 1.0)?)?,
            after,
        )
    };
    let kind = kind_of(sun, from, to)?;
    Ok(LunarMonthSpan {
        from,
        to,
        sign: sign_at(sun, from.get())?,
        kind,
    })
}

/// An instant so many days earlier.
fn back(at: JulianDay<Utc>, days: f64) -> Result<JulianDay<Utc>, Error> {
    JulianDay::try_new(at.get() - days).map_err(Error::from)
}

/// An instant so many days later.
fn forward(at: JulianDay<Utc>, days: f64) -> Result<JulianDay<Utc>, Error> {
    JulianDay::try_new(at.get() + days).map_err(Error::from)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::expect_used,
        clippy::panic,
        reason = "a test fails by panicking and names the month at fault"
    )]

    use super::{LunarModel, MonthKind, month_at};
    use teistro_core::quantity::{JulianDay, Utc};
    use teistro_siddhanta::{Parameters, SuryaSiddhanta, Trig};

    fn text() -> SuryaSiddhanta {
        SuryaSiddhanta::new(Parameters::TEXT, Trig::Exact)
    }

    /// The two months the corpus marks adhika are the two the rule finds.
    ///
    /// Delhi in August 1947 and Fairbanks in June 2015 are the only
    /// adhika months the corpus records, and they are the whole of the
    /// evidence it can give: it records the answer and none of the
    /// inputs, so it can falsify a wrong rule and cannot derive the right
    /// one (`calendar-indian-lunisolar-measured.md` §2).
    #[test]
    fn the_two_recorded_adhika_months_are_found() {
        let model = text();
        for (jd, what) in [
            (2_432_412.5, "Delhi, 15 August 1947"),
            (2_457_194.5, "Fairbanks, 21 June 2015"),
        ] {
            let month = month_at(&model, &model, JulianDay::<Utc>::literal(jd))
                .unwrap_or_else(|e| panic!("{what}: {e}"));
            assert_eq!(month.kind, MonthKind::Adhika, "{what}");
        }
    }

    /// An adhika month and the nija month after it share a name.
    ///
    /// The design's §4: the Sun stands in the same sign at both new
    /// moons, so the naming rule needs no case for the intercalary month.
    /// August 1947 is Shravana twice over.
    #[test]
    fn an_adhika_month_and_the_next_share_a_sign() {
        let model = text();
        let adhika = month_at(&model, &model, JulianDay::<Utc>::literal(2_432_412.5))
            .expect("the adhika month");
        let next = month_at(
            &model,
            &model,
            JulianDay::<Utc>::literal(adhika.to.get() + 1.0),
        )
        .expect("the month after it");
        assert_eq!(adhika.kind, MonthKind::Adhika);
        assert_eq!(next.kind, MonthKind::Nija);
        assert_eq!(
            adhika.sign, next.sign,
            "the intercalary month and the true one take the same name"
        );
    }

    /// A run of months is contiguous, and every one is classified.
    ///
    /// The classification is total — the measurement found no month in
    /// 12 368 holding three sankrantis — so a walk over a year must find
    /// a kind for each and no gap between them.
    #[test]
    fn a_year_of_months_is_contiguous_and_classified() {
        let model = text();
        let mut at = JulianDay::<Utc>::literal(2_451_545.0);
        let mut previous_end: Option<f64> = None;
        for step in 0..13 {
            let month =
                month_at(&model, &model, at).unwrap_or_else(|e| panic!("month {step}: {e}"));
            assert!(month.from.get() <= at.get() && at.get() < month.to.get());
            assert!(month.sign < 12);
            if let Some(end) = previous_end {
                assert!(
                    (month.from.get() - end).abs() < 1e-3,
                    "month {step} opens where the last closed"
                );
            }
            previous_end = Some(month.to.get());
            at = JulianDay::literal(month.to.get() + 1.0);
        }
    }

    /// A kshaya month is found where the measurement says one is.
    ///
    /// The corpus records none, so this is the rule held to the
    /// measurement rather than to an authority — which is what the design
    /// page says the state of it is. The month is the one the pass finds
    /// around January 1964.
    #[test]
    fn a_kshaya_month_is_found_where_the_measurement_says() {
        let model = text();
        let month = month_at(&model, &model, JulianDay::<Utc>::literal(2_438_395.0))
            .expect("the month of January 1964");
        assert_eq!(
            month.kind,
            MonthKind::Kshaya,
            "the measurement puts a kshaya month here"
        );
    }

    /// The Moon a model gives is the Moon the month is bounded by.
    #[test]
    fn the_elongation_is_nought_at_a_month_boundary() {
        let model = text();
        let month =
            month_at(&model, &model, JulianDay::<Utc>::literal(2_451_545.0)).expect("a month");
        for edge in [month.from.get(), month.to.get()] {
            let elongation = model.elongation_deg(edge).expect("the text answers");
            let from_zero = elongation.min(360.0 - elongation);
            assert!(
                from_zero < 1e-3,
                "a month boundary is a new moon, not {elongation}°"
            );
        }
    }
}
