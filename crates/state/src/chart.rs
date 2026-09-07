//! The assembly: every graha of a founded chart, with what it is.
//!
//! The entry point takes the whole foundation rather than one graha,
//! because three of the rules need the others: the temporary friendship
//! needs the dispositor's sign, the lajjitadi need who shares a sign, and
//! a body cannot know it is at war on its own.

use serde::Serialize;
use teistro_chart::foundation::ChartFoundation;
use teistro_core::angle::Nas;
use teistro_core::catalogue::{
    AvasthaBaladi, AvasthaDeeptadi, AvasthaJagradadi, Dignity, Graha, Rashi,
};
use teistro_core::error::Error;
use teistro_core::quantity::Degrees;
use teistro_core::settings::Settings;

use crate::avastha::{self, AtWar, Lajjitadi, Placement, War};
use crate::boundary::Boundaries;
use crate::burn::{self, Combustion};
use crate::dignity::{self, Friendship};

/// Which way a body is going, and how fast.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Motion {
    /// Whether it is going backwards through the zodiac.
    pub retrograde: bool,
    /// Degrees a day, negative when retrograde.
    pub speed_deg_per_day: f64,
}

/// What one graha is, in one chart.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct GrahaState {
    /// Which graha.
    pub graha: Graha,
    /// The sign it stands in.
    pub sign: Rashi,
    /// The bhava it falls in, under the chart's placement system.
    pub house: u8,
    /// Its dignity.
    pub dignity: Dignity,
    /// How it stands to its dispositor, three ways.
    pub friendship: Friendship,
    /// What the Sun does to it.
    pub combustion: Combustion,
    /// Which way it is going.
    pub motion: Motion,
    /// Which fifth of its sign it stands in.
    pub age: AvasthaBaladi,
    /// Awake, dreaming or asleep.
    pub wakefulness: AvasthaJagradadi,
    /// The bright state, where the SDK can decide one.
    pub deeptadi: Option<AvasthaDeeptadi>,
    /// The lajjitadi that hold, and the ones nothing can decide yet.
    pub lajjitadi: Lajjitadi,
    /// The war it is in, if it is in one.
    pub war: Option<War>,
    /// How near it stands to a classification boundary.
    pub boundaries: Boundaries,
}

impl GrahaState {
    /// Whether the Sun burns it at all.
    #[must_use]
    pub const fn is_combust(&self) -> bool {
        self.combustion.is_combust()
    }

    /// Whether it stands in a sign it is at home in.
    #[must_use]
    pub const fn is_at_home(&self) -> bool {
        matches!(
            self.dignity,
            Dignity::Exalted
                | Dignity::DeepExalted
                | Dignity::Mooltrikona
                | Dignity::OwnSign
                | Dignity::GreatFriend
        )
    }

    /// Whether it lost a war it is in.
    #[must_use]
    pub fn lost_its_war(&self) -> bool {
        self.war.is_some_and(|war| !war.is_winner)
    }
}

