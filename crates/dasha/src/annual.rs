//! A year cut into shares: the kernel of the annual dashas
//! (`03-design/annual-dashas.md`).
//!
//! The Mudda, the Varsha Yogini and the Patyayini divide **one year**, the
//! year a solar return opens, among a ring of lords, each taking a share.
//! The year may open part-way through the first lord's share, and then the
//! part already run closes the year, so that lord appears twice. Nothing
//! else in the build has that shape: a natal dasha ends its cycle after its
//! last lord, and its periods are years of a fixed length where these are
//! shares of an interval whose clock is itself a choice.
//!
//! **Every boundary is a share first and an instant last.** A period's
//! place in the year is computed from its path as a fraction of the year,
//! exactly as the natal tree computes a share of its parent, and only then
//! does the [`Clock`] turn the fraction into a Julian day. Children
//! partition their parent in shares whatever the clock: the first begins
//! on its parent's start and the last ends on its parent's end.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::settings::BirthPeriod;

use crate::row::DashaName;
use crate::tree::{Path, Period, Timeline};

/// One lord of a year's ring, and its weight among the ring's.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_dasha::Share;
///
/// // A Patyayini share: Leo's lagna, ruled by the Sun, weighed by its arc.
/// let lagna = Share { lord: Graha::Sun, sign: Some(Rashi::Leo), weight: 12.5 };
/// assert_eq!(lagna.sign, Some(Rashi::Leo));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Share {
    /// Its lord: the graha, or the lord of the sign when the share is a
    /// sign's.
    pub lord: Graha,
    /// The sign the share belongs to when it is not a graha's: the
    /// Patyayini's lagna, named by the sign it rises in.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub sign: Option<Rashi>,
    /// Its weight: a lord's share of the year is its weight over the
    /// ring's. Years for a nakshatra system, arc for the Patyayini.
    pub weight: f64,
}

/// What a year is divided among: the ring, where it starts, and how much
/// of the first lord's share is still to run when the year opens.
///
/// ```
/// use teistro_core::catalogue::{DashaSystem, Graha};
/// use teistro_dasha::row::DashaName;
/// use teistro_dasha::{Share, YearRing};
///
/// let lords = [Graha::Sun, Graha::Moon, Graha::Mars, Graha::Mercury];
/// let ring = YearRing {
///     system: DashaName::Catalogued(DashaSystem::Mudda),
///     ring: lords
///         .iter()
///         .zip(1..)
///         .map(|(&lord, weight)| Share { lord, sign: None, weight: f64::from(weight) })
///         .collect(),
///     first: 0,
///     remaining: None,
/// };
///
/// ring.validate()?;
/// assert_eq!((ring.total(), ring.mahadashas()), (10.0, 4));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct YearRing {
    /// Which system.
    pub system: DashaName,
    /// The lords in order, round which the year runs.
    pub ring: Vec<Share>,
    /// The place in the ring the year opens with.
    pub first: usize,
    /// How much of the first lord's share was still to run when the year
    /// opened, 0 to 1; the rest closes the year. Nothing when the first
    /// lord runs its whole share from the start and the year ends with the
    /// lord before it, as the Patyayini's does.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub remaining: Option<f64>,
}

impl YearRing {
    /// The checks a ring must pass, each refused by the field that is
    /// wrong.
    ///
    /// ```
    /// # use teistro_core::catalogue::{DashaSystem, Graha};
    /// # use teistro_dasha::row::DashaName;
    /// # use teistro_dasha::{Share, YearRing};
    /// #
    /// # let lords = [Graha::Sun, Graha::Moon, Graha::Mars, Graha::Mercury];
    /// # let ring = YearRing {
    /// #     system: DashaName::Catalogued(DashaSystem::Mudda),
    /// #     ring: lords
    /// #         .iter()
    /// #         .zip(1..)
    /// #         .map(|(&lord, weight)| Share { lord, sign: None, weight: f64::from(weight) })
    /// #         .collect(),
    /// #     first: 0,
    /// #     remaining: None,
    /// # };
    /// ring.validate()?;
    /// let mut late = ring.clone();
    /// late.first = 4;
    /// assert_eq!(late.validate().unwrap_err().field(), Some("first"));
    /// let mut over = ring;
    /// over.remaining = Some(1.5);
    /// assert_eq!(over.validate().unwrap_err().field(), Some("remaining"));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// An empty ring, or one longer than a path can index (`ring`); a
    /// weight that is negative or not a number (`ring[i].weight`); weights
    /// that sum to nothing (`ring`); a first place outside the ring
    /// (`first`); a remainder outside 0 to 1 (`remaining`).
    pub fn validate(&self) -> Result<(), Error> {
        if self.ring.is_empty() || self.ring.len() > usize::from(u8::MAX) {
            return Err(Error::invalid_arg(format!(
                "a year's ring holds 1 to {} lords, not {}",
                u8::MAX,
                self.ring.len()
            ))
            .with_field("ring"));
        }
        for (index, share) in self.ring.iter().enumerate() {
            if !share.weight.is_finite() || share.weight < 0.0 {
                return Err(Error::invalid_arg(format!(
                    "a share's weight is a number of zero or more, not {}",
                    share.weight
                ))
                .with_field(format!("ring[{index}].weight")));
            }
        }
        let total = self.total();
        if total <= 0.0 || !total.is_finite() {
            return Err(
                Error::invalid_arg("a year's ring has no weight to divide it by")
                    .with_field("ring"),
            );
        }
        if self.first >= self.ring.len() {
            return Err(Error::invalid_arg(format!(
                "the year opens at place {} of a ring of {}",
                self.first,
                self.ring.len()
            ))
            .with_field("first"));
        }
        if let Some(remaining) = self.remaining
            && !(0.0..=1.0).contains(&remaining)
        {
            return Err(Error::invalid_arg(format!(
                "what remains of the first share is 0 to 1, not {remaining}"
            ))
            .with_field("remaining"));
        }
        Ok(())
    }

