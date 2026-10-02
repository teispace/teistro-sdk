//! The almuten: the planet with the most dignities at a place, over a
//! list of places, or in the whole figure, and Lilly's Part of Fortune,
//! one of the places a lord of the geniture is sought in
//! (`03-design/essential-dignities.md` §The almuten).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;

use crate::dignity::{CHALDEAN_ORDER, DignityRules, Scores, Sect, essential_dignity};
use crate::fortitude::Fortitudes;

/// What of a place an almuten's dignities are counted from (crux C218).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum PlaceReading {
    /// The place's degree: all five of Lilly's dignities, house,
    /// exaltation, triplicity, term and face. The default, because two of
    /// the five are held only by a degree.
    #[default]
    Degree,
    /// The place's sign: its house, exaltation and triplicity, the three a
    /// sign holds whole. Lilly's wording for a house, "the Signe ... upon
    /// the Cusp" (p. 49).
    Sign,
}

/// How the Part of Fortune is taken by night (crux C220).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum FortuneRule {
    /// Lilly's (pp. 143–144): the ascendant plus the Moon less the Sun, by
    /// day or night, which he gives as Ptolemy's. The default.
    #[default]
    DayAndNight,
    /// The rule Lilly reports and sets aside: by night, the ascendant plus
    /// the Sun less the Moon. The Tajika Punya saham reverses the same way.
    ReversedByNight,
}

/// The Part of Fortune's longitude, in degrees in `[0, 360)`.
///
/// ```
/// use teistro_hellenistic::{FortuneRule, Sect, part_of_fortune};
///
/// // Lilly, pp. 143–144: the Moon at 21°18′ Virgo, the Sun at 4°18′
/// // Aries, 23°27′ Leo rising: Fortune at 10°27′ Aquarius.
/// let at = |sign: f64, deg: f64, min: f64| sign * 30.0 + deg + min / 60.0;
/// let fortune = part_of_fortune(
///     at(4.0, 23.0, 27.0),
///     at(0.0, 4.0, 18.0),
///     at(5.0, 21.0, 18.0),
///     Sect::Night,
///     FortuneRule::DayAndNight,
/// );
/// assert!((fortune - at(10.0, 10.0, 27.0)).abs() < 1e-9);
/// ```
#[must_use]
pub fn part_of_fortune(
    ascendant_deg: f64,
    sun_deg: f64,
    moon_deg: f64,
    sect: Sect,
    rule: FortuneRule,
) -> f64 {
    let (from, to) = match (rule, sect) {
        (FortuneRule::ReversedByNight, Sect::Night) => (moon_deg, sun_deg),
        _ => (sun_deg, moon_deg),
    };
    (ascendant_deg + to - from).rem_euclid(360.0)
}

/// Each of the seven's total, and which hold the most.
///
/// An almuten is a ranking, not a planet: Lilly gives no rule for a tie,
/// so every planet tied at the top is an almuten (crux C219), and Chapter
/// CV's "partaker" is the next total down.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Almuten {
    /// The seven's totals, in the Chaldean order, Saturn first.
    pub totals: [i16; 7],
}

impl Almuten {
    /// A planet's total; `None` for a graha outside the seven.
    #[must_use]
    pub fn total(&self, planet: Graha) -> Option<i16> {
        CHALDEAN_ORDER
            .iter()
            .zip(self.totals)
            .find_map(|(&each, total)| (each == planet).then_some(total))
    }

    /// Every planet holding the greatest total, in the Chaldean order: one
    /// unless they tie.
    #[must_use]
    pub fn almutens(&self) -> Vec<Graha> {
        self.holding(self.totals.iter().copied().max())
    }

    /// Every planet holding the next total below the almutens': Chapter
    /// CV's "partaker". Empty when all seven tie.
    #[must_use]
    pub fn partakers(&self) -> Vec<Graha> {
        let top = self.totals.iter().copied().max();
        self.holding(self.totals.iter().copied().filter(|&t| Some(t) < top).max())
    }

