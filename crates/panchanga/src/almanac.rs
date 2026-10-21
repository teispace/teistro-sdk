//! The almanac itself: one day at one place, assembled and stamped.
//!
//! Everything beneath this module answers one question well; this one
//! asks each of them once, asks the provider once, and hands the answers
//! on with a provenance. The rule it follows is the chart foundation's:
//! it holds what is needed to read a day and never what a module above it
//! computes.
//!
//! The range is the primary shape (principle 5). A month of days is what
//! an application actually asks for, and it is far cheaper than thirty
//! days computed one at a time: consecutive windows share a boundary, so
//! one crossing search over the month replaces thirty overlapping ones.

use serde::{Deserialize, Serialize};
use teistro_astro::completion::Completion;
use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::lockstep::Lockstep;
use teistro_astro::precession::PrecessionModel;
use teistro_astro::rise_set::Solver;
use teistro_calendar::solar::{SolarModel, civil_day_lights};
use teistro_calendar::{CalendarDate, CalendarSystem, FixedDay};
use teistro_chart::zodiac::ChartZodiac;
use teistro_core::catalogue::{Direction, Rashi, Ritu, Vara};
use teistro_core::envelope::{Envelope, Hash, Provenance, Version, content_hash};
use teistro_core::error::{Error, Status};
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Place, Ut1, Utc};
use teistro_core::settings::{
    Centre, GhatiReckoning, MoonEvents, Resolved, RituReckoning, Settings,
};
use teistro_core::time::LocalClock;
use teistro_port_ephemeris::{Body, EphemerisProvider, Frame, Horizon, HorizonEventKind};
use teistro_time::ghati::{self, GhatiPala};
use teistro_time::hora::{self, Hora};
use teistro_time::local_day::{
    Carried, LocalDay, local_day, local_day_carrying, local_day_lit, local_midnight,
};

use crate::festival::{FestivalDay, Moonrise};
use crate::limb::{self, Limbs, RangeCrossings, Zodiac};
use crate::month::{self, LunarMonth};
use crate::omen::{self, Omens};
use crate::period::{self, Durmuhurta, Kaalas, Muhurtas, Part};
use crate::sankalpa::{self, AyanaReckoning, Sankalpa};
use crate::sky::{self, MoonDay, SunDay};
use crate::span::{self, Span, Sunrises};
use crate::year::{LunarYear, name_counts};
use teistro_calendar::samvatsara::JovianYear;

/// The almanac of one day at one place.
///
/// A plain value: no interior mutability, no handle, no lazy field. Two
/// panchangas of the same inputs under the same settings are equal field
/// for field, which is what the determinism contract asks of everything
/// that crosses a binding.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Panchanga {
    /// The day, with its arc, its vara, its polar state and the sunrise
    /// convention it was reckoned under.
    pub day: LocalDay,
    /// What the limb spans are clipped to: the arc under a sunrise
    /// boundary, the civil day under a midnight one.
    pub window: Interval,
    /// The four moving limbs.
    pub limbs: Limbs,
    /// The inauspicious eighths of the daylight the day has.
    pub kaalas: Vec<Kaalas>,
    /// Eight choghadiya of the daylight and eight of the night, when the
    /// day has both.
    pub choghadiya: Vec<Part>,
    /// Twenty-four horas, from `time::hora`.
    pub horas: Vec<Hora>,
    /// The thirty muhurtas, with Abhijit and Brahma muhurta named.
    pub muhurtas: Muhurtas,
    /// The day's censured muhurtas under `panchanga.durmuhurta`, in the
    /// order they run (`03-design/panchanga-day.md` §21, crux C398).
    pub durmuhurtas: Vec<Durmuhurta>,
    /// The lunar month, under both conventions.
    pub month: LunarMonth,
    /// What the Moon did.
    pub moon: MoonDay,
    /// What the Sun did.
    pub sun: SunDay,
    /// What the day is said to be.
    pub omens: Omens,
    /// How the day's ghatis are counted, `day.ghati_reckoning` as
    /// applied: what [`Panchanga::ghati_pala`] reads.
    pub ghati_reckoning: GhatiReckoning,
}

impl Panchanga {
    /// The day's vara, which is the arc's and not the civil date's.
    #[must_use]
    pub const fn vara(&self) -> Vara {
        self.day.vara
    }

    /// The day's Nepal Sambat date: its year, its month and its half
    /// (`03-design/calendar-indian-lunisolar.md` §11).
    #[must_use]
    pub fn nepal_sambat(&self) -> crate::NepalSambatDate {
        crate::NepalSambatDate::of(self)
    }

    /// Which of the day's two sunrises a span of this day was running
    /// at ([`Span::sunrises`]), by the day's own sunrises.
    #[must_use]
    pub fn sunrises<T>(&self, span: &Span<T>) -> Sunrises {
        span.sunrises(self.day.sunrise, self.day.next_sunrise)
    }

    /// The member of a limb that names two days running: the one this
    /// day's sunrise and the next both fall in (vriddhi).
    ///
    /// ```no_run
    /// # fn day() -> teistro_panchanga::almanac::Panchanga { unimplemented!() }
    /// let day = day();
    /// if let Some(tithi) = day.vriddhi(&day.limbs.tithi) {
    ///     println!("{tithi:?} names tomorrow too");
    /// }
    /// for tithi in day.kshaya(&day.limbs.tithi) {
    ///     println!("{tithi:?} names no day");
    /// }
    /// ```
    #[must_use]
    pub fn vriddhi<T: Copy>(&self, spans: &[Span<T>]) -> Option<T> {
        spans
            .iter()
            .find(|span| self.sunrises(span).is_vriddhi())
            .map(|span| span.member)
    }

    /// The members of a limb that name no day: begun after this day's
    /// sunrise and ended before the next (kshaya).
    #[must_use]
    pub fn kshaya<T: Copy>(&self, spans: &[Span<T>]) -> Vec<T> {
        spans
            .iter()
            .filter(|span| self.sunrises(span).is_kshaya())
            .map(|span| span.member)
            .collect()
    }

    /// An instant of the day as ghati, pala and vipala from its sunrise,
    /// counted under `ghati_reckoning`; an instant past the next sunrise
    /// counts as the next sunrise, and one before this sunrise as the
    /// sunrise, so a span's end reads as the patro prints it, clipped to
    /// the day.
    ///
    /// # Errors
    ///
    /// A reckoning this build does not know.
    pub fn ghati_pala(&self, instant: JulianDay<Utc>) -> Result<GhatiPala, Error> {
        let clamped = if instant.get() < self.day.sunrise.get() {
            self.day.sunrise
        } else if instant.get() >= self.day.next_sunrise.get() {
            // The day's own count runs to its next sunrise, which is
            // outside `LocalDay::contains`; its full length is the count.
            return Ok(ghati::day_length(
                &self.day,
                self.ghati_reckoning.try_into()?,
            ));
        } else {
            instant
        };
        ghati::ghati_pala(&self.day, clamped, self.ghati_reckoning.try_into()?)
    }

    /// The direction not to travel in.
    #[must_use]
    pub const fn disha_shool(&self) -> Direction {
        self.omens.disha_shool
    }

