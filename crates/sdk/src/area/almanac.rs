//! `sdk.almanac`: a panchanga — the five limbs of a day and its periods,
//! for a day or a run of them.

use serde::Serialize;
use teistro_astro::Completion;
use teistro_astro::eclipse::{Eclipses, EclipsesHere, ShadowRule};
use teistro_astro::precession::PrecessionModel;
use teistro_calendar::CalendarDate;
use teistro_core::envelope::{Convention, Envelope, Hash, content_hash};
use teistro_core::error::Error;
use teistro_core::quantity::Place;
use teistro_core::time::UtcOffset;
#[cfg(feature = "muhurta")]
use teistro_muhurta::Answer;
use teistro_panchanga::almanac::{Almanac, Panchanga};
use teistro_panchanga::festival::{FestivalDay, Observances, ekadashis, following, observances};
use teistro_panchanga::nepal_sambat::NepalSambatDate;
use teistro_panchanga::year::LunarYear;

use teistro_port_ephemeris::{EphemerisProvider, Horizon};
use teistro_time::local_day::local_midnight;

use crate::area::{drik_sun, system_of};
use crate::context::Context;
use crate::ephemeris::no_ephemeris;
use crate::festival_request::FestivalRequest;
#[cfg(feature = "muhurta")]
use crate::muhurta_request::MuhurtaRequest;

/// `sdk.almanac`: what a Nepali or Indian almanac prints — each day's
/// tithi, nakshatra, yoga, karana and vara, and the periods that divide
/// it.
///
/// **A run of days is the shape, and one day is the run of one.** A
/// chart is consulted once; a panchanga every morning. Consecutive days
/// share a boundary — day n's next sunrise is day n+1's sunrise — so a
/// week asked for together costs much less than seven days asked for
/// separately, which is why `of` takes a range.
#[derive(Clone, Copy, Debug)]
pub struct AlmanacArea<'a> {
    context: &'a Context,
}

#[cfg(feature = "muhurta")]
mod muhurta;

impl<'a> AlmanacArea<'a> {
    pub(crate) fn of_context(context: &'a Context) -> AlmanacArea<'a> {
        AlmanacArea { context }
    }

    /// The context this area was read off.
    #[must_use]
    pub fn context(self) -> &'a Context {
        self.context
    }

    /// Every day from one date to another, inclusive, at a place — in
    /// **one crossing**.
    ///
    /// # Errors
    ///
    /// A context with no ephemeris, a calendar the SDK does not ship, a
    /// range the wrong way round, or the first day the almanac refuses.
    pub fn of(
        self,
        from: &CalendarDate,
        to: &CalendarDate,
        place: &Place,
        offset: UtcOffset,
    ) -> Result<Envelope<Vec<Panchanga>>, Error> {
        let days = self.unsealed(from, to, place, offset)?;
        Ok(Envelope::sealing(days.value, days.provenance))
    }

