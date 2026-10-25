//! The **day's results**, read from a day's chart
//! (`03-design/varshesha.md`, "The day's results").
//!
//! *Tajika Nilakanthi*, Varsha Tantra, the chapter on the month's and the
//! day's results, vv. 16–33. Like the month's ([`crate::masa_phala`]), the
//! verses work in clauses that none weighs against another, so this
//! module reports the clauses that hold and gives no verdict:
//!
//! - v. 18: benefics strong in a kendra, a trikona or the 11th from the
//!   lagna or the Moon, and, the gloss says, "as well", malefics in the
//!   3rd, 6th or 11th;
//! - v. 19: the day's, the year's, the month's and the Muntha's lords
//!   where they stand;
//! - vv. 20–21: each house's navamsha lord joined or befriended by a
//!   benefic, or seen by the Moon;
//! - vv. 26–29: the malefics and benefics about the lagna, the 2nd, the
//!   4th, the 7th and the 9th, the waning Moon in the 1st or 8th, and the
//!   Moon with Mars in the 6th or 8th;
//! - vv. 30–33: the Moon's state, one of twelve of 2°30′ each.
//!
//! The malefics are the chapter's own, the Sun, Mars, Saturn and the
//! waning Moon, and the benefics the rest (crux C460). Verses 22–25 read
//! the navamsha signs of the 6th, 7th, 8th and 12th "joined" by planets,
//! and do not say in which chart; they are not built.
//!
//! ```
//! use teistro_tajika::MoonState;
//!
//! // The gloss's example: the Moon at 2s 4°25′40″ has passed one state,
//! // so she is in the second, Naṣṭa.
//! let moon = 60.0 + 4.0 + 25.0 / 60.0 + 40.0 / 3600.0;
//! assert_eq!(MoonState::of_longitude(moon), MoonState::Nashta);
//! ```

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::error::Error;
use teistro_core::house::House;

use crate::bala::{AnnualSky, SEVEN, finite_longitude, navamsha_lord, sign_of_longitude};
use crate::masa_phala::{Bond, LordClause, MasaPhalaRules, lord_clauses, strength_of};
use crate::states::AnnualStates;
use crate::yoga::Strength;

/// The Moon's twelve states in a sign, 2°30′ each, in the gloss's order
/// (vv. 31–33).
///
/// ```
/// # use teistro_tajika::AnnualSky;
/// #
/// # // Charak's Example Chart, the forty-first year's: signs counted from Aries as 0.
/// # let at = |sign: f64, deg: f64, min: f64| sign * 30.0 + deg + min / 60.0;
/// # let sky = AnnualSky {
/// #     sun_deg: at(4.0, 3.0, 50.0),
/// #     moon_deg: at(1.0, 9.0, 40.0),
/// #     mars_deg: at(7.0, 7.0, 42.0),
/// #     mercury_deg: at(4.0, 18.0, 20.0),
/// #     jupiter_deg: at(8.0, 9.0, 38.0),
/// #     venus_deg: at(4.0, 21.0, 45.0),
/// #     saturn_deg: at(6.0, 17.0, 13.0),
/// # };
/// # use teistro_core::catalogue::Graha;
/// # use teistro_tajika::{AnnualStates, DayLords, MasaPhalaRules, dina_phala};
/// #
/// # let lagna = at(7.0, 9.0, 26.0);
/// # let lords = DayLords {
/// #     dinesha: Graha::Saturn,
/// #     varshesha: Graha::Sun,
/// #     masesha: Graha::Mars,
/// #     muntha: Graha::Jupiter,
/// # };
/// # let read = |sky: &AnnualSky| dina_phala(lagna, sky, &AnnualStates::default(), &lords, MasaPhalaRules::default());
/// # let day = read(&sky)?;
///
/// use teistro_tajika::MoonState;
///
/// // The Moon at Taurus 9°40′ has passed three states: the fourth, Jaya.
/// assert_eq!(day.moon_state, MoonState::Jaya);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MoonState {
    /// *Pravāsa*: a stay abroad.
    Pravasa,
    /// *Naṣṭa*: loss of wealth.
    Nashta,
    /// *Mṛta*: fear of death.
    Mrita,
    /// *Jaya*: victory.
    Jaya,
    /// *Hāsya*: pleasure.
    Hasya,
    /// *Rati*: union.
    Rati,
    /// *Krīḍita*: happiness.
    Kridita,
    /// *Prasupta*: sleep and quarrel.
    Prasupta,
    /// *Bhukta*: bodily pain.
    Bhukta,
    /// *Jvara*: fear.
    Jvara,
    /// *Kampita*: loss.
    Kampita,
    /// *Susthita*: ease.
    Susthita,
}

impl MoonState {
    /// All twelve, in the gloss's order.
    ///
    /// ```
    /// use teistro_tajika::MoonState;
    ///
    /// // Each holds 2°30′ of a sign.
    /// for (passed, state) in MoonState::ALL.into_iter().enumerate() {
    ///     assert_eq!(MoonState::of_longitude(2.5 * passed as f64 + 1.0), state);
    /// }
    /// ```
    pub const ALL: [MoonState; 12] = [
        MoonState::Pravasa,
        MoonState::Nashta,
        MoonState::Mrita,
        MoonState::Jaya,
        MoonState::Hasya,
        MoonState::Rati,
        MoonState::Kridita,
        MoonState::Prasupta,
        MoonState::Bhukta,
        MoonState::Jvara,
        MoonState::Kampita,
        MoonState::Susthita,
    ];

