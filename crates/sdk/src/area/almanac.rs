//! `sdk.almanac`: a panchanga — the five limbs of a day and its periods,
//! for a day or a run of them.

use serde::Serialize;
use teistro_astro::Completion;
use teistro_astro::precession::PrecessionModel;
use teistro_astro::visibility::Heliacal;
use teistro_calendar::{CalendarDate, CalendarSystem};
use teistro_chart::foundation::Founder;
use teistro_core::envelope::{Convention, Envelope, Hash, canonical_json, content_hash};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_core::time::UtcOffset;
use teistro_muhurta::sources::Over;
use teistro_muhurta::{Answer, ProviderSources, search};
use teistro_panchanga::almanac::{Almanac, Panchanga};
use teistro_port_ephemeris::{EphemerisProvider, Horizon};
use teistro_time::local_day::local_midnight;

use crate::area::{drik_sun, system_of};
use crate::context::Context;
use crate::ephemeris::no_ephemeris;
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

    /// The windows an activity's rules leave between two dates, inclusive,
    /// at a place: each judged clause by clause and ranked, and the days
    /// the season closed whole, named by what closed them
    /// (`03-design/muhurta.md`).
    ///
    /// Only the days the season leaves open are founded, and only the
    /// best of them are cut into windows. The provenance names the two
    /// choices the search applies beside the settings — how asta is seen
    /// and the instant the chart zodiac is taken at — in its
    /// `applied_conventions`.
    ///
    /// ```no_run
    /// use teistro::{CalendarDate, Context, MuhurtaRequest};
    /// use teistro::catalogue::Calendar;
    /// use teistro::muhurta::ActivityRules;
    /// use teistro::quantity::{Altitude, Latitude, Longitude, Place};
    /// use teistro::UtcOffset;
    ///
    /// let sdk = Context::builder().build()?;
    /// let kathmandu = Place::new(Latitude::try_new(27.7172)?, Longitude::try_new(85.324)?, Altitude::try_new(1400.0)?);
    /// let (from, to) = (
    ///     CalendarDate::defined(Calendar::Gregorian, 2026, 11, 20),
    ///     CalendarDate::defined(Calendar::Gregorian, 2026, 12, 15),
    /// );
    /// let found = sdk.almanac().muhurta(&from, &to, &kathmandu, UtcOffset::literal(5, 45, 0),
    ///     &MuhurtaRequest::new(ActivityRules::raman_marriage()))?;
    /// for window in &found.value.windows {
    ///     println!("{:?}: {} clauses, barred by {:?}", window.at, window.clauses.len(), window.barred_by);
    /// }
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A request [`MuhurtaRequest::check`] refuses; a context with no
    /// ephemeris; a calendar the SDK does not ship; a range the wrong way
    /// round or longer than the almanac's; or whatever the search refuses.
    pub fn muhurta(
        self,
        from: &CalendarDate,
        to: &CalendarDate,
        place: &Place,
        offset: UtcOffset,
        request: &MuhurtaRequest,
    ) -> Result<Envelope<Answer>, Error> {
        // The first day stamps the answer and is one the search would found
        // anyway, so it is handed on rather than founded twice.
        let (answer, _) = self.searching(from, to, place, offset, request, |almanac| {
            let first = almanac.day(from, place)?;
            Ok(Envelope::new(vec![first.value], first.provenance))
        })?;
        Ok(answer)
    }

    /// As [`AlmanacArea::muhurta`], with **every day of the range** beside
    /// the answer, as [`AlmanacArea::of_each`] gives them: the almanac a
    /// consumer electing a time shows beside its windows. The range is
    /// founded once, and the search reads its days from it rather than
    /// founding them again.
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::muhurta`].
    pub fn muhurta_with_days(
        self,
        from: &CalendarDate,
        to: &CalendarDate,
        place: &Place,
        offset: UtcOffset,
        request: &MuhurtaRequest,
    ) -> Result<MuhurtaDays, Error> {
        let (answer, days) = self.searching(from, to, place, offset, request, |almanac| {
            almanac.between(from, to, place)
        })?;
        let (days, day_hashes) = Envelope::sealing_each(days.value, days.provenance);
        Ok(MuhurtaDays {
            days,
            day_hashes,
            answer,
        })
    }

    /// The search over the days `found` founds, stamped with their
    /// provenance, its input and what it applied.
    fn searching(
        self,
        from: &CalendarDate,
        to: &CalendarDate,
        place: &Place,
        offset: UtcOffset,
        request: &MuhurtaRequest,
        found: impl for<'f> FnOnce(
            &Almanac<'f, dyn EphemerisProvider + 'f>,
        ) -> Result<Envelope<Vec<Panchanga>>, Error>,
    ) -> Result<(Envelope<Answer>, Envelope<Vec<Panchanga>>), Error> {
        request.check()?;
        let provider = self.context.ephemeris().ok_or_else(no_ephemeris)?;
        let resolved = self.context.resolved();
        let settings = &resolved.settings;
        let (precession, delta_t) = (PrecessionModel::default(), self.context.delta_t());
        let calendar = system_of(from.calendar)?;
        let zodiac_at = middle(calendar, offset, from, to)?;
        let model = drik_sun(self.context, provider);
        let almanac = Almanac::new(
            provider, resolved, &model, calendar, &offset, precession, delta_t,
        );
        let founder = Founder::new(
            provider, resolved, &model, calendar, &offset, precession, delta_t,
        );
        let completion = Completion::new(provider, settings.provider.overrides, delta_t);
        let heliacal = Heliacal::new(
            &completion,
            *place,
            *request.asta(),
            Horizon::from_convention(settings.day.sunrise),
            delta_t,
        );
        let over = Over {
            almanac: &almanac,
            founder: &founder,
            completion: &completion,
            heliacal: &heliacal,
            calendar,
            clock: &offset,
            settings,
            precession,
            delta_t,
        };
        let days = found(&almanac)?;
        let sources = ProviderSources::new(&over, *place, zodiac_at)?.with_days(&days.value);
        let answer = search(&sources, &request.over(from, to))?;
        let mut provenance = days.provenance.clone();
        provenance.input_hash = content_hash(&MuhurtaInput {
            from: from.to_string(),
            to: to.to_string(),
            place: *place,
            utc_offset_seconds: offset.seconds(),
            request,
        });
        provenance.applied_conventions.extend([
            Convention {
                knob: String::from("muhurta.asta"),
                value: canonical_json(request.asta()),
                reason: String::from(
                    "how Guru and Shukra are seen, which decides the weeks their asta closes",
                ),
            },
            Convention {
                knob: String::from("muhurta.zodiacAt"),
                value: zodiac_at.get().to_string(),
                reason: String::from(
                    "one chart zodiac serves the search, taken at the range's middle so that no day's is more than half the range's drift away",
                ),
            },
        ]);
        Ok((Envelope::sealing(answer, provenance), days))
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
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MuhurtaInput<'r> {
    from: String,
    to: String,
    place: Place,
    utc_offset_seconds: i32,
    request: &'r MuhurtaRequest,
}

/// The middle of a range of civil days, from the first's local midnight
/// to the midnight after the last.
fn middle(
    calendar: &dyn CalendarSystem,
    clock: UtcOffset,
    from: &CalendarDate,
    to: &CalendarDate,
) -> Result<JulianDay<Utc>, Error> {
    let start = local_midnight(&clock, calendar.fixed_of(from)?)?;
    let end = local_midnight(&clock, calendar.fixed_of(to)?.plus_days(1))?;
    Ok(JulianDay::try_new(f64::midpoint(start.get(), end.get()))?)
}
