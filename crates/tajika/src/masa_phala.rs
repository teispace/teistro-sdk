//! The **month's results**, read from a month's chart
//! (`03-design/varshesha.md`, "The month's results").
//!
//! *Tajika Nilakanthi*, Varsha Tantra, the chapter on the month's and the
//! day's results, vv. 7–13. The verses work in clauses. Each names a
//! configuration and what the month brings when it holds, several can
//! hold at once, and none is weighed against another, so this module
//! reports the clauses that hold and gives no verdict.
//!
//! Each house is read through two planets: **the house's navamsha lord**,
//! the lord of the navamsha its point falls in (the lagna's degree carried
//! into the house's sign, crux C453), and **the lord's navamsha lord**,
//! the lord of the navamsha the house's sign lord stands in. Verses 7 and
//! 8 read the first house, v. 9 every house the same way, v. 10 the four
//! lords the house's navamsha lord and the Moon see, and v. 11 the house's
//! navamsha lord's strength. Verses 12 and 13 judge the month's lords
//! where they stand.
//!
//! ```
//! use teistro_core::catalogue::Graha;
//! use teistro_tajika::{
//!     AnnualSky, AnnualStates, HouseClause, MasaPhalaRules, MonthLords, masa_phala,
//! };
//!
//! // Aries rising at 5°, its navamsha Taurus's, so Venus is the lagna's
//! // navamsha lord; Mars, the lagna's lord, at 91° stands in Cancer's
//! // navamsha (the Moon's). Venus in Leo sees the Moon in Sagittarius
//! // from the 5th.
//! let sky = AnnualSky {
//!     sun_deg: 200.0, moon_deg: 250.0, mars_deg: 91.0, mercury_deg: 210.0,
//!     jupiter_deg: 20.0, venus_deg: 130.0, saturn_deg: 300.0,
//! };
//! let lords = MonthLords {
//!     varsha_lagna: Graha::Jupiter,
//!     masesha: Graha::Mars,
//!     varshesha: Graha::Sun,
//!     masa_lagna: Graha::Mars,
//!     muntha: Graha::Saturn,
//! };
//! let month = masa_phala(5.0, &sky, &AnnualStates::default(), &lords, MasaPhalaRules::default())?;
//! let first = &month.houses[0];
//! assert_eq!((first.amsha_lord, first.lord_amsha_lord), (Graha::Venus, Graha::Moon));
//! assert!(first.clauses.contains(&HouseClause::Friendly));
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::error::Error;
use teistro_core::house::House;

use crate::bala::{AnnualSky, Bala, SEVEN, finite_longitude, navamsha_lord, sign_of_longitude};
use crate::drishti::Drishti;
use crate::moon::KshinaRule;
use crate::states::AnnualStates;
use crate::yoga::{Strength, YOGA_STRONG_FROM, YOGA_WEAK_BELOW, YogaRules, strengths};

/// A clause of vv. 7–11 about one house, in the verses' order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HouseClause {
    /// The house's navamsha lord is aspected by the lord's navamsha lord
    /// with a friendly aspect (3, 5, 9, 11), or shares its sign (vv. 7, 9).
    Friendly,
    /// The Moon aspects both with a friendly aspect; when she is one of
    /// them, the other (vv. 7, 9; crux C454).
    MoonFriendly,
    /// The two aspect each other with an inimical aspect, the 4th, 7th or
    /// 10th: a shared sign is v. 7's companionship (vv. 8, 9; crux C455).
    Inimical,
    /// The Moon aspects both with an inimical aspect (v. 8).
    MoonInimical,
    /// One is debilitated and the other combust: ease won with effort
    /// (v. 8).
    LowAndSet,
    /// Both are debilitated (v. 8).
    BothLow,
    /// Both are combust (v. 8).
    BothSet,
    /// The two are one planet, so no clause about the pair holds
    /// (crux C456).
    OnePlanet,
    /// The house's navamsha lord is strong (v. 11).
    Strong,
    /// The house's navamsha lord is weak (v. 11).
    Weak,
}

