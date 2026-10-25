//! **Kamboola's grades**: the office (*adhikāra*) each of the Moon, the
//! lagnesha and the karyesha holds, as Tajika Nilakanthi's Samjna Tantra
//! vv. 36–46 and their gloss grade a Kamboola (1893, leaves n50–n56), and
//! the Prashna Tantra's v. 21 repeats for a query (n204–n205).
//!
//! The gloss (n51) gives four offices:
//!
//! - **uttama**: in its own sign or its exaltation;
//! - **madhyama**: in its own hudda, drekkāṇa or navāṁśa;
//! - **sama**: none of these;
//! - **adhama**: in its debilitation or an enemy's sign.
//!
//! A Kamboola is named by two of them, the Moon's first and the pair's
//! second: *uttamottama* … *adhamādhama*, sixteen cells.
//!
//! **A graha may hold several offices at once, and the definition does not
//! say which one counts** (crux C371). The gloss's eleven worked examples
//! do: read in this order, debilitation, a division, own sign or
//! exaltation, an enemy's sign, they reproduce nine of the twelve the
//! acceptance test grades, where "the highest office counts" reproduces
//! six. The order is [`OfficeOrder::Gloss`], and the rivals are kept by
//! name. Its drekkāṇa is the Tajika one the same print's Prashna Tantra
//! gloss defines (n205), which the examples bear out over the Parashari
//! one, 9 to 7.
//!
//! **The pair's office is named only when the two lords hold the same
//! one** (crux C372): every printed example gives them one office, and
//! none grades two lords that differ. Both lords' offices are reported
//! either way.
//!
//! ```
//! use teistro_core::catalogue::Graha;
//! use teistro_tajika::{Adhikara, AnnualSky, OfficeRules, kamboola_grade};
//!
//! // The gloss's example to v. 39 (n53): Venus and Mars, the lords of
//! // Libra rising and its 7th, each in its debilitation; the Moon in
//! // Cancer, her own sign.
//! let sky = AnnualSky {
//!     sun_deg: 15.0, moon_deg: 102.0, mars_deg: 105.0, mercury_deg: 195.0,
//!     jupiter_deg: 15.0, venus_deg: 160.0, saturn_deg: 15.0,
//! };
//! let grade = kamboola_grade(Graha::Venus, Graha::Mars, &sky, OfficeRules::default())?;
//! assert_eq!(grade.cell(), Some([Adhikara::Uttama, Adhikara::Adhama]));
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, SignDegree};
use teistro_core::error::Error;

use crate::bala::{
    AnnualSky, HuddaTable, Relation, SEVEN, drekkana_lord, hudda_lord_in, navamsha_lord,
    parashari_drekkana_lord,
};

/// One of the four offices, best first.
///
/// ```
/// # use teistro_core::catalogue::Graha;
/// # use teistro_tajika::{AnnualSky, OfficeRules, offices};
/// #
/// # let sky = AnnualSky {
/// #     sun_deg: 15.0, moon_deg: 102.0, mars_deg: 105.0, mercury_deg: 195.0,
/// #     jupiter_deg: 15.0, venus_deg: 160.0, saturn_deg: 15.0,
/// # };
///
/// use teistro_tajika::Adhikara;
///
/// assert_eq!(offices(Graha::Moon, &sky, OfficeRules::default())?.adhikara, Adhikara::Uttama);
/// assert_eq!(offices(Graha::Mars, &sky, OfficeRules::default())?.adhikara, Adhikara::Adhama);
/// // Best first.
/// assert!(Adhikara::Uttama < Adhikara::Adhama);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Adhikara {
    /// Own sign or exaltation.
    Uttama,
    /// Own hudda, drekkāṇa or navāṁśa.
    Madhyama,
    /// None of the offices.
    Sama,
    /// Debilitation or an enemy's sign.
    Adhama,
}

impl Adhikara {
    /// The member's key, as serde writes it and every binding reads it
    /// back: `UTTAMA`.
    ///
    /// ```
    /// use teistro_tajika::Adhikara;
    ///
    /// assert_eq!(Adhikara::Madhyama.key(), "MADHYAMA");
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Adhikara::Uttama => "UTTAMA",
            Adhikara::Madhyama => "MADHYAMA",
            Adhikara::Sama => "SAMA",
            Adhikara::Adhama => "ADHAMA",
        }
    }

    /// Every office, best first.
    ///
    /// ```
    /// use teistro_tajika::Adhikara;
    ///
    /// let keys = Adhikara::ALL.map(Adhikara::key);
    /// assert_eq!(keys, ["UTTAMA", "MADHYAMA", "SAMA", "ADHAMA"]);
    /// ```
    pub const ALL: [Adhikara; 4] = [
        Adhikara::Uttama,
        Adhikara::Madhyama,
        Adhikara::Sama,
        Adhikara::Adhama,
    ];
}

