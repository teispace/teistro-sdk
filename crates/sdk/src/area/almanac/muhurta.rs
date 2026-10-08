//! The muhurta search over an almanac's days (`03-design/muhurta.md`): the
//! windows an activity's rules leave, and the days beside them.

use teistro_astro::Completion;
use teistro_astro::precession::PrecessionModel;
use teistro_astro::visibility::Heliacal;
use teistro_calendar::{CalendarDate, CalendarSystem};
use teistro_chart::foundation::Founder;
use teistro_core::envelope::{Convention, Envelope, canonical_json, content_hash};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_core::time::UtcOffset;
use teistro_muhurta::sources::Over;
use teistro_muhurta::{Answer, ProviderSources, search};
use teistro_panchanga::almanac::{Almanac, Panchanga};
use teistro_port_ephemeris::{EphemerisProvider, Horizon};
use teistro_time::local_day::local_midnight;

use super::{AlmanacAnswer, AlmanacArea, AlmanacRequest, MuhurtaDays, MuhurtaInput};
use crate::area::{drik_sun, system_of};
use crate::ephemeris::no_ephemeris;
use crate::muhurta_request::MuhurtaRequest;

impl AlmanacArea<'_> {
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
        let asked = AlmanacRequest::new().with_muhurta(request.clone());
        let AlmanacAnswer {
            days,
            day_hashes,
            muhurta,
            ..
        } = self.asked(from, to, place, offset, &asked)?;
        let answer = muhurta.ok_or_else(|| {
            Error::internal("a request with a muhurta search answered none, which cannot happen")
        })?;
        Ok(MuhurtaDays {
            days,
            day_hashes,
            answer,
        })
    }

    /// The search over the days `found` founds, stamped with their
    /// provenance, its input and what it applied.
    pub(super) fn searching(
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
