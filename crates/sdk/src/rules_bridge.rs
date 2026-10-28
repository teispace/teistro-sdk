//! The rules kernel, reachable from a chart the SDK computed.
//!
//! `teistro-rules` evaluates rules over a [`RuleChart`]: each body where it
//! stands, with its dignity, its motion, whether the Sun burns it, and its
//! navamsha. Everything in that list the SDK already computes — the
//! foundation gives the longitudes and the lagna, the state gives the dignity
//! and the combustion, `teistro-vargas` gives the navamsha — but nothing put
//! them together, so 800-odd shipped rules were reachable only from inside the
//! rules crate. [`rule_chart`] is that join.
//!
//! It computes the chara karakas from the longitudes, as BPHS ch. 32 does,
//! with the eight-karaka scheme in the verse's order; a consumer who wants the
//! recording engine's order recomputes with
//! [`RuleChart::with_chara_karakas`]. [`rule_vargas`] gives the divisional
//! charts an `in-varga` condition reads. A chart's panchanga and strengths are
//! passed in because they are separate readings with their own settings.
//! [`rule_periods`] hands the rules the periods of a dasha running at an
//! instant, so a result can say whether they deliver it.
//!
//! [`RuleInputs`] does all of that from one chart reading: ask for the
//! sections with [`ChartRequest::with_rule_inputs`](crate::ChartRequest::with_rule_inputs),
//! read the chart, and take an evaluator from what came back.

use teistro_chart::foundation::ChartFoundation;
use teistro_chart::moment::{BirthEclipse, Moment, RisingSign};
use teistro_core::angle::Nas;
use teistro_core::catalogue::{Dignity, Graha, Rashi, Varga};
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::Degrees;
use teistro_core::quantity::{Depth, JulianDay, Utc};
use teistro_core::settings::{EightKarakaOrder, Settings};
use teistro_dasha::{Chain, Period, Timeline};
use teistro_rules::longevity::{Vulnerability, age_span};
use teistro_rules::{
    Body, Eclipse, EightKarakas, Evaluator, House, LifeClass, Panchanga, Placement, PointAt,
    Readings, RuleChart, Running, StrengthMeasure, Strengths, Tables, VargaSigns,
};
use teistro_serial::Document;
use teistro_state::GrahaState;
use teistro_strength::ShadbalaReading;
use teistro_time::{DayState, LocalDay};
use teistro_vargas::{Scheme, sign};

/// Ghatikas in a day: sixty, which is what a limb's elapsed and remaining
/// are counted in (BPHS ch. 92 measures a gandanta this way).
const GHATIKAS_PER_DAY: f64 = 60.0;
/// Hours in a day.
const HOURS_PER_DAY: f64 = 24.0;