    fn holding(&self, total: Option<i16>) -> Vec<Graha> {
        CHALDEAN_ORDER
            .into_iter()
            .zip(self.totals)
            .filter_map(|(planet, each)| (Some(each) == total).then_some(planet))
            .collect()
    }
}

/// The dignities each of the seven holds at one place: what it scores
/// there by `scores`, debilities left out, since no planet stands there.
/// A house's almuten is the almuten of its cusp.
///
/// # Errors
///
/// An invalid-argument error at `longitude` for one that is not a number.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_hellenistic::{DignityRules, PlaceReading, Scores, Sect, almuten_of};
///
/// // 5° Aries by day: the Sun's exaltation 4 and fiery triplicity 3
/// // outweigh Mars's house 5 and face 1, and Jupiter's term 2.
/// let at = almuten_of(5.0, Sect::Day, &DignityRules::LILLY, &Scores::LILLY, PlaceReading::Degree)?;
/// assert_eq!(at.almutens(), [Graha::Sun]);
/// assert_eq!(at.total(Graha::Mars), Some(6));
///
/// // By the sign alone the face and term drop out, and the Sun still leads.
/// let sign = almuten_of(5.0, Sect::Day, &DignityRules::LILLY, &Scores::LILLY, PlaceReading::Sign)?;
/// assert_eq!((sign.total(Graha::Sun), sign.total(Graha::Mars)), (Some(7), Some(5)));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
pub fn almuten_of(
    longitude_deg: f64,
    sect: Sect,
    rules: &DignityRules,
    scores: &Scores,
    reading: PlaceReading,
) -> Result<Almuten, Error> {
    let mut totals = [0; 7];
    for (total, planet) in totals.iter_mut().zip(CHALDEAN_ORDER) {
        let at = essential_dignity(planet, longitude_deg, sect, rules)?;
        // The three a sign holds whole first, then the two a degree holds.
        let held = [
            (at.house, scores.house),
            (at.exaltation, scores.exaltation),
            (at.triplicity, scores.triplicity),
            (at.term, scores.term),
            (at.face, scores.face),
        ];
        let counted = match reading {
            PlaceReading::Degree => held.len(),
            PlaceReading::Sign => 3,
        };
        *total = held
            .into_iter()
            .take(counted)
            .filter(|&(holds, _)| holds)
            .map(|(_, worth)| i16::from(worth))
            .sum();
    }
    Ok(Almuten { totals })
}

/// The seven's dignities summed over several places: Chapter CV's lord of
/// the geniture by the ascendant, mid-heaven, Sun, Moon and Part of
/// Fortune, or any list a reader names.
///
/// # Errors
///
/// An invalid-argument error at `places` for an empty list, which would
/// tie all seven at nothing, and at `places[i]` for a place that is not a
/// number.
pub fn almuten_of_places(
    places_deg: &[f64],
    sect: Sect,
    rules: &DignityRules,
    scores: &Scores,
    reading: PlaceReading,
) -> Result<Almuten, Error> {
    if places_deg.is_empty() {
        return Err(Error::invalid_arg("no place to count dignities in")
            .with_field("places")
            .with_hint(
                "Lilly's five are the ascendant, mid-heaven, Sun, Moon and Part of Fortune",
            ));
    }
    let mut totals = [0_i16; 7];
    for (i, &place) in places_deg.iter().enumerate() {
        let at = almuten_of(place, sect, rules, scores, reading)
            .map_err(|why| why.with_field(format!("places[{i}]")))?;
        for (total, each) in totals.iter_mut().zip(at.totals) {
            *total += each;
        }
    }
    Ok(Almuten { totals })
}