/// One clause of the gloss's definition that a graha meets.
///
/// ```
/// # use teistro_core::catalogue::Graha;
/// # use teistro_tajika::{AnnualSky, OfficeRules, offices};
/// #
/// # let sky = AnnualSky {
/// #     sun_deg: 15.0, moon_deg: 102.0, mars_deg: 105.0, mercury_deg: 195.0,
/// #     jupiter_deg: 15.0, venus_deg: 160.0, saturn_deg: 15.0,
/// # };
///
/// use teistro_tajika::OfficeClause;
///
/// // Mars in Cancer, his debilitation.
/// let mars = offices(Graha::Mars, &sky, OfficeRules::default())?;
/// assert!(mars.held.contains(&OfficeClause::Debilitated));
/// assert!(!mars.held.contains(&OfficeClause::OwnSign));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OfficeClause {
    /// In its own sign: uttama.
    OwnSign,
    /// In its exaltation sign: uttama.
    Exalted,
    /// In its own hudda, by [`OfficeRules::hudda`]: madhyama.
    OwnHudda,
    /// In its own drekkāṇa, by [`OfficeRules::drekkana`]: madhyama.
    OwnDrekkana,
    /// In its own navāṁśa: madhyama.
    OwnNavamsha,
    /// In its debilitation sign: adhama.
    Debilitated,
    /// In a sign whose lord stands to it as [`Relation::Enemy`], under
    /// [`EnemySign::Tajika`]: adhama.
    EnemySign,
}

impl OfficeClause {
    /// The member's key, as serde writes it and every binding reads it
    /// back: `OWN_SIGN`.
    ///
    /// ```
    /// use teistro_tajika::OfficeClause;
    ///
    /// assert_eq!(OfficeClause::OwnNavamsha.key(), "OWN_NAVAMSHA");
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            OfficeClause::OwnSign => "OWN_SIGN",
            OfficeClause::Exalted => "EXALTED",
            OfficeClause::OwnHudda => "OWN_HUDDA",
            OfficeClause::OwnDrekkana => "OWN_DREKKANA",
            OfficeClause::OwnNavamsha => "OWN_NAVAMSHA",
            OfficeClause::Debilitated => "DEBILITATED",
            OfficeClause::EnemySign => "ENEMY_SIGN",
        }
    }

    /// Every clause, in the definition's order.
    ///
    /// ```
    /// use teistro_tajika::{Adhikara, OfficeClause};
    ///
    /// // Two give uttama, three madhyama, two adhama; none gives sama.
    /// let giving = |office| OfficeClause::ALL.iter().filter(|clause| clause.office() == office).count();
    /// assert_eq!([Adhikara::Uttama, Adhikara::Madhyama, Adhikara::Sama, Adhikara::Adhama].map(giving), [2, 3, 0, 2]);
    /// ```
    pub const ALL: [OfficeClause; 7] = [
        OfficeClause::OwnSign,
        OfficeClause::Exalted,
        OfficeClause::OwnHudda,
        OfficeClause::OwnDrekkana,
        OfficeClause::OwnNavamsha,
        OfficeClause::Debilitated,
        OfficeClause::EnemySign,
    ];

    /// The office the clause gives on its own.
    ///
    /// ```
    /// use teistro_tajika::{Adhikara, OfficeClause};
    ///
    /// assert_eq!(OfficeClause::Exalted.office(), Adhikara::Uttama);
    /// assert_eq!(OfficeClause::OwnHudda.office(), Adhikara::Madhyama);
    /// assert_eq!(OfficeClause::EnemySign.office(), Adhikara::Adhama);
    /// ```
    #[must_use]
    pub const fn office(self) -> Adhikara {
        match self {
            OfficeClause::OwnSign | OfficeClause::Exalted => Adhikara::Uttama,
            OfficeClause::OwnHudda | OfficeClause::OwnDrekkana | OfficeClause::OwnNavamsha => {
                Adhikara::Madhyama
            }
            OfficeClause::Debilitated | OfficeClause::EnemySign => Adhikara::Adhama,
        }
    }
}

/// Which office counts when a graha holds more than one (crux C371).
///
/// ```
/// # use teistro_core::catalogue::Graha;
/// # use teistro_tajika::{AnnualSky, OfficeRules, offices};
/// #
/// # let sky = AnnualSky {
/// #     sun_deg: 15.0, moon_deg: 102.0, mars_deg: 105.0, mercury_deg: 195.0,
/// #     jupiter_deg: 15.0, venus_deg: 160.0, saturn_deg: 15.0,
/// # };
///
/// use teistro_tajika::{Adhikara, OfficeOrder};
///
/// let early = AnnualSky { moon_deg: 91.0, ..sky };
/// let read = |order| offices(Graha::Moon, &early, OfficeRules { order, ..OfficeRules::default() });
/// // The gloss's vv. 40-41: the division counts before her own sign.
/// assert_eq!(read(OfficeOrder::Gloss)?.adhikara, Adhikara::Madhyama);
/// assert_eq!(read(OfficeOrder::Highest)?.adhikara, Adhikara::Uttama);
/// assert_eq!(read(OfficeOrder::Lowest)?.adhikara, Adhikara::Madhyama);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OfficeOrder {
    /// The order the gloss's worked examples follow: debilitation, then a
    /// division it rules, then its own sign or exaltation, then an
    /// enemy's sign. A debilitated graha in its own navāṁśa is adhama (the
    /// examples to vv. 39 and 45), the Moon in Cancer in her own navāṁśa
    /// madhyama (vv. 40, 41), Venus in her own hudda in her enemy the
    /// Moon's sign madhyama (v. 38).
    #[default]
    Gloss,
    /// The best office it holds.
    Highest,
    /// The worst office it holds.
    Lowest,
}