    /// The ring's weights summed: what the whole year stands for.
    ///
    /// ```
    /// # use teistro_core::catalogue::{DashaSystem, Graha};
    /// # use teistro_dasha::row::DashaName;
    /// # use teistro_dasha::{Share, YearRing};
    /// #
    /// # let lords = [Graha::Sun, Graha::Moon, Graha::Mars, Graha::Mercury];
    /// # let ring = YearRing {
    /// #     system: DashaName::Catalogued(DashaSystem::Mudda),
    /// #     ring: lords
    /// #         .iter()
    /// #         .zip(1..)
    /// #         .map(|(&lord, weight)| Share { lord, sign: None, weight: f64::from(weight) })
    /// #         .collect(),
    /// #     first: 0,
    /// #     remaining: None,
    /// # };
    /// assert_eq!(ring.total(), 1.0 + 2.0 + 3.0 + 4.0);
    /// ```
    #[must_use]
    pub fn total(&self) -> f64 {
        self.ring.iter().map(|share| share.weight).sum()
    }

    /// How many mahadashas the year holds: one for each lord, and one more
    /// when the first lord's share is split across the year's two ends.
    ///
    /// ```
    /// # use teistro_core::catalogue::{DashaSystem, Graha};
    /// # use teistro_dasha::row::DashaName;
    /// # use teistro_dasha::{Share, YearRing};
    /// #
    /// # let lords = [Graha::Sun, Graha::Moon, Graha::Mars, Graha::Mercury];
    /// # let ring = YearRing {
    /// #     system: DashaName::Catalogued(DashaSystem::Mudda),
    /// #     ring: lords
    /// #         .iter()
    /// #         .zip(1..)
    /// #         .map(|(&lord, weight)| Share { lord, sign: None, weight: f64::from(weight) })
    /// #         .collect(),
    /// #     first: 0,
    /// #     remaining: None,
    /// # };
    /// assert_eq!(ring.mahadashas(), 4);
    /// // Opening part-way through the Sun's share splits it over the two ends.
    /// let split = YearRing { remaining: Some(0.5), ..ring };
    /// assert_eq!(split.mahadashas(), 5);
    /// ```
    #[must_use]
    pub fn mahadashas(&self) -> usize {
        self.ring.len() + usize::from(self.remaining.is_some())
    }
}

/// How a fraction of the year becomes an instant.
///
/// A monotone map held as **knots**: Julian days (UTC) at equal steps of
/// the year, the first where the year opens and the last where it closes.
/// Two knots spread the year evenly; 361 put each of the Sun's degrees at
/// the instant it crosses it. A fraction between two knots is placed
/// linearly between them, and one outside the year (a sub-period sized
/// against a share that began before the year did) is extrapolated along
/// the end segment.
///
/// ```
/// use teistro_core::interval::Interval;
/// use teistro_dasha::Clock;
///
/// // A year whose first half runs faster than its second.
/// let clock = Clock::new(vec![2_460_000.5, 2_460_170.5, 2_460_365.5])?;
/// assert_eq!(clock.at(0.5), 2_460_170.5);
/// assert_eq!(clock.at(0.25), 2_460_085.5);
/// assert_eq!(clock.year(), Interval::literal(2_460_000.5, 2_460_365.5));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Clock {
    knots: Vec<f64>,
}

impl Clock {
    /// A clock through its knots.
    ///
    /// ```
    /// use teistro_core::interval::Interval;
    /// use teistro_dasha::Clock;
    ///
    /// assert_eq!(Clock::new(vec![2_460_000.5]).unwrap_err().field(), Some("knots"));
    /// assert!(Clock::new(vec![2_460_365.5, 2_460_000.5]).is_err(), "it runs forwards");
    /// assert_eq!(Clock::new(vec![2_460_000.5, 2_460_365.5])?.knots().len(), 2);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Fewer than two knots, one that is not a number, or two that do not
    /// increase, named `knots`.
    pub fn new(knots: Vec<f64>) -> Result<Clock, Error> {
        let refused = |why: String| Err(Error::invalid_arg(why).with_field("knots"));
        if knots.len() < 2 {
            return refused(format!(
                "a year's clock needs its two ends at least, not {} knots",
                knots.len()
            ));
        }
        if knots.iter().any(|knot| !knot.is_finite()) {
            return refused(String::from(
                "a year's clock has a knot that is not a number",
            ));
        }
        if let Some(step) = knots
            .windows(2)
            .position(|pair| matches!(pair, [from, to] if to <= from))
        {
            return refused(format!(
                "a year's clock runs forwards, and knot {} does not follow knot {step}",
                step + 1
            ));
        }
        Ok(Clock { knots })
    }

    /// The clock that spreads the year evenly over an interval.
    ///
    /// ```
    /// use teistro_core::interval::Interval;
    /// use teistro_dasha::Clock;
    ///
    /// let clock = Clock::even(Interval::literal(2_460_000.5, 2_460_360.5))?;
    /// assert_eq!(clock.at(0.5), 2_460_180.5);
    /// assert!(Clock::even(Interval::literal(2_460_000.5, 2_460_000.5)).is_err());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// An empty year, named `knots`.
    pub fn even(year: Interval) -> Result<Clock, Error> {
        Clock::new(vec![year.from.get(), year.to.get()])
    }