    /// As [`AlmanacArea::of`], with **each day's own content hash** beside
    /// the range's, from the one serialisation that seals the range: what
    /// a caller handing out one day at a time stamps it with, since the
    /// range's provenance hashes the list.
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::of`].
    pub fn of_each(
        self,
        from: &CalendarDate,
        to: &CalendarDate,
        place: &Place,
        offset: UtcOffset,
    ) -> Result<(Envelope<Vec<Panchanga>>, Vec<Hash>), Error> {
        let days = self.unsealed(from, to, place, offset)?;
        Ok(Envelope::sealing_each(days.value, days.provenance))
    }

    /// The days, **not yet sealed**: each public call seals once, over the
    /// value it publishes.
    fn unsealed(
        self,
        from: &CalendarDate,
        to: &CalendarDate,
        place: &Place,
        offset: UtcOffset,
    ) -> Result<Envelope<Vec<Panchanga>>, Error> {
        self.with_almanac(from, offset, |almanac| almanac.between(from, to, place))
    }

    /// Runs `work` over this context's almanac for a range written in
    /// `from`'s calendar, on `offset`'s clock.
    fn with_almanac<T>(
        self,
        from: &CalendarDate,
        offset: UtcOffset,
        work: impl for<'f> FnOnce(&Almanac<'f, dyn EphemerisProvider + 'f>) -> Result<T, Error>,
    ) -> Result<T, Error> {
        let provider = self.context.ephemeris().ok_or_else(no_ephemeris)?;
        let calendar = system_of(from.calendar)?;
        let model = drik_sun(self.context, provider);
        work(&Almanac::new(
            provider,
            self.context.resolved(),
            &model,
            calendar,
            &offset,
            PrecessionModel::default(),
            self.context.delta_t(),
        ))
    }

    /// The days a set of festival rules falls on between two dates,
    /// inclusive, at a place (`03-design/festival-rules.md`): each with
    /// the case between the tithi's two days and the guard that decided.
    ///
    /// A tithi beginning the day before `from` can fall on it, and one on
    /// `to` is judged against the day after (two, when it holds two
    /// sunrises). So the days are founded from the day before `from` to
    /// two after `to`, once, and the provenance says so in its
    /// `applied_conventions`.
    ///
    /// ```no_run
    /// use teistro::{CalendarDate, Context, FestivalPack, FestivalRequest};
    /// use teistro::catalogue::Calendar;
    /// use teistro::quantity::{Altitude, Latitude, Longitude, Place};
    /// use teistro::UtcOffset;
    ///
    /// let sdk = Context::builder().build()?;
    /// let delhi = Place::new(Latitude::try_new(28.6139)?, Longitude::try_new(77.209)?, Altitude::try_new(216.0)?);
    /// let (from, to) = (
    ///     CalendarDate::defined(Calendar::Gregorian, 2026, 1, 1),
    ///     CalendarDate::defined(Calendar::Gregorian, 2026, 12, 31),
    /// );
    /// let found = sdk.almanac().festivals(&from, &to, &delhi, UtcOffset::literal(5, 30, 0),
    ///     &FestivalRequest::from(FestivalPack::Dharmasindhu))?;
    /// for observance in &found.value.observances {
    ///     println!("{} on {} ({:?})", observance.rule, observance.day, observance.case);
    /// }
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A request [`FestivalRequest::check`] refuses, naming its field
    /// under `rules`; a context with no ephemeris; a calendar the SDK does
    /// not ship; or a range the almanac refuses.
    pub fn festivals(
        self,
        from: &CalendarDate,
        to: &CalendarDate,
        place: &Place,
        offset: UtcOffset,
        request: &FestivalRequest,
    ) -> Result<Envelope<Observances>, Error> {
        request.check()?;
        let days = self.unsealed(from, to, place, offset)?;
        self.reckoning(from, to, place, offset, request, &days)
    }

