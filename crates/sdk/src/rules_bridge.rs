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
//! What it cannot fill it leaves empty rather than guessing: the SDK computes
//! no chara karakas, so a rule naming one answers false until it does, and a
//! chart's panchanga and strengths are passed in because they are separate
//! readings with their own settings.

use teistro_chart::foundation::ChartFoundation;
use teistro_core::angle::Nas;
use teistro_core::catalogue::{Dignity, Graha, Rashi, Varga};
use teistro_core::error::Error;
use teistro_core::quantity::Degrees;
use teistro_rules::{Body, House, Panchanga, Placement, RuleChart, Strengths};
use teistro_state::GrahaState;
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
    })
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
    let degrees = Degrees::try_new(longitude.rem_euclid(360.0))
        .map_err(|invalid| Error::internal(invalid.to_string()))?;
    Ok(sign(&Scheme::of(Varga::D9), Nas::from_degrees(degrees)))
}
