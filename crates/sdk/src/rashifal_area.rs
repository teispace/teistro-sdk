//! The rashifal through the façade (`03-design/rashifal.md` step 2): the
//! reference day's sunrise from the almanac, the nine places then, and the
//! period's ingresses and stations found exactly.

use teistro_astro::events::Lattice;
use teistro_calendar::{CalendarDate, FixedDay};
use teistro_chart::foundation::{TransitEvent, TransitEventKind};
use teistro_core::envelope::{Convention, Envelope};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::time::UtcOffset;
use teistro_gochar::hits::{self, Hit, HitEvent};
use teistro_gochar::{GocharRules, Transit};
use teistro_panchanga::span;
use teistro_rashifal::baseline::Panchanga as Limbs;
use teistro_rashifal::{PeriodEvent, RashifalRules, rashifal, reference_day};

use crate::area::{AlmanacArea, ChartArea, system_of};
use crate::hit_request::{motion, station};
use crate::rashifal_request::{RashifalPeriod, RashifalRequest, Snapshot};

/// How far apart the two places that tell a graha's motion are, in days:
/// a minute, well inside the shortest stationary spell of any graha.
const MOTION_STEP_DAYS: f64 = 1.0 / 1440.0;

impl ChartArea<'_> {
    /// One period read for each of the twelve signs, each taken as a
    /// reader's janma rāśi: Phaladeepika ch. 26's gochar from it at the
    /// reference day's sunrise (C358, C359), Saturn's standing from it,
    /// and every ingress and station of the period counted from it (C360).
    ///
    /// The period runs from local midnight of its first day to local
    /// midnight after its last, so an ingress on the last evening counts;
    /// the sky is read on the middle day, the earlier of two.
    ///
    /// ```no_run
    /// # use teistro::CalendarDate;
    /// # use teistro::catalogue::{Calendar, Rashi};
    /// # use teistro::quantity::{Altitude, Latitude, Longitude, Place};
    /// # use teistro::rashifal::baseline::Period;
    /// # use teistro::{Context, Ephemeris, RashifalRequest, UtcOffset};
    /// let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// let kathmandu = Place::new(
    ///     Latitude::literal(27.7172),
    ///     Longitude::literal(85.324),
    ///     Altitude::literal(1400.0),
    /// );
    /// let october = RashifalRequest::between(
    ///     CalendarDate::defined(Calendar::Gregorian, 2026, 10, 1),
    ///     CalendarDate::defined(Calendar::Gregorian, 2026, 10, 31),
    ///     kathmandu,
    ///     UtcOffset::literal(5, 45, 0),
    /// );
    /// let read = sdk.chart().rashifal(&october)?.value;
    /// assert_eq!(read.reference.day, 16);
    /// let leo = read.of(Rashi::Leo);
    /// println!("Saturn in the {} from Leo", leo.saturn.house);
    /// let score = read.baseline_score(Rashi::Leo, Period::Monthly);
    /// println!("the baseline engine's score: {}", score.overall);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// What [`RashifalRequest::check`] refuses; a last day before the
    /// first, named `last`; a calendar the SDK does not ship; whatever the
    /// almanac or the ephemeris refuses.
    pub fn rashifal(self, request: &RashifalRequest) -> Result<Envelope<RashifalPeriod>, Error> {
        let Envelope { value, provenance } = self.rashifal_many(std::slice::from_ref(request))?;
        let Some(one) = value.into_iter().next() else {
            return Err(Error::internal("a batch of one period read none"));
        };
        Ok(Envelope::sealing(one, provenance))
    }

    /// Many periods, each read as [`ChartArea::rashifal`] reads it alone,
    /// under one founder: a year of daily readings sets the ephemeris up
    /// once, not once a day.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::rashifal`], for the first period refused; no
    /// period at all is `INVALID_ARG` naming `requests`.
    pub fn rashifal_many(
        self,
        requests: &[RashifalRequest],
    ) -> Result<Envelope<Vec<RashifalPeriod>>, Error> {
        if requests.is_empty() {
            return Err(Error::invalid_arg("no period to read").with_field("requests"));
        }
        let prepared = requests
            .iter()
            .map(|request| self.prepared(request))
            .collect::<Result<Vec<_>, Error>>()?;
        let searched = self.founding(UtcOffset::UTC, |founder| {
            prepared
                .iter()
                .map(|one| {
                    let place = *one.request.place();
                    let places = founder.longitudes(
                        &[
                            one.instant,
                            JulianDay::literal(one.instant.get() + MOTION_STEP_DAYS),
                        ],
                        &place,
                    )?;
                    let events = founder.transit_events(
                        &place,
                        one.request.events(),
                        &[Lattice::SIGNS],
                        true,
                        one.window,
                    )?;
                    Ok((places, events.value))
                })
                .collect::<Result<Vec<_>, Error>>()
        })?;
        let gochar = GocharRules::of(self.context().settings());
        let mut provenance = None;
        let mut periods = Vec::with_capacity(prepared.len());
        for (one, (places, events)) in prepared.into_iter().zip(searched) {
            let [now, then] = places.value.as_slice() else {
                return Err(Error::internal(
                    "two instants were placed and the founder answered otherwise",
                ));
            };
            let transits = now.map(Transit::at_longitude);
            let retrograde = std::array::from_fn(|graha| match (now.get(graha), then.get(graha)) {
                // The shorter way round from one place to the next.
                (Some(at), Some(next)) => (next - at + 540.0).rem_euclid(360.0) < 180.0,
                _ => false,
            });
            let rules = RashifalRules {
                gochar,
                spells: one.request.spells().to_vec(),
            };
            let events: Vec<PeriodEvent> = events.iter().map(period_event).collect();
            let readings = rashifal(&transits, &events, &rules);
            let stamp = provenance.get_or_insert(places.provenance);
            let snapshot = snapshot_convention(one.request.snapshot());
            if !stamp.applied_conventions.contains(&snapshot) {
                stamp.applied_conventions.push(snapshot);
            }
            periods.push(RashifalPeriod {
                first: one.request.first().clone(),
                last: one.request.last().clone(),
                reference: one.reference,
                instant: one.instant,
                transits,
                retrograde,
                panchanga: one.limbs,
                readings,
            });
        }
        let provenance =
            provenance.ok_or_else(|| Error::internal("a batch of periods was read in no group"))?;
        Ok(Envelope::sealing(periods, provenance))
    }

    /// What a period needs before the sky is placed: its reference day, the
    /// instant, that day's limbs and the window its events are found in.
    fn prepared(self, request: &RashifalRequest) -> Result<Prepared<'_>, Error> {
        request.check()?;
        let calendar = system_of(request.first().calendar)?;
        let first = calendar.fixed_of(request.first())?;
        let last = calendar.fixed_of(request.last())?;
        let days = u32::try_from(first.days_until(last) + 1)
            .ok()
            .filter(|days| *days > 0)
            .ok_or_else(|| {
                Error::invalid_arg("a period's last day comes before its first").with_field("last")
            })?;
        let middle = first.plus_days(i64::from(reference_day(days)));
        let reference = calendar.date_of(middle)?;
        let midnight = |day: FixedDay| local_midnight(day, request.offset());

        let almanac = AlmanacArea::of_context(self.context())
            .day(&reference, request.place(), request.offset())?
            .value;
        let sunrise = almanac.day.sunrise;
        let instant = match request.snapshot() {
            Snapshot::Sunrise => sunrise,
            Snapshot::Clock { hour, minute } => JulianDay::literal(
                midnight(middle)?.get() + (f64::from(hour) * 60.0 + f64::from(minute)) / 1440.0,
            ),
        };
        let unnamed = |limb: &str| {
            Error::internal(format!("the almanac's day names no {limb} at its sunrise"))
        };
        let limbs = Limbs {
            tithi: span::at(&almanac.limbs.tithi, sunrise)
                .ok_or_else(|| unnamed("tithi"))?
                .member,
            yoga: span::at(&almanac.limbs.yoga, sunrise)
                .ok_or_else(|| unnamed("yoga"))?
                .member,
            muhurta_yogas: u8::try_from(almanac.omens.yogas.len()).unwrap_or(u8::MAX),
        };
        Ok(Prepared {
            request,
            reference,
            instant,
            limbs,
            window: (midnight(first)?, midnight(last.plus_days(1))?),
        })
    }
}