impl HouseClause {
    /// Every clause, in the verses' order.
    pub const ALL: [HouseClause; 10] = [
        HouseClause::Friendly,
        HouseClause::MoonFriendly,
        HouseClause::Inimical,
        HouseClause::MoonInimical,
        HouseClause::LowAndSet,
        HouseClause::BothLow,
        HouseClause::BothSet,
        HouseClause::OnePlanet,
        HouseClause::Strong,
        HouseClause::Weak,
    ];

    /// The member's key, as serde writes it: `MOON_FRIENDLY`.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            HouseClause::Friendly => "FRIENDLY",
            HouseClause::MoonFriendly => "MOON_FRIENDLY",
            HouseClause::Inimical => "INIMICAL",
            HouseClause::MoonInimical => "MOON_INIMICAL",
            HouseClause::LowAndSet => "LOW_AND_SET",
            HouseClause::BothLow => "BOTH_LOW",
            HouseClause::BothSet => "BOTH_SET",
            HouseClause::OnePlanet => "ONE_PLANET",
            HouseClause::Strong => "STRONG",
            HouseClause::Weak => "WEAK",
        }
    }
}

/// The month's lords the results read, in the order the verses name them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MonthLord {
    /// The lord of the year chart's lagna (vv. 10, 12, 13).
    VarshaLagna,
    /// The Masesha, the month's lord (vv. 10, 12, 13).
    Masesha,
    /// The Varshesha, the year's lord (vv. 10, 12, 13).
    Varshesha,
    /// The lord of the month chart's lagna (v. 10).
    MasaLagna,
    /// The lord of the Muntha's sign this month (v. 12).
    Muntha,
}

impl MonthLord {
    /// All five, in the verses' order.
    pub const ALL: [MonthLord; 5] = [
        MonthLord::VarshaLagna,
        MonthLord::Masesha,
        MonthLord::Varshesha,
        MonthLord::MasaLagna,
        MonthLord::Muntha,
    ];

    /// The four v. 10 asks the house's navamsha lord and the Moon to see.
    pub const SEEN: [MonthLord; 4] = [
        MonthLord::VarshaLagna,
        MonthLord::Masesha,
        MonthLord::Varshesha,
        MonthLord::MasaLagna,
    ];

    /// The member's key, as serde writes it: `VARSHA_LAGNA`.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            MonthLord::VarshaLagna => "VARSHA_LAGNA",
            MonthLord::Masesha => "MASESHA",
            MonthLord::Varshesha => "VARSHESHA",
            MonthLord::MasaLagna => "MASA_LAGNA",
            MonthLord::Muntha => "MUNTHA",
        }
    }

    /// Whether v. 12 judges this lord.
    #[must_use]
    pub const fn judged_by_affliction(self) -> bool {
        !matches!(self, MonthLord::MasaLagna)
    }

    /// Whether v. 13 judges this lord.
    #[must_use]
    pub const fn judged_by_prosperity(self) -> bool {
        matches!(
            self,
            MonthLord::VarshaLagna | MonthLord::Masesha | MonthLord::Varshesha
        )
    }
}

/// Who holds each of the five offices this month.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct MonthLords {
    /// The lord of the year chart's lagna.
    pub varsha_lagna: Graha,
    /// The month's lord.
    pub masesha: Graha,
    /// The year's lord.
    pub varshesha: Graha,
    /// The lord of the month chart's lagna.
    pub masa_lagna: Graha,
    /// The lord of the Muntha's sign this month.
    pub muntha: Graha,
}

impl MonthLords {
    /// Who holds one office.
    #[must_use]
    pub const fn holder(&self, office: MonthLord) -> Graha {
        match office {
            MonthLord::VarshaLagna => self.varsha_lagna,
            MonthLord::Masesha => self.masesha,
            MonthLord::Varshesha => self.varshesha,
            MonthLord::MasaLagna => self.masa_lagna,
            MonthLord::Muntha => self.muntha,
        }
    }
}

