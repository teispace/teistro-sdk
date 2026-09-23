//! The **sixteen Tajika yogas** of the annual chart
//! (`03-design/tajika-yogas.md`).
//!
//! # They answer a question, they are not facts about a chart
//!
//! This is the thing that decides the module's shape, and it is not how
//! a Parashari yoga works. Gaja Kesari either holds in a chart or does
//! not. **Fourteen of these sixteen are judgements about a pair**:
//!
//! - the **lagnesha**, the lord of the annual lagna — fixed by the chart;
//! - the **karyesha**, the lord of the house *the matter asked about*
//!   belongs to, which the chart cannot know.
//!
//! So "what yogas does this year have?" is not a well-formed question.
//! "Is the marriage promised this year?" is: it fixes the karyesha as the
//! seventh lord, and the sixteen say whether the promise is fulfilled,
//! delayed, carried by someone else, or negated. [`year_yogas`]
//! therefore takes a [`House`] and answers for it.
//!
//! # What is built
//!
//! Four of the sixteen — the ones that need the pair's own aspects and
//! nothing else — and the other twelve say so rather than being
//! silently absent: [`YearYogas::unanswered`] lists them at every call,
//! because "Kamboola did not hold" and "this build cannot tell you about
//! Kamboola" are different statements and a consumer that cannot tell
//! them apart has been misled. [`YearYoga::awaiting`] carries the reason
//! for each, and `check-muntha` counts both sets from the type.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::error::Error;
use teistro_core::house::House;

use crate::bala::{AnnualSky, SEVEN, sign_of_longitude};
use crate::drishti::{
    Between, DrishtiRules, between_with_rules, between_within, deeptamsha, speed_rank,
};

/// One of the sixteen yogas of Tajika's own reckoning (K.S. Charak, *A
/// Textbook of Varshaphala*, Table X-3).
///
/// Named in the table's order. Only [`YearYoga::Ikabala`] and
/// [`YearYoga::Induvara`] are facts about a chart alone; the other
/// fourteen are judgements about the lagnesha and the karyesha.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum YearYoga {
    /// Every planet in a kendra or a panaphara.
    Ikabala,
    /// Every planet in an apoklima.
    Induvara,
    /// The pair are coming together, in one of the three kinds
    /// [`crate::Yoga`] enumerates.
    Ithasala,
    /// The pair are drawing apart.
    Ishrafa,
    /// The two do not aspect, and a planet **faster than both** carries
    /// the light between them: past one, coming to the other.
    Nakta,
    /// The two do not aspect, and a planet **slower than both** gathers
    /// their light: both are coming to it.
    Yamaya,
    /// An Ithasala a malefic destroys.
    Manau,
    /// An Ithasala the Moon joins.
    Kamboola,
    /// An Ithasala an unqualified Moon completes on entering the next
    /// sign.
    GairiKamboola,
    /// An Ithasala an unqualified Moon negates by standing apart from it.
    Khallasara,
    /// An Ithasala where either of the pair is afflicted.
    Rudda,
    /// An Ithasala where the slower is strong and the faster weak.
    DuhphaliKuttha,
    /// Both weak, and one in Ithasala with a third, strong planet.
    DutthotthaDavira,
    /// No aspect and no Ithasala, the karyesha completing one from the
    /// next sign.
    Tambira,
    /// Both powerful, well placed and under benefic influence.
    Kuttha,
    /// Both weak, in the trika houses, combust or retrograde.
    Durapha,
}

impl YearYoga {
    /// All sixteen, in the source's own table order.
    pub const ALL: [YearYoga; 16] = [
        YearYoga::Ikabala,
        YearYoga::Induvara,
        YearYoga::Ithasala,
        YearYoga::Ishrafa,
        YearYoga::Nakta,
        YearYoga::Yamaya,
        YearYoga::Manau,
        YearYoga::Kamboola,
        YearYoga::GairiKamboola,
        YearYoga::Khallasara,
        YearYoga::Rudda,
        YearYoga::DuhphaliKuttha,
        YearYoga::DutthotthaDavira,
        YearYoga::Tambira,
        YearYoga::Kuttha,
        YearYoga::Durapha,
    ];