/// A period ready for its sky ([`ChartArea::prepared`]).
struct Prepared<'r> {
    request: &'r RashifalRequest,
    reference: CalendarDate,
    instant: JulianDay<Utc>,
    limbs: Limbs,
    window: (JulianDay<Utc>, JulianDay<Utc>),
}

/// The convention a snapshot is stamped with.
fn snapshot_convention(snapshot: Snapshot) -> Convention {
    Convention {
        knob: String::from("rashifal.snapshot"),
        value: match snapshot {
            Snapshot::Sunrise => String::from("SUNRISE"),
            Snapshot::Clock { hour, minute } => format!("{hour:02}:{minute:02}"),
        },
        reason: String::from(
            "a period is read at one instant of its middle day, sunrise unless asked otherwise (C358, C359)",
        ),
    }
}

/// Midnight starting `day` at a civil offset, in UTC.
fn local_midnight(day: FixedDay, offset: UtcOffset) -> Result<JulianDay<Utc>, Error> {
    let jd = day
        .jd_at_midnight()
        .map_err(|invalid| Error::invalid_arg(invalid.to_string()).with_field("first"))?;
    Ok(JulianDay::literal(
        jd.get() - f64::from(offset.seconds()) / 86_400.0,
    ))
}

/// A searched event as the period counts it: an ingress in the sign it
/// enters, a station in the sign it stands in.
fn period_event(event: &TransitEvent) -> PeriodEvent {
    let (happened, sign) = match event.kind {
        TransitEventKind::Crossing {
            boundary_deg,
            direction,
            ..
        } => {
            let ingress = hits::sign_ingress(boundary_deg, motion(direction));
            let into = match ingress {
                HitEvent::SignIngress { into, .. } => into,
                _ => Transit::at_longitude(boundary_deg).sign,
            };
            (ingress, into)
        }
        TransitEventKind::Station {
            kind,
            longitude_deg,
        } => (station(kind), Transit::at_longitude(longitude_deg).sign),
    };
    PeriodEvent {
        hit: Hit {
            instant: event.instant,
            graha: event.graha,
            event: happened,
        },
        sign,
    }
}
