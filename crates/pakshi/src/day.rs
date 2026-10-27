//! A day's yamas as instants, and a native's birds read over them.
//!
//! A day runs from one sunrise to the next: its day half to sunset, its
//! night half on to the next sunrise, and both belong to the weekday whose
//! sunrise began them (crux P9), so "Tuesday night" runs into Wednesday's
//! civil date. Each half is five yamas (AG p5 v. 9; AY p. 102).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Paksha, Vara};
use teistro_core::error::Error;

use crate::tables::{
    Activity, Bird, Half, Quality, Relation, Relations, Sub, SubLengths, activity, death_bird,
    first_eater, relation, subs,
};

/// How a half is cut into yamas (crux P3).
///
/// ```
/// # use teistro_core::catalogue::{Paksha, Vara};
/// # use teistro_pakshi::{Bird, Day, Rules, read_day};
/// # // A bright Wednesday from 6:00 to 18:00, the cock's.
/// # let day = Day::new(0.0, 0.5, 1.0, Vara::Budhavara, Paksha::Shukla)?;
/// # let read = read_day(&day, Bird::Cock, &Rules::default());
/// use teistro_pakshi::Clock;
///
/// // A short day: sunset at 15:36. The stretched clock cuts the real day
/// // in five; the nazhigai clock runs thirty nazhigai from sunrise whatever.
/// let short = Day::new(0.0, 0.4, 1.0, Vara::Budhavara, Paksha::Shukla)?;
/// let stretched = read_day(&short, Bird::Cock, &Rules::default());
/// let nazhigai = read_day(&short, Bird::Cock, &Rules { clock: Clock::Nazhigai, ..Rules::default() });
/// assert!((stretched.yamas[4].span.to - 0.4).abs() < 1e-12);
/// assert!((nazhigai.yamas[4].span.to - 0.5).abs() < 1e-12);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "PakshiClock"))]
pub enum Clock {
    /// A fifth of the real day and a fifth of the real night, however
    /// long each is (AG's "a fifth of the day"; AY's watches).
    #[default]
    Stretched,
    /// Six nazhigai (2 h 24 min) each from sunrise, the day half the first
    /// thirty nazhigai and the night the next thirty, the 60-nazhigai day
    /// AG p5 v. 9 counts in, whatever the real sunset.
    Nazhigai,
}

/// What a day is read under.
///
/// ```
/// use teistro_pakshi::{Clock, Relations, Rules, SubLengths};
///
/// let rules: Rules = serde_json::from_str(r#"{"clock": "NAZHIGAI"}"#).unwrap();
/// assert_eq!((rules.clock, rules.subs, rules.relations), (Clock::Nazhigai, SubLengths::Agastya, Relations::Agastya));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "PakshiRules"))]
pub struct Rules {
    /// How the halves are cut into yamas.
    pub clock: Clock,
    /// How long the sub-periods run.
    pub subs: SubLengths,
    /// Whose friends and enemies the sub-periods' owners are judged by.
    pub relations: Relations,
}

/// One day of the almanac, as a reading needs it: its sunrise, sunset and
/// next sunrise as Julian days (UTC), its weekday and its paksha.
///
/// ```
/// # use teistro_core::catalogue::{Paksha, Vara};
/// # use teistro_pakshi::{Bird, Day, Rules, read_day};
/// # // A bright Wednesday from 6:00 to 18:00, the cock's.
/// # let day = Day::new(0.0, 0.5, 1.0, Vara::Budhavara, Paksha::Shukla)?;
/// # let read = read_day(&day, Bird::Cock, &Rules::default());
/// assert_eq!((day.vara, day.paksha), (Vara::Budhavara, Paksha::Shukla));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Day {
    /// The sunrise that begins it.
    pub sunrise: f64,
    /// Its sunset.
    pub sunset: f64,
    /// The sunrise that ends it.
    pub next_sunrise: f64,
    /// The weekday of its sunrise.
    pub vara: Vara,
    /// The paksha it is read in (crux P2: the one at its sunrise).
    pub paksha: Paksha,
}