    /// As [`AlmanacArea::festivals`], with the range's days beside the
    /// answer, founded once and sealed as [`AlmanacArea::of_each`] seals
    /// them: what a consumer shows the observances on.
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::festivals`].
    pub fn festivals_with_days(
        self,
        from: &CalendarDate,
        to: &CalendarDate,
        place: &Place,
        offset: UtcOffset,
        request: &FestivalRequest,
    ) -> Result<FestivalDays, Error> {
        let asked = AlmanacRequest::new().with_festivals(request.clone());
        let AlmanacAnswer {
            days,
            day_hashes,
            festivals,
            ..
        } = self.asked(from, to, place, offset, &asked)?;
        let answer = festivals.ok_or_else(|| {
            Error::internal("a request with festivals answered none, which cannot happen")
        })?;
        Ok(FestivalDays {
            days,
            day_hashes,
            answer,
        })
    }

    /// The observances over the range's own `days`, stamped with their
    /// provenance. The widening is founded as runs of its own on each
    /// side, so the range's run is exactly the one [`AlmanacArea::of_each`]
    /// founds and seals, and nothing is founded twice.
    fn reckoning(
        self,
        from: &CalendarDate,
        to: &CalendarDate,
        place: &Place,
        offset: UtcOffset,
        request: &FestivalRequest,
        days: &Envelope<Vec<Panchanga>>,
    ) -> Result<Envelope<Observances>, Error> {
        let calendar = system_of(from.calendar)?;
        let (first, last) = (calendar.fixed_of(from)?, calendar.fixed_of(to)?);
        // Two days each side, which the rules' own pairs need, and as many
        // again as a rule counts from another's day: before, so a day
        // counted into the range finds the observance it counts from;
        // after, so an observance in the range always reaches its count
        // rather than leaving the counted one unjudged.
        let widen = 2 + i64::from(request.reach());
        let (before, after) = (
            calendar.date_of(first.plus_days(-widen))?,
            (
                calendar.date_of(last.plus_days(1))?,
                calendar.date_of(last.plus_days(widen))?,
            ),
        );
        let (earlier, later) = self.with_almanac(from, offset, |almanac| {
            Ok((
                almanac
                    .between(&before, &calendar.date_of(first.plus_days(-1))?, place)?
                    .value,
                almanac.between(&after.0, &after.1, place)?.value,
            ))
        })?;
        let viewed: Vec<FestivalDay> = earlier
            .iter()
            .chain(&days.value)
            .chain(&later)
            .map(FestivalDay::from)
            .collect();
        let karmakala = observances(request.rules(), &viewed)?;
        let counted = following(request.following(), &karmakala.observances, &viewed)?;
        let mut found = karmakala
            .merged(counted)
            .merged(ekadashis(request.ekadashis(), &viewed)?);
        let inside = |date: &CalendarDate| {
            calendar
                .fixed_of(date)
                .is_ok_and(|day| first.days_until(day) >= 0 && day.days_until(last) >= 0)
        };
        found
            .observances
            .retain(|observance| inside(&observance.day));
        found.ekadashis.retain(|fast| inside(&fast.day));
        let (start, end) = (
            days.value.first().map(|day| day.day.sunrise.get()),
            days.value.last().map(|day| day.day.next_sunrise.get()),
        );
        found.unjudged.retain(|unjudged| {
            start.is_some_and(|start| unjudged.tithi.to.get() > start)
                && end.is_some_and(|end| unjudged.tithi.from.get() < end)
        });
        let mut provenance = days.provenance.clone();
        provenance.input_hash = content_hash(&FestivalInput {
            from: from.to_string(),
            to: to.to_string(),
            place: *place,
            utc_offset_seconds: offset.seconds(),
            request,
        });
        provenance.applied_conventions.push(Convention {
            knob: String::from("festival.days"),
            value: format!("{before}..{}", after.1),
            reason: String::from(
                "a tithi beginning the day before the range can fall in it, and an 11th beginning then is pierced or not at an arunodaya in the night before; one at its end is judged against the day after, or two when it holds two sunrises; and a rule counted from another's day reaches as many days as it counts",
            ),
        });
        Ok(Envelope::sealing(found, provenance))
    }

    /// Every day of a range with whatever `request` asks beside it — a
    /// muhurta search, festivals, the lunar years, the eclipses, the
    /// Nepal Sambat dates — over the days **founded
    /// once**, sealed as [`AlmanacArea::of_each`] seals them. What the C
    /// boundary answers a panchanga request with.
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::of_each`], and whatever each thing asked refuses.
    pub fn asked(
        self,
        from: &CalendarDate,
        to: &CalendarDate,
        place: &Place,
        offset: UtcOffset,
        request: &AlmanacRequest,
    ) -> Result<AlmanacAnswer, Error> {
        if let Some(festivals) = &request.festivals {
            festivals.check()?;
        }
        #[cfg(feature = "muhurta")]
        let (muhurta, days) = match &request.muhurta {
            Some(asked) => {
                let (answer, days) = self.searching(from, to, place, offset, asked, |almanac| {
                    almanac.between(from, to, place)
                })?;
                (Some(answer), days)
            }
            None => (None, self.unsealed(from, to, place, offset)?),
        };
        #[cfg(not(feature = "muhurta"))]
        let days = self.unsealed(from, to, place, offset)?;
        let festivals = request
            .festivals
            .as_ref()
            .map(|asked| self.reckoning(from, to, place, offset, asked, &days))
            .transpose()?;
        let years = request
            .years
            .then(|| self.years(from, to, place, offset))
            .transpose()?;
        let eclipses = request
            .eclipses
            .then(|| self.eclipses(from, to, place, offset))
            .transpose()?;
        let nepal_sambat = request.nepal_sambat.then(|| {
            Envelope::sealing(
                days.value.iter().map(Panchanga::nepal_sambat).collect(),
                days.provenance.clone(),
            )
        });
        let (days, day_hashes) = Envelope::sealing_each(days.value, days.provenance);
        Ok(AlmanacAnswer {
            days,
            day_hashes,
            #[cfg(feature = "muhurta")]
            muhurta,
            festivals,
            years,
            eclipses,
            nepal_sambat,
        })
    }

    /// The lunar years a range's days fall in, each with the name the
    /// sixty-year cycle gives it under `calendars.samvatsara` and the
    /// Jovian years that ran in it (`03-design/samvatsara-measured.md`).
    ///
    /// A year runs from the sunrise opening Chaitra Shukla Pratipada to
    /// the next year's, so a day's year is the one holding its sunrise.
    ///
    /// ```no_run
    /// use teistro::{CalendarDate, Context, UtcOffset};
    /// use teistro::catalogue::{Calendar, Samvatsara};
    /// use teistro::quantity::{Altitude, Latitude, Longitude, Place};
    ///
    /// let sdk = Context::builder().profile("nepali-default").build()?;
    /// let kathmandu = Place::new(Latitude::try_new(27.7172)?, Longitude::try_new(85.324)?, Altitude::try_new(1400.0)?);
    /// let day = CalendarDate::defined(Calendar::Gregorian, 2021, 6, 1);
    /// let years = sdk.almanac().years(&day, &day, &kathmandu, UtcOffset::literal(5, 45, 0))?;
    /// // Ananda began and ended inside VS 2077, so 2078 is Rakshasa.
    /// assert_eq!(years.value[0].samvatsara, Samvatsara::Rakshasa);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::of`], and a place whose sunrise the polar policy
    /// leaves undefined on a pratipada.
    pub fn years(
        self,
        from: &CalendarDate,
        to: &CalendarDate,
        place: &Place,
        offset: UtcOffset,
    ) -> Result<Envelope<Vec<LunarYear>>, Error> {
        let years = self.with_almanac(from, offset, |almanac| almanac.years(from, to, place))?;
        Ok(Envelope::sealing(years.value, years.provenance))
    }

    /// Every eclipse whose greatest moment falls in a range of civil
    /// days, from the first day's local midnight to the midnight after
    /// the last, each with how `place` sees it (`03-design/eclipses.md`).
    ///
    /// The shadow is `panchanga.eclipse_shadow`'s, and a body is seen
    /// while it stands above `panchanga.eclipse_horizon`'s horizon, the
    /// eye's by default: the upper limb, with refraction. A lunar eclipse comes back
    /// with its contacts and the Moon's altitude at each; a solar one with
    /// the place's own contacts and magnitude. `seen` on each view is
    /// `None` where the place does not see it, which is what a sutak asks.
    ///
    /// ```no_run
    /// use teistro::{CalendarDate, Context, UtcOffset};
    /// use teistro::catalogue::Calendar;
    /// use teistro::quantity::{Altitude, Latitude, Longitude, Place};
    ///
    /// let sdk = Context::builder().profile("nepali-default").build()?;
    /// let kathmandu = Place::new(Latitude::try_new(27.7172)?, Longitude::try_new(85.324)?, Altitude::try_new(1400.0)?);
    /// let (from, to) = (
    ///     CalendarDate::defined(Calendar::Gregorian, 2025, 1, 1),
    ///     CalendarDate::defined(Calendar::Gregorian, 2025, 12, 31),
    /// );
    /// let year = sdk.almanac().eclipses(&from, &to, &kathmandu, UtcOffset::literal(5, 45, 0))?;
    /// for lunar in &year.value.lunar {
    ///     if let Some(seen) = lunar.here.seen {
    ///         println!("{:?} seen from {:?} to {:?}", lunar.eclipse.kind, seen.from, seen.to);
    ///     }
    /// }
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A context with no ephemeris or a classical one (whose eclipse is
    /// its own method, C188), a calendar the SDK does not ship, a range
    /// the wrong way round, or an instant the ephemeris cannot answer.
    pub fn eclipses(
        self,
        from: &CalendarDate,
        to: &CalendarDate,
        place: &Place,
        offset: UtcOffset,
    ) -> Result<Envelope<EclipsesHere>, Error> {
        let provider = self.context.ephemeris().ok_or_else(no_ephemeris)?;
        let calendar = system_of(from.calendar)?;
        let start = local_midnight(&offset, calendar.fixed_of(from)?)?;
        let end = local_midnight(&offset, calendar.fixed_of(to)?.plus_days(1))?;
        let settings = self.context.settings();
        let delta_t = self.context.delta_t();
        let sky = Completion::new(provider, settings.provider.overrides, delta_t);
        let horizon = Horizon::from_convention(settings.panchanga.eclipse_horizon);
        let found = Eclipses::new(&sky, delta_t)
            .with_shadow(ShadowRule::of_setting(settings.panchanga.eclipse_shadow)?)
            .here_between(start.relabel(), end.relabel(), *place, &horizon)?;
        let mut provenance = self.context.stamped(
            content_hash(&EclipseInput {
                from: from.to_string(),
                to: to.to_string(),
                place: *place,
                utc_offset_seconds: offset.seconds(),
            }),
            sky.apparent_frame(),
            Vec::new(),
        );
        provenance.applied_conventions.push(Convention {
            knob: String::from("eclipse.window"),
            value: format!("JD {} to {}", start.get(), end.get()),
            reason: String::from(
                "an eclipse belongs to the range when its greatest moment falls between the first day's local midnight and the midnight after the last, UT1 read as UTC",
            ),
        });
        Ok(Envelope::sealing(found, provenance))
    }

    /// One day: the run of one, unwrapped.
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::of`].
    pub fn day(
        self,
        date: &CalendarDate,
        place: &Place,
        offset: UtcOffset,
    ) -> Result<Envelope<Panchanga>, Error> {
        let Envelope { value, provenance } = self.unsealed(date, date, place, offset)?;
        let Some(one) = value.into_iter().next() else {
            return Err(Error::internal(
                "a range of one day answered no panchanga, which cannot happen",
            ));
        };
        // Sealed over the day, as `chart().found` is and for the same
        // reason: the hash belongs to the value this envelope holds, and a
        // day is not a range of one — which is never hashed at all.
        Ok(Envelope::sealing(one, provenance))
    }
}

/// What an almanac is asked beside its days
/// ([`AlmanacArea::asked`]): each is optional, and asking for none is
/// [`AlmanacArea::of_each`].
///
/// ```
/// use teistro::{AlmanacRequest, FestivalPack, FestivalRequest, MuhurtaRequest};
/// use teistro::muhurta::ActivityRules;
///
/// let asked = AlmanacRequest::new()
///     .with_muhurta(MuhurtaRequest::new(ActivityRules::raman_marriage()))
///     .with_festivals(FestivalRequest::from(FestivalPack::Dharmasindhu));
/// assert!(asked.muhurta().is_some() && asked.festivals().is_some());
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AlmanacRequest {
    #[cfg(feature = "muhurta")]
    muhurta: Option<MuhurtaRequest>,
    festivals: Option<FestivalRequest>,
    years: bool,
    eclipses: bool,
    nepal_sambat: bool,
}

impl AlmanacRequest {
    /// Nothing beside the days.
    #[must_use]
    pub fn new() -> AlmanacRequest {
        AlmanacRequest::default()
    }

    /// With a muhurta search over the days.
    #[cfg(feature = "muhurta")]
    #[must_use]
    pub fn with_muhurta(mut self, request: MuhurtaRequest) -> AlmanacRequest {
        self.muhurta = Some(request);
        self
    }

    /// With the days festival rules fall on.
    #[must_use]
    pub fn with_festivals(mut self, request: FestivalRequest) -> AlmanacRequest {
        self.festivals = Some(request);
        self
    }

    /// With the lunar years the days fall in ([`AlmanacArea::years`]).
    #[must_use]
    pub fn with_years(mut self) -> AlmanacRequest {
        self.years = true;
        self
    }

    /// With the eclipses of the days and the place's view of each
    /// ([`AlmanacArea::eclipses`]).
    #[must_use]
    pub fn with_eclipses(mut self) -> AlmanacRequest {
        self.eclipses = true;
        self
    }

    /// With each day's Nepal Sambat date, which its lunar month and its
    /// sunrise settle ([`Panchanga::nepal_sambat`],
    /// `03-design/calendar-indian-lunisolar.md` §11).
    #[must_use]
    pub fn with_nepal_sambat(mut self) -> AlmanacRequest {
        self.nepal_sambat = true;
        self
    }

    /// The muhurta search asked, if any.
    #[cfg(feature = "muhurta")]
    #[must_use]
    pub fn muhurta(&self) -> Option<&MuhurtaRequest> {
        self.muhurta.as_ref()
    }

    /// The festival rules asked, if any.
    #[must_use]
    pub fn festivals(&self) -> Option<&FestivalRequest> {
        self.festivals.as_ref()
    }

    /// Whether the lunar years were asked.
    #[must_use]
    pub fn years(&self) -> bool {
        self.years
    }

    /// Whether the eclipses were asked.
    #[must_use]
    pub fn eclipses(&self) -> bool {
        self.eclipses
    }

    /// Whether the days' Nepal Sambat dates were asked.
    #[must_use]
    pub fn nepal_sambat(&self) -> bool {
        self.nepal_sambat
    }
}

/// The days of a range and what was asked beside them: what
/// [`AlmanacArea::asked`] gives.
#[derive(Clone, Debug, PartialEq)]
pub struct AlmanacAnswer {
    /// Every day of the range, sealed as [`AlmanacArea::of_each`] seals it.
    pub days: Envelope<Vec<Panchanga>>,
    /// Each day's own content hash, in the range's order.
    pub day_hashes: Vec<Hash>,
    /// The muhurta search's answer, when one was asked.
    #[cfg(feature = "muhurta")]
    pub muhurta: Option<Envelope<Answer>>,
    /// The observances, when festivals were asked.
    pub festivals: Option<Envelope<Observances>>,
    /// The lunar years the days fall in, when they were asked.
    pub years: Option<Envelope<Vec<LunarYear>>>,
    /// The eclipses of the days with the place's view of each, when they
    /// were asked.
    pub eclipses: Option<Envelope<EclipsesHere>>,
    /// Each day's Nepal Sambat date, in the days' order, when they were
    /// asked: read off the days themselves, so sealed under their
    /// provenance.
    pub nepal_sambat: Option<Envelope<Vec<NepalSambatDate>>>,
}

/// What was asked beside a range's days, each section written as a
/// binding holds it: its catalogue members in full and its envelope
/// sealed over that value, so its content hash is the hash of what a
/// binding reads (`03-design/muhurta-at-the-boundary.md` §4). What
/// [`AlmanacAnswer::sections`] gives; a section not asked is `None`.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlmanacSections {
    /// The muhurta search's answer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub muhurta: Option<Envelope<serde_json::Value>>,
    /// The observances.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub festivals: Option<Envelope<serde_json::Value>>,
    /// The lunar years.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub years: Option<Envelope<serde_json::Value>>,
    /// The eclipses and the place's view of each.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eclipses: Option<Envelope<serde_json::Value>>,
    /// Each day's Nepal Sambat date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nepal_sambat: Option<Envelope<serde_json::Value>>,
}

