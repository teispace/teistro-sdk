//! A dasha as a chart document carries it: what it was computed from, its
//! balance, and its periods to the depth the settings ask for.
//!
//! The document holds rows and not the kernel, so a consumer in any
//! language reads the timeline as data. A caller who wants deeper periods,
//! or the chain at an instant, rebuilds the [`DashaCursor`] the reading came
//! from and asks it: the reading carries everything that takes.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{DashaSystem, Graha, Nakshatra, Rashi};
use teistro_core::interval::Interval;
use teistro_core::quantity::Depth;
use teistro_core::settings::{DecennialDivision, FirdariaNodes, GrahaDivision};

use crate::balance::BalanceAtBirth;
use crate::decennials::{DECENNIAL_STARS, DecennialDasha};
use crate::definition::DashaDefinition;
use crate::firdaria::{DESCENDING, FirdariaDasha};
use crate::graha::GrahaDasha;
use crate::kalachakra::{KalachakraDasha, KalachakraRules};
use crate::rashi::{RashiDasha, RashiDefinition, RashiRules};
use crate::releasing::{ProfectionDasha, ReleasingDasha};
use crate::row::{DashaName, UduDefinition};
use crate::tree::{Dasha, Period, Rules, Timeline};

/// One dasha of a chart.
///
/// A nakshatra-seeded dasha carries the nakshatra that seeds it and its
/// balance at birth; a sign-based one has neither, and its periods carry
/// their signs.
///
/// ```
/// use teistro_core::angle::Nas;
/// use teistro_core::catalogue::{DashaSystem, Graha};
/// use teistro_core::quantity::{Degrees, Depth, JulianDay};
/// use teistro_core::settings::root;
/// use teistro_dasha::{Birth, Dasha, Rules, Timeline, VIMSHOTTARI};
///
/// // A Moon a third of the way into Anuradha, Saturn's nakshatra: two thirds
/// // of his nineteen years remain at birth.
/// let moon = Nas::from_degrees(Degrees::try_new(16.0 * 360.0 / 27.0 + 360.0 / 81.0)?);
/// let birth = Birth { instant: JulianDay::literal(2_447_995.5), moon, moon_span: None };
/// let dasha = Dasha::new(&VIMSHOTTARI, &birth, Rules::of(&root().dasha, DashaSystem::Vimshottari))?;
/// // Saturn's balance ends 12 years 8 months after birth.
/// let saturn_ends = 2_447_995.5 + 19.0 * 365.25 * 2.0 / 3.0;
/// use teistro_core::catalogue::Nakshatra;
/// use teistro_dasha::DashaReading;
///
/// // Nine mahadashas, each with its nine antardashas, from Saturn's balance.
/// let reading = DashaReading::of(&dasha, Depth::try_new(2)?, None);
/// assert_eq!((reading.seed, reading.first_lord), (Some(Nakshatra::Anuradha), Graha::Saturn));
/// assert_eq!(reading.periods.len(), 9 + 9 * 9);
/// assert!((reading.periods[0].interval.to.get() - saturn_ends).abs() < 1e-6);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DashaReading {
    /// Which system: a catalogued one, or a consumer's by the key it was
    /// registered under.
    pub system: DashaName,
    /// A consumer's system's definition, of whichever kernel runs it,
    /// carried so a stored document rebuilds its periods whatever its
    /// context has registered since; nothing for a catalogued system.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub definition: Option<DashaDefinition>,
    /// The choices it was computed under. A sign-based dasha reads only the
    /// year length and what follows the cycle.
    pub rules: Rules,
    /// The Kalachakra's own choices, when it is the Kalachakra.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub kalachakra: Option<KalachakraRules>,
    /// A sign-based dasha's dual-lord and start readings. A document stored
    /// before they were recorded was computed under the recording engine's,
    /// which is what rebuilding it assumes when this is absent.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub rashi: Option<RashiRules>,
    /// The nakshatra the Moon stood in, which seeds a nakshatra-seeded
    /// dasha; nothing for a sign-based one.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub seed: Option<Nakshatra>,
    /// The lord it starts with.
    pub first_lord: Graha,
    /// Whether the seed lay outside a conditional system's nakshatras, and
    /// started at the first lord because the rules let it.
    pub overflow: bool,
    /// The Moon's stay in its nakshatra, when the balance was temporal.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub moon_span: Option<Interval>,
    /// What remained of the first period at birth; nothing for a sign-based
    /// dasha, whose first period runs whole from birth.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub balance: Option<BalanceAtBirth>,
    /// How many levels the periods go down.
    pub depth: Depth,
    /// Every period of the birth cycle to `depth`, depth first in time
    /// order: a mahadasha, then its antardashas and theirs, then the next.
    pub periods: Vec<PeriodRow>,
}

/// One period, as a row.
///
/// ```
/// # use teistro_core::angle::Nas;
/// # use teistro_core::catalogue::{DashaSystem, Graha};
/// # use teistro_core::quantity::{Degrees, Depth, JulianDay};
/// # use teistro_core::settings::root;
/// # use teistro_dasha::{Birth, Dasha, Rules, Timeline, VIMSHOTTARI};
/// #
/// # // A Moon a third of the way into Anuradha, Saturn's nakshatra: two thirds
/// # // of his nineteen years remain at birth.
/// # let moon = Nas::from_degrees(Degrees::try_new(16.0 * 360.0 / 27.0 + 360.0 / 81.0)?);
/// # let birth = Birth { instant: JulianDay::literal(2_447_995.5), moon, moon_span: None };
/// # let dasha = Dasha::new(&VIMSHOTTARI, &birth, Rules::of(&root().dasha, DashaSystem::Vimshottari))?;
/// # // Saturn's balance ends 12 years 8 months after birth.
/// # let saturn_ends = 2_447_995.5 + 19.0 * 365.25 * 2.0 / 3.0;
/// # use teistro_dasha::DashaReading;
/// #
/// # let reading = DashaReading::of(&dasha, Depth::try_new(2)?, None);
///
/// // Saturn's mahadasha, then his own antardasha within it.
/// let antar = &reading.periods[1];
/// assert_eq!((antar.path.as_str(), antar.lord, antar.sign), ("0/0", Graha::Saturn, None));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PeriodRow {
    /// Where it sits: the place at each level, from the mahadasha down,
    /// joined by `/` (`2/5/3`).
    pub path: String,
    /// The sign it is the period of, in a sign-based dasha.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub sign: Option<Rashi>,
    /// Its lord.
    pub lord: Graha,
    /// When it runs.
    pub interval: Interval,
}