impl OfficeOrder {
    /// The member's key, as serde writes it and every binding reads it
    /// back: `GLOSS`.
    ///
    /// ```
    /// use teistro_tajika::OfficeOrder;
    ///
    /// assert_eq!(OfficeOrder::default().key(), "GLOSS");
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            OfficeOrder::Gloss => "GLOSS",
            OfficeOrder::Highest => "HIGHEST",
            OfficeOrder::Lowest => "LOWEST",
        }
    }
}

/// Which drekkāṇa a graha's own is read in.
///
/// ```
/// # use teistro_core::catalogue::Graha;
/// # use teistro_tajika::{AnnualSky, OfficeRules, offices};
/// #
/// # let sky = AnnualSky {
/// #     sun_deg: 15.0, moon_deg: 102.0, mars_deg: 105.0, mercury_deg: 195.0,
/// #     jupiter_deg: 15.0, venus_deg: 160.0, saturn_deg: 15.0,
/// # };
///
/// use teistro_tajika::{DrekkanaReading, OfficeClause};
///
/// let early = AnnualSky { moon_deg: 91.0, ..sky };
/// // Cancer's first drekkana is Venus's by the weekday cycle, the Moon's by Parashara.
/// let read = |drekkana| offices(Graha::Moon, &early, OfficeRules { drekkana, ..OfficeRules::default() });
/// assert!(!read(DrekkanaReading::Tajika)?.held.contains(&OfficeClause::OwnDrekkana));
/// assert!(read(DrekkanaReading::Parashari)?.held.contains(&OfficeClause::OwnDrekkana));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DrekkanaReading {
    /// The Tajika one, the weekday cycle from Mars ([`drekkana_lord`]),
    /// which the 1893 print's Prashna Tantra gloss defines (n205).
    #[default]
    Tajika,
    /// The Parashari one: the sign, its fifth, its ninth. The Samjna
    /// Tantra gloss's "the Moon in Cancer, in the first drekkāṇa" reads
    /// it; its other examples do not need it.
    Parashari,
}

impl DrekkanaReading {
    /// The member's key, as serde writes it and every binding reads it
    /// back: `TAJIKA`.
    ///
    /// ```
    /// use teistro_tajika::DrekkanaReading;
    ///
    /// assert_eq!(DrekkanaReading::Parashari.key(), "PARASHARI");
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            DrekkanaReading::Tajika => "TAJIKA",
            DrekkanaReading::Parashari => "PARASHARI",
        }
    }

    /// The lord of the drekkāṇa a longitude falls in.
    fn lord(self, longitude_deg: f64) -> Graha {
        match self {
            DrekkanaReading::Tajika => drekkana_lord(longitude_deg),
            DrekkanaReading::Parashari => parashari_drekkana_lord(longitude_deg),
        }
    }
}

/// Whether an enemy's sign makes a graha adhama.
///
/// ```
/// # use teistro_core::catalogue::Graha;
/// # use teistro_tajika::{AnnualSky, OfficeRules, offices};
/// #
/// # let sky = AnnualSky {
/// #     sun_deg: 15.0, moon_deg: 102.0, mars_deg: 105.0, mercury_deg: 195.0,
/// #     jupiter_deg: 15.0, venus_deg: 160.0, saturn_deg: 15.0,
/// # };
///
/// use teistro_tajika::{EnemySign, OfficeClause};
///
/// let ignored = OfficeRules { enemy: EnemySign::Ignored, ..OfficeRules::default() };
/// for graha in teistro_tajika::SEVEN {
///     assert!(!offices(graha, &sky, ignored)?.held.contains(&OfficeClause::EnemySign));
/// }
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EnemySign {
    /// As the definition states it, the enemy read as Tajika reads one:
    /// the sign's lord in the 1st, 4th, 7th or 10th from the graha
    /// ([`Relation::between`]), as the Panchavargiya bala and the empty
    /// road (vv. 55–56) read it. The gloss's example to v. 41 calls
    /// Mercury in Libra sama with Venus in the 4th from it.
    #[default]
    Tajika,
    /// Not read: no worked example needs it.
    Ignored,
}

impl EnemySign {
    /// The member's key, as serde writes it and every binding reads it
    /// back: `TAJIKA`.
    ///
    /// ```
    /// use teistro_tajika::EnemySign;
    ///
    /// assert_eq!(EnemySign::Ignored.key(), "IGNORED");
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            EnemySign::Tajika => "TAJIKA",
            EnemySign::Ignored => "IGNORED",
        }
    }
}