    /// What this build still needs before it can answer for this yoga;
    /// nothing when it answers today.
    ///
    /// Matched exhaustively on purpose: a yoga added to [`YearYoga::ALL`]
    /// cannot compile until somebody has said which side of this it
    /// falls on, and `check-muntha` prints both sets with these reasons.
    #[must_use]
    pub const fn awaiting(self) -> Option<&'static str> {
        match self {
            YearYoga::Ithasala | YearYoga::Ishrafa | YearYoga::Nakta | YearYoga::Yamaya => None,
            YearYoga::Ikabala | YearYoga::Induvara => Some(
                "the whole-sign houses of the annual lagna, which the pair's own reckoning does not need",
            ),
            YearYoga::Manau | YearYoga::Kamboola | YearYoga::Khallasara => {
                Some("a third planet's own aspects on the pair, and Tajika's malefics")
            }
            YearYoga::GairiKamboola => Some(
                "an unqualified Moon, and where it will stand in the next sign: the only one of the sixteen that asks what happens next",
            ),
            YearYoga::Tambira => {
                Some("the karyesha at a sign's end, completing an Ithasala from the next")
            }
            YearYoga::Rudda
            | YearYoga::DuhphaliKuttha
            | YearYoga::DutthotthaDavira
            | YearYoga::Kuttha
            | YearYoga::Durapha => Some(
                "strong and weak, which the source floors only for the year lord (crux), and the chart's own dignities",
            ),
        }
    }

    /// Whether this build answers for it.
    #[must_use]
    pub const fn is_built(self) -> bool {
        self.awaiting().is_none()
    }
}

/// One of the sixteen, found holding, with what made it hold.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Held {
    /// Which of the sixteen.
    pub yoga: YearYoga,
    /// The pair's own relation, where that is what made it: an Ithasala
    /// or an Ishrafa between the lagnesha and the karyesha.
    pub between: Option<Between>,
    /// The third planet that carried or gathered the light, where one
    /// did: Nakta's faster intermediary or Yamaya's slower one.
    pub through: Option<Graha>,
    /// How the intermediary stands to the lagnesha, and to the karyesha.
    pub legs: Option<[Between; 2]>,
}

/// Every one of the sixteen this build can answer for, for one matter.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct YearYogas {
    /// The house the question was asked about, counted from the annual
    /// lagna by whole signs.
    pub house: House,
    /// The sign that house falls in.
    pub sign: Rashi,
    /// The **lagnesha**: the lord of the annual lagna.
    pub lagnesha: Graha,
    /// The **karyesha**: the lord of the house asked about.
    pub karyesha: Graha,
    /// Whether one planet is both.
    ///
    /// Always true of the **first** house, whose lord is the lagna's lord
    /// by definition — so a question about the native's own self can
    /// never be a judgement about a pair. True of one further house
    /// under a lagna ruled by one of the five that rule two signs, and of
    /// no other under Cancer or Leo.
    ///
    /// A planet makes no yoga with itself, so the fourteen pair yogas
    /// have nothing to judge when this is true. Whether the tradition
    /// reads that as the matter being promised outright is a question no
    /// text in reach answers, so it is reported and not decided.
    pub same_lord: bool,
    /// How the two stand to each other, where they are two.
    pub between: Option<Between>,
    /// Every one of the sixteen that holds.
    pub held: Vec<Held>,
    /// The ones this build cannot yet answer for.
    ///
    /// Never empty today, and that is the point: an absent yoga in
    /// [`YearYogas::held`] means it did not hold **only** for the ones
    /// not listed here.
    pub unanswered: Vec<YearYoga>,
}

