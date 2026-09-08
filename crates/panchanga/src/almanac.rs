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
use teistro_astro::ayanamsha::Basis;
use teistro_astro::completion::Completion;
use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::precession::PrecessionModel;
use teistro_astro::rise_set::Solver;
use teistro_calendar::solar::SolarModel;
use teistro_calendar::{CalendarDate, CalendarSystem};
use teistro_chart::zodiac::ChartZodiac;
use teistro_core::catalogue::{Direction, Rashi, Vara};
use teistro_core::envelope::{
    CALCULATION_VERSION, Envelope, Hash, Provenance, Version, content_hash,
};
use teistro_core::error::{Error, Status};
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Place, Ut1, Utc};
use teistro_core::settings::{AyanamshaBasis, Centre, MoonEvents, Resolved, Settings};
use teistro_core::time::LocalClock;
use teistro_port_ephemeris::{Body, EphemerisProvider, Frame, Horizon, HorizonEventKind};
use teistro_time::hora::{self, Hora};
use teistro_time::local_day::{LocalDay, local_day, local_midnight};

use crate::limb::{self, Limbs, Zodiac};
use crate::month::{self, LunarMonth};
use crate::omen::{self, Omens};
use crate::period::{self, Kaalas, Muhurtas, Part};
use crate::sky::{self, MoonDay, SunDay};
use crate::span::{self, Span};

/// The almanac of one day at one place.
///
/// A plain value: no interior mutability, no handle, no lazy field. Two
/// panchangas of the same inputs under the same settings are equal field
/// for field, which is what the determinism contract asks of everything
/// that crosses a binding.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    /// The lunar month, under both conventions.
    pub month: LunarMonth,
    /// What the Moon did.
    pub moon: MoonDay,
    /// What the Sun did.
    pub sun: SunDay,
    /// What the day is said to be.
    pub omens: Omens,
}

impl Panchanga {
    /// The day's vara, which is the arc's and not the civil date's.
    #[must_use]
    pub const fn vara(&self) -> Vara {
        self.day.vara
    }

    /// The direction not to travel in.
    #[must_use]
    pub const fn disha_shool(&self) -> Direction {
        self.omens.disha_shool
    }

    /// The tithi running at an instant.
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
struct Input {
    date: String,
    latitude_deg: f64,
    longitude_deg: f64,
    altitude_m: f64,
}

/// The same, for a range.
#[derive(Serialize, Deserialize)]
struct RangeInput {
    from: String,
    to: String,
    latitude_deg: f64,
    longitude_deg: f64,
    altitude_m: f64,
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
    /// muhurta yoga table the SDK does not ship.
    pub fn day(&self, date: &CalendarDate, place: &Place) -> Result<Envelope<Panchanga>, Error> {
        let value = self.value(date, place)?;
        let provenance = self.provenance(content_hash(&Input {
            date: date.to_string(),
            latitude_deg: place.latitude.get(),
            longitude_deg: place.longitude.get(),
            altitude_m: place.altitude.get(),
        }));
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
        let (first, last) = (self.calendar.fixed_of(from)?, self.calendar.fixed_of(to)?);
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
        let mut values = Vec::with_capacity(count);
        for step in 0..count {
            let offset = i64::try_from(step).unwrap_or(0);
            let date = self.calendar.date_of(first.plus_days(offset))?;
            values.push(self.value(&date, place)?);
        }
        let provenance = self.provenance(content_hash(&RangeInput {
            from: from.to_string(),
            to: to.to_string(),
            latitude_deg: place.latitude.get(),
            longitude_deg: place.longitude.get(),
            altitude_m: place.altitude.get(),
        }));
        Ok(Envelope::new(values, provenance))
    }