    /// The member's key, as serde writes it: `PRAVASA`.
    ///
    /// ```
    /// use teistro_tajika::MoonState;
    ///
    /// assert_eq!(MoonState::Susthita.key(), "SUSTHITA");
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            MoonState::Pravasa => "PRAVASA",
            MoonState::Nashta => "NASHTA",
            MoonState::Mrita => "MRITA",
            MoonState::Jaya => "JAYA",
            MoonState::Hasya => "HASYA",
            MoonState::Rati => "RATI",
            MoonState::Kridita => "KRIDITA",
            MoonState::Prasupta => "PRASUPTA",
            MoonState::Bhukta => "BHUKTA",
            MoonState::Jvara => "JVARA",
            MoonState::Kampita => "KAMPITA",
            MoonState::Susthita => "SUSTHITA",
        }
    }

    /// The state a Moon at `longitude_deg` is in: v. 31's degrees within
    /// the sign doubled and divided by five give the states passed, and
    /// the next is hers.
    ///
    /// ```
    /// use teistro_tajika::MoonState;
    ///
    /// assert_eq!(MoonState::of_longitude(0.0), MoonState::Pravasa);
    /// // Only the degrees within the sign count.
    /// assert_eq!(MoonState::of_longitude(359.9), MoonState::Susthita);
    /// assert_eq!(MoonState::of_longitude(-0.1), MoonState::of_longitude(359.9));
    /// ```
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "degrees within a sign, 0..30, times two over five is 0..12"
    )]
    pub fn of_longitude(longitude_deg: f64) -> MoonState {
        let passed = (longitude_deg.rem_euclid(30.0) * 2.0 / 5.0) as usize;
        MoonState::ALL
            .get(passed)
            .copied()
            .unwrap_or(MoonState::Susthita)
    }
}

/// The day's lords v. 19 judges, in its order.
///
/// ```
/// # use teistro_tajika::AnnualSky;
/// #
/// # // Charak's Example Chart, the forty-first year's: signs counted from Aries as 0.
/// # let at = |sign: f64, deg: f64, min: f64| sign * 30.0 + deg + min / 60.0;
/// # let sky = AnnualSky {
/// #     sun_deg: at(4.0, 3.0, 50.0),
/// #     moon_deg: at(1.0, 9.0, 40.0),
/// #     mars_deg: at(7.0, 7.0, 42.0),
/// #     mercury_deg: at(4.0, 18.0, 20.0),
/// #     jupiter_deg: at(8.0, 9.0, 38.0),
/// #     venus_deg: at(4.0, 21.0, 45.0),
/// #     saturn_deg: at(6.0, 17.0, 13.0),
/// # };
/// # use teistro_core::catalogue::Graha;
/// # use teistro_tajika::{AnnualStates, DayLords, MasaPhalaRules, dina_phala};
/// #
/// # let lagna = at(7.0, 9.0, 26.0);
/// # let lords = DayLords {
/// #     dinesha: Graha::Saturn,
/// #     varshesha: Graha::Sun,
/// #     masesha: Graha::Mars,
/// #     muntha: Graha::Jupiter,
/// # };
/// # let read = |sky: &AnnualSky| dina_phala(lagna, sky, &AnnualStates::default(), &lords, MasaPhalaRules::default());
/// # let day = read(&sky)?;
///
/// use teistro_tajika::DayLord;
///
/// let offices: Vec<DayLord> = day.lords.iter().map(|lord| lord.office).collect();
/// assert_eq!(offices, DayLord::ALL);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DayLord {
    /// The Dinesha, the day's lord.
    Dinesha,
    /// The Varshesha, the year's lord.
    Varshesha,
    /// The Masesha, the month's lord.
    Masesha,
    /// The lord of the Muntha's sign this day.
    Muntha,
}

impl DayLord {
    /// All four, in the verse's order.
    ///
    /// ```
    /// use teistro_tajika::DayLord;
    ///
    /// assert_eq!((DayLord::ALL[0], DayLord::ALL[3]), (DayLord::Dinesha, DayLord::Muntha));
    /// ```
    pub const ALL: [DayLord; 4] = [
        DayLord::Dinesha,
        DayLord::Varshesha,
        DayLord::Masesha,
        DayLord::Muntha,
    ];

    /// The member's key, as serde writes it: `DINESHA`.
    ///
    /// ```
    /// use teistro_tajika::DayLord;
    ///
    /// assert_eq!(DayLord::Varshesha.key(), "VARSHESHA");
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            DayLord::Dinesha => "DINESHA",
            DayLord::Varshesha => "VARSHESHA",
            DayLord::Masesha => "MASESHA",
            DayLord::Muntha => "MUNTHA",
        }
    }
}

/// Who holds each of v. 19's four offices this day.
///
/// ```
/// # use teistro_tajika::AnnualSky;
/// #
/// # // Charak's Example Chart, the forty-first year's: signs counted from Aries as 0.
/// # let at = |sign: f64, deg: f64, min: f64| sign * 30.0 + deg + min / 60.0;
/// # let sky = AnnualSky {
/// #     sun_deg: at(4.0, 3.0, 50.0),
/// #     moon_deg: at(1.0, 9.0, 40.0),
/// #     mars_deg: at(7.0, 7.0, 42.0),
/// #     mercury_deg: at(4.0, 18.0, 20.0),
/// #     jupiter_deg: at(8.0, 9.0, 38.0),
/// #     venus_deg: at(4.0, 21.0, 45.0),
/// #     saturn_deg: at(6.0, 17.0, 13.0),
/// # };
/// # use teistro_core::catalogue::Graha;
/// # use teistro_tajika::{AnnualStates, DayLords, MasaPhalaRules, dina_phala};
/// #
/// # let lagna = at(7.0, 9.0, 26.0);
/// # let lords = DayLords {
/// #     dinesha: Graha::Saturn,
/// #     varshesha: Graha::Sun,
/// #     masesha: Graha::Mars,
/// #     muntha: Graha::Jupiter,
/// # };
/// # let read = |sky: &AnnualSky| dina_phala(lagna, sky, &AnnualStates::default(), &lords, MasaPhalaRules::default());
/// # let day = read(&sky)?;
///
/// // A lord outside the seven is refused by its office.
/// let nodal = DayLords { masesha: Graha::Rahu, ..lords };
/// let refused = dina_phala(lagna, &sky, &AnnualStates::default(), &nodal, MasaPhalaRules::default()).unwrap_err();
/// assert_eq!(refused.field(), Some("lords.masesha"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct DayLords {
    /// The day's lord.
    pub dinesha: Graha,
    /// The year's lord.
    pub varshesha: Graha,
    /// The month's lord.
    pub masesha: Graha,
    /// The lord of the Muntha's sign this day.
    pub muntha: Graha,
}