    /// The year it runs over.
    ///
    /// ```
    /// use teistro_core::interval::Interval;
    /// use teistro_dasha::Clock;
    ///
    /// let year = Interval::literal(2_460_000.5, 2_460_365.75);
    /// assert_eq!(Clock::even(year)?.year(), year);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn year(&self) -> Interval {
        let first = self.knots.first().copied().unwrap_or_default();
        let last = self.knots.last().copied().unwrap_or(first);
        Interval::literal(first, last.max(first))
    }

    /// Its knots.
    ///
    /// ```
    /// use teistro_core::interval::Interval;
    /// use teistro_dasha::Clock;
    ///
    /// let clock = Clock::new(vec![2_460_000.5, 2_460_170.5, 2_460_365.5])?;
    /// assert_eq!(clock.knots(), [2_460_000.5, 2_460_170.5, 2_460_365.5]);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn knots(&self) -> &[f64] {
        &self.knots
    }

    fn segments(&self) -> usize {
        self.knots.len().saturating_sub(1).max(1)
    }

    /// The instant a fraction of the year falls at: exactly a knot at a
    /// knot, linear between, extrapolated past either end.
    ///
    /// ```
    /// use teistro_core::interval::Interval;
    /// use teistro_dasha::Clock;
    ///
    /// let clock = Clock::even(Interval::literal(2_460_000.5, 2_460_360.5))?;
    /// assert_eq!((clock.at(0.0), clock.at(1.0)), (2_460_000.5, 2_460_360.5));
    /// // A tenth before the year is extrapolated along it.
    /// assert!((clock.at(-0.1) - 2_459_964.5).abs() < 1e-6);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    #[expect(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::float_cmp,
        reason = "a clock holds a few hundred knots and the segment is clamped into them; \
                  a fraction exactly on a knot answers the knot itself, not a sum near it"
    )]
    pub fn at(&self, fraction: f64) -> f64 {
        let segments = self.segments();
        let scaled = fraction * segments as f64;
        let index = (scaled.floor().max(0.0) as usize).min(segments - 1);
        let within = scaled - index as f64;
        let (Some(&from), Some(&to)) = (self.knots.get(index), self.knots.get(index + 1)) else {
            return self.knots.first().copied().unwrap_or_default();
        };
        if within == 0.0 {
            from
        } else if within == 1.0 {
            to
        } else {
            from + within * (to - from)
        }
    }

    /// The fraction of the year an instant falls at: the inverse of
    /// [`Clock::at`].
    ///
    /// ```
    /// use teistro_core::interval::Interval;
    /// use teistro_dasha::Clock;
    ///
    /// let clock = Clock::new(vec![2_460_000.5, 2_460_170.5, 2_460_365.5])?;
    /// assert_eq!(clock.fraction_of(2_460_170.5), 0.5);
    /// assert!((clock.fraction_of(clock.at(0.8)) - 0.8).abs() < 1e-12);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    #[expect(
        clippy::cast_precision_loss,
        reason = "a clock holds a few hundred knots"
    )]
    pub fn fraction_of(&self, instant: f64) -> f64 {
        let segments = self.segments();
        let index = self
            .knots
            .partition_point(|knot| *knot <= instant)
            .clamp(1, segments)
            - 1;
        let (Some(&from), Some(&to)) = (self.knots.get(index), self.knots.get(index + 1)) else {
            return 0.0;
        };
        (index as f64 + (instant - from) / (to - from)) / segments as f64
    }
}

/// Where a period sits in the year, in fractions of it: the span it runs,
/// the span its sub-periods divide, and its lord's place in the ring.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Place {
    span: (f64, f64),
    whole: (f64, f64),
    seat: usize,
}

/// One year's dasha: a ring over a clock.
///
/// ```
/// use teistro_core::catalogue::{DashaSystem, Graha};
/// use teistro_dasha::row::DashaName;
/// use teistro_dasha::{Share, YearRing};
///
/// let lords = [Graha::Sun, Graha::Moon, Graha::Mars, Graha::Mercury];
/// let ring = YearRing {
///     system: DashaName::Catalogued(DashaSystem::Mudda),
///     ring: lords
///         .iter()
///         .zip(1..)
///         .map(|(&lord, weight)| Share { lord, sign: None, weight: f64::from(weight) })
///         .collect(),
///     first: 0,
///     remaining: None,
/// };
/// use teistro_core::interval::Interval;
/// use teistro_core::settings::BirthPeriod;
/// use teistro_dasha::{Clock, Timeline, YearDasha};
///
/// // The Sun's tenth of a 360-day year, then the Moon's fifth.
/// let clock = Clock::even(Interval::literal(2_460_000.5, 2_460_360.5))?;
/// let dasha = YearDasha::new(ring, clock, BirthPeriod::Compressed)?;
/// let days: Vec<_> = dasha.mahadashas().map(|p| (p.lord, p.interval.days())).collect();
/// assert_eq!(days[..2], [(Graha::Sun, 36.0), (Graha::Moon, 72.0)]);
/// assert_eq!(dasha.mahadashas().last().map(|p| p.interval.to), Some(dasha.year().to));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct YearDasha {
    ring: YearRing,
    clock: Clock,
    birth_period: BirthPeriod,
    /// Where each mahadasha begins as a fraction of the year, and where
    /// the last ends: `mahadashas + 1` fractions, the first 0 and the last
    /// exactly 1.
    bounds: Vec<f64>,
}

