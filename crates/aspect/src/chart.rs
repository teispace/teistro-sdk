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
///
/// ```
/// use teistro_aspect::{Drishti, Strength};
/// use teistro_core::boundary::Boundaries;
/// use teistro_core::catalogue::Graha;
///
/// // Saturn at 2° of its sign looks fully at Jupiter in the seventh, at 14°.
/// let saturn = Boundaries { sign_deg: 2.0, nakshatra_deg: 2.0, pada_deg: 1.3 };
/// let jupiter = Boundaries { sign_deg: 14.0, nakshatra_deg: 4.0, pada_deg: 0.7 };
/// let drishti = Drishti {
///     from: Graha::Saturn,
///     to: Graha::Jupiter,
///     houses: 7,
///     strength: Strength::Full,
///     from_edge: saturn,
///     to_edge: jupiter,
/// };
/// assert!(drishti.strength.is_full());
/// ```
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
    ///
    /// ```
    /// use teistro_aspect::{Drishti, Strength};
    /// use teistro_core::boundary::Boundaries;
    /// use teistro_core::catalogue::Graha;
    ///
    /// // Saturn at 2° of its sign looks fully at Jupiter in the seventh, at 14°.
    /// let saturn = Boundaries { sign_deg: 2.0, nakshatra_deg: 2.0, pada_deg: 1.3 };
    /// let jupiter = Boundaries { sign_deg: 14.0, nakshatra_deg: 4.0, pada_deg: 0.7 };
    /// let drishti = Drishti {
    ///     from: Graha::Saturn,
    ///     to: Graha::Jupiter,
    ///     houses: 7,
    ///     strength: Strength::Full,
    ///     from_edge: saturn,
    ///     to_edge: jupiter,
    /// };
    /// // Firm against a one-degree doubt, not against a three-degree one.
    /// assert!(drishti.is_firm(1.0));
    /// assert!(!drishti.is_firm(3.0));
    /// ```
    #[must_use]
    pub fn is_firm(&self, tolerance_deg: f64) -> bool {
        self.from_edge.sign_deg > tolerance_deg && self.to_edge.sign_deg > tolerance_deg
    }

    /// How near either end stands to the sign edge that decides this,
    /// degrees.
    ///
    /// ```
    /// use teistro_aspect::{Drishti, Strength};
    /// use teistro_core::boundary::Boundaries;
    /// use teistro_core::catalogue::Graha;
    ///
    /// // Saturn at 2° of its sign looks fully at Jupiter in the seventh, at 14°.
    /// let saturn = Boundaries { sign_deg: 2.0, nakshatra_deg: 2.0, pada_deg: 1.3 };
    /// let jupiter = Boundaries { sign_deg: 14.0, nakshatra_deg: 4.0, pada_deg: 0.7 };
    /// let drishti = Drishti {
    ///     from: Graha::Saturn,
    ///     to: Graha::Jupiter,
    ///     houses: 7,
    ///     strength: Strength::Full,
    ///     from_edge: saturn,
    ///     to_edge: jupiter,
    /// };
    /// assert_eq!(drishti.nearest_edge_deg(), 2.0);
    /// ```
    #[must_use]
    pub fn nearest_edge_deg(&self) -> f64 {
        self.from_edge.sign_deg.min(self.to_edge.sign_deg)
    }
}

