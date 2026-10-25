//! The almuten: the planet with the most dignities at a place, over a
//! list of places, or in the whole figure
//! (`03-design/essential-dignities.md` §The almuten).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;

use crate::dignity::{CHALDEAN_ORDER, DignityRules, Scores, Sect, essential_dignity};
use crate::lot::{FortuneRule, part_of_fortune};

/// What of a place an almuten's dignities are counted from (crux C218).
///
/// ```
/// use teistro_hellenistic::{DignityRules, PlaceReading, Scores, Sect, almuten_of};
///
/// use teistro_core::catalogue::Graha;
///
/// // 5° Libra by day: Saturn's exaltation, triplicity and term, Venus's house.
/// let degree = almuten_of(185.0, Sect::Day, &DignityRules::LILLY, &Scores::LILLY, PlaceReading::Degree)?;
/// assert_eq!((degree.total(Graha::Saturn), degree.total(Graha::Venus)), (Some(9), Some(5)));
/// // The sign alone drops Saturn's term.
/// let sign = almuten_of(185.0, Sect::Day, &DignityRules::LILLY, &Scores::LILLY, PlaceReading::Sign)?;
/// assert_eq!(sign.total(Graha::Saturn), Some(7));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
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

/// Each of the seven's total, and which hold the most.
///
/// An almuten is a ranking, not a planet: Lilly gives no rule for a tie,
/// so every planet tied at the top is an almuten (crux C219), and Chapter
/// CV's "partaker" is the next total down.
///
/// ```
/// use teistro_hellenistic::{DignityRules, PlaceReading, Scores, Sect, almuten_of};
///
/// use teistro_core::catalogue::Graha;
///
/// // 5° Aries by day, Saturn first: Jupiter's term 2, Mars 6, the Sun 7.
/// let at = almuten_of(5.0, Sect::Day, &DignityRules::LILLY, &Scores::LILLY, PlaceReading::Degree)?;
/// assert_eq!(at.totals, [0, 2, 6, 7, 0, 0, 0]);
/// # let _ = Graha::Sun;
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
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
    ///
    /// ```
    /// use teistro_hellenistic::{DignityRules, PlaceReading, Scores, Sect, almuten_of};
    ///
    /// use teistro_core::catalogue::Graha;
    ///
    /// let at = almuten_of(5.0, Sect::Day, &DignityRules::LILLY, &Scores::LILLY, PlaceReading::Degree)?;
    /// assert_eq!(at.total(Graha::Sun), Some(7));
    /// assert_eq!(at.total(Graha::Rahu), None);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn total(&self, planet: Graha) -> Option<i16> {
        CHALDEAN_ORDER
            .iter()
            .zip(self.totals)
            .find_map(|(&each, total)| (each == planet).then_some(total))
    }

    /// Every planet holding the greatest total, in the Chaldean order: one
    /// unless they tie.
    ///
    /// ```
    /// use teistro_hellenistic::{DignityRules, PlaceReading, Scores, Sect, almuten_of};
    ///
    /// use teistro_core::catalogue::Graha;
    ///
    /// // Nothing scores, so all seven tie at the top.
    /// let nothing = Scores::new(0, 0, 0, 0, 0, 0, 0, 0);
    /// let at = almuten_of(5.0, Sect::Day, &DignityRules::LILLY, &nothing, PlaceReading::Degree)?;
    /// assert_eq!(at.almutens().len(), 7);
    /// let lilly = almuten_of(5.0, Sect::Day, &DignityRules::LILLY, &Scores::LILLY, PlaceReading::Degree)?;
    /// assert_eq!(lilly.almutens(), [Graha::Sun]);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn almutens(&self) -> Vec<Graha> {
        self.holding(self.totals.iter().copied().max())
    }

    /// Every planet holding the next total below the almutens': Chapter
    /// CV's "partaker". Empty when all seven tie.
    ///
    /// ```
    /// use teistro_hellenistic::{DignityRules, PlaceReading, Scores, Sect, almuten_of};
    ///
    /// use teistro_core::catalogue::Graha;
    ///
    /// // The Sun leads at 5° Aries, and Mars partakes.
    /// let at = almuten_of(5.0, Sect::Day, &DignityRules::LILLY, &Scores::LILLY, PlaceReading::Degree)?;
    /// assert_eq!(at.partakers(), [Graha::Mars]);
    /// let nothing = Scores::new(0, 0, 0, 0, 0, 0, 0, 0);
    /// assert!(almuten_of(5.0, Sect::Day, &DignityRules::LILLY, &nothing, PlaceReading::Degree)?.partakers().is_empty());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_hellenistic::{DignityRules, PlaceReading, Scores, Sect, almuten_of_places};
