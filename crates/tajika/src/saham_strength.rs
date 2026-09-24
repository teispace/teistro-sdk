//! A saham's **strength**, clause by clause (`03-design/tajika-saham-strength.md`).
//!
//! The source (K. S. Charak, *A Textbook of Varshaphala*, ch. XI) lists
//! what makes a saham strong and what makes it weak, then judges its worked
//! sahams in words and never by a score — and one saham can meet clauses on
//! both lists. So this reports every clause, each named and evaluated, and
//! the facts it was judged from, and gives **no verdict**.
//!
//! ```
//! use teistro_tajika::{AnnualSky, Saham, SahamSky, SahamStrengthRules, saham_strength};
//!
//! let at = |sign: f64, deg: f64| sign * 30.0 + deg;
//! // The source's Example Chart, its forty-first year, by day.
//! let sky = AnnualSky {
//!     sun_deg: at(4.0, 3.8), moon_deg: at(1.0, 9.7), mars_deg: at(7.0, 7.7),
//!     mercury_deg: at(4.0, 18.3), jupiter_deg: at(8.0, 9.6), venus_deg: at(4.0, 21.8),
//!     saturn_deg: at(6.0, 17.2),
//! };
//! let chart = SahamSky::new(sky, at(7.0, 9.4), at(4.0, 13.2), true);
//! let punya = &saham_strength(&chart, &[Saham::Punya], None, SahamStrengthRules::default())?[0];
//! // "Associated with its own lord as well as two benefics, Mercury and Venus."
//! assert!(punya.lord_conjoins && punya.with_benefic);
//! assert!(punya.strong().iter().any(|(_, holds)| *holds));
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Nature};
use teistro_core::error::Error;

use crate::SEVEN;
use crate::bala::{Bala, Relation, panchavargiya, sign_of_longitude};
use crate::drishti::Drishti;
use crate::harsha::{HarshaGrade, HarshaRules, harsha};
use crate::saham::{Saham, SahamPlace, SahamRules, SahamSky, sahams};
use crate::yoga::{BENEFICS, qualification};

/// Which planets a saham's clauses call benefic and malefic.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum SahamNatures {
    /// The chapter's own: the Moon, Mercury, Jupiter and Venus benefic,
    /// the Sun, Mars and Saturn malefic — its forty-seventh year calls the
    /// Sun "another malefic", its birth Punya is with "all the natural
    /// benefics (viz., the Moon, Mercury, Jupiter and Venus)" — the
    /// [`BENEFICS`] Kuttha reads, so the two cannot drift apart.
    #[default]
    Chapter,
    /// The catalogue's Parashari natures, under which Mercury is neither.
    Parashari,
}

impl SahamNatures {
    /// Whether `graha` is a benefic under this reading.
    #[must_use]
    pub fn is_benefic(self, graha: Graha) -> bool {
        match self {
            SahamNatures::Chapter => BENEFICS.contains(&graha),
            SahamNatures::Parashari => nature(graha) == Some(Nature::Benefic),
        }
    }

    /// Whether `graha` is a malefic under this reading.
    #[must_use]
    pub fn is_malefic(self, graha: Graha) -> bool {
        match self {
            SahamNatures::Chapter => matches!(graha, Graha::Sun | Graha::Mars | Graha::Saturn),
            SahamNatures::Parashari => nature(graha) == Some(Nature::Malefic),
        }
    }
}

fn nature(graha: Graha) -> Option<Nature> {
    graha.attributes().descriptors.map(|about| about.nature)
}

/// Whose friendship the "friend" and "inimical planet" clauses read.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Friendship {
    /// Tajika's positional friendship, the only one the source defines
    /// (ch. VI): friends at houses 3, 5, 9 and 11 from each other, enemies
    /// at 1, 4, 7 and 10 — so two planets **in one sign are enemies**, and
    /// a saham whose lord sits in its sign has no friend for company.
    #[default]
    Positional,
    /// The catalogue's natural friendships, which do not move with the
    /// chart.
    Natural,
}

impl Friendship {
    /// How `graha` stands to `lord` under this reading.
    fn relation(self, graha: Graha, lord: Graha, sky: &crate::AnnualSky) -> Relation {
        if graha == lord {
            return Relation::Own;
        }
        match self {
            Friendship::Positional => Relation::between(graha, lord, sky),
            Friendship::Natural => {
                let of = lord.attributes();
                if of.friends.contains(&graha) {
                    Relation::Friend
                } else if of.enemies.contains(&graha) {
                    Relation::Enemy
                } else {
                    Relation::Neutral
                }
            }
        }
    }
}