/// A clause of vv. 12–13 about one of the month's lords, in the verses'
/// order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LordClause {
    /// In the 6th, 8th or 12th from the month lagna (v. 12).
    Trika,
    /// Sharing a sign with one of the chapter's malefics, the Sun, Mars,
    /// Saturn and the waning Moon, itself excepted (v. 12; crux C460).
    WithMalefic,
    /// Aspected by one of them with an inimical aspect, the 1st, 4th, 7th
    /// or 10th as the gloss lists them (v. 12; cruxes C455, C460).
    MaleficInimical,
    /// Strong, by dignity or Vishwa bala (v. 13).
    Strong,
    /// In a kendra, a trikona or the 11th (v. 13).
    WellPlaced,
}

impl LordClause {
    /// Every clause, in the verses' order.
    pub const ALL: [LordClause; 5] = [
        LordClause::Trika,
        LordClause::WithMalefic,
        LordClause::MaleficInimical,
        LordClause::Strong,
        LordClause::WellPlaced,
    ];

    /// The member's key, as serde writes it: `WITH_MALEFIC`.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            LordClause::Trika => "TRIKA",
            LordClause::WithMalefic => "WITH_MALEFIC",
            LordClause::MaleficInimical => "MALEFIC_INIMICAL",
            LordClause::Strong => "STRONG",
            LordClause::WellPlaced => "WELL_PLACED",
        }
    }
}

/// One house of the month, read through its two planets.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct MonthHouse {
    /// The house, counted by whole signs from the month lagna.
    pub house: House,
    /// The lord of the navamsha the house's point falls in.
    pub amsha_lord: Graha,
    /// The lord of the navamsha the house's sign lord stands in.
    pub lord_amsha_lord: Graha,
    /// The clauses of vv. 7–11 that hold, in the verses' order.
    pub clauses: Vec<HouseClause>,
    /// The lords of v. 10 the house's navamsha lord aspects with a
    /// friendly aspect or shares a sign with, itself excepted.
    pub seen_by_amsha_lord: Vec<MonthLord>,
    /// The lords of v. 10 the Moon aspects with a friendly aspect or
    /// shares a sign with, herself excepted.
    pub seen_by_moon: Vec<MonthLord>,
}

impl MonthHouse {
    /// Whether v. 11 reads good from the house's navamsha lord: weak for
    /// the 6th, 8th and 12th, strong for the others.
    #[must_use]
    pub fn fruitful(&self) -> bool {
        let wanted = if self.house.is_trika() {
            HouseClause::Weak
        } else {
            HouseClause::Strong
        };
        self.clauses.contains(&wanted)
    }

    /// Whether v. 11 reads ill: the reverse of [`MonthHouse::fruitful`].
    /// A middling lord is neither.
    #[must_use]
    pub fn unfruitful(&self) -> bool {
        let wanted = if self.house.is_trika() {
            HouseClause::Strong
        } else {
            HouseClause::Weak
        };
        self.clauses.contains(&wanted)
    }
}

/// Where one of the month's lords stands.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct MonthLordPlace {
    /// The office.
    pub office: MonthLord,
    /// Who holds it.
    pub graha: Graha,
    /// The house it stands in, by whole signs from the month lagna.
    pub house: House,
    /// The clauses of vv. 12–13 that hold, in the verses' order.
    pub clauses: Vec<LordClause>,
}

impl MonthLordPlace {
    /// Whether v. 12 holds: one of its four lords in the 6th, 8th or 12th
    /// with a malefic or aspected by one inimically (crux C457).
    #[must_use]
    pub fn afflicted(&self) -> bool {
        self.office.judged_by_affliction()
            && self.clauses.contains(&LordClause::Trika)
            && (self.clauses.contains(&LordClause::WithMalefic)
                || self.clauses.contains(&LordClause::MaleficInimical))
    }