    /// One day's value, unstamped.
    fn value(&self, date: &CalendarDate, place: &Place) -> Result<Panchanga, Error> {
        let settings = self.settings();
        // The date is taken through the calendar and back, so that a day
        // asked for by a date a caller wrote and the same day reached
        // from an instant are equal values and not merely the same day:
        // a `CalendarDate` a caller builds carries no era numbers and one
        // the calendar renders does.
        let date = self.calendar.date_of(self.calendar.fixed_of(date)?)?;
        let day = local_day(
            self.model,
            self.calendar,
            self.clock,
            place,
            &date,
            settings.day.polar_day_policy,
        )?;
        let window = self.window(&day)?;
        let daylight = Interval::new(day.sunrise, day.sunset)?;
        let night = Interval::new(day.sunset, day.next_sunrise)?;

        let completion = Completion::new(self.provider, settings.provider.overrides, self.delta_t);
        let frame = self.frame(window.from)?;
        let mut longitudes = completion.longitudes(frame);
        if frame.centre == teistro_port_ephemeris::Centre::Topocentric {
            longitudes = longitudes.with_observer(*place);
        }
        let limbs = limb::limbs(&longitudes, window, self.zodiac(window.from)?)?;

        let horas = hora::horas(&day, settings.day.hora_reckoning.try_into()?)?;
        let previous_night = self.previous_night(place, &date)?;
        let muhurtas = period::muhurtas(daylight, night, previous_night, day.vara);
        let month = self.month(&longitudes, window, &limbs)?;
        let moon = self.moon(&completion, place, &day, window)?;
        let sun = sky::sun_day(self.signs(&longitudes, Body::Sun, window)?);
        let omens = Omens {
            panchaka: omen::panchaka(&limbs.nakshatra),
            yogas: omen::yogas(
                &settings.panchanga.muhurta_tables,
                day.vara,
                &limbs.nakshatra,
                &limbs.tithi,
            )?,
            disha_shool: omen::disha_shool(day.vara),
        };
        Ok(Panchanga {
            kaalas: period::kaalas(daylight, day.vara),
            choghadiya: period::choghadiya(daylight, night, day.vara),
            horas,
            muhurtas,
            month,
            moon,
            sun,
            omens,
            limbs,
            window,
            day,
        })
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

    /// The frame the day's limbs are read in.
    ///
    /// Always tropical: the shift is applied by the limb kernel at each
    /// instant, so the elongation, the Moon's longitude and their sum are
    /// all measured from one ayanamsha. The centre is
    /// `panchanga.centre`, which is geocentric by default however the
    /// chart is computed — an almanac is geocentric everywhere.
    fn frame(&self, at: JulianDay<Utc>) -> Result<Frame, Error> {
        let zodiac = ChartZodiac::of(
            self.settings(),
            tt(at, self.delta_t)?,
            self.precession,
            self.delta_t,
        )?;
        Ok(Frame {
            centre: match self.settings().panchanga.centre {
                Centre::Topocentric => teistro_port_ephemeris::Centre::Topocentric,
                _ => teistro_port_ephemeris::Centre::Geocentric,
            },
            ..zodiac.request
        })
    }

    /// The ayanamsha the limbs are measured from.
    fn zodiac(&self, at: JulianDay<Utc>) -> Result<Zodiac, Error> {
        let chart = ChartZodiac::of(
            self.settings(),
            tt(at, self.delta_t)?,
            self.precession,
            self.delta_t,
        )?;
        Ok(Zodiac {
            ayanamsha: chart.ayanamsha,
            basis: if self.settings().frame.ayanamsha_basis == AyanamshaBasis::True {
                Basis::True
            } else {
                Basis::Mean
            },
            precession: self.precession,
            delta_t: self.delta_t,
        })
    }

    /// The lunar month: the amanta month the new moon's solar sign names,
    /// and the purnimanta one that follows from the tithi the day opens
    /// in.
    ///
    /// Whether the month is intercalary is the Indian lunisolar
    /// calendar's to decide; the name is right either way.
    fn month<S: teistro_astro::events::Longitudes + ?Sized>(
        &self,
        longitudes: &S,
        window: Interval,
        limbs: &Limbs,
    ) -> Result<LunarMonth, Error> {
        let at_sunrise = limbs
            .tithi
            .first()
            .map(|span| span.member)
            .ok_or_else(|| Error::internal("a day with no tithi"))?;
        let amanta = limb::amanta_month(longitudes, window, self.zodiac(window.from)?)?;
        Ok(month::of(
            amanta,
            at_sunrise,
            self.settings().calendars.lunar_month,
        ))
    }

    /// The Moon's day: its rises and sets in the window the profile
    /// names, and the signs it stood in.
    fn moon(
        &self,
        completion: &Completion<'_, P>,
        place: &Place,
        day: &LocalDay,
        window: Interval,
    ) -> Result<MoonDay, Error> {
        let search = match self.settings().panchanga.moon_events {
            // The engine's reading: the first rise and set at or after
            // the local civil midnight (entry 18).
            MoonEvents::CivilDay => {
                let midnight = local_midnight(self.clock, self.calendar.fixed_of(&day.date)?)?;
                Interval::new(midnight, next_by(midnight, 1.0))?
            }
            _ => window,
        };
        let solver = Solver::new(
            completion,
            Body::Moon,
            *place,
            Horizon::from_convention(self.settings().day.sunrise),
            self.delta_t,
        );
        let rises = events(&solver, HorizonEventKind::Rise, search)?;
        let sets = events(&solver, HorizonEventKind::Set, search)?;
        let longitudes = completion.longitudes(self.frame(window.from)?);
        Ok(MoonDay {
            rises,
            sets,
            signs: self.signs(&longitudes, Body::Moon, window)?,
            window: search,
        })
    }

    /// The signs a body stood in over the window.
    fn signs<S: teistro_astro::events::Longitudes + ?Sized>(
        &self,
        longitudes: &S,
        body: Body,
        window: Interval,
    ) -> Result<Vec<Span<Rashi>>, Error> {
        limb::signs(longitudes, body, window, self.zodiac(window.from)?)
    }

    /// The stamp every value carries.
    fn provenance(&self, input: Hash) -> Provenance {
        let mut provenance = Provenance::new(
            Version::parse(env!("CARGO_PKG_VERSION")).unwrap_or(Version::new(0, 0, 0)),
            CALCULATION_VERSION,
            teistro_core::catalogue::SCHEMA_VERSION,
            self.resolved.profile.as_str(),
            self.settings().hash(),
            input,
        );
        provenance.time.delta_t_model = self.delta_t.key().to_string();
        provenance.time.leap_table = teistro_time::leap::version().to_string();
        provenance
    }
}

/// Every event of a kind inside a window, in order.
///
/// A moonrise and the next are about 24 h 50 m apart, so a 24-hour window
/// holds none, one or two of each; the loop is what says so.
fn events(
    solver: &Solver<'_>,
    kind: HorizonEventKind,
    window: Interval,
) -> Result<Vec<JulianDay<Utc>>, Error> {
    let mut found = Vec::new();
    let mut from = window.from.get();
    while from < window.to.get() {
        let remaining = window.to.get() - from;
        let Some(event) = solver.event(kind, JulianDay::<Ut1>::literal(from), remaining)? else {
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