impl DayLords {
    /// Who holds one office.
    ///
    /// ```
    /// use teistro_core::catalogue::Graha;
    /// use teistro_tajika::{DayLord, DayLords};
    ///
    /// let lords = DayLords { dinesha: Graha::Saturn, varshesha: Graha::Sun, masesha: Graha::Mars, muntha: Graha::Jupiter };
    /// assert_eq!(lords.holder(DayLord::Masesha), Graha::Mars);
    /// assert_eq!(DayLord::ALL.map(|office| lords.holder(office))[0], Graha::Saturn);
    /// ```
    #[must_use]
    pub const fn holder(&self, office: DayLord) -> Graha {
        match office {
            DayLord::Dinesha => self.dinesha,
            DayLord::Varshesha => self.varshesha,
            DayLord::Masesha => self.masesha,
            DayLord::Muntha => self.muntha,
        }
    }
}

/// Where one of the day's lords stands, with the clauses of the month's
/// lords: v. 19 reads `TRIKA`, `WITH_MALEFIC`, `MALEFIC_INIMICAL` and
/// `WELL_PLACED` of them.
///
/// ```
/// # use teistro_tajika::AnnualSky;
/// #
/// # // Charak's Example Chart, the forty-first year's: signs counted from Aries as 0.
/// # let at = |sign: f64, deg: f64, min: f64| sign * 30.0 + deg + min / 60.0;
/// # let sky = AnnualSky {
/// #     sun_deg: at(4.0, 3.0, 50.0),
/// #     moon_deg: at(1.0, 9.0, 40.0),
/// #     mars_deg: at(7.0, 7.0, 42.0),
/// #     mercury_deg: at(4.0, 18.0, 20.0),
/// #     jupiter_deg: at(8.0, 9.0, 38.0),
/// #     venus_deg: at(4.0, 21.0, 45.0),
/// #     saturn_deg: at(6.0, 17.0, 13.0),
/// # };
/// # use teistro_core::catalogue::Graha;
/// # use teistro_tajika::{AnnualStates, DayLords, MasaPhalaRules, dina_phala};
/// #
/// # let lagna = at(7.0, 9.0, 26.0);
/// # let lords = DayLords {
/// #     dinesha: Graha::Saturn,
/// #     varshesha: Graha::Sun,
/// #     masesha: Graha::Mars,
/// #     muntha: Graha::Jupiter,
/// # };
/// # let read = |sky: &AnnualSky| dina_phala(lagna, sky, &AnnualStates::default(), &lords, MasaPhalaRules::default());
/// # let day = read(&sky)?;
///
/// use teistro_core::house::House;
/// use teistro_tajika::LordClause;
///
/// // The year's lord, the Sun, in Leo: the tenth from Scorpio.
/// let sun = &day.lords[1];
/// assert_eq!((sun.graha, sun.house), (Graha::Sun, House::try_new(10)?));
/// assert!(sun.clauses.contains(&LordClause::WellPlaced));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct DayLordPlace {
    /// The office.
    pub office: DayLord,
    /// Who holds it.
    pub graha: Graha,
    /// The house it stands in, by whole signs from the day lagna.
    pub house: House,
    /// The clauses that hold, in the month's verses' order.
    pub clauses: Vec<LordClause>,
}

impl DayLordPlace {
    /// Whether v. 19's ill holds: in the 6th, 8th or 12th with a malefic,
    /// or aspected by one inimically, as the gloss reads "joined".
    ///
    /// ```
    /// # use teistro_tajika::AnnualSky;
    /// #
    /// # // Charak's Example Chart, the forty-first year's: signs counted from Aries as 0.
    /// # let at = |sign: f64, deg: f64, min: f64| sign * 30.0 + deg + min / 60.0;
    /// # let sky = AnnualSky {
    /// #     sun_deg: at(4.0, 3.0, 50.0),
    /// #     moon_deg: at(1.0, 9.0, 40.0),
    /// #     mars_deg: at(7.0, 7.0, 42.0),
    /// #     mercury_deg: at(4.0, 18.0, 20.0),
    /// #     jupiter_deg: at(8.0, 9.0, 38.0),
    /// #     venus_deg: at(4.0, 21.0, 45.0),
    /// #     saturn_deg: at(6.0, 17.0, 13.0),
    /// # };
    /// # use teistro_core::catalogue::Graha;
    /// # use teistro_tajika::{AnnualStates, DayLords, MasaPhalaRules, dina_phala};
    /// #
    /// # let lagna = at(7.0, 9.0, 26.0);
    /// # let lords = DayLords {
    /// #     dinesha: Graha::Saturn,
    /// #     varshesha: Graha::Sun,
    /// #     masesha: Graha::Mars,
    /// #     muntha: Graha::Jupiter,
    /// # };
    /// # let read = |sky: &AnnualSky| dina_phala(lagna, sky, &AnnualStates::default(), &lords, MasaPhalaRules::default());
    /// # let day = read(&sky)?;
    ///
    /// // Saturn in Libra, the twelfth, alone and unharmed; Mars beside him afflicts him.
    /// assert!(!day.lords[0].afflicted());
    /// let joined = AnnualSky { mars_deg: at(6.0, 2.0, 0.0), ..sky };
    /// assert!(read(&joined)?.lords[0].afflicted());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn afflicted(&self) -> bool {
        self.clauses.contains(&LordClause::Trika)
            && (self.clauses.contains(&LordClause::WithMalefic)
                || self.clauses.contains(&LordClause::MaleficInimical))
    }

    /// Whether v. 19's good holds: in a kendra, a trikona or the 11th.
    ///
    /// ```
    /// # use teistro_tajika::AnnualSky;
    /// #
    /// # // Charak's Example Chart, the forty-first year's: signs counted from Aries as 0.
    /// # let at = |sign: f64, deg: f64, min: f64| sign * 30.0 + deg + min / 60.0;
    /// # let sky = AnnualSky {
    /// #     sun_deg: at(4.0, 3.0, 50.0),
    /// #     moon_deg: at(1.0, 9.0, 40.0),
    /// #     mars_deg: at(7.0, 7.0, 42.0),
    /// #     mercury_deg: at(4.0, 18.0, 20.0),
    /// #     jupiter_deg: at(8.0, 9.0, 38.0),
    /// #     venus_deg: at(4.0, 21.0, 45.0),
    /// #     saturn_deg: at(6.0, 17.0, 13.0),
    /// # };
    /// # use teistro_core::catalogue::Graha;
    /// # use teistro_tajika::{AnnualStates, DayLords, MasaPhalaRules, dina_phala};
    /// #
    /// # let lagna = at(7.0, 9.0, 26.0);
    /// # let lords = DayLords {
    /// #     dinesha: Graha::Saturn,
    /// #     varshesha: Graha::Sun,
    /// #     masesha: Graha::Mars,
    /// #     muntha: Graha::Jupiter,
    /// # };
    /// # let read = |sky: &AnnualSky| dina_phala(lagna, sky, &AnnualStates::default(), &lords, MasaPhalaRules::default());
    /// # let day = read(&sky)?;
    ///
    /// // The Sun in the tenth prospers; Saturn in the twelfth does not.
    /// assert!(day.lords[1].prospers());
    /// assert!(!day.lords[0].prospers());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn prospers(&self) -> bool {
        self.clauses.contains(&LordClause::WellPlaced)
    }
}

