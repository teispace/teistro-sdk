//! Reception: two planets each in a dignity of the other's
//! (`03-design/essential-dignities.md` §Reception).
//!
//! Lilly (p. 112) calls it reception when two planets "are in each others
//! dignity", by house the strongest, and "by triplicity terme or face, or
//! any essentiall dignity". Whose dignity a planet stands in is the other
//! planet's [`essential_dignity`](crate::essential_dignity) at its place, so
//! reception needs no table of its own.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Graha;

use crate::dignity::EssentialDignity;

/// One of the five essential dignities, the kinds a reception is by.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum DignityKind {
    /// The sign is the planet's house.
    House,
    /// The sign is its exaltation.
    Exaltation,
    /// It rules the sign's triplicity in a chart of this sect.
    Triplicity,
    /// The degree is in its term.
    Term,
    /// The degree is in its face.
    Face,
}

impl DignityKind {
    /// The five, strongest first, as Lilly scores them (p. 115).
    pub const ALL: [DignityKind; 5] = [
        DignityKind::House,
        DignityKind::Exaltation,
        DignityKind::Triplicity,
        DignityKind::Term,
        DignityKind::Face,
    ];
}

impl EssentialDignity {
    /// Whether this dignity holds.
    ///
    /// ```
    /// use teistro_core::catalogue::Graha;
    /// use teistro_hellenistic::{DignityKind, DignityRules, Sect, essential_dignity};
    ///
    /// // Mars at 5° Aries: his house and his face.
    /// let mars = essential_dignity(Graha::Mars, 5.0, Sect::Day, &DignityRules::LILLY)?;
    /// let held: Vec<_> = DignityKind::ALL.into_iter().filter(|&kind| mars.holds(kind)).collect();
    /// assert_eq!(held, [DignityKind::House, DignityKind::Face]);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub const fn holds(&self, kind: DignityKind) -> bool {
        match kind {
            DignityKind::House => self.house,
            DignityKind::Exaltation => self.exaltation,
            DignityKind::Triplicity => self.triplicity,
            DignityKind::Term => self.term,
            DignityKind::Face => self.face,
        }
    }
}

/// Two planets each standing in at least one of the other's five
/// dignities.
///
/// Each side is reported whole, so a reception by the same kind both ways
/// ([`mutual`](Reception::mutual), Lilly's examples) and one by different
/// kinds (a mixed reception, which Lilly neither names nor scores) are
/// both read off the same record (C210).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Reception {
    /// The two, in the Chaldean order.
    pub planets: [Graha; 2],
    /// The second's dignities at the first's place: what the first stands
    /// in of the second's.
    pub first_in: EssentialDignity,
    /// The first's dignities at the second's place.
    pub second_in: EssentialDignity,
}

impl Reception {
    /// Whether each stands in the other's dignity of this kind.
    #[must_use]
    pub const fn mutual_by(&self, kind: DignityKind) -> bool {
        self.first_in.holds(kind) && self.second_in.holds(kind)
    }

    /// The kinds each stands in of the other's, strongest first; empty for
    /// a mixed reception.
    pub fn mutual(&self) -> impl Iterator<Item = DignityKind> + '_ {
        DignityKind::ALL
            .into_iter()
            .filter(|&kind| self.mutual_by(kind))
    }
}