    /// The direction the day's nakshatras block travel in, while they run
    /// ([`omen::nakshatra_shool`]): empty on a day the Moon crosses none
    /// of the four.
    ///
    /// ```no_run
    /// # fn day() -> teistro_panchanga::almanac::Panchanga { unimplemented!() }
    /// let day = day();
    /// println!("not {:?} by the vara", day.disha_shool());
    /// for shool in day.nakshatra_shool() {
    ///     println!("not {:?} until {}", shool.member, shool.inside.to);
    /// }
    /// ```
    #[must_use]
    pub fn nakshatra_shool(&self) -> Vec<Span<Direction>> {
        omen::nakshatra_shools(&self.limbs.nakshatra)
    }

    /// The direction the Moon dwells in over the day, by the sign it
    /// stood in ([`omen::chandra_vasa`]).
    #[must_use]
    pub fn chandra_vasa(&self) -> Vec<Span<Direction>> {
        omen::chandra_vasas(&self.moon.signs)
    }

    /// Whose chandrashtama the day holds: each member is a **natal** Moon
    /// sign, and its intervals the Moon's run through the eighth sign from
    /// it ([`omen::chandrashtama_of`]).
    ///
    /// ```no_run
    /// # fn day() -> teistro_panchanga::almanac::Panchanga { unimplemented!() }
    /// for span in day().chandrashtama() {
    ///     println!("chandrashtama for a {:?} Moon until {}", span.member, span.inside.to);
    /// }
    /// ```
    #[must_use]
    pub fn chandrashtama(&self) -> Vec<Span<Rashi>> {
        omen::chandrashtama(&self.moon.signs)
    }

    /// Whether an instant falls inside one of the day's durmuhurtas.
    #[must_use]
    pub fn is_durmuhurta(&self, instant: JulianDay<Utc>) -> bool {
        self.durmuhurtas
            .iter()
            .any(|durmuhurta| durmuhurta.at.contains(instant))
    }

    /// The tithi running at an instant.
    ///
    /// One of **four**, one a moving limb, and for a while there were two:
    /// the tithi and the nakshatra could be asked for at an instant and
    /// the yoga and the karana could not, though the data and `span::at`
    /// were both already here. A caller wanting the limbs at a birth had
    /// to reach into `limbs` for half of them.
    #[must_use]
    pub fn tithi_at(
        &self,
        instant: JulianDay<Utc>,
    ) -> Option<&Span<teistro_core::catalogue::Tithi>> {
        span::at(&self.limbs.tithi, instant)
    }

    /// The nakshatra the Moon was in at an instant.
    #[must_use]
    pub fn nakshatra_at(
        &self,
        instant: JulianDay<Utc>,
    ) -> Option<&Span<teistro_core::catalogue::Nakshatra>> {
        span::at(&self.limbs.nakshatra, instant)
    }

    /// The nitya yoga running at an instant.
    #[must_use]
    pub fn yoga_at(&self, instant: JulianDay<Utc>) -> Option<&Span<teistro_core::catalogue::Yoga>> {
        span::at(&self.limbs.yoga, instant)
    }

    /// The karana running at an instant.
    #[must_use]
    pub fn karana_at(
        &self,
        instant: JulianDay<Utc>,
    ) -> Option<&Span<teistro_core::catalogue::Karana>> {
        span::at(&self.limbs.karana, instant)
    }

    /// The hora running at an instant.
    #[must_use]
    pub fn hora_at(&self, instant: JulianDay<Utc>) -> Option<&Hora> {
        self.horas
            .iter()
            .find(|hora| instant.get() >= hora.start.get() && instant.get() < hora.end.get())
            .or_else(|| {
                self.horas
                    .last()
                    .filter(|hora| instant.get() <= hora.end.get())
            })
    }

    /// The choghadiya running at an instant.
    #[must_use]
    pub fn choghadiya_at(&self, instant: JulianDay<Utc>) -> Option<&Part> {
        period::choghadiya_at(&self.choghadiya, instant)
    }

    /// Whether an instant falls inside one of the day's inauspicious
    /// eighths.
    #[must_use]
    pub fn is_inauspicious(&self, instant: JulianDay<Utc>) -> bool {
        period::is_inauspicious(&self.kaalas, instant)
    }
}

/// What the almanac hashes as its input.
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct Input {
    date: String,
    latitude_deg: f64,
    longitude_deg: f64,
    altitude_m: f64,
}

/// What a sankalpa's answer is a function of.
#[derive(Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct SankalpaInput {
    at: f64,
    latitude_deg: f64,
    longitude_deg: f64,
    altitude_m: f64,
    ayana: AyanaReckoning,
}

/// How long a moment is, days, over which a body's sign at an instant is
/// read: a tenth of a second, inside which no body changes sign but on
/// the instant itself.
const MOMENT_DAYS: f64 = 1e-6;

/// The same, for a range.
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct RangeInput {
    from: String,
    to: String,
    latitude_deg: f64,
    longitude_deg: f64,
    altitude_m: f64,
}

/// How many days the range from `first` to `last` holds, inclusive,
/// refused as every range is: one that ends before it begins, or holds
/// more than [`MOST_DAYS`], named by `to`.
///
/// # Errors
///
/// `INVALID_ARG` for a range the wrong way round, `OUT_OF_RANGE` for one
/// too long.
fn days_in(
    first: FixedDay,
    last: FixedDay,
    from: &CalendarDate,
    to: &CalendarDate,
) -> Result<usize, Error> {
    let days = first.days_until(last);
    if days < 0 {
        return Err(Error::invalid_arg(format!(
            "a range of days ends before it begins: {from} to {to}"
        ))
        .with_field("to"));
    }
    let count = usize::try_from(days).unwrap_or(0) + 1;
    if count > MOST_DAYS {
        return Err(Error::new(
            Status::OutOfRange,
            format!("a range holds at most {MOST_DAYS} days, not {count}"),
        )
        .with_field("to"));
    }
    Ok(count)
}

/// The most days one range may hold.
///
/// A year and a day: an application asking for more is asking for a
/// different shape, and the refusal names the limit rather than running
/// for a minute.
pub const MOST_DAYS: usize = 366;

/// The almanac over a provider, a solar model for the day's arcs, a
/// calendar and a clock.
///
/// The same collaborators the chart foundation takes, because it is the
/// same day model underneath.
pub struct Almanac<'a, P: EphemerisProvider + ?Sized> {
    provider: &'a P,
    resolved: &'a Resolved,
    model: &'a dyn SolarModel,
    calendar: &'a dyn CalendarSystem,
    clock: &'a dyn LocalClock,
    precession: PrecessionModel,
    delta_t: DeltaTModel,
}

impl<P: EphemerisProvider + ?Sized> core::fmt::Debug for Almanac<'_, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Almanac")
            .field("profile", &self.resolved.profile.as_str())
            .field("model", &self.model.describe())
            .finish_non_exhaustive()
    }
}