/// A clause of vv. 20–21 about one house's navamsha lord, in the verses'
/// order.
///
/// ```
/// # use teistro_tajika::AnnualSky;
/// #
/// # // Charak's Example Chart, the forty-first year's: signs counted from Aries as 0.
/// # let at = |sign: f64, deg: f64, min: f64| sign * 30.0 + deg + min / 60.0;
/// # let sky = AnnualSky {
/// #     sun_deg: at(4.0, 3.0, 50.0),
/// #     moon_deg: at(1.0, 9.0, 40.0),
/// #     mars_deg: at(7.0, 7.0, 42.0),
/// #     mercury_deg: at(4.0, 18.0, 20.0),
/// #     jupiter_deg: at(8.0, 9.0, 38.0),
/// #     venus_deg: at(4.0, 21.0, 45.0),
/// #     saturn_deg: at(6.0, 17.0, 13.0),
/// # };
/// # use teistro_core::catalogue::Graha;
/// # use teistro_tajika::{AnnualStates, DayLords, MasaPhalaRules, dina_phala};
/// #
/// # let lagna = at(7.0, 9.0, 26.0);
/// # let lords = DayLords {
/// #     dinesha: Graha::Saturn,
/// #     varshesha: Graha::Sun,
/// #     masesha: Graha::Mars,
/// #     muntha: Graha::Jupiter,
/// # };
/// # let read = |sky: &AnnualSky| dina_phala(lagna, sky, &AnnualStates::default(), &lords, MasaPhalaRules::default());
/// # let day = read(&sky)?;
///
/// use teistro_tajika::DayHouseClause;
///
/// // Every clause any house holds is one of the verses'.
/// for house in &day.houses {
///     assert!(house.clauses.iter().all(|clause| DayHouseClause::ALL.contains(clause)));
/// }
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DayHouseClause {
    /// Sharing a sign with a benefic, itself excepted.
    WithBenefic,
    /// Aspected by a benefic with a friendly aspect.
    BeneficFriendly,
    /// Aspected by the Moon with a friendly aspect (v. 20).
    MoonFriendly,
    /// Sharing a sign with a malefic, itself excepted: the gloss's
    /// "otherwise".
    WithMalefic,
    /// Aspected by a malefic with an inimical aspect, the 4th, 7th or 10th.
    MaleficInimical,
}

impl DayHouseClause {
    /// Every clause, in the verses' order.
    ///
    /// ```
    /// use teistro_tajika::DayHouseClause;
    ///
    /// assert_eq!(DayHouseClause::ALL[2], DayHouseClause::MoonFriendly);
    /// ```
    pub const ALL: [DayHouseClause; 5] = [
        DayHouseClause::WithBenefic,
        DayHouseClause::BeneficFriendly,
        DayHouseClause::MoonFriendly,
        DayHouseClause::WithMalefic,
        DayHouseClause::MaleficInimical,
    ];

    /// The member's key, as serde writes it: `WITH_BENEFIC`.
    ///
    /// ```
    /// use teistro_tajika::DayHouseClause;
    ///
    /// assert_eq!(DayHouseClause::MaleficInimical.key(), "MALEFIC_INIMICAL");
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            DayHouseClause::WithBenefic => "WITH_BENEFIC",
            DayHouseClause::BeneficFriendly => "BENEFIC_FRIENDLY",
            DayHouseClause::MoonFriendly => "MOON_FRIENDLY",
            DayHouseClause::WithMalefic => "WITH_MALEFIC",
            DayHouseClause::MaleficInimical => "MALEFIC_INIMICAL",
        }
    }
}

/// One house of the day, read through its navamsha lord.
///
/// ```
/// # use teistro_tajika::AnnualSky;
/// #
/// # // Charak's Example Chart, the forty-first year's: signs counted from Aries as 0.
/// # let at = |sign: f64, deg: f64, min: f64| sign * 30.0 + deg + min / 60.0;
/// # let sky = AnnualSky {
/// #     sun_deg: at(4.0, 3.0, 50.0),
/// #     moon_deg: at(1.0, 9.0, 40.0),
/// #     mars_deg: at(7.0, 7.0, 42.0),
/// #     mercury_deg: at(4.0, 18.0, 20.0),
/// #     jupiter_deg: at(8.0, 9.0, 38.0),
/// #     venus_deg: at(4.0, 21.0, 45.0),
/// #     saturn_deg: at(6.0, 17.0, 13.0),
/// # };
/// # use teistro_core::catalogue::Graha;
/// # use teistro_tajika::{AnnualStates, DayLords, MasaPhalaRules, dina_phala};
/// #
/// # let lagna = at(7.0, 9.0, 26.0);
/// # let lords = DayLords {
/// #     dinesha: Graha::Saturn,
/// #     varshesha: Graha::Sun,
/// #     masesha: Graha::Mars,
/// #     muntha: Graha::Jupiter,
/// # };
/// # let read = |sky: &AnnualSky| dina_phala(lagna, sky, &AnnualStates::default(), &lords, MasaPhalaRules::default());
/// # let day = read(&sky)?;
///
/// use teistro_core::house::House;
///
/// // The first house's point is the lagna, Scorpio 9°26′, in Virgo's navamsha.
/// let first = &day.houses[0];
/// assert_eq!((first.house, first.amsha_lord), (House::try_new(1)?, Graha::Mercury));
/// assert_eq!(day.houses.len(), 12);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct DayHouse {
    /// The house, counted by whole signs from the day lagna.
    pub house: House,
    /// The lord of the navamsha the house's point falls in, the lagna's
    /// degree carried into its sign (crux C453).
    pub amsha_lord: Graha,
    /// The clauses that hold, in the verses' order.
    pub clauses: Vec<DayHouseClause>,
}