/// How the offices are read where the gloss leaves a choice.
///
/// ```
/// # use teistro_core::catalogue::Graha;
/// # use teistro_tajika::{AnnualSky, OfficeRules, offices};
/// #
/// # let sky = AnnualSky {
/// #     sun_deg: 15.0, moon_deg: 102.0, mars_deg: 105.0, mercury_deg: 195.0,
/// #     jupiter_deg: 15.0, venus_deg: 160.0, saturn_deg: 15.0,
/// # };
///
/// use teistro_tajika::{HuddaTable, OfficeClause};
///
/// // Venus at 10° Aquarius: in her own hudda by the gloss's table, not by Charak's.
/// let aquarius = AnnualSky { venus_deg: 310.0, ..sky };
/// let charak = OfficeRules { hudda: HuddaTable::Charak, ..OfficeRules::default() };
/// assert!(offices(Graha::Venus, &aquarius, OfficeRules::default())?.held.contains(&OfficeClause::OwnHudda));
/// assert!(!offices(Graha::Venus, &aquarius, charak)?.held.contains(&OfficeClause::OwnHudda));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct OfficeRules {
    /// Which office counts when a graha holds several.
    pub order: OfficeOrder,
    /// Which drekkāṇa is a graha's own.
    pub drekkana: DrekkanaReading,
    /// Whether an enemy's sign is read.
    pub enemy: EnemySign,
    /// Which printing of the hudda table. The gloss's example to v. 43
    /// calls Venus at Aquarius 10° officeless, which Charak's Aquarius
    /// gives and the gloss's own table (n205) does not (crux C109).
    pub hudda: HuddaTable,
}

/// The offices one graha holds.
///
/// ```
/// # use teistro_core::catalogue::Graha;
/// # use teistro_tajika::{AnnualSky, OfficeRules, offices};
/// #
/// # let sky = AnnualSky {
/// #     sun_deg: 15.0, moon_deg: 102.0, mars_deg: 105.0, mercury_deg: 195.0,
/// #     jupiter_deg: 15.0, venus_deg: 160.0, saturn_deg: 15.0,
/// # };
///
/// use teistro_tajika::{Adhikara, OfficeClause};
///
/// let early = AnnualSky { moon_deg: 91.0, ..sky };
/// let moon = offices(Graha::Moon, &early, OfficeRules::default())?;
/// assert_eq!(moon.graha, Graha::Moon);
/// assert_eq!(&moon.held[..1], [OfficeClause::OwnSign]);
/// assert!(moon.held.contains(&OfficeClause::OwnNavamsha));
/// assert_eq!(moon.adhikara, Adhikara::Madhyama);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "TajikaOffices"))]
#[serde(rename_all = "camelCase")]
pub struct Offices {
    /// Whose.
    pub graha: Graha,
    /// Every clause of the definition it meets, in the definition's
    /// order.
    pub held: Vec<OfficeClause>,
    /// The office that counts, by [`OfficeOrder`].
    pub adhikara: Adhikara,
}

/// A Kamboola's grade: the Moon's office and the pair's.
///
/// ```
/// # use teistro_core::catalogue::Graha;
/// # use teistro_tajika::{AnnualSky, OfficeRules, offices};
/// #
/// # let sky = AnnualSky {
/// #     sun_deg: 15.0, moon_deg: 102.0, mars_deg: 105.0, mercury_deg: 195.0,
/// #     jupiter_deg: 15.0, venus_deg: 160.0, saturn_deg: 15.0,
/// # };
///
/// use teistro_tajika::{Adhikara, kamboola_grade};
///
/// let grade = kamboola_grade(Graha::Venus, Graha::Mars, &sky, OfficeRules::default())?;
/// assert_eq!((grade.lagnesha.graha, grade.karyesha.graha), (Graha::Venus, Graha::Mars));
/// assert_eq!(grade.pair, Some(Adhikara::Adhama));
/// assert_eq!(grade.moon.adhikara, Adhikara::Uttama);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct KamboolaGrade {
    /// The readings it was graded under.
    pub rules: OfficeRules,
    /// The Moon's offices.
    pub moon: Offices,
    /// The lagnesha's.
    pub lagnesha: Offices,
    /// The karyesha's.
    pub karyesha: Offices,
    /// The pair's office, when the two lords hold the same one; `None`
    /// when they differ, which no printed example grades (crux C372).
    pub pair: Option<Adhikara>,
}

impl KamboolaGrade {
    /// The cell the gloss names, the Moon's office first and the pair's
    /// second (`[Uttama, Madhyama]` is *uttama-madhyama*), when the pair
    /// has one.
    ///
    /// ```
    /// # use teistro_core::catalogue::Graha;
    /// # use teistro_tajika::{AnnualSky, OfficeRules, offices};
    /// #
    /// # let sky = AnnualSky {
    /// #     sun_deg: 15.0, moon_deg: 102.0, mars_deg: 105.0, mercury_deg: 195.0,
    /// #     jupiter_deg: 15.0, venus_deg: 160.0, saturn_deg: 15.0,
    /// # };
    ///
    /// use teistro_tajika::{Adhikara, kamboola_grade};
    ///
    /// // Uttama-adhama; with the Moon for a lord the two differ and no cell is named.
    /// let grade = kamboola_grade(Graha::Venus, Graha::Mars, &sky, OfficeRules::default())?;
    /// assert_eq!(grade.cell(), Some([Adhikara::Uttama, Adhikara::Adhama]));
    /// assert_eq!(kamboola_grade(Graha::Moon, Graha::Mars, &sky, OfficeRules::default())?.cell(), None);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn cell(&self) -> Option<[Adhikara; 2]> {
        self.pair.map(|pair| [self.moon.adhikara, pair])
    }
}