/// The limbs **at the birth**, from the almanac of the birth's day.
///
/// The document's panchanga is the day's almanac — every limb that touches
/// the day, as spans — and a rule asks about the one limb that was running.
/// The two are a query apart, and for a while nothing made it: `RuleChart`
/// carried `panchanga: None` from every document, so the **11 shipped
/// rules that read a limb could never hold**, six arishtas, all four
/// gandantas and one computed dosha. They said so rather than failing
/// silently — "`DAGDHA_RASHI` needs a tithi, and the chart has none" — but a
/// consumer had no way to give them one, which is the dead end the
/// maintainer's rule forbids.
///
/// `None` when the almanac carries no tithi at the instant, which is the
/// one limb every panchanga-reading rule needs.
///
/// The rising sign's span and the eclipse are the instant's rather than the
/// day's, and come from the document's `moment` when it carries one; without
/// it the lagna's edge and an eclipse at birth are unknown, and a rule that
/// reads either does not hold. They were unknown everywhere until 2026-10-12,
/// when a sky of 6000 charts found `LAGNA_GANDANTA` and
/// `BJ_ECLIPSED_MOON_IN_LAGNA_MARS_EIGHTH` never holding
/// (`03-design/rule-cofiring-measured.md`).
fn birth_limbs(
    day: &teistro_panchanga::almanac::Panchanga,
    instant: JulianDay<Utc>,
    pada: teistro_rules::Pada,
    moment: Option<&Moment>,
) -> Option<Panchanga> {
    let tithi = day.tithi_at(instant)?;
    let nakshatra = day.nakshatra_at(instant)?;
    Some(Panchanga {
        tithi: tithi.member,
        vara: day.vara(),
        nakshatra: nakshatra.member,
        // The pada a rule reads is the **Moon's**, which is its
        // longitude's and not the nakshatra span's, so the caller works it
        // out rather than this function guessing from what it has.
        pada,
        yoga: day.yoga_at(instant).map(|span| span.member)?,
        karana: day.karana_at(instant).map(|span| span.member)?,
        spans: teistro_rules::Spans {
            tithi: ghatikas(tithi.whole, instant),
            nakshatra: ghatikas(nakshatra.whole, instant),
            lagna: moment.and_then(|moment| lagna_span(&moment.rising, instant)),
            night: night_span(&day.day, instant),
        },
        by_day: Some(
            instant.get() >= day.day.sunrise.get() && instant.get() < day.day.sunset.get(),
        ),
        // How near is near is the rule's window (`birth-on-sankranti`'s
        // `windowHours`), so the chart carries the distance and not a verdict.
        sankranti_hours: day
            .sun
            .sankranti
            .map(|at| (instant.get() - at.get()) * HOURS_PER_DAY),
        tithi_kshaya: Some(day.sunrises(tithi).is_kshaya()),
        eclipse: moment.and_then(|moment| match moment.eclipse {
            BirthEclipse::Lunar { .. } => Some(Eclipse::Lunar),
            BirthEclipse::Solar { .. } => Some(Eclipse::Solar),
            BirthEclipse::None | BirthEclipse::Unjudged => None,
        }),
    })
}

/// How much of the rising sign had risen at an instant and how much was
/// left, in ghatikas; an edge the search did not reach, which only a place
/// inside a polar circle has, is infinitely far, so the instant is never at
/// it.
fn lagna_span(rising: &RisingSign, instant: JulianDay<Utc>) -> Option<teistro_rules::Span> {
    let at = instant.get();
    let elapsed = rising
        .rose
        .map_or(f64::INFINITY, |rose| (at - rose.get()) * GHATIKAS_PER_DAY);
    let remaining = rising
        .sets
        .map_or(f64::INFINITY, |sets| (sets.get() - at) * GHATIKAS_PER_DAY);
    (elapsed >= 0.0 && remaining >= 0.0).then_some(teistro_rules::Span { elapsed, remaining })
}

/// How much of the night had passed at an instant and how much was left, in
/// ghatikas: sunset to the next sunrise, under the day's own convention,
/// which the Indian profiles place at the disc's centre, where BPHS ch. 9
/// v. 14 measures the twilights from. Nothing by day, or on a day whose
/// Sun did not both rise and set.
fn night_span(day: &LocalDay, instant: JulianDay<Utc>) -> Option<teistro_rules::Span> {
    if !matches!(day.state, DayState::Normal) {
        return None;
    }
    let night = Interval::new(day.sunset, day.next_sunrise).ok()?;
    ghatikas(night, instant)
}

/// The pada of the nakshatra the Moon stands in, one to four.
///
/// `None` for a foundation that places no Moon, which is a chart the rest
/// of this module refuses by name anyway.
fn moons_pada(foundation: &ChartFoundation) -> Option<teistro_rules::Pada> {
    let moon = foundation
        .grahas
        .iter()
        .find(|position| position.graha == Graha::Moon)?;
    let longitude = Nas::from_degrees(Degrees::try_new(moon.longitude_deg).ok()?);
    // `Nas::pada` counts from zero and a rule's `Pada` from one.
    teistro_rules::Pada::try_new(longitude.pada().get() + 1).ok()
}