impl PeriodRow {
    /// How deep it sits: 1 for a mahadasha, 2 for an antardasha — the
    /// number of places in its [`path`](PeriodRow::path).
    ///
    /// ```
    /// # use teistro_core::angle::Nas;
    /// # use teistro_core::catalogue::{DashaSystem, Graha};
    /// # use teistro_core::quantity::{Degrees, Depth, JulianDay};
    /// # use teistro_core::settings::root;
    /// # use teistro_dasha::{Birth, Dasha, Rules, Timeline, VIMSHOTTARI};
    /// #
    /// # // A Moon a third of the way into Anuradha, Saturn's nakshatra: two thirds
    /// # // of his nineteen years remain at birth.
    /// # let moon = Nas::from_degrees(Degrees::try_new(16.0 * 360.0 / 27.0 + 360.0 / 81.0)?);
    /// # let birth = Birth { instant: JulianDay::literal(2_447_995.5), moon, moon_span: None };
    /// # let dasha = Dasha::new(&VIMSHOTTARI, &birth, Rules::of(&root().dasha, DashaSystem::Vimshottari))?;
    /// # // Saturn's balance ends 12 years 8 months after birth.
    /// # let saturn_ends = 2_447_995.5 + 19.0 * 365.25 * 2.0 / 3.0;
    /// # use teistro_dasha::DashaReading;
    /// #
    /// # let reading = DashaReading::of(&dasha, Depth::try_new(2)?, None);
    ///
    /// assert_eq!((reading.periods[0].level(), reading.periods[1].level()), (1, 2));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn level(&self) -> usize {
        self.path.split('/').count()
    }

    /// A period as a row.
    ///
    /// ```
    /// # use teistro_core::angle::Nas;
    /// # use teistro_core::catalogue::{DashaSystem, Graha};
    /// # use teistro_core::quantity::{Degrees, Depth, JulianDay};
    /// # use teistro_core::settings::root;
    /// # use teistro_dasha::{Birth, Dasha, Rules, Timeline, VIMSHOTTARI};
    /// #
    /// # // A Moon a third of the way into Anuradha, Saturn's nakshatra: two thirds
    /// # // of his nineteen years remain at birth.
    /// # let moon = Nas::from_degrees(Degrees::try_new(16.0 * 360.0 / 27.0 + 360.0 / 81.0)?);
    /// # let birth = Birth { instant: JulianDay::literal(2_447_995.5), moon, moon_span: None };
    /// # let dasha = Dasha::new(&VIMSHOTTARI, &birth, Rules::of(&root().dasha, DashaSystem::Vimshottari))?;
    /// # // Saturn's balance ends 12 years 8 months after birth.
    /// # let saturn_ends = 2_447_995.5 + 19.0 * 365.25 * 2.0 / 3.0;
    ///
    /// use teistro_dasha::PeriodRow;
    ///
    /// let first = dasha.mahadashas().next().ok_or("a mahadasha")?;
    /// let row = PeriodRow::of(&first);
    /// assert_eq!((row.path.as_str(), row.lord), ("0", Graha::Saturn));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn of(period: &Period) -> PeriodRow {
        PeriodRow {
            path: period.path.to_string(),
            sign: period.sign,
            lord: period.lord,
            interval: period.interval,
        }
    }
}

/// A graha dasha's seats and their sub-periods' shares, as a stored
/// reading holds them: what [`GrahaDasha::with_shares`](crate::GrahaDasha::with_shares)
/// rebuilds it from.
///
/// ```
/// # use teistro_core::catalogue::Graha;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::{AfterCycle, GrahaDivision, YearLength};
/// # use teistro_dasha::{GrahaDasha, NAISARGIKA_LORDS, Timeline};
/// #
/// # // The Naisargika dasha: its seven lords from birth, in proportion.
/// # let birth = JulianDay::literal(2_451_545.0);
/// # let natural = GrahaDasha::new(NAISARGIKA_LORDS.to_vec(), birth, YearLength::Julian36525, GrahaDivision::Proportional, AfterCycle::End)?;
/// # use teistro_core::catalogue::DashaSystem;
/// # use teistro_core::quantity::Depth;
/// # use teistro_core::settings::root;
/// # use teistro_dasha::{DashaReading, Rules};
/// #
/// # let mut rules = Rules::of(&root().dasha, DashaSystem::Naisargika);
/// # rules.year_length = YearLength::Julian36525;
/// # let reading = DashaReading::of_time_lord(DashaSystem::Naisargika, &natural, rules, Depth::try_new(2)?);
///
/// // Seven seats, the Moon's year first, each divided among the seven.
/// let stored = reading.graha_shares().ok_or("two levels stored")?;
/// assert_eq!(stored.seats.len(), 7);
/// assert_eq!((stored.seats[0].0, stored.seats[0].1), (Graha::Moon, None));
/// assert!((stored.seats[0].2 - 1.0).abs() < 1e-9);
/// assert_eq!(stored.shares[0].len(), 7);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct GrahaShares {
    /// Each seat's lord, the sign its period is of, and its years.
    pub seats: Vec<(Graha, Option<Rashi>, f64)>,
    /// Each seat's sub-periods: the seats they are of, with their parts.
    pub shares: Vec<Vec<(usize, f64)>>,
}