    /// Whether v. 13 holds: one of its three lords strong and in a kendra,
    /// a trikona or the 11th.
    #[must_use]
    pub fn prospers(&self) -> bool {
        self.office.judged_by_prosperity()
            && self.clauses.contains(&LordClause::Strong)
            && self.clauses.contains(&LordClause::WellPlaced)
    }
}

/// The month's results: its twelve houses and its five lords.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MasaPhala {
    /// The twelve houses, first to twelfth.
    pub houses: Vec<MonthHouse>,
    /// The five lords, in [`MonthLord::ALL`]'s order.
    pub lords: Vec<MonthLordPlace>,
}

/// The floors the month's and the day's results judge strong and weak by,
/// the year yogas' own by default, and when the Moon is waning.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct MasaPhalaRules {
    /// The Vishwa bala below which a planet with no dignity is weak.
    pub weak_below: Bala,
    /// The Vishwa bala from which a planet is strong without a dignity;
    /// never below `weak_below`.
    pub strong_from: Bala,
    /// When the Moon is waning, and so one of the chapter's malefics
    /// rather than its benefics (crux C460); the Samjna Tantra gloss's own
    /// by default.
    pub kshina: KshinaRule,
}

impl Default for MasaPhalaRules {
    fn default() -> MasaPhalaRules {
        MasaPhalaRules {
            weak_below: YOGA_WEAK_BELOW,
            strong_from: YOGA_STRONG_FROM,
            kshina: KshinaRule::default(),
        }
    }
}

impl MasaPhalaRules {
    /// Refuses floors that would let a planet be strong and weak at once.
    ///
    /// # Errors
    ///
    /// `strongFrom` below `weakBelow`.
    pub fn check(self) -> Result<MasaPhalaRules, Error> {
        if self.strong_from < self.weak_below {
            return Err(Error::invalid_arg(format!(
                "strength begins at {} but weakness runs up to {}: a planet between would be both",
                self.strong_from, self.weak_below
            ))
            .with_field("strongFrom")
            .with_hint(format!(
                "raise strongFrom to at least {}, or lower weakBelow",
                self.weak_below
            )));
        }
        Ok(self)
    }

    /// The seven's strengths under these floors.
    pub(crate) fn strengths(self, sky: &AnnualSky) -> Result<[Strength; 7], Error> {
        strengths(
            sky,
            YogaRules {
                weak_below: self.weak_below,
                strong_from: self.strong_from,
                ..YogaRules::default()
            },
        )
    }

    /// Whether the Moon is waning in `sky`.
    pub(crate) fn waning(self, sky: &AnnualSky) -> bool {
        self.kshina
            .holds((sky.moon_deg - sky.sun_deg).rem_euclid(360.0))
    }

    /// The chapter's malefics in `sky`: the Sun, Mars, Saturn, and the Moon
    /// when she is waning (the gloss on v. 28; crux C460).
    pub(crate) fn malefics(self, sky: &AnnualSky) -> Vec<Graha> {
        let mut out = vec![Graha::Sun, Graha::Mars, Graha::Saturn];
        if self.waning(sky) {
            out.push(Graha::Moon);
        }
        out
    }

    /// The chapter's benefics in `sky`, the rest of the seven: Mercury,
    /// Jupiter, Venus, and the Moon when she is not waning (crux C460).
    pub(crate) fn benefics(self, sky: &AnnualSky) -> Vec<Graha> {
        let mut out = vec![Graha::Mercury, Graha::Jupiter, Graha::Venus];
        if !self.waning(sky) {
            out.push(Graha::Moon);
        }
        out
    }
}

/// What two planets' signs make of them in vv. 7–10.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Bond {
    /// One planet.
    Same,
    /// Sharing a sign: v. 7's companionship.
    Conjoined,
    /// The 3rd, 5th, 9th or 11th.
    Friendly,
    /// The 4th, 7th or 10th.
    Inimical,
    /// The 2nd, 6th, 8th or 12th.
    Unaspected,
}

