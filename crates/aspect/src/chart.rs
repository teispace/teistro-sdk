//! The assembly: every relation of a founded chart.
//!
//! The entry point takes the whole foundation rather than one graha,
//! because a relation is between two bodies and no body knows on its own
//! who is looking at it.
//!
//! Every accessor lends its relations rather than building a fresh
//! `Vec`, because `rules` will ask "is this aspected by a malefic" once
//! per predicate per chart and should not allocate to find out.

use serde::{Deserialize, Deserializer, Serialize};
use teistro_chart::foundation::ChartFoundation;
use teistro_core::angle::Nas;
use teistro_core::boundary::Boundaries;
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::error::Error;
use teistro_core::quantity::Degrees;
use teistro_core::settings::Settings;

use crate::drishti::{self, Strength};
use crate::rashi;

/// One body's gaze at another.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Drishti {
    /// The body looking.
    pub from: Graha,
    /// The body looked at.
    pub to: Graha,
    /// Which house of the first's sign the second stands in, counting
    /// inclusively from one.
    pub houses: u8,
    /// How strongly.
    pub strength: Strength,
    /// How near the looking body stands to a sign edge, which is what
    /// would change this relation.
    pub from_edge: Boundaries,
    /// How near the body looked at stands to one.
    pub to_edge: Boundaries,
}

impl Drishti {
    /// Whether both ends stand further than a caller's own tolerance
    /// from a sign edge, degrees — so the relation would survive an
    /// ayanamsha that moved by that much.
    ///
    /// The tolerance is the caller's because it is a property of its
    /// provider's accuracy and not of the tradition. A whole-sign
    /// drishti is a step function: over the corpus, moving every body
    /// the recording engine flagged as near an edge across it changes 52
    /// of 6696 relations (`03-design/aspect-drishti-measured.md` §5).
    #[must_use]
    pub fn is_firm(&self, tolerance_deg: f64) -> bool {
        self.from_edge.sign_deg > tolerance_deg && self.to_edge.sign_deg > tolerance_deg
    }

    /// How near either end stands to the sign edge that decides this,
    /// degrees.
    #[must_use]
    pub fn nearest_edge_deg(&self) -> f64 {
        self.from_edge.sign_deg.min(self.to_edge.sign_deg)
    }
}

/// Two bodies that look at each other.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Mutual {
    /// One of them.
    pub first: Graha,
    /// The other.
    pub second: Graha,
    /// Which house of the first's sign the second stands in.
    pub houses: u8,
    /// How strongly the first looks at the second.
    pub outward: Strength,
    /// How strongly the second looks back.
    pub back: Strength,
}

impl Mutual {
    /// Whether both look fully.
    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.outward.is_full() && self.back.is_full()
    }
}

/// Where a body stands, as this module needs it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "AspectPlaced"))]
struct Placed {
    graha: Graha,
    sign: Rashi,
    edge: Boundaries,
}

/// Every relation of one founded chart.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Aspects {
    /// Every drishti of any strength, ordered by the looking body and
    /// then by the body looked at, in the foundation's own order.
    relations: Vec<Drishti>,
    /// Where each body stands.
    placed: Vec<Placed>,
    /// The drishti table the settings named.
    table: &'static str,
}

impl<'de> Deserialize<'de> for Aspects {
    /// Reads the relations back, and the **table by its name**.
    ///
    /// The table is a `&'static str` that names one of the tables this
    /// build ships, and no document can produce one, so it is looked up
    /// rather than owned — the same rule [`crate::orb::Angle`] and a
    /// divisional scheme are read under. [`crate::drishti::table`] is
    /// the lookup, so the set of names lives in one place.
    ///
    /// # Errors
    ///
    /// A table this build does not ship, named.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Aspects, D::Error> {
        /// The fields a document carries, as it carries them.
        #[derive(Deserialize)]
        struct Written {
            relations: Vec<Drishti>,
            placed: Vec<Placed>,
            table: String,
        }
        let written = Written::deserialize(deserializer)?;
        Ok(Aspects {
            relations: written.relations,
            placed: written.placed,
            table: crate::drishti::table(&written.table).map_err(serde::de::Error::custom)?,
        })
    }
}