impl DashaReading {
    /// A nakshatra-seeded dasha's reading, its periods to `depth` levels.
    ///
    /// `moon_span` is the Moon's stay the balance read, when it read one.
    ///
    /// ```
    /// # use teistro_core::angle::Nas;
    /// # use teistro_core::catalogue::{DashaSystem, Graha};
    /// # use teistro_core::quantity::{Degrees, Depth, JulianDay};
    /// # use teistro_core::settings::root;
    /// # use teistro_dasha::{Birth, Dasha, Rules, Timeline, VIMSHOTTARI};
    /// #
    /// # // A Moon a third of the way into Anuradha, Saturn's nakshatra: two thirds
    /// # // of his nineteen years remain at birth.
    /// # let moon = Nas::from_degrees(Degrees::try_new(16.0 * 360.0 / 27.0 + 360.0 / 81.0)?);
    /// # let birth = Birth { instant: JulianDay::literal(2_447_995.5), moon, moon_span: None };
    /// # let dasha = Dasha::new(&VIMSHOTTARI, &birth, Rules::of(&root().dasha, DashaSystem::Vimshottari))?;
    /// # // Saturn's balance ends 12 years 8 months after birth.
    /// # let saturn_ends = 2_447_995.5 + 19.0 * 365.25 * 2.0 / 3.0;
    ///
    /// use teistro_dasha::DashaReading;
    ///
    /// let reading = DashaReading::of(&dasha, Depth::try_new(1)?, None);
    /// assert_eq!(reading.periods.len(), 9);
    /// assert!(reading.balance.is_some() && reading.moon_span.is_none());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn of(dasha: &Dasha, depth: Depth, moon_span: Option<Interval>) -> DashaReading {
        let row = dasha.row();
        let seat = dasha.seat();
        DashaReading {
            system: row.system.clone(),
            definition: None,
            rules: dasha.rules(),
            kalachakra: None,
            rashi: None,
            seed: Some(dasha.seed()),
            first_lord: row
                .lords
                .get(seat.lord)
                .map_or(Graha::Ketu, |lord| lord.graha),
            overflow: seat.overflow,
            moon_span,
            balance: Some(dasha.balance()),
            depth,
            periods: rows(dasha, depth),
        }
    }

    /// A consumer's system's reading: [`DashaReading::of`], carrying the
    /// definition the dasha was built from.
    ///
    /// ```
    /// use teistro_core::angle::Nas;
    /// use teistro_core::catalogue::Nakshatra;
    /// use teistro_core::quantity::{Degrees, JulianDay};
    /// use teistro_core::settings::root;
    /// use teistro_dasha::{Birth, Dasha, DashaDefinition, DashaReading, Rules, UduDefinition, VIMSHOTTARI};
    ///
    /// // A consumer's row: carried whole, so a stored document rebuilds it.
    /// let traya = UduDefinition {
    ///     lords: VIMSHOTTARI.lords.to_vec(),
    ///     ..UduDefinition::of("ACME_TRAYA", Nakshatra::Krittika)
    /// };
    /// let row = traya.row();
    /// let moon = Nas::from_degrees(Degrees::try_new(10.0)?);
    /// let birth = Birth { instant: JulianDay::literal(2_451_545.0), moon, moon_span: None };
    /// let rules = Rules::of_definition(&root().dasha, &DashaDefinition::from(traya.clone()));
    /// let dasha = Dasha::new(&row, &birth, rules)?;
    /// let reading = DashaReading::of_registered(&dasha, &traya, None);
    /// assert_eq!(reading.definition, Some(DashaDefinition::Udu(traya)));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn of_registered(
        dasha: &Dasha,
        definition: &UduDefinition,
        moon_span: Option<Interval>,
    ) -> DashaReading {
        DashaReading {
            definition: Some(DashaDefinition::Udu(definition.clone())),
            ..DashaReading::of(dasha, definition.depth, moon_span)
        }
    }

    /// A reading of a consumer's own sign-based system, carrying the
    /// definition it was built from, as a registered nakshatra-seeded one
    /// does: a stored document rebuilds its periods from what it was
    /// computed under and not from what a context has registered since.
    ///
    /// ```
    /// use teistro_core::catalogue::{DashaSystem, Dignity, Rashi};
    /// use teistro_core::quantity::JulianDay;
    /// use teistro_core::settings::{AfterCycle, YearLength, root};
    /// use teistro_dasha::rashi::{RashiChart, RashiDasha, RashiDefinition, RashiRules};
    /// use teistro_dasha::{DashaDefinition, DashaReading, Rules};
    ///
    /// let mine = RashiDefinition::of("ACME_CHARA");
    /// let chart = RashiChart {
    ///     lagna: Rashi::Aries,
    ///     arudha_lagna: Rashi::Aries,
    ///     navamsa_lagna: Rashi::Aries,
    ///     signs: [Rashi::Aries; 9],
    ///     dignities: [Dignity::Neutral; 9],
    ///     brahma: None,
    /// };
    /// let birth = JulianDay::literal(2_451_545.0);
    /// let dasha = RashiDasha::new(&mine.row(), &chart, birth, YearLength::Julian36525, AfterCycle::End, RashiRules::BPHS)?;
    /// let rules = Rules::of(&root().dasha, DashaSystem::Chara);
    /// let reading = DashaReading::of_registered_rashi(&dasha, &mine, rules);
    /// assert_eq!(reading.definition, Some(DashaDefinition::Rashi(mine)));
    /// assert_eq!(reading.periods.first().and_then(|row| row.sign), Some(Rashi::Aries));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn of_registered_rashi(
        dasha: &RashiDasha,
        definition: &RashiDefinition,
        rules: Rules,
    ) -> DashaReading {
        DashaReading {
            definition: Some(DashaDefinition::Rashi(definition.clone())),
            ..DashaReading::of_rashi(dasha, rules, definition.depth)
        }
    }

    /// A sign-based dasha's reading under `rules`, its periods to `depth`
    /// levels.
    ///
    /// ```
    /// # use teistro_core::quantity::JulianDay;
    /// # use teistro_core::catalogue::{Dignity, Rashi};
    /// # use teistro_core::settings::{AfterCycle, YearLength};
    /// # use teistro_dasha::rashi::{CHARA, RashiChart, RashiDasha, RashiRules};
    /// #
    /// # let chart = RashiChart {
    /// #     lagna: Rashi::Aries,
    /// #     arudha_lagna: Rashi::Leo,
    /// #     navamsa_lagna: Rashi::Sagittarius,
    /// #     signs: [
    /// #         Rashi::Leo, Rashi::Cancer, Rashi::Capricorn, Rashi::Leo, Rashi::Sagittarius,
    /// #         Rashi::Virgo, Rashi::Aquarius, Rashi::Gemini, Rashi::Sagittarius,
    /// #     ],
    /// #     dignities: [Dignity::Neutral; 9],
    /// #     brahma: None,
    /// # };
    /// # let birth = JulianDay::literal(2_451_545.0);
    /// # let chara = RashiDasha::new(&CHARA, &chart, birth, YearLength::Julian36525, AfterCycle::End, RashiRules::BPHS)?;
    ///
    /// use teistro_core::catalogue::{DashaSystem, Graha};
    /// use teistro_core::quantity::Depth;
    /// use teistro_core::settings::root;
    /// use teistro_dasha::{DashaReading, Rules};
    ///
    /// let rules = Rules::of(&root().dasha, DashaSystem::Chara);
    /// let reading = DashaReading::of_rashi(&chara, rules, Depth::try_new(1)?);
    /// // Aries first, Mars its lord; no seed and no balance.
    /// assert_eq!((reading.first_lord, reading.rashi), (Graha::Mars, Some(RashiRules::BPHS)));
    /// assert!(reading.seed.is_none() && reading.balance.is_none());
    /// assert_eq!(reading.periods.len(), 12);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn of_rashi(dasha: &RashiDasha, rules: Rules, depth: Depth) -> DashaReading {
        let periods = rows(dasha, depth);
        DashaReading {
            system: dasha.row().system.clone(),
            definition: None,
            rules,
            kalachakra: None,
            rashi: Some(dasha.rules()),
            seed: None,
            first_lord: periods.first().map_or(Graha::Sun, |row| row.lord),
            overflow: false,
            moon_span: None,
            balance: None,
            depth,
            periods,
        }
    }
}