impl Day {
    /// A day from its instants.
    ///
    /// ```
    /// use teistro_core::catalogue::{Paksha, Vara};
    /// use teistro_pakshi::Day;
    ///
    /// assert!(Day::new(0.0, 0.5, 1.0, Vara::Ravivara, Paksha::Shukla).is_ok());
    /// let backwards = Day::new(0.0, -0.1, 1.0, Vara::Ravivara, Paksha::Shukla).unwrap_err();
    /// assert_eq!(backwards.field(), Some("sunset"));
    /// let early = Day::new(0.0, 0.5, 0.4, Vara::Ravivara, Paksha::Shukla).unwrap_err();
    /// assert_eq!(early.field(), Some("nextSunrise"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` when the instants are not finite and in order,
    /// sunrise before sunset before the next sunrise (`sunset`,
    /// `nextSunrise`, naming the first out of order).
    pub fn new(
        sunrise: f64,
        sunset: f64,
        next_sunrise: f64,
        vara: Vara,
        paksha: Paksha,
    ) -> Result<Day, Error> {
        let finite = sunrise.is_finite() && sunset.is_finite() && next_sunrise.is_finite();
        if !finite || sunset <= sunrise {
            return Err(Error::invalid_arg(format!(
                "a day's sunset follows its sunrise: {sunrise} then {sunset}"
            ))
            .with_field("sunset"));
        }
        if next_sunrise <= sunset {
            return Err(Error::invalid_arg(format!(
                "a day's next sunrise follows its sunset: {sunset} then {next_sunrise}"
            ))
            .with_field("nextSunrise"));
        }
        Ok(Day {
            sunrise,
            sunset,
            next_sunrise,
            vara,
            paksha,
        })
    }

    /// Where half `half` begins and ends under `clock`.
    fn bounds(&self, half: Half, clock: Clock) -> (f64, f64) {
        match (clock, half) {
            (Clock::Stretched, Half::Day) => (self.sunrise, self.sunset),
            (Clock::Stretched, Half::Night) => (self.sunset, self.next_sunrise),
            (Clock::Nazhigai, Half::Day) => (self.sunrise, self.sunrise + 0.5),
            (Clock::Nazhigai, Half::Night) => (self.sunrise + 0.5, self.sunrise + 1.0),
        }
    }
}

/// A span of time, Julian days (UTC).
///
/// ```
/// # use teistro_core::catalogue::{Paksha, Vara};
/// # use teistro_pakshi::{Bird, Day, Rules, read_day};
/// # // A bright Wednesday from 6:00 to 18:00, the cock's.
/// # let day = Day::new(0.0, 0.5, 1.0, Vara::Budhavara, Paksha::Shukla)?;
/// # let read = read_day(&day, Bird::Cock, &Rules::default());
/// // The day's five yamas meet end to end.
/// for pair in read.yamas.windows(2) {
///     assert_eq!(pair[0].span.to, pair[1].span.from);
/// }
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "PakshiSpan"))]
pub struct Span {
    /// Its start.
    pub from: f64,
    /// Its end, excluded.
    pub to: f64,
}

/// One sub-period of a native's yama, timed.
///
/// ```
/// # use teistro_core::catalogue::{Paksha, Vara};
/// # use teistro_pakshi::{Bird, Day, Rules, read_day};
/// # // A bright Wednesday from 6:00 to 18:00, the cock's.
/// # let day = Day::new(0.0, 0.5, 1.0, Vara::Budhavara, Paksha::Shukla)?;
/// # let read = read_day(&day, Bird::Cock, &Rules::default());
/// use teistro_pakshi::Relation;
///
/// let yama = &read.yamas[1];
/// assert_eq!(yama.subs.len(), 5);
/// assert_eq!(yama.subs[0].owner_is, Relation::Own);
/// assert_eq!(yama.subs[0].span.from, yama.span.from);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SubPeriod {
    /// What is done, by whose main activity, for what share.
    #[serde(flatten)]
    pub sub: Sub,
    /// How the native's bird regards the owner.
    pub owner_is: Relation,
    /// When.
    pub span: Span,
}