impl Bond {
    pub(crate) fn between(a: Graha, b: Graha, sky: &AnnualSky) -> Bond {
        if a == b {
            return Bond::Same;
        }
        match Drishti::between_signs(sky.sign_of(a), sky.sign_of(b)) {
            Drishti::Inimical if sky.sign_of(a) == sky.sign_of(b) => Bond::Conjoined,
            Drishti::Friendly | Drishti::SecretlyFriendly => Bond::Friendly,
            Drishti::Inimical | Drishti::SecretlyInimical => Bond::Inimical,
            Drishti::None => Bond::Unaspected,
        }
    }

    /// v. 10's "seen with a friendly aspect or joined".
    pub(crate) const fn sees(self) -> bool {
        matches!(self, Bond::Conjoined | Bond::Friendly)
    }
}

/// Whether the Moon holds `bond` to every member of the pair that is not
/// herself, and there is one (crux C454).
fn moon_holds(pair: [Graha; 2], sky: &AnnualSky, bond: Bond) -> bool {
    let others: Vec<Graha> = pair
        .into_iter()
        .filter(|&graha| graha != Graha::Moon)
        .collect();
    !others.is_empty()
        && others
            .iter()
            .all(|&graha| Bond::between(Graha::Moon, graha, sky) == bond)
}

pub(crate) fn debilitated(graha: Graha, sky: &AnnualSky) -> bool {
    graha
        .attributes()
        .debilitation
        .is_some_and(|at| at.sign == sky.sign_of(graha))
}

pub(crate) fn strength_of(graha: Graha, strengths: &[Strength; 7]) -> Option<Strength> {
    strengths.iter().copied().find(|one| one.graha == graha)
}

/// The month's results from its chart: `month_lagna_deg` and `sky` are the
/// month chart's, `states` its combust and retrograde planets, `lords` who
/// holds the five offices this month.
///
/// # Errors
///
/// A longitude that is not a number, named by its field
/// (`month_lagna_deg` for the lagna); states no chart can hold, as
/// [`AnnualStates::check`]; a lord outside the seven, named
/// `lords.masesha` and so on; floors [`MasaPhalaRules::check`] refuses.
pub fn masa_phala(
    month_lagna_deg: f64,
    sky: &AnnualSky,
    states: &AnnualStates,
    lords: &MonthLords,
    rules: MasaPhalaRules,
) -> Result<MasaPhala, Error> {
    sky.check()?;
    finite_longitude("month_lagna_deg", month_lagna_deg)?;
    states.validate()?;
    let rules = rules.check()?;
    for office in MonthLord::ALL {
        let graha = lords.holder(office);
        if !SEVEN.contains(&graha) {
            let field = match office {
                MonthLord::VarshaLagna => "lords.varshaLagna",
                MonthLord::Masesha => "lords.masesha",
                MonthLord::Varshesha => "lords.varshesha",
                MonthLord::MasaLagna => "lords.masaLagna",
                MonthLord::Muntha => "lords.muntha",
            };
            return Err(Error::invalid_arg(format!(
                "{graha:?} is not one of the seven that hold a month's office"
            ))
            .with_field(field));
        }
    }
    let strengths = rules.strengths(sky)?;
    let malefics = rules.malefics(sky);
    let lagna = sign_of_longitude(month_lagna_deg);
    let within = month_lagna_deg.rem_euclid(30.0);
    let houses = House::ALL
        .into_iter()
        .map(|house| month_house(house, lagna, within, sky, states, lords, &strengths))
        .collect();
    let lords = MonthLord::ALL
        .into_iter()
        .map(|office| {
            let graha = lords.holder(office);
            let (house, clauses) = lord_clauses(graha, lagna, sky, &strengths, &malefics);
            MonthLordPlace {
                office,
                graha,
                house,
                clauses,
            }
        })
        .collect();
    Ok(MasaPhala { houses, lords })
}