impl DashaReading {
    /// The Kalachakra's reading under the dasha group's `rules`, its periods
    /// to `depth` levels or to the antardashas it stops at, whichever is
    /// shallower, and the depth it carries says which (crux C58).
    ///
    /// ```
    /// # use teistro_core::angle::Nas;
    /// # use teistro_core::catalogue::{Nakshatra, Rashi};
    /// # use teistro_core::quantity::{Degrees, JulianDay};
    /// # use teistro_core::settings::{AfterCycle, Balance, KalachakraAfterNinth, KalachakraBalance, KalachakraMembership, YearLength};
    /// # use teistro_dasha::{Birth, KalachakraDasha, KalachakraRules, Timeline};
    /// #
    /// # let rules = KalachakraRules {
    /// #     balance: Balance::Spatial,
    /// #     year_length: YearLength::Julian36525,
    /// #     after_cycle: AfterCycle::End,
    /// #     membership: KalachakraMembership::Listed,
    /// #     balance_of: KalachakraBalance::FirstSign,
    /// #     after_ninth: KalachakraAfterNinth::Reverse,
    /// # };
    /// # // A Moon a quarter of the way through Ashwini's first pada.
    /// # let moon = Nas::from_degrees(Degrees::try_new(360.0 / 108.0 / 4.0)?);
    /// # let birth = Birth { instant: JulianDay::literal(2_451_545.0), moon, moon_span: None };
    /// # let dasha = KalachakraDasha::new(&birth, rules)?;
    ///
    /// use teistro_core::catalogue::DashaSystem;
    /// use teistro_core::quantity::Depth;
    /// use teistro_core::settings::root;
    /// use teistro_dasha::DashaReading;
    ///
    /// // Asked for three levels, it carries the two the Kalachakra has.
    /// let rules = teistro_dasha::Rules::of(&root().dasha, DashaSystem::Kalachakra);
    /// let reading = DashaReading::of_kalachakra(&dasha, rules, Depth::try_new(3)?, None);
    /// assert_eq!(reading.depth.get(), 2);
    /// assert_eq!(reading.kalachakra, Some(dasha.rules()));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn of_kalachakra(
        dasha: &KalachakraDasha,
        rules: Rules,
        depth: Depth,
        moon_span: Option<Interval>,
    ) -> DashaReading {
        let carried = Depth::try_new(depth.get().min(2)).unwrap_or(depth);
        let periods = rows(dasha, carried);
        DashaReading {
            system: DashaName::Catalogued(DashaSystem::Kalachakra),
            definition: None,
            rules,
            kalachakra: Some(dasha.rules()),
            rashi: None,
            seed: Some(dasha.seed()),
            first_lord: periods.first().map_or(Graha::Sun, |row| row.lord),
            overflow: false,
            moon_span,
            balance: Some(dasha.balance()),
            depth: carried,
            periods,
        }
    }
}

