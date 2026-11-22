//! The rashifal through the façade (`03-design/rashifal.md` step 2): the
//! reference day's sunrise from the almanac, the nine places then, and the
//! period's ingresses and stations found exactly.

use teistro_astro::events::Lattice;
use teistro_astro::events::{greatest_rate, least_rate};
use teistro_calendar::{CalendarDate, FixedDay};
use teistro_chart::foundation::{TransitEvent, TransitEventKind};
use teistro_core::catalogue::Graha;
use teistro_core::envelope::{Convention, Envelope};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::time::UtcOffset;
use teistro_gochar::hits::{self, Hit, HitEvent};
use teistro_gochar::{GocharRules, Transit};
use teistro_panchanga::span;
use teistro_port_ephemeris::Body;
use teistro_rashifal::baseline::Panchanga as Limbs;
use teistro_rashifal::{PeriodEvent, RashifalRules, rashifal_holding, reference_day};

use crate::area::{AlmanacArea, ChartArea, system_of};
use crate::hit_request::{motion, station};
use crate::rashifal_request::{
    RashifalAnswer, RashifalBatch, RashifalPeriod, RashifalRequest, Snapshot,
};
use crate::reading::ReadClock;

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
    /// # #[cfg(feature = "builtin-ephemeris")] {
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
    /// let score = read.baseline_score(Rashi::Leo, Period::Monthly, sdk.settings().rashifal.score_bands);
    /// println!("the baseline engine's score: {}, {}", score.overall, score.band());
    /// # }
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
        // A rashifal's sky is a transit's, seen from where
        // `gochar.transit_centre` puts it (crux C778).
        let searched = self.transiting(|founder| {
            prepared
                .iter()
                .map(|one| {
                    let place = *one.request.place();
                    // The snapshot, a minute after it for the motion, and the
                    // period's two ends for which graha keeps its sign.
                    let places = founder.longitudes(
                        &[
                            one.instant,
                            JulianDay::literal(one.instant.get() + MOTION_STEP_DAYS),
                            one.window.0,
                            one.window.1,
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
        // The provenance states the first period's clock at its instant.
        let stated = prepared.first().map(|one| (one.clock.clone(), one.instant));
        let mut provenance = None;
        let mut periods = Vec::with_capacity(prepared.len());
        for (one, (places, events)) in prepared.into_iter().zip(searched) {
            let [now, then, opening, closing] = places.value.as_slice() else {
                return Err(Error::internal(
                    "four instants were placed and the founder answered otherwise",
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
            let days = one.window.1.get() - one.window.0.get();
            let holds = std::array::from_fn(|at| {
                let graha = Graha::ALL.get(at).copied();
                let tracked = graha.is_some_and(|graha| one.request.events().contains(&graha));
                match (graha, opening.get(at), closing.get(at)) {
                    (Some(graha), _, _) if tracked => !events.iter().any(|event| {
                        event.hit.graha == graha
                            && matches!(event.hit.event, HitEvent::SignIngress { .. })
                    }),
                    (Some(graha), Some(&from), Some(&to)) => keeps_its_sign(graha, from, to, days),
                    _ => false,
                }
            });
            let readings = rashifal_holding(&transits, &events, holds, &rules);
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
        let mut provenance =
            provenance.ok_or_else(|| Error::internal("a batch of periods was read in no group"))?;
        if let Some((clock, at)) = stated {
            provenance.time.state_clock(&clock, at);
        }
        Ok(Envelope::sealing(periods, self.transit_stamped(provenance)))
    }

    /// A batch as a binding asks it ([`RashifalBatch`]): each period read
    /// as [`ChartArea::rashifal_many`] reads it, with the baseline engine's
    /// score of each sign beside it when the batch names a period for it,
    /// and each sign's plan when the batch asks for words.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::rashifal_many`].
    pub fn rashifal_answers(
        self,
        batch: &RashifalBatch,
    ) -> Result<Envelope<Vec<RashifalAnswer>>, Error> {
        let Envelope { value, provenance } = self.rashifal_many(&batch.periods)?;
        let interpret = self.context().interpret();
        // Each score banded as the context's settings band it, per call
        // through a derived context (`03-design/rashifal.md`, "The score
        // band").
        let bands = self.context().settings().rashifal.score_bands;
        let answers = value
            .into_iter()
            .map(|period| RashifalAnswer {
                plans: batch.plans.then(|| {
                    period
                        .readings
                        .iter()
                        .map(|reading| interpret.rashifal(reading))
                        .collect()
                }),
                baseline: batch.baseline.map(|asked| {
                    period
                        .readings
                        .iter()
                        .map(|reading| period.baseline_score(reading.rashi, asked, bands))
                        .collect()
                }),
                period,
            })
            .collect();
        Ok(Envelope::sealing(answers, provenance))
    }

    /// What a period needs before the sky is placed: its reference day, the
    /// instant, that day's limbs and the window its events are found in.
    fn prepared(self, request: &RashifalRequest) -> Result<Prepared<'_>, Error> {
        request.check()?;
        let calendar = system_of(request.first().calendar)?;
        let (first, last, days) = request.days()?;
        let middle = first.plus_days(i64::from(reference_day(days)));
        let reference = calendar.date_of(middle)?;
        // A zone is read at each day's own offset (`03-design/time-and-timezone.md` §3.4).
        let clock = ReadClock::of(request.clock(), self.context())?;
        let midnight = |day: FixedDay| local_midnight(day, clock.offset_on(day)?);

        let almanac = AlmanacArea::of_context(self.context())
            .on(&clock)
            .day(&reference, request.place(), clock.offset_on(middle)?)?
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
        let window = (midnight(first)?, midnight(last.plus_days(1))?);
        Ok(Prepared {
            request,
            clock,
            reference,
            instant,
            limbs,
            window,
        })
    }
}

/// A period ready for its sky ([`ChartArea::prepared`]).
struct Prepared<'r> {
    request: &'r RashifalRequest,
    /// The clock its days are read on.
    clock: ReadClock,
    reference: CalendarDate,
    instant: JulianDay<Utc>,
    limbs: Limbs,
    window: (JulianDay<Utc>, JulianDay<Utc>),
}

/// Whether a graha whose ingresses were not searched keeps one sign from
/// `from` to `to` (sidereal degrees) over `days`. A body that never turns
/// (the Sun, the Moon) kept it when both ends share a sign and it could
/// not have gone the whole way round; one that can turn kept it only when
/// even its greatest rate could not have reached either edge of the sign.
/// The nodes are taken as the true node's, which turns, so a period the
/// bound cannot clear says "may change" rather than "holds".
fn keeps_its_sign(graha: Graha, from: f64, to: f64, days: f64) -> bool {
    const SIGN: f64 = 30.0;
    let body = match graha {
        Graha::Sun => Body::Sun,
        Graha::Moon => Body::Moon,
        Graha::Mars => Body::Mars,
        Graha::Mercury => Body::Mercury,
        Graha::Jupiter => Body::Jupiter,
        Graha::Venus => Body::Venus,
        Graha::Saturn => Body::Saturn,
        _ => Body::TrueNode,
    };
    let sign = |at: f64| (at.rem_euclid(360.0) / SIGN).floor();
    let reach = greatest_rate(body) * days;
    if least_rate(body).is_some() {
        sign(from) == sign(to) && reach < 360.0 - SIGN
    } else {
        let into = from.rem_euclid(SIGN);
        reach < into.min(SIGN - into)
    }
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

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn a_steady_body_keeps_its_sign_unless_it_could_have_gone_round() {
        // The Moon over one day, mid-sign at both ends.
        assert!(keeps_its_sign(Graha::Moon, 100.0, 112.0, 1.0));
        // Into the next sign by the day's end.
        assert!(!keeps_its_sign(Graha::Moon, 115.0, 128.0, 1.0));
        // Over a month she comes round to the sign she left.
        assert!(!keeps_its_sign(Graha::Moon, 100.0, 101.0, 29.0));
        // The Sun over a week, mid-sign.
        assert!(keeps_its_sign(Graha::Sun, 212.0, 219.0, 7.0));
    }

    #[test]
    fn a_body_that_turns_keeps_its_sign_only_out_of_reach_of_its_edges() {
        // Mars mid-sign over a week: 6.3 degrees at most, 15 to an edge.
        assert!(keeps_its_sign(Graha::Mars, 135.0, 136.0, 7.0));
        // Near an edge over a month he may have crossed and come back.
        assert!(!keeps_its_sign(Graha::Mars, 121.0, 122.0, 30.0));
        // The nodes are bounded as the true node's.
        assert!(!keeps_its_sign(Graha::Rahu, 300.5, 300.3, 7.0));
        assert!(keeps_its_sign(Graha::Rahu, 315.0, 314.8, 7.0));
    }
}