/// How much of a limb had passed at an instant and how much was left, in
/// ghatikas, or nothing when the instant falls outside it.
fn ghatikas(whole: Interval, instant: JulianDay<Utc>) -> Option<teistro_rules::Span> {
    let (from, to, at) = (whole.from.get(), whole.to.get(), instant.get());
    (at >= from && at <= to).then_some(teistro_rules::Span {
        elapsed: (at - from) * GHATIKAS_PER_DAY,
        remaining: (to - at) * GHATIKAS_PER_DAY,
    })
}

/// The chart the rules read, assembled from what the SDK computed.
///
/// `states` are the graha states of the same foundation — the SDK's
/// `teistro_state::state` — and are matched to the foundation's grahas by
/// name; a graha the states do not carry is refused rather than defaulted,
/// because a rule reading its dignity would otherwise answer on a value
/// nobody computed.
///
/// # Errors
///
/// A graha the foundation or the states do not carry, named.
pub fn rule_chart(
    foundation: &ChartFoundation,
    states: &[GrahaState],
    panchanga: Option<Panchanga>,
    strengths: Option<Strengths>,
) -> Result<RuleChart, Error> {
    let lagna = foundation.lagna_deg.rem_euclid(360.0);
    let lagna_sign = rashi_of(lagna)?;
    let mut placements: [Placement; 10] = [lagna_placement(lagna, lagna_sign)?; 10];
    for body in Body::ALL {
        let Body::Graha(graha) = body else {
            continue;
        };
        let placement = graha_placement(foundation, states, graha, lagna_sign)?;
        if let Some(at) = placements.get_mut(body.index()) {
            *at = placement;
        }
    }
    Ok(RuleChart {
        placements,
        panchanga,
        strengths,
    }
    .with_chara_karakas(EightKarakas::Parashara))
}

/// Everything the rules read of one chart reading: the chart, the points it
/// carried and its divisional charts.
///
/// ```no_run
/// # #[cfg(feature = "builtin-ephemeris")] {
/// use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
/// use teistro::rules::{Readings, shipped};
/// use teistro::{ChartRequest, Context, Ephemeris, RuleInputs, UtcOffset};
///
/// let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
/// let place = Place::new(Latitude::try_new(27.7)?, Longitude::try_new(85.3)?, Altitude::try_new(1400.0)?);
/// let rules = shipped::nabhasas();
/// let request = ChartRequest::at(place, UtcOffset::try_from_seconds(20_700)?).with_rule_inputs(rules);
/// let document = sdk.chart().reading(JulianDay::literal(2_447_995.489_583_333_5), &request)?.value;
/// let inputs = RuleInputs::of(&document)?;
/// let evaluator = inputs.evaluator(Readings::TEXTS).with_rules(rules);
/// let present = rules.iter().filter(|rule| evaluator.evaluate(rule).present).count();
/// # let _ = present;
/// # }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct RuleInputs {
    /// The chart, its karakas computed and its strengths the reading's
    /// Shadbala when it carried one.
    pub chart: RuleChart,
    /// The upagrahas and special lagnas the reading carried, each in its sign.
    pub points: Vec<PointAt>,
    /// The divisional charts the reading carried whose grahas and lagna are
    /// cast in one division; a mixed chart is not one a rule can step into.
    pub vargas: Vec<VargaSigns>,
}