impl DashaReading {
    /// A Hellenistic time lord's reading (`03-design/hellenistic-time-lords.md`),
    /// or a graha dasha's (`03-design/dasha-kernels.md`), its periods to
    /// `depth` levels. It carries no seed and no balance, since its first
    /// period runs whole from birth; a time lord's first period's sign is
    /// the sign it starts from, and a graha dasha's mahadashas are its
    /// lords and their years, which is all a stored document needs to
    /// rebuild either.
    ///
    /// ```
    /// # use teistro_core::catalogue::Graha;
    /// # use teistro_core::quantity::JulianDay;
    /// # use teistro_core::settings::{FirdariaNodes, YearLength};
    /// # use teistro_dasha::{FirdariaDasha, Timeline};
    /// #
    /// # // A night birth: the firdaria begin from the Moon.
    /// # let birth = JulianDay::literal(2_451_545.0);
    /// # let night = FirdariaDasha::new(Graha::Moon, birth, YearLength::Julian36525, FirdariaNodes::End)?;
    ///
    /// use teistro_core::catalogue::DashaSystem;
    /// use teistro_core::quantity::Depth;
    /// use teistro_core::settings::root;
    /// use teistro_dasha::{DashaReading, Rules};
    ///
    /// let rules = Rules::of(&root().dasha, DashaSystem::Firdaria);
    /// let reading = DashaReading::of_time_lord(DashaSystem::Firdaria, &night, rules, Depth::try_new(1)?);
    /// assert_eq!(reading.first_lord, Graha::Moon);
    /// assert!(reading.seed.is_none() && reading.balance.is_none());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn of_time_lord(
        system: DashaSystem,
        timeline: &impl Timeline,
        rules: Rules,
        depth: Depth,
    ) -> DashaReading {
        let periods = rows(timeline, depth);
        DashaReading {
            system: DashaName::Catalogued(system),
            definition: None,
            rules,
            kalachakra: None,
            rashi: None,
            seed: None,
            first_lord: periods.first().map_or(Graha::Sun, |row| row.lord),
            overflow: false,
            moon_span: None,
            balance: None,
            depth,
            periods,
        }
    }

    /// A graha dasha's lords and their years, from its stored mahadashas:
    /// each span in the reading's years, rounded to `whole` years when the
    /// system's years are counts. Nothing when it stores no period.
    ///
    /// ```
    /// # use teistro_core::catalogue::Graha;
    /// # use teistro_core::quantity::JulianDay;
    /// # use teistro_core::settings::{AfterCycle, GrahaDivision, YearLength};
    /// # use teistro_dasha::{GrahaDasha, NAISARGIKA_LORDS, Timeline};
    /// #
    /// # // The Naisargika dasha: its seven lords from birth, in proportion.
    /// # let birth = JulianDay::literal(2_451_545.0);
    /// # let natural = GrahaDasha::new(NAISARGIKA_LORDS.to_vec(), birth, YearLength::Julian36525, GrahaDivision::Proportional, AfterCycle::End)?;
    /// # use teistro_core::catalogue::DashaSystem;
    /// # use teistro_core::quantity::Depth;
    /// # use teistro_core::settings::root;
    /// # use teistro_dasha::{DashaReading, Rules};
    /// #
    /// # let mut rules = Rules::of(&root().dasha, DashaSystem::Naisargika);
    /// # rules.year_length = YearLength::Julian36525;
    /// # let reading = DashaReading::of_time_lord(DashaSystem::Naisargika, &natural, rules, Depth::try_new(2)?);
    ///
    /// let lords = reading.graha_lords(true).ok_or("mahadashas stored")?;
    /// assert_eq!(lords[0], (Graha::Moon, 1.0));
    /// assert_eq!(lords[6], (Graha::Saturn, 50.0));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn graha_lords(&self, whole: bool) -> Option<Vec<(Graha, f64)>> {
        let year_days = self.rules.year_length.days();
        let lords: Vec<(Graha, f64)> = self
            .periods
            .iter()
            .filter(|row| row.level() == 1)
            .map(|row| {
                let years = row.interval.days() / year_days;
                (row.lord, if whole { years.round() } else { years })
            })
            .collect();
        (!lords.is_empty()).then_some(lords)
    }

    /// How a graha dasha's reading divides, when its rows show it: a first
    /// mahadasha whose sub-periods are equal, among lords whose years are
    /// not, is divided equally (BPHS ch. 51 vv. 3 and 4), and any other in
    /// proportion (vv. 1 and 2). Nothing when it stores one level, or when
    /// every lord's years are equal, which divide alike either way.
    ///
    /// ```
    /// # use teistro_core::catalogue::Graha;
    /// # use teistro_core::quantity::JulianDay;
    /// # use teistro_core::settings::{AfterCycle, GrahaDivision, YearLength};
    /// # use teistro_dasha::{GrahaDasha, NAISARGIKA_LORDS, Timeline};
    /// #
    /// # // The Naisargika dasha: its seven lords from birth, in proportion.
    /// # let birth = JulianDay::literal(2_451_545.0);
    /// # let natural = GrahaDasha::new(NAISARGIKA_LORDS.to_vec(), birth, YearLength::Julian36525, GrahaDivision::Proportional, AfterCycle::End)?;
    /// # use teistro_core::catalogue::DashaSystem;
    /// # use teistro_core::quantity::Depth;
    /// # use teistro_core::settings::root;
    /// # use teistro_dasha::{DashaReading, Rules};
    /// #
    /// # let mut rules = Rules::of(&root().dasha, DashaSystem::Naisargika);
    /// # rules.year_length = YearLength::Julian36525;
    /// # let reading = DashaReading::of_time_lord(DashaSystem::Naisargika, &natural, rules, Depth::try_new(2)?);
    ///
    /// assert_eq!(reading.graha_division(), Some(GrahaDivision::Proportional));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn graha_division(&self) -> Option<GrahaDivision> {
        let first = self.periods.iter().find(|row| row.level() == 1)?;
        let parts: Vec<f64> = self
            .periods
            .iter()
            .filter(|row| row.level() == 2 && row.path.starts_with(&format!("{}/", first.path)))
            .map(|row| row.interval.days())
            .collect();
        let years: Vec<f64> = self
            .periods
            .iter()
            .filter(|row| row.level() == 1)
            .map(|row| row.interval.days())
            .collect();
        let alike = |spans: &[f64]| {
            spans.iter().all(|span| {
                spans
                    .first()
                    .is_some_and(|first| (span - first).abs() < 1e-6)
            })
        };
        if parts.len() < 2 || alike(&years) {
            return None;
        }
        Some(if alike(&parts) {
            GrahaDivision::Equal
        } else {
            GrahaDivision::Proportional
        })
    }

    /// An ayurdaya dasha's seats and shares, from its stored periods: each
    /// mahadasha's lord, the sign it is of and its span in the reading's
    /// years, and each one's sub-periods as the seats they are of with
    /// their parts of its span. Nothing when it stores no second level, or
    /// a sub-period of no stored mahadasha's seat.
    ///
    /// ```
    /// # use teistro_core::catalogue::Graha;
    /// # use teistro_core::quantity::JulianDay;
    /// # use teistro_core::settings::{AfterCycle, GrahaDivision, YearLength};
    /// # use teistro_dasha::{GrahaDasha, NAISARGIKA_LORDS, Timeline};
    /// #
    /// # // The Naisargika dasha: its seven lords from birth, in proportion.
    /// # let birth = JulianDay::literal(2_451_545.0);
    /// # let natural = GrahaDasha::new(NAISARGIKA_LORDS.to_vec(), birth, YearLength::Julian36525, GrahaDivision::Proportional, AfterCycle::End)?;
    /// # use teistro_core::catalogue::DashaSystem;
    /// # use teistro_core::quantity::Depth;
    /// # use teistro_core::settings::root;
    /// # use teistro_dasha::{DashaReading, Rules};
    /// #
    /// # let mut rules = Rules::of(&root().dasha, DashaSystem::Naisargika);
    /// # rules.year_length = YearLength::Julian36525;
    /// # let reading = DashaReading::of_time_lord(DashaSystem::Naisargika, &natural, rules, Depth::try_new(2)?);
    ///
    /// let stored = reading.graha_shares().ok_or("two levels stored")?;
    /// assert_eq!(stored.seats[6].0, Graha::Saturn);
    /// let parts: f64 = stored.shares[0].iter().map(|(_, part)| part).sum();
    /// assert!((parts - 1.0).abs() < 1e-9, "the parts fill their mahadasha");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn graha_shares(&self) -> Option<GrahaShares> {
        let year_days = self.rules.year_length.days();
        let mahadashas: Vec<&PeriodRow> =
            self.periods.iter().filter(|row| row.level() == 1).collect();
        let seats: Vec<(Graha, Option<Rashi>, f64)> = mahadashas
            .iter()
            .map(|row| (row.lord, row.sign, row.interval.days() / year_days))
            .collect();
        let mut shares = Vec::with_capacity(mahadashas.len());
        for parent in &mahadashas {
            let under = format!("{}/", parent.path);
            let row: Option<Vec<(usize, f64)>> = self
                .periods
                .iter()
                .filter(|row| row.level() == 2 && row.path.starts_with(&under))
                .map(|child| {
                    let seat = seats
                        .iter()
                        .position(|seat| (seat.0, seat.1) == (child.lord, child.sign))?;
                    Some((seat, child.interval.days() / parent.interval.days()))
                })
                .collect();
            shares.push(row.filter(|row| !row.is_empty())?);
        }
        (!seats.is_empty()).then_some(GrahaShares { seats, shares })
    }

    /// The sign a time lord's reading starts from: its first period's.
    ///
    /// ```
    /// # use teistro_core::quantity::JulianDay;
    /// # use teistro_core::catalogue::{Dignity, Rashi};
    /// # use teistro_core::settings::{AfterCycle, YearLength};
    /// # use teistro_dasha::rashi::{CHARA, RashiChart, RashiDasha, RashiRules};
    /// #
    /// # let chart = RashiChart {
    /// #     lagna: Rashi::Aries,
    /// #     arudha_lagna: Rashi::Leo,
    /// #     navamsa_lagna: Rashi::Sagittarius,
    /// #     signs: [
    /// #         Rashi::Leo, Rashi::Cancer, Rashi::Capricorn, Rashi::Leo, Rashi::Sagittarius,
    /// #         Rashi::Virgo, Rashi::Aquarius, Rashi::Gemini, Rashi::Sagittarius,
    /// #     ],
    /// #     dignities: [Dignity::Neutral; 9],
    /// #     brahma: None,
    /// # };
    /// # let birth = JulianDay::literal(2_451_545.0);
    /// # let chara = RashiDasha::new(&CHARA, &chart, birth, YearLength::Julian36525, AfterCycle::End, RashiRules::BPHS)?;
    ///
    /// use teistro_core::catalogue::DashaSystem;
    /// use teistro_core::quantity::Depth;
    /// use teistro_core::settings::root;
    /// use teistro_dasha::{DashaReading, Rules};
    ///
    /// let rules = Rules::of(&root().dasha, DashaSystem::Chara);
    /// let reading = DashaReading::of_rashi(&chara, rules, Depth::try_new(1)?);
    /// assert_eq!(reading.start_sign(), Some(Rashi::Aries));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn start_sign(&self) -> Option<Rashi> {
        self.periods.first().and_then(|row| row.sign)
    }

    /// Where a firdaria reading's round places the nodes: after Mars when
    /// its stored periods put the Head before the seven are done, at the
    /// end otherwise (a day round, whose Mars is last, reads the same
    /// both ways).
    ///
    /// ```
    /// # use teistro_core::catalogue::Graha;
    /// # use teistro_core::quantity::JulianDay;
    /// # use teistro_core::settings::{FirdariaNodes, YearLength};
    /// # use teistro_dasha::{FirdariaDasha, Timeline};
    /// #
    /// # // A night birth: the firdaria begin from the Moon.
    /// # let birth = JulianDay::literal(2_451_545.0);
    /// # let night = FirdariaDasha::new(Graha::Moon, birth, YearLength::Julian36525, FirdariaNodes::End)?;
    /// # use teistro_core::catalogue::DashaSystem;
    /// # use teistro_core::quantity::Depth;
    /// # use teistro_core::settings::root;
    /// # use teistro_dasha::{DashaReading, Rules};
    /// #
    /// # let rules = Rules::of(&root().dasha, DashaSystem::Firdaria);
    /// # let reading = DashaReading::of_time_lord(DashaSystem::Firdaria, &night, rules, Depth::try_new(1)?);
    ///
    /// assert_eq!(reading.firdaria_nodes(), FirdariaNodes::End);
    /// // The nodes after Mars, mid-round.
    /// let early = FirdariaDasha::new(Graha::Moon, birth, YearLength::Julian36525, FirdariaNodes::AfterMars)?;
    /// let moved = DashaReading::of_time_lord(DashaSystem::Firdaria, &early, rules, Depth::try_new(1)?);
    /// assert_eq!(moved.firdaria_nodes(), FirdariaNodes::AfterMars);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn firdaria_nodes(&self) -> FirdariaNodes {
        let head = self
            .periods
            .iter()
            .filter(|row| row.level() == 1)
            .position(|row| row.lord == Graha::Rahu);
        match head {
            Some(at) if at < DESCENDING.len() => FirdariaNodes::AfterMars,
            _ => FirdariaNodes::End,
        }
    }

    /// The order a decennials reading runs the seven in: its first
    /// periods' lords, or nothing when it stores fewer than seven.
    ///
    /// ```
    /// # use teistro_core::catalogue::Graha;
    /// # use teistro_core::quantity::JulianDay;
    /// # use teistro_core::settings::{DecennialDivision, YearLength};
    /// # use teistro_dasha::{DecennialDasha, Timeline, decennial_order};
    /// #
    /// # // Valens's night nativity (VI.5): the Moon in Pisces 18°, Venus in Aries,
    /// # // Jupiter in Libra, Saturn in Sagittarius, Mars early in Aquarius, the Sun
    /// # // in Aquarius and Mercury in Pisces before the Moon.
    /// # let places = [
    /// #     (Graha::Moon, 348.0), (Graha::Venus, 10.0), (Graha::Jupiter, 195.0), (Graha::Saturn, 255.0),
    /// #     (Graha::Mars, 301.0), (Graha::Sun, 315.0), (Graha::Mercury, 340.0),
    /// # ];
    /// # let order = decennial_order(Graha::Moon, &places)?;
    /// # let birth = JulianDay::literal(2_451_545.0);
    /// # let valens = DecennialDasha::new(order, birth, YearLength::Savana360, DecennialDivision::Proportional)?;
    /// # use teistro_core::catalogue::DashaSystem;
    /// # use teistro_core::quantity::Depth;
    /// # use teistro_core::settings::root;
    /// # use teistro_dasha::{DashaReading, Rules};
    /// #
    /// # let rules = Rules::of(&root().dasha, DashaSystem::Decennials);
    /// # let reading = DashaReading::of_time_lord(DashaSystem::Decennials, &valens, rules, Depth::try_new(3)?);
    ///
    /// assert_eq!(reading.decennial_order(), Some(order));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn decennial_order(&self) -> Option<[Graha; DECENNIAL_STARS]> {
        let mut order = [Graha::Sun; DECENNIAL_STARS];
        let mut lords = self
            .periods
            .iter()
            .filter(|row| row.level() == 1)
            .map(|row| row.lord);
        for slot in &mut order {
            *slot = lords.next()?;
        }
        Some(order)
    }

    /// How a decennials reading divides below the second level, when its
    /// rows show it: a third-level period a thirtieth of its mahadasha is a
    /// cycle, 129 of Valens's days in the 129 months' 3870 under any year
    /// (VI.5), which no proportion of the minimum years gives: the first
    /// third-level period is the square of its star's years over 129 of
    /// the mahadasha, a thirtieth only for a star of 23.55 years. Nothing
    /// when it stores two levels or fewer.
    ///
    /// ```
    /// # use teistro_core::catalogue::Graha;
    /// # use teistro_core::quantity::JulianDay;
    /// # use teistro_core::settings::{DecennialDivision, YearLength};
    /// # use teistro_dasha::{DecennialDasha, Timeline, decennial_order};
    /// #
    /// # // Valens's night nativity (VI.5): the Moon in Pisces 18°, Venus in Aries,
    /// # // Jupiter in Libra, Saturn in Sagittarius, Mars early in Aquarius, the Sun
    /// # // in Aquarius and Mercury in Pisces before the Moon.
    /// # let places = [
    /// #     (Graha::Moon, 348.0), (Graha::Venus, 10.0), (Graha::Jupiter, 195.0), (Graha::Saturn, 255.0),
    /// #     (Graha::Mars, 301.0), (Graha::Sun, 315.0), (Graha::Mercury, 340.0),
    /// # ];
    /// # let order = decennial_order(Graha::Moon, &places)?;
    /// # let birth = JulianDay::literal(2_451_545.0);
    /// # let valens = DecennialDasha::new(order, birth, YearLength::Savana360, DecennialDivision::Proportional)?;
    /// # use teistro_core::catalogue::DashaSystem;
    /// # use teistro_core::quantity::Depth;
    /// # use teistro_core::settings::root;
    /// # use teistro_dasha::{DashaReading, Rules};
    /// #
    /// # let rules = Rules::of(&root().dasha, DashaSystem::Decennials);
    /// # let reading = DashaReading::of_time_lord(DashaSystem::Decennials, &valens, rules, Depth::try_new(3)?);
    ///
    /// assert_eq!(reading.decennial_division(), Some(DecennialDivision::Proportional));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn decennial_division(&self) -> Option<DecennialDivision> {
        let first = self.periods.iter().find(|row| row.level() == 1)?;
        let third = self.periods.iter().find(|row| row.level() == 3)?;
        let share = third.interval.days() / first.interval.days();
        Some(if (share - 1.0 / 30.0).abs() < 1e-9 {
            DecennialDivision::Cycles
        } else {
            DecennialDivision::Proportional
        })
    }
}