/// One yama of a native's day.
///
/// ```
/// # use teistro_core::catalogue::{Paksha, Vara};
/// # use teistro_pakshi::{Bird, Day, Rules, read_day};
/// # // A bright Wednesday from 6:00 to 18:00, the cock's.
/// # let day = Day::new(0.0, 0.5, 1.0, Vara::Budhavara, Paksha::Shukla)?;
/// # let read = read_day(&day, Bird::Cock, &Rules::default());
/// use teistro_pakshi::{Activity, Half, Quality};
///
/// let second = &read.yamas[1];
/// assert_eq!((second.half, second.yama), (Half::Day, 2));
/// assert_eq!((second.activity, second.quality), (Activity::Sleeping, Quality::Bad));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Yama {
    /// Day or night.
    pub half: Half,
    /// Which yama of the half, 1 to 5.
    pub yama: u8,
    /// When.
    pub span: Span,
    /// The native's bird's main activity.
    pub activity: Activity,
    /// How it is judged.
    pub quality: Quality,
    /// Its sub-periods, in order.
    pub subs: Vec<SubPeriod>,
}

/// A native's bird read over one day.
///
/// ```
/// # use teistro_core::catalogue::{Paksha, Vara};
/// # use teistro_pakshi::{Bird, Day, Rules, read_day};
/// # // A bright Wednesday from 6:00 to 18:00, the cock's.
/// # let day = Day::new(0.0, 0.5, 1.0, Vara::Budhavara, Paksha::Shukla)?;
/// # let read = read_day(&day, Bird::Cock, &Rules::default());
/// use teistro_pakshi::Half;
///
/// assert_eq!(read.yamas.len(), 10);
/// assert_eq!(read.yamas[5].half, Half::Night);
/// // The owl eats first by day; the death bird is reported, and is not the cock.
/// assert_eq!(read.eaters[0], Bird::Owl);
/// assert!(!read.dead_today);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Reading {
    /// The day.
    pub day: Day,
    /// The native's bird.
    pub bird: Bird,
    /// The bird dead the whole day and night (crux P10: reported beside the
    /// yamas, which it does not change).
    pub death_bird: Bird,
    /// Whether that is the native's bird.
    pub dead_today: bool,
    /// The birds eating in the first yama of the day and of the night.
    pub eaters: [Bird; 2],
    /// The ten yamas, the day's five and then the night's.
    pub yamas: Vec<Yama>,
}

/// A native's bird read over a day under `rules`.
///
/// ```
/// use teistro_core::catalogue::{Paksha, Vara};
/// use teistro_pakshi::{Activity, Bird, Day, Rules, read_day};
///
/// // A bright Wednesday from 6:00 to 18:00: the cock sleeps in the second
/// // yama and dies in the third.
/// let day = Day::new(0.0, 0.5, 1.0, Vara::Budhavara, Paksha::Shukla)?;
/// let read = read_day(&day, Bird::Cock, &Rules::default());
/// assert_eq!(read.yamas[1].activity, Activity::Sleeping);
/// assert_eq!(read.yamas[2].activity, Activity::Dying);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[must_use]
pub fn read_day(day: &Day, bird: Bird, rules: &Rules) -> Reading {
    let yamas = [Half::Day, Half::Night]
        .into_iter()
        .flat_map(|half| (0..5_u8).map(move |yama| (half, yama)))
        .map(|(half, yama)| yama_of(day, bird, *rules, half, yama))
        .collect();
    let dead = death_bird(day.paksha, day.vara);
    Reading {
        day: *day,
        bird,
        death_bird: dead,
        dead_today: dead == bird,
        eaters: [Half::Day, Half::Night].map(|half| first_eater(day.paksha, half, day.vara)),
        yamas,
    }
}