/// The offices `graha`, one of the seven, holds in `sky`.
///
/// ```
/// # use teistro_core::catalogue::Graha;
/// # use teistro_tajika::{AnnualSky, OfficeRules, offices};
/// #
/// # let sky = AnnualSky {
/// #     sun_deg: 15.0, moon_deg: 102.0, mars_deg: 105.0, mercury_deg: 195.0,
/// #     jupiter_deg: 15.0, venus_deg: 160.0, saturn_deg: 15.0,
/// # };
///
/// use teistro_tajika::Adhikara;
///
/// assert_eq!(offices(Graha::Venus, &sky, OfficeRules::default())?.adhikara, Adhikara::Adhama);
/// assert_eq!(offices(Graha::Rahu, &sky, OfficeRules::default()).unwrap_err().field(), Some("graha"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// A longitude that is not a number, named by its field, or a graha
/// outside the seven (`graha`).
pub fn offices(graha: Graha, sky: &AnnualSky, rules: OfficeRules) -> Result<Offices, Error> {
    sky.check()?;
    one_of_seven(graha, "graha")?;
    Ok(offices_in(graha, sky, rules))
}

/// The grade of a Kamboola between `lagnesha` and `karyesha`.
///
/// Whether the Kamboola holds is [`crate::year_yogas`]'s question; this
/// grades the three as they stand, which is also what a reader weighing a
/// Kamboola that will form wants.
///
/// ```
/// # use teistro_core::catalogue::Graha;
/// # use teistro_tajika::{AnnualSky, OfficeRules, offices};
/// #
/// # let sky = AnnualSky {
/// #     sun_deg: 15.0, moon_deg: 102.0, mars_deg: 105.0, mercury_deg: 195.0,
/// #     jupiter_deg: 15.0, venus_deg: 160.0, saturn_deg: 15.0,
/// # };
///
/// use teistro_tajika::kamboola_grade;
///
/// let refused = kamboola_grade(Graha::Venus, Graha::Ketu, &sky, OfficeRules::default()).unwrap_err();
/// assert_eq!(refused.field(), Some("karyesha"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// A longitude that is not a number, named by its field, or a lord
/// outside the seven (`lagnesha`, `karyesha`).
pub fn kamboola_grade(
    lagnesha: Graha,
    karyesha: Graha,
    sky: &AnnualSky,
    rules: OfficeRules,
) -> Result<KamboolaGrade, Error> {
    sky.check()?;
    one_of_seven(lagnesha, "lagnesha")?;
    one_of_seven(karyesha, "karyesha")?;
    let lagnesha = offices_in(lagnesha, sky, rules);
    let karyesha = offices_in(karyesha, sky, rules);
    Ok(KamboolaGrade {
        rules,
        moon: offices_in(Graha::Moon, sky, rules),
        pair: (lagnesha.adhikara == karyesha.adhikara).then_some(lagnesha.adhikara),
        lagnesha,
        karyesha,
    })
}

/// Refuses a graha outside the seven, naming `field`.
fn one_of_seven(graha: Graha, field: &str) -> Result<(), Error> {
    if SEVEN.contains(&graha) {
        return Ok(());
    }
    Err(Error::invalid_arg(format!(
        "{graha:?} is not one of the seven that hold an office"
    ))
    .with_field(field.to_owned()))
}

/// The clauses of the definition `graha` meets, in its order, on a
/// checked sky.
///
/// The empty road (vv. 55–56) reads the same five dignities and two
/// faults, so it asks this under the default rules rather than listing
/// them again.
pub(crate) fn held(graha: Graha, sky: &AnnualSky, rules: OfficeRules) -> Vec<OfficeClause> {
    let longitude = sky.longitude_of(graha);
    let sign = sky.sign_of(graha);
    let lord = sign.attributes().lord;
    let attributes = graha.attributes();
    let in_sign = |fixed: Option<SignDegree>| fixed.is_some_and(|fixed| fixed.sign == sign);
    OfficeClause::ALL
        .into_iter()
        .filter(|clause| match clause {
            OfficeClause::OwnSign => lord == graha,
            OfficeClause::Exalted => in_sign(attributes.exaltation),
            OfficeClause::OwnHudda => hudda_lord_in(rules.hudda, longitude) == graha,
            OfficeClause::OwnDrekkana => rules.drekkana.lord(longitude) == graha,
            OfficeClause::OwnNavamsha => navamsha_lord(longitude) == graha,
            OfficeClause::Debilitated => in_sign(attributes.debilitation),
            OfficeClause::EnemySign => {
                rules.enemy == EnemySign::Tajika
                    && Relation::between(graha, lord, sky) == Relation::Enemy
            }
        })
        .collect()
}

/// [`offices`] on a checked sky, for one of the seven.
fn offices_in(graha: Graha, sky: &AnnualSky, rules: OfficeRules) -> Offices {
    let held = held(graha, sky, rules);
    Offices {
        graha,
        adhikara: counted(&held, rules.order),
        held,
    }
}