/// Every period of the birth cycle to `depth`, depth first, as rows.
fn rows(timeline: &impl Timeline, depth: Depth) -> Vec<PeriodRow> {
    fn collect(timeline: &impl Timeline, period: Period, depth: usize, out: &mut Vec<PeriodRow>) {
        out.push(PeriodRow::of(&period));
        if period.path.depth() < depth {
            for child in timeline.children(&period) {
                collect(timeline, child, depth, out);
            }
        }
    }
    let mut out = Vec::new();
    let levels = usize::from(depth.get());
    for maha in timeline.mahadashas() {
        collect(timeline, maha, levels, &mut out);
    }
    out
}

/// The cursor behind any dasha a document carries: nakshatra-seeded or
/// sign-based, read the same way through [`Timeline`].
///
/// ```
/// # use teistro_core::catalogue::Graha;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::{FirdariaNodes, YearLength};
/// # use teistro_dasha::{FirdariaDasha, Timeline};
/// #
/// # // A night birth: the firdaria begin from the Moon.
/// # let birth = JulianDay::literal(2_451_545.0);
/// # let night = FirdariaDasha::new(Graha::Moon, birth, YearLength::Julian36525, FirdariaNodes::End)?;
///
/// use teistro_dasha::DashaCursor;
///
/// // Read through Timeline whatever the kernel.
/// let cursor = DashaCursor::Firdaria(night.clone());
/// assert_eq!(cursor.mahadashas().count(), night.mahadashas().count());
/// assert_eq!(cursor.mahadasha(0, 0).map(|period| period.lord), Some(Graha::Moon));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum DashaCursor {
    /// A nakshatra-seeded dasha.
    Nakshatra(Dasha),
    /// A sign-based dasha.
    Rashi(RashiDasha),
    /// The Kalachakra.
    Kalachakra(KalachakraDasha),
    /// Valens's releasing from a lot.
    Releasing(ReleasingDasha),
    /// The profected year.
    Profection(ProfectionDasha),
    /// The firdaria.
    Firdaria(FirdariaDasha),
    /// Valens's decennials.
    Decennials(DecennialDasha),
    /// A graha dasha, its lords and years from the chart.
    Graha(GrahaDasha),
}