/// One yama of a native's day.
fn yama_of(day: &Day, bird: Bird, rules: Rules, half: Half, yama: u8) -> Yama {
    let (start, end) = day.bounds(half, rules.clock);
    let length = (end - start) / 5.0;
    // Each end is the next yama's start, computed the same way, so the
    // yamas meet to the bit and an instant is in exactly one.
    let from = start + length * f64::from(yama);
    let to = if yama == 4 {
        end
    } else {
        start + length * f64::from(yama + 1)
    };
    let index = usize::from(yama);
    let main = activity(bird, day.paksha, half, day.vara, index);
    let mut at = from;
    let parts = subs(bird, day.paksha, half, day.vara, index, rules.subs);
    let last = parts.len() - 1;
    let subs = parts
        .into_iter()
        .enumerate()
        .map(|(k, sub)| {
            let start = at;
            at = if k == last {
                to
            } else {
                start + length * f64::from(sub.share) / 144.0
            };
            SubPeriod {
                sub,
                owner_is: relation(bird, sub.owner, day.paksha, rules.relations),
                span: Span {
                    from: start,
                    to: at,
                },
            }
        })
        .collect();
    Yama {
        half,
        yama: yama + 1,
        span: Span { from, to },
        activity: main,
        quality: main.quality(),
        subs,
    }
}

/// The yama and sub-period running at `instant`.
///
/// ```
/// # use teistro_core::catalogue::{Paksha, Vara};
/// # use teistro_pakshi::{Bird, Day, Rules, read_day};
/// # // A bright Wednesday from 6:00 to 18:00, the cock's.
/// # let day = Day::new(0.0, 0.5, 1.0, Vara::Budhavara, Paksha::Shukla)?;
/// # let read = read_day(&day, Bird::Cock, &Rules::default());
/// use teistro_pakshi::now;
///
/// // 09:36 is in the day's second yama, two fifths of a day-half in.
/// let at = now(&read, 0.15)?;
/// assert_eq!(at.yama.yama, 2);
/// assert!(usize::from(at.sub) < at.yama.subs.len());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Now {
    /// The yama.
    pub yama: Yama,
    /// Which of its sub-periods, from 0.
    pub sub: u8,
}

/// The yama and sub-period of `reading` running at `instant`.
///
/// ```
/// # use teistro_core::catalogue::{Paksha, Vara};
/// # use teistro_pakshi::{Bird, Day, Rules, read_day};
/// # // A bright Wednesday from 6:00 to 18:00, the cock's.
/// # let day = Day::new(0.0, 0.5, 1.0, Vara::Budhavara, Paksha::Shukla)?;
/// # let read = read_day(&day, Bird::Cock, &Rules::default());
/// use teistro_pakshi::now;
///
/// assert!(now(&read, 0.0).is_ok());
/// // The next sunrise ends the day, and is not in it.
/// assert_eq!(now(&read, 1.0).unwrap_err().field(), Some("instant"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// `INVALID_ARG` on `instant` outside the day's yamas: sunrise to the
/// next sunrise, or under [`Clock::Nazhigai`] sunrise to sixty nazhigai
/// after it.
pub fn now(reading: &Reading, instant: f64) -> Result<Now, Error> {
    for yama in &reading.yamas {
        if yama.span.from <= instant && instant < yama.span.to {
            let sub = yama
                .subs
                .iter()
                .position(|sub| sub.span.from <= instant && instant < sub.span.to)
                .and_then(|at| u8::try_from(at).ok())
                .unwrap_or(0);
            return Ok(Now {
                yama: yama.clone(),
                sub,
            });
        }
    }
    Err(Error::invalid_arg(format!(
        "the instant {instant} is outside the day's yamas, {} to {}",
        reading
            .yamas
            .first()
            .map_or(reading.day.sunrise, |yama| yama.span.from),
        reading
            .yamas
            .last()
            .map_or(reading.day.next_sunrise, |yama| yama.span.to)
    ))
    .with_field("instant"))
}