/// The office that counts among those `held` gives.
fn counted(held: &[OfficeClause], order: OfficeOrder) -> Adhikara {
    let offices = held.iter().map(|clause| clause.office());
    let found = match order {
        OfficeOrder::Gloss => {
            let first =
                |wanted: &[OfficeClause]| held.iter().find(|clause| wanted.contains(clause));
            first(&[OfficeClause::Debilitated])
                .or_else(|| {
                    first(&[
                        OfficeClause::OwnHudda,
                        OfficeClause::OwnDrekkana,
                        OfficeClause::OwnNavamsha,
                    ])
                })
                .or_else(|| first(&[OfficeClause::OwnSign, OfficeClause::Exalted]))
                .or_else(|| first(&[OfficeClause::EnemySign]))
                .map(|clause| clause.office())
        }
        // `Adhikara` is ordered best first, so the best is the least.
        OfficeOrder::Highest => offices.min(),
        OfficeOrder::Lowest => offices.max(),
    };
    found.unwrap_or(Adhikara::Sama)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests")]

    use super::*;
    use crate::bala::hudda_lord;

    use Adhikara::{Adhama, Madhyama, Sama, Uttama};

    /// One of the gloss's worked examples (Samjna Tantra vv. 38–45, leaves
    /// n52–n56): the pair, the sky, and the grade it prints.
    struct Example {
        verse: &'static str,
        lagnesha: Graha,
        karyesha: Graha,
        sky: AnnualSky,
        moon: Adhikara,
        pair: Adhikara,
    }

    /// The seven in the order Sun, Moon, Mars, Mercury, Jupiter, Venus,
    /// Saturn.
    const fn sky(at: [f64; 7]) -> AnnualSky {
        AnnualSky {
            sun_deg: at[0],
            moon_deg: at[1],
            mars_deg: at[2],
            mercury_deg: at[3],
            jupiter_deg: at[4],
            venus_deg: at[5],
            saturn_deg: at[6],
        }
    }

    /// The twelve examples the gloss grades with its places.
    ///
    /// Where the gloss gives a degree it is used; where it gives only a
    /// sign and the office, the degree is one where that office holds
    /// under the default readings and no other does. Every graha the
    /// gloss does not place stands where it is no sign lord's enemy, so
    /// the grade is the three's own. The example to v. 42b (Mars "in his
    /// own hudda" at Libra 10°, Mercury's in every table) is left out: its
    /// office is a slip, held by
    /// `a_printed_own_hudda_that_no_table_gives_is_a_slip`.
    const EXAMPLES: [Example; 12] = {
        use Graha::{Jupiter, Mars, Mercury, Saturn, Sun, Venus};
        [
            // Fortune, Libra rising: Venus Cancer 10° and Mercury Aries
            // 14°, each in its own hudda; the Moon Cancer 18°, own sign.
            ex(
                "38",
                Venus,
                Mercury,
                [15., 108., 45., 14., 15., 100., 15.],
                Uttama,
                Madhyama,
            ),
            // Kingdom, Gemini rising: Mercury and Jupiter in a neutral's
            // signs, the Moon exalted in Taurus.
            ex(
                "38b",
                Mercury,
                Jupiter,
                [285., 33., 15., 122., 175., 75., 15.],
                Uttama,
                Sama,
            ),
            // Wife, Libra rising: Venus Virgo 10° and Mars Cancer 15°,
            // debilitated; the Moon Cancer 12°.
            ex(
                "39",
                Venus,
                Mars,
                [15., 102., 105., 195., 15., 160., 15.],
                Uttama,
                Adhama,
            ),
            // Wife, Libra rising: Venus Libra 24°, Mars Aries 28°, each in
            // its own sign; the Moon Capricorn 22°, her own navāṁśa.
            ex(
                "39b",
                Venus,
                Mars,
                [15., 292., 28., 15., 15., 204., 315.],
                Madhyama,
                Uttama,
            ),
            // Progeny, Gemini rising: Mercury Libra 8° and Venus Gemini
            // 8°, each in its own hudda; the Moon in Cancer's first
            // drekkāṇa, her own navāṁśa.
            ex(
                "40",
                Mercury,
                Venus,
                [15., 92., 15., 188., 15., 68., 15.],
                Madhyama,
                Madhyama,
            ),
            // Progeny, Taurus rising: Venus Capricorn 4° and Mercury Libra
            // 5°, in a neutral's hudda; the Moon Cancer 3°.
            ex(
                "41a",
                Venus,
                Mercury,
                [15., 93., 15., 185., 15., 274., 315.],
                Madhyama,
                Sama,
            ),
            // Fortune, Aries rising: Mars in Cancer and Jupiter in
            // Capricorn, debilitated; the Moon in Cancer, her own
            // drekkāṇa.
            ex(
                "41b",
                Mars,
                Jupiter,
                [15., 115., 100., 15., 285., 15., 315.],
                Madhyama,
                Adhama,
            ),
            // Wealth, Libra rising: Venus in Libra, Mars exalted in
            // Capricorn; the Moon in Gemini, a neutral's drekkāṇa.
            ex(
                "42a",
                Venus,
                Mars,
                [15., 75., 295., 45., 15., 195., 315.],
                Sama,
                Uttama,
            ),
            // Wealth, Aries rising: Mars Leo 10°, Venus Aquarius 10°, the
            // Moon Libra 10°, "all of them sama" (the glossator's own).
            ex(
                "43a",
                Mars,
                Venus,
                [165., 190., 130., 15., 15., 310., 345.],
                Sama,
                Sama,
            ),
            // Son, Gemini rising: Mercury in Pisces and Venus in Virgo,
            // debilitated; the Moon without office.
            ex(
                "43b",
                Mercury,
                Venus,
                [15., 245., 15., 345., 285., 177., 15.],
                Sama,
                Adhama,
            ),
            // Happiness, Leo rising: the Sun exalted in Aries, Mars in
            // Capricorn; the Moon debilitated in Scorpio.
            ex(
                "44",
                Sun,
                Mars,
                [3., 213.5, 270.5, 15., 15., 15., 315.],
                Adhama,
                Uttama,
            ),
            // Son, Virgo rising: Mercury Capricorn 3°, own hudda; Saturn
            // Pisces 5°, "own hudda"; the Moon Scorpio 3°.
            ex(
                "45",
                Mercury,
                Saturn,
                [15., 213., 15., 273., 285., 15., 335.],
                Adhama,
                Madhyama,
            ),
        ]
    };

    const fn ex(
        verse: &'static str,
        lagnesha: Graha,
        karyesha: Graha,
        at: [f64; 7],
        moon: Adhikara,
        pair: Adhikara,
    ) -> Example {
        Example {
            verse,
            lagnesha,
            karyesha,
            sky: sky(at),
            moon,
            pair,
        }
    }

    /// The verses whose printed grade `rules` reproduce, and the others.
    fn reproduced(rules: OfficeRules) -> (Vec<&'static str>, Vec<&'static str>) {
        let (ok, not): (Vec<&Example>, Vec<&Example>) = EXAMPLES.iter().partition(|example| {
            let grade =
                kamboola_grade(example.lagnesha, example.karyesha, &example.sky, rules).unwrap();
            grade.cell() == Some([example.moon, example.pair])
        });
        let verses =
            |examples: Vec<&Example>| examples.iter().map(|example| example.verse).collect();
        (verses(ok), verses(not))
    }

    /// The default readings reproduce nine of the twelve, and each of the
    /// three they miss misses on a different question, named.
    #[test]
    fn the_default_reproduces_nine_printed_grades_and_names_the_three_it_misses() {
        let (_, missed) = reproduced(OfficeRules::default());
        // v. 39b: Venus at Libra 24° holds her own sign, hudda and
        // navāṁśa, and the gloss calls her uttama; vv. 40 and 41 call the
        // Moon in Cancer in her own navāṁśa madhyama. No one order gives
        // both (C371).
        // v. 41a: Venus stands 4th from Mercury in Libra, a Tajika enemy,
        // and the gloss calls Mercury sama.
        // v. 43a: Venus at Aquarius 10° is in her own hudda by the gloss's
        // own table, and the gloss calls her sama (C109).
        assert_eq!(missed, ["39b", "41a", "43a"]);
    }

    /// Each reading the defaults chose is the one the examples bear out
    /// against its rival, counted.
    #[test]
    fn each_default_is_the_reading_the_examples_bear_out() {
        let count = |rules: OfficeRules| reproduced(rules).0.len();
        let default = OfficeRules::default();
        assert_eq!(count(default), 9);
        let order = |order| count(OfficeRules { order, ..default });
        assert_eq!(order(OfficeOrder::Highest), 6);
        assert_eq!(order(OfficeOrder::Lowest), 7);
        let drekkana = DrekkanaReading::Parashari;
        assert_eq!(
            count(OfficeRules {
                drekkana,
                ..default
            }),
            7
        );
        // The two the defaults keep against the examples: an enemy's sign,
        // which the definition states, and the gloss's own hudda table.
        // Each costs one example, and the cost is recorded here.
        let enemy = EnemySign::Ignored;
        assert_eq!(count(OfficeRules { enemy, ..default }), 10);
        let hudda = HuddaTable::Charak;
        assert_eq!(count(OfficeRules { hudda, ..default }), 10);
        assert_eq!(
            count(OfficeRules {
                enemy,
                hudda,
                ..default
            }),
            11
        );
    }

    /// The examples to v. 42b and v. 45 print an own hudda no table gives,
    /// as `a-table-checks-itself` exposes a slip; v. 45's Saturn is
    /// madhyama all the same, by his own Tajika drekkāṇa.
    #[test]
    fn a_printed_own_hudda_that_no_table_gives_is_a_slip() {
        for table in [HuddaTable::Nilakanthi, HuddaTable::Charak] {
            // Mars at Libra 10°, Saturn at Pisces 5°.
            assert_eq!(hudda_lord_in(table, 190.0), Graha::Mercury);
            assert_eq!(hudda_lord_in(table, 335.0), Graha::Venus);
        }
        let v45 = &EXAMPLES[11];
        let saturn = offices(Graha::Saturn, &v45.sky, OfficeRules::default()).unwrap();
        assert_eq!(saturn.held, [OfficeClause::OwnDrekkana]);
        assert_eq!(saturn.adhikara, Madhyama);
    }

    /// The gloss's order puts a debilitation before a division, a division
    /// before the sign, and the sign before an enemy, each as an example
    /// prints it.
    #[test]
    fn the_gloss_order_is_each_examples_precedence() {
        let order = |held: &[OfficeClause]| counted(held, OfficeOrder::Gloss);
        // v. 39: Venus debilitated in her own hudda.
        assert_eq!(
            order(&[OfficeClause::OwnHudda, OfficeClause::Debilitated]),
            Adhama
        );
        // v. 40: the Moon in Cancer in her own navāṁśa.
        assert_eq!(
            order(&[OfficeClause::OwnSign, OfficeClause::OwnNavamsha]),
            Madhyama
        );
        // v. 38: Venus in her own hudda in the Moon's sign, her enemy.
        assert_eq!(
            order(&[OfficeClause::OwnHudda, OfficeClause::EnemySign]),
            Madhyama
        );
        // v. 44: the Sun exalted, with Mars 10th from him.
        assert_eq!(
            order(&[OfficeClause::Exalted, OfficeClause::EnemySign]),
            Uttama
        );
        assert_eq!(order(&[]), Sama);
        assert_eq!(
            counted(
                &[OfficeClause::OwnSign, OfficeClause::Debilitated],
                OfficeOrder::Highest
            ),
            Uttama
        );
        assert_eq!(
            counted(
                &[OfficeClause::OwnSign, OfficeClause::OwnHudda],
                OfficeOrder::Lowest
            ),
            Madhyama
        );
    }

    /// Two lords of different offices make no cell, and both offices are
    /// still reported.
    #[test]
    fn lords_of_different_offices_make_no_cell() {
        let v39b = &EXAMPLES[3];
        let grade = kamboola_grade(
            v39b.lagnesha,
            v39b.karyesha,
            &v39b.sky,
            OfficeRules::default(),
        )
        .unwrap();
        assert_eq!(grade.lagnesha.adhikara, Madhyama);
        assert_eq!(grade.karyesha.adhikara, Uttama);
        assert_eq!(grade.pair, None);
        assert_eq!(grade.cell(), None);
    }

    /// An enemy's sign is read only under [`EnemySign::Tajika`], and the
    /// table the offices read is the one the rules name.
    #[test]
    fn the_rules_reach_every_clause() {
        let v43a = &EXAMPLES[8];
        let venus = |rules| offices(Graha::Venus, &v43a.sky, rules).unwrap().held;
        assert_eq!(venus(OfficeRules::default()), [OfficeClause::OwnHudda]);
        assert_eq!(hudda_lord(310.0), Graha::Venus);
        let hudda = HuddaTable::Charak;
        assert_eq!(
            venus(OfficeRules {
                hudda,
                ..OfficeRules::default()
            }),
            []
        );
        let v41a = &EXAMPLES[5];
        let mercury = |enemy| {
            offices(
                Graha::Mercury,
                &v41a.sky,
                OfficeRules {
                    enemy,
                    ..OfficeRules::default()
                },
            )
            .unwrap()
            .held
        };
        assert_eq!(mercury(EnemySign::Tajika), [OfficeClause::EnemySign]);
        assert_eq!(mercury(EnemySign::Ignored), []);
    }

    /// A body outside the seven, and a sky that is not a number, are
    /// refused by their fields.
    #[test]
    fn a_node_or_an_unplaced_sky_is_refused_by_its_field() {
        let sky = EXAMPLES[0].sky;
        let rules = OfficeRules::default();
        let field =
            |result: Result<KamboolaGrade, Error>| result.unwrap_err().field().map(str::to_owned);
        assert_eq!(
            field(kamboola_grade(Graha::Rahu, Graha::Venus, &sky, rules)).as_deref(),
            Some("lagnesha")
        );
        assert_eq!(
            field(kamboola_grade(Graha::Venus, Graha::Ketu, &sky, rules)).as_deref(),
            Some("karyesha")
        );
        let unplaced = AnnualSky {
            venus_deg: f64::NAN,
            ..sky
        };
        assert_eq!(
            field(kamboola_grade(
                Graha::Venus,
                Graha::Mercury,
                &unplaced,
                rules
            ))
            .as_deref(),
            Some("venus_deg")
        );
        assert_eq!(
            offices(Graha::Rahu, &sky, rules).unwrap_err().field(),
            Some("graha")
        );
    }

    /// Every member's key is the one serde writes.
    #[test]
    fn every_key_is_serdes() {
        let serde = |value: serde_json::Value| value.as_str().unwrap().to_owned();
        for office in Adhikara::ALL {
            assert_eq!(serde(serde_json::to_value(office).unwrap()), office.key());
        }
        for clause in OfficeClause::ALL {
            assert_eq!(serde(serde_json::to_value(clause).unwrap()), clause.key());
        }
        for order in [
            OfficeOrder::Gloss,
            OfficeOrder::Highest,
            OfficeOrder::Lowest,
        ] {
            assert_eq!(serde(serde_json::to_value(order).unwrap()), order.key());
        }
        for reading in [DrekkanaReading::Tajika, DrekkanaReading::Parashari] {
            assert_eq!(serde(serde_json::to_value(reading).unwrap()), reading.key());
        }
        for enemy in [EnemySign::Tajika, EnemySign::Ignored] {
            assert_eq!(serde(serde_json::to_value(enemy).unwrap()), enemy.key());
        }
    }
}