fn month_house(
    house: House,
    lagna: Rashi,
    within_deg: f64,
    sky: &AnnualSky,
    states: &AnnualStates,
    lords: &MonthLords,
    strengths: &[Strength; 7],
) -> MonthHouse {
    let sign = house.sign_from(lagna);
    // The house's point is the lagna's degree carried into its sign
    // (crux C453).
    let amsha_lord = navamsha_lord(f64::from(sign.id()) * 30.0 + within_deg);
    let lord_amsha_lord = navamsha_lord(sky.longitude_of(sign.attributes().lord));
    let pair = [amsha_lord, lord_amsha_lord];
    let bond = Bond::between(amsha_lord, lord_amsha_lord, sky);
    let low = pair.map(|graha| debilitated(graha, sky));
    let set = pair.map(|graha| states.is_combust(graha));
    let strength = strength_of(amsha_lord, strengths);
    let mut clauses = Vec::new();
    for clause in HouseClause::ALL {
        let holds = match clause {
            HouseClause::Friendly => bond.sees(),
            HouseClause::MoonFriendly => moon_holds(pair, sky, Bond::Friendly),
            HouseClause::Inimical => bond == Bond::Inimical,
            HouseClause::MoonInimical => moon_holds(pair, sky, Bond::Inimical),
            HouseClause::LowAndSet => {
                bond != Bond::Same && ((low[0] && set[1]) || (set[0] && low[1]))
            }
            HouseClause::BothLow => low[0] && low[1],
            HouseClause::BothSet => set[0] && set[1],
            HouseClause::OnePlanet => bond == Bond::Same,
            HouseClause::Strong => strength.is_some_and(Strength::is_strong),
            HouseClause::Weak => strength.is_some_and(Strength::is_weak),
        };
        if holds {
            clauses.push(clause);
        }
    }
    let seen = |by: Graha| {
        MonthLord::SEEN
            .into_iter()
            .filter(|&office| Bond::between(by, lords.holder(office), sky).sees())
            .collect()
    };
    MonthHouse {
        house,
        amsha_lord,
        lord_amsha_lord,
        clauses,
        seen_by_amsha_lord: seen(amsha_lord),
        seen_by_moon: seen(Graha::Moon),
    }
}