/// A clause of vv. 18 and 26–29 about the day's chart as a whole, in the
/// verses' order.
///
/// ```
/// # use teistro_tajika::AnnualSky;
/// #
/// # // Charak's Example Chart, the forty-first year's: signs counted from Aries as 0.
/// # let at = |sign: f64, deg: f64, min: f64| sign * 30.0 + deg + min / 60.0;
/// # let sky = AnnualSky {
/// #     sun_deg: at(4.0, 3.0, 50.0),
/// #     moon_deg: at(1.0, 9.0, 40.0),
/// #     mars_deg: at(7.0, 7.0, 42.0),
/// #     mercury_deg: at(4.0, 18.0, 20.0),
/// #     jupiter_deg: at(8.0, 9.0, 38.0),
/// #     venus_deg: at(4.0, 21.0, 45.0),
/// #     saturn_deg: at(6.0, 17.0, 13.0),
/// # };
/// # use teistro_core::catalogue::Graha;
/// # use teistro_tajika::{AnnualStates, DayLords, MasaPhalaRules, dina_phala};
/// #
/// # let lagna = at(7.0, 9.0, 26.0);
/// # let lords = DayLords {
/// #     dinesha: Graha::Saturn,
/// #     varshesha: Graha::Sun,
/// #     masesha: Graha::Mars,
/// #     muntha: Graha::Jupiter,
/// # };
/// # let read = |sky: &AnnualSky| dina_phala(lagna, sky, &AnnualStates::default(), &lords, MasaPhalaRules::default());
/// # let day = read(&sky)?;
///
/// use teistro_tajika::DayClause;
///
/// // Jupiter carried into Cancer, the ninth from Scorpio.
/// assert!(!day.clauses.contains(&DayClause::BeneficNinth));
/// let ninth = AnnualSky { jupiter_deg: at(3.0, 10.0, 0.0), ..sky };
/// assert!(read(&ninth)?.clauses.contains(&DayClause::BeneficNinth));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DayClause {
    /// Every benefic strong and in a kendra, a trikona or the 11th from
    /// the lagna (v. 18).
    BeneficsWellPlacedFromLagna,
    /// The same counted from the Moon, she not counted (v. 18; crux C461).
    BeneficsWellPlacedFromMoon,
    /// Every malefic in the 3rd, 6th or 11th from the lagna: the gloss's
    /// "then as well", a second way to the same good (v. 18; crux C461).
    MaleficsWellPlacedFromLagna,
    /// The same counted from the Moon, she not counted (v. 18).
    MaleficsWellPlacedFromMoon,
    /// Malefics in both the 2nd and the 12th: loss (v. 26).
    MaleficsFlank,
    /// A benefic in the 12th: spending well (v. 26).
    BeneficTwelfth,
    /// A direct malefic in the 12th and a retrograde one in the 2nd, the
    /// gloss's scissors closing on the lagna: illness (v. 26).
    PapaKartari,
    /// The same of benefics: good (v. 26).
    ShubhaKartari,
    /// The waning Moon in the 1st or the 8th, joined or aspected by a
    /// malefic (v. 27).
    WaningMoonAfflicted,
    /// The Moon with Mars in the 6th or the 8th (v. 28).
    MoonMarsDusthana,
    /// A malefic in the 4th: a fall from a mount (v. 28).
    MaleficFourth,
    /// A benefic in the 7th: winning at play (v. 29).
    BeneficSeventh,
    /// A benefic in the 2nd: ease (v. 29).
    BeneficSecond,
    /// A benefic in the 9th: dharma, fortune, wealth, honour (v. 29).
    BeneficNinth,
}

impl DayClause {
    /// Every clause, in the verses' order.
    ///
    /// ```
    /// use teistro_tajika::DayClause;
    ///
    /// // v. 18's four first, v. 29's three last.
    /// assert_eq!(DayClause::ALL[0], DayClause::BeneficsWellPlacedFromLagna);
    /// assert_eq!(DayClause::ALL[13], DayClause::BeneficNinth);
    /// ```
    pub const ALL: [DayClause; 14] = [
        DayClause::BeneficsWellPlacedFromLagna,
        DayClause::BeneficsWellPlacedFromMoon,
        DayClause::MaleficsWellPlacedFromLagna,
        DayClause::MaleficsWellPlacedFromMoon,
        DayClause::MaleficsFlank,
        DayClause::BeneficTwelfth,
        DayClause::PapaKartari,
        DayClause::ShubhaKartari,
        DayClause::WaningMoonAfflicted,
        DayClause::MoonMarsDusthana,
        DayClause::MaleficFourth,
        DayClause::BeneficSeventh,
        DayClause::BeneficSecond,
        DayClause::BeneficNinth,
    ];

    /// The member's key, as serde writes it: `PAPA_KARTARI`.
    ///
    /// ```
    /// use teistro_tajika::DayClause;
    ///
    /// assert_eq!(DayClause::ShubhaKartari.key(), "SHUBHA_KARTARI");
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            DayClause::BeneficsWellPlacedFromLagna => "BENEFICS_WELL_PLACED_FROM_LAGNA",
            DayClause::BeneficsWellPlacedFromMoon => "BENEFICS_WELL_PLACED_FROM_MOON",
            DayClause::MaleficsWellPlacedFromLagna => "MALEFICS_WELL_PLACED_FROM_LAGNA",
            DayClause::MaleficsWellPlacedFromMoon => "MALEFICS_WELL_PLACED_FROM_MOON",
            DayClause::MaleficsFlank => "MALEFICS_FLANK",
            DayClause::BeneficTwelfth => "BENEFIC_TWELFTH",
            DayClause::PapaKartari => "PAPA_KARTARI",
            DayClause::ShubhaKartari => "SHUBHA_KARTARI",
            DayClause::WaningMoonAfflicted => "WANING_MOON_AFFLICTED",
            DayClause::MoonMarsDusthana => "MOON_MARS_DUSTHANA",
            DayClause::MaleficFourth => "MALEFIC_FOURTH",
            DayClause::BeneficSeventh => "BENEFIC_SEVENTH",
            DayClause::BeneficSecond => "BENEFIC_SECOND",
            DayClause::BeneficNinth => "BENEFIC_NINTH",
        }
    }
}