///
/// // 5° Aries and 5° Libra by day: Saturn's 9 outweighs the Sun's 7.
/// let both = almuten_of_places(&[5.0, 185.0], Sect::Day, &DignityRules::LILLY, &Scores::LILLY, PlaceReading::Degree)?;
/// assert_eq!(both.almutens(), [Graha::Saturn]);
/// let none = almuten_of_places(&[], Sect::Day, &DignityRules::LILLY, &Scores::LILLY, PlaceReading::Degree).unwrap_err();
/// assert_eq!(none.field(), Some("places"));
/// let broken = almuten_of_places(&[5.0, f64::NAN], Sect::Day, &DignityRules::LILLY, &Scores::LILLY, PlaceReading::Degree).unwrap_err();
/// assert_eq!(broken.field(), Some("places[1]"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
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

/// How a chart's almutens are read: a place by its degree or its sign,
/// and Fortune by night.
///
/// The default is Lilly's: the degree, and Fortune the same by day and
/// night.
///
/// ```
/// use teistro_hellenistic::{AlmutenRules, FortuneRule, PlaceReading};
///
/// assert_eq!(AlmutenRules::default(), AlmutenRules::LILLY);
/// assert_eq!((AlmutenRules::LILLY.place, AlmutenRules::LILLY.fortune), (PlaceReading::Degree, FortuneRule::DayAndNight));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default)]
#[non_exhaustive]
pub struct AlmutenRules {
    /// What of a place its dignities are counted from (C218).
    pub place: PlaceReading,
    /// How Fortune is taken by night (C220).
    pub fortune: FortuneRule,
}

impl AlmutenRules {
    /// Lilly's: the degree, and Fortune the same by day and night.
    ///
    /// ```
    /// use teistro_hellenistic::{AlmutenRules, FortuneRule};
    ///
    /// // Lilly's Fortune is the same by day and night, unlike the lots' default.
    /// assert_eq!(AlmutenRules::LILLY.fortune, FortuneRule::DayAndNight);
    /// ```
    pub const LILLY: AlmutenRules = AlmutenRules {
        place: PlaceReading::Degree,
        fortune: FortuneRule::DayAndNight,
    };

    /// These rules with a place read otherwise.
    ///
    /// ```
    /// use teistro_hellenistic::{AlmutenRules, PlaceReading};
    ///
    /// assert_eq!(AlmutenRules::LILLY.with_place(PlaceReading::Sign).place, PlaceReading::Sign);
    /// ```
    #[must_use]
    pub const fn with_place(mut self, place: PlaceReading) -> AlmutenRules {
        self.place = place;
        self
    }

    /// These rules with Fortune taken otherwise by night.
    ///
    /// ```
    /// use teistro_hellenistic::{AlmutenRules, FortuneRule};
    ///
    /// assert_eq!(AlmutenRules::LILLY.with_fortune(FortuneRule::ReversedByNight).fortune, FortuneRule::ReversedByNight);
    /// ```
    #[must_use]
    pub const fn with_fortune(mut self, fortune: FortuneRule) -> AlmutenRules {
        self.fortune = fortune;
        self
    }
}