impl<'a, P: EphemerisProvider + ?Sized> Almanac<'a, P> {
    /// An almanac.
    pub const fn new(
        provider: &'a P,
        resolved: &'a Resolved,
        model: &'a dyn SolarModel,
        calendar: &'a dyn CalendarSystem,
        clock: &'a dyn LocalClock,
        precession: PrecessionModel,
        delta_t: DeltaTModel,
    ) -> Almanac<'a, P> {
        Almanac {
            provider,
            resolved,
            model,
            calendar,
            clock,
            precession,
            delta_t,
        }
    }

    /// The resolved settings every part reads.
    const fn settings(&self) -> &Settings {
        &self.resolved.settings
    }

    /// The almanac of one date at one place.
    ///
    /// # Errors
    ///
    /// The day's own refusals (a polar day under `UNDEFINED`, a date the
    /// calendar does not have), an ayanamsha the catalogue cannot
    /// evaluate, a provider that cannot answer for the window, or a
    /// muhurta yoga or durmuhurta table the SDK does not ship.
    pub fn day(&self, date: &CalendarDate, place: &Place) -> Result<Envelope<Panchanga>, Error> {
        let (value, frame) = self.value(date, place, &mut Memo::default())?;
        let provenance = self.provenance(
            content_hash(&Input {
                date: date.to_string(),
                latitude_deg: place.latitude.get(),
                longitude_deg: place.longitude.get(),
                altitude_m: place.altitude.get(),
            }),
            frame,
        );
        Ok(Envelope::new(value, provenance))
    }

    /// The almanac of the day an instant belongs to, which before sunrise
    /// is not the day of its civil date.
    ///
    /// # Errors
    ///
    /// As [`Almanac::day`].
    pub fn at(&self, instant: JulianDay<Utc>, place: &Place) -> Result<Envelope<Panchanga>, Error> {
        let day = teistro_chart::day::chart_day(
            self.model,
            self.calendar,
            self.clock,
            place,
            instant,
            self.settings().day.polar_day_policy,
        )?;
        self.day(&day.day.date, place)
    }

    /// The almanac of every day in a range, inclusive of both ends.
    ///
    /// # Errors
    ///
    /// As [`Almanac::day`], plus `OUT_OF_RANGE` for a range that ends
    /// before it begins or holds more than [`MOST_DAYS`] days.
    pub fn between(
        &self,
        from: &CalendarDate,
        to: &CalendarDate,
        place: &Place,
    ) -> Result<Envelope<Vec<Panchanga>>, Error> {
        self.run(from, to, place, Moon::Searched, Self::value)
    }

    /// What festival rules read of every day in a range, inclusive of
    /// both ends ([`FestivalDay`]): its sunrise, sunset and next sunrise,
    /// its lunar month and its tithis and nakshatras, **each the value
    /// [`Almanac::between`] gives that day**, to the bit, and stamped as
    /// it stamps the range. The day's other parts are not computed, the
    /// Moon's rises and sets least of all, which are most of a day's
    /// requests: a year of festivals is a year of these.
    ///
    /// # Errors
    ///
    /// As [`Almanac::between`], less the refusals of the parts it does not
    /// compute.
    pub fn festival_days(
        &self,
        from: &CalendarDate,
        to: &CalendarDate,
        place: &Place,
    ) -> Result<Envelope<Vec<FestivalDay>>, Error> {
        self.run(from, to, place, Moon::Unread, Self::festival_value)
    }

    /// Fills each day's first moonrise between its sunrise and the next
    /// ([`FestivalDay::moonrise`]), for the rules that read one
    /// ([`crate::festival::Window::Moonrise`]): every day's search in
    /// lockstep, so a round of them is one request. The search is the one
    /// [`Almanac::between`] makes over a sunrise day, so a day's moonrise
    /// is its panchanga's first, to the bit. A polar day is left
    /// unsearched, since it has no windows.
    ///
    /// # Errors
    ///
    /// A search the provider refuses.
    pub fn moonrises(&self, days: &mut [FestivalDay], place: &Place) -> Result<(), Error> {
        let completion = Completion::new(
            self.provider,
            self.settings().provider.overrides,
            self.delta_t,
        );
        let solver = self.moon_solver(&completion, place);
        let round = Lockstep::new();
        let searched: Vec<Interval> = days
            .iter()
            .filter(|day| day.normal)
            .map(FestivalDay::sunrise_day)
            .collect();
        let mut found = round
            .run(searched.iter().map(|window| {
                solver.event_in(
                    &round,
                    HorizonEventKind::Rise,
                    JulianDay::<Ut1>::literal(window.from.get()),
                    window.days(),
                )
            }))
            .into_iter();
        for day in days.iter_mut().filter(|day| day.normal) {
            let event = found
                .next()
                .ok_or_else(|| Error::internal("a moonrise search came apart"))??;
            day.moonrise = match event {
                Some(event) if event.instant.get() < day.next_sunrise.get() => {
                    Moonrise::Rose(JulianDay::literal(event.instant.get()))
                }
                _ => Moonrise::NoRise,
            };
        }
        Ok(())
    }

    /// Every day of a range as `value` computes it, the range found
    /// ahead and the days carried one to the next, under one stamp.
    fn run<T>(
        &self,
        from: &CalendarDate,
        to: &CalendarDate,
        place: &Place,
        moon: Moon,
        value: impl Fn(&Self, &CalendarDate, &Place, &mut Memo) -> Result<(T, Frame), Error>,
    ) -> Result<Envelope<Vec<T>>, Error> {
        let (first, last) = (self.calendar.fixed_of(from)?, self.calendar.fixed_of(to)?);
        let count = days_in(first, last, from, to)?;
        let mut values = Vec::with_capacity(count);
        // The frame is the first day's: the batch asks the provider for
        // one frame, so one is what the stamp names -- and it comes back
        // from the day that computed it rather than being computed
        // again.
        let mut frame = None;
        // What one day found and the next can read rather than search
        // again: the day before's sunrise and sunset, and the lunation.
        let mut memo = Memo {
            last: Some(last),
            ..Memo::default()
        };
        self.ahead(first, count, place, moon, &mut memo);
        for step in 0..count {
            let offset = i64::try_from(step).unwrap_or(0);
            let date = self.calendar.date_of(first.plus_days(offset))?;
            let (value, asked) = value(self, &date, place, &mut memo)?;
            frame = frame.or(Some(asked));
            values.push(value);
        }
        let Some(frame) = frame else {
            return Err(Error::internal("a range of days answered no day"));
        };
        let provenance = self.provenance(
            content_hash(&RangeInput {
                from: from.to_string(),
                to: to.to_string(),
                latitude_deg: place.latitude.get(),
                longitude_deg: place.longitude.get(),
                altitude_m: place.altitude.get(),
            }),
            frame,
        );
        Ok(Envelope::new(values, provenance))
    }

    /// The time a sankalpa recites for a rite begun at `at`
    /// (`03-design/sankalpa.md`): the day's year, ayana, season and month
    /// as its sunrise holds them, and the tithi, nakshatra, yoga, karana
    /// and the Sun's, the Moon's and Jupiter's signs as the instant does.
    ///
    /// The day is the one the instant falls in, sunrise to sunrise, so a
    /// rite before dawn is the day before's. The month and the season are
    /// reckoned as the profile's days are, and the half of the year under
    /// `ayana`; the answer names each.
    ///
    /// # Errors
    ///
    /// As [`Almanac::day`] for the instant's day and the one before, and
    /// [`Almanac::years`] for its year.
    pub fn sankalpa(
        &self,
        at: JulianDay<Utc>,
        place: &Place,
        ayana: AyanaReckoning,
    ) -> Result<Envelope<Sankalpa>, Error> {
        let settings = self.settings();
        // The civil day the instant falls on, and the day before it when
        // the instant is before that day's sunrise.
        let (civil, _) = FixedDay::from_local_jd(self.clock.local_jd(at));
        let mut found = None;
        for back in [0, 1] {
            let date = self.calendar.date_of(civil.plus_days(-back))?;
            let (day, frame) = self.value(&date, place, &mut Memo::default())?;
            if day.window.from.get() <= at.get() && at.get() < day.window.to.get() {
                found = Some((day, frame));
                break;
            }
        }
        let Some((day, frame)) = found else {
            return Err(Error::new(
                Status::OutOfRange,
                format!(
                    "the instant {at} falls in neither its civil day's window nor the day before's"
                ),
            ));
        };
        let year = self
            .years(&day.day.date, &day.day.date, place)?
            .value
            .into_iter()
            .next()
            .ok_or_else(|| Error::internal("a day's own range answered no lunar year"))?;
        let completion = Completion::new(self.provider, settings.provider.overrides, self.delta_t);
        let (instant_frame, zodiac) = self.at_instant(&completion, at)?;
        let mut longitudes = completion.longitudes(instant_frame);
        if instant_frame.centre == teistro_port_ephemeris::Centre::Topocentric {
            longitudes = longitudes.with_observer(*place);
        }
        let sign = |body: Body, from: JulianDay<Utc>, zodiac: Zodiac| -> Result<Rashi, Error> {
            Self::signs(
                &longitudes,
                body,
                Interval::new(from, next_by(from, MOMENT_DAYS))?,
                zodiac,
            )?
            .first()
            .map(|span| span.member)
            .ok_or_else(|| Error::internal(format!("{body:?} stood in no sign at {from}")))
        };
        let ayana_at_sunrise = match ayana {
            AyanaReckoning::Sidereal => day.sun.ayana,
            AyanaReckoning::Tropical => sky::ayana_of(sign(
                Body::Sun,
                day.day.sunrise,
                Zodiac {
                    ayanamsha: None,
                    ..zodiac
                },
            )?),
        };
        let limb = |name: &str| {
            Error::internal(format!(
                "the day's {name} spans do not hold the instant {at}"
            ))
        };
        let tithi = sankalpa::at(&day.limbs.tithi, at).ok_or_else(|| limb("tithi"))?;
        let value = Sankalpa {
            at,
            year,
            ayana: ayana_at_sunrise,
            ayana_reckoning: ayana,
            ritu: day.sun.ritu,
            ritu_reckoning: settings.panchanga.ritu,
            month: day.month,
            paksha: tithi.attributes().paksha,
            tithi,
            nakshatra: sankalpa::at(&day.limbs.nakshatra, at).ok_or_else(|| limb("nakshatra"))?,
            yoga: sankalpa::at(&day.limbs.yoga, at).ok_or_else(|| limb("yoga"))?,
            karana: sankalpa::at(&day.limbs.karana, at).ok_or_else(|| limb("karana"))?,
            sun: sign(Body::Sun, at, zodiac)?,
            moon: sign(Body::Moon, at, zodiac)?,
            jupiter: sign(Body::Jupiter, at, zodiac)?,
            date: day.day.date,
            vara: day.day.vara,
            sunrise: day.day.sunrise,
            next_sunrise: day.day.next_sunrise,
        };
        let provenance = self.provenance(
            content_hash(&SankalpaInput {
                at: at.get(),
                latitude_deg: place.latitude.get(),
                longitude_deg: place.longitude.get(),
                altitude_m: place.altitude.get(),
                ayana,
            }),
            frame,
        );
        Ok(Envelope::new(value, provenance))
    }

    /// The lunar years the days of a range fall in, each named by the
    /// sixty-year cycle under `calendars.samvatsara`
    /// (`03-design/samvatsara-measured.md`).
    ///
    /// A year runs from the sunrise opening Chaitra Shukla Pratipada to
    /// the next year's, so a day's year is the one holding its sunrise;
    /// the first year is the one holding the range's first day, and the
    /// list runs until a year holds its last. The openings are found in
    /// this almanac's sky, the one that named the days' months.
    ///
    /// # Errors
    ///
    /// As [`Almanac::between`], and a place whose sunrise the polar
    /// policy leaves undefined on a pratipada.
    pub fn years(
        &self,
        from: &CalendarDate,
        to: &CalendarDate,
        place: &Place,
    ) -> Result<Envelope<Vec<LunarYear>>, Error> {
        let (first, last) = (self.calendar.fixed_of(from)?, self.calendar.fixed_of(to)?);
        days_in(first, last, from, to)?;
        let start = local_midnight(self.clock, first)?;
        let end = local_midnight(self.clock, last.plus_days(1))?;
        let completion = Completion::new(
            self.provider,
            self.settings().provider.overrides,
            self.delta_t,
        );
        let count = self.settings().calendars.samvatsara;
        // The opening at or before the range's first instant, then the
        // one before it too when the range starts ahead of that year's
        // first sunrise.
        let mut opened = self.opening(&completion, place, start)?;
        let mut began = self.pratipada(place, opened)?;
        if began.get() > start.get() {
            opened = self.opening(&completion, place, next_by(opened, -1.0))?;
            began = self.pratipada(place, opened)?;
        }
        // Each year's opening and first sunrise, and the next year's
        // after the last: a year's name can wait on the next one's.
        let mut run = vec![(opened, began)];
        while let Some(&(last, first_sunrise)) = run.last() {
            if first_sunrise.get() >= end.get() && run.len() > 1 {
                break;
            }
            // A lunar year is at most 385 days, so the opening before
            // 390 days on is the next one.
            let next = self.opening(&completion, place, next_by(last, 390.0))?;
            run.push((next, self.pratipada(place, next)?));
        }
        let text = &teistro_calendar::samvatsara::Parameters::TEXT;
        let mut running = run
            .iter()
            .map(|&(_, sunrise)| JovianYear::at(text, sunrise).map(|year| year.count))
            .collect::<Result<Vec<i64>, Error>>()?;
        // The years before the first, as many as the naming reads.
        let mut earliest = opened;
        let mut before = 0;
        while (before..running.len()).any(|index| name_counts(&running, index, count).is_none()) {
            if before == MOST_YEARS_BACK {
                return Err(Error::new(
                    Status::NotConverged,
                    format!(
                        "the samvatsara at {began} was not settled by {MOST_YEARS_BACK} years before it"
                    ),
                ));
            }
            earliest = self.opening(&completion, place, next_by(earliest, -1.0))?;
            let sunrise = self.pratipada(place, earliest)?;
            running.insert(0, JovianYear::at(text, sunrise)?.count);
            before += 1;
        }
        let named = |index: usize| {
            name_counts(&running, before + index, count).ok_or_else(|| {
                Error::internal("a lunar year's name was checked settled and is not")
            })
        };
        let mut years = Vec::with_capacity(run.len().saturating_sub(1));
        for (index, pair) in run.windows(2).enumerate() {
            let &[(opened, began), (_, ended)] = pair else {
                continue;
            };
            years.push(LunarYear::of(
                count,
                opened,
                began,
                ended,
                (named(index)?, named(index + 1)?),
            )?);
        }
        let frame = self.at_instant(&completion, opened)?.0;
        let provenance = self.provenance(
            content_hash(&RangeInput {
                from: from.to_string(),
                to: to.to_string(),
                latitude_deg: place.latitude.get(),
                longitude_deg: place.longitude.get(),
                altitude_m: place.altitude.get(),
            }),
            frame,
        );
        Ok(Envelope::new(years, provenance))
    }

    /// The new moon that opened the first Chaitra at or before an
    /// instant, in this almanac's sky.
    fn opening(
        &self,
        completion: &Completion<'_, P>,
        place: &Place,
        at: JulianDay<Utc>,
    ) -> Result<JulianDay<Utc>, Error> {
        let (frame, zodiac) = self.at_instant(completion, at)?;
        let mut longitudes = completion.longitudes(frame);
        if frame.centre == teistro_port_ephemeris::Centre::Topocentric {
            longitudes = longitudes.with_observer(*place);
        }
        crate::year::chaitra_opening(&longitudes, zodiac, at)
    }

    /// The sunrise that opens the day after a new moon's: the first
    /// sunrise after it, whichever civil date it falls on.
    fn pratipada(&self, place: &Place, new_moon: JulianDay<Utc>) -> Result<JulianDay<Utc>, Error> {
        let day = teistro_chart::day::chart_day(
            self.model,
            self.calendar,
            self.clock,
            place,
            new_moon,
            self.settings().day.polar_day_policy,
        )?;
        Ok(day.day.next_sunrise)
    }

    /// One day's value, unstamped, **and the frame it asked the provider
    /// for** — which the stamp needs and which this is the only place
    /// that computes.
    ///
    /// Handed back rather than recomputed. Asking `self.frame` a second
    /// time for the stamp cost `panchanga` **8.95%** of its instructions
    /// and the instruction-count gate refused it at a 3% budget: a frame
    /// is an ayanamsha at an instant, so it is a precession evaluation
    /// and not a field read.
    /// What festival rules read of one day, unstamped, and its frame: the
    /// first steps of [`Almanac::value`], taken the same way over the same
    /// memo, so each part is the value that day's panchanga holds.
    fn festival_value(
        &self,
        date: &CalendarDate,
        place: &Place,
        memo: &mut Memo,
    ) -> Result<(FestivalDay, Frame), Error> {
        let date = self.calendar.date_of(self.calendar.fixed_of(date)?)?;
        let fixed = self.calendar.fixed_of(&date)?;
        let (day, _) = self.local_day_of(&date, fixed, place, memo)?;
        let window = self.window(&day)?;
        // The daylight and the night a panchanga divides, refused alike.
        Interval::new(day.sunrise, day.sunset)?;
        Interval::new(day.sunset, day.next_sunrise)?;
        let completion = Completion::new(
            self.provider,
            self.settings().provider.overrides,
            self.delta_t,
        );
        let (frame, zodiac) = self.at_instant(&completion, window.from)?;
        let mut longitudes = completion.longitudes(frame);
        if frame.centre == teistro_port_ephemeris::Centre::Topocentric {
            longitudes = longitudes.with_observer(*place);
        }
        let known = memo.crossings(&longitudes, (zodiac, frame), window, fixed)?;
        let limbs = limb::limbs_with(&longitudes, window, zodiac, known.as_ref())?;
        let month = self.month(&longitudes, window, &limbs, (zodiac, frame), memo)?;
        memo.known = known.map(|known| (frame, known));
        Ok((
            FestivalDay::of(&day, month, limbs.tithi, limbs.nakshatra),
            frame,
        ))
    }

    fn value(
        &self,
        date: &CalendarDate,
        place: &Place,
        memo: &mut Memo,
    ) -> Result<(Panchanga, Frame), Error> {
        let settings = self.settings();
        // The date is taken through the calendar and back, so that a day
        // asked for by a date a caller wrote and the same day reached
        // from an instant are equal values and not merely the same day:
        // a `CalendarDate` a caller builds carries no era numbers and one
        // the calendar renders does.
        let date = self.calendar.date_of(self.calendar.fixed_of(date)?)?;
        let fixed = self.calendar.fixed_of(&date)?;
        let (day, found_moon) = self.local_day_of(&date, fixed, place, memo)?;
        let window = self.window(&day)?;
        let daylight = Interval::new(day.sunrise, day.sunset)?;
        let night = Interval::new(day.sunset, day.next_sunrise)?;

        let completion = Completion::new(self.provider, settings.provider.overrides, self.delta_t);
        // One ayanamsha evaluation for the day; everything below reads
        // it rather than asking again at the same instant.
        let (frame, zodiac) = self.at_instant(&completion, window.from)?;
        let mut longitudes = completion.longitudes(frame);
        if frame.centre == teistro_port_ephemeris::Centre::Topocentric {
            longitudes = longitudes.with_observer(*place);
        }
        let known = memo.crossings(&longitudes, (zodiac, frame), window, fixed)?;
        let limbs = limb::limbs_with(&longitudes, window, zodiac, known.as_ref())?;

        let horas = hora::horas(&day, settings.day.hora_reckoning.try_into()?)?;
        let previous_night = match memo.day.take() {
            Some((before, yesterday)) if before.plus_days(1) == fixed => {
                Interval::new(yesterday.sunset, yesterday.next_sunrise).ok()
            }
            _ => self.previous_night(place, &date)?,
        };
        memo.day = Some((fixed, day.clone()));
        let muhurtas = period::muhurtas(daylight, night, previous_night, day.vara);
        let durmuhurtas = period::durmuhurtas(&muhurtas, day.vara, settings.panchanga.durmuhurta)?;
        let month = self.month(&longitudes, window, &limbs, (zodiac, frame), memo)?;
        // The Moon's own day reads geocentric longitudes, which are the
        // range's crossings' source unless the almanac is topocentric.
        let geocentric = known
            .as_ref()
            .filter(|_| frame.centre != teistro_port_ephemeris::Centre::Topocentric);
        let moon = self.moon(
            &completion,
            place,
            (&day, window),
            (frame, zodiac, geocentric),
            found_moon,
        )?;
        let sun_signs =
            limb::signs_known(&longitudes, Body::Sun, window, zodiac, None, known.as_ref())?;
        let ritu = self.ritu(
            (&date, place),
            &sun_signs,
            &month,
            (&longitudes, window, zodiac),
        )?;
        let sun = sky::sun_day(sun_signs, ritu);
        let omens = Omens {
            panchaka: omen::panchaka(
                &limbs.nakshatra,
                settings.panchanga.panchaka_start,
                |within| {
                    limb::signs_known(
                        &longitudes,
                        Body::Moon,
                        within,
                        zodiac,
                        None,
                        known.as_ref(),
                    )
                },
            )?,
            yogas: omen::yogas(
                &settings.panchanga.muhurta_tables,
                day.vara,
                &limbs.nakshatra,
                &limbs.tithi,
            )?,
            varjyam: omen::varjyam(&limbs.nakshatra, settings.panchanga.varjyam)?,
            disha_shool: omen::disha_shool(day.vara),
        };
        memo.known = known.map(|known| (frame, known));
        Ok((
            Panchanga {
                kaalas: period::kaalas(daylight, day.vara),
                choghadiya: period::choghadiya(daylight, night, day.vara),
                horas,
                muhurtas,
                durmuhurtas,
                month,
                moon,
                sun,
                omens,
                limbs,
                window,
                day,
                ghati_reckoning: settings.day.ghati_reckoning,
            },
            frame,
        ))
    }

    /// A day's local day, and its Moon's rises and sets when the range
    /// found them ahead ([`Almanac::ahead`]); a day found alone is
    /// carried from the day before.
    fn local_day_of(
        &self,
        date: &CalendarDate,
        fixed: FixedDay,
        place: &Place,
        memo: &mut Memo,
    ) -> Result<(LocalDay, Option<MoonAhead>), Error> {
        if let Some(ahead) = memo.ahead.take_day(fixed) {
            return Ok((ahead.day?, ahead.moon));
        }
        let day = local_day_carrying(
            self.model,
            self.calendar,
            self.clock,
            place,
            date,
            self.settings().day.polar_day_policy,
            &mut memo.light,
        )?;
        Ok((day, None))
    }

    /// The window the day's limbs are clipped to: the arc under a sunrise
    /// boundary, and the civil day under a midnight one.
    ///
    /// This is the first reader of `day.day_boundary`, declared in Phase
    /// 1 and unread until now. The *periods* are not affected by it: a
    /// choghadiya divides the daylight whatever the window is.
    fn window(&self, day: &LocalDay) -> Result<Interval, Error> {
        use teistro_core::settings::DayBoundary;
        match self.settings().day.day_boundary {
            DayBoundary::Sunset => Interval::new(day.sunset, day.sunset)
                .and_then(|_| Interval::new(day.sunset, next_by(day.sunset, 1.0))),
            DayBoundary::Noon | DayBoundary::Midnight => {
                let midnight = local_midnight(self.clock, self.calendar.fixed_of(&day.date)?)?;
                Interval::new(midnight, next_by(midnight, 1.0))
            }
            _ => Interval::new(day.sunrise, day.next_sunrise),
        }
    }

    /// The night that ends at this day's sunrise, which is the night
    /// Brahma muhurta belongs to.
    ///
    /// The recording engine sizes it from the night that *follows* the
    /// day instead — a median ten seconds out over the corpus, and entry
    /// 17 of the deliberate-difference registry.
    fn previous_night(
        &self,
        place: &Place,
        date: &CalendarDate,
    ) -> Result<Option<Interval>, Error> {
        let fixed = self.calendar.fixed_of(date)?.plus_days(-1);
        let Ok(yesterday) = self.calendar.date_of(fixed) else {
            return Ok(None);
        };
        let Ok(before) = local_day(
            self.model,
            self.calendar,
            self.clock,
            place,
            &yesterday,
            self.settings().day.polar_day_policy,
        ) else {
            return Ok(None);
        };
        Ok(Interval::new(before.sunset, before.next_sunrise).ok())
    }

    /// **One ayanamsha evaluation for a day**, and the two things read
    /// off it: the frame to ask the provider for, and the zodiac the
    /// limbs are measured from.
    ///
    /// The frame is tropical — the shift is applied by the limb kernel
    /// at each instant, so the elongation, the Moon's longitude and
    /// their sum are all measured from one ayanamsha — except where the
    /// provider **defines** the zodiac, when it is the provider's own
    /// sidereal frame and nothing is shifted
    /// ([`ChartZodiac::searched`]). Its centre is `panchanga.centre`,
    /// geocentric by default however the chart is computed, because an
    /// almanac is geocentric everywhere.
    ///
    /// The two used to be separate calls, and a day asked for one or the
    /// other **six times** at the same instant: the frame for the
    /// provider, the zodiac for the limbs, the zodiac again for the
    /// month, and the zodiac once more for each of the Sun's and the
    /// Moon's signs — plus a frame inside the Moon's own day. Every one
    /// of them ran `ChartZodiac::of`, which is a precession evaluation
    /// rather than a field read, and every one of them returned the same
    /// answer because the instant is the same.
    ///
    /// One evaluation, passed down. The instruction-count gate is what
    /// made the waste visible, by refusing an unrelated 8.95% and
    /// sending a reader through this path.
    fn at_instant(
        &self,
        completion: &Completion<'_, P>,
        at: JulianDay<Utc>,
    ) -> Result<(Frame, Zodiac), Error> {
        let searched = ChartZodiac::searched(
            completion,
            self.settings(),
            tt(at, self.delta_t)?,
            (self.precession, self.delta_t),
        )?;
        let frame = Frame {
            centre: match self.settings().panchanga.centre {
                Centre::Topocentric => teistro_port_ephemeris::Centre::Topocentric,
                _ => teistro_port_ephemeris::Centre::Geocentric,
            },
            ..searched.frame
        };
        Ok((frame, searched.zodiac))
    }

    /// The lunar month: the amanta month the new moon's solar sign names,
    /// and the purnimanta one that follows from the tithi the day opens
    /// in.
    ///
    /// Whether the month is intercalary the Indian lunisolar calendar
    /// decides, from the count of sankrantis between the month's two new
    /// moons (`03-design/calendar-indian-lunisolar.md` §2). The **name**
    /// is right either way and needs no case for the intercalary month:
    /// the Sun stands in the same sign at an adhika month's new moon as
    /// at the following nija month's, so `amanta_month` names both.
    ///
    /// The rule runs over **this** almanac's sky rather than the text's,
    /// because the month it marks is the one this almanac named.
    fn month<S: teistro_astro::events::Longitudes + ?Sized>(
        &self,
        longitudes: &S,
        window: Interval,
        limbs: &Limbs,
        (zodiac, frame): (Zodiac, Frame),
        memo: &mut Memo,
    ) -> Result<LunarMonth, Error> {
        let at_sunrise = limbs
            .tithi
            .first()
            .map(|span| span.member)
            .ok_or_else(|| Error::internal("a day with no tithi"))?;
        let from = window.from.get();
        let lunation = match memo.lunation {
            // A later day of the same lunation: the last new moon before
            // its window is the one already found, which the anchored
            // scan refines to the same instant from any window holding
            // it, and the next is found from it alone. So the span, its
            // name and its kind are the ones a search would return.
            Some(known)
                if known.frame == frame
                    && known.span.from.get() < from
                    && from < known.span.to.get() =>
            {
                known
            }
            _ => {
                let span = limb::lunar_month_span(longitudes, window, zodiac)?;
                let found = Lunation {
                    span,
                    frame,
                    amanta: limb::masa_at(longitudes, span.from, zodiac)?,
                    kind: teistro_calendar::lunisolar::kind_of(self.model, span.from, span.to)?,
                };
                memo.lunation = Some(found);
                found
            }
        };
        let (amanta, kind) = (lunation.amanta, lunation.kind);
        Ok(month::of(
            amanta,
            at_sunrise,
            self.settings().calendars.lunar_month,
            kind,
        ))
    }

    /// The Moon's day: its rises and sets in the window the profile
    /// names, and the signs it stood in.
    ///
    /// The frame is **passed in**, not computed: its caller has already
    /// asked for the same one, and a frame is an ayanamsha at an instant
    /// rather than a field read. Recomputing it here was a third
    /// precession evaluation per day that nothing needed.
    fn moon(
        &self,
        completion: &Completion<'_, P>,
        place: &Place,
        (day, window): (&LocalDay, Interval),
        (frame, zodiac, known): (Frame, Zodiac, Option<&RangeCrossings>),
        ahead: Option<MoonAhead>,
    ) -> Result<MoonDay, Error> {
        let search = self.moon_search(day, window)?;
        let (rises, sets) = match ahead {
            Some(found) if found.search == search => (found.rises?, found.sets?),
            _ => {
                let solver = self.moon_solver(completion, place);
                (
                    events(&solver, HorizonEventKind::Rise, search)?,
                    events(&solver, HorizonEventKind::Set, search)?,
                )
            }
        };
        let longitudes = completion.longitudes(frame);
        Ok(MoonDay {
            rises,
            sets,
            signs: limb::signs_known(&longitudes, Body::Moon, window, zodiac, None, known)?,
            window: search,
        })
    }

    /// The window a day's moonrises and moonsets are searched in.
    fn moon_search(&self, day: &LocalDay, window: Interval) -> Result<Interval, Error> {
        Ok(match self.settings().panchanga.moon_events {
            // The engine's reading: the first rise and set at or after
            // the local civil midnight (entry 18).
            MoonEvents::CivilDay => {
                let midnight = local_midnight(self.clock, self.calendar.fixed_of(&day.date)?)?;
                Interval::new(midnight, next_by(midnight, 1.0))?
            }
            _ => window,
        })
    }

    /// The solver a day's moonrises and moonsets are found by.
    fn moon_solver<'c>(&self, completion: &'c Completion<'_, P>, place: &Place) -> Solver<'c> {
        Solver::new(
            completion,
            Body::Moon,
            *place,
            Horizon::from_convention(self.settings().day.sunrise),
            self.delta_t,
        )
    }

    /// What a range's days can find before any of them is computed, found
    /// together: every day's local day, carried from the one before, and
    /// every day's moonrises and moonsets, searched in lockstep so that a
    /// round of every day's searches is one request
    /// ([`teistro_astro::lockstep`]). Each search takes the steps it
    /// takes for its day alone, so each day is what it is alone, to the
    /// bit; a day's failure is kept for that day to report, as it would
    /// have when the range reached it.
    fn ahead(&self, first: FixedDay, count: usize, place: &Place, moon: Moon, memo: &mut Memo) {
        let settings = self.settings();
        // Every day's light and the one after the last, found together:
        // a day reads its own and the next one's.
        let civil: Vec<FixedDay> = (0..=count)
            .map(|step| first.plus_days(i64::try_from(step).unwrap_or(0)))
            .collect();
        let lights = civil_day_lights(self.model, self.clock, place, &civil);
        let mut days = Vec::with_capacity(count);
        for (step, pair) in lights.windows(2).enumerate() {
            let fixed = first.plus_days(i64::try_from(step).unwrap_or(0));
            let day = match pair {
                [Ok(today), Ok(tomorrow)] => self.calendar.date_of(fixed).and_then(|date| {
                    local_day_lit(
                        self.model,
                        self.clock,
                        place,
                        (&date, fixed),
                        settings.day.polar_day_policy,
                        (*today, *tomorrow),
                    )
                }),
                [Err(error), _] | [_, Err(error)] => Err(error.clone()),
                _ => Err(Error::internal("a pair of day lights came apart")),
            };
            days.push((fixed, day));
        }
        let searches: Vec<Option<Interval>> = days
            .iter()
            .map(|(_, day)| {
                let day = day.as_ref().ok().filter(|_| moon == Moon::Searched)?;
                self.window(day)
                    .and_then(|window| self.moon_search(day, window))
                    .ok()
            })
            .collect();
        let completion = Completion::new(self.provider, settings.provider.overrides, self.delta_t);
        let solver = self.moon_solver(&completion, place);
        let round = Lockstep::new();
        let mut found = round
            .run(searches.iter().flatten().flat_map(|search| {
                [HorizonEventKind::Rise, HorizonEventKind::Set]
                    .map(|kind| events_with(&solver, Some(&round), kind, *search))
            }))
            .into_iter();
        memo.ahead = Ahead {
            first,
            days: days
                .into_iter()
                .zip(searches)
                .map(|((_, day), search)| AheadDay {
                    day,
                    moon: search.map(|search| MoonAhead {
                        search,
                        rises: found.next().unwrap_or_else(|| Ok(Vec::new())),
                        sets: found.next().unwrap_or_else(|| Ok(Vec::new())),
                    }),
                })
                .map(Some)
                .collect(),
        };
    }

    /// The day's season under `panchanga.ritu`.
    ///
    /// The solar season is the solar month's, and a month begins on the
    /// civil day `panchanga.solar_month_start` places its sankranti on:
    /// the Sun's sign at sunrise names the next month for a day whose
    /// sankranti falls after dawn, which Nepal's daily panchanga refutes
    /// (`03-design/ritu-measured.md`). The lunar season is the amanta
    /// month's, an adhika month taking its name's; the tropical has no
    /// civil rule to follow and is read as the day opens, the one reading
    /// that searches again, and only when asked.
    fn ritu<S: teistro_astro::events::Longitudes + ?Sized>(
        &self,
        (date, place): (&CalendarDate, &Place),
        sun: &[Span<Rashi>],
        month: &LunarMonth,
        (longitudes, window, zodiac): (&S, Interval, Zodiac),
    ) -> Result<Ritu, Error> {
        let settings = self.settings();
        let opening = |signs: &[Span<Rashi>]| {
            signs
                .first()
                .map_or(Ritu::Vasanta, |span| sky::ritu_of(span.member))
        };
        Ok(match settings.panchanga.ritu {
            RituReckoning::Solar => {
                let fixed = self.calendar.fixed_of(date)?;
                sky::solar_month(
                    settings.panchanga.solar_month_start,
                    fixed,
                    sun,
                    (place, self.clock, self.model),
                )?
                .map_or_else(|| opening(sun), sky::ritu_of)
            }
            RituReckoning::Lunar => month.amanta.attributes().ritu,
            RituReckoning::Tropical => {
                let tropical = Zodiac {
                    ayanamsha: None,
                    ..zodiac
                };
                opening(&Self::signs(longitudes, Body::Sun, window, tropical)?)
            }
            other => {
                return Err(Error::unsupported(format!(
                    "the panchanga does not know the panchanga.ritu value {other:?} yet"
                ))
                .with_field("panchanga.ritu"));
            }
        })
    }

    /// The signs a body stood in over the window.
    fn signs<S: teistro_astro::events::Longitudes + ?Sized>(
        longitudes: &S,
        body: Body,
        window: Interval,
        zodiac: Zodiac,
    ) -> Result<Vec<Span<Rashi>>, Error> {
        limb::signs(longitudes, body, window, zodiac)
    }

    /// The stamp every value carries.
    ///
    /// **The provider is named**, as the chart foundation names it. It
    /// was not until a Rust example printed `provider` off an almanac's
    /// envelope and got an empty string: a stored page said nothing
    /// about what computed it, in all four bindings, because nothing
    /// printed the field.
    ///
    /// `flags_used` is **empty and not a guess**. The chart passes the
    /// completion's steps there, and this path does not have them: its
    /// positions come through `FrameLongitudes`, which keeps no step
    /// list, so there is nothing here to vouch for. An empty list is the
    /// refusal; a plausible one would be a fabrication.
    fn provenance(&self, input: Hash, frame: Frame) -> Provenance {
        let mut provenance = self.resolved.provenance(
            Version::parse(env!("CARGO_PKG_VERSION")).unwrap_or(Version::new(0, 0, 0)),
            input,
        );
        provenance.provider = self
            .provider
            .capabilities()
            .identity
            .stamp(frame, Vec::new());
        provenance.time.delta_t_model = self.delta_t.key().to_string();
        provenance.time.leap_table = teistro_time::leap::version().to_string();
        provenance
    }
}