/// Every reception among planets standing at these places, given each
/// one's dignities at a longitude; pairs in the Chaldean order of
/// `planets`.
pub(crate) fn receptions<E>(
    planets: &[(Graha, f64)],
    mut dignity_at: impl FnMut(Graha, f64) -> Result<EssentialDignity, E>,
) -> Result<Vec<Reception>, E> {
    let mut found = Vec::new();
    for (k, &(first, first_deg)) in planets.iter().enumerate() {
        for &(second, second_deg) in planets.iter().skip(k + 1) {
            let first_in = dignity_at(second, first_deg)?;
            let second_in = dignity_at(first, second_deg)?;
            if !first_in.peregrine() && !second_in.peregrine() {
                found.push(Reception {
                    planets: [first, second],
                    first_in,
                    second_in,
                });
            }
        }
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index their own results"
    )]

    use super::{DignityKind, Reception, receptions};
    use crate::{DignityRules, Sect, essential_dignity};
    use teistro_core::catalogue::Graha;
    use teistro_core::error::Error;

    fn between(first: (Graha, f64), second: (Graha, f64), sect: Sect) -> Vec<Reception> {
        receptions(&[first, second], |planet, at| {
            essential_dignity(planet, at, sect, &DignityRules::LILLY)
        })
        .unwrap()
    }

    fn mutual(found: &[Reception]) -> Vec<DignityKind> {
        found.iter().flat_map(Reception::mutual).collect()
    }

    /// Lilly, p. 112: "☉ in ♈ and ♂ in ♌, here is reception of these two
    /// Planets by Houses".
    #[test]
    fn the_sun_in_aries_and_mars_in_leo_by_house() {
        for sect in [Sect::Day, Sect::Night] {
            let found = between((Graha::Mars, 125.0), (Graha::Sun, 5.0), sect);
            assert!(mutual(&found).contains(&DignityKind::House), "{sect:?}");
        }
    }

    /// p. 112: "♀ in ♈, and ☉ in ♉, here is reception by triplicity, if
    /// the Question or Nativity be by day"; by night fire is Jupiter's and
    /// earth the Moon's, and neither is received.
    #[test]
    fn venus_in_aries_and_the_sun_in_taurus_by_triplicity_by_day_only() {
        let day = between((Graha::Sun, 45.0), (Graha::Venus, 15.0), Sect::Day);
        assert_eq!(mutual(&day), [DignityKind::Triplicity]);
        let night = between((Graha::Sun, 45.0), (Graha::Venus, 15.0), Sect::Night);
        assert!(!mutual(&night).contains(&DignityKind::Triplicity));
    }

    /// p. 112: "♀ in the 24. of ♈ and ♂ in the 16. of ♊, here is
    /// reception by terme, ♂ being in the terms of ♀, and she in his".
    #[test]
    fn venus_at_24_aries_and_mars_at_16_gemini_by_term() {
        let found = between((Graha::Mars, 75.5), (Graha::Venus, 23.5), Sect::Day);
        assert_eq!(found.len(), 1);
        assert!(found[0].mutual_by(DignityKind::Term));
        assert_eq!(found[0].planets, [Graha::Mars, Graha::Venus]);
    }

    /// Saturn in Libra and Venus in Capricorn, each in the other's house:
    /// mutual by house, and Saturn's exaltation is no part of it.
    #[test]
    fn saturn_in_libra_and_venus_in_capricorn_by_house() {
        let found = between((Graha::Saturn, 190.0), (Graha::Venus, 275.0), Sect::Day);
        assert_eq!(mutual(&found), [DignityKind::House]);
    }

    /// Each side whole: Jupiter at 5° Aries is in Mars's house and face,
    /// Mars at 5° Cancer in Jupiter's exaltation. A mixed reception,
    /// reported and mutual by nothing.
    #[test]
    fn a_mixed_reception_is_reported_and_mutual_by_nothing() {
        let found = between((Graha::Jupiter, 5.0), (Graha::Mars, 95.0), Sect::Day);
        assert_eq!(found.len(), 1);
        let one = found[0];
        assert_eq!(one.planets, [Graha::Jupiter, Graha::Mars]);
        assert!(one.first_in.house && one.first_in.face && !one.first_in.exaltation);
        assert!(one.second_in.exaltation && !one.second_in.house);
        assert_eq!(mutual(&found), []);
    }

    /// One way only is no reception: Mars in Leo, the Sun in Libra.
    #[test]
    fn one_way_is_not_reported() {
        assert_eq!(
            between((Graha::Mars, 125.0), (Graha::Sun, 185.0), Sect::Day),
            []
        );
    }

    #[test]
    fn a_refusal_is_passed_back() {
        let why = receptions(&[(Graha::Sun, 0.0), (Graha::Moon, 1.0)], |_, _| {
            Err::<crate::EssentialDignity, _>(Error::invalid_arg("no").with_field("here"))
        })
        .unwrap_err();
        assert_eq!(why.field(), Some("here"));
    }
}