/// A chart's almutens, three ways, with the rules that made them.
///
/// ```
/// # use teistro_hellenistic::ChartSky;
/// #
/// # let chart = ChartSky {
/// #     saturn_deg: 285.0, jupiter_deg: 245.0, mars_deg: 190.0, sun_deg: 15.0,
/// #     venus_deg: 25.0, mercury_deg: 40.0, moon_deg: 100.0,
/// #     sun_altitude_deg: 30.0, moon_altitude_deg: -20.0, daylight: true,
/// # };
/// # use teistro_core::catalogue::HouseSystem;
/// # use teistro_hellenistic::AccidentalSky;
/// #
/// # let sky = AccidentalSky {
/// #     speeds_deg_per_day: [0.03, 0.08, 0.5, 0.99, 1.2, 1.6, 13.0],
/// #     houses: HouseSystem::Equal,
/// #     cusps_deg: std::array::from_fn(|k| k as f64 * 30.0),
/// #     ascendant_deg: 0.0,
/// #     midheaven_deg: 270.0,
/// #     north_node_deg: 250.0,
/// #     regulus_deg: 150.0, spica_deg: 204.0, algol_deg: 56.0,
/// # };
/// use teistro_hellenistic::{AlmutenRules, FortitudeRequest, PlaceReading, almuten_of};
///
/// let read = FortitudeRequest::default().read(&chart, &sky)?;
/// let almutens = read.almutens;
/// // Fortune by day: the ascendant 0° plus the Moon's 85° from the Sun.
/// assert_eq!((almutens.rules, almutens.fortune_deg), (AlmutenRules::LILLY, 85.0));
/// // A house's is its cusp's.
/// let first = almuten_of(sky.cusps_deg[0], read.dignities.sect, &read.dignities.rules, &read.dignities.scores, PlaceReading::Degree)?;
/// assert_eq!(almutens.houses[0], first);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Almutens {
    /// The rules they were read under.
    pub rules: AlmutenRules,
    /// The Part of Fortune's longitude, one of the five places.
    pub fortune_deg: f64,
    /// Lilly's almuten of the figure: each planet's net, essential and
    /// accidental, "most powerfull in the whole Scheame" (p. 49).
    pub figure: Almuten,
    /// Chapter CV's: each planet's dignities over the ascendant,
    /// midheaven, Sun, Moon and Part of Fortune.
    pub places: Almuten,
    /// Each house's, of its cusp, first to twelfth.
    pub houses: [Almuten; 12],
}

/// What the almutens are read from beyond the nets: the angles, the
/// luminaries, the cusps, and the chart's sect and essential tables.
pub(crate) struct AlmutenSky<'a> {
    pub(crate) ascendant_deg: f64,
    pub(crate) midheaven_deg: f64,
    pub(crate) sun_deg: f64,
    pub(crate) moon_deg: f64,
    pub(crate) moon_altitude_deg: f64,
    pub(crate) cusps_deg: &'a [f64; 12],
    pub(crate) sect: Sect,
    pub(crate) rules: &'a DignityRules,
    pub(crate) scores: &'a Scores,
}

impl AlmutenRules {
    /// The three almutens of a chart whose nets are `nets`, in the
    /// Chaldean order.
    pub(crate) fn read(self, nets: [i16; 7], sky: &AlmutenSky<'_>) -> Result<Almutens, Error> {
        let fortune_deg = part_of_fortune(
            sky.ascendant_deg,
            sky.sun_deg,
            sky.moon_deg,
            self.fortune.reverses(sky.sect, sky.moon_altitude_deg),
        );
        let places = [
            sky.ascendant_deg,
            sky.midheaven_deg,
            sky.sun_deg,
            sky.moon_deg,
            fortune_deg,
        ];
        let of = |cusp: f64| almuten_of(cusp, sky.sect, sky.rules, sky.scores, self.place);
        let mut houses = [Almuten { totals: [0; 7] }; 12];
        for (house, &cusp) in houses.iter_mut().zip(sky.cusps_deg) {
            *house = of(cusp)?;
        }
        Ok(Almutens {
            rules: self,
            fortune_deg,
            figure: Almuten { totals: nets },
            places: almuten_of_places(&places, sky.sect, sky.rules, sky.scores, self.place)?,
            houses,
        })
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_core::catalogue::Graha;

    use super::{Almuten, PlaceReading, almuten_of, almuten_of_places};
    use crate::dignity::{DignityRules, Scores, Sect};

    const LILLY: (&DignityRules, &Scores) = (&DignityRules::LILLY, &Scores::LILLY);

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