impl RuleInputs {
    /// The rules' reading of a chart document.
    ///
    /// The document's panchanga is the day's almanac and a rule asks about
    /// the one limb that was **running**, so the limbs at the birth are
    /// read out of it at the chart's own instant
    /// (`birth_limbs`). A document without that section carries no
    /// limbs, and the 11 shipped rules that read one say so by name rather
    /// than failing: ask for it with `ChartRequest::with_panchanga`, which
    /// `with_rule_inputs` does.
    ///
    /// # Errors
    ///
    /// A document without its graha states, which every dignity and
    /// combustion a rule reads comes from, naming the section to ask for; and
    /// whatever [`rule_chart`] refuses.
    pub fn of(document: &Document) -> Result<RuleInputs, Error> {
        let states = document.state.as_deref().ok_or_else(|| {
            Error::invalid_arg(
                "the document carries no graha states, which a rule reads every dignity and \
                 combustion from; ask for them with `ChartRequest::with_state` or \
                 `with_rule_inputs`",
            )
            .with_field("state")
        })?;
        let strengths = document.shadbala.as_ref().map(shadbala_strengths);
        let limbs = document.panchanga.as_ref().and_then(|day| {
            birth_limbs(
                day,
                document.foundation.instant,
                moons_pada(&document.foundation)?,
                document.moment.as_ref(),
            )
        });
        let chart = rule_chart(&document.foundation, states, limbs, strengths)?;
        let points = document.points.as_ref().map_or_else(Vec::new, |points| {
            points
                .all()
                .iter()
                .map(|derived| PointAt {
                    point: derived.point,
                    longitude: derived.longitude_deg,
                    sign: derived.sign,
                })
                .collect()
        });
        let vargas = document
            .vargas
            .iter()
            .filter_map(|varga| {
                let division = varga.axis.grahas.varga?;
                if varga.axis.lagna.varga != Some(division) {
                    return None;
                }
                let mut signs = [varga.lagna.sign; 10];
                for body in Body::ALL {
                    if let Body::Graha(graha) = body {
                        let slot = signs.get_mut(body.index())?;
                        *slot = varga.graha(graha)?.sign;
                    }
                }
                Some(VargaSigns {
                    varga: division,
                    signs,
                })
            })
            .collect();
        Ok(RuleInputs {
            chart,
            points,
            vargas,
        })
    }

    /// The inputs with the eight chara karakas named as the settings name
    /// them (`jaimini.eight_karaka_order`, crux C101): the ranking is the
    /// sky's, and [`RuleInputs::of`] names it in the verses' order.
    ///
    /// ```no_run
    /// # #[cfg(feature = "builtin-ephemeris")] {
    /// use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// use teistro::{ChartRequest, Context, Ephemeris, RuleInputs, UtcOffset};
    ///
    /// // The baseline engine's naming: the least of eight is the Pitrikaraka.
    /// let sdk = Context::builder()
    ///     .profile("conformance-baseline")
    ///     .ephemeris([Ephemeris::Builtin])
    ///     .build()?;
    /// let place = Place::new(Latitude::try_new(27.7)?, Longitude::try_new(85.3)?, Altitude::try_new(1400.0)?);
    /// let request = ChartRequest::at(place, UtcOffset::try_from_seconds(20_700)?).with_state();
    /// let document = sdk.chart().reading(JulianDay::literal(2_447_995.489_583_333_5), &request)?.value;
    /// let inputs = RuleInputs::of(&document)?.under(sdk.settings());
    /// # let _ = inputs;
    /// # }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn under(mut self, settings: &Settings) -> RuleInputs {
        self.chart = self.chart.with_chara_karakas(karaka_order(settings));
        self
    }

    /// An evaluator over the chart under `readings`, given the points, the
    /// divisions and the tables the SDK ships ([`Tables::classical`], which
    /// [`RuleRequest::rule_set`](crate::RuleRequest::rule_set) holds every
    /// rule to); give it the rules it reads by key with
    /// [`Evaluator::with_rules`].
    #[must_use]
    pub fn evaluator(&self, readings: Readings) -> Evaluator<'_> {
        Evaluator::new(&self.chart, readings)
            .with_points(&self.points)
            .with_vargas(&self.vargas)
            .with_tables(Tables::classical())
    }
}

/// The rule chart of a foundation and its states, its eight karakas named
/// as the settings name them: what every context reading of the karakas
/// starts from.
pub(crate) fn ruled(
    foundation: &ChartFoundation,
    states: &[GrahaState],
    settings: &Settings,
) -> Result<RuleChart, Error> {
    Ok(rule_chart(foundation, states, None, None)?.with_chara_karakas(karaka_order(settings)))
}