impl Aspects {
    /// Every relation of a founded chart.
    ///
    /// # Errors
    ///
    /// `UNSUPPORTED` for a drishti table the SDK does not ship, and
    /// `INVALID_ARG` for a longitude the foundation could not have
    /// produced. Every rule is otherwise total over a founded chart, and
    /// a chart with fewer than two bodies gives no relations rather than
    /// an error.
    pub fn of(foundation: &ChartFoundation, settings: &Settings) -> Result<Aspects, Error> {
        let table = drishti::table(&settings.aspect.drishti_table)?;
        let nodes = settings.aspect.node_aspects;
        let mut placed = Vec::with_capacity(foundation.grahas.len());
        for position in &foundation.grahas {
            let longitude = canonical(position.longitude_deg)?;
            placed.push(Placed {
                graha: position.graha,
                sign: longitude.sign(),
                edge: Boundaries::of(longitude),
            });
        }
        let mut relations = Vec::with_capacity(placed.len() * placed.len());
        for from in &placed {
            for to in &placed {
                if from.graha == to.graha {
                    continue;
                }
                let houses = drishti::house_count(from.sign, to.sign);
                let strength = drishti::quarters_under(from.graha, houses, nodes);
                if !strength.is_any() {
                    continue;
                }
                relations.push(Drishti {
                    from: from.graha,
                    to: to.graha,
                    houses,
                    strength,
                    from_edge: from.edge,
                    to_edge: to.edge,
                });
            }
        }
        Ok(Aspects {
            relations,
            placed,
            table,
        })
    }

    /// The drishti table these were computed under, which a value
    /// carries so a reader need not go back to the settings.
    #[must_use]
    pub const fn table(&self) -> &'static str {
        self.table
    }

    /// Every relation, strongest first among those a body casts.
    #[must_use]
    pub fn all(&self) -> &[Drishti] {
        &self.relations
    }

    /// The relations a body casts.
    pub fn cast_by(&self, graha: Graha) -> impl Iterator<Item = &Drishti> {
        self.relations
            .iter()
            .filter(move |relation| relation.from == graha)
    }

    /// The relations a body receives.
    pub fn on(&self, graha: Graha) -> impl Iterator<Item = &Drishti> {
        self.relations
            .iter()
            .filter(move |relation| relation.to == graha)
    }

    /// The relation one body casts on another, if there is one.
    #[must_use]
    pub fn between(&self, from: Graha, to: Graha) -> Option<&Drishti> {
        self.relations
            .iter()
            .find(|relation| relation.from == from && relation.to == to)
    }

    /// Whether one body aspects another at all.
    #[must_use]
    pub fn aspects(&self, from: Graha, to: Graha) -> bool {
        self.between(from, to).is_some()
    }

    /// Whether one body aspects another fully.
    #[must_use]
    pub fn aspects_fully(&self, from: Graha, to: Graha) -> bool {
        self.between(from, to)
            .is_some_and(|relation| relation.strength.is_full())
    }

    /// The strongest relation on a body, or `None` when nothing looks at
    /// it.
    #[must_use]
    pub fn strongest_on(&self, graha: Graha) -> Option<&Drishti> {
        self.on(graha).max_by_key(|relation| relation.strength)
    }

    /// Every pair that looks at each other, each pair once, in the
    /// foundation's order.
    ///
    /// The rule is [`mutual_pairs`]'s: taking a pair once is a property of
    /// a **set of relations** rather than of this container, so it lives
    /// over a slice and a reader that holds only the relations — a
    /// narrative composer, a pass measuring one — shares this answer
    /// rather than writing a second one.
    pub fn mutual(&self) -> impl Iterator<Item = Mutual> {
        mutual_pairs(&self.relations)
    }

    /// The bodies sharing a sign with this one, which is what the
    /// tradition means by a conjunction.
    pub fn conjunct(&self, graha: Graha) -> impl Iterator<Item = Graha> {
        let here = self.sign(graha);
        self.placed.iter().filter_map(move |other| {
            (other.graha != graha && Some(other.sign) == here).then_some(other.graha)
        })
    }

    /// Whether one body aspects another under the **rashi** drishti,
    /// which is a relation between their signs and is always mutual.
    #[must_use]
    pub fn rashi_aspects(&self, from: Graha, to: Graha) -> bool {
        match (self.sign(from), self.sign(to)) {
            (Some(from), Some(to)) => rashi::aspects(from, to),
            _ => false,
        }
    }

    /// The sign a body stands in.
    #[must_use]
    pub fn sign(&self, graha: Graha) -> Option<Rashi> {
        self.placed
            .iter()
            .find(|placed| placed.graha == graha)
            .map(|placed| placed.sign)
    }
}