impl YearDasha {
    /// A year's dasha: `ring` over `clock`, with the first lord's two
    /// pieces divided as `birth_period` divides a natal birth period.
    ///
    /// ```
    /// # use teistro_core::catalogue::{DashaSystem, Graha};
    /// # use teistro_dasha::row::DashaName;
    /// # use teistro_dasha::{Share, YearRing};
    /// #
    /// # let lords = [Graha::Sun, Graha::Moon, Graha::Mars, Graha::Mercury];
    /// # let ring = YearRing {
    /// #     system: DashaName::Catalogued(DashaSystem::Mudda),
    /// #     ring: lords
    /// #         .iter()
    /// #         .zip(1..)
    /// #         .map(|(&lord, weight)| Share { lord, sign: None, weight: f64::from(weight) })
    /// #         .collect(),
    /// #     first: 0,
    /// #     remaining: None,
    /// # };
    /// use teistro_core::interval::Interval;
    /// use teistro_core::settings::BirthPeriod;
    /// use teistro_dasha::{Clock, YearDasha};
    ///
    /// let clock = Clock::even(Interval::literal(2_460_000.5, 2_460_360.5))?;
    /// let empty = YearRing { ring: Vec::new(), ..ring };
    /// let refused = YearDasha::new(empty, clock, BirthPeriod::Compressed).unwrap_err();
    /// assert_eq!(refused.field(), Some("ring"));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A ring its checks refuse ([`YearRing::validate`]).
    pub fn new(
        ring: YearRing,
        clock: Clock,
        birth_period: BirthPeriod,
    ) -> Result<YearDasha, Error> {
        ring.validate()?;
        let total = ring.total();
        let lords = ring.ring.len();
        let weight = |place: usize| {
            ring.ring
                .get(place % lords)
                .map_or(0.0, |share| share.weight)
        };
        let mut bounds = Vec::with_capacity(ring.mahadashas() + 1);
        bounds.push(0.0);
        let mut run = 0.0;
        let steps = match ring.remaining {
            Some(remaining) => {
                run += remaining * weight(ring.first);
                bounds.push(run / total);
                1..lords
            }
            None => 0..lords - 1,
        };
        for step in steps {
            run += weight(ring.first + step);
            bounds.push(run / total);
        }
        // The last lord closes the year: exactly, and never past it.
        bounds.push(1.0);
        for bound in &mut bounds {
            *bound = bound.min(1.0);
        }
        Ok(YearDasha {
            ring,
            clock,
            birth_period,
            bounds,
        })
    }