impl Timeline for DashaCursor {
    fn breadth(&self) -> usize {
        match self {
            DashaCursor::Nakshatra(dasha) => dasha.breadth(),
            DashaCursor::Rashi(dasha) => dasha.breadth(),
            DashaCursor::Kalachakra(dasha) => dasha.breadth(),
            DashaCursor::Releasing(dasha) => dasha.breadth(),
            DashaCursor::Profection(dasha) => dasha.breadth(),
            DashaCursor::Firdaria(dasha) => dasha.breadth(),
            DashaCursor::Decennials(dasha) => dasha.breadth(),
            DashaCursor::Graha(dasha) => dasha.breadth(),
        }
    }

    fn mahadasha(&self, cycle: u32, index: usize) -> Option<Period> {
        match self {
            DashaCursor::Nakshatra(dasha) => dasha.mahadasha(cycle, index),
            DashaCursor::Rashi(dasha) => dasha.mahadasha(cycle, index),
            DashaCursor::Kalachakra(dasha) => dasha.mahadasha(cycle, index),
            DashaCursor::Releasing(dasha) => dasha.mahadasha(cycle, index),
            DashaCursor::Profection(dasha) => dasha.mahadasha(cycle, index),
            DashaCursor::Firdaria(dasha) => dasha.mahadasha(cycle, index),
            DashaCursor::Decennials(dasha) => dasha.mahadasha(cycle, index),
            DashaCursor::Graha(dasha) => dasha.mahadasha(cycle, index),
        }
    }

    fn mahadasha_at(&self, instant: f64) -> Option<Period> {
        match self {
            DashaCursor::Nakshatra(dasha) => dasha.mahadasha_at(instant),
            DashaCursor::Rashi(dasha) => dasha.mahadasha_at(instant),
            DashaCursor::Kalachakra(dasha) => dasha.mahadasha_at(instant),
            DashaCursor::Releasing(dasha) => dasha.mahadasha_at(instant),
            DashaCursor::Profection(dasha) => dasha.mahadasha_at(instant),
            DashaCursor::Firdaria(dasha) => dasha.mahadasha_at(instant),
            DashaCursor::Decennials(dasha) => dasha.mahadasha_at(instant),
            DashaCursor::Graha(dasha) => dasha.mahadasha_at(instant),
        }
    }

    fn child(&self, parent: &Period, index: usize) -> Option<Period> {
        match self {
            DashaCursor::Nakshatra(dasha) => dasha.child(parent, index),
            DashaCursor::Rashi(dasha) => dasha.child(parent, index),
            DashaCursor::Kalachakra(dasha) => dasha.child(parent, index),
            DashaCursor::Releasing(dasha) => dasha.child(parent, index),
            DashaCursor::Profection(dasha) => dasha.child(parent, index),
            DashaCursor::Firdaria(dasha) => dasha.child(parent, index),
            DashaCursor::Decennials(dasha) => dasha.child(parent, index),
            DashaCursor::Graha(dasha) => dasha.child(parent, index),
        }
    }
}