impl AlmanacAnswer {
    /// The sections answered beside the days, each written in full and
    /// sealed over what is written: the one writer the C boundary's
    /// blob and the agent server's `almanac.days` share.
    ///
    /// # Errors
    ///
    /// `INTERNAL` if a section does not serialise, which a value the
    /// façade built cannot do.
    pub fn sections(&self) -> Result<AlmanacSections, Error> {
        Ok(AlmanacSections {
            #[cfg(feature = "muhurta")]
            muhurta: sealed(self.muhurta.as_ref(), teistro_muhurta::spelling::in_full)?,
            #[cfg(not(feature = "muhurta"))]
            muhurta: None,
            festivals: sealed(self.festivals.as_ref(), Observances::in_full)?,
            years: sealed(self.years.as_ref(), |years| LunarYear::in_full(years))?,
            eclipses: sealed(self.eclipses.as_ref(), EclipsesHere::in_full)?,
            nepal_sambat: sealed(self.nepal_sambat.as_ref(), |dates| {
                NepalSambatDate::in_full(dates)
            })?,
        })
    }
}

/// A section written by `in_full` and sealed over what it wrote, or
/// `None` when it was not asked.
fn sealed<T>(
    answer: Option<&Envelope<T>>,
    in_full: impl FnOnce(&T) -> Result<serde_json::Value, Error>,
) -> Result<Option<Envelope<serde_json::Value>>, Error> {
    answer
        .map(|answer| {
            Ok(Envelope::sealing(
                in_full(&answer.value)?,
                answer.provenance.clone(),
            ))
        })
        .transpose()
}