    /// The ring it runs.
    ///
    /// ```
    /// # use teistro_core::catalogue::{DashaSystem, Graha};
    /// # use teistro_dasha::row::DashaName;
    /// # use teistro_dasha::{Share, YearRing};
    /// #
    /// # let lords = [Graha::Sun, Graha::Moon, Graha::Mars, Graha::Mercury];
    /// # let ring = YearRing {
    /// #     system: DashaName::Catalogued(DashaSystem::Mudda),
    /// #     ring: lords
    /// #         .iter()
    /// #         .zip(1..)
    /// #         .map(|(&lord, weight)| Share { lord, sign: None, weight: f64::from(weight) })
    /// #         .collect(),
    /// #     first: 0,
    /// #     remaining: None,
    /// # };
    /// # use teistro_core::interval::Interval;
    /// # use teistro_core::settings::BirthPeriod;
    /// # use teistro_dasha::{Clock, Timeline, YearDasha};
    /// #
    /// # let clock = Clock::even(Interval::literal(2_460_000.5, 2_460_360.5))?;
    /// # let dasha = YearDasha::new(ring.clone(), clock.clone(), BirthPeriod::Compressed)?;
    ///
    /// assert_eq!(dasha.ring(), &ring);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub const fn ring(&self) -> &YearRing {
        &self.ring
    }

    /// The clock it runs on.
    ///
    /// ```
    /// # use teistro_core::catalogue::{DashaSystem, Graha};
    /// # use teistro_dasha::row::DashaName;
    /// # use teistro_dasha::{Share, YearRing};
    /// #
    /// # let lords = [Graha::Sun, Graha::Moon, Graha::Mars, Graha::Mercury];
    /// # let ring = YearRing {
    /// #     system: DashaName::Catalogued(DashaSystem::Mudda),
    /// #     ring: lords
    /// #         .iter()
    /// #         .zip(1..)
    /// #         .map(|(&lord, weight)| Share { lord, sign: None, weight: f64::from(weight) })
    /// #         .collect(),
    /// #     first: 0,
    /// #     remaining: None,
    /// # };
    /// # use teistro_core::interval::Interval;
    /// # use teistro_core::settings::BirthPeriod;
    /// # use teistro_dasha::{Clock, Timeline, YearDasha};
    /// #
    /// # let clock = Clock::even(Interval::literal(2_460_000.5, 2_460_360.5))?;
    /// # let dasha = YearDasha::new(ring.clone(), clock.clone(), BirthPeriod::Compressed)?;
    ///
    /// assert_eq!(dasha.clock(), &clock);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub const fn clock(&self) -> &Clock {
        &self.clock
    }

    /// How the first lord's two pieces are divided.
    ///
    /// ```
    /// # use teistro_core::catalogue::{DashaSystem, Graha};
    /// # use teistro_dasha::row::DashaName;
    /// # use teistro_dasha::{Share, YearRing};
    /// #
    /// # let lords = [Graha::Sun, Graha::Moon, Graha::Mars, Graha::Mercury];
    /// # let ring = YearRing {
    /// #     system: DashaName::Catalogued(DashaSystem::Mudda),
    /// #     ring: lords
    /// #         .iter()
    /// #         .zip(1..)
    /// #         .map(|(&lord, weight)| Share { lord, sign: None, weight: f64::from(weight) })
    /// #         .collect(),
    /// #     first: 0,
    /// #     remaining: None,
    /// # };
    /// # use teistro_core::interval::Interval;
    /// # use teistro_core::settings::BirthPeriod;
    /// # use teistro_dasha::{Clock, Timeline, YearDasha};
    /// #
    /// # let clock = Clock::even(Interval::literal(2_460_000.5, 2_460_360.5))?;
    /// # let dasha = YearDasha::new(ring.clone(), clock.clone(), BirthPeriod::Compressed)?;
    ///
    /// assert_eq!(dasha.birth_period(), BirthPeriod::Compressed);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub const fn birth_period(&self) -> BirthPeriod {
        self.birth_period
    }

    /// The year it divides.
    ///
    /// ```
    /// # use teistro_core::catalogue::{DashaSystem, Graha};
    /// # use teistro_dasha::row::DashaName;
    /// # use teistro_dasha::{Share, YearRing};
    /// #
    /// # let lords = [Graha::Sun, Graha::Moon, Graha::Mars, Graha::Mercury];
    /// # let ring = YearRing {
    /// #     system: DashaName::Catalogued(DashaSystem::Mudda),
    /// #     ring: lords
    /// #         .iter()
    /// #         .zip(1..)
    /// #         .map(|(&lord, weight)| Share { lord, sign: None, weight: f64::from(weight) })
    /// #         .collect(),
    /// #     first: 0,
    /// #     remaining: None,
    /// # };
    /// # use teistro_core::interval::Interval;
    /// # use teistro_core::settings::BirthPeriod;
    /// # use teistro_dasha::{Clock, Timeline, YearDasha};
    /// #
    /// # let clock = Clock::even(Interval::literal(2_460_000.5, 2_460_360.5))?;
    /// # let dasha = YearDasha::new(ring.clone(), clock.clone(), BirthPeriod::Compressed)?;
    ///
    /// assert_eq!(dasha.year(), Interval::literal(2_460_000.5, 2_460_360.5));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn year(&self) -> Interval {
        self.clock.year()
    }

    fn lords(&self) -> usize {
        self.ring.ring.len()
    }

    fn share(&self, seat: usize) -> Option<&Share> {
        self.ring.ring.get(seat % self.lords().max(1))
    }

    /// The first lord's whole share, as a fraction of the year.
    fn first_share(&self) -> f64 {
        self.share(self.ring.first)
            .map_or(0.0, |share| share.weight / self.ring.total())
    }

    fn mahadasha_place(&self, index: usize) -> Option<Place> {
        let span = (*self.bounds.get(index)?, *self.bounds.get(index + 1)?);
        let seat = (self.ring.first + index) % self.lords();
        let split = self.ring.remaining.is_some() && self.birth_period == BirthPeriod::Elapsed;
        let whole = if split && index == 0 {
            (span.1 - self.first_share(), span.1)
        } else if split && index == self.lords() {
            (span.0, span.0 + self.first_share())
        } else {
            span
        };
        Some(Place { span, whole, seat })
    }

    /// The child at `index` of a period placed at `parent`: the ring from
    /// the parent's own lord, each its share of the parent's whole, cut to
    /// the parent's span. A child cut away entirely is nothing; a share of
    /// no weight inside its parent is kept, empty, in its place.
    #[expect(
        clippy::float_cmp,
        reason = "a child cut to nothing is exactly empty, the ends set equal by the cut"
    )]
    fn child_place(&self, parent: Place, index: usize) -> Option<Place> {
        let lords = self.lords();
        if index >= lords {
            return None;
        }
        let total = self.ring.total();
        let run = |count: usize| -> f64 {
            (0..count)
                .filter_map(|step| self.share(parent.seat + step))
                .map(|share| share.weight)
                .sum()
        };
        let (from, to) = parent.whole;
        let boundary = |count: usize| -> f64 {
            match count {
                0 => from,
                c if c == lords => to,
                c => from + (to - from) * run(c) / total,
            }
        };
        let whole = (boundary(index), boundary(index + 1));
        let span = (whole.0.max(parent.span.0), whole.1.min(parent.span.1));
        if span.1 < span.0 || (span.1 == span.0 && whole.1 > whole.0) {
            return None;
        }
        Some(Place {
            span,
            whole,
            seat: (parent.seat + index) % lords,
        })
    }

    fn place(&self, path: &Path) -> Option<Place> {
        let (&root, rest) = path.indices().split_first()?;
        let mut place = self.mahadasha_place(usize::from(root))?;
        for &index in rest {
            place = self.child_place(place, usize::from(index))?;
        }
        Some(place)
    }

    fn period(&self, path: Path, place: Place) -> Option<Period> {
        let share = self.share(place.seat)?;
        let at = |(from, to): (f64, f64)| {
            let (from, to) = (self.clock.at(from), self.clock.at(to));
            Interval::literal(from, to.max(from))
        };
        Some(Period::of_share(
            share.lord,
            share.sign,
            path,
            at(place.span),
            at(place.whole),
            place.seat,
        ))
    }
}

impl Timeline for YearDasha {
    fn breadth(&self) -> usize {
        self.lords()
    }

    /// The mahadasha at `index`; a year has one cycle, so nothing past it.
    fn mahadasha(&self, cycle: u32, index: usize) -> Option<Period> {
        if cycle > 0 {
            return None;
        }
        let place = self.mahadasha_place(index)?;
        self.period(Path::root(0, u8::try_from(index).ok()?), place)
    }