/// Two bodies that look at each other.
///
/// ```
/// use teistro_aspect::{Mutual, Strength};
/// use teistro_core::catalogue::Graha;
///
/// // Mars looks at Saturn in its fourth fully, and Saturn back from Mars's
/// // tenth, its own special house.
/// let pair = Mutual {
///     first: Graha::Mars,
///     second: Graha::Saturn,
///     houses: 4,
///     outward: Strength::Full,
///     back: Strength::Full,
/// };
/// assert!(pair.is_full());
/// ```
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
    ///
    /// ```
    /// use teistro_aspect::{Mutual, Strength};
    /// use teistro_core::catalogue::Graha;
    ///
    /// // The Moon looks at the Sun in its fifth by half, and the Sun back from
    /// // the ninth by half.
    /// let trine = Mutual {
    ///     first: Graha::Moon,
    ///     second: Graha::Sun,
    ///     houses: 5,
    ///     outward: Strength::Half,
    ///     back: Strength::Half,
    /// };
    /// assert!(!trine.is_full());
    /// ```
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
///
/// ```
/// use teistro_aspect::Aspects;
/// use teistro_chart::sample;
/// use teistro_core::catalogue::Graha;
///
/// let aspects = Aspects::of(&sample::foundation()?, &sample::resolved().settings)?;
/// // Every relation it holds is one of some strength.
/// assert!(aspects.all().iter().all(|drishti| drishti.strength.is_any()));
/// assert!(aspects.sign(Graha::Sun).is_some());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
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
    /// ```
    /// use teistro_aspect::{Aspects, drishti};
    /// use teistro_chart::sample;
    /// use teistro_core::catalogue::Graha;
    ///
    /// let settings = sample::resolved().settings;
    /// let aspects = Aspects::of(&sample::foundation()?, &settings)?;
    /// // Each relation is the table's between the two signs.
    /// let (sun, moon) = (aspects.sign(Graha::Sun), aspects.sign(Graha::Moon));
    /// if let (Some(sun), Some(moon)) = (sun, moon) {
    ///     let strength = drishti::between_under(Graha::Sun, sun, moon, settings.aspect.node_aspects);
    ///     assert_eq!(aspects.between(Graha::Sun, Graha::Moon).map(|d| d.strength), strength.is_any().then_some(strength));
    /// }
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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
    ///
    /// ```
    /// use teistro_aspect::Aspects;
    /// use teistro_chart::sample;
    ///
    /// let aspects = Aspects::of(&sample::foundation()?, &sample::resolved().settings)?;
    /// assert_eq!(aspects.table(), teistro_aspect::drishti::PARASHARA);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub const fn table(&self) -> &'static str {
        self.table
    }

    /// Every relation, strongest first among those a body casts.
    ///
    /// ```
    /// use teistro_aspect::Aspects;
    /// use teistro_chart::sample;
    ///
    /// let aspects = Aspects::of(&sample::foundation()?, &sample::resolved().settings)?;
    /// // No body looks at itself.
    /// assert!(aspects.all().iter().all(|drishti| drishti.from != drishti.to));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn all(&self) -> &[Drishti] {
        &self.relations
    }

    /// The relations a body casts.
    ///
    /// ```
    /// use teistro_aspect::Aspects;
    /// use teistro_chart::sample;
    /// use teistro_core::catalogue::Graha;
    ///
    /// let aspects = Aspects::of(&sample::foundation()?, &sample::resolved().settings)?;
    /// assert!(aspects.cast_by(Graha::Jupiter).all(|drishti| drishti.from == Graha::Jupiter));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    pub fn cast_by(&self, graha: Graha) -> impl Iterator<Item = &Drishti> {
        self.relations
            .iter()
            .filter(move |relation| relation.from == graha)
    }

    /// The relations a body receives.
    ///
    /// ```
    /// use teistro_aspect::Aspects;
    /// use teistro_chart::sample;
    /// use teistro_core::catalogue::Graha;
    ///
    /// let aspects = Aspects::of(&sample::foundation()?, &sample::resolved().settings)?;
    /// assert!(aspects.on(Graha::Moon).all(|drishti| drishti.to == Graha::Moon));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    pub fn on(&self, graha: Graha) -> impl Iterator<Item = &Drishti> {
        self.relations
            .iter()
            .filter(move |relation| relation.to == graha)
    }

    /// The relation one body casts on another, if there is one.
    ///
    /// ```
    /// use teistro_aspect::Aspects;
    /// use teistro_chart::sample;
    /// use teistro_core::catalogue::Graha;
    ///
    /// let aspects = Aspects::of(&sample::foundation()?, &sample::resolved().settings)?;
    /// if let Some(drishti) = aspects.between(Graha::Saturn, Graha::Moon) {
    ///     assert_eq!((drishti.from, drishti.to), (Graha::Saturn, Graha::Moon));
    /// }
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn between(&self, from: Graha, to: Graha) -> Option<&Drishti> {
        self.relations
            .iter()
            .find(|relation| relation.from == from && relation.to == to)
    }

    /// Whether one body aspects another at all.
    ///
    /// ```
    /// use teistro_aspect::Aspects;
    /// use teistro_chart::sample;
    /// use teistro_core::catalogue::Graha;
    ///
    /// let aspects = Aspects::of(&sample::foundation()?, &sample::resolved().settings)?;
    /// assert_eq!(aspects.aspects(Graha::Mars, Graha::Venus), aspects.between(Graha::Mars, Graha::Venus).is_some());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn aspects(&self, from: Graha, to: Graha) -> bool {
        self.between(from, to).is_some()
    }

    /// Whether one body aspects another fully.
    ///
    /// ```
    /// use teistro_aspect::Aspects;
    /// use teistro_chart::sample;
    /// use teistro_core::catalogue::Graha;
    ///
    /// let aspects = Aspects::of(&sample::foundation()?, &sample::resolved().settings)?;
    /// // A full aspect is an aspect.
    /// assert!(!aspects.aspects_fully(Graha::Mars, Graha::Venus) || aspects.aspects(Graha::Mars, Graha::Venus));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn aspects_fully(&self, from: Graha, to: Graha) -> bool {
        self.between(from, to)
            .is_some_and(|relation| relation.strength.is_full())
    }

    /// The strongest relation on a body, or `None` when nothing looks at
    /// it.
    ///
    /// ```
    /// use teistro_aspect::Aspects;
    /// use teistro_chart::sample;
    /// use teistro_core::catalogue::Graha;
    ///
    /// let aspects = Aspects::of(&sample::foundation()?, &sample::resolved().settings)?;
    /// if let Some(strongest) = aspects.strongest_on(Graha::Moon) {
    ///     assert!(aspects.on(Graha::Moon).all(|drishti| drishti.strength <= strongest.strength));
    /// }
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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
    ///
    /// ```
    /// use teistro_aspect::Aspects;
    /// use teistro_chart::sample;
    ///
    /// let aspects = Aspects::of(&sample::foundation()?, &sample::resolved().settings)?;
    /// for pair in aspects.mutual() {
    ///     assert!(aspects.aspects(pair.first, pair.second) && aspects.aspects(pair.second, pair.first));
    /// }
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    pub fn mutual(&self) -> impl Iterator<Item = Mutual> {
        mutual_pairs(&self.relations)
    }

    /// The bodies sharing a sign with this one, which is what the
    /// tradition means by a conjunction.
    ///
    /// ```
    /// use teistro_aspect::Aspects;
    /// use teistro_chart::sample;
    /// use teistro_core::catalogue::Graha;
    ///
    /// let aspects = Aspects::of(&sample::foundation()?, &sample::resolved().settings)?;
    /// // Rahu and Ketu are never in one sign.
    /// assert!(aspects.conjunct(Graha::Rahu).all(|other| other != Graha::Ketu));
    /// assert!(aspects.conjunct(Graha::Sun).all(|other| aspects.sign(other) == aspects.sign(Graha::Sun)));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    pub fn conjunct(&self, graha: Graha) -> impl Iterator<Item = Graha> {
        let here = self.sign(graha);
        self.placed.iter().filter_map(move |other| {
            (other.graha != graha && Some(other.sign) == here).then_some(other.graha)
        })
    }

    /// Whether one body aspects another under the **rashi** drishti,
    /// which is a relation between their signs and is always mutual.
    ///
    /// ```
    /// use teistro_aspect::Aspects;
    /// use teistro_chart::sample;
    /// use teistro_core::catalogue::Graha;
    ///
    /// let aspects = Aspects::of(&sample::foundation()?, &sample::resolved().settings)?;
    /// // The rashi drishti is always mutual.
    /// assert_eq!(aspects.rashi_aspects(Graha::Sun, Graha::Saturn), aspects.rashi_aspects(Graha::Saturn, Graha::Sun));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn rashi_aspects(&self, from: Graha, to: Graha) -> bool {
        match (self.sign(from), self.sign(to)) {
            (Some(from), Some(to)) => rashi::aspects(from, to),
            _ => false,
        }
    }

    /// The sign a body stands in.
    ///
    /// ```
    /// use teistro_aspect::Aspects;
    /// use teistro_chart::sample;
    /// use teistro_core::catalogue::{Graha, Rashi};
    ///
    /// let foundation = sample::foundation()?;
    /// let aspects = Aspects::of(&foundation, &sample::resolved().settings)?;
    /// let moon = foundation.graha(Graha::Moon).map(|moon| u16::from(moon.sign_index()));
    /// assert_eq!(aspects.sign(Graha::Moon), moon.and_then(Rashi::from_id));
    /// // Only the nine are placed.
    /// assert_eq!(aspects.sign(Graha::Uranus), None);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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
#[cfg_attr(coverage_nightly, coverage(off))]
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