/// The rules' naming of eight chara karakas under the settings
/// (`jaimini.eight_karaka_order`, crux C101).
pub(crate) fn karaka_order(settings: &Settings) -> EightKarakas {
    match settings.jaimini.eight_karaka_order {
        EightKarakaOrder::PitrikarakaLast => EightKarakas::PitrikarakaLast,
        _ => EightKarakas::Parashara,
    }
}

/// A Shadbala reading as the rules compare strengths: each graha's rupas and
/// the rupas it must reach. The nodes and the lagna have none.
fn shadbala_strengths(reading: &ShadbalaReading) -> Strengths {
    let mut of = [None; 10];
    let mut required = [None; 10];
    for graha in &reading.grahas {
        let at = Body::Graha(graha.graha).index();
        if let (Some(value), Some(needed)) = (of.get_mut(at), required.get_mut(at)) {
            *value = Some(graha.rupas);
            *needed = Some(graha.required_rupas);
        }
    }
    Strengths {
        measure: StrengthMeasure::Shadbala,
        of,
        required,
    }
}

/// A sub-period whose major period's lord is a maraka, inside the ages a class
/// of life runs to, with what its periods are as marakas. It is a period of
/// vulnerability to be read with the span of life, never a date, and its
/// [`Vulnerability::presentation`] says so to whoever renders it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MarakaWindow {
    /// When it runs.
    pub interval: Interval,
    /// The major period's lord.
    pub major: Graha,
    /// The sub-period's lord.
    pub sub: Graha,
    /// What the two are as marakas (BPHS ch. 44).
    pub vulnerability: Vulnerability,
}

/// The maraka windows of a dasha over the ages a class of life runs to (BPHS
/// ch. 44 vv. 3 to 5 and 10 to 11): every sub-period whose major period's lord
/// carries a reason, in time order, each with its vulnerability, so a caller
/// can narrow to v. 8's malefic major period in a malefic sub-period or to the
/// star periods v. 15 gives the class. Ages are years of 365.25 days from
/// `birth`; the classes past a hundred years run to the end of the dasha.
///
/// # Errors
///
/// A birth whose ages fall outside the Julian day's range.
pub fn maraka_windows(
    evaluator: &Evaluator<'_>,
    timeline: &impl Timeline,
    birth: JulianDay<Utc>,
    class: LifeClass,
) -> Result<Vec<MarakaWindow>, Error> {
    const YEAR_DAYS: f64 = 365.25;
    const BEYOND_YEARS: f64 = 1000.0;
    let marakas = evaluator.marakas();
    let (from, to) = age_span(class);
    let at = |years: f64| {
        JulianDay::try_new(birth.get() + years * YEAR_DAYS)
            .map_err(|invalid| Error::invalid_arg(invalid.to_string()).with_field("birth"))
    };
    let window = Interval::new(at(from)?, at(to.unwrap_or(BEYOND_YEARS))?)?;
    let depth = Depth::try_new(2).map_err(|invalid| Error::internal(invalid.to_string()))?;
    let running = |period: &Period| Running {
        lord: period.lord,
        sign: period.sign,
    };
    let mut major: Option<Period> = None;
    let mut windows = Vec::new();
    for period in timeline.periods(window, depth) {
        if period.path.indices().len() == 1 {
            major = Some(period);
            continue;
        }
        let Some(parent) = major else {
            continue;
        };
        let vulnerability = evaluator.vulnerability(&marakas, [running(&parent), running(&period)]);
        if vulnerability
            .levels
            .first()
            .is_some_and(|reasons| !reasons.is_empty())
        {
            windows.push(MarakaWindow {
                interval: period.interval,
                major: parent.lord,
                sub: period.lord,
                vulnerability,
            });
        }
    }
    Ok(windows)
}

/// The periods of a dasha chain as the rules read them, from the mahadasha
/// down: each period's lord and, in a sign-based dasha, its sign. For
/// [`Evaluator::delivery`](teistro_rules::Evaluator::delivery).
pub fn rule_periods(chain: &Chain) -> impl Iterator<Item = Running> + '_ {
    chain.iter().map(|period| Running {
        lord: period.lord,
        sign: period.sign,
    })
}