    /// Each bound is read through the clock as a period's own ends are, so
    /// an instant on a boundary falls in the period that begins there, and
    /// one outside the year in none.
    fn mahadasha_at(&self, instant: f64) -> Option<Period> {
        let begun = self
            .bounds
            .partition_point(|bound| self.clock.at(*bound) <= instant);
        if begun >= self.bounds.len() {
            return None;
        }
        self.mahadasha(0, begun.checked_sub(1)?)
    }

    fn child(&self, parent: &Period, index: usize) -> Option<Period> {
        let place = self.child_place(self.place(&parent.path)?, index)?;
        self.period(parent.path.child(u8::try_from(index).ok()?)?, place)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::float_cmp,
        reason = "tests fail by panicking, index their own fixtures and compare exact values"
    )]

    use teistro_core::catalogue::DashaSystem;
    use teistro_core::quantity::{Depth, JulianDay};

    use super::*;
    use crate::row::VIMSHOTTARI;

    const OPENS: f64 = 2_445_932.5;

    fn vimshottari_ring(first: usize, remaining: Option<f64>) -> YearRing {
        YearRing {
            system: DashaName::Catalogued(DashaSystem::Mudda),
            ring: VIMSHOTTARI
                .lords
                .iter()
                .map(|lord| Share {
                    lord: lord.graha,
                    sign: None,
                    weight: f64::from(lord.years),
                })
                .collect(),
            first,
            remaining,
        }
    }

    fn days(year_days: f64) -> Clock {
        Clock::even(Interval::literal(OPENS, OPENS + year_days)).unwrap()
    }

    fn lengths(dasha: &YearDasha) -> Vec<(Graha, f64)> {
        dasha
            .mahadashas()
            .map(|period| (period.lord, period.interval.days()))
            .collect()
    }

    #[test]
    fn the_year_opens_on_the_balance_and_closes_on_what_was_run() {
        // Charak's forty-first year: Rahu, with 9°32′ of 13°20′ to run.
        let remaining = (9.0 * 60.0 + 32.0) / 800.0;
        let dasha = YearDasha::new(
            vimshottari_ring(5, Some(remaining)),
            days(360.0),
            BirthPeriod::Compressed,
        )
        .unwrap();
        let rows = lengths(&dasha);
        assert_eq!(rows.len(), 10);
        assert_eq!(rows[0].0, Graha::Rahu);
        assert!((rows[0].1 - 38.61).abs() < 0.005, "{}", rows[0].1);
        assert_eq!(
            rows.iter()
                .skip(1)
                .take(8)
                .map(|row| row.0)
                .collect::<Vec<_>>(),
            [
                Graha::Jupiter,
                Graha::Saturn,
                Graha::Mercury,
                Graha::Ketu,
                Graha::Venus,
                Graha::Sun,
                Graha::Moon,
                Graha::Mars
            ]
        );
        assert_eq!(rows[9].0, Graha::Rahu);
        assert!((rows[9].1 - 15.39).abs() < 0.005, "{}", rows[9].1);
        // The two ends are the year's, exactly.
        let year = dasha.year();
        assert_eq!(dasha.mahadasha(0, 0).unwrap().interval.from, year.from);
        assert_eq!(dasha.mahadasha(0, 9).unwrap().interval.to, year.to);
        assert!(dasha.mahadasha(0, 10).is_none());
        assert!(dasha.mahadasha(1, 0).is_none());
    }

    #[test]
    fn without_a_balance_the_ring_runs_once_from_its_first() {
        let dasha = YearDasha::new(
            vimshottari_ring(0, None),
            days(360.0),
            BirthPeriod::Compressed,
        )
        .unwrap();
        let rows = lengths(&dasha);
        assert_eq!(rows.len(), 9);
        assert_eq!(rows[0], (Graha::Ketu, 21.0));
        assert_eq!(rows[8].0, Graha::Mercury);
        assert!((rows.iter().map(|row| row.1).sum::<f64>() - 360.0).abs() < 1e-9);
    }

    #[test]
    fn a_whole_or_spent_first_share_leaves_an_empty_piece_in_its_place() {
        for (remaining, empty) in [(1.0, 9), (0.0, 0)] {
            let dasha = YearDasha::new(
                vimshottari_ring(2, Some(remaining)),
                days(360.0),
                BirthPeriod::Compressed,
            )
            .unwrap();
            assert_eq!(dasha.mahadashas().count(), 10);
            assert!(dasha.mahadasha(0, empty).unwrap().interval.is_empty());
            // An empty piece runs at no instant.
            let chain = dasha.at(JulianDay::literal(OPENS), Depth::try_new(1).unwrap());
            assert!(!chain.iter().next().unwrap().interval.is_empty());
        }
    }

    #[test]
    fn sub_periods_start_with_their_parent_and_partition_it() {
        let dasha = YearDasha::new(
            vimshottari_ring(5, Some(0.4)),
            days(360.0),
            BirthPeriod::Compressed,
        )
        .unwrap();
        for maha in dasha.mahadashas() {
            let children: Vec<Period> = dasha.children(&maha).collect();
            assert_eq!(children.len(), 9);
            assert_eq!(children[0].lord, maha.lord);
            assert_eq!(children[0].interval.from, maha.interval.from);
            assert_eq!(children[8].interval.to, maha.interval.to);
            for pair in children.windows(2) {
                assert_eq!(pair[0].interval.to, pair[1].interval.from);
            }
        }
    }

    #[test]
    fn the_first_lords_pieces_are_divided_as_a_birth_period_is() {
        let remaining = 0.25;
        let year = |birth_period| {
            YearDasha::new(
                vimshottari_ring(5, Some(remaining)),
                days(360.0),
                birth_period,
            )
            .unwrap()
        };
        // Compressed: each piece among all nine.
        let compressed = year(BirthPeriod::Compressed);
        let head = compressed.mahadasha(0, 0).unwrap();
        assert_eq!(compressed.children(&head).count(), 9);
        // Elapsed: the head is the end of Rahu's share and the tail its
        // start, so the head keeps the last sub-lords and the tail the
        // first, one of them cut in two.
        let elapsed = year(BirthPeriod::Elapsed);
        let head = elapsed.mahadasha(0, 0).unwrap();
        let tail = elapsed.mahadasha(0, 9).unwrap();
        assert!((head.whole().days() - 54.0).abs() < 1e-9);
        let head_lords: Vec<Graha> = elapsed.children(&head).map(|child| child.lord).collect();
        let tail_lords: Vec<Graha> = elapsed.children(&tail).map(|child| child.lord).collect();
        assert_eq!(head_lords.last(), Some(&Graha::Mars));
        assert_eq!(tail_lords.first(), Some(&Graha::Rahu));
        assert_eq!(head_lords.len() + tail_lords.len(), 10);
        assert_eq!(
            elapsed.children(&head).next().unwrap().interval.from,
            head.interval.from
        );
        assert_eq!(
            elapsed.children(&tail).last().unwrap().interval.to,
            tail.interval.to
        );
    }

    #[test]
    fn the_chain_at_an_instant_is_found_through_any_clock() {
        // A clock that runs twice as fast in the second half of the year.
        let clock = Clock::new(vec![OPENS, OPENS + 240.0, OPENS + 360.0]).unwrap();
        let dasha =
            YearDasha::new(vimshottari_ring(0, None), clock, BirthPeriod::Compressed).unwrap();
        for maha in dasha.mahadashas() {
            let mid = JulianDay::literal(maha.interval.from.get() + maha.interval.days() / 2.0);
            let chain = dasha.at(mid, Depth::try_new(3).unwrap());
            assert_eq!(chain.len(), 3);
            assert_eq!(chain.iter().next().unwrap().path, maha.path);
        }
        let depth = Depth::try_new(1).unwrap();
        assert!(dasha.at(JulianDay::literal(OPENS - 1.0), depth).is_empty());
        assert!(
            dasha
                .at(JulianDay::literal(OPENS + 360.0), depth)
                .is_empty()
        );
    }

    #[test]
    fn a_clock_places_its_knots_exactly_and_inverts() {
        let clock = Clock::new(vec![10.0, 12.0, 20.0]).unwrap();
        assert_eq!(clock.at(0.0), 10.0);
        assert_eq!(clock.at(0.5), 12.0);
        assert_eq!(clock.at(1.0), 20.0);
        assert_eq!(clock.at(0.75), 16.0);
        assert_eq!(clock.at(-0.5), 8.0, "extrapolated along the first segment");
        assert_eq!(clock.at(1.25), 24.0, "and along the last");
        for fraction in [0.0, 0.1, 0.5, 0.9, 1.0] {
            assert!((clock.fraction_of(clock.at(fraction)) - fraction).abs() < 1e-12);
        }
        assert_eq!(Clock::new(vec![1.0]).unwrap_err().field(), Some("knots"));
        assert_eq!(
            Clock::new(vec![1.0, 1.0]).unwrap_err().field(),
            Some("knots")
        );
        assert_eq!(
            Clock::new(vec![1.0, f64::NAN]).unwrap_err().field(),
            Some("knots")
        );
    }

    #[test]
    fn a_ring_is_refused_by_the_field_that_is_wrong() {
        let refused = |ring: YearRing| ring.validate().unwrap_err().field().map(String::from);
        let mut empty = vimshottari_ring(0, None);
        empty.ring.clear();
        assert_eq!(refused(empty).as_deref(), Some("ring"));
        let mut negative = vimshottari_ring(0, None);
        negative.ring[3].weight = -1.0;
        assert_eq!(refused(negative).as_deref(), Some("ring[3].weight"));
        let mut weightless = vimshottari_ring(0, None);
        for share in &mut weightless.ring {
            share.weight = 0.0;
        }
        assert_eq!(refused(weightless).as_deref(), Some("ring"));
        assert_eq!(refused(vimshottari_ring(9, None)).as_deref(), Some("first"));
        assert_eq!(
            refused(vimshottari_ring(0, Some(1.5))).as_deref(),
            Some("remaining")
        );
    }

    #[test]
    fn a_share_of_no_weight_keeps_its_place_and_runs_no_time() {
        let mut ring = vimshottari_ring(0, None);
        ring.ring[4].weight = 0.0;
        let dasha = YearDasha::new(ring, days(360.0), BirthPeriod::Compressed).unwrap();
        let maha = dasha.mahadasha(0, 4).unwrap();
        assert!(maha.interval.is_empty());
        assert_eq!(maha.lord, Graha::Mars);
        let first = dasha.mahadasha(0, 0).unwrap();
        assert!(dasha.children(&first).nth(4).unwrap().interval.is_empty());
    }

    fn ring(weights: &[f64], first: usize, remaining: Option<f64>) -> YearRing {
        let lords = [Graha::Sun, Graha::Moon, Graha::Mars, Graha::Mercury];
        YearRing {
            system: DashaName::Catalogued(DashaSystem::Mudda),
            ring: weights
                .iter()
                .zip(lords.iter().cycle())
                .map(|(&weight, &lord)| Share {
                    lord,
                    sign: None,
                    weight,
                })
                .collect(),
            first,
            remaining,
        }
    }

    /// A year holds a mahadasha a lord, and one more when the first lord's
    /// share is split across its two ends; a ring holds 1 to 255 lords.
    #[test]
    fn a_ring_counts_its_mahadashas_and_holds_255_lords_at_most() {
        assert_eq!(vimshottari_ring(0, None).mahadashas(), 9);
        assert_eq!(vimshottari_ring(0, Some(0.5)).mahadashas(), 10);
        assert!(ring(&[1.0; 255], 0, None).validate().is_ok());
        let past = ring(&[1.0; 256], 0, None).validate().unwrap_err();
        assert_eq!(past.field(), Some("ring"));
        let empty = ring(&[], 0, None).validate().unwrap_err();
        assert_eq!(empty.field(), Some("ring"));
        let knots = vec![OPENS, OPENS + 100.0, OPENS + 365.0];
        assert_eq!(Clock::new(knots.clone()).unwrap().knots(), knots);
    }

    /// Every period's seat is its lord's place in the ring, from a first
    /// place past the ring's middle round to the start again.
    #[test]
    fn every_period_sits_at_its_lord_s_place_in_the_ring() {
        let ring = vimshottari_ring(7, Some(0.25));
        let lords = ring.ring.clone();
        let dasha = YearDasha::new(ring, days(365.25), BirthPeriod::Compressed).unwrap();
        let depth = Depth::try_new(2).unwrap();
        for period in dasha.periods(dasha.year(), depth) {
            let seat = period.seat();
            assert!(seat < lords.len(), "{period:?}");
            assert_eq!(lords[seat].lord, period.lord, "{period:?}");
        }
    }

    /// A period's last child ends where the period's whole span does, to
    /// the bit, though the ring's weights summed from the child's own lord
    /// round to a different total than summed from the first.
    #[test]
    fn the_last_child_closes_its_parent_exactly() {
        // 0.1 + 0.2 + 0.3 is 0.6000000000000001; from the second, 0.6.
        let dasha = YearDasha::new(
            ring(&[0.1, 0.2, 0.3], 0, None),
            days(365.25),
            BirthPeriod::Compressed,
        )
        .unwrap();
        for maha in dasha.mahadashas() {
            let children: Vec<Period> = dasha.children(&maha).collect();
            assert_eq!(children.len(), 3, "{maha:?}");
            assert_eq!(
                children.last().unwrap().whole().to,
                maha.whole().to,
                "{maha:?}"
            );
            for pair in children.windows(2) {
                assert_eq!(pair[0].whole().to, pair[1].whole().from);
                assert!(pair[0].interval.days() > 0.0);
            }
        }
    }

    /// The last child's end is its parent's own, not the sum of the
    /// shares from the child's lord, which rounds to another total: from
    /// the 1e16, the two ones vanish, where in the ring's order they count.
    #[test]
    fn the_last_child_ends_on_its_parent_whatever_the_sums_round_to() {
        let clock = Clock::new(vec![0.0, 1.0]).unwrap();
        let dasha = YearDasha::new(
            ring(&[1.0, 1.0, 1e16], 0, None),
            clock,
            BirthPeriod::Compressed,
        )
        .unwrap();
        let heavy = dasha.mahadasha(0, 2).unwrap();
        assert_eq!(heavy.whole().to.get(), 1.0);
        let last = dasha.children(&heavy).last().unwrap();
        assert_eq!(last.whole().to.get(), 1.0);
    }

    /// Under the elapsed reading the first mahadasha's whole share began
    /// before the year: a child that ran out before the year opened is
    /// nothing, even one ending exactly as it opens, and a share of no
    /// weight is kept, empty, in its place.
    #[test]
    fn a_child_before_the_year_is_cut_and_an_empty_share_kept() {
        // The first lord, the Sun, has a quarter of the year and half of it
        // left: its whole runs from an eighth before the year to an eighth
        // into it. Its children, from the Sun: the Sun (an eighth before to
        // a sixteenth before), the Moon (to the year's opening, exactly) and
        // Mars (the rest).
        let dasha = YearDasha::new(
            ring(&[1.0, 1.0, 2.0], 0, Some(0.5)),
            days(256.0),
            BirthPeriod::Elapsed,
        )
        .unwrap();
        let first = dasha.mahadasha(0, 0).unwrap();
        assert_eq!(first.interval.from.get(), OPENS);
        assert_eq!(first.whole().from.get(), OPENS - 32.0);
        let lords: Vec<Graha> = dasha.children(&first).map(|child| child.lord).collect();
        assert_eq!(lords, [Graha::Mars]);
        // A weightless Moon keeps its place, empty.
        let dasha = YearDasha::new(
            ring(&[1.0, 0.0, 2.0], 0, None),
            days(256.0),
            BirthPeriod::Compressed,
        )
        .unwrap();
        let sun = dasha.mahadasha(0, 0).unwrap();
        let children: Vec<(Graha, f64)> = dasha
            .children(&sun)
            .map(|child| (child.lord, child.interval.days()))
            .collect();
        assert_eq!(children.len(), 3);
        assert_eq!(children[1], (Graha::Moon, 0.0));
    }

    /// No mahadasha runs outside the year.
    #[test]
    fn the_year_s_mahadashas_lie_inside_it() {
        let dasha = YearDasha::new(
            vimshottari_ring(0, None),
            days(360.0),
            BirthPeriod::Compressed,
        )
        .unwrap();
        assert!(dasha.mahadasha_at(OPENS - 1e-6).is_none());
        assert!(dasha.mahadasha_at(OPENS + 360.0).is_none());
        assert_eq!(dasha.mahadasha_at(OPENS).unwrap().path.to_string(), "0");
        assert_eq!(
            dasha
                .mahadasha_at(OPENS + 360.0 - 1e-6)
                .unwrap()
                .path
                .to_string(),
            "8"
        );
    }
}