/// The readings a saham's strength is judged under.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
pub struct SahamStrengthRules {
    /// How the sahams themselves are placed.
    pub sahams: SahamRules,
    /// How the lord's Harsha bala is read.
    pub harsha: HarshaRules,
    /// Which planets are benefic and malefic.
    pub natures: SahamNatures,
    /// Whose friendship "friend" and "inimical" read.
    pub friendship: Friendship,
    /// The Vishwa bala below which the lord is weak: five units, the
    /// source's, by default.
    pub weak_below: Bala,
}

impl Default for SahamStrengthRules {
    fn default() -> SahamStrengthRules {
        SahamStrengthRules {
            sahams: SahamRules::default(),
            harsha: HarshaRules::default(),
            natures: SahamNatures::default(),
            friendship: Friendship::default(),
            weak_below: Bala::new(5, 0, 0),
        }
    }
}

/// One saham's strength: every clause of the source's two lists, and the
/// facts they were read from.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[expect(
    clippy::struct_excessive_bools,
    reason = "the independent clauses of two printed lists, any of which may hold at once; \
              collapsing them into a verdict would be a rule the source does not state. \
              `SahamStrength::strong` and `weak` walk them without naming fields."
)]
pub struct SahamStrength {
    /// Which saham.
    pub saham: Saham,
    /// Where it fell: its sign, that sign's lord — the saham's lord — and
    /// its house.
    pub place: SahamPlace,
    /// The aspect each of the seven casts on the saham's sign, in the
    /// catalogue's order; one in the saham's own sign casts
    /// `Drishti::Inimical`, and is also its company.
    pub aspects: [Drishti; 7],
    /// Whether each of the seven stands in the saham's sign: its company.
    pub company: [bool; 7],
    /// How each of the seven stands to the saham's lord, under the
    /// friendship read.
    pub relations: [Relation; 7],
    /// The lord's Panchavargiya Vishwa bala.
    pub lord_vishwa: Bala,
    /// The lord's Harsha bala.
    pub lord_harsha: HarshaGrade,
    /// Whether the saham's sign is Rahu's or Ketu's, when the chart
    /// placed the nodes; the source's forty-sixth year counts it against
    /// the saham, and its lists do not.
    pub in_node_axis: Option<bool>,

    /// Strong (a): the lord in its exaltation sign.
    pub lord_exalted: bool,
    /// Strong (a): the lord in a sign it owns.
    pub lord_own_sign: bool,
    /// Strong (a), "in the vargas": the lord in a Hudda it rules.
    pub lord_own_hudda: bool,
    /// Strong (a), "in the vargas": the lord in a Drekkana it rules.
    pub lord_own_drekkana: bool,
    /// Strong (a), "in the vargas": the lord in a Navamsha it rules.
    pub lord_own_navamsha: bool,
    /// Strong (a): the lord in a sign whose lord is its friend.
    pub lord_in_friends_sign: bool,
    /// Strong (b): a friend of the lord in the saham's sign.
    pub with_friend: bool,
    /// Strong (b): a natural benefic in the saham's sign.
    pub with_benefic: bool,
    /// Strong (b): the year lord in the saham's sign; never for a birth
    /// chart, which has none.
    pub with_year_lord: bool,
    /// Strong (c): the lord in the saham's sign.
    pub lord_conjoins: bool,
    /// Strong (c): the lord's sign aspecting the saham's.
    pub lord_aspects_saham: bool,
    /// Strong (c): the lord's sign aspecting the lagna's.
    pub lord_aspects_lagna: bool,
    /// Weak (a): the lord below the Vishwa floor.
    pub lord_weak_vishwa: bool,
    /// Weak (b): the lord with no Harsha bala at all.
    pub lord_lacks_harsha: bool,
    /// Weak (d): an enemy of the lord in the saham's sign.
    pub with_enemy: bool,
    /// Weak (d): a natural malefic in the saham's sign.
    pub with_malefic: bool,
}