/// The divisional charts of a rule chart, each body's sign in each division
/// under the SDK's classical scheme for it, for an evaluator's
/// [`with_vargas`](teistro_rules::Evaluator::with_vargas).
///
/// # Errors
///
/// A longitude that is not a finite number, which no division can place.
pub fn rule_vargas(
    chart: &RuleChart,
    vargas: impl IntoIterator<Item = Varga>,
) -> Result<Vec<VargaSigns>, Error> {
    vargas
        .into_iter()
        .map(|varga| {
            let mut signs = [Rashi::Aries; 10];
            for (slot, body) in signs.iter_mut().zip(Body::ALL) {
                *slot = varga_sign(varga, chart.placement(body).longitude)?;
            }
            Ok(VargaSigns { varga, signs })
        })
        .collect()
}

/// The lagna's own placement: a point, so it is never retrograde, the Sun
/// never burns it, and it holds no dignity of its own.
fn lagna_placement(longitude: f64, sign: Rashi) -> Result<Placement, Error> {
    Ok(Placement {
        longitude,
        sign,
        // Core's refusal is already the SDK's error type, so the
        // bridge no longer restates it.
        house: House::try_new(1)?,
        dignity: Dignity::Neutral,
        retrograde: false,
        combust: false,
        karaka7: None,
        karaka8: None,
        navamsha: navamsha_of(longitude)?,
    })
}

/// One graha's placement, joined from the foundation and its state.
fn graha_placement(
    foundation: &ChartFoundation,
    states: &[GrahaState],
    graha: Graha,
    lagna_sign: Rashi,
) -> Result<Placement, Error> {
    let position = foundation.graha(graha).ok_or_else(|| {
        Error::invalid_arg(format!(
            "the foundation carries no `{}`, which a rule chart needs",
            graha.key()
        ))
        .with_field("grahas")
    })?;
    let state = states
        .iter()
        .find(|state| state.graha == graha)
        .ok_or_else(|| {
            Error::invalid_arg(format!(
                "the states carry no `{}`, whose dignity and combustion a rule chart needs",
                graha.key()
            ))
            .with_field("states")
        })?;
    let longitude = position.longitude_deg.rem_euclid(360.0);
    Ok(Placement {
        longitude,
        sign: state.sign,
        // Whole signs from the lagna, which is what every rule counts by;
        // the state's own house follows the chart's placement system, and a
        // rule that meant the chalit would have to say so.
        house: House::between(lagna_sign, state.sign),
        dignity: state.dignity,
        retrograde: position.speed_deg_per_day < 0.0,
        combust: state.is_combust(),
        karaka7: None,
        karaka8: None,
        navamsha: navamsha_of(longitude)?,
    })
}

/// The sign a sidereal longitude falls in.
fn rashi_of(longitude: f64) -> Result<Rashi, Error> {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a longitude brought into 0..360 over thirty is 0 to 11"
    )]
    let at = (longitude.rem_euclid(360.0) / 30.0) as u16;
    Rashi::from_id(at.min(11))
        .ok_or_else(|| Error::internal("a longitude under 360 fell outside the twelve signs"))
}

/// The navamsha sign of a sidereal longitude, under the catalogue's scheme.
pub(crate) fn navamsha_of(longitude: f64) -> Result<Rashi, Error> {
    varga_sign(Varga::D9, longitude)
}

/// The sign of a sidereal longitude in a division, under the SDK's classical
/// scheme for it.
pub(crate) fn varga_sign(varga: Varga, longitude: f64) -> Result<Rashi, Error> {
    let degrees = Degrees::try_new(longitude.rem_euclid(360.0)).map_err(|invalid| {
        Error::invalid_arg(format!(
            "a longitude of {longitude} cannot be placed: {invalid}"
        ))
        .with_field("longitude")
    })?;
    Ok(sign(&Scheme::of(varga), Nas::from_degrees(degrees)))
}