/// Whether a range's days read the Moon's rises and sets, and so whether
/// [`Almanac::ahead`] searches them.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Moon {
    Searched,
    Unread,
}

/// What one day of a range leaves for the next, so that the next reads it
/// rather than searching for it again. Every field is a value the next
/// day would compute from the same arguments, so a range's days are the
/// days asked for one at a time, to the bit.
#[derive(Default)]
struct Memo {
    /// The last day computed, by its fixed day: the next day's previous
    /// night is its sunset to its next sunrise.
    day: Option<(FixedDay, LocalDay)>,
    /// The lunation the last day fell in.
    lunation: Option<Lunation>,
    /// The next day's light, which the last day read as its tomorrow.
    light: Carried,
    /// The last day of the range, when the days are a range: how far the
    /// crossings found for the first day are worth finding.
    last: Option<FixedDay>,
    /// What the range found for its days before computing them.
    ahead: Ahead,
    /// The limbs' and the signs' crossings found once for the range
    /// ([`RangeCrossings`]), by the frame their source was read in, read
    /// by every day whose window they cover.
    known: Option<(Frame, RangeCrossings)>,
}

/// A range's days as found ahead of computing them ([`Almanac::ahead`]).
struct Ahead {
    first: FixedDay,
    days: Vec<Option<AheadDay>>,
}