/// The day's results: the Moon's state, the twelve houses, the four lords
/// and the clauses about the chart as a whole.
///
/// ```
/// # use teistro_tajika::AnnualSky;
/// #
/// # // Charak's Example Chart, the forty-first year's: signs counted from Aries as 0.
/// # let at = |sign: f64, deg: f64, min: f64| sign * 30.0 + deg + min / 60.0;
/// # let sky = AnnualSky {
/// #     sun_deg: at(4.0, 3.0, 50.0),
/// #     moon_deg: at(1.0, 9.0, 40.0),
/// #     mars_deg: at(7.0, 7.0, 42.0),
/// #     mercury_deg: at(4.0, 18.0, 20.0),
/// #     jupiter_deg: at(8.0, 9.0, 38.0),
/// #     venus_deg: at(4.0, 21.0, 45.0),
/// #     saturn_deg: at(6.0, 17.0, 13.0),
/// # };
/// # use teistro_core::catalogue::Graha;
/// # use teistro_tajika::{AnnualStates, DayLords, MasaPhalaRules, dina_phala};
/// #
/// # let lagna = at(7.0, 9.0, 26.0);
/// # let lords = DayLords {
/// #     dinesha: Graha::Saturn,
/// #     varshesha: Graha::Sun,
/// #     masesha: Graha::Mars,
/// #     muntha: Graha::Jupiter,
/// # };
/// # let read = |sky: &AnnualSky| dina_phala(lagna, sky, &AnnualStates::default(), &lords, MasaPhalaRules::default());
/// # let day = read(&sky)?;
///
/// assert_eq!((day.houses.len(), day.lords.len()), (12, 4));
/// // The clauses hold in the verses' order.
/// assert!(day.clauses.windows(2).all(|pair| pair[0] < pair[1]));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct DinaPhala {
    /// The Moon's state (vv. 30–33).
    pub moon_state: MoonState,
    /// The twelve houses, first to twelfth.
    pub houses: Vec<DayHouse>,
    /// The four lords, in [`DayLord::ALL`]'s order.
    pub lords: Vec<DayLordPlace>,
    /// The clauses about the chart as a whole that hold, in the verses'
    /// order.
    pub clauses: Vec<DayClause>,
}

/// The day's results from its chart: `day_lagna_deg` and `sky` are the
/// day chart's, `states` its combust and retrograde planets, `lords` who
/// holds v. 19's offices this day.
///
/// ```
/// # use teistro_tajika::AnnualSky;
/// #
/// # // Charak's Example Chart, the forty-first year's: signs counted from Aries as 0.
/// # let at = |sign: f64, deg: f64, min: f64| sign * 30.0 + deg + min / 60.0;
/// # let sky = AnnualSky {
/// #     sun_deg: at(4.0, 3.0, 50.0),
/// #     moon_deg: at(1.0, 9.0, 40.0),
/// #     mars_deg: at(7.0, 7.0, 42.0),
/// #     mercury_deg: at(4.0, 18.0, 20.0),
/// #     jupiter_deg: at(8.0, 9.0, 38.0),
/// #     venus_deg: at(4.0, 21.0, 45.0),
/// #     saturn_deg: at(6.0, 17.0, 13.0),
/// # };
/// # use teistro_core::catalogue::Graha;
/// # use teistro_tajika::{AnnualStates, DayLords, MasaPhalaRules, dina_phala};
/// #
/// # let lagna = at(7.0, 9.0, 26.0);
/// # let lords = DayLords {
/// #     dinesha: Graha::Saturn,
/// #     varshesha: Graha::Sun,
/// #     masesha: Graha::Mars,
/// #     muntha: Graha::Jupiter,
/// # };
/// # let read = |sky: &AnnualSky| dina_phala(lagna, sky, &AnnualStates::default(), &lords, MasaPhalaRules::default());
/// # let day = read(&sky)?;
///
/// let refused = dina_phala(f64::NAN, &sky, &AnnualStates::default(), &lords, MasaPhalaRules::default()).unwrap_err();
/// assert_eq!(refused.field(), Some("day_lagna_deg"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// A longitude that is not a number, named by its field (`day_lagna_deg`
/// for the lagna); states no chart can hold, as [`AnnualStates::check`];
/// a lord outside the seven, named `lords.dinesha` and so on; floors
/// [`MasaPhalaRules::check`] refuses.
pub fn dina_phala(
    day_lagna_deg: f64,
    sky: &AnnualSky,
    states: &AnnualStates,
    lords: &DayLords,
    rules: MasaPhalaRules,
) -> Result<DinaPhala, Error> {
    sky.check()?;
    finite_longitude("day_lagna_deg", day_lagna_deg)?;
    states.validate()?;
    let rules = rules.check()?;
    for office in DayLord::ALL {
        let graha = lords.holder(office);
        if !SEVEN.contains(&graha) {
            let field = match office {
                DayLord::Dinesha => "lords.dinesha",
                DayLord::Varshesha => "lords.varshesha",
                DayLord::Masesha => "lords.masesha",
                DayLord::Muntha => "lords.muntha",
            };
            return Err(Error::invalid_arg(format!(
                "{graha:?} is not one of the seven that hold a day's office"
            ))
            .with_field(field));
        }
    }
    let strengths = rules.strengths(sky)?;
    let malefics = rules.malefics(sky);
    let benefics = rules.benefics(sky);
    let lagna = sign_of_longitude(day_lagna_deg);
    let within = day_lagna_deg.rem_euclid(30.0);
    let houses = House::ALL
        .into_iter()
        .map(|house| {
            let sign = house.sign_from(lagna);
            let amsha_lord = navamsha_lord(f64::from(sign.id()) * 30.0 + within);
            DayHouse {
                house,
                amsha_lord,
                clauses: house_clauses(amsha_lord, sky, &benefics, &malefics),
            }
        })
        .collect();
    let lords = DayLord::ALL
        .into_iter()
        .map(|office| {
            let graha = lords.holder(office);
            let (house, clauses) = lord_clauses(graha, lagna, sky, &strengths, &malefics);
            DayLordPlace {
                office,
                graha,
                house,
                clauses,
            }
        })
        .collect();
    let chart = Placed {
        lagna,
        sky,
        states,
        strengths: &strengths,
        benefics: &benefics,
        malefics: &malefics,
        waning: rules.waning(sky),
    };
    let clauses = DayClause::ALL
        .into_iter()
        .filter(|&clause| chart.holds(clause))
        .collect();
    Ok(DinaPhala {
        moon_state: MoonState::of_longitude(sky.moon_deg),
        houses,
        lords,
        clauses,
    })
}

