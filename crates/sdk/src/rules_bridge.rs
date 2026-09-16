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
use teistro_core::angle::Nas;
use teistro_core::catalogue::{Dignity, Graha, Rashi, Varga};
use teistro_core::error::Error;
use teistro_core::quantity::Degrees;
use teistro_dasha::Chain;
use teistro_rules::{
    Body, EightKarakas, Evaluator, House, Panchanga, Placement, PointAt, Readings, RuleChart,
    Running, StrengthMeasure, Strengths, VargaSigns,
};
use teistro_serial::Document;
use teistro_state::GrahaState;
use teistro_strength::ShadbalaReading;
use teistro_vargas::{Scheme, sign};

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
    /// The document's panchanga is the day's almanac, not the limbs at the
    /// birth a rule asks about, so the chart carries none; a caller with the
    /// birth's limbs sets `chart.panchanga`.
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
        let chart = rule_chart(&document.foundation, states, None, strengths)?;
        let points = document.points.as_ref().map_or_else(Vec::new, |points| {
            points
                .all()
                .iter()
                .map(|derived| PointAt {
                    point: derived.point,
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

    /// An evaluator over the chart under `readings`, given the points and the
    /// divisions; give it the rules it reads by key with
    /// [`Evaluator::with_rules`].
    #[must_use]
    pub fn evaluator(&self, readings: Readings) -> Evaluator<'_> {
        Evaluator::new(&self.chart, readings)
            .with_points(&self.points)
            .with_vargas(&self.vargas)
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
        house: House::try_new(1).map_err(Error::internal)?,
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
fn navamsha_of(longitude: f64) -> Result<Rashi, Error> {
    varga_sign(Varga::D9, longitude)
}

/// The sign of a sidereal longitude in a division, under the SDK's classical
/// scheme for it.
fn varga_sign(varga: Varga, longitude: f64) -> Result<Rashi, Error> {
    let degrees = Degrees::try_new(longitude.rem_euclid(360.0)).map_err(|invalid| {
        Error::invalid_arg(format!(
            "a longitude of {longitude} cannot be placed: {invalid}"
        ))
        .with_field("longitude")
    })?;
    Ok(sign(&Scheme::of(varga), Nas::from_degrees(degrees)))
}