/// What every graha of a founded chart is.
///
/// # Errors
///
/// `UNSUPPORTED` for a combustion table the SDK does not ship, and
/// `INVALID_ARG` for a longitude the foundation could not have produced.
/// Every rule is otherwise total over a founded chart.
pub fn state(foundation: &ChartFoundation, settings: &Settings) -> Result<Vec<GrahaState>, Error> {
    let orbs = burn::table(&settings.state.combustion_orbs)?;
    // Where everything stands, which three of the rules read.
    let mut signs: Vec<(Graha, Rashi)> = Vec::with_capacity(foundation.grahas.len());
    let mut fighters: Vec<AtWar> = Vec::with_capacity(foundation.grahas.len());
    for position in &foundation.grahas {
        let longitude = canonical(position.longitude_deg)?;
        signs.push((position.graha, longitude.sign()));
        fighters.push(AtWar {
            graha: position.graha,
            longitude_deg: position.longitude_deg.rem_euclid(360.0),
            latitude_deg: position.latitude_deg,
        });
    }
    let sign_of = |graha: Graha| {
        signs
            .iter()
            .find(|(who, _)| *who == graha)
            .map(|(_, sign)| *sign)
    };

    // The dignities first: the lajjitadi read them.
    let mut placements: Vec<Placement> = Vec::with_capacity(foundation.grahas.len());
    let mut friendships: Vec<Friendship> = Vec::with_capacity(foundation.grahas.len());
    for position in &foundation.grahas {
        let longitude = canonical(position.longitude_deg)?;
        let sign = longitude.sign();
        let friendship = dignity::friendship(position.graha, sign, sign_of);
        placements.push(Placement {
            graha: position.graha,
            sign,
            house: position.house.bhava,
            dignity: dignity::dignity(
                position.graha,
                sign,
                degrees_in_sign(longitude),
                friendship.compound,
            ),
        });
        friendships.push(friendship);
    }

    let mut states = Vec::with_capacity(foundation.grahas.len());
    for ((position, placement), friendship) in
        foundation.grahas.iter().zip(&placements).zip(&friendships)
    {
        let longitude = canonical(position.longitude_deg)?;
        states.push(GrahaState {
            graha: position.graha,
            sign: placement.sign,
            house: placement.house,
            dignity: placement.dignity,
            friendship: *friendship,
            combustion: burn::combustion(
                position.graha,
                from_sun(foundation, position.graha),
                position.is_retrograde(),
                orbs,
            ),
            motion: Motion {
                retrograde: position.is_retrograde(),
                speed_deg_per_day: position.speed_deg_per_day,
            },
            age: avastha::age(placement.sign, degrees_in_sign(longitude)),
            wakefulness: avastha::wakefulness(placement.dignity),
            deeptadi: avastha::deeptadi(placement.dignity),
            lajjitadi: avastha::lajjitadi(*placement, &placements),
            war: avastha::war(position.graha, &fighters),
            boundaries: Boundaries::of(longitude),
        });
    }
    Ok(states)
}

/// How far a body stands from the Sun, degrees, or `None` when the chart
/// carries no Sun or the body *is* the Sun.
fn from_sun(foundation: &ChartFoundation, graha: Graha) -> Option<f64> {
    if graha == Graha::Sun {
        return None;
    }
    let sun = foundation
        .grahas
        .iter()
        .find(|position| position.graha == Graha::Sun)?;
    let body = foundation
        .grahas
        .iter()
        .find(|position| position.graha == graha)?;
    Some(avastha::separation(body.longitude_deg, sun.longitude_deg))
}

/// A longitude the foundation carries, as the canonical angle.
fn canonical(degrees: f64) -> Result<Nas, Error> {
    Ok(Nas::from_degrees(Degrees::try_new(
        degrees.rem_euclid(360.0),
    )?))
}

/// How far into its sign an angle is, degrees.
fn degrees_in_sign(longitude: Nas) -> f64 {
    longitude.in_sign().to_degrees()
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "tests fail by panicking"
    )]

    use super::{canonical, degrees_in_sign};
    use teistro_core::catalogue::Rashi;

    #[test]
    fn a_longitude_is_wrapped_before_it_is_canonical() {
        assert_eq!(canonical(370.0).unwrap(), canonical(10.0).unwrap());
        assert_eq!(canonical(-10.0).unwrap(), canonical(350.0).unwrap());
        assert!(canonical(f64::NAN).is_err(), "a NaN is refused, not placed");
    }

    #[test]
    fn a_degree_inside_a_sign_is_measured_from_its_start() {
        let at = canonical(45.5).unwrap();
        assert_eq!(at.sign(), Rashi::Taurus);
        assert!((degrees_in_sign(at) - 15.5).abs() < 1e-9);
        assert!(degrees_in_sign(canonical(0.0).unwrap()).abs() < 1e-9);
    }
}