/// A clause of the source's **strong** list (ch. XI, 1 A), in its order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum StrongClause {
    /// (a) Its lord is exalted.
    LordExalted,
    /// (a) Its lord is in its own sign.
    LordOwnSign,
    /// (a) "In the vargas": its lord is in its own Hudda.
    LordOwnHudda,
    /// (a) "In the vargas": its lord is in its own Drekkana.
    LordOwnDrekkana,
    /// (a) "In the vargas": its lord is in its own Navamsha.
    LordOwnNavamsha,
    /// (a) Its lord is in a sign belonging to its friend.
    LordInFriendsSign,
    /// (b) It is with a friend of its lord.
    WithFriend,
    /// (b) It is with a natural benefic.
    WithBenefic,
    /// (b) It is with the year lord.
    WithYearLord,
    /// (c) Its lord conjoins it.
    LordConjoins,
    /// (c) Its lord aspects it.
    LordAspectsSaham,
    /// (c) Its lord aspects the lagna.
    LordAspectsLagna,
}

impl StrongClause {
    /// Every clause, in the source's order.
    pub const ALL: [StrongClause; 12] = [
        StrongClause::LordExalted,
        StrongClause::LordOwnSign,
        StrongClause::LordOwnHudda,
        StrongClause::LordOwnDrekkana,
        StrongClause::LordOwnNavamsha,
        StrongClause::LordInFriendsSign,
        StrongClause::WithFriend,
        StrongClause::WithBenefic,
        StrongClause::WithYearLord,
        StrongClause::LordConjoins,
        StrongClause::LordAspectsSaham,
        StrongClause::LordAspectsLagna,
    ];

    /// The clause in words.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            StrongClause::LordExalted => "its lord is exalted",
            StrongClause::LordOwnSign => "its lord is in its own sign",
            StrongClause::LordOwnHudda => "its lord is in its own Hudda",
            StrongClause::LordOwnDrekkana => "its lord is in its own Drekkana",
            StrongClause::LordOwnNavamsha => "its lord is in its own Navamsha",
            StrongClause::LordInFriendsSign => "its lord is in a friend's sign",
            StrongClause::WithFriend => "it is with a friend of its lord",
            StrongClause::WithBenefic => "it is with a natural benefic",
            StrongClause::WithYearLord => "it is with the year lord",
            StrongClause::LordConjoins => "its lord conjoins it",
            StrongClause::LordAspectsSaham => "its lord aspects it",
            StrongClause::LordAspectsLagna => "its lord aspects the lagna",
        }
    }
}

/// A clause of the source's **weak** list (ch. XI, 2), in its order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum WeakClause {
    /// (a) Its lord is under the Panchavargiya floor.
    LordWeakVishwa,
    /// (b) Its lord has no Harsha bala.
    LordLacksHarsha,
    /// (c) Its lord neither aspects nor conjoins it.
    LordApart,
    /// (d) It is with an enemy of its lord.
    WithEnemy,
    /// (d) It is with a natural malefic.
    WithMalefic,
}

impl WeakClause {
    /// Every clause, in the source's order.
    pub const ALL: [WeakClause; 5] = [
        WeakClause::LordWeakVishwa,
        WeakClause::LordLacksHarsha,
        WeakClause::LordApart,
        WeakClause::WithEnemy,
        WeakClause::WithMalefic,
    ];

    /// The clause in words.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            WeakClause::LordWeakVishwa => "its lord is under the Panchavargiya floor",
            WeakClause::LordLacksHarsha => "its lord has no Harsha bala",
            WeakClause::LordApart => "its lord neither aspects nor conjoins it",
            WeakClause::WithEnemy => "it is with an enemy of its lord",
            WeakClause::WithMalefic => "it is with a natural malefic",
        }
    }
}

impl SahamStrength {
    /// Whether one clause of the strong list holds.
    #[must_use]
    pub const fn holds_strong(&self, clause: StrongClause) -> bool {
        match clause {
            StrongClause::LordExalted => self.lord_exalted,
            StrongClause::LordOwnSign => self.lord_own_sign,
            StrongClause::LordOwnHudda => self.lord_own_hudda,
            StrongClause::LordOwnDrekkana => self.lord_own_drekkana,
            StrongClause::LordOwnNavamsha => self.lord_own_navamsha,
            StrongClause::LordInFriendsSign => self.lord_in_friends_sign,
            StrongClause::WithFriend => self.with_friend,
            StrongClause::WithBenefic => self.with_benefic,
            StrongClause::WithYearLord => self.with_year_lord,
            StrongClause::LordConjoins => self.lord_conjoins,
            StrongClause::LordAspectsSaham => self.lord_aspects_saham,
            StrongClause::LordAspectsLagna => self.lord_aspects_lagna,
        }
    }