impl Default for Ahead {
    fn default() -> Self {
        Ahead {
            first: FixedDay::new(0),
            days: Vec::new(),
        }
    }
}

impl Ahead {
    /// What was found ahead for a day, once.
    fn take_day(&mut self, day: FixedDay) -> Option<AheadDay> {
        let at = usize::try_from(self.first.days_until(day)).ok()?;
        self.days.get_mut(at)?.take()
    }
}

/// One day found ahead: its local day, and its Moon's rises and sets when
/// its window could be had.
struct AheadDay {
    day: Result<LocalDay, Error>,
    moon: Option<MoonAhead>,
}

/// A day's moonrises and moonsets found ahead, in the window searched.
struct MoonAhead {
    search: Interval,
    rises: Result<Vec<JulianDay<Utc>>, Error>,
    sets: Result<Vec<JulianDay<Utc>>, Error>,
}

impl Memo {
    /// The range's crossings for a day, read in the day's frame, found on
    /// the first day that needs them: from this day's window to the range's last day and a day
    /// past it, which a last day's window ends inside. A day outside them,
    /// or under another zodiac, is searched for alone, so a crossing set
    /// that does not cover a day never changes it; a single day asked for
    /// alone has no range and searches as it always did.
    fn crossings<S: teistro_astro::events::Longitudes + ?Sized>(
        &mut self,
        longitudes: &S,
        (zodiac, frame): (Zodiac, Frame),
        window: Interval,
        day: FixedDay,
    ) -> Result<Option<RangeCrossings>, Error> {
        if let Some((read_in, known)) = self.known.take()
            && read_in == frame
            && known.answers(zodiac, window)
        {
            return Ok(Some(known));
        }
        let Some(last) = self.last else {
            return Ok(None);
        };
        let left = day.days_until(last);
        if left < 1 {
            return Ok(None);
        }
        // A whole number of days, a few thousand at most.
        #[allow(
            clippy::cast_precision_loss,
            reason = "a range's length in days is a small whole number"
        )]
        let to = window.from.get() + left as f64 + 2.0;
        let run = Interval::new(window.from, JulianDay::<Utc>::literal(to))?;
        RangeCrossings::over(longitudes, zodiac, run).map(Some)
    }
}