impl YearYogas {
    /// Whether a named yoga holds.
    ///
    /// `None` where this build cannot say, which is not the same answer
    /// as `Some(false)` and is why this returns an option rather than a
    /// bool.
    #[must_use]
    pub fn holds(&self, yoga: YearYoga) -> Option<bool> {
        if yoga.is_built() {
            Some(self.held.iter().any(|one| one.yoga == yoga))
        } else {
            None
        }
    }
}

/// The sixteen yogas of an annual chart, for one matter, under the
/// readings this module defaults to.
///
/// # Errors
///
/// An annual lagna that is not a number, named `annual_lagna_deg`; a sky
/// that does not place one of the seven.
pub fn year_yogas(
    annual_lagna_deg: f64,
    house: House,
    sky: &AnnualSky,
) -> Result<YearYogas, Error> {
    year_yogas_with_rules(annual_lagna_deg, house, sky, DrishtiRules::default())
}

/// The sixteen yogas for one matter, under stated readings.
///
/// # Errors
///
/// As [`year_yogas`].
pub fn year_yogas_with_rules(
    annual_lagna_deg: f64,
    house: House,
    sky: &AnnualSky,
    rules: DrishtiRules,
) -> Result<YearYogas, Error> {
    if !annual_lagna_deg.is_finite() {
        return Err(
            Error::invalid_arg("the annual lagna must be a number of degrees")
                .with_field(String::from("annual_lagna_deg")),
        );
    }
    let lagna = sign_of_longitude(annual_lagna_deg);
    let sign = house.sign_from(lagna);
    let lagnesha = lagna.attributes().lord;
    let karyesha = sign.attributes().lord;
    let same_lord = lagnesha == karyesha;
    let between = if same_lord {
        None
    } else {
        Some(between_with_rules(lagnesha, karyesha, sky, rules)?)
    };
    let mut held = Vec::new();
    if let Some(pair) = between {
        if let Some(yoga) = pair.yoga {
            held.push(Held {
                yoga: if yoga.is_ithasala() {
                    YearYoga::Ithasala
                } else {
                    YearYoga::Ishrafa
                },
                between: Some(pair),
                through: None,
                legs: None,
            });
        } else if !pair.drishti.is_aspect() {
            // Only a pair that does not aspect at all can be reached by a
            // third planet: the source asks for the light to be carried
            // where there is no aspect to carry it.
            held.extend(carried(lagnesha, karyesha, sky, rules)?);
        }
    }
    Ok(YearYogas {
        house,
        sign,
        lagnesha,
        karyesha,
        same_lord,
        between,
        held,
        unanswered: YearYoga::ALL
            .into_iter()
            .filter(|yoga| !yoga.is_built())
            .collect(),
    })
}