impl DashaCursor {
    /// The nakshatra-seeded dasha, when it is one.
    ///
    /// ```
    /// # use teistro_core::angle::Nas;
    /// # use teistro_core::catalogue::{DashaSystem, Graha};
    /// # use teistro_core::quantity::{Degrees, Depth, JulianDay};
    /// # use teistro_core::settings::root;
    /// # use teistro_dasha::{Birth, Dasha, Rules, Timeline, VIMSHOTTARI};
    /// #
    /// # // A Moon a third of the way into Anuradha, Saturn's nakshatra: two thirds
    /// # // of his nineteen years remain at birth.
    /// # let moon = Nas::from_degrees(Degrees::try_new(16.0 * 360.0 / 27.0 + 360.0 / 81.0)?);
    /// # let birth = Birth { instant: JulianDay::literal(2_447_995.5), moon, moon_span: None };
    /// # let dasha = Dasha::new(&VIMSHOTTARI, &birth, Rules::of(&root().dasha, DashaSystem::Vimshottari))?;
    /// # // Saturn's balance ends 12 years 8 months after birth.
    /// # let saturn_ends = 2_447_995.5 + 19.0 * 365.25 * 2.0 / 3.0;
    ///
    /// use teistro_dasha::DashaCursor;
    ///
    /// let cursor = DashaCursor::Nakshatra(dasha);
    /// assert!(cursor.nakshatra().is_some() && cursor.rashi().is_none());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn nakshatra(&self) -> Option<&Dasha> {
        match self {
            DashaCursor::Nakshatra(dasha) => Some(dasha),
            _ => None,
        }
    }

    /// The sign-based dasha, when it is one.
    ///
    /// ```
    /// # use teistro_core::quantity::JulianDay;
    /// # use teistro_core::catalogue::{Dignity, Rashi};
    /// # use teistro_core::settings::{AfterCycle, YearLength};
    /// # use teistro_dasha::rashi::{CHARA, RashiChart, RashiDasha, RashiRules};
    /// #
    /// # let chart = RashiChart {
    /// #     lagna: Rashi::Aries,
    /// #     arudha_lagna: Rashi::Leo,
    /// #     navamsa_lagna: Rashi::Sagittarius,
    /// #     signs: [
    /// #         Rashi::Leo, Rashi::Cancer, Rashi::Capricorn, Rashi::Leo, Rashi::Sagittarius,
    /// #         Rashi::Virgo, Rashi::Aquarius, Rashi::Gemini, Rashi::Sagittarius,
    /// #     ],
    /// #     dignities: [Dignity::Neutral; 9],
    /// #     brahma: None,
    /// # };
    /// # let birth = JulianDay::literal(2_451_545.0);
    /// # let chara = RashiDasha::new(&CHARA, &chart, birth, YearLength::Julian36525, AfterCycle::End, RashiRules::BPHS)?;
    ///
    /// use teistro_dasha::DashaCursor;
    ///
    /// let cursor = DashaCursor::Rashi(chara);
    /// assert_eq!(cursor.rashi().map(|dasha| dasha.signs()[0]), Some(Rashi::Aries));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn rashi(&self) -> Option<&RashiDasha> {
        match self {
            DashaCursor::Rashi(dasha) => Some(dasha),
            _ => None,
        }
    }

    /// The Kalachakra, when it is the Kalachakra.
    ///
    /// ```
    /// # use teistro_core::angle::Nas;
    /// # use teistro_core::catalogue::{Nakshatra, Rashi};
    /// # use teistro_core::quantity::{Degrees, JulianDay};
    /// # use teistro_core::settings::{AfterCycle, Balance, KalachakraAfterNinth, KalachakraBalance, KalachakraMembership, YearLength};
    /// # use teistro_dasha::{Birth, KalachakraDasha, KalachakraRules, Timeline};
    /// #
    /// # let rules = KalachakraRules {
    /// #     balance: Balance::Spatial,
    /// #     year_length: YearLength::Julian36525,
    /// #     after_cycle: AfterCycle::End,
    /// #     membership: KalachakraMembership::Listed,
    /// #     balance_of: KalachakraBalance::FirstSign,
    /// #     after_ninth: KalachakraAfterNinth::Reverse,
    /// # };
    /// # // A Moon a quarter of the way through Ashwini's first pada.
    /// # let moon = Nas::from_degrees(Degrees::try_new(360.0 / 108.0 / 4.0)?);
    /// # let birth = Birth { instant: JulianDay::literal(2_451_545.0), moon, moon_span: None };
    /// # let dasha = KalachakraDasha::new(&birth, rules)?;
    ///
    /// use teistro_dasha::DashaCursor;
    ///
    /// let cursor = DashaCursor::Kalachakra(dasha);
    /// assert!(cursor.kalachakra().is_some() && cursor.nakshatra().is_none());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn kalachakra(&self) -> Option<&KalachakraDasha> {
        match self {
            DashaCursor::Kalachakra(dasha) => Some(dasha),
            _ => None,
        }
    }

    /// Valens's releasing, when it is releasing.
    ///
    /// ```
    /// use teistro_core::catalogue::Rashi;
    /// use teistro_core::quantity::JulianDay;
    /// use teistro_core::settings::YearLength;
    /// use teistro_dasha::{DashaCursor, ReleasingDasha};
    ///
    /// let birth = JulianDay::literal(2_451_545.0);
    /// let cursor = DashaCursor::Releasing(ReleasingDasha::new(Rashi::Leo, birth, YearLength::Savana360));
    /// assert!(cursor.releasing().is_some() && cursor.profection().is_none());
    /// ```
    #[must_use]
    pub const fn releasing(&self) -> Option<&ReleasingDasha> {
        match self {
            DashaCursor::Releasing(dasha) => Some(dasha),
            _ => None,
        }
    }

    /// The profected year, when it is a profection.
    ///
    /// ```
    /// use teistro_core::catalogue::Rashi;
    /// use teistro_core::quantity::JulianDay;
    /// use teistro_core::settings::YearLength;
    /// use teistro_dasha::{DashaCursor, ProfectionDasha};
    ///
    /// let birth = JulianDay::literal(2_451_545.0);
    /// let cursor = DashaCursor::Profection(ProfectionDasha::new(Rashi::Virgo, birth, YearLength::Julian36525));
    /// assert_eq!(cursor.profection().and_then(|dasha| dasha.sign_of_year(1)), Some(Rashi::Virgo));
    /// ```
    #[must_use]
    pub const fn profection(&self) -> Option<&ProfectionDasha> {
        match self {
            DashaCursor::Profection(dasha) => Some(dasha),
            _ => None,
        }
    }

    /// The firdaria, when it is the firdaria.
    ///
    /// ```
    /// # use teistro_core::catalogue::Graha;
    /// # use teistro_core::quantity::JulianDay;
    /// # use teistro_core::settings::{FirdariaNodes, YearLength};
    /// # use teistro_dasha::{FirdariaDasha, Timeline};
    /// #
    /// # // A night birth: the firdaria begin from the Moon.
    /// # let birth = JulianDay::literal(2_451_545.0);
    /// # let night = FirdariaDasha::new(Graha::Moon, birth, YearLength::Julian36525, FirdariaNodes::End)?;
    ///
    /// use teistro_dasha::DashaCursor;
    ///
    /// let cursor = DashaCursor::Firdaria(night);
    /// assert!(cursor.firdaria().is_some() && cursor.decennials().is_none());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn firdaria(&self) -> Option<&FirdariaDasha> {
        match self {
            DashaCursor::Firdaria(dasha) => Some(dasha),
            _ => None,
        }
    }

    /// The decennials' kernel, when that is what this is.
    ///
    /// ```
    /// # use teistro_core::catalogue::Graha;
    /// # use teistro_core::quantity::JulianDay;
    /// # use teistro_core::settings::{DecennialDivision, YearLength};
    /// # use teistro_dasha::{DecennialDasha, Timeline, decennial_order};
    /// #
    /// # // Valens's night nativity (VI.5): the Moon in Pisces 18°, Venus in Aries,
    /// # // Jupiter in Libra, Saturn in Sagittarius, Mars early in Aquarius, the Sun
    /// # // in Aquarius and Mercury in Pisces before the Moon.
    /// # let places = [
    /// #     (Graha::Moon, 348.0), (Graha::Venus, 10.0), (Graha::Jupiter, 195.0), (Graha::Saturn, 255.0),
    /// #     (Graha::Mars, 301.0), (Graha::Sun, 315.0), (Graha::Mercury, 340.0),
    /// # ];
    /// # let order = decennial_order(Graha::Moon, &places)?;
    /// # let birth = JulianDay::literal(2_451_545.0);
    /// # let valens = DecennialDasha::new(order, birth, YearLength::Savana360, DecennialDivision::Proportional)?;
    ///
    /// use teistro_dasha::DashaCursor;
    ///
    /// let cursor = DashaCursor::Decennials(valens);
    /// assert!(cursor.decennials().is_some() && cursor.firdaria().is_none());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn decennials(&self) -> Option<&DecennialDasha> {
        match self {
            DashaCursor::Decennials(dasha) => Some(dasha),
            _ => None,
        }
    }

    /// The graha dasha, when it is one.
    ///
    /// ```
    /// # use teistro_core::catalogue::Graha;
    /// # use teistro_core::quantity::JulianDay;
    /// # use teistro_core::settings::{AfterCycle, GrahaDivision, YearLength};
    /// # use teistro_dasha::{GrahaDasha, NAISARGIKA_LORDS, Timeline};
    /// #
    /// # // The Naisargika dasha: its seven lords from birth, in proportion.
    /// # let birth = JulianDay::literal(2_451_545.0);
    /// # let natural = GrahaDasha::new(NAISARGIKA_LORDS.to_vec(), birth, YearLength::Julian36525, GrahaDivision::Proportional, AfterCycle::End)?;
    ///
    /// use teistro_dasha::DashaCursor;
    ///
    /// let cursor = DashaCursor::Graha(natural);
    /// assert_eq!(cursor.graha().map(|dasha| dasha.lords().len()), Some(7));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn graha(&self) -> Option<&GrahaDasha> {
        match self {
            DashaCursor::Graha(dasha) => Some(dasha),
            _ => None,
        }
    }
}