/// Where `graha` stands from `lagna`, and the clauses of vv. 12–13 (and of
/// the day's v. 19) that hold of it, `malefics` the chapter's own.
pub(crate) fn lord_clauses(
    graha: Graha,
    lagna: Rashi,
    sky: &AnnualSky,
    strengths: &[Strength; 7],
    malefics: &[Graha],
) -> (House, Vec<LordClause>) {
    let house = House::between(lagna, sky.sign_of(graha));
    let malefics = malefics.iter().copied().filter(|&malefic| malefic != graha);
    let mut clauses = Vec::new();
    for clause in LordClause::ALL {
        let holds = match clause {
            LordClause::Trika => house.is_trika(),
            LordClause::WithMalefic => malefics
                .clone()
                .any(|malefic| sky.sign_of(malefic) == sky.sign_of(graha)),
            // The gloss's 1, 4, 7, 10: here a shared sign is inimical
            // (crux C455).
            LordClause::MaleficInimical => malefics.clone().any(|malefic| {
                matches!(
                    Drishti::between_signs(sky.sign_of(malefic), sky.sign_of(graha)),
                    Drishti::Inimical | Drishti::SecretlyInimical
                )
            }),
            LordClause::Strong => strength_of(graha, strengths).is_some_and(Strength::is_strong),
            LordClause::WellPlaced => matches!(house.get(), 1 | 4 | 5 | 7 | 9 | 10 | 11),
        };
        if holds {
            clauses.push(clause);
        }
    }
    (house, clauses)
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

    fn lords() -> MonthLords {
        MonthLords {
            varsha_lagna: Graha::Jupiter,
            masesha: Graha::Mars,
            varshesha: Graha::Sun,
            masa_lagna: Graha::Mars,
            muntha: Graha::Saturn,
        }
    }

    /// Aries rising at 5°: the lagna's navamsha is Taurus's, Venus's;
    /// Mars at 91° stands in Cancer's navamsha, the Moon's.
    fn sky() -> AnnualSky {
        AnnualSky {
            sun_deg: 200.0,
            moon_deg: 250.0,
            mars_deg: 91.0,
            mercury_deg: 210.0,
            jupiter_deg: 20.0,
            venus_deg: 130.0,
            saturn_deg: 300.0,
        }
    }

    fn read(sky: &AnnualSky, states: &AnnualStates) -> MasaPhala {
        masa_phala(5.0, sky, states, &lords(), MasaPhalaRules::default()).unwrap()
    }

    #[test]
    fn the_first_house_is_read_through_the_lagnas_navamsha_lord_and_its_lords() {
        let month = read(&sky(), &AnnualStates::default());
        assert_eq!(month.houses.len(), 12);
        let first = &month.houses[0];
        assert_eq!(first.house.get(), 1);
        // Venus (Leo) and the Moon (Sagittarius) are 5th from each other,
        // and the Moon is herself one of the pair.
        assert_eq!(
            (first.amsha_lord, first.lord_amsha_lord),
            (Graha::Venus, Graha::Moon)
        );
        assert!(first.clauses.contains(&HouseClause::Friendly));
        assert!(first.clauses.contains(&HouseClause::MoonFriendly));
        assert!(!first.clauses.contains(&HouseClause::Inimical));
    }

    #[test]
    fn a_shared_sign_is_companionship_and_a_square_is_enmity() {
        let mut sky = sky();
        // The Moon beside Venus in Leo: conjoined, and the Moon one of the
        // pair, so she has no other to see but Venus, whom she joins.
        sky.moon_deg = 125.0;
        let first = &read(&sky, &AnnualStates::default()).houses[0];
        assert!(first.clauses.contains(&HouseClause::Friendly));
        assert!(!first.clauses.contains(&HouseClause::MoonFriendly));
        // The Moon in Scorpio, 4th from Venus.
        sky.moon_deg = 220.0;
        let first = &read(&sky, &AnnualStates::default()).houses[0];
        assert!(first.clauses.contains(&HouseClause::Inimical));
        assert!(first.clauses.contains(&HouseClause::MoonInimical));
        assert!(!first.clauses.contains(&HouseClause::Friendly));
    }

    #[test]
    fn one_planet_in_both_roles_holds_no_pair_clause() {
        let mut sky = sky();
        // Mars at 5° of Aries stands in the lagna's own navamsha, Venus's.
        sky.mars_deg = 5.0;
        let first = &read(&sky, &AnnualStates::default()).houses[0];
        assert_eq!(first.lord_amsha_lord, Graha::Venus);
        assert_eq!(first.amsha_lord, Graha::Venus);
        assert!(first.clauses.contains(&HouseClause::OnePlanet));
        assert!(!first.clauses.contains(&HouseClause::Friendly));
        assert!(!first.clauses.contains(&HouseClause::Inimical));
    }

    #[test]
    fn debility_and_combustion_are_read_on_both_roles() {
        let mut sky = sky();
        // Venus debilitated in Virgo; the Moon combust.
        sky.venus_deg = 160.0;
        let states = AnnualStates {
            retrograde: Vec::new(),
            combust: vec![Graha::Moon],
        };
        let first = &read(&sky, &states).houses[0];
        assert!(first.clauses.contains(&HouseClause::LowAndSet));
        assert!(!first.clauses.contains(&HouseClause::BothLow));
        let both = AnnualStates {
            retrograde: Vec::new(),
            combust: vec![Graha::Moon, Graha::Venus],
        };
        assert!(
            read(&sky, &both).houses[0]
                .clauses
                .contains(&HouseClause::BothSet)
        );
    }

    #[test]
    fn verse_eleven_reverses_for_the_trika() {
        let month = read(&sky(), &AnnualStates::default());
        for house in &month.houses {
            assert!(!(house.fruitful() && house.unfruitful()));
            let strong = house.clauses.contains(&HouseClause::Strong);
            assert_eq!(
                house.fruitful() && strong,
                strong && !house.house.is_trika()
            );
        }
    }

    #[test]
    fn the_lords_are_judged_where_they_stand() {
        let mut sky = sky();
        // The Sun (the Varshesha) in Virgo, the 6th from Aries, with
        // Saturn: verse 12.
        sky.sun_deg = 165.0;
        sky.saturn_deg = 170.0;
        let month = read(&sky, &AnnualStates::default());
        assert_eq!(month.lords.len(), 5);
        let sun = month
            .lords
            .iter()
            .find(|one| one.office == MonthLord::Varshesha)
            .unwrap();
        assert_eq!((sun.graha, sun.house.get()), (Graha::Sun, 6));
        assert!(sun.clauses.contains(&LordClause::WithMalefic));
        assert!(sun.clauses.contains(&LordClause::MaleficInimical));
        assert!(sun.afflicted());
        // The month lagna's lord is not one of v. 12's four.
        let masa = MonthLordPlace {
            office: MonthLord::MasaLagna,
            ..sun.clone()
        };
        assert!(!masa.afflicted());
        // Jupiter (the year lagna's lord) exalted in Cancer, the 4th.
        sky.jupiter_deg = 95.0;
        let jupiter = read(&sky, &AnnualStates::default()).lords[0].clone();
        assert!(jupiter.clauses.contains(&LordClause::Strong));
        assert!(jupiter.prospers());
    }

    #[test]
    fn verse_ten_lists_the_lords_each_sees() {
        let month = read(&sky(), &AnnualStates::default());
        let first = &month.houses[0];
        // Venus in Leo sees Jupiter in Aries from the 5th, and Mars in
        // Cancer from nowhere (the 12th).
        assert!(first.seen_by_amsha_lord.contains(&MonthLord::VarshaLagna));
        assert!(!first.seen_by_amsha_lord.contains(&MonthLord::Masesha));
        // The Moon in Sagittarius sees Jupiter in Aries from the 5th.
        assert!(first.seen_by_moon.contains(&MonthLord::VarshaLagna));
    }

    #[test]
    fn refusals_name_their_fields() {
        let bad = MonthLords {
            muntha: Graha::Rahu,
            ..lords()
        };
        let why = masa_phala(
            5.0,
            &sky(),
            &AnnualStates::default(),
            &bad,
            MasaPhalaRules::default(),
        )
        .unwrap_err();
        assert_eq!(why.field(), Some("lords.muntha"));
        let floors = MasaPhalaRules {
            weak_below: Bala::new(9, 0, 0),
            strong_from: Bala::new(8, 0, 0),
            ..MasaPhalaRules::default()
        };
        let why = masa_phala(5.0, &sky(), &AnnualStates::default(), &lords(), floors).unwrap_err();
        assert_eq!(why.field(), Some("strongFrom"));
        let why = masa_phala(
            f64::NAN,
            &sky(),
            &AnnualStates::default(),
            &lords(),
            MasaPhalaRules::default(),
        )
        .unwrap_err();
        assert_eq!(why.field(), Some("month_lagna_deg"));
    }

    #[test]
    fn keys_are_serdes() {
        for clause in HouseClause::ALL {
            assert_eq!(
                serde_json::to_string(&clause).unwrap(),
                format!("\"{}\"", clause.key())
            );
        }
        for clause in LordClause::ALL {
            assert_eq!(
                serde_json::to_string(&clause).unwrap(),
                format!("\"{}\"", clause.key())
            );
        }
        for office in MonthLord::ALL {
            assert_eq!(
                serde_json::to_string(&office).unwrap(),
                format!("\"{}\"", office.key())
            );
        }
    }
}