    /// Whether one clause of the weak list holds.
    #[must_use]
    pub const fn holds_weak(&self, clause: WeakClause) -> bool {
        match clause {
            WeakClause::LordWeakVishwa => self.lord_weak_vishwa,
            WeakClause::LordLacksHarsha => self.lord_lacks_harsha,
            WeakClause::LordApart => !self.lord_conjoins && !self.lord_aspects_saham,
            WeakClause::WithEnemy => self.with_enemy,
            WeakClause::WithMalefic => self.with_malefic,
        }
    }

    /// The source's strong list, in its order, each clause and whether it
    /// holds.
    #[must_use]
    pub fn strong(&self) -> [(StrongClause, bool); 12] {
        StrongClause::ALL.map(|clause| (clause, self.holds_strong(clause)))
    }

    /// The source's weak list, in its order, each clause and whether it
    /// holds.
    #[must_use]
    pub fn weak(&self) -> [(WeakClause, bool); 5] {
        WeakClause::ALL.map(|clause| (clause, self.holds_weak(clause)))
    }

    /// In the 6th, 8th or 12th, where the source says a saham "is
    /// handicapped, and generally gives adverse results".
    #[must_use]
    pub fn is_handicapped(&self) -> bool {
        matches!(self.place.house.get(), 6 | 8 | 12)
    }
}

/// The strength of each saham asked for, in the order asked, read from a
/// chart and, for an annual chart, its year lord.
///
/// `year_lord` is `None` for a birth chart, which has none; the year-lord
/// clause then never holds.
///
/// # Errors
///
/// As [`sahams`]; a year lord outside the seven, named `year_lord`; Rahu
/// that is not a number, named `rahu_deg`.
pub fn saham_strength(
    sky: &SahamSky,
    which: &[Saham],
    year_lord: Option<Graha>,
    rules: SahamStrengthRules,
) -> Result<Vec<SahamStrength>, Error> {
    if let Some(lord) = year_lord.filter(|lord| !SEVEN.contains(lord)) {
        return Err(
            Error::invalid_arg(format!("{lord:?} is not one of the seven a year lord is"))
                .with_field("year_lord"),
        );
    }
    sky.check()?;
    let read = sahams(sky, which, rules.sahams)?;
    let bala = panchavargiya(&sky.sky)?;
    let happy = harsha(&sky.sky, sky.lagna_deg, sky.by_day, rules.harsha)?;
    let context = Context {
        sky,
        year_lord,
        rules,
        bala,
        happy,
    };
    read.points
        .iter()
        .map(|point| context.one(point.saham, point.point))
        .collect()
}

/// What every saham of one chart is judged against, read once.
struct Context<'a> {
    sky: &'a SahamSky,
    year_lord: Option<Graha>,
    rules: SahamStrengthRules,
    bala: [crate::Panchavargiya; 7],
    happy: [crate::Harsha; 7],
}