/// A festival reckoning's answer beside the range's days: what
/// [`AlmanacArea::festivals_with_days`] gives.
#[derive(Clone, Debug, PartialEq)]
pub struct FestivalDays {
    /// Every day of the range, sealed as [`AlmanacArea::of_each`] seals it.
    pub days: Envelope<Vec<Panchanga>>,
    /// Each day's own content hash, in the range's order.
    pub day_hashes: Vec<Hash>,
    /// The observances.
    pub answer: Envelope<Observances>,
}

#[cfg(feature = "muhurta")]
/// A muhurta search's answer beside the days it searched: what
/// [`AlmanacArea::muhurta_with_days`] gives.
#[derive(Clone, Debug, PartialEq)]
pub struct MuhurtaDays {
    /// Every day of the range, sealed as [`AlmanacArea::of_each`] seals it.
    pub days: Envelope<Vec<Panchanga>>,
    /// Each day's own content hash, in the range's order.
    pub day_hashes: Vec<Hash>,
    /// The search's answer.
    pub answer: Envelope<Answer>,
}

/// What a muhurta answer is a function of, beside the settings: its hash
/// is the answer's input hash.
#[cfg(feature = "muhurta")]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MuhurtaInput<'r> {
    from: String,
    to: String,
    place: Place,
    utc_offset_seconds: i32,
    request: &'r MuhurtaRequest,
}

/// What an eclipse answer is a function of, beside the settings.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EclipseInput {
    from: String,
    to: String,
    place: Place,
    utc_offset_seconds: i32,
}

/// What a festival answer is a function of, beside the settings.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FestivalInput<'r> {
    from: String,
    to: String,
    place: Place,
    utc_offset_seconds: i32,
    request: &'r FestivalRequest,
}