fn house_clauses(
    amsha_lord: Graha,
    sky: &AnnualSky,
    benefics: &[Graha],
    malefics: &[Graha],
) -> Vec<DayHouseClause> {
    let others = |class: &[Graha], wanted: Bond| {
        class
            .iter()
            .any(|&graha| Bond::between(graha, amsha_lord, sky) == wanted)
    };
    DayHouseClause::ALL
        .into_iter()
        .filter(|clause| match clause {
            DayHouseClause::WithBenefic => others(benefics, Bond::Conjoined),
            DayHouseClause::BeneficFriendly => others(benefics, Bond::Friendly),
            DayHouseClause::MoonFriendly => {
                Bond::between(Graha::Moon, amsha_lord, sky) == Bond::Friendly
            }
            DayHouseClause::WithMalefic => others(malefics, Bond::Conjoined),
            DayHouseClause::MaleficInimical => others(malefics, Bond::Inimical),
        })
        .collect()
}

/// v. 18's houses for the benefics: the kendras, the trikonas, the 11th.
const KENDRA_TRIKONA_AYA: [u8; 7] = [1, 4, 5, 7, 9, 10, 11];

/// v. 18's houses for the malefics: the 3rd, the 6th, the 11th.
const UPACHAYA_OF_MALEFICS: [u8; 3] = [3, 6, 11];

/// The day chart as the whole-chart clauses read it.
struct Placed<'a> {
    lagna: Rashi,
    sky: &'a AnnualSky,
    states: &'a AnnualStates,
    strengths: &'a [Strength; 7],
    benefics: &'a [Graha],
    malefics: &'a [Graha],
    waning: bool,
}