impl Context<'_> {
    fn one(&self, saham: Saham, place: SahamPlace) -> Result<SahamStrength, Error> {
        let stars = &self.sky.sky;
        let lord = place.lord;
        let at = SEVEN
            .iter()
            .position(|one| *one == lord)
            .unwrap_or_default();
        let lord_sign = stars.sign_of(lord);
        let lagna = sign_of_longitude(self.sky.lagna_deg);
        let aspects = SEVEN.map(|one| Drishti::between_signs(stars.sign_of(one), place.sign));
        let company = SEVEN.map(|one| stars.sign_of(one) == place.sign);
        let relations = SEVEN.map(|one| self.rules.friendship.relation(one, lord, stars));
        let natures = self.rules.natures;
        let in_company = |test: &dyn Fn(usize, Graha) -> bool| {
            SEVEN
                .iter()
                .enumerate()
                .any(|(k, one)| company.get(k).copied().unwrap_or(false) && test(k, *one))
        };
        let related = |k: usize, want: Relation| relations.get(k) == Some(&want);
        let qualified = qualification(lord, stars)?;
        let attributes = lord.attributes();
        let lord_vishwa = self.bala.get(at).map(|one| one.vishwa).unwrap_or_default();
        let lord_harsha = self
            .happy
            .get(at)
            .map_or(HarshaGrade::Nirbala, |one| one.grade);
        let sign_lord = lord_sign.attributes().lord;
        Ok(SahamStrength {
            saham,
            place,
            aspects,
            company,
            relations,
            lord_vishwa,
            lord_harsha,
            in_node_axis: self.sky.rahu_deg.map(|rahu| {
                place.sign == sign_of_longitude(rahu)
                    || place.sign == sign_of_longitude(rahu + 180.0)
            }),
            lord_exalted: attributes.exaltation.is_some_and(|at| at.sign == lord_sign),
            lord_own_sign: attributes.own.contains(&lord_sign),
            lord_own_hudda: qualified.own_hudda,
            lord_own_drekkana: qualified.own_drekkana,
            lord_own_navamsha: qualified.own_navamsha,
            lord_in_friends_sign: sign_lord != lord
                && self.rules.friendship.relation(sign_lord, lord, stars) == Relation::Friend,
            with_friend: in_company(&|k, one| one != lord && related(k, Relation::Friend)),
            with_benefic: in_company(&|_, one| natures.is_benefic(one)),
            with_year_lord: self
                .year_lord
                .is_some_and(|year| in_company(&|_, one| one == year)),
            lord_conjoins: lord_sign == place.sign,
            lord_aspects_saham: Drishti::between_signs(lord_sign, place.sign).is_aspect(),
            lord_aspects_lagna: Drishti::between_signs(lord_sign, lagna).is_aspect(),
            lord_weak_vishwa: lord_vishwa < self.rules.weak_below,
            lord_lacks_harsha: lord_harsha == HarshaGrade::Nirbala,
            with_enemy: in_company(&|k, one| one != lord && related(k, Relation::Enemy)),
            with_malefic: in_company(&|_, one| natures.is_malefic(one)),
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index what they asked for"
    )]

    use super::{
        Friendship, SahamNatures, SahamStrengthRules, StrongClause, WeakClause, saham_strength,
    };
    use crate::{AnnualSky, Saham, SahamSky};
    use teistro_core::catalogue::Graha;

    fn at(sign: u8, deg: u8, min: u8) -> f64 {
        f64::from(sign) * 30.0 + f64::from(deg) + f64::from(min) / 60.0
    }

    /// The source's Example Chart, the forty-first year, by day, with its
    /// printed lagna and midheaven.
    fn x1() -> SahamSky {
        let sky = AnnualSky {
            sun_deg: at(4, 3, 50),
            moon_deg: at(1, 9, 40),
            mars_deg: at(7, 7, 42),
            mercury_deg: at(4, 18, 20),
            jupiter_deg: at(8, 9, 38),
            venus_deg: at(4, 21, 45),
            saturn_deg: at(6, 17, 13),
        };
        SahamSky::new(sky, at(7, 9, 26), at(4, 13, 10), true)
    }

    /// The forty-first year's Punya, Leo 15°16′ in the tenth: "associated
    /// with its own lord as well as two benefics, Mercury and Venus … the
    /// Sun, is very strong and happens to be the year lord".
    #[test]
    fn the_forty_first_years_punya_is_what_the_source_says() {
        let punya = &saham_strength(
            &x1(),
            &[Saham::Punya],
            Some(Graha::Sun),
            SahamStrengthRules::default(),
        )
        .unwrap()[0];
        assert_eq!(punya.place.house.get(), 10);
        assert_eq!(
            punya.company,
            [true, false, false, true, false, true, false]
        );
        assert!(punya.lord_conjoins);
        assert!(punya.lord_own_sign, "the Sun in Leo");
        assert!(punya.with_benefic, "Mercury and Venus");
        assert!(punya.with_year_lord, "the Sun is the year lord");
        assert!(!punya.lord_weak_vishwa, "the Sun's Vishwa bala is 14:20:15");
        assert!(!punya.is_handicapped());
        // Under the chapter's natures the Sun, in its company, is a
        // malefic; under Parashari ones too. The report says so and does
        // not weigh it.
        assert!(punya.with_malefic);
    }

    /// The forty-first year's Raja: "the lord of the Raja Saham, i.e.,
    /// Saturn, is exalted".
    #[test]
    fn the_forty_first_years_raja_has_its_lord_exalted() {
        let raja = &saham_strength(
            &x1(),
            &[Saham::Raja],
            Some(Graha::Sun),
            SahamStrengthRules::default(),
        )
        .unwrap()[0];
        assert_eq!(raja.place.lord, Graha::Saturn);
        assert!(raja.lord_exalted);
        assert!(raja.strong()[0].1);
    }

    /// The Matri saham: "the Moon, though exalted in the seventh house …
    /// receives no friendly aspect".
    #[test]
    fn the_matri_sahams_lord_is_exalted() {
        let matri = &saham_strength(
            &x1(),
            &[Saham::Matri],
            Some(Graha::Sun),
            SahamStrengthRules::default(),
        )
        .unwrap()[0];
        assert_eq!(matri.place.lord, Graha::Moon);
        assert!(matri.lord_exalted);
    }

    #[test]
    fn the_readings_change_what_they_name_and_nothing_else() {
        let asked = [Saham::Punya];
        let chapter = saham_strength(&x1(), &asked, None, SahamStrengthRules::default()).unwrap();
        let parashari = saham_strength(
            &x1(),
            &asked,
            None,
            SahamStrengthRules {
                natures: SahamNatures::Parashari,
                ..SahamStrengthRules::default()
            },
        )
        .unwrap();
        // Mercury is a benefic in the chapter and neither in the catalogue;
        // Venus keeps the company benefic either way.
        assert!(chapter[0].with_benefic && parashari[0].with_benefic);
        assert_eq!(chapter[0].place, parashari[0].place);
        // With the lord in the saham's sign, positional friendship makes
        // its companions enemies; natural friendship does not move with the
        // chart.
        assert!(chapter[0].with_enemy);
        let natural = saham_strength(
            &x1(),
            &asked,
            None,
            SahamStrengthRules {
                friendship: Friendship::Natural,
                ..SahamStrengthRules::default()
            },
        )
        .unwrap();
        // The Sun's natural enemies include Venus, its natural neutral is
        // Mercury: Venus in its company is an enemy either way.
        assert!(natural[0].with_enemy);
        assert!(!natural[0].with_friend);
        // No year lord, no year-lord clause.
        assert!(!chapter[0].with_year_lord);
    }

    #[test]
    fn the_lists_walk_every_clause_in_the_sources_order() {
        let punya = &saham_strength(&x1(), &[Saham::Punya], None, SahamStrengthRules::default())
            .unwrap()[0];
        let strong = punya.strong();
        assert_eq!(strong[9], (StrongClause::LordConjoins, true));
        assert_eq!(strong[9].0.name(), "its lord conjoins it");
        let weak = punya.weak();
        assert_eq!(weak[2], (WeakClause::LordApart, false));
    }

    #[test]
    fn the_nodes_are_read_only_when_given() {
        let asked = [Saham::Punya];
        let none = saham_strength(&x1(), &asked, None, SahamStrengthRules::default()).unwrap();
        assert_eq!(none[0].in_node_axis, None);
        // Rahu in Aquarius puts Ketu in Leo, the Punya's sign.
        let axis = x1().with_rahu(at(10, 5, 0));
        let read = saham_strength(&axis, &asked, None, SahamStrengthRules::default()).unwrap();
        assert_eq!(read[0].in_node_axis, Some(true));
        let off = x1().with_rahu(at(0, 5, 0));
        let read = saham_strength(&off, &asked, None, SahamStrengthRules::default()).unwrap();
        assert_eq!(read[0].in_node_axis, Some(false));
    }

    #[test]
    fn what_cannot_be_read_is_refused_by_name() {
        let asked = [Saham::Punya];
        let refused = saham_strength(
            &x1(),
            &asked,
            Some(Graha::Rahu),
            SahamStrengthRules::default(),
        )
        .unwrap_err();
        assert_eq!(refused.field(), Some("year_lord"));
        let refused = saham_strength(
            &x1().with_rahu(f64::NAN),
            &asked,
            None,
            SahamStrengthRules::default(),
        )
        .unwrap_err();
        assert_eq!(refused.field(), Some("rahu_deg"));
    }

    #[test]
    fn the_rules_read_camel_case_and_fill_the_rest() {
        let rules: SahamStrengthRules =
            serde_json::from_str(r#"{"natures":"parashari","weakBelow":14400}"#).unwrap();
        assert_eq!(rules.natures, SahamNatures::Parashari);
        assert_eq!(rules.weak_below.units(), 4);
        assert_eq!(rules.friendship, Friendship::Positional);
    }
}