/// A lunar month as the almanac names it: its span between two new
/// moons, the frame they were found in, its amanta name and its kind.
#[derive(Clone, Copy)]
struct Lunation {
    span: Interval,
    frame: Frame,
    amanta: teistro_core::catalogue::Masa,
    kind: teistro_calendar::lunisolar::MonthKind,
}

/// The most lunar years before a range the samvatsara's naming may read:
/// measured, it reads three at most.
const MOST_YEARS_BACK: usize = 12;

/// Every event of a kind inside a window, in order.
///
/// A moonrise and the next are about 24 h 50 m apart, so a 24-hour window
/// holds none, one or two of each; the loop is what says so.
fn events(
    solver: &Solver<'_>,
    kind: HorizonEventKind,
    window: Interval,
) -> Result<Vec<JulianDay<Utc>>, Error> {
    teistro_astro::lockstep::finished(events_with(solver, None, kind, window)).unwrap_or_else(
        || {
            Err(Error::internal(
                "an event search on the spot waited for a round",
            ))
        },
    )
}

/// [`events`], its searches on the spot or in `round`'s requests.
async fn events_with<'a>(
    solver: &Solver<'a>,
    round: Option<&Lockstep<'a>>,
    kind: HorizonEventKind,
    window: Interval,
) -> Result<Vec<JulianDay<Utc>>, Error> {
    let mut found = Vec::new();
    let mut from = window.from.get();
    while from < window.to.get() {
        let remaining = window.to.get() - from;
        let start = JulianDay::<Ut1>::literal(from);
        let event = match round {
            Some(round) => solver.event_in(round, kind, start, remaining).await?,
            None => solver.event(kind, start, remaining)?,
        };
        let Some(event) = event else {
            break;
        };
        let at = event.instant.get();
        if at >= window.to.get() {
            break;
        }
        found.push(JulianDay::literal(at));
        // Past the event by a minute, so the next search cannot find the
        // same crossing again.
        from = at + 1.0 / 1440.0;
    }
    Ok(found)
}

/// An instant so many days on.
fn next_by(instant: JulianDay<Utc>, days: f64) -> JulianDay<Utc> {
    JulianDay::literal(instant.get() + days)
}

/// The terrestrial time of an instant.
fn tt(
    instant: JulianDay<Utc>,
    delta_t: DeltaTModel,
) -> Result<JulianDay<teistro_core::quantity::Tt>, Error> {
    let (tt, _) = teistro_astro::scale::tt_of(JulianDay::<Ut1>::literal(instant.get()), delta_t)?;
    Ok(tt)
}