impl Fortitudes {
    /// The almuten of the figure, Lilly's lord of the geniture: each
    /// planet's [`net`](Fortitudes::net), essential and accidental, "most
    /// powerfull in the whole Scheame" (p. 49).
    #[must_use]
    pub fn almuten(&self) -> Almuten {
        let mut totals = [0; 7];
        for (total, planet) in totals.iter_mut().zip(CHALDEAN_ORDER) {
            *total = self.net(planet).unwrap_or_default();
        }
        Almuten { totals }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_core::catalogue::Graha;

    use super::{
        Almuten, FortuneRule, PlaceReading, almuten_of, almuten_of_places, part_of_fortune,
    };
    use crate::dignity::{DignityRules, Scores, Sect};

    const LILLY: (&DignityRules, &Scores) = (&DignityRules::LILLY, &Scores::LILLY);

    #[test]
    fn by_night_the_reversal_takes_the_suns_distance_from_the_moon() {
        let (asc, sun, moon) = (100.0, 10.0, 40.0);
        let lilly = |sect| part_of_fortune(asc, sun, moon, sect, FortuneRule::DayAndNight);
        let reversed = |sect| part_of_fortune(asc, sun, moon, sect, FortuneRule::ReversedByNight);
        assert_eq!(lilly(Sect::Day), 130.0);
        assert_eq!(lilly(Sect::Night), 130.0);
        assert_eq!(reversed(Sect::Day), 130.0);
        assert_eq!(reversed(Sect::Night), 70.0);
        // Lilly's check (p. 144): at the full Moon, Fortune in the seventh.
        assert_eq!(
            part_of_fortune(0.0, 0.0, 180.0, Sect::Day, FortuneRule::DayAndNight),
            180.0
        );
        assert_eq!(
            part_of_fortune(10.0, 350.0, 0.0, Sect::Day, FortuneRule::DayAndNight),
            20.0
        );
    }

    #[test]
    fn a_tie_names_every_almuten_and_the_partakers_below() {
        let tied = Almuten {
            totals: [3, 7, 7, 0, 2, 3, 0],
        };
        assert_eq!(tied.almutens(), [Graha::Jupiter, Graha::Mars]);
        assert_eq!(tied.partakers(), [Graha::Saturn, Graha::Mercury]);
        let level = Almuten { totals: [1; 7] };
        assert_eq!(level.almutens().len(), 7);
        assert_eq!(level.partakers(), []);
        assert_eq!(tied.total(Graha::Rahu), None);
    }

    #[test]
    fn the_sign_drops_the_term_and_the_face() {
        // 26° Capricorn by night: Saturn's house 5 and term 2 (Lilly's
        // last term, 25°–30°); Mars's exaltation 4; the Moon's earthy
        // triplicity by night 3; the Sun's face, the third decan's, 1.
        let (rules, scores) = LILLY;
        let at = |reading| almuten_of(296.0, Sect::Night, rules, scores, reading).unwrap();
        let (degree, sign) = (at(PlaceReading::Degree), at(PlaceReading::Sign));
        assert_eq!(degree.almutens(), [Graha::Saturn]);
        assert_eq!(degree.total(Graha::Saturn), Some(7));
        assert_eq!(sign.total(Graha::Saturn), Some(5));
        assert_eq!(
            (degree.total(Graha::Sun), sign.total(Graha::Sun)),
            (Some(1), Some(0))
        );
        for (planet, (whole, held)) in crate::CHALDEAN_ORDER
            .into_iter()
            .zip(sign.totals.into_iter().zip(degree.totals))
        {
            assert!(
                whole <= held,
                "{planet:?}: the sign gave {whole}, the degree {held}"
            );
        }
        assert_ne!(sign.totals, degree.totals);
    }

    #[test]
    fn places_sum_and_a_bad_list_is_refused_where_it_is_bad() {
        let (rules, scores) = LILLY;
        let one = almuten_of(5.0, Sect::Day, rules, scores, PlaceReading::Degree).unwrap();
        let twice =
            almuten_of_places(&[5.0, 5.0], Sect::Day, rules, scores, PlaceReading::Degree).unwrap();
        for (single, double) in one.totals.into_iter().zip(twice.totals) {
            assert_eq!(double, 2 * single);
        }
        let empty =
            almuten_of_places(&[], Sect::Day, rules, scores, PlaceReading::Degree).unwrap_err();
        assert_eq!(empty.field(), Some("places"));
        let nan = almuten_of_places(
            &[5.0, f64::NAN],
            Sect::Day,
            rules,
            scores,
            PlaceReading::Degree,
        )
        .unwrap_err();
        assert_eq!(nan.field(), Some("places[1]"));
    }
}