/// Nakta and Yamaya: the light carried or gathered by a third planet.
///
/// Both ask the same question of every other planet and differ only in
/// which way its speed must fall, so they are found in one pass rather
/// than two that would each walk the seven.
fn carried(
    lagnesha: Graha,
    karyesha: Graha,
    sky: &AnnualSky,
    rules: DrishtiRules,
) -> Result<Vec<Held>, Error> {
    let rank = |graha: Graha| speed_rank(graha).unwrap_or(usize::MAX);
    let (of_lagna, of_karya) = (rank(lagnesha), rank(karyesha));
    let mut found = Vec::new();
    for third in SEVEN {
        if third == lagnesha || third == karyesha {
            continue;
        }
        // The source measures the third planet's reach by **its own**
        // deeptamsha and not by the mean it would share with each.
        let Some(orb_deg) = deeptamsha(third) else {
            continue;
        };
        let here = rank(third);
        let (faster_than_both, slower_than_both) = (
            here < of_lagna && here < of_karya,
            here > of_lagna && here > of_karya,
        );
        if !faster_than_both && !slower_than_both {
            continue;
        }
        let to_lagna = between_within(third, lagnesha, sky, orb_deg, rules)?;
        let to_karya = between_within(third, karyesha, sky, orb_deg, rules)?;
        let (Some(first), Some(second)) = (to_lagna.yoga, to_karya.yoga) else {
            continue;
        };
        let yoga = if faster_than_both {
            // Translation: past one and coming to the other, so exactly
            // one of the two legs is an Ishrafa.
            if first.is_ithasala() == second.is_ithasala() {
                continue;
            }
            YearYoga::Nakta
        } else {
            // Collection: both of the pair are coming to it, so neither
            // leg may be an Ishrafa.
            if !first.is_ithasala() || !second.is_ithasala() {
                continue;
            }
            YearYoga::Yamaya
        };
        found.push(Held {
            yoga,
            between: None,
            through: Some(third),
            legs: Some([to_lagna, to_karya]),
        });
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index what they asked for"
    )]

    use super::{Held, YearYoga, YearYogas, year_yogas, year_yogas_with_rules};
    use crate::bala::AnnualSky;
    use crate::drishti::{DrishtiRules, SubDegree, Yoga};
    use teistro_core::catalogue::{Graha, Rashi};
    use teistro_core::house::House;

    /// The source's own worked annual chart (K.S. Charak, ch. III): Scorpio
    /// rising, the seven where its Table places them.
    fn worked() -> AnnualSky {
        let at = |sign: f64, deg: f64, min: f64| sign * 30.0 + deg + min / 60.0;
        AnnualSky {
            sun_deg: at(4.0, 3.0, 50.0),
            moon_deg: at(1.0, 9.0, 40.0),
            mars_deg: at(7.0, 7.0, 42.0),
            mercury_deg: at(4.0, 18.0, 20.0),
            jupiter_deg: at(8.0, 9.0, 38.0),
            venus_deg: at(4.0, 21.0, 45.0),
            saturn_deg: at(6.0, 17.0, 13.0),
        }
    }

    /// Scorpio 9°26′, the annual lagna of the source's worked year.
    const WORKED_LAGNA_DEG: f64 = 7.0 * 30.0 + 9.0 + 26.0 / 60.0;

    fn house(number: u8) -> House {
        House::try_new(number).unwrap()
    }

    fn asked(number: u8) -> YearYogas {
        year_yogas(WORKED_LAGNA_DEG, house(number), &worked()).unwrap()
    }

    /// The lagnesha is the lord of the annual lagna and nothing else; the
    /// karyesha is the lord of the house asked about, counted from it by
    /// whole signs.
    #[test]
    fn the_pair_is_the_lagna_lord_and_the_matter_lord() {
        // Scorpio rising: Mars is the lagnesha for every question.
        for number in 1..=12u8 {
            assert_eq!(asked(number).lagnesha, Graha::Mars);
        }
        // The seventh from Scorpio is Taurus, whose lord is Venus.
        let marriage = asked(7);
        assert_eq!(marriage.sign, Rashi::Taurus);
        assert_eq!(marriage.karyesha, Graha::Venus);
        assert_eq!(marriage.house.get(), 7);
        // The tenth from Scorpio is Leo, whose lord is the Sun.
        assert_eq!(asked(10).karyesha, Graha::Sun);
    }

    /// One planet is both lords for exactly one house of the twelve, and
    /// the pair yogas have nothing to judge there.
    #[test]
    fn one_planet_may_be_both_lords_and_that_is_reported() {
        // The first house is the lagna itself, so its lord is always the
        // lagnesha: a question about the self is never about a pair.
        let self_ = asked(1);
        assert_eq!(self_.karyesha, Graha::Mars);
        assert!(self_.same_lord);
        assert_eq!(self_.between, None, "a planet makes no yoga with itself");
        assert!(self_.held.is_empty());
        // Mars rules Scorpio and Aries too, and Aries is the sixth from
        // Scorpio -- so two of the twelve are like that under this lagna.
        let same = asked(6);
        assert_eq!(same.karyesha, Graha::Mars);
        assert!(same.same_lord);
        let both = (1..=12u8).filter(|number| asked(*number).same_lord).count();
        assert_eq!(both, 2, "the first, and the lagnesha's other sign");
        // Under a lagna one of the luminaries rules, only the first is:
        // the Sun rules Leo alone and the Moon Cancer alone.
        for lagna_deg in [4.0 * 30.0 + 10.0, 3.0 * 30.0 + 10.0] {
            let alone = (1..=12u8)
                .filter(|number| {
                    year_yogas(lagna_deg, house(*number), &worked())
                        .unwrap()
                        .same_lord
                })
                .count();
            assert_eq!(alone, 1, "only the first house");
        }
    }

    /// A pair that aspects and stands inside its orb makes the Ithasala or
    /// the Ishrafa, and the answer carries which kind.
    #[test]
    fn the_pair_makes_its_own_yoga_and_says_which_kind() {
        // The tenth: Mars and the Sun, the source's own worked pair. The
        // Sun is faster and behind, so they are coming together.
        let found = asked(10);
        assert_eq!(found.karyesha, Graha::Sun);
        assert_eq!(found.holds(YearYoga::Ithasala), Some(true));
        assert_eq!(found.holds(YearYoga::Ishrafa), Some(false));
        let held = &found.held[0];
        assert_eq!(held.yoga, YearYoga::Ithasala);
        assert_eq!(
            held.between.unwrap().yoga,
            Some(Yoga::IthasalaVartamana),
            "and by more than a degree"
        );
        assert_eq!(held.through, None, "no third planet carried it");
    }

    /// A yoga this build cannot answer for is **not** reported as absent:
    /// `holds` says nothing, and every one of the ten is listed.
    #[test]
    fn an_unbuilt_yoga_says_so_rather_than_reading_as_absent() {
        let found = asked(7);
        assert_eq!(found.holds(YearYoga::Kamboola), None);
        assert_eq!(found.unanswered.len(), 12);
        for yoga in &found.unanswered {
            assert!(yoga.awaiting().is_some(), "{yoga:?} carries its reason");
            assert_eq!(found.holds(*yoga), None);
        }
        // Six are built, ten are not, and the two sets are the sixteen.
        let built = YearYoga::ALL.into_iter().filter(|one| one.is_built());
        assert_eq!(built.count() + found.unanswered.len(), YearYoga::ALL.len());
    }

    /// Nakta: the two do not aspect, and a planet faster than both is past
    /// one and coming to the other, within **its own** deeptamsha.
    #[test]
    fn nakta_carries_the_light_between_two_that_do_not_aspect() {
        // Mars at Scorpio 10° and Jupiter at Sagittarius 20°: the second
        // house from the first, which is no aspect at all. The Moon at
        // Leo 14° is faster than both, aspects Scorpio (from the tenth)
        // and Sagittarius (from the ninth), is 4° past Mars within its
        // sign and 6° behind Jupiter -- both inside its own 12°.
        let sky = AnnualSky {
            mars_deg: 7.0 * 30.0 + 10.0,
            jupiter_deg: 8.0 * 30.0 + 20.0,
            moon_deg: 4.0 * 30.0 + 14.0,
            ..worked()
        };
        // Scorpio rising, the second house: Sagittarius, ruled by Jupiter.
        let found = year_yogas(WORKED_LAGNA_DEG, house(2), &sky).unwrap();
        assert_eq!(
            (found.lagnesha, found.karyesha),
            (Graha::Mars, Graha::Jupiter)
        );
        assert!(!found.between.unwrap().drishti.is_aspect(), "no aspect");
        assert_eq!(found.holds(YearYoga::Nakta), Some(true));
        let nakta: &Held = found
            .held
            .iter()
            .find(|one| one.yoga == YearYoga::Nakta)
            .unwrap();
        assert_eq!(nakta.through, Some(Graha::Moon));
        let legs = nakta.legs.unwrap();
        // Exactly one leg is an Ishrafa: past one, coming to the other.
        let coming = legs.iter().filter(|leg| leg.yoga.unwrap().is_ithasala());
        assert_eq!(coming.count(), 1);
    }

    /// Yamaya: the intermediary is slower than both and both come to it.
    #[test]
    fn yamaya_gathers_the_light_of_two_that_do_not_aspect() {
        // The Moon at Aries 10° and Venus at Taurus 12° do not aspect --
        // the second house again. Saturn at Cancer 16° is slower than
        // both, aspects Aries (from the fourth) and Taurus (from the
        // third), and stands 6° ahead of the Moon and 4° ahead of Venus
        // within their signs, so both are coming to it inside Saturn's
        // own 9°.
        let sky = AnnualSky {
            moon_deg: 10.0,
            venus_deg: 30.0 + 12.0,
            saturn_deg: 3.0 * 30.0 + 16.0,
            ..worked()
        };
        // Aries rising makes Mars the lagnesha, so ask under Cancer
        // rising instead: the Moon rules the lagna and Venus the fifth
        // (Scorpio is the fifth... ) -- use Taurus rising, where Venus is
        // the lagnesha and the Moon rules the third, Cancer.
        let taurus = 30.0 + 5.0;
        let found = year_yogas(taurus, house(3), &sky).unwrap();
        assert_eq!(
            (found.lagnesha, found.karyesha),
            (Graha::Venus, Graha::Moon)
        );
        assert!(!found.between.unwrap().drishti.is_aspect(), "no aspect");
        assert_eq!(found.holds(YearYoga::Yamaya), Some(true));
        let yamaya = found
            .held
            .iter()
            .find(|one| one.yoga == YearYoga::Yamaya)
            .unwrap();
        assert_eq!(yamaya.through, Some(Graha::Saturn));
        // Neither leg is an Ishrafa: both of the pair are coming to it.
        assert!(
            yamaya
                .legs
                .unwrap()
                .iter()
                .all(|leg| leg.yoga.unwrap().is_ithasala())
        );
    }

    /// A pair that aspects is never reached by a third planet: the source
    /// asks for the light to be carried only where there is no aspect.
    #[test]
    fn a_pair_that_aspects_is_never_carried() {
        for number in 1..=12u8 {
            let found = asked(number);
            let Some(pair) = found.between else { continue };
            if pair.drishti.is_aspect() {
                assert!(
                    found.held.iter().all(|one| one.through.is_none()),
                    "house {number} aspects and was carried anyway"
                );
            }
        }
    }

    /// A lagna that is not a number is refused by name, before anything is
    /// looked up.
    #[test]
    fn a_lagna_that_is_not_a_number_is_refused_by_name() {
        let refused = year_yogas(f64::NAN, house(1), &worked()).unwrap_err();
        assert_eq!(refused.field(), Some("annual_lagna_deg"));
    }

    /// The readings the source leaves open reach this module too: they
    /// decide the pair's own band exactly as they do a bare pair's.
    #[test]
    fn the_readings_reach_the_pair() {
        // Mars at Scorpio 10°30′ and the Sun at Leo 10°: the Sun is the
        // faster and half a degree past, which is the contested band.
        let sky = AnnualSky {
            mars_deg: 7.0 * 30.0 + 10.5,
            sun_deg: 4.0 * 30.0 + 11.0,
            ..worked()
        };
        let under = |sub_degree| {
            year_yogas_with_rules(
                WORKED_LAGNA_DEG,
                house(10),
                &sky,
                DrishtiRules { sub_degree },
            )
            .unwrap()
        };
        assert!(under(SubDegree::Poorna).between.unwrap().disputed());
        assert_eq!(
            under(SubDegree::Poorna).held[0].between.unwrap().yoga,
            Some(Yoga::IthasalaPoorna)
        );
        assert_eq!(
            under(SubDegree::Ishrafa).held[0].between.unwrap().yoga,
            Some(Yoga::Ishrafa)
        );
        assert!(under(SubDegree::None).held.is_empty());
    }
}