/// Every pair among these relations that looks at each other, each pair
/// once, in the order the relations first name a body as the one looking.
///
/// That order is the founding order where the relations came from a founded
/// chart, because dropping the bodies that cast nothing keeps what is left
/// in the same relative order — and a body that casts nothing is in no
/// mutual pair either.
///
/// ```
/// use teistro_aspect::{Drishti, Strength, mutual_pairs};
/// # use teistro_core::boundary::Boundaries;
/// # use teistro_core::catalogue::Graha;
/// # let edge = Boundaries { sign_deg: 15.0, nakshatra_deg: 6.0, pada_deg: 3.0 };
/// # let at = |from, to, houses, strength| Drishti {
/// #     from, to, houses, strength, from_edge: edge, to_edge: edge,
/// # };
/// // Saturn and Jupiter look at each other; Mars looks at Saturn alone.
/// let relations = [
///     at(Graha::Saturn, Graha::Jupiter, 7, Strength::Full),
///     at(Graha::Jupiter, Graha::Saturn, 7, Strength::Full),
///     at(Graha::Mars, Graha::Saturn, 4, Strength::ThreeQuarters),
/// ];
/// let pairs: Vec<_> = mutual_pairs(&relations).collect();
/// assert_eq!(pairs.len(), 1, "once per pair, and Mars is not looked back at");
/// assert_eq!(pairs[0].first, Graha::Saturn);
/// ```
pub fn mutual_pairs(relations: &[Drishti]) -> impl Iterator<Item = Mutual> + '_ {
    /// Where a body first appears as the one looking, which is the order
    /// the relations were built in.
    fn order(relations: &[Drishti], graha: Graha) -> Option<usize> {
        relations.iter().position(|relation| relation.from == graha)
    }
    relations.iter().filter_map(move |outward| {
        let back = relations
            .iter()
            .find(|back| back.from == outward.to && back.to == outward.from)?;
        // Once per pair, taking the direction listed first.
        let first = order(relations, outward.from)?;
        let second = order(relations, outward.to)?;
        (first < second).then_some(Mutual {
            first: outward.from,
            second: outward.to,
            houses: outward.houses,
            outward: outward.strength,
            back: back.strength,
        })
    })
}

/// A longitude the foundation carries, as the canonical angle.
fn canonical(degrees: f64) -> Result<Nas, Error> {
    Ok(Nas::from_degrees(Degrees::try_new(
        degrees.rem_euclid(360.0),
    )?))
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "tests fail by panicking"
    )]

    use super::canonical;
    use teistro_core::catalogue::Rashi;

    #[test]
    fn a_longitude_is_wrapped_before_it_is_canonical() {
        assert_eq!(canonical(370.0).unwrap(), canonical(10.0).unwrap());
        assert_eq!(canonical(-10.0).unwrap(), canonical(350.0).unwrap());
        assert_eq!(canonical(45.0).unwrap().sign(), Rashi::Taurus);
        assert!(canonical(f64::NAN).is_err(), "a NaN is refused, not placed");
    }
}