impl Placed<'_> {
    /// The house `graha` stands in, counted from `from`.
    fn house_from(&self, from: Rashi, graha: Graha) -> u8 {
        House::between(from, self.sky.sign_of(graha)).get()
    }

    fn house(&self, graha: Graha) -> u8 {
        self.house_from(self.lagna, graha)
    }

    fn any_in(&self, class: &[Graha], house: u8) -> bool {
        class.iter().any(|&graha| self.house(graha) == house)
    }

    /// Whether every one of `class` but the Moon, when counting from her,
    /// stands in `houses` from `from` and, for the benefics, is strong
    /// (v. 18).
    fn all_placed(&self, class: &[Graha], from_moon: bool, houses: &[u8], strong: bool) -> bool {
        let from = if from_moon {
            self.sky.sign_of(Graha::Moon)
        } else {
            self.lagna
        };
        class
            .iter()
            .filter(|&&graha| !(from_moon && graha == Graha::Moon))
            .all(|&graha| {
                houses.contains(&self.house_from(from, graha))
                    && (!strong
                        || strength_of(graha, self.strengths).is_some_and(Strength::is_strong))
            })
    }

    /// The gloss's scissors: one of `class` moving forward in the 12th and
    /// one moving back in the 2nd.
    fn kartari(&self, class: &[Graha]) -> bool {
        let twelfth = class
            .iter()
            .any(|&graha| self.house(graha) == 12 && !self.states.is_retrograde(graha));
        let second = class
            .iter()
            .any(|&graha| self.house(graha) == 2 && self.states.is_retrograde(graha));
        twelfth && second
    }

    fn holds(&self, clause: DayClause) -> bool {
        let moon = self.house(Graha::Moon);
        match clause {
            DayClause::BeneficsWellPlacedFromLagna => {
                self.all_placed(self.benefics, false, &KENDRA_TRIKONA_AYA, true)
            }
            DayClause::BeneficsWellPlacedFromMoon => {
                self.all_placed(self.benefics, true, &KENDRA_TRIKONA_AYA, true)
            }
            DayClause::MaleficsWellPlacedFromLagna => {
                self.all_placed(self.malefics, false, &UPACHAYA_OF_MALEFICS, false)
            }
            DayClause::MaleficsWellPlacedFromMoon => {
                self.all_placed(self.malefics, true, &UPACHAYA_OF_MALEFICS, false)
            }
            DayClause::MaleficsFlank => {
                self.any_in(self.malefics, 2) && self.any_in(self.malefics, 12)
            }
            DayClause::BeneficTwelfth => self.any_in(self.benefics, 12),
            DayClause::PapaKartari => self.kartari(self.malefics),
            DayClause::ShubhaKartari => self.kartari(self.benefics),
            DayClause::WaningMoonAfflicted => {
                self.waning
                    && matches!(moon, 1 | 8)
                    && self.malefics.iter().any(|&graha| {
                        !matches!(
                            Bond::between(graha, Graha::Moon, self.sky),
                            Bond::Same | Bond::Unaspected
                        )
                    })
            }
            DayClause::MoonMarsDusthana => {
                matches!(moon, 6 | 8)
                    && self.sky.sign_of(Graha::Mars) == self.sky.sign_of(Graha::Moon)
            }
            DayClause::MaleficFourth => self.any_in(self.malefics, 4),
            DayClause::BeneficSeventh => self.any_in(self.benefics, 7),
            DayClause::BeneficSecond => self.any_in(self.benefics, 2),
            DayClause::BeneficNinth => self.any_in(self.benefics, 9),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;

    fn lords() -> DayLords {
        DayLords {
            dinesha: Graha::Venus,
            varshesha: Graha::Sun,
            masesha: Graha::Mars,
            muntha: Graha::Saturn,
        }
    }

    /// Aries rising at 5°; the Moon 120° past the Sun, so waxing.
    fn sky() -> AnnualSky {
        AnnualSky {
            sun_deg: 100.0,
            moon_deg: 220.0,
            mars_deg: 300.0,
            mercury_deg: 110.0,
            jupiter_deg: 20.0,
            venus_deg: 130.0,
            saturn_deg: 190.0,
        }
    }

    fn read(sky: &AnnualSky, states: &AnnualStates) -> DinaPhala {
        dina_phala(5.0, sky, states, &lords(), MasaPhalaRules::default()).unwrap()
    }

    #[test]
    fn the_moons_state_is_the_one_after_those_passed() {
        let at =
            |sign: f64, deg: f64, min: f64, sec: f64| sign * 30.0 + deg + min / 60.0 + sec / 3600.0;
        assert_eq!(
            MoonState::of_longitude(at(2.0, 4.0, 25.0, 40.0)),
            MoonState::Nashta
        );
        assert_eq!(
            MoonState::of_longitude(at(5.0, 0.0, 0.0, 0.0)),
            MoonState::Pravasa
        );
        assert_eq!(
            MoonState::of_longitude(at(5.0, 29.0, 59.0, 0.0)),
            MoonState::Susthita
        );
        assert_eq!(
            MoonState::of_longitude(at(0.0, 2.0, 30.0, 0.0)),
            MoonState::Nashta
        );
        assert_eq!(
            read(&sky(), &AnnualStates::default()).moon_state,
            MoonState::Hasya
        );
    }

    #[test]
    fn the_scissors_close_by_motion() {
        let mut sky = sky();
        // Saturn direct in Pisces (the 12th), Mars in Taurus (the 2nd).
        sky.saturn_deg = 345.0;
        sky.mars_deg = 45.0;
        let direct = read(&sky, &AnnualStates::default());
        assert!(direct.clauses.contains(&DayClause::MaleficsFlank));
        assert!(!direct.clauses.contains(&DayClause::PapaKartari));
        let backward = AnnualStates {
            retrograde: vec![Graha::Mars],
            combust: Vec::new(),
        };
        assert!(
            read(&sky, &backward)
                .clauses
                .contains(&DayClause::PapaKartari)
        );
    }

    #[test]
    fn the_waning_moon_is_a_malefic() {
        let mut sky = sky();
        // The Moon 30° behind the Sun, in the 8th (Scorpio), Saturn
        // beside her.
        sky.sun_deg = 250.0;
        sky.moon_deg = 220.0;
        sky.saturn_deg = 225.0;
        let day = read(&sky, &AnnualStates::default());
        assert!(day.clauses.contains(&DayClause::WaningMoonAfflicted));
        // Waxing, she is a benefic and v. 27 cannot hold.
        sky.sun_deg = 100.0;
        assert!(
            !read(&sky, &AnnualStates::default())
                .clauses
                .contains(&DayClause::WaningMoonAfflicted)
        );
    }

    #[test]
    fn malefics_in_the_third_sixth_and_eleventh_are_a_way_to_good() {
        let mut sky = sky();
        // The Sun in Gemini (3rd), Mars in Virgo (6th), Saturn in Aquarius
        // (11th); the Moon 150° past the Sun, waxing, so a benefic.
        sky.sun_deg = 70.0;
        sky.mercury_deg = 80.0;
        sky.mars_deg = 160.0;
        sky.saturn_deg = 310.0;
        let day = read(&sky, &AnnualStates::default());
        assert!(
            day.clauses
                .contains(&DayClause::MaleficsWellPlacedFromLagna)
        );
        // From the Moon in Scorpio they are the 8th, the 11th and the 4th.
        assert!(!day.clauses.contains(&DayClause::MaleficsWellPlacedFromMoon));
    }

    #[test]
    fn the_moon_with_mars_in_the_sixth_or_eighth() {
        let mut sky = sky();
        sky.mars_deg = 225.0;
        assert!(
            read(&sky, &AnnualStates::default())
                .clauses
                .contains(&DayClause::MoonMarsDusthana)
        );
    }

    #[test]
    fn benefics_in_the_seventh_second_and_ninth() {
        let mut sky = sky();
        // Jupiter in Libra (7th), Venus in Taurus (2nd), Mercury in
        // Sagittarius (9th).
        sky.jupiter_deg = 190.0;
        sky.venus_deg = 40.0;
        sky.mercury_deg = 250.0;
        let day = read(&sky, &AnnualStates::default());
        for clause in [
            DayClause::BeneficSeventh,
            DayClause::BeneficSecond,
            DayClause::BeneficNinth,
        ] {
            assert!(day.clauses.contains(&clause), "{clause:?}");
        }
    }

    #[test]
    fn a_houses_navamsha_lord_is_read_against_both_classes() {
        let day = read(&sky(), &AnnualStates::default());
        let first = &day.houses[0];
        // Aries at 5°: Taurus's navamsha, Venus, in Leo; Jupiter in Aries
        // sees Leo from the 5th, the Sun in Cancer is beside it.
        assert_eq!(first.amsha_lord, Graha::Venus);
        assert!(first.clauses.contains(&DayHouseClause::BeneficFriendly));
        assert!(!first.clauses.contains(&DayHouseClause::WithMalefic));
        assert_eq!(day.houses.len(), 12);
    }

    #[test]
    fn the_lords_are_judged_where_they_stand() {
        let day = read(&sky(), &AnnualStates::default());
        let offices: Vec<_> = day.lords.iter().map(|one| one.office).collect();
        assert_eq!(offices, DayLord::ALL);
        // Venus in Leo, the 5th: well placed.
        assert!(day.lords[0].prospers());
        assert!(!day.lords[0].afflicted());
    }

    #[test]
    fn refusals_name_their_fields() {
        let bad = DayLords {
            dinesha: Graha::Ketu,
            ..lords()
        };
        let why = dina_phala(
            5.0,
            &sky(),
            &AnnualStates::default(),
            &bad,
            MasaPhalaRules::default(),
        )
        .unwrap_err();
        assert_eq!(why.field(), Some("lords.dinesha"));
        let why = dina_phala(
            f64::INFINITY,
            &sky(),
            &AnnualStates::default(),
            &lords(),
            MasaPhalaRules::default(),
        )
        .unwrap_err();
        assert_eq!(why.field(), Some("day_lagna_deg"));
    }

    fn wire<T: Serialize>(value: T) -> String {
        serde_json::to_string(&value).unwrap()
    }

    #[test]
    fn keys_are_serdes() {
        for state in MoonState::ALL {
            assert_eq!(wire(state), format!("\"{}\"", state.key()));
        }
        for clause in DayClause::ALL {
            assert_eq!(wire(clause), format!("\"{}\"", clause.key()));
        }
        for clause in DayHouseClause::ALL {
            assert_eq!(wire(clause), format!("\"{}\"", clause.key()));
        }
        for lord in DayLord::ALL {
            assert_eq!(wire(lord), format!("\"{}\"", lord.key()));
        }
    }
}
